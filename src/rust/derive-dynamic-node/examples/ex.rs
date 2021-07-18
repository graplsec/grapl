use derive_dynamic_node::{
    GraplSessionId,
    GraplStaticId,
    NodeDescription,
};
use log::info;
use rust_proto::graph_descriptions::*;
use serde_derive::Deserialize;

fn read_log() -> &'static [u8] {
    unimplemented!()
}

#[derive(NodeDescription, GraplSessionId)]
pub struct SpecialProcess {
    #[grapl(create_time, immutable)]
    pub create_time: u64,
    #[grapl(last_seen_time, increment)]
    pub seen_at: u64,
    #[grapl(terminate_time, immutable)]
    pub terminate_time: u64,
    #[grapl(pseudo_key, immutable)]
    pub process_id: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct InstanceDetails {
    #[serde(rename = "arn")]
    arn: String,
    #[serde(rename = "imageId")]
    image_id: String,
    #[serde(rename = "instanceId")]
    instance_id: String,
    #[serde(rename = "instanceState")]
    instance_state: String,
    #[serde(rename = "instanceType")]
    instance_type: String,
    #[serde(rename = "launchTime")]
    launch_time: u64,
}

#[allow(dead_code)]
#[derive(NodeDescription, GraplStaticId)]
pub struct AwsEc2Instance {
    #[grapl(static_id, immutable)]
    arn: String,
    #[grapl(static_id, immutable)]
    launch_time: u64,
}

impl IAwsEc2InstanceNode for AwsEc2InstanceNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }

    fn with_arn(&mut self, arn: impl Into<ImmutableStrProp>) -> &mut Self {
        info!("custom arn handler");
        self.get_mut_dynamic_node().set_property("arn", arn.into());
        self
    }
}

fn main() {
    let raw_guard_duty_alert = read_log();

    let log: InstanceDetails = serde_json::from_slice(raw_guard_duty_alert).unwrap();

    let mut ec2 = AwsEc2InstanceNode::new(AwsEc2InstanceNode::static_strategy());
    ec2.with_arn(log.arn).with_launch_time(log.launch_time);

    let _launch_time = ec2.get_launch_time().expect("Couldn't find launch_time!");
    let _arn = ec2.get_arn().expect("Couldn't find arn!");

    let mut graph = GraphDescription::new();
    graph.add_node(ec2);
}
