use derive_dynamic_node::{
    GraplSessionId,
    NodeDescription,
};
use rust_proto::graph_descriptions::*;

#[derive(NodeDescription, GraplSessionId)]
pub struct IpConnection {
    #[grapl(pseudo_key, immutable)]
    src_ip_address: String,
    #[grapl(pseudo_key, immutable)]
    dst_ip_address: String,
    #[grapl(pseudo_key, immutable)]
    protocol: String,
    #[grapl(create_time, immutable)]
    created_timestamp: u64,
    #[grapl(terminate_time, immutable)]
    terminated_timestamp: u64,
    #[grapl(last_seen_time, increment)]
    last_seen_timestamp: u64,
}

impl IIpConnectionNode for IpConnectionNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
