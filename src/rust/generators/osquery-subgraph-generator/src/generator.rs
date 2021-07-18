use async_trait::async_trait;
use rust_proto::graph_descriptions::*;
use sqs_executor::{
    cache::Cache,
    errors::{
        CheckedError,
        Recoverable,
    },
    event_handler::{
        CompletedEvents,
        EventHandler,
    },
    event_status::EventStatus,
};

use crate::{
    metrics::OSQuerySubgraphGeneratorMetrics,
    parsers::OSQueryEvent,
};

#[derive(Clone)]
pub struct OSQuerySubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    cache: C,
    metrics: OSQuerySubgraphGeneratorMetrics,
}

impl<C> OSQuerySubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    pub fn new(cache: C, metrics: OSQuerySubgraphGeneratorMetrics) -> Self {
        Self { cache, metrics }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum OSQuerySubgraphGeneratorError {}

impl CheckedError for OSQuerySubgraphGeneratorError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Persistent
    }
}

#[async_trait]
impl<C: Cache> EventHandler for OSQuerySubgraphGenerator<C> {
    type InputEvent = Vec<OSQueryEvent>;
    type OutputEvent = GraphDescription;
    type Error = OSQuerySubgraphGeneratorError;

    #[tracing::instrument(skip(self, events, completed))]
    async fn handle_event(
        &mut self,
        events: Self::InputEvent,
        completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        tracing::info!(
            message = "Processing incoming events.",
            num_events = events.len()
        );

        // Skip events we've successfully processed and stored in the event cache.
        let events = self.cache.filter_cached(&events).await;

        let final_subgraph = events
            .into_iter()
            .map(|event| {
                completed.add_identity(&event, EventStatus::Success);
                GraphDescription::from(event)
            })
            .fold(GraphDescription::new(), |mut current_graph, subgraph| {
                current_graph.merge(&subgraph);
                current_graph
            });

        tracing::info!(
            message = "Completed mapping subgraphs",
            num_completed = completed.len()
        );
        self.metrics.report_subgraph_generation();

        Ok(final_subgraph)
    }
}
