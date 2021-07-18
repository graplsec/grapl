use derive_dynamic_node::{
    GraplSessionId,
    NodeDescription,
};
use rust_proto::graph_descriptions::*;

#[derive(NodeDescription, GraplSessionId)]
pub struct File {
    #[grapl(pseudo_key, immutable)]
    asset_id: String,
    #[grapl(pseudo_key, immutable)]
    file_path: String,
    #[grapl(create_time, immutable)]
    created_timestamp: u64,
    #[grapl(terminate_time, immutable)]
    deleted_timestamp: u64,
    #[grapl(last_seen_time, increment)]
    last_seen_timestamp: u64,
    #[grapl(immutable)]
    file_name: String,
    #[grapl(immutable)]
    file_extension: String,
    #[grapl(immutable)]
    file_mime_type: String,
    #[grapl(immutable)]
    file_description: String,
    #[grapl(immutable)]
    file_product: String,
    #[grapl(immutable)]
    file_company: String,
    #[grapl(immutable)]
    file_directory: String,
    #[grapl(immutable)]
    file_inode: u64,
}

impl IFileNode for FileNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
