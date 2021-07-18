use derive_dynamic_node::{
    GraplStaticId,
    NodeDescription,
};
use rust_proto::graph_descriptions::*;

#[derive(NodeDescription, GraplStaticId)]
pub struct Asset {
    #[grapl(static_id, immutable)]
    asset_id: String,
    #[grapl(immutable)]
    hostname: String,
    #[grapl(immutable)]
    launch_time: u64,
    #[grapl(increment)]
    last_seen_time: u64,
}

impl IAssetNode for AssetNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
