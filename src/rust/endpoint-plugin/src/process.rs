use derive_dynamic_node::{
    GraplSessionId,
    NodeDescription,
};
use rust_proto::graph_descriptions::*;

#[derive(NodeDescription, GraplSessionId)]
pub struct Process {
    #[grapl(pseudo_key, immutable)]
    asset_id: String,

    #[grapl(pseudo_key, immutable)]
    process_id: u64,

    #[grapl(immutable)]
    process_guid: String,

    #[grapl(create_time, immutable)]
    created_timestamp: u64,

    #[grapl(terminate_time, immutable)]
    terminated_timestamp: u64,

    #[grapl(last_seen_time, increment)]
    last_seen_timestamp: u64,

    #[grapl(immutable)]
    process_name: String,

    #[grapl(immutable)]
    process_command_line: String,

    #[grapl(immutable)]
    operating_system: String,
}

impl IProcessNode for ProcessNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
