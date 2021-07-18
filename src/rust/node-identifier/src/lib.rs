use std::collections::HashMap;

use async_trait::async_trait;
use dynamic_sessiondb::{
    DynamicMappingDb,
    NodeDescriptionIdentifier,
};
use failure::Error;
use grapl_config::{
    env_helpers::{
        s3_event_emitters_from_env,
        FromEnv,
    },
    event_caches,
};
use grapl_observe::metric_reporter::MetricReporter;
use grapl_service::{
    decoder::ProtoDecoder,
    serialization::IdentifiedGraphSerializer,
};
use grapl_utils::rusoto_ext::dynamodb::GraplDynamoDbClientExt;
use rusoto_dynamodb::DynamoDbClient;
use rusoto_sqs::SqsClient;
use rust_proto::graph_descriptions::{
    GraphDescription,
    IdentifiedGraph,
    IdentifiedNode,
    NodeDescription,
};
use sessiondb::SessionDb;
use sqs_executor::{
    cache::Cache,
    event_handler::{
        CompletedEvents,
        EventHandler,
    },
    make_ten,
    s3_event_retriever::S3PayloadRetriever,
    time_based_key_fn,
};
use tap::tap::TapOptional;
use tracing::{
    info,
    warn,
};

use crate::error::NodeIdentifierError;

pub mod dynamic_sessiondb;
mod error;
pub mod sessiondb;
pub mod sessions;

/**
    The `NodeIdentifier` takes in graphs of previously unidentified nodes and performs identification
    based on the configured strategies for that node type.

    The strategies come in two variants:

    * [Session](`rust_proto::graph_descriptions::Session`) - strategy used for nodes with lifetimes (e.g. process, network connection)
    * [Static](`rust_proto::graph_descriptions::Static`) - strategy used for nodes with canonical and unique identifiers (e.g. aws events)
*/
#[derive(Clone)]
pub struct NodeIdentifier<D, CacheT>
where
    D: GraplDynamoDbClientExt,
    CacheT: Cache,
{
    dynamic_identifier: NodeDescriptionIdentifier<D>,
    node_id_db: D,
    should_default: bool,
    cache: CacheT,
}

impl<D, CacheT> NodeIdentifier<D, CacheT>
where
    D: GraplDynamoDbClientExt,
    CacheT: Cache,
{
    pub fn new(
        dynamic_identifier: NodeDescriptionIdentifier<D>,
        node_id_db: D,
        should_default: bool,
        cache: CacheT,
    ) -> Self {
        Self {
            dynamic_identifier,
            node_id_db,
            should_default,
            cache,
        }
    }

    // todo: We should be yielding IdentifiedNode's here
    #[tracing::instrument(fields(node_key=?node.node_key), skip(self, node))]
    async fn attribute_node_key(&self, node: &NodeDescription) -> Result<IdentifiedNode, Error> {
        let new_node = self
            .dynamic_identifier
            .attribute_dynamic_node(&node)
            .await?;
        Ok(new_node.into())
    }

    /// Performs batch identification of unidentified nodes into identified nodes.
    ///
    /// A map of unidentified node keys to identified node keys will be returned in addition to the
    /// last error, if any, that occurred while identifying nodes.
    #[tracing::instrument(skip(self, unidentified_subgraph, identified_graph))]
    pub async fn identify_nodes(
        &self,
        unidentified_subgraph: &GraphDescription,
        identified_graph: &mut IdentifiedGraph,
    ) -> (HashMap<String, String>, Option<failure::Error>) {
        let mut identified_nodekey_map = HashMap::new();
        let mut attribution_failure = None;

        // new method
        for (unidentified_node_key, unidentified_node) in unidentified_subgraph.nodes.iter() {
            let identified_node = match self.attribute_node_key(&unidentified_node).await {
                Ok(identified_node) => identified_node,
                Err(e) => {
                    warn!(
                        message="Failed to attribute node_key",
                        node_key=?unidentified_node_key,
                        error=?e
                    );
                    attribution_failure = Some(e);
                    continue;
                }
            };

            identified_nodekey_map.insert(
                unidentified_node_key.to_owned(),
                identified_node.clone_node_key(),
            );
            identified_graph.add_node(identified_node);
        }

        (identified_nodekey_map, attribution_failure)
    }

    /// Takes the edges in the `unidentified_graph` and inserts ones that can be properly identified
    /// into the `identified_graph` using the `identified_nodekey_map` returned from a previous
    /// node key identification process.
    #[tracing::instrument(skip(
        self,
        unidentified_subgraph,
        identified_graph,
        identified_nodekey_map
    ))]
    pub fn identify_edges(
        &self,
        unidentified_subgraph: &GraphDescription,
        identified_graph: &mut IdentifiedGraph,
        identified_nodekey_map: HashMap<String, String>,
    ) {
        let identified_node_edges = unidentified_subgraph.edges.iter()
            // filter out all edges for nodes that were not identified (also gets our from_key)
            .filter_map(|(from_key, edge_list)| {
                identified_nodekey_map.get(from_key)
                    .tap_none(|| tracing::warn!(
                        message = concat!(
                            "Could not get node_key mapping for from_key.",
                            "This means we can't identify the edge because we rely on the node to first be identified."
                        ),
                        from_key=?from_key,
                    ))
                    .map(|identified_from_key| (identified_from_key, edge_list))
            });

        // for each of the edges from identified nodes...
        for (identified_from_key, edge_list) in identified_node_edges {
            // map the to_key for each edge with the new, identified to_key
            let identified_edges = edge_list.edges.iter()
                .filter_map(|edge| {
                    // check if
                    identified_nodekey_map.get(&edge.to_node_key)
                        .tap_none(|| tracing::warn!(
                                message=concat!(
                                    "Could not get node_key mapping for to_key.",
                                    "This means we can't identify the edge because we rely on the node to first be identified."
                                ),
                                to_key=?edge.to_node_key
                            )
                        )
                        .map(|identified_to_edge| (identified_to_edge, &edge.edge_name))
                });

            // add all identified edges into the `identified_graph`
            for (identified_to_key, edge_name) in identified_edges {
                identified_graph.add_edge(
                    edge_name.to_owned(),
                    identified_from_key.to_owned(),
                    identified_to_key.to_owned(),
                );
            }
        }
    }
}

#[async_trait]
impl<D, CacheT> EventHandler for NodeIdentifier<D, CacheT>
where
    D: GraplDynamoDbClientExt,
    CacheT: Cache,
{
    type InputEvent = GraphDescription;
    type OutputEvent = IdentifiedGraph;
    type Error = NodeIdentifierError;

    #[tracing::instrument(skip(self, unidentified_subgraph, _completed))]
    async fn handle_event(
        &mut self,
        unidentified_subgraph: Self::InputEvent,
        _completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        /*
           1. for the nodes in the unidentified graph, do the following
               1. identify each node
               2. record the old node key -> new node key mapping
               3. record which nodes failed to identify
               4. place the correctly identified nodes into a new, identified graph
           2. for the edges in the unidentified graph, do the following
               1. for both the 'from' and 'to' sections, look up the old-node-key -> new-node-key map to fetch updated values
                   1. if both are present, update edge and add it to the identified graph
                   2. if one or more are missing, warn with an appropriate error message and skip the edge
           3. if identified graph is empty, return as full error
           4. if any nodes failed to identify or any errors occurred, return as a partial error
           5. return as full graph
        */

        info!(
            message="Identifying a new graph",
            node_count=?unidentified_subgraph.nodes.len(),
            edge_count=?unidentified_subgraph.edges.len(),
        );

        if unidentified_subgraph.is_empty() {
            warn!("Received empty subgraph.");
            return Ok(IdentifiedGraph::new());
        }

        let mut identified_graph = IdentifiedGraph::new();

        let (identified_nodekey_map, attribution_failure) = self
            .identify_nodes(&unidentified_subgraph, &mut identified_graph)
            .await;

        info!(
            message="Performed node identification",
            total_edges=?unidentified_subgraph.nodes.len(),
            identified_edges=?identified_graph.nodes.len()
        );

        self.identify_edges(
            &unidentified_subgraph,
            &mut identified_graph,
            identified_nodekey_map,
        );

        info!(
            message = "Performed edge identification",
            total_edges = unidentified_subgraph.edges.len(),
            identified_edges = identified_graph.edges.len()
        );

        match attribution_failure {
            Some(_) if identified_graph.is_empty() => Err(Err(NodeIdentifierError::Unexpected)),
            Some(_) => {
                /* todo: error message is misleading. someone reading this would believe we identified
                 * a smaller number of nodes that actually identified (due to identities of nodes coalescing)
                 */
                warn!(
                    message = "Partial Success",
                    identified_nodes = identified_graph.nodes.len(),
                    identified_edges = identified_graph.edges.len(),
                );

                Err(Ok((identified_graph, NodeIdentifierError::Unexpected))) // todo: Use a real error here
            }
            None => {
                info!("Identified all nodes");

                Ok(identified_graph)
            }
        }
    }
}

pub async fn handler(should_default: bool) -> Result<(), Box<dyn std::error::Error>> {
    let (env, _guard) = grapl_config::init_grapl_env!();
    let source_queue_url = grapl_config::source_queue_url();

    tracing::info!(
        source_queue_url=?source_queue_url,
        env=?env,
        "handler_init"
    );

    let sqs_client = SqsClient::from_env();
    let cache = &mut event_caches(&env).await;
    let serializer = &mut make_ten(async { IdentifiedGraphSerializer::default() }).await;
    let s3_emitter = &mut s3_event_emitters_from_env(&env, time_based_key_fn).await;

    let s3_payload_retriever = &mut make_ten(async {
        S3PayloadRetriever::new(
            |region_str| grapl_config::env_helpers::init_s3_client(&region_str),
            ProtoDecoder::default(),
            MetricReporter::new(&env.service_name),
        )
    })
    .await;

    let dynamo = DynamoDbClient::from_env();
    let dyn_session_db = SessionDb::new(dynamo.clone(), grapl_config::dynamic_session_table_name());
    let dyn_mapping_db = DynamicMappingDb::new(dynamo.clone());

    let dyn_node_identifier =
        NodeDescriptionIdentifier::new(dyn_session_db, dyn_mapping_db, should_default);

    let node_identifier = &mut make_ten(async {
        NodeIdentifier::new(
            dyn_node_identifier,
            dynamo,
            should_default,
            cache[0].to_owned(),
        )
    })
    .await;

    info!("Starting process_loop");
    sqs_executor::process_loop(
        grapl_config::source_queue_url(),
        std::env::var("DEAD_LETTER_QUEUE_URL").expect("DEAD_LETTER_QUEUE_URL"),
        cache,
        sqs_client.clone(),
        node_identifier,
        s3_payload_retriever,
        s3_emitter,
        serializer,
        MetricReporter::new(&env.service_name),
    )
    .await;

    info!("Exiting");
    Ok(())
}
