use anyhow::{Result, bail};
use async_trait::async_trait;
use futures::StreamExt;
use rdkafka::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use thiserror::Error;

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
}

#[derive(Debug, Clone)]
pub struct KafkaConfig {
    pub brokers: String,
    pub topic: String,
    pub group_id: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            brokers: "127.0.0.1:29092".to_string(),
            topic: "events".to_string(),
            group_id: "kafka-streaming-group".to_string(),
            timeout_ms: 5000,
            max_retries: 5,
        }
    }
}

#[derive(Clone)]
pub struct EventProducer {
    producer: FutureProducer,
    topic: String,
    timeout: Duration,
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

        Ok(EventProducer {
            producer,
            topic: config.topic,
            timeout: Duration::from_secs(config.timeout_ms / 1000),
        })
    }

    pub async fn send_event<K, V>(&self, key: K, payload: V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let record = FutureRecord::to(&self.topic).payload(payload.as_ref()).key(key.as_ref());

        self.producer
            .send(record, self.timeout)
            .await
            .map_err(|(err, _)| KafkaError::MessageSend(err.to_string()))?;

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
    max_retries: u32,
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

        Ok(EventConsumer { consumer, handler, max_retries: config.max_retries })
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

        while retries < self.max_retries {
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

        bail!("Max retries exceeded")
    }
}
