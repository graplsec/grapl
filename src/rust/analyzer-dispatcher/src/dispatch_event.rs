use prost::Message;
use rust_proto::{
    graph_descriptions::*,
    services::ServiceMessage,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use sqs_executor::completion_event_serializer::CompletionEventSerializer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerDispatchEvent {
    key: String,
    subgraph: MergedGraph,
}

impl AnalyzerDispatchEvent {
    pub fn new(key: String, subgraph: MergedGraph) -> Self {
        Self { key, subgraph }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerDispatchEvents {
    events: Vec<AnalyzerDispatchEvent>,
}

impl AnalyzerDispatchEvents {
    pub fn new() -> Self {
        Self { events: vec![] }
    }
}

impl From<Vec<AnalyzerDispatchEvent>> for AnalyzerDispatchEvents {
    fn from(events: Vec<AnalyzerDispatchEvent>) -> Self {
        Self { events }
    }
}

impl ServiceMessage for AnalyzerDispatchEvents {
    const TYPE_NAME: &'static str = "AnalyzerDispatchEvent";
}

#[derive(thiserror::Error, Debug)]
pub enum DispatchEventEncoderError {
    #[error("IO")]
    Io(#[from] std::io::Error),
    #[error("EncodeError")]
    JsonEncodeError(#[from] serde_json::Error),
    #[error("ProtoEncodeError")]
    ProtoEncodeError(#[from] prost::EncodeError),
}

#[derive(Clone, Debug, Default)]
pub struct AnalyzerDispatchSerializer {
    proto: Vec<u8>,
}

impl CompletionEventSerializer for AnalyzerDispatchSerializer {
    type CompletedEvent = AnalyzerDispatchEvents;
    type Output = Vec<u8>;
    type Error = DispatchEventEncoderError;

    fn serialize_completed_events(
        &mut self,
        completed_events: &[Self::CompletedEvent],
    ) -> Result<Vec<Self::Output>, Self::Error> {
        let unique_events: Vec<_> = completed_events
            .iter()
            .map(|e| &e.events)
            .flatten()
            .collect();

        let mut final_subgraph = MergedGraph::new();

        for event in unique_events.iter() {
            final_subgraph.merge(&event.subgraph);
        }

        let mut serialized = Vec::with_capacity(unique_events.len());

        let mut buf = Vec::with_capacity(5000);
        final_subgraph.encode(&mut buf)?;

        for event in unique_events {
            let event = json!({
                "key": event.key,
                "subgraph": buf,
            });

            serialized.push(serde_json::to_vec(&event)?);
        }

        Ok(serialized)
    }
}
