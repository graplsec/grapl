use derive_dynamic_node::{
    GraplSessionId,
    NodeDescription,
};
use rust_proto::graph_descriptions::*;

#[derive(NodeDescription, GraplSessionId)]
pub struct ProcessInboundConnection {
    #[grapl(pseudo_key, immutable)]
    asset_id: String,
    #[grapl(create_time, immutable)]
    created_timestamp: u64,
    #[grapl(terminate_time, immutable)]
    terminated_timestamp: u64,
    #[grapl(last_seen_time, increment)]
    last_seen_timestamp: u64,
    #[grapl(pseudo_key, immutable)]
    port: u64,
    #[grapl(immutable)]
    ip_address: String,
    #[grapl(immutable)]
    protocol: String,
}

impl IProcessInboundConnectionNode for ProcessInboundConnectionNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
