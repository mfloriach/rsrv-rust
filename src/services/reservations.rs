use crate::distributed_lock::DistributedLock;
use crate::kafka::ReservationExpired;
use crate::queues::{EventProducer, KafkaConfig};
use crate::repositories::ReservationRepository;
use anyhow::{Ok, Result};
use chrono::Utc;
use redis::Client;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct ReservationService {
    redis_client: Client,
    reservation_repository: ReservationRepository,
}

impl ReservationService {
    pub fn new(redis_client: Client, reservation_repository: ReservationRepository) -> Self {
        Self { redis_client, reservation_repository }
    }

    pub async fn create_reservation(
        &self,
        user_id: Uuid,
        event_id: Uuid,
        seats: i64,
    ) -> Result<()> {
        let ttl = Duration::from_millis(5000);
        let key = format!("{}{}", user_id, event_id);
        let lock =
            DistributedLock::new(self.redis_client.clone(), user_id, key, ttl).acquire().await?;

        let reservation_id = self.reservation_repository.create(user_id, event_id, seats).await?;

        lock.release().await?;

        self.add_to_delay_queu(reservation_id).await?;

        Ok(())
    }

    async fn add_to_delay_queu(&self, reservation_id: Uuid) -> Result<()> {
        let producer = EventProducer::new(KafkaConfig {
            brokers: "127.0.0.1:29092".to_string(),
            topic: "reservation.delay".to_string(),
            group_id: "kafka-streaming-group".to_string(),
            timeout_ms: 50000000,
            max_retries: 3,
        })?;

        let event = ReservationExpired { reservation_id, occurred_at: Utc::now() };
        let payload = serde_json::to_string(&event)?;

        Ok(producer.send_event(reservation_id.to_string(), payload).await?)
    }
}
