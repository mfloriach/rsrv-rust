use anyhow::Result;
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use rsv::configuration::get_configuration;
use rsv::database::Database;
use rsv::kafka::ReservationExpired;
use rsv::queues::{EventConsumer, KafkaConfig, MessageHandler};
use secrecy::ExposeSecret;
use std::time::Duration;
use uuid::Uuid;

struct MessagePrinter {
    tx: Sender<Uuid>,
    db_pool: Database,
}

impl MessagePrinter {
    fn new(tx: Sender<Uuid>, db_pool: Database) -> Box<Self> {
        Box::new(MessagePrinter { tx, db_pool })
    }
}

#[async_trait]
impl MessageHandler for MessagePrinter {
    async fn handle(&self, _key: &[u8], payload: &[u8]) -> Result<()> {
        let event: ReservationExpired = serde_json::from_slice(payload)?;

        match sqlx::query!(
            r#"
                UPDATE seats s
                SET status = 'available'
                FROM reservation_seats rs
                WHERE s.id = rs.seat_id
                AND rs.reservation_id = $1;
                "#,
            event.reservation_id
        )
        .execute(self.db_pool.get_connection())
        .await
        {
            Ok(_) => tracing::info!(
                // worker = id,
                reservation_id = %event.reservation_id,
                "Seat expired become free"
            ),
            Err(err) => tracing::info!(
                // worker = id,
                reservation_id = %event.reservation_id,
                error = %err,
                "Seat error remain block"
            ),
        };
        // self.tx.send(event.reservation_id).await?;

        Ok(())
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let (tx, rx) = async_channel::bounded::<Uuid>(100_000);

    // let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool =
        Database::new("postgres://myuser:mysecretpassword@db:5432/mydatabase").await;

    // let mut handles = Vec::new();
    // for i in 0..8 {
    //     handles.push(tokio::spawn(worker(i as usize, rx.clone(), connection_pool.clone())));
    // }

    let consumer = EventConsumer::new(
        KafkaConfig {
            brokers: "kafka:9092".to_string(),
            topic: "reservation.delay".to_string(),
            group_id: "kafka-streaming-group".to_string(),
            timeout_ms: 50000000,
            max_retries: 3,
        },
        MessagePrinter::new(tx, connection_pool),
    )?;

    consumer.start().await.expect("dsfdsfds");

    // tokio::spawn(async move {
    //     let _ = consumer.start().await;
    // });

    // tokio::signal::ctrl_c().await?;

    // tracing::info!("Shutting down...");

    // for handle in handles {
    //     handle.abort();
    // }

    Ok(())
}

// async fn ingestor(tx: Sender<Uuid>) {
//     let consumer = EventConsumer::new(
//         KafkaConfig {
//             brokers: "kafka:9092".to_string(),
//             topic: "reservation.delay".to_string(),
//             group_id: "kafka-streaming-group".to_string(),
//             timeout_ms: 50000000,
//             max_retries: 3,
//         },
//         MessagePrinter::new(tx),
//     )
//     .expect("errro");

//     let _ = consumer.start().await;
// }

// async fn worker(id: usize, rx: Receiver<Uuid>, db_pool: Database) {
//     tracing::info!("Worker {id} started");

//     loop {
//         while let Ok(reservation_id) = rx.recv().await {
//             match sqlx::query!(
//                 r#"
//                 UPDATE seats s
//                 SET status = 'available'
//                 FROM reservation_seats rs
//                 WHERE s.id = rs.seat_id
//                 AND rs.reservation_id = $1;
//                 "#,
//                 reservation_id
//             )
//             .execute(db_pool.get_connection())
//             .await
//             {
//                 Ok(_) => tracing::info!(
//                     worker = id,
//                     reservation_id = %reservation_id,
//                     "Seat expired become free"
//                 ),
//                 Err(err) => tracing::info!(
//                     worker = id,
//                     reservation_id = %reservation_id,
//                     error = %err,
//                     "Seat error remain block"
//                 ),
//             };
//         }

//         tokio::time::sleep(Duration::from_secs(1)).await;
//     }
// }
