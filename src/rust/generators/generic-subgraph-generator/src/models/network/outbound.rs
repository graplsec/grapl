use std::convert::TryFrom;

use endpoint_plugin::{
    AssetNode,
    IAssetNode,
    IIpAddressNode,
    IIpPortNode,
    INetworkConnectionNode,
    IProcessNode,
    IProcessOutboundConnectionNode,
    IpAddressNode,
    IpPortNode,
    NetworkConnectionNode,
    ProcessNode,
    ProcessOutboundConnectionNode,
};
use rust_proto::graph_descriptions::*;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ProcessOutboundConnectionLog {
    pid: u64,
    protocol: String,
    src_port: u32,
    dst_port: u32,
    src_hostname: String,
    src_ip_addr: String,
    dst_ip_addr: String,
    timestamp: u64,
}

impl TryFrom<ProcessOutboundConnectionLog> for GraphDescription {
    type Error = String;

    fn try_from(conn_log: ProcessOutboundConnectionLog) -> Result<Self, Self::Error> {
        let mut graph = GraphDescription::new();

        let mut asset = AssetNode::new(AssetNode::static_strategy());
        asset
            .with_asset_id(conn_log.src_hostname.clone())
            .with_hostname(conn_log.src_hostname.clone());

        // A process creates an outbound connection to dst_port
        let mut process = ProcessNode::new(ProcessNode::session_strategy());
        process
            .with_asset_id(conn_log.src_hostname.clone())
            .with_process_id(conn_log.pid)
            .with_last_seen_timestamp(conn_log.timestamp);

        let mut outbound =
            ProcessOutboundConnectionNode::new(ProcessOutboundConnectionNode::session_strategy());
        outbound
            .with_asset_id(conn_log.src_hostname.clone())
            .with_ip_address(conn_log.src_ip_addr.clone())
            .with_protocol(conn_log.protocol.clone())
            .with_port(conn_log.src_port)
            .with_created_timestamp(conn_log.timestamp);

        let mut src_ip = IpAddressNode::new(IpAddressNode::static_strategy());
        src_ip
            .with_ip_address(conn_log.src_ip_addr.clone())
            .with_last_seen_timestamp(conn_log.timestamp);

        let mut dst_ip = IpAddressNode::new(IpAddressNode::static_strategy());
        dst_ip
            .with_ip_address(conn_log.dst_ip_addr.clone())
            .with_last_seen_timestamp(conn_log.timestamp);

        let mut src_port = IpPortNode::new(IpPortNode::static_strategy());
        src_port
            .with_ip_address(conn_log.src_ip_addr.clone())
            .with_port(conn_log.src_port)
            .with_protocol(conn_log.protocol.clone());

        let mut dst_port = IpPortNode::new(IpPortNode::static_strategy());
        dst_port
            .with_ip_address(conn_log.dst_ip_addr.clone())
            .with_port(conn_log.dst_port)
            .with_protocol(conn_log.protocol.clone());

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
        graph.add_edge("asset_ip", asset.clone_node_key(), src_ip.clone_node_key());

        // A process spawns on an asset
        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            process.clone_node_key(),
        );

        // A process creates a connection
        graph.add_edge(
            "created_connections",
            process.clone_node_key(),
            outbound.clone_node_key(),
        );

        // The connection is over an IP + Port
        graph.add_edge(
            "connected_over",
            outbound.clone_node_key(),
            src_port.clone_node_key(),
        );

        // The connection is to a dst ip + port
        graph.add_edge(
            "connected_to",
            outbound.clone_node_key(),
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
        graph.add_node(outbound);
        graph.add_node(src_ip);
        graph.add_node(dst_ip);
        graph.add_node(src_port);
        graph.add_node(dst_port);
        graph.add_node(network_connection);

        Ok(graph)
    }
}
