use anyhow::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;
use sqlx::types::Json;
use sqlx::{Executor, Postgres, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct IdempotencyRepository;

impl IdempotencyRepository {
    pub async fn create<'e, T>(
        &self,
        aggregate_id: Uuid,
        payload: &T,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<Uuid>
    where
        T: Serialize,
    {
        let outbox_id = Uuid::now_v7();
        sqlx::query!(
            r#"
            INSERT INTO idempotency (aggregate_id, payload)
            VALUES ($1, $2)
            "#,
            aggregate_id,
            Json(payload) as _
        )
        .execute(executor)
        .await?;

        Ok(outbox_id)
    }

    pub async fn find_by_aggregate<'e, T>(
        &self,
        aggregate_id: Uuid,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let row = sqlx::query(
            r#"
            SELECT payload
            FROM idempotency
            WHERE aggregate_id = $1
            "#,
        )
        .bind(aggregate_id)
        .fetch_optional(executor)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let Json(payload): Json<T> = row.try_get("payload")?;

        Ok(Some(payload))
    }
}
