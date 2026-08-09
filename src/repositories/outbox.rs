use anyhow::Result;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

#[derive(Clone)]
pub struct OutboxRepository;

impl OutboxRepository {
    pub async fn create<'e>(
        &self,
        aggregate_id: Uuid,
        event_type: &str,
        payload: &str,
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
            event_type,
            serde_json::Value::String(payload.into())
        )
        .execute(executor)
        .await?;

        Ok(outbox_id)
    }
}
