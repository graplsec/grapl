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
pub struct ProcessEvent {
    host_identifier: String,
    calendar_time: String,
    unix_time: u64,
    action: OSQueryAction,
    columns: ProcessEventColumns,
}

/// See https://osquery.io/schema/4.5.0/#processes
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct ProcessEventColumns {
    #[serde(deserialize_with = "from_str")]
    pid: u64,
    name: Option<String>,
    path: String,
    cmdline: String,
    #[serde(deserialize_with = "from_str")]
    parent: i64,
    #[serde(deserialize_with = "from_str")]
    time: i64,
}

impl From<ProcessEvent> for GraphDescription {
    #[tracing::instrument]
    fn from(process_event: ProcessEvent) -> Self {
        tracing::trace!(message = "Building Graph from ProcessEvent.");

        let mut graph = GraphDescription::new();

        // this field can be -1 in cases of error
        // https://osquery.io/schema/4.5.1/#processes
        let process_start_time = if process_event.columns.time == -1 {
            process_event.unix_time
        } else {
            process_event.columns.time as u64
        };

        let mut asset = AssetNode::new(AssetNode::static_strategy());
        asset
            .with_asset_id(process_event.host_identifier.clone())
            .with_hostname(process_event.host_identifier.clone());

        let mut child = ProcessNode::new(ProcessNode::session_strategy());
        child
            .with_asset_id(process_event.host_identifier.clone())
            .with_created_timestamp(process_start_time)
            .with_last_seen_timestamp(process_start_time)
            .with_process_name(process_event.columns.name.clone().unwrap_or("".to_string()))
            .with_process_id(process_event.columns.pid);

        if !process_event.columns.path.is_empty() {
            let mut child_exe = FileNode::new(FileNode::session_strategy());
            child_exe
                .with_asset_id(process_event.host_identifier.clone())
                .with_file_path(process_event.columns.path.clone())
                .with_last_seen_timestamp(process_start_time);

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

        // OSQuery can record -1 for ppid if a parent is not able to be determined
        // https://osquery.io/schema/4.5.1/#process_events
        if process_event.columns.parent >= 0 {
            let mut parent_process = ProcessNode::new(ProcessNode::session_strategy());
            parent_process
                .with_asset_id(process_event.host_identifier.clone())
                .with_process_id(process_event.columns.parent as u64)
                .with_last_seen_timestamp(process_start_time);

            graph.add_edge(
                "children",
                parent_process.clone_node_key(),
                child.clone_node_key(),
            );

            graph.add_edge(
                "asset_processes",
                asset.clone_node_key(),
                parent_process.clone_node_key(),
            );

            graph.add_node(parent_process);
        }

        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            child.clone_node_key(),
        );

        graph.add_node(child);
        graph.add_node(asset);

        graph
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::OSQueryEvent;

    #[test]
    fn parse_pack_grapl_processes_json() {
        let test_json = std::fs::read_to_string("sample_data/unit/pack_grapl_processes.json")
            .expect("unable to read test file.");

        let event: OSQueryEvent =
            serde_json::from_str(&test_json).expect("serde_json::from_str failed.");
        match event {
            OSQueryEvent::Process(_) => {}
            _ => panic!("expected OSQueryEvent::Process"),
        };
    }
}
