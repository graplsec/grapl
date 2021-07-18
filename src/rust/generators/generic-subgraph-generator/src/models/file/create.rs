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
pub struct FileCreate {
    creator_process_id: u64,
    creator_process_name: Option<String>,
    path: String,
    hostname: String,
    timestamp: u64,
}

impl TryFrom<FileCreate> for GraphDescription {
    type Error = String;

    fn try_from(file_create: FileCreate) -> Result<Self, Self::Error> {
        let mut asset = AssetNode::new(AssetNode::static_strategy());
        asset
            .with_hostname(file_create.hostname.clone())
            .with_asset_id(file_create.hostname.clone());

        let mut creator = ProcessNode::new(ProcessNode::session_strategy());
        creator
            .with_asset_id(file_create.hostname.clone())
            .with_process_name(file_create.creator_process_name.unwrap_or_default())
            .with_process_id(file_create.creator_process_id)
            .with_last_seen_timestamp(file_create.timestamp);

        let mut file = FileNode::new(FileNode::session_strategy());
        file.with_asset_id(file_create.hostname.clone())
            .with_created_timestamp(file_create.timestamp)
            .with_file_path(file_create.path);

        let mut graph = GraphDescription::new();

        graph.add_edge(
            "created_files",
            creator.clone_node_key(),
            file.clone_node_key(),
        );

        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            creator.clone_node_key(),
        );

        graph.add_edge(
            "files_on_asset",
            asset.clone_node_key(),
            file.clone_node_key(),
        );

        graph.add_node(asset);
        graph.add_node(creator);
        graph.add_node(file);

        Ok(graph)
    }
}
