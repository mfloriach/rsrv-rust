use anyhow::Result;
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use chrono::Utc;
use futures::future::join_all;
use rsv::infrastructure::logger::init_logger;
use rsv::infrastructure::queues::{EventConsumer, EventProducer, KafkaConfig, MessageHandler};
use rsv::types::{ReservationExpired, ReservationId};
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
async fn main() -> Result<()> {
    init_logger();

    let (tx, rx) = async_channel::bounded::<ReservationExpired>(100_000);

    let mut ingestor_handle = tokio::spawn(async move { ingestor(tx).await });

    let producer = producer()?;
    let handles: Vec<_> =
        (0..8).map(|id| tokio::spawn(worker(id, rx.clone(), producer.clone()))).collect();

    tokio::select! {
        result = &mut ingestor_handle => {
            result??;
            join_all(handles).await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        }
        signal = tokio::signal::ctrl_c() => {
            signal?;
            tracing::info!("shutdown signal received");
            ingestor_handle.abort();
            for handle in handles {
                handle.abort();
                let _ = handle.await;
            }
        }
    }

    Ok(())
}

fn producer() -> Result<EventProducer> {
    let brokers = env::var("KAFKA_BROKER_PRODUCER")?;
    let topic = env::var("KAFKA_TOPIC_PRODUCER")?;
    let group_id = env::var("KAFKA_GROUP_ID_PRODUCER")?;

    let config = KafkaConfig::builder()
        .brokers(brokers)
        .topic(topic)
        .group_id(group_id)
        .timeout_ms(50_000_000_u64)
        .max_retries(3_u32)
        .build()
        .map_err(|error| anyhow::anyhow!(error))?;

    EventProducer::new(config)
}

async fn ingestor(tx: Sender<ReservationExpired>) -> Result<()> {
    let brokers = env::var("KAFKA_BROKER_CONSUMER")?;
    let topic = env::var("KAFKA_TOPIC_CONSUMER")?;
    let group_id = env::var("KAFKA_GROUP_ID_CONSUMER")?;

    let config = KafkaConfig::builder()
        .brokers(brokers)
        .topic(topic)
        .group_id(group_id)
        .timeout_ms(50_000_000_u64)
        .max_retries(3_u32)
        .build()
        .map_err(|error| anyhow::anyhow!(error))?;
    let consumer = EventConsumer::new(config, MessagePrinter::new(tx))?;

    consumer.start().await?;

    Ok(())
}

async fn worker(id: usize, rx: Receiver<ReservationExpired>, producer: EventProducer) {
    tracing::info!("Worker {id} started");

    let mut wheel = match TimingWheel::new(1 * 60) {
        Ok(wheel) => wheel,
        Err(error) => {
            tracing::error!(%error, "failed to initialize timing wheel");
            return;
        }
    };
    let mut ticker = tokio::time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            Ok(reservation) = rx.recv() => {
                let delay = (reservation.expired_at - Utc::now())
                    .to_std()
                    .unwrap_or(Duration::ZERO);

                wheel.add(reservation.reservation_id.0, delay);
            }

            _ = ticker.tick() => {
                let expired_at = wheel.tick();
                for reservation_id in expired_at {
                    let payload = serde_json::to_string(&ReservationExpired{reservation_id: ReservationId(reservation_id), expired_at: Utc::now()}).unwrap();
                    producer.send_event(reservation_id, payload).await.unwrap();
                    println!("Expire {}", reservation_id);
                }
            }
        }
    }
}
