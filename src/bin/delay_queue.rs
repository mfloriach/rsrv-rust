use anyhow::Result;
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use chrono::Utc;
use rsv::database::Database;
use rsv::kafka::ReservationExpired;
use rsv::logger::init_logger;
use rsv::queues::{EventConsumer, KafkaConfig, MessageHandler};
use std::net::TcpListener;
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
async fn main() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    let (tx, rx) = async_channel::bounded::<Uuid>(100_000);

    let connection_pool =
        Database::new("postgres://myuser:mysecretpassword@db:5432/mydatabase").await;

    let consumer = EventConsumer::new(
        KafkaConfig {
            brokers: "kafka:9092".to_string(),
            topic: "reservation.delay".to_string(),
            group_id: "kafka-streaming-group".to_string(),
            timeout_ms: 50000000,
            max_retries: 3,
        },
        MessagePrinter::new(tx),
    )
    .expect("asdsadas");

    let mut handles = Vec::new();
    for i in 0..8 {
        handles.push(tokio::spawn(worker(i as usize, rx.clone(), connection_pool.clone())));
    }

    consumer.start().await.expect("could not start consumer");
}

async fn worker(id: usize, rx: Receiver<Uuid>, db_pool: Database) {
    tracing::info!("Worker {id} started");

    loop {
        while let Ok(reservation_id) = rx.recv().await {
            match sqlx::query!(
                r#"
                UPDATE seats s
                SET status = 'available'
                FROM reservation_seats rs
                WHERE s.id = rs.seat_id
                AND rs.reservation_id = $1;
                "#,
                reservation_id
            )
            .execute(db_pool.get_connection())
            .await
            {
                Ok(_) => tracing::info!(
                    worker = id,
                    reservation_id = %reservation_id,
                    "Seat expired become free"
                ),
                Err(err) => tracing::info!(
                    worker = id,
                    reservation_id = %reservation_id,
                    error = %err,
                    "Seat error remain block"
                ),
            };
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
