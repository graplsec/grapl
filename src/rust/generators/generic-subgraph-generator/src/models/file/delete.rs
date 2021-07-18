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
pub struct FileDelete {
    deleter_process_id: u64,
    deleter_process_name: Option<String>,
    path: String,
    hostname: String,
    timestamp: u64,
}

impl TryFrom<FileDelete> for GraphDescription {
    type Error = String;

    fn try_from(file_delete: FileDelete) -> Result<Self, Self::Error> {
        let mut asset = AssetNode::new(AssetNode::static_strategy());
        asset
            .with_hostname(file_delete.hostname.clone())
            .with_asset_id(file_delete.hostname.clone());

        let mut deleter = ProcessNode::new(ProcessNode::session_strategy());
        deleter
            .with_asset_id(file_delete.hostname.clone())
            .with_process_name(file_delete.deleter_process_name.unwrap_or_default())
            .with_process_id(file_delete.deleter_process_id)
            .with_last_seen_timestamp(file_delete.timestamp);

        let mut file = FileNode::new(FileNode::session_strategy());
        file.with_asset_id(file_delete.hostname)
            .with_deleted_timestamp(file_delete.timestamp)
            .with_file_path(file_delete.path);

        let mut graph = GraphDescription::new();

        graph.add_edge("deleted", deleter.clone_node_key(), file.clone_node_key());

        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            deleter.clone_node_key(),
        );

        graph.add_edge(
            "files_on_asset",
            asset.clone_node_key(),
            file.clone_node_key(),
        );

        graph.add_node(asset);
        graph.add_node(deleter);
        graph.add_node(file);

        Ok(graph)
    }
}
