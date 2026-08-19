use anyhow::Result;
use serde::Serialize;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxEventType {
    ReservationExpire,
}

impl OutboxEventType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ReservationExpire => "reservation_expire",
        }
    }
}

#[derive(Clone)]
pub struct OutboxRepository;

impl OutboxRepository {
    pub async fn create<'e>(
        &self,
        aggregate_id: Uuid,
        event_type: OutboxEventType,
        payload: String,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<Uuid> {
        let outbox_id = Uuid::now_v7();
        sqlx::query!(
            r#"
        INSERT INTO outbox (id, aggregate_id, event_type, payload)
        VALUES ($1, $2, $3, $4)
        "#,
            outbox_id,
            aggregate_id,
            event_type.as_str(),
            serde_json::from_str::<serde_json::Value>(&payload)?,
        )
        .execute(executor)
        .await?;

        Ok(outbox_id)
    }
}
