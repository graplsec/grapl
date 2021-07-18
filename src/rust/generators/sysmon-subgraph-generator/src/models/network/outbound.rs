use endpoint_plugin::{
    AssetNode,
    IAssetNode,
    IIpAddressNode,
    IIpConnectionNode,
    IIpPortNode,
    INetworkConnectionNode,
    IProcessNode,
    IProcessOutboundConnectionNode,
    IpAddressNode,
    IpConnectionNode,
    IpPortNode,
    NetworkConnectionNode,
    ProcessNode,
    ProcessOutboundConnectionNode,
};
use rust_proto::graph_descriptions::*;
use sysmon::NetworkEvent;

use crate::{
    generator::SysmonGeneratorError,
    models::utc_to_epoch,
};

/// Creates a subgraph describing an outbound `NetworkEvent`
///
/// Subgraph generation for an outbound `NetworkEvent` includes the following:
/// * An `Asset` node - indicating the asset in which the outbound `NetworkEvent` occurred
/// * A `Process` node - indicating the process which triggered the outbound `NetworkEvent`
/// * A subject `OutboundConnection` node - indicating the network connection triggered by the process
/// * Source and Destination IP Address and Port nodes
/// * IP connection and Network connection nodes
pub fn generate_outbound_connection_subgraph(
    conn_log: &NetworkEvent,
) -> Result<GraphDescription, SysmonGeneratorError> {
    let timestamp = utc_to_epoch(&conn_log.event_data.utc_time)?;

    let mut graph = GraphDescription::new();

    let mut asset = AssetNode::new(AssetNode::static_strategy());
    asset
        .with_asset_id(conn_log.system.computer.computer.clone())
        .with_hostname(conn_log.system.computer.computer.clone());

    // A process creates an outbound connection to dst_port
    let mut process = ProcessNode::new(ProcessNode::session_strategy());
    process
        .with_asset_id(conn_log.system.computer.computer.clone())
        .with_process_id(conn_log.event_data.process_id)
        .with_last_seen_timestamp(timestamp);

    let mut outbound =
        ProcessOutboundConnectionNode::new(ProcessOutboundConnectionNode::identity_strategy());
    outbound
        .with_asset_id(conn_log.system.computer.computer.clone())
        .with_hostname(conn_log.system.computer.computer.clone())
        .with_ip_address(conn_log.event_data.source_ip.clone())
        .with_protocol(conn_log.event_data.protocol.clone())
        .with_port(conn_log.event_data.source_port)
        .with_created_timestamp(timestamp);

    let mut src_ip = IpAddressNode::new(IpAddressNode::identity_strategy());
    src_ip
        .with_ip_address(conn_log.event_data.source_ip.clone())
        .with_last_seen_timestamp(timestamp);

    let mut dst_ip = IpAddressNode::new(IpAddressNode::identity_strategy());
    dst_ip
        .with_ip_address(conn_log.event_data.destination_ip.clone())
        .with_last_seen_timestamp(timestamp);

    let mut src_port = IpPortNode::new(IpPortNode::identity_strategy());
    src_port
        .with_ip_address(conn_log.event_data.source_ip.clone())
        .with_port(conn_log.event_data.source_port)
        .with_protocol(conn_log.event_data.protocol.clone());

    let mut dst_port = IpPortNode::new(IpPortNode::identity_strategy());
    dst_port
        .with_ip_address(conn_log.event_data.destination_ip.clone())
        .with_port(conn_log.event_data.destination_port)
        .with_protocol(conn_log.event_data.protocol.clone());

    let mut network_connection =
        NetworkConnectionNode::new(NetworkConnectionNode::identity_strategy());
    network_connection
        .with_src_ip_address(conn_log.event_data.source_ip.clone())
        .with_src_port(conn_log.event_data.source_port)
        .with_dst_ip_address(conn_log.event_data.destination_ip.clone())
        .with_dst_port(conn_log.event_data.destination_port)
        .with_protocol(conn_log.event_data.protocol.clone())
        .with_created_timestamp(timestamp);

    let mut ip_connection = IpConnectionNode::new(IpConnectionNode::identity_strategy());
    ip_connection
        .with_src_ip_address(conn_log.event_data.source_ip.clone())
        .with_dst_ip_address(conn_log.event_data.destination_ip.clone())
        .with_protocol(conn_log.event_data.protocol.clone())
        .with_created_timestamp(timestamp);

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

    // The outbound process connection is to a dst ip + port
    graph.add_edge(
        "connected_to",
        outbound.clone_node_key(),
        dst_port.clone_node_key(),
    );

    // There is also a connection between the two IP addresses

    graph.add_edge(
        "ip_connections",
        src_ip.clone_node_key(),
        ip_connection.clone_node_key(),
    );

    graph.add_edge(
        "ip_connections",
        dst_ip.clone_node_key(),
        ip_connection.clone_node_key(),
    );

    graph.add_edge(
        "network_connections",
        src_port.clone_node_key(),
        network_connection.clone_node_key(),
    );

    graph.add_edge(
        "network_connections",
        dst_port.clone_node_key(),
        network_connection.clone_node_key(),
    );

    graph.add_node(asset);
    graph.add_node(process);
    graph.add_node(outbound);
    graph.add_node(src_ip);
    graph.add_node(dst_ip);
    graph.add_node(src_port);
    graph.add_node(dst_port);
    graph.add_node(network_connection);
    graph.add_node(ip_connection);

    Ok(graph)
}
