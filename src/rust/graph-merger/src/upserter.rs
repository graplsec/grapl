use std::{
    collections::HashMap,
    sync::Arc,
};

use dgraph_query_lib::{
    condition::{
        Condition,
        ConditionValue,
    },
    mutation::{
        Mutation,
        MutationBuilder,
        MutationPredicateValue,
        MutationUID,
        MutationUnit,
    },
    predicate::{
        Field,
        Predicate,
    },
    query::QueryBuilder,
    queryblock::{
        QueryBlock,
        QueryBlockBuilder,
        QueryBlockType,
    },
    upsert::{
        Upsert,
        UpsertBlock,
    },
    ToQueryString,
};
use dgraph_tonic::{
    Client as DgraphClient,
    Mutate,
    Mutation as DgraphMutation,
    MutationResponse,
    Query,
};
use futures::StreamExt;
use futures_retry::{
    FutureRetry,
    RetryPolicy,
};
use grapl_utils::iter_ext::GraplIterExt;
pub use rust_proto::node_property::Property::{
    DecrementOnlyInt as ProtoDecrementOnlyIntProp,
    DecrementOnlyUint as ProtoDecrementOnlyUintProp,
    ImmutableInt as ProtoImmutableIntProp,
    ImmutableStr as ProtoImmutableStrProp,
    ImmutableUint as ProtoImmutableUintProp,
    IncrementOnlyInt as ProtoIncrementOnlyIntProp,
    IncrementOnlyUint as ProtoIncrementOnlyUintProp,
};
use rust_proto::{
    graph_descriptions::*,
    node_property::Property,
};

use crate::upsert_util;

const DGRAPH_CONCURRENCY_UPSERTS: usize = 8;
// DGraph Live Loader uses a size of 1,000 elements and they claim this has relatively good performance
const DGRAPH_UPSERT_CHUNK_SIZE: usize = 1024;

pub struct GraphMergeHelper {}

impl GraphMergeHelper {
    pub async fn upsert_into(
        &self,
        dgraph_client: Arc<DgraphClient>,
        identified_graph: &IdentifiedGraph,
        merged_graph: &mut MergedGraph,
    ) {
        let node_key_map_to_uid = self
            .upsert_nodes(dgraph_client.clone(), identified_graph, merged_graph)
            .await;
        self.upsert_edges(dgraph_client, &identified_graph, node_key_map_to_uid)
            .await;
    }

    async fn upsert_nodes(
        &self,
        dgraph_client: Arc<DgraphClient>,
        identified_graph: &IdentifiedGraph,
        merged_graph: &mut MergedGraph,
    ) -> HashMap<String, u64> {
        let mut key_to_query_name = HashMap::new();
        let mut node_upserts = Vec::with_capacity(identified_graph.nodes.len());
        for (unique_id, node) in identified_graph.nodes.values().enumerate() {
            let (query, upserts) = upsert_util::build_upserts(
                unique_id as u128,
                &node.node_key,
                &node.node_type,
                &node.properties,
                &mut key_to_query_name,
            );
            node_upserts.push((query, upserts));
        }

        tracing::info!(message = "Upserting nodes", count = node_upserts.len());
        let responses: Vec<dgraph_tonic::Response> = futures::stream::iter(
            node_upserts
                .into_iter()
                .chunks_owned(DGRAPH_UPSERT_CHUNK_SIZE),
        )
        .map(move |upsert_chunk| {
            let mut combined_query = String::new();
            let mut all_mutations = Vec::new();
            for (query_block, mutations) in upsert_chunk.iter() {
                combined_query.push_str(&query_block);
                all_mutations.extend_from_slice(mutations);
            }

            let combined_query = format!(
                r"
            {{
                    {}
            }}
            ",
                combined_query
            );

            tracing::debug!(message="Generated query for upsert", combined_query=?combined_query);

            let dgraph_client = dgraph_client.clone();
            Self::enforce_transaction(move || {
                let mut txn = dgraph_client.new_mutated_txn();
                txn.upsert_and_commit_now(combined_query.clone(), all_mutations.clone())
            })
        })
        .buffer_unordered(DGRAPH_CONCURRENCY_UPSERTS)
        .collect::<Vec<_>>()
        .await;

        let mut node_key_map_to_uid = HashMap::new();
        let mut uids = HashMap::new();
        for response in responses.iter() {
            let query_responses: serde_json::Value = match serde_json::from_slice(&response.json) {
                Ok(response) => response,
                Err(e) => {
                    tracing::error!(message="Failed to parse JSON response for upsert", error=?e);
                    continue;
                }
            };
            tracing::debug!(
                message="Received upsert response",
                query_response=?query_responses,
                uids=?response.uids,
            );
            uids.extend(response.uids.clone());
            extract_node_key_map_uid(&query_responses, &mut node_key_map_to_uid);
        }

        for node in identified_graph.nodes.values() {
            let IdentifiedNode {
                node_key,
                node_type,
                properties,
            } = node.to_owned();

            let uid = node_key_map_to_uid
                .get(&node_key)
                .copied()
                .or_else(|| uid_from_uids(&node_key, &key_to_query_name, &uids));
            let uid = match uid {
                Some(uid) => uid,
                None => {
                    tracing::error!(
                        message="Failed to retrieve uid associated with node_key",
                        node_key=?node_key,
                    );
                    continue;
                }
            };

            merged_graph.add_node(MergedNode {
                properties,
                uid,
                node_key,
                node_type,
            });
        }
        node_key_map_to_uid
    }

    async fn upsert_edges(
        &self,
        dgraph_client: Arc<DgraphClient>,
        identified_graph: &IdentifiedGraph,
        mut node_key_to_uid: HashMap<String, u64>,
    ) {
        let all_edges: Vec<_> = identified_graph
            .edges
            .iter()
            .flat_map(|(_, EdgeList { edges })| edges)
            .map(
                |Edge {
                     from_node_key,
                     to_node_key,
                     edge_name,
                 }| {
                    (
                        from_node_key.clone(),
                        to_node_key.clone(),
                        edge_name.clone(),
                    )
                },
            )
            .collect();

        let mut unresolved = vec![];
        for (from_node_key, to_node_key, edge_name) in all_edges.iter() {
            if !node_key_to_uid.contains_key(from_node_key) {
                unresolved.push(from_node_key);
            }
            if !node_key_to_uid.contains_key(to_node_key) {
                unresolved.push(to_node_key);
            }
        }
        unresolved.sort_unstable();
        unresolved.dedup();
        if !unresolved.is_empty() {
            let m = query_for_node_keys(dgraph_client.clone(), &unresolved[..]).await;
            for (node_key, uid) in m.into_iter() {
                node_key_to_uid.insert(node_key, uid);
            }
        }

        let mut mutations = Vec::with_capacity(all_edges.len());
        for items in all_edges.into_iter().chunks_owned(DGRAPH_UPSERT_CHUNK_SIZE) {
            let mut mutation_units = vec![];

            for (from_node_key, to_node_key, edge_name) in items.iter() {
                let from_uid = node_key_to_uid.get(from_node_key);
                let to_uid = node_key_to_uid.get(to_node_key);
                let (from_uid, to_uid) = match (from_uid, to_uid) {
                    (Some(from_uid), Some(to_uid)) => (*from_uid, *to_uid),
                    (from_uid, to_uid) => {
                        tracing::error!(
                            message="Could not retrieve uids",
                            from_uid=?from_uid,
                            to_uid=?to_uid,
                            from_node_key=?from_node_key,
                            to_node_key=?to_node_key,
                        );
                        continue;
                    }
                };

                let (from_uid, to_uid) = (from_uid.to_string(), to_uid.to_string());
                let mutation_unit = MutationUnit::new(MutationUID::uid(&from_uid)).predicate(
                    &edge_name,
                    MutationPredicateValue::Edges(vec![MutationUID::uid(&to_uid)]),
                );
                mutation_units.push(mutation_unit);
            }
            let mutation = MutationBuilder::default()
                .set(mutation_units)
                .build()
                .unwrap();
            mutations.push(mutation);
        }

        futures::stream::iter(mutations.into_iter())
            .map(|mutation| {
                let dgraph_client = dgraph_client.clone();
                Self::enforce_transaction(move || {
                    let mut dgraph_mutation = dgraph_tonic::Mutation::new();
                    dgraph_mutation.set_set_json(&mutation.set).unwrap_or_else(
                        |e| tracing::error!(message="Failed to set json for mutation", error=?e),
                    );

                    let mut txn = dgraph_client.new_mutated_txn();
                    txn.mutate_and_commit_now(dgraph_mutation.clone())
                })
            })
            .buffer_unordered(DGRAPH_CONCURRENCY_UPSERTS)
            .collect::<Vec<_>>()
            .await;
    }

    async fn enforce_transaction<Factory, Txn>(f: Factory) -> dgraph_tonic::Response
    where
        Factory: FnMut() -> Txn + 'static + Unpin,
        Txn: std::future::Future<Output = Result<dgraph_tonic::Response, anyhow::Error>>,
    {
        let handle_upsert_err = UpsertErrorHandler {};
        let (response, attempts) = FutureRetry::new(f, handle_upsert_err)
            .await
            .expect("Surfaced an error despite retry strategy while performing an upsert.");

        tracing::info!(message = "Performed upsert", attempts = attempts);

        response
    }
}

pub struct UpsertErrorHandler {}

impl futures_retry::ErrorHandler<anyhow::Error> for UpsertErrorHandler {
    type OutError = anyhow::Error;

    fn handle(&mut self, attempt: usize, e: anyhow::Error) -> RetryPolicy<Self::OutError> {
        let attempt = attempt as u64;
        tracing::warn!(
            message="Failed to enforce transaction",
            error=?e,
            attempt=?attempt,
        );
        match attempt {
            0..=5 => RetryPolicy::Repeat,
            t @ 6..=20 => RetryPolicy::WaitRetry(std::time::Duration::from_millis(10 * t as u64)),
            21..=u64::MAX => RetryPolicy::ForwardError(e),
        }
    }
}

async fn query_for_node_keys(
    dgraph_client: Arc<DgraphClient>,
    node_keys: &[&String],
) -> HashMap<String, u64> {
    let mut resolved_nodes = HashMap::new();

    let mut query_blocks = Vec::with_capacity(node_keys.len());
    for node_key in node_keys.iter() {
        let query_block = gen_node_key_query(node_key);
        query_blocks.push(query_block);
    }

    let query = QueryBuilder::default()
        .query_blocks(query_blocks)
        .build()
        .unwrap();

    let mut txn = dgraph_client.new_read_only_txn();
    let query_responses = txn
        .query(query.to_query_string())
        .await
        .expect("query failed");

    let query_responses: HashMap<String, Vec<HashMap<String, String>>> =
        serde_json::from_slice(&query_responses.json).expect("response failed to parse");

    for (_, query_response) in query_responses.into_iter() {
        let query_response = match query_response.as_slice() {
            [query_response] => query_response,
            [] => {
                tracing::error!(message = "Empty response for node_key");
                continue;
            }
            res => {
                tracing::error!(message = "Too many responses for node_key", count=?res.len());
                continue;
            }
        };
        let node_key = query_response.get("node_key");
        let uid = query_response.get("uid");
        let (node_key, uid) = match (node_key, uid) {
            (Some(node_key), Some(uid)) => (node_key, uid),
            (missing_key, missing_uid) => {
                tracing::error!(message="Unable to retrieve node_key and uid", node_key=?missing_key, uid=?missing_uid);
                continue;
            }
        };
        let uid = u64::from_str_radix(&uid[2..], 16).expect("uid is not valid hex");
        resolved_nodes.insert(node_key.to_owned(), uid);
    }

    resolved_nodes
}

fn gen_node_key_query(node_key: &str) -> QueryBlock {
    QueryBlockBuilder::default()
        .query_type(QueryBlockType::query())
        .root_filter(Condition::EQ(
            "node_key".to_string(),
            ConditionValue::string(node_key),
        ))
        .predicates(vec![
            Predicate::Field(Field::new("uid")),
            Predicate::Field(Field::new("node_key")),
        ])
        .first(1)
        .build()
        .unwrap()
}

fn uid_from_uids(
    node_key: &str,
    key_to_query_name: &HashMap<String, String>,
    uids: &HashMap<String, String>,
) -> Option<u64> {
    let query_name = key_to_query_name.get(node_key)?;
    let uid = uids.get(query_name)?;
    Some(u64::from_str_radix(&uid[2..], 16).expect("uid is not valid hex"))
}

fn extract_node_key_map_uid(
    dgraph_response: &serde_json::Value,
    node_key_map_to_uid: &mut HashMap<String, u64>,
) {
    let query_responses = dgraph_response.as_object().expect("Invalid response");

    for query_response in query_responses.values() {
        let query_response = query_response.as_array().expect("Invalid response");
        for query_response in query_response {
            let uid = query_response
                .get("uid")
                .expect("uid")
                .as_str()
                .expect("uid");
            let node_key = query_response
                .get("node_key")
                .expect("node_key")
                .as_str()
                .expect("node_key");

            // dgraph uids are hex encoded as '0x1b'
            let uid = u64::from_str_radix(&uid[2..], 16).expect("uid is not valid hex");
            node_key_map_to_uid.insert(node_key.to_owned(), uid);
        }
    }
}
