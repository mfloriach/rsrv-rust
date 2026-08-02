use anyhow::Result;
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use rsv::infrastructure::database::Database;
use rsv::infrastructure::logger::init_logger;
use rsv::infrastructure::queues::{EventConsumer, KafkaConfig, MessageHandler};
use rsv::repositories::ReservationRepository;
use rsv::services::ReservationExpired;
use std::env;
use std::time::Duration;
use uuid::Uuid;

struct MessagePrinter {
    tx: Sender<Uuid>,
}

impl MessagePrinter {
    fn new(tx: Sender<Uuid>) -> Box<Self> {
        Box::new(MessagePrinter { tx })
    }
}

#[async_trait]
impl MessageHandler for MessagePrinter {
    async fn handle(&self, _key: &[u8], payload: &[u8]) -> Result<()> {
        let event: ReservationExpired = serde_json::from_slice(payload)?;
        self.tx.send(event.reservation_id).await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();

    let database_url = env::var("DATABASE_URL")?;

    let (tx, rx) = async_channel::bounded::<Uuid>(100_000);

    let ingestor_handle = tokio::spawn(async move { ingestor(tx).await });

    let connection_pool = Database::new(&database_url.into_boxed_str()).await;
    let reservation_repository = ReservationRepository::new(connection_pool.clone());

    let mut handles = Vec::new();
    for i in 0..8 {
        handles.push(tokio::spawn(worker(i as usize, rx.clone(), reservation_repository.clone())));
    }

    ingestor_handle.await??;

    for handle in handles {
        handle.await?;
    }

    Ok(())
}

async fn ingestor(tx: Sender<Uuid>) -> Result<()> {
    let broker = env::var("KAFKA_BROKER")?;
    let topic = env::var("KAFKA_TOPIC")?;
    let group_id = env::var("KAFKA_GROUP_ID")?;

    let consumer = EventConsumer::new(
        KafkaConfig { brokers: broker, topic, group_id, timeout_ms: 50000000, max_retries: 3 },
        MessagePrinter::new(tx),
    )?;

    consumer.start().await?;

    Ok(())
}

async fn worker(id: usize, rx: Receiver<Uuid>, reservation_repository: ReservationRepository) {
    tracing::info!("Worker {id} started");

    loop {
        while let Ok(reservation_id) = rx.recv().await {
            match reservation_repository.expired(reservation_id).await {
                Ok(_) => tracing::info!(
                    worker = id,
                    reservation_id = %reservation_id,
                    "Reservation expired"
                ),
                Err(err) => {
                    tracing::info!(
                        worker = id,
                        reservation_id = %reservation_id,
                        error = %err,
                        "Seat error remain block"
                    )
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
