#![allow(unused_must_use)]

use std::fmt::Debug;

pub use grapl_config;
use grapl_config::{
    event_caches,
    ServiceEnv,
};
pub use grapl_observe::metric_reporter::MetricReporter;
use grapl_service::serialization::GraphDescriptionSerializer;
use rusoto_s3::S3Client;
use rusoto_sqs::SqsClient;
pub use rust_proto::graph_descriptions::*;
use sqs_executor::{
    errors::CheckedError,
    event_decoder::PayloadDecoder,
    event_handler::EventHandler,
    make_ten,
    redis_cache::RedisCache,
    s3_event_retriever::S3PayloadRetriever,
    time_based_key_fn,
};
use tracing::info;

use crate::grapl_config::env_helpers::{
    s3_event_emitters_from_env,
    FromEnv,
};

pub async fn run_graph_generator<
    InputEventT,
    HandlerErrorT,
    InitGenerator,
    PayloadDecoderT,
    DecoderErrorT,
    EventHandlerT,
>(
    env: ServiceEnv,
    init_generator: InitGenerator,
    payload_decoder: PayloadDecoderT,
) where
    InputEventT: Send + 'static,
    InitGenerator: Clone + Send + 'static + Fn(RedisCache) -> EventHandlerT,
    EventHandlerT: EventHandler<
            InputEvent = InputEventT,
            OutputEvent = GraphDescription,
            Error = HandlerErrorT,
        >
        + Send
        + Sync
        + 'static
        + Clone,
    HandlerErrorT: Debug + CheckedError + Send + Sync + 'static,
    PayloadDecoderT:
        PayloadDecoder<InputEventT, DecoderError = DecoderErrorT> + Send + Sync + Clone + 'static,
    DecoderErrorT: CheckedError + Send + 'static,
{
    let sqs_client = SqsClient::from_env();
    let _s3_client = S3Client::from_env();
    let cache = &mut event_caches(&env).await;

    let subgraph_generator = &mut make_ten(async { (init_generator)(cache[0].clone()) }).await;

    let serializer = &mut make_ten(async { GraphDescriptionSerializer::default() }).await;

    let s3_emitter = &mut s3_event_emitters_from_env(&env, time_based_key_fn).await;

    let s3_payload_retriever = &mut make_ten(async {
        S3PayloadRetriever::new(
            |region_str| {
                info!("Initializing new s3 client: {}", &region_str);
                grapl_config::env_helpers::init_s3_client(&region_str)
            },
            payload_decoder,
            MetricReporter::new(&env.service_name),
        )
    })
    .await;

    info!("Starting process_loop");
    let queue_url = grapl_config::source_queue_url();

    sqs_executor::process_loop(
        queue_url,
        std::env::var("DEAD_LETTER_QUEUE_URL").expect("DEAD_LETTER_QUEUE_URL"),
        cache,
        sqs_client.clone(),
        subgraph_generator,
        s3_payload_retriever,
        s3_emitter,
        serializer,
        MetricReporter::new(&env.service_name),
    )
    .await;
}
