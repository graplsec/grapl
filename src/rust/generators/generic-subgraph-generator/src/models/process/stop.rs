use std::convert::TryFrom;

use endpoint_plugin::{
    AssetNode,
    IAssetNode,
    IProcessNode,
    ProcessNode,
};
use rust_proto::graph_descriptions::*;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ProcessStop {
    process_id: u64,
    name: String,
    hostname: String,
    timestamp: u64,
}

impl TryFrom<ProcessStop> for GraphDescription {
    type Error = String;

    fn try_from(process_stop: ProcessStop) -> Result<Self, Self::Error> {
        let mut asset = AssetNode::new(AssetNode::static_strategy());
        asset
            .with_hostname(process_stop.hostname.clone())
            .with_asset_id(process_stop.hostname.clone());

        let mut terminated_process = ProcessNode::new(ProcessNode::session_strategy());
        terminated_process
            .with_process_name(process_stop.name)
            .with_asset_id(process_stop.hostname)
            .with_process_id(process_stop.process_id)
            .with_terminated_timestamp(process_stop.timestamp);

        let mut graph = GraphDescription::new();

        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            terminated_process.clone_node_key(),
        );

        graph.add_node(asset);
        graph.add_node(terminated_process);

        Ok(graph)
    }
}
