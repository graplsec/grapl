use std::sync::Arc;

use dgraph_tonic::{
    Client as DgraphClient,
    Mutate,
};
use futures_retry::{
    FutureRetry,
    RetryPolicy,
};
use grapl_graph_descriptions::{
    Edge,
    IdentifiedNode,
};

use crate::mutations::{
    edge_mutation::EdgeUpsertGenerator,
    node_mutation::NodeUpsertGenerator,
};

#[derive(thiserror::Error, Debug)]
pub enum UpsertManagerError {
    // #[error("NodeUpstertError")]
    // NodeUpsertError(#[from] NodeUpstertError)
    #[error("Failed to decode edge upsert response")]
    UnexpectedDgraphResponseJson(#[from] serde_json::Error),
    #[error("InvalidUid")]
    InvalidUid(#[from] InvalidUid),
    #[error("DgraphError")]
    DgraphError(#[from] anyhow::Error),
}

pub struct UpsertManager {
    pub dgraph_client: Arc<DgraphClient>,
    pub node_upsert_generator: NodeUpsertGenerator,
    pub edge_upsert_generator: EdgeUpsertGenerator,
}

impl UpsertManager {
    pub async fn upsert_node(&mut self, node: &IdentifiedNode) -> Result<u64, UpsertManagerError> {
        let (creation_var_name, query, mutations) = self
            .node_upsert_generator
            .generate_upserts(0u128, 0u128, node);

        let combined_query = format!(
            r"
            {{
                {query}
            }}
        ",
            query = query
        );

        let dgraph_client = self.dgraph_client.clone();
        let mutations = mutations.to_vec();
        let res = enforce_transaction(move || {
            let txn = dgraph_client.new_mutated_txn();
            txn.upsert_and_commit_now(combined_query.clone(), mutations.clone())
        })
        .await?;

        Ok(extract_uid(&creation_var_name, &res)?)
    }

    pub async fn upsert_edge(
        &mut self,
        forward_edge: Edge,
        reverse_edge: Edge,
    ) -> Result<(u64, u64), UpsertManagerError> {
        let (query, mutations) = self
            .edge_upsert_generator
            .generate_upserts(&forward_edge, &reverse_edge);
        let dgraph_client = self.dgraph_client.clone();
        let mutations = mutations.to_vec();
        let query = query.to_string();
        let res = enforce_transaction(move || {
            let txn = dgraph_client.new_mutated_txn();
            txn.upsert_and_commit_now(query.clone(), mutations.clone())
        })
        .await?;

        let j: UidMap = serde_json::from_slice(&res.json)?;
        let (from_uid, to_uid) = if j.uidmap[0].node_key == forward_edge.from_node_key {
            (&j.uidmap[0].uid, &j.uidmap[1].uid)
        } else {
            (&j.uidmap[1].uid, &j.uidmap[0].uid)
        };

        let from_uid = uid_from_str(from_uid)?;
        let to_uid = uid_from_str(to_uid)?;

        Ok((from_uid, to_uid))
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct UidAndNodeKey {
    node_key: String,
    uid: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct UidMap {
    uidmap: Vec<UidAndNodeKey>,
}

#[derive(thiserror::Error, Debug)]
pub enum InvalidUid {
    #[error("Encoded uid is too short")]
    TooShort(usize),
    #[error("Failed to parse uid from base 16 string")]
    UidParseError(#[from] std::num::ParseIntError),
}

fn uid_from_str(hex_encoded: &str) -> Result<u64, InvalidUid> {
    if hex_encoded.len() < 2 {
        return Err(InvalidUid::TooShort(hex_encoded.len()));
    }

    Ok(u64::from_str_radix(&hex_encoded[2..], 16)?)
}

fn extract_uid(
    creation_var_name: &str,
    res: &dgraph_tonic::Response,
) -> Result<u64, UpsertManagerError> {
    let uid = res.uids.get(creation_var_name);
    match uid {
        Some(uid) => Ok(uid_from_str(&uid)?),
        None => {
            let creation_var_name = format!("q_{}", creation_var_name);
            let v: serde_json::Value = serde_json::from_slice(&res.json)?;

            let uid: String = serde_json::from_value(v[creation_var_name][0]["uid"].clone())?;
            Ok(uid_from_str(&uid)?)
        }
    }
}

async fn enforce_transaction<Factory, Txn>(
    f: Factory,
) -> Result<dgraph_tonic::Response, anyhow::Error>
where
    Factory: FnMut() -> Txn + 'static + Unpin,
    Txn: std::future::Future<Output = Result<dgraph_tonic::Response, anyhow::Error>>,
{
    let handle_upsert_err = UpsertErrorHandler {};
    match FutureRetry::new(f, handle_upsert_err).await {
        Ok((response, attempts)) => {
            tracing::info!(message = "Performed upsert", attempts = attempts);
            Ok(response)
        }
        Err((response, attempts)) => {
            tracing::warn!(message = "Failed to perform upsert", attempts = attempts, error=?response);
            Err(response)
        }
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
