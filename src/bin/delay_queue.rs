use anyhow::Result;
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use chrono::Utc;
use rsv::infrastructure::queues::{EventConsumer, EventProducer, KafkaConfig, MessageHandler};
use rsv::services::ReservationExpired;
use rsv::workers::delay_queues::time_wheel::TimingWheel;
use std::env;
use std::time::Duration;

struct MessagePrinter {
    tx: Sender<ReservationExpired>,
}

impl MessagePrinter {
    fn new(tx: Sender<ReservationExpired>) -> Box<Self> {
        Box::new(MessagePrinter { tx })
    }
}

#[async_trait]
impl MessageHandler for MessagePrinter {
    async fn handle(&self, _key: &[u8], payload: &[u8]) -> Result<()> {
        let event: ReservationExpired = serde_json::from_slice(payload)?;
        self.tx.send(event).await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    let (tx, rx) = async_channel::bounded::<ReservationExpired>(100_000);

    let ingestor_handle = tokio::spawn(async move { ingestor(tx).await });

    let producer = producer().unwrap();
    let mut handles = Vec::new();
    for i in 0..8 {
        handles.push(tokio::spawn(worker(i as usize, rx.clone(), producer.clone())));
    }

    let _ = ingestor_handle.await.unwrap();

    for handle in handles {
        handle.await.unwrap();
    }
}

fn producer() -> Result<EventProducer> {
    let brokers = env::var("KAFKA_BROKER_PRODUCER").unwrap();
    let topic = env::var("KAFKA_TOPIC_PRODUCER").unwrap();
    let group_id = env::var("KAFKA_GROUP_ID_PRODUCER").unwrap();

    EventProducer::new(KafkaConfig {
        brokers,
        topic,
        group_id,
        timeout_ms: 50000000,
        max_retries: 3,
    })
}

async fn ingestor(tx: Sender<ReservationExpired>) -> Result<()> {
    let brokers = env::var("KAFKA_BROKER_CONSUMER").unwrap();
    let topic = env::var("KAFKA_TOPIC_CONSUMER").unwrap();
    let group_id = env::var("KAFKA_GROUP_ID_CONSUMER").unwrap();

    let consumer = EventConsumer::new(
        KafkaConfig { brokers, topic, group_id, timeout_ms: 50000000, max_retries: 3 },
        MessagePrinter::new(tx),
    )?;

    consumer.start().await?;

    Ok(())
}

async fn worker(id: usize, rx: Receiver<ReservationExpired>, producer: EventProducer) {
    tracing::info!("Worker {id} started");

    let mut wheel = TimingWheel::new(1 * 60);
    let mut ticker = tokio::time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            Ok(reservation) = rx.recv() => {
                let delay = (reservation.expired_at - Utc::now())
                    .to_std()
                    .unwrap_or(Duration::ZERO);

                wheel.add(reservation.reservation_id, delay);
            }

            _ = ticker.tick() => {
                let expired_at = wheel.tick();
                for reservation_id in expired_at {
                    let payload = serde_json::to_string(&ReservationExpired{reservation_id, expired_at: Utc::now()}).unwrap();
                    producer.send_event(reservation_id, payload).await.unwrap();
                    println!("Expire {}", reservation_id);
                }
            }
        }
    }
}
