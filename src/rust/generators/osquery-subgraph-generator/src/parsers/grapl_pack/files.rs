use endpoint_plugin::{
    AssetNode,
    FileNode,
    IAssetNode,
    IFileNode,
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
pub struct FileEvent {
    host_identifier: String,
    calendar_time: String,
    unix_time: u64,
    action: OSQueryAction,
    columns: FileEventColumns,
}

/// See https://osquery.io/schema/4.5.0/#processes
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct FileEventColumns {
    target_path: String,
    action: OSQueryFileAction,
    inode: String,
    md5: String,
    sha1: String,
    sha256: String,
    size: String,
    #[serde(deserialize_with = "from_str")]
    time: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum OSQueryFileAction {
    Accessed,
    AttributesModified,
    Updated,
    Created,
    Deleted,
    MovedFrom,
    MovedTo,
    Opened,
}

impl From<FileEvent> for GraphDescription {
    #[tracing::instrument]
    fn from(file_event: FileEvent) -> Self {
        tracing::trace!(message = "Building Graph from FileEvent.");

        let mut graph = GraphDescription::new();

        let mut asset = AssetNode::new(AssetNode::static_strategy());
        asset
            .with_asset_id(file_event.host_identifier.clone())
            .with_hostname(file_event.host_identifier.clone());

        let mut subject_file = FileNode::new(FileNode::session_strategy());
        subject_file
            .with_asset_id(file_event.host_identifier.clone())
            .with_file_path(file_event.columns.target_path.clone())
            .with_last_seen_timestamp(file_event.columns.time);

        /*
           Technically this might not be 100% correct but the moved_to and moved_from events
           seem like they could easily be represented by using create/deletes.
        */
        match &file_event.columns.action {
            OSQueryFileAction::Created | OSQueryFileAction::MovedFrom => {
                subject_file.with_created_timestamp(file_event.columns.time)
            }
            OSQueryFileAction::Deleted | OSQueryFileAction::MovedTo => {
                subject_file.with_deleted_timestamp(file_event.columns.time)
            }
            _ => subject_file.with_last_seen_timestamp(file_event.columns.time),
        };

        graph.add_edge(
            "files_on_asset",
            asset.clone_node_key(),
            subject_file.clone_node_key(),
        );

        graph.add_node(asset);
        graph.add_node(subject_file);

        graph
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::OSQueryEvent;

    #[test]
    fn parse_pack_grapl_files_json() {
        let test_json = std::fs::read_to_string("sample_data/unit/pack_grapl_files.json")
            .expect("unable to read test file.");

        let event: OSQueryEvent =
            serde_json::from_str(&test_json).expect("serde_json::from_str failed.");
        match event {
            OSQueryEvent::File(_) => {}
            _ => panic!("expected OSQueryEvent::File"),
        };
    }
}
