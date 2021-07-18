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

use super::from_str;
use crate::parsers::OSQueryAction;

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ProcessFileInteractionEvent {
    host_identifier: String,
    calendar_time: String,
    unix_time: u64,
    action: OSQueryAction,
    columns: ProcessFileInteractionEventColumns,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct ProcessFileInteractionEventColumns {
    #[serde(deserialize_with = "from_str")]
    fd: u64,
    path: String,
    #[serde(deserialize_with = "from_str")]
    pid: u64,
}

impl From<ProcessFileInteractionEvent> for GraphDescription {
    #[tracing::instrument]
    fn from(process_file_event: ProcessFileInteractionEvent) -> Self {
        tracing::trace!(message = "Building Graph from ProcessFileInteractionEvent.");

        let mut graph = GraphDescription::new();

        let mut asset = AssetNode::new(AssetNode::static_strategy());
        asset
            .with_asset_id(process_file_event.host_identifier.clone())
            .with_hostname(process_file_event.host_identifier.clone());

        let mut process = ProcessNode::new(ProcessNode::session_strategy());
        process
            .with_asset_id(process_file_event.host_identifier.clone())
            .with_last_seen_timestamp(process_file_event.unix_time)
            .with_process_id(process_file_event.columns.pid);

        let mut file = FileNode::new(FileNode::session_strategy());
        file.with_asset_id(process_file_event.host_identifier.clone())
            .with_file_path(process_file_event.columns.path.clone());

        // TODO: maybe we should set deleted time and created time for the file here?
        match process_file_event.action {
            OSQueryAction::Added => {
                file.with_created_timestamp(process_file_event.unix_time);

                graph.add_edge(
                    "created_files",
                    process.clone_node_key(),
                    file.clone_node_key(),
                );
            }
            OSQueryAction::Removed => {
                file.with_deleted_timestamp(process_file_event.unix_time);

                graph.add_edge(
                    "deleted_files",
                    process.clone_node_key(),
                    file.clone_node_key(),
                );
            }
            _ => {
                file.with_last_seen_timestamp(process_file_event.unix_time);
            }
        };

        graph.add_edge(
            "files_on_asset",
            asset.clone_node_key(),
            file.clone_node_key(),
        );

        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            process.clone_node_key(),
        );

        graph.add_node(asset);
        graph.add_node(file);
        graph.add_node(process);

        graph
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::OSQueryEvent;

    #[test]
    fn parse_pack_grapl_process_files_json() {
        let test_json = std::fs::read_to_string("sample_data/unit/pack_grapl_process-files.json")
            .expect("unable to read test file.");

        let event: OSQueryEvent =
            serde_json::from_str(&test_json).expect("serde_json::from_str failed.");
        match event {
            OSQueryEvent::ProcessFileAction(_) => {}
            _ => panic!("expected OSQueryEvent::ProcessFileAction"),
        };
    }
}
