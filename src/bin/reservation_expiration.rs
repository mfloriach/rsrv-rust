use anyhow::Result;
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use dotenvy::dotenv;
use rsv::infrastructure::database::Database;
use rsv::infrastructure::logger::init_logger;
use rsv::infrastructure::queues::{EventConsumer, KafkaConfig, MessageHandler};
use rsv::repositories::{ReservationRepository, ReservationStatus};
use rsv::types::{ReservationExpired, ReservationId};
use std::env;
use tokio_util::sync::CancellationToken;

struct MessagePrinter {
    tx: Sender<ReservationId>,
}

impl MessagePrinter {
    fn new(tx: Sender<ReservationId>) -> Box<Self> {
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
    dotenv().ok();

    init_logger();

    let (tx, rx) = async_channel::bounded::<ReservationId>(100_000);

    let mut ingestor_handle = tokio::spawn(async move { ingestor(tx).await });

    let database_url = env::var("DATABASE_URL")?;
    let connection_pool = Database::new(&database_url).await?;

    let token = CancellationToken::new();

    let mut handles = Vec::new();
    for i in 0..8 {
        handles.push(tokio::spawn(worker(
            i as usize,
            rx.clone(),
            token.clone(),
            connection_pool.clone(),
        )));
    }

    tokio::select! {
        result = &mut ingestor_handle => {
            result??;
            token.cancel();
        }
        signal = tokio::signal::ctrl_c() => {
            signal?;
            tracing::info!("shutdown signal received");
            ingestor_handle.abort();
            token.cancel();
            connection_pool.disconnect().await;
        }
    }

    Ok(())
}

async fn ingestor(tx: Sender<ReservationId>) -> Result<()> {
    let broker = env::var("KAFKA_BROKER")?;
    let topic = env::var("KAFKA_TOPIC")?;
    let group_id = env::var("KAFKA_GROUP_ID")?;

    let config = KafkaConfig::builder()
        .brokers(broker)
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

async fn worker(
    id: usize,
    rx: Receiver<ReservationId>,
    token: CancellationToken,
    db_pool: Database,
) {
    tracing::info!("Worker {id} started");

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                tracing::info!("Worker {id} shutting down");
                break;
            }

            result = rx.recv() => {
                let Ok(reservation_id) = result else {
                    tracing::info!("Channel closed");
                    break;
                };

                match ReservationRepository.update_status(reservation_id, ReservationStatus::Expired, db_pool.get_connection()).await {
                    Ok(_) => tracing::info!(
                        worker = id,
                        reservation_id = %reservation_id,
                        "Reservation expired"
                    ),
                    Err(err) => tracing::error!(
                        worker = id,
                        reservation_id = %reservation_id,
                        error = %err,
                        "Failed to expire reservation"
                    ),
                }
            }
        }
    }
}
