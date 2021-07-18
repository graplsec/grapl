#[cfg(feature = "integration")]
mod integration_tests {
    use kafka_metrics_exporter::KafkaMetricExporterBuilder;
    use metrics::{
        counter,
        histogram,
    };
    use rdkafka::{
        config::{
            FromClientConfig,
            RDKafkaLogLevel,
        },
        consumer::{
            stream_consumer::StreamConsumer,
            CommitMode,
            Consumer,
            DefaultConsumerContext,
        },
        producer::FutureProducer,
        util::DefaultRuntime,
    };
    use tokio_stream::StreamExt;

    fn producer_init() -> Result<FutureProducer, Box<dyn std::error::Error>> {
        let brokers = "kafka-broker:9092";
        let mut client_config = rdkafka::ClientConfig::new();
        client_config
            .set("client.id", "test-producer")
            .set("queue.buffering.max.ms", "0")
            .set("bootstrap.servers", brokers);
        tracing::info!(config=?client_config, message="Created Producer ClientConfig");

        let producer: FutureProducer = FutureProducer::from_config(&client_config)?;
        Ok(producer)
    }

    fn consumer_init(
    ) -> Result<StreamConsumer<DefaultConsumerContext, DefaultRuntime>, Box<dyn std::error::Error>>
    {
        let brokers = "kafka-broker:9092";
        let mut client_config = rdkafka::ClientConfig::new();
        client_config
            .set("group.id", "integration-tests-consumers")
            .set("client.id", "test-consumer")
            .set("bootstrap.servers", brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("max.poll.interval.ms", "10000")
            .set("request.timeout.ms", "1000")
            .set("auto.offset.reset", "earliest")
            .set_log_level(RDKafkaLogLevel::Debug);
        tracing::info!(config=?client_config, message="Created Consumer ClientConfig");

        let consumer = StreamConsumer::from_config(&client_config)?;
        consumer
            .subscribe(&["metrics"])
            .expect("Can't subscribe to specified topic");
        Ok(consumer)
    }

    #[tracing::instrument]
    #[tokio::test]
    async fn smoketest() -> Result<(), Box<dyn std::error::Error>> {
        let subscriber = ::tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(::tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .finish();
        let _ = ::tracing::subscriber::set_global_default(subscriber);

        tracing::info!(topic_name = "metrics", message = "Starting smoketest");
        let producer: FutureProducer = producer_init()?;
        KafkaMetricExporterBuilder::new("metrics", producer).install()?;
        tracing::info!(topic_name = "metrics", message = "Created producer");

        histogram!("process.query_time", 1234f64);
        counter!("process.query_row_count", 1000);

        let consumer = consumer_init()?;
        tracing::info!(message = "Creating stream");
        let mut stream = consumer.stream();
        let metric_0 = tokio::time::timeout(std::time::Duration::from_secs(3), stream.next())
            .await?
            .expect("metric_0")?;
        let metric_1 = tokio::time::timeout(std::time::Duration::from_secs(3), stream.next())
            .await?
            .expect("metric_1")?;

        consumer
            .commit_message(&metric_0, CommitMode::Sync)
            .expect("commit_message");
        consumer
            .commit_message(&metric_1, CommitMode::Sync)
            .expect("commit_message");
        Ok(())
    }
}
