use std::{
    collections::HashMap,
    io::Stdout,
    marker::PhantomData,
    time::Duration,
};

use async_trait::async_trait;
use futures::FutureExt;
use grapl_observe::{
    metric_reporter::{
        tag,
        HistogramUnit,
        MetricReporter,
    },
    timers::{
        time_it,
        TimedFutureExt,
    },
};
use rusoto_core::RusotoError;
use rusoto_s3::{
    GetObjectError,
    GetObjectRequest,
    S3,
};
use rusoto_sqs::Message as SqsMessage;
use rust_proto::services::Metadata;
use tokio::{
    io::AsyncReadExt,
    time::error::Elapsed,
};
use tracing::{
    debug,
    error,
};

use crate::{
    errors::{
        CheckedError,
        Recoverable,
    },
    event_decoder::PayloadDecoder,
    PayloadRetriever,
};

pub struct S3PayloadRetriever<S, SInit, D, E, DecoderErrorT>
where
    S: S3 + Clone + Send + Sync + 'static,
    SInit: (Fn(String) -> S) + Clone + Send + Sync + 'static,
    D: PayloadDecoder<E, DecoderError = DecoderErrorT> + Clone + Send + 'static,
    DecoderErrorT: CheckedError + Send + 'static,
    E: Send + 'static,
{
    s3_init: SInit,
    s3_clients: HashMap<String, S>,
    decoder: D,
    metric_reporter: MetricReporter<Stdout>,
    phantom: PhantomData<(E, DecoderErrorT)>,
}

impl<S, SInit, D, E, DecoderErrorT> Clone for S3PayloadRetriever<S, SInit, D, E, DecoderErrorT>
where
    S: S3 + Clone + Send + Sync + 'static,
    SInit: (Fn(String) -> S) + Clone + Send + Sync + 'static,
    D: PayloadDecoder<E, DecoderError = DecoderErrorT> + Clone + Send + 'static,
    DecoderErrorT: CheckedError + Send + 'static,
    E: Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            s3_init: self.s3_init.clone(),
            s3_clients: self.s3_clients.clone(),
            decoder: self.decoder.clone(),
            metric_reporter: self.metric_reporter.clone(),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<S, SInit, D, E, DecoderErrorT> S3PayloadRetriever<S, SInit, D, E, DecoderErrorT>
where
    S: S3 + Clone + Send + Sync + 'static,
    SInit: (Fn(String) -> S) + Clone + Send + Sync + 'static,
    D: PayloadDecoder<E, DecoderError = DecoderErrorT> + Clone + Send + 'static,
    DecoderErrorT: CheckedError + Send + 'static,
    E: Send + 'static,
{
    pub fn new(s3: SInit, decoder: D, metric_reporter: MetricReporter<Stdout>) -> Self {
        Self {
            s3_init: s3,
            s3_clients: HashMap::new(),
            decoder,
            metric_reporter,
            phantom: PhantomData,
        }
    }

    pub fn get_client(&mut self, region: String) -> S {
        match self.s3_clients.get(&region) {
            Some(s3) => s3.clone(),
            None => {
                let client = (self.s3_init)(region.clone());
                self.s3_clients.insert(region.to_string(), client.clone());
                client
            }
        }
    }
}

// PayloadDecoder

#[derive(thiserror::Error, Debug)]
pub enum S3PayloadRetrieverError<DecoderErrorT>
where
    DecoderErrorT: CheckedError + 'static,
{
    #[error("S3Error: {0}")]
    S3Error(#[from] RusotoError<GetObjectError>),
    #[error("Decode error")]
    DecodeError(#[from] DecoderErrorT),
    #[error("EnvelopeDecode error")]
    EnvelopeDecode(#[from] prost::DecodeError),
    #[error("IO")]
    Io(#[from] std::io::Error),
    #[error("JSON")]
    Json(#[from] serde_json::Error),
    #[error("Timeout")]
    Timeout(#[from] Elapsed),
}

impl<DecoderErrorT> CheckedError for S3PayloadRetrieverError<DecoderErrorT>
where
    DecoderErrorT: CheckedError + 'static,
{
    fn error_type(&self) -> Recoverable {
        match self {
            Self::S3Error(_) => Recoverable::Transient,
            Self::DecodeError(_) => Recoverable::Persistent,
            Self::EnvelopeDecode(_) => Recoverable::Persistent,
            Self::Io(_) => Recoverable::Transient,
            Self::Json(_) => Recoverable::Persistent,
            Self::Timeout(_) => Recoverable::Transient,
        }
    }
}

#[async_trait]
impl<S, SInit, D, E, DecoderErrorT> PayloadRetriever<E>
    for S3PayloadRetriever<S, SInit, D, E, DecoderErrorT>
where
    S: S3 + Clone + Send + Sync + 'static,
    SInit: (Fn(String) -> S) + Clone + Send + Sync + 'static,
    D: PayloadDecoder<E, DecoderError = DecoderErrorT> + Clone + Send + 'static,
    E: Send + 'static,
    DecoderErrorT: CheckedError + Send + 'static,
{
    type Message = SqsMessage;
    type Error = S3PayloadRetrieverError<DecoderErrorT>;

    #[tracing::instrument(skip(self, msg))]
    async fn retrieve_event(
        &mut self,
        msg: &Self::Message,
    ) -> Result<Option<(Metadata, E)>, Self::Error> {
        let body = msg.body.as_ref().unwrap();
        debug!("Got body from message: {}", body);
        let event: serde_json::Value = serde_json::from_str(body)?;

        if let Some(Some(event_str)) = event.get("Event").map(serde_json::Value::as_str) {
            if event_str == "s3:TestEvent" {
                return Ok(None);
            }
        }
        let record = &event["Records"][0]["s3"];

        let bucket = record["bucket"]["name"].as_str().expect("bucket name");
        let key = record["object"]["key"].as_str().expect("object key");

        let inner_loop_span = tracing::trace_span!(
            "s3.retrieve_event",
            bucket=?bucket,
            key=?key,
        );
        let _enter = inner_loop_span.enter();

        debug!(message = "Retrieving S3 payload",);

        let region = &event["Records"][0]["awsRegion"].as_str().expect("region");
        let s3 = self.get_client(region.to_string());
        let s3_data = s3.get_object(GetObjectRequest {
            bucket: bucket.to_string(),
            key: key.to_string(),
            ..Default::default()
        });

        let s3_data = tokio::time::timeout(Duration::from_secs(3), s3_data);
        let s3_data = s3_data
            .timed()
            .map(|(s3_data, ms)| {
                self.metric_reporter
                    .histogram(
                        "s3_consumer.get_object.ms",
                        ms as f64,
                        &[tag("success", s3_data.is_ok())],
                    )
                    .unwrap_or_else(
                        |e| error!(message="failed to report s3_consumer.get_object.ms", error=?e),
                    );
                s3_data
            })
            .await??;

        let object_size = record["object"]["size"].as_u64().unwrap_or_default();
        let prealloc = if object_size < 1024 {
            1024
        } else {
            object_size as usize
        };

        debug!(message="Retrieved s3 payload", object_size=?object_size);

        let mut body = Vec::with_capacity(prealloc);

        s3_data
            .body
            .expect("Missing S3 body")
            .into_async_read()
            .read_to_end(&mut body)
            .timed()
            .map(|(res, ms)| {
                self.metric_reporter
                    .histogram(
                        "s3_consumer.read_to_end.ms",
                        ms as f64,
                        &[tag("success", res.is_ok())],
                    )
                    .unwrap_or_else(
                        |e| error!(message="failed to report s3_consumer.read_to_end.ms", error=?e),
                    );
                res
            })
            .await?;

        self.metric_reporter
            .gauge("s3_retriever.bytes", body.len() as f64, &[])
            .unwrap_or_else(|e| error!(message="failed to report s3_retriever.bytes", error=?e));

        debug!("Read s3 payload body");

        let envelope: rust_proto::services::Envelope = prost::Message::decode(&body[..])?;
        let meta = envelope
            .metadata
            .expect("Metadata must be set at the front of the pipeline");
        let body = envelope.inner_message;

        let (decoded, ms) = time_it(|| self.decoder.decode(body));

        self.metric_reporter
            .histogram_with_units(
                "s3_retriever.decoded.micros",
                ms.as_micros() as f64,
                HistogramUnit::Micros,
                &[tag("success", true)][..],
            )
            .unwrap_or_else(
                |e| error!(message="failed to report s3_retriever.decoded.micros", error=?e),
            );

        Ok(Some((meta, decoded?)))
    }
}
