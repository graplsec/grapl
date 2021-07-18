use std::{
    collections::HashMap,
    io::Stdout,
};

use grapl_observe::metric_reporter::MetricReporter;
use grapl_utils::{
    future_ext::GraplFutureExt,
    rusoto_ext::dynamodb::GraplDynamoDbClientExt,
};
use lazy_static::lazy_static;
use rusoto_dynamodb::{
    AttributeValue,
    BatchGetItemInput,
    DynamoDbClient,
    KeysAndAttributes,
};
use rust_proto::graph_descriptions::Edge;
use serde::{
    Deserialize,
    Serialize,
};

use crate::service::GraphMergerError;

lazy_static! {
    /// timeout for dynamodb queries
    static ref DYNAMODB_QUERY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);
}

#[derive(Clone)]
pub struct ReverseEdgeResolver {
    dynamo: DynamoDbClient,
    r_edge_cache: std::sync::Arc<std::sync::Mutex<lru::LruCache<String, String>>>,
    metric_reporter: MetricReporter<Stdout>,
}

impl ReverseEdgeResolver {
    pub fn new(
        dynamo: DynamoDbClient,
        metric_reporter: MetricReporter<Stdout>,
        cache_size: usize,
    ) -> Self {
        let r_edge_cache = lru::LruCache::new(cache_size);
        let r_edge_cache = std::sync::Arc::new(std::sync::Mutex::new(r_edge_cache));
        Self {
            dynamo,
            r_edge_cache,
            metric_reporter,
        }
    }

    pub async fn resolve_reverse_edges(
        &self,
        edges: Vec<Edge>,
    ) -> Result<Vec<Edge>, GraphMergerError> {
        if edges.is_empty() {
            return Ok(vec![]);
        }
        let (mut reversed, remaining) = self.resolve_reverse_edges_from_cache(edges).await;
        if remaining.is_empty() {
            return Ok(reversed);
        }
        let mut edge_names: Vec<&String> = remaining.iter().map(|e| &e.edge_name).collect();
        edge_names.sort_unstable();
        edge_names.dedup();

        let resolved = get_r_edges_from_dynamodb(&self.dynamo, &edge_names).await?;

        let cache = self.r_edge_cache.clone();
        let mut cache = cache.lock().unwrap();
        for f_edge in remaining.into_iter() {
            if let Some(Some(r_edge_name)) = resolved.get(&f_edge.edge_name) {
                cache.put(f_edge.edge_name.clone(), r_edge_name.clone());
                cache.put(r_edge_name.clone(), f_edge.edge_name.clone());
                reversed.push(reverse_edge(&f_edge, r_edge_name.to_owned()));
            }
        }
        reversed.sort_unstable();
        reversed.dedup();
        Ok(reversed)
    }

    pub async fn resolve_reverse_edges_from_cache(
        &self,
        edges: Vec<Edge>,
    ) -> (Vec<Edge>, Vec<Edge>) {
        let mut reversed = vec![];
        let mut remaining = vec![];
        let mut cache_hit = 0;
        let mut cache_miss = 0;
        let cache = self.r_edge_cache.clone();
        let mut cache = cache.lock().unwrap();
        for edge in edges.into_iter() {
            match cache.get(&edge.edge_name).map(String::from) {
                Some(r_edge_name) => {
                    reversed.push(reverse_edge(&edge, r_edge_name));
                    cache_hit += 1;
                }
                None => {
                    remaining.push(edge);
                    cache_miss += 1;
                }
            }
        }
        drop(cache);

        self.metric_reporter.clone().counter(
            "reverse_resolver.cache.hit.count",
            cache_hit as f64,
            0.10,
            &[],
        );
        self.metric_reporter.clone().counter(
            "reverse_resolver.cache.miss.count",
            cache_miss as f64,
            0.10,
            &[],
        );
        (reversed, remaining)
    }
}

fn reverse_edge(edge: &Edge, reverse_edge_name: String) -> Edge {
    Edge {
        from_node_key: edge.to_node_key.to_owned(),
        to_node_key: edge.from_node_key.to_owned(),
        edge_name: reverse_edge_name,
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct EdgeMapping {
    r_edge: String,
}

/// Returns a HashMap of f_edge -> Optional r_edge entries from dynamodb
pub async fn get_r_edges_from_dynamodb(
    client: &DynamoDbClient,
    f_edges: &[&String],
) -> Result<HashMap<String, Option<String>>, GraphMergerError> {
    let schema_table_name = std::env::var("GRAPL_SCHEMA_TABLE").expect("GRAPL_SCHEMA_TABLE");

    let keys_and_attributes = make_keys(f_edges);
    tracing::debug!(
        message="Querying dynamodb for reverse edges",
        edge_count=?keys_and_attributes.keys.len(),
    );
    let mut request_items = HashMap::with_capacity(1);
    request_items.insert(schema_table_name.clone(), keys_and_attributes);

    let query = BatchGetItemInput {
        request_items,
        return_consumed_capacity: None,
    };

    /*
       1. Map timeout error
       2. Map Rusoto error
       3. Grab responses (HashMap<String, Vec<HashMap<String, AttributeValue>>>) (or return error)
       4. Pull the entire set of responses out for the schema table (Vec<HashMap<String, AttributeValue>>) (or return error)
    */
    let schema_table_response: Vec<HashMap<String, AttributeValue>> = client
        .batch_get_item_reliably(query)
        .timeout(*DYNAMODB_QUERY_TIMEOUT)
        .await
        .map_err(|e| GraphMergerError::Unexpected(e.to_string()))?
        .map_err(|e| GraphMergerError::Unexpected(e.to_string()))?
        .responses
        .ok_or(GraphMergerError::Unexpected(
            "Failed to fetch results from dynamodb".to_string(),
        ))?
        .remove(&schema_table_name)
        .ok_or(GraphMergerError::Unexpected(
            "Missing data from expected table in dynamodb".to_string(),
        ))?;

    /*
       1. Remove entries without f_edge and r_edge
       2. Grab both f_edge and r_edge properties as strings (`.s`)
       3. If both are present, return; otherwise error and filter entry out
    */
    Ok(schema_table_response
        .into_iter()
        .filter(|hashmap| hashmap.contains_key("f_edge") && hashmap.contains_key("r_edge"))
        .filter_map(|hashmap| {
            match (
                hashmap.get("f_edge").map(|item| item.s.clone()),
                hashmap.get("r_edge").map(|item| item.s.clone()),
            ) {
                (Some(Some(f_edge)), Some(r_edge)) => Some((f_edge, r_edge)),
                (Some(Some(f_edge)), _) => {
                    tracing::error!(
                        message="Missing r_edge for f_edge in dynamodb schema.",
                        f_edge=?f_edge,
                    );
                    None
                }
                (None, Some(Some(r_edge))) => {
                    tracing::error!(
                        message="Failed to associate retrieved r_edge with an f_edge",
                        r_edge=?r_edge,
                    );
                    None
                }
                _ => None,
            }
        })
        .collect())
}

fn make_keys(f_edges: &[&String]) -> KeysAndAttributes {
    let keys: Vec<HashMap<String, AttributeValue>> = f_edges
        .iter()
        .map(|f_edge| {
            let mut key_map = HashMap::new();
            key_map.insert(
                "f_edge".to_string(),
                AttributeValue {
                    s: Some(f_edge.to_string()),
                    ..Default::default()
                },
            );

            key_map
        })
        .collect();

    KeysAndAttributes {
        consistent_read: Some(true),
        keys,
        ..Default::default()
    }
}
