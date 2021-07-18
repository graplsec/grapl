use derive_dynamic_node::{
    GraplSessionId,
    NodeDescription,
};
use rust_proto::graph_descriptions::*;

#[derive(NodeDescription, GraplSessionId)]
pub struct SpecialProcess {
    #[grapl(create_time, immutable)]
    pub create_time: u64,
    #[grapl(last_seen_time, immutable)]
    pub seen_at: u64,
    #[grapl(terminate_time, immutable)]
    pub terminate_time: u64,
    #[grapl(pseudo_key, immutable)]
    pub process_id: u64,
}

impl ISpecialProcessNode for SpecialProcessNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}

#[test]
fn test_session() {
    let mut special_proc = SpecialProcessNode::new(SpecialProcessNode::session_strategy());

    special_proc.with_create_time(0u64);
    special_proc.with_seen_at(1u64);
    special_proc.with_terminate_time(2u64);
    special_proc.with_process_id(3u64);
    assert_eq!(
        special_proc.get_process_id().unwrap(),
        ImmutableUintProp { prop: 3 }
    );

    let strategy = special_proc.get_dynamic_node().id_strategy[0]
        .strategy
        .as_ref()
        .unwrap();

    let strategy = match strategy {
        rust_proto::graph_descriptions::id_strategy::Strategy::Session(strategy) => strategy,
        _ => panic!("Expected session"),
    };
    assert_eq!(strategy.create_time, 0u64);
    assert_eq!(strategy.last_seen_time, 1u64);
    assert_eq!(strategy.terminate_time, 2u64);
}
