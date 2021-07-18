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
    metrics::SysmonSubgraphGeneratorMetrics,
    models::SysmonTryFrom,
};

#[derive(thiserror::Error, Debug)]
pub enum SysmonGeneratorError {
    #[error("NegativeEventTime")]
    NegativeEventTime(i64),
    #[error("TimeError")]
    TimeError(#[from] chrono::ParseError),
    #[error("Unsupported event type")]
    UnsupportedEventType(String),
}

impl CheckedError for SysmonGeneratorError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::NegativeEventTime(_) => Recoverable::Persistent,
            Self::TimeError(_) => Recoverable::Persistent,
            Self::UnsupportedEventType(_) => Recoverable::Persistent,
        }
    }
}

#[derive(Clone)]
pub struct SysmonSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    cache: C,
    metrics: SysmonSubgraphGeneratorMetrics,
}

impl<C> SysmonSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    pub fn new(cache: C, metrics: SysmonSubgraphGeneratorMetrics) -> Self {
        Self { cache, metrics }
    }
}

#[async_trait]
impl<C> EventHandler for SysmonSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    type InputEvent = Vec<sysmon::Event>;
    type OutputEvent = GraphDescription;
    type Error = SysmonGeneratorError;

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

        let mut last_error: Option<SysmonGeneratorError> = None;

        let subgraphs: Vec<_> = events
            .into_iter()
            .filter_map(|event| {
                let result = GraphDescription::try_from(event.clone());
                self.metrics.report_subgraph_generation(&result);
                match result {
                    Ok(graph) => {
                        completed.add_identity(event, EventStatus::Success);
                        Some(graph)
                    }
                    Err(error) => {
                        completed.add_identity(event, EventStatus::Failure);
                        tracing::error!(message="GraphDescription::try_from failed with.", error=?error);
                        last_error = Some(error);
                        None
                    }
                }
            })
            .collect();

        let final_subgraph =
            subgraphs
                .iter()
                .fold(GraphDescription::new(), |mut current_graph, subgraph| {
                    current_graph.merge(&subgraph);
                    current_graph
                });

        tracing::info!(
            message = "Completed mapping subgraphs.",
            num_graphs = completed.len()
        );

        let final_result = match (last_error, subgraphs.is_empty()) {
            (None, _) => Ok(final_subgraph),
            (Some(error), false) => Err(Ok((final_subgraph, error))),
            (Some(error), true) => Err(Err(error)),
        };

        self.metrics.report_handle_event_success(&final_result);

        final_result
    }
}
