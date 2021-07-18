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
pub struct FileWrite {
    writer_pid: u64,
    writer_process_name: Option<String>,
    path: String,
    hostname: String,
    timestamp: u64,
}

impl TryFrom<FileWrite> for GraphDescription {
    type Error = String;

    fn try_from(file_write: FileWrite) -> Result<Self, Self::Error> {
        let mut asset = AssetNode::new(AssetNode::static_strategy());
        asset
            .with_hostname(file_write.hostname.clone())
            .with_asset_id(file_write.hostname.clone());

        let mut writer = ProcessNode::new(ProcessNode::session_strategy());
        writer
            .with_process_name(file_write.writer_process_name.unwrap_or_default())
            .with_asset_id(file_write.hostname.clone())
            .with_process_id(file_write.writer_pid)
            .with_last_seen_timestamp(file_write.timestamp);

        let mut file = FileNode::new(FileNode::session_strategy());
        file.with_asset_id(file_write.hostname.clone())
            .with_last_seen_timestamp(file_write.timestamp)
            .with_file_path(file_write.path);

        let mut graph = GraphDescription::new();

        graph.add_edge(
            "wrote_files",
            writer.clone_node_key(),
            file.clone_node_key(),
        );

        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            writer.clone_node_key(),
        );

        graph.add_edge(
            "files_on_asset",
            asset.clone_node_key(),
            file.clone_node_key(),
        );

        graph.add_node(asset);
        graph.add_node(writer);
        graph.add_node(file);

        Ok(graph)
    }
}
