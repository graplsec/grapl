use derive_dynamic_node::{
    GraplStaticId,
    NodeDescription,
};
use rust_proto::graph_descriptions::*;

#[derive(NodeDescription, GraplStaticId)]
pub struct IpPort {
    #[grapl(static_id, immutable)]
    ip_address: String,
    #[grapl(static_id, immutable)]
    port: u64,
    #[grapl(immutable)]
    protocol: String,
    #[grapl(decrement)]
    first_seen_timestamp: u64,
    #[grapl(increment)]
    last_seen_timestamp: u64,
}

impl IIpPortNode for IpPortNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
