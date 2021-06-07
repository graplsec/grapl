use std::{
    io::Stdout,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use dgraph_tonic::{Client as DgraphClient, Mutate, Query};
use failure::{bail, Error};
use grapl_config::{
    env_helpers::{s3_event_emitters_from_env, FromEnv},
    event_caches,
};
pub use grapl_graph_descriptions::graph_mutation_service::graph_mutation_rpc_client::GraphMutationRpcClient;
use grapl_graph_descriptions::{
    graph_description::{Edge, EdgeList, IdentifiedGraph, IdentifiedNode, MergedGraph, MergedNode},
    graph_mutation_service::*,
};
use grapl_observe::{
    dgraph_reporter::DgraphMetricReporter,
    metric_reporter::{tag, MetricReporter},
};
use grapl_service::{decoder::ProtoDecoder, serialization::MergedGraphSerializer};
use grapl_utils::{future_ext::GraplFutureExt, rusoto_ext::dynamodb::GraplDynamoDbClientExt};
use lazy_static::lazy_static;
use rusoto_dynamodb::{
    AttributeValue, BatchGetItemInput, DynamoDb, DynamoDbClient, GetItemInput, KeysAndAttributes,
};
use rusoto_s3::S3Client;
use rusoto_sqs::SqsClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqs_executor::{
    cache::{Cache, Cacheable},
    errors::{CheckedError, Recoverable},
    event_handler::{CompletedEvents, EventHandler},
    make_ten,
    s3_event_retriever::S3PayloadRetriever,
};
use tonic::transport::Channel;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct GraphMerger<CacheT>
where
    CacheT: Cache + Clone + Send + Sync + 'static,
{
    graph_mutation_client: GraphMutationRpcClient<Channel>,
    metric_reporter: MetricReporter<Stdout>,
    cache: CacheT,
}

impl<CacheT> GraphMerger<CacheT>
where
    CacheT: Cache + Clone + Send + Sync + 'static,
{
    pub async fn new(
        graph_mutation_client: GraphMutationRpcClient<Channel>,
        metric_reporter: MetricReporter<Stdout>,
        cache: CacheT,
    ) -> Self {
        Self {
            graph_mutation_client,
            metric_reporter,
            cache,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GraphMergerError {
    #[error("UnexpectedError")]
    Unexpected(String),
    #[error("GrpcError")]
    GrpcError(#[from] tonic::Status),
}

impl CheckedError for GraphMergerError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Transient
    }
}

#[async_trait]
impl<CacheT: Cache> EventHandler for GraphMerger<CacheT> {
    type InputEvent = IdentifiedGraph;
    type OutputEvent = MergedGraph;
    type Error = GraphMergerError;

    async fn handle_event(
        &mut self,
        subgraph: Self::InputEvent,
        _completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        if subgraph.is_empty() {
            warn!("Attempted to merge empty subgraph. Short circuiting.");
            return Ok(MergedGraph::default());
        }

        info!(
            message=
            "handling new subgraph",
            nodes=?subgraph.nodes.len(),
            edges=?subgraph.edges.len(),
        );

        let mut merged_graph = MergedGraph::new();
        let mut node_responses = Vec::with_capacity(subgraph.nodes.len());
        let mut edge_responses = Vec::with_capacity(subgraph.edges.len());

        let mut failed = None;

        for (_, node) in subgraph.nodes.into_iter() {
            let mut graph_mutation_client = self.graph_mutation_client.clone();
            let r = graph_mutation_client
                .set_node(SetNodeRequest {
                    node: Some(node.clone()),
                })
                .await;

            match r {
                Err(e) => {
                    tracing::error!(message="Failed to upsert node2", err=?e);
                    failed = Some(e);
                }
                Ok(r) => {
                    let node_uid = r.into_inner().rpc_result.unwrap();
                    node_responses.push((node_uid, node));
                }
            }
        }
        for (_, EdgeList { edges }) in subgraph.edges.into_iter() {
            for edge in edges {
                let mut graph_mutation_client = self.graph_mutation_client.clone();
                let r = graph_mutation_client
                    .set_edge(SetEdgeRequest {
                        edge: Some(edge.clone()),
                    })
                    .await;
                match r {
                    Err(e) => {
                        tracing::error!(message="Failed to upsert edge", err=?e);
                        failed = Some(e);
                    }
                    Ok(r) => {
                        let edge_res = r.into_inner().rpc_result.unwrap();
                        edge_responses.push((edge_res, edge))
                    }
                }
            }
        }

        for (node_uid, node) in node_responses {
            tracing::debug!(message="set_node", set_node_res=?node_uid);
            if let set_node_result::RpcResult::Set(SetNodeSuccess { node_uid }) = node_uid {
                merged_graph.add_node(MergedNode {
                    uid: node_uid,
                    node_key: node.node_key,
                    node_type: node.node_type,
                    properties: node.properties,
                });
            } else {
                tracing::error!(message="Failed to upsert node2", err=?node_uid);
            }
        }

        for (set_edge_res, edge) in edge_responses {
            tracing::debug!(message="set_edge", set_edge_res=?set_edge_res);
            if let set_edge_result::RpcResult::Set(SetEdgeSuccess { src_uid, dst_uid }) =
                set_edge_res
            {
                merged_graph.add_edge(
                    edge.edge_name,
                    edge.from_node_key,
                    src_uid,
                    edge.to_node_key,
                    dst_uid,
                );
            }
        }

        if let Some(e) = failed {
            Err(Ok((merged_graph, e.into())))
        } else {
            Ok(merged_graph)
        }
    }
}

pub fn time_based_key_fn(_event: &[u8]) -> String {
    let cur_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    let cur_day = cur_ms - (cur_ms % 86400);

    format!("{}/{}-{}", cur_day, cur_ms, uuid::Uuid::new_v4())
}
