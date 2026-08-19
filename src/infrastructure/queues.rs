use anyhow::Result;
use async_trait::async_trait;
use derive_builder::Builder;
use futures::StreamExt;
use rdkafka::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::Producer;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use thiserror::Error;
use validator::Validate;

#[derive(Debug, Error)]
pub enum KafkaError {
    #[error("Failed to create Kafka client: {0}")]
    ClientCreation(String),

    #[error("Failed to send message: {0}")]
    MessageSend(String),

    #[error("Message delivery failed: {0}")]
    MessageDelivery(String),

    #[error("Failed to deserialize message: {0}")]
    Deserialization(#[from] serde_json::Error),

    #[error("Connection timeout: {0}")]
    Timeout(String),

    #[error("Max retries exceeded")]
    MaxRetries,
}

#[derive(Debug, Clone, Builder, Validate)]
#[builder(setter(into), build_fn(error = "String"))]
pub struct KafkaConfig {
    #[builder(default = "\"127.0.0.1:29092\".to_string()")]
    pub brokers: String,

    #[builder(default = "\"events\".to_string()")]
    pub topic: String,

    #[builder(default = "\"kafka-streaming-group\".to_string()")]
    pub group_id: String,

    #[builder(default = "5000")]
    #[validate(range(
        min = 5000,
        max = 30000,
        message = "Timeout must be between 5000 and 30000 milliseconds"
    ))]
    pub timeout_ms: u64,

    #[builder(default = "5")]
    #[validate(range(min = 1, max = 5, message = "Max retries must be between 1 and 5"))]
    pub max_retries: u32,
}

impl KafkaConfig {
    #[must_use]
    pub fn builder() -> KafkaConfigBuilder {
        KafkaConfigBuilder::default()
    }
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self::builder().build().expect("all Kafka configuration fields have defaults")
    }
}

#[derive(Clone)]
pub struct EventProducer {
    producer: FutureProducer,
    config: KafkaConfig,
}

impl EventProducer {
    pub fn new(config: KafkaConfig) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.brokers)
            .set("message.timeout.ms", config.timeout_ms.to_string())
            .set("compression.type", "gzip")
            .set("retry.backoff.ms", "500")
            .set("request.required.acks", "all")
            .set("queue.buffering.max.messages", "100000")
            .create()
            .map_err(|e| KafkaError::ClientCreation(e.to_string()))?;

        Ok(EventProducer { producer, config })
    }

    pub async fn send_event<K, V>(&self, key: K, payload: V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let record =
            FutureRecord::to(&self.config.topic).payload(payload.as_ref()).key(key.as_ref());

        self.producer
            .send(record, Duration::from_millis(self.config.timeout_ms))
            .await
            .map_err(|(err, _)| KafkaError::MessageSend(err.to_string()))?;

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        self.producer.flush(Duration::from_secs(10))?;
        Ok(())
    }
}

#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, key: &[u8], payload: &[u8]) -> Result<()>;
}

pub struct EventConsumer {
    consumer: StreamConsumer,
    handler: Box<dyn MessageHandler>,
    config: KafkaConfig,
}

impl EventConsumer {
    pub fn new(config: KafkaConfig, handler: Box<dyn MessageHandler>) -> Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &config.brokers)
            .set("group.id", &config.group_id)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "6000")
            .set("max.poll.interval.ms", "300000")
            .create()
            .map_err(|e| KafkaError::ClientCreation(e.to_string()))?;

        consumer
            .subscribe(&[&config.topic])
            .map_err(|e| KafkaError::ClientCreation(e.to_string()))?;

        Ok(EventConsumer { consumer, handler, config })
    }

    pub async fn start(&self) -> Result<()> {
        let mut message_stream = self.consumer.stream();

        while let Some(message_result) = message_stream.next().await {
            match message_result {
                Ok(message) => {
                    let key = message.key().unwrap_or_default();
                    let payload = message.payload().unwrap_or_default();

                    match self.process_with_retry(key, payload).await {
                        Ok(_) => {
                            self.consumer
                                .commit_message(&message, CommitMode::Async)
                                .map_err(|e| KafkaError::MessageDelivery(e.to_string()))?;
                        }
                        Err(e) => {
                            tracing::error!("Failed to process message: {}", e);
                            // Implement dead letter queue logic here
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error receiving message: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }

        Ok(())
    }

    async fn process_with_retry(&self, key: &[u8], payload: &[u8]) -> Result<()> {
        let mut retries = 0;
        let mut backoff = Duration::from_millis(100);

        while retries < self.config.max_retries {
            match self.handler.handle(key, payload).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    tracing::warn!("Retry {} failed: {}", retries, e);
                    retries += 1;
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;
                }
            }
        }

        Err(KafkaError::MaxRetries.into())
    }
}

#[cfg(test)]
mod tests {
    use super::KafkaConfig;

    #[test]
    fn kafka_config_builder_uses_safe_defaults() {
        let config = KafkaConfig::builder().build().expect("all fields have defaults");

        assert_eq!(config.brokers, "127.0.0.1:29092");
        assert_eq!(config.topic, "events");
        assert_eq!(config.group_id, "kafka-streaming-group");
        assert_eq!(config.timeout_ms, 5_000);
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn kafka_config_builder_overrides_client_settings() {
        let config = KafkaConfig::builder()
            .brokers("kafka:9092")
            .topic("reservation.expire")
            .group_id("reservation-expiring")
            .timeout_ms(10_000_u64)
            .max_retries(3_u32)
            .build()
            .expect("all fields have defaults");

        assert_eq!(config.brokers, "kafka:9092");
        assert_eq!(config.topic, "reservation.expire");
        assert_eq!(config.group_id, "reservation-expiring");
        assert_eq!(config.timeout_ms, 10_000);
        assert_eq!(config.max_retries, 3);
    }

    // fn assert_kafka_client<T: KafkaClient>() {}

    // #[test]
    // fn producer_and_consumer_share_a_common_client_trait() {
    //     assert_kafka_client::<super::EventProducer>();
    //     assert_kafka_client::<super::EventConsumer>();
    // }
}
