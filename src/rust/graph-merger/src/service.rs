use std::{
    collections::HashMap,
    fmt::Debug,
    io::Stdout,
    sync::{
        Arc,
        Mutex,
    },
    time::{
        Duration,
        SystemTime,
        UNIX_EPOCH,
    },
};

use async_trait::async_trait;
use dgraph_tonic::{
    Client as DgraphClient,
    Mutate,
    Query,
};
use failure::{
    bail,
    Error,
};
use grapl_config::{
    env_helpers::{
        s3_event_emitters_from_env,
        FromEnv,
    },
    event_caches,
};
use grapl_observe::{
    dgraph_reporter::DgraphMetricReporter,
    metric_reporter::{
        tag,
        MetricReporter,
    },
};
use grapl_service::{
    decoder::ProtoDecoder,
    serialization::MergedGraphSerializer,
};
use grapl_utils::{
    future_ext::GraplFutureExt,
    rusoto_ext::dynamodb::GraplDynamoDbClientExt,
};
use lazy_static::lazy_static;
use rusoto_dynamodb::{
    AttributeValue,
    BatchGetItemInput,
    DynamoDb,
    DynamoDbClient,
    GetItemInput,
    KeysAndAttributes,
};
use rusoto_s3::S3Client;
use rusoto_sqs::SqsClient;
use rust_proto::graph_descriptions::{
    Edge,
    EdgeList,
    IdentifiedGraph,
    IdentifiedNode,
    MergedGraph,
    MergedNode,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use sqs_executor::{
    cache::{
        Cache,
        Cacheable,
    },
    errors::{
        CheckedError,
        Recoverable,
    },
    event_handler::{
        CompletedEvents,
        EventHandler,
    },
    make_ten,
    s3_event_retriever::S3PayloadRetriever,
};
use tracing::{
    error,
    info,
    warn,
};

use crate::{
    reverse_resolver,
    reverse_resolver::{
        get_r_edges_from_dynamodb,
        ReverseEdgeResolver,
    },
    upsert_util,
    upserter,
};

#[derive(Clone)]
pub struct GraphMerger<CacheT: Cache> {
    mg_client: Arc<DgraphClient>,
    reverse_edge_resolver: ReverseEdgeResolver,
    metric_reporter: MetricReporter<Stdout>,
    cache: CacheT,
}

impl<CacheT: Cache> GraphMerger<CacheT> {
    pub fn new(
        mg_alphas: Vec<String>,
        reverse_edge_resolver: ReverseEdgeResolver,
        metric_reporter: MetricReporter<Stdout>,
        cache: CacheT,
    ) -> Self {
        let mg_client = DgraphClient::new(mg_alphas).expect("Failed to create dgraph client.");

        Self {
            mg_client: Arc::new(mg_client),
            reverse_edge_resolver,
            metric_reporter,
            cache,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GraphMergerError {
    #[error("UnexpectedError")]
    Unexpected(String),
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

        let uncached_nodes = subgraph.nodes.into_iter().map(|(_, n)| n);
        let mut uncached_edges: Vec<_> = subgraph
            .edges
            .into_iter()
            .flat_map(|e| e.1.into_vec())
            .collect();
        let reverse = self
            .reverse_edge_resolver
            .resolve_reverse_edges(uncached_edges.clone())
            .await
            .map_err(Err)?;

        uncached_edges.extend_from_slice(&reverse[..]);

        let mut merged_graph = MergedGraph::new();
        let mut uncached_subgraph = IdentifiedGraph::new();

        for node in uncached_nodes {
            uncached_subgraph.add_node(node);
        }

        for edge in uncached_edges {
            uncached_subgraph.add_edge(edge.edge_name, edge.from_node_key, edge.to_node_key);
        }

        upserter::GraphMergeHelper {}
            .upsert_into(
                self.mg_client.clone(),
                &uncached_subgraph,
                &mut merged_graph,
            )
            .await;

        Ok(merged_graph)
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
