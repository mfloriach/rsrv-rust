use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use rsv::kafka::ReservationExpired;
use rsv::queues::{EventConsumer, KafkaConfig, MessageHandler};
use rsv::workers::reservation_expiration_worker;

struct MessagePrinter {}

impl MessagePrinter {
    fn new() -> Box<Self> {
        Box::new(MessagePrinter {})
    }
}

#[async_trait]
impl MessageHandler for MessagePrinter {
    async fn handle(&self, key: &[u8], payload: &[u8]) -> Result<()> {
        println!("Key: {}", String::from_utf8_lossy(key));
        println!("Payload: {}", String::from_utf8_lossy(payload));

        let event: ReservationExpired = serde_json::from_slice(payload)?;

        println!("{}", event.occurred_at - Utc::now());

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let consumer = EventConsumer::new(
        KafkaConfig {
            brokers: "kafka:9092".to_string(),
            topic: "reservation.delay".to_string(),
            group_id: "kafka-streaming-group".to_string(),
            timeout_ms: 50000000,
            max_retries: 3,
        },
        MessagePrinter::new(),
    )
    .expect("asdsadas");

    consumer.start().await.expect("dsfdsfds");
}
