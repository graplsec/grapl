use endpoint_plugin::{
    AssetNode,
    FileNode,
    IAssetNode,
    IFileNode,
    IProcessNode,
    ProcessNode,
};
use rust_proto::graph_descriptions::*;
use sysmon::FileCreateEvent;

use crate::{
    generator::SysmonGeneratorError,
    models::{
        get_image_name,
        strip_file_zone_identifier,
        utc_to_epoch,
    },
};

/// Creates a subgrqph describing a `FileCreateEvent`
///
/// The subgraph generation for a `FileCreateEvent` includes the following:
/// * A creator `Process` node - denotes the process that created the file
/// * A subject `File` node - the file that is created as part of this event
pub fn generate_file_create_subgraph(
    file_create: &FileCreateEvent,
) -> Result<GraphDescription, SysmonGeneratorError> {
    let timestamp = utc_to_epoch(&file_create.event_data.creation_utc_time)?;
    let mut graph = GraphDescription::new();

    let mut asset = AssetNode::new(AssetNode::static_strategy());
    asset
        .with_asset_id(file_create.system.computer.computer.clone())
        .with_hostname(file_create.system.computer.computer.clone());

    let mut creator = ProcessNode::new(ProcessNode::session_strategy());
    creator
        .with_asset_id(file_create.system.computer.computer.clone())
        .with_process_id(file_create.event_data.process_id)
        .with_process_name(get_image_name(&file_create.event_data.image.clone()).unwrap())
        .with_last_seen_timestamp(timestamp);

    let mut file = FileNode::new(FileNode::session_strategy());
    file.with_asset_id(file_create.system.computer.computer.clone())
        .with_file_path(strip_file_zone_identifier(
            &file_create.event_data.target_filename,
        ))
        .with_created_timestamp(timestamp);

    graph.add_edge(
        "process_asset",
        creator.clone_node_key(),
        asset.clone_node_key(),
    );

    graph.add_edge(
        "created_files",
        creator.clone_node_key(),
        file.clone_node_key(),
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
