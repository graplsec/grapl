use std::convert::TryFrom;

use endpoint_plugin::{
    AssetNode,
    IAssetNode,
    IIpAddressNode,
    IIpPortNode,
    INetworkConnectionNode,
    IProcessInboundConnectionNode,
    IProcessNode,
    IpAddressNode,
    IpPortNode,
    NetworkConnectionNode,
    ProcessInboundConnectionNode,
    ProcessNode,
};
use rust_proto::graph_descriptions::*;
use serde::{
    Deserialize,
    Serialize,
};

// In an inbound connection "src" is where the connection is coming from
#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ProcessInboundConnectionLog {
    /// The pid of the process receiving the connection
    pid: u64,
    src_ip_addr: String,
    src_port: u32,
    dst_port: u32,
    dst_hostname: String,
    dst_ip_addr: String,
    protocol: String,
    timestamp: u64,
}

impl TryFrom<ProcessInboundConnectionLog> for GraphDescription {
    type Error = String;

    fn try_from(conn_log: ProcessInboundConnectionLog) -> Result<Self, Self::Error> {
        let mut graph = GraphDescription::new();

        let mut asset = AssetNode::new(AssetNode::static_strategy());
        asset
            .with_asset_id(conn_log.dst_hostname.clone())
            .with_hostname(conn_log.dst_hostname.clone());

        // A process creates an outbound connection to dst_port
        let mut process = ProcessNode::new(ProcessNode::session_strategy());
        process
            .with_asset_id(conn_log.dst_hostname.clone())
            .with_process_id(conn_log.pid)
            .with_last_seen_timestamp(conn_log.timestamp);

        let mut inbound =
            ProcessInboundConnectionNode::new(ProcessInboundConnectionNode::session_strategy());
        inbound
            .with_asset_id(conn_log.dst_hostname.clone())
            .with_ip_address(conn_log.dst_ip_addr.clone())
            .with_protocol(conn_log.protocol.clone())
            .with_port(conn_log.dst_port)
            .with_created_timestamp(conn_log.timestamp);

        let mut dst_ip = IpAddressNode::new(IpAddressNode::static_strategy());
        dst_ip
            .with_ip_address(conn_log.dst_ip_addr.clone())
            .with_last_seen_timestamp(conn_log.timestamp);

        let mut src_ip = IpAddressNode::new(IpAddressNode::static_strategy());
        src_ip
            .with_ip_address(conn_log.src_ip_addr.clone())
            .with_last_seen_timestamp(conn_log.timestamp);

        let mut src_port = IpPortNode::new(IpPortNode::static_strategy());
        src_port
            .with_ip_address(conn_log.src_ip_addr.clone())
            .with_protocol(conn_log.protocol.clone())
            .with_port(conn_log.src_port);

        let mut dst_port = IpPortNode::new(IpPortNode::static_strategy());
        dst_port
            .with_ip_address(conn_log.dst_ip_addr.clone())
            .with_protocol(conn_log.protocol.clone())
            .with_port(conn_log.dst_port);

        let mut network_connection =
            NetworkConnectionNode::new(NetworkConnectionNode::session_strategy());
        network_connection
            .with_src_ip_address(conn_log.src_ip_addr)
            .with_src_port(conn_log.src_port)
            .with_dst_ip_address(conn_log.dst_ip_addr)
            .with_dst_port(conn_log.dst_port)
            .with_protocol(conn_log.protocol)
            .with_created_timestamp(conn_log.timestamp);

        // An asset is assigned an IP
        graph.add_edge("asset_ip", asset.clone_node_key(), dst_ip.clone_node_key());

        // A process spawns on an asset
        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            process.clone_node_key(),
        );

        // A process creates a connection
        graph.add_edge(
            "received_connection",
            process.clone_node_key(),
            inbound.clone_node_key(),
        );

        // The connection is over an IP + Port
        graph.add_edge(
            "bound_port",
            inbound.clone_node_key(),
            src_port.clone_node_key(),
        );

        // The connection is to a dst ip + port
        graph.add_edge(
            "connected_to",
            inbound.clone_node_key(),
            dst_port.clone_node_key(),
        );

        // There is a network connection between the src and dst ports
        graph.add_edge(
            "outbound_connection_to",
            src_port.clone_node_key(),
            network_connection.clone_node_key(),
        );

        graph.add_edge(
            "inbound_connection_to",
            network_connection.clone_node_key(),
            dst_port.clone_node_key(),
        );

        graph.add_node(asset);
        graph.add_node(process);
        graph.add_node(inbound);
        graph.add_node(dst_ip);
        graph.add_node(src_ip);
        graph.add_node(src_port);
        graph.add_node(dst_port);
        graph.add_node(network_connection);

        Ok(graph)
    }
}
