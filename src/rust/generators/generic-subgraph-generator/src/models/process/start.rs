use std::convert::TryFrom;

use endpoint_plugin::{
    AssetNode,
    FileNode,
    IAssetNode,
    IFileNode,
    IProcessNode,
    ProcessNode,
};
use rust_proto::graph_descriptions::*;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ProcessStart {
    process_id: u64,
    parent_process_id: u64,
    name: String,
    hostname: String,
    arguments: String,
    timestamp: u64,
    exe: Option<String>,
}

impl TryFrom<ProcessStart> for GraphDescription {
    type Error = String;

    fn try_from(process_start: ProcessStart) -> Result<Self, Self::Error> {
        let mut graph = GraphDescription::new();

        let mut asset = AssetNode::new(AssetNode::static_strategy());
        asset
            .with_asset_id(process_start.hostname.clone())
            .with_hostname(process_start.hostname.clone());

        let mut parent = ProcessNode::new(ProcessNode::session_strategy());
        parent
            .with_asset_id(process_start.hostname.clone())
            .with_process_id(process_start.parent_process_id)
            .with_last_seen_timestamp(process_start.timestamp);

        let mut child = ProcessNode::new(ProcessNode::session_strategy());
        child
            .with_asset_id(process_start.hostname.clone())
            .with_process_name(process_start.name)
            .with_process_id(process_start.process_id)
            .with_created_timestamp(process_start.timestamp);

        if let Some(exe_path) = process_start.exe {
            let mut child_exe = FileNode::new(FileNode::session_strategy());
            child_exe
                .with_asset_id(process_start.hostname)
                .with_last_seen_timestamp(process_start.timestamp)
                .with_file_path(exe_path);

            graph.add_edge(
                "bin_file",
                child.clone_node_key(),
                child_exe.clone_node_key(),
            );

            graph.add_edge(
                "files_on_asset",
                asset.clone_node_key(),
                child_exe.clone_node_key(),
            );
            graph.add_node(child_exe);
        }

        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            parent.clone_node_key(),
        );

        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            child.clone_node_key(),
        );

        graph.add_edge("children", parent.clone_node_key(), child.clone_node_key());

        graph.add_node(asset);
        graph.add_node(parent);
        graph.add_node(child);

        Ok(graph)
    }
}
