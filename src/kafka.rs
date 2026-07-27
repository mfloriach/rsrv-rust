use anyhow::Result;
use chrono::{DateTime, Utc};
use rdkafka::consumer::Consumer;
use rdkafka::producer::FutureProducer;
use rdkafka::{config::ClientConfig, consumer::StreamConsumer};
use serde::Serialize;
use uuid::Uuid;

pub fn create_producer() -> Result<FutureProducer> {
    Ok(ClientConfig::new().set("bootstrap.servers", "localhost:29092").create()?)
}

pub async fn create_consumer() -> StreamConsumer {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "kafka:9092")
        .set("group.id", "reservation-worker")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Failed to create consumer");

    consumer.subscribe(&["reservation-events"]).expect("dsfd");

    return consumer;
}

#[derive(serde::Deserialize, Debug, Serialize)]
pub struct ReservationExpired {
    pub reservation_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}
