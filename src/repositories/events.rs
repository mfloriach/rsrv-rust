use crate::infrastructure::database::Database;
use anyhow::{Ok, Result};
use chrono::{DateTime, TimeZone, Utc};
use serde::Serialize;
use sqlx::FromRow;
use sqlx::{Postgres, QueryBuilder, Transaction};
use uuid::Uuid;

#[derive(FromRow, Serialize)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EventRepository {
    db: Database,
}

impl EventRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        name: String,
        description: Option<String>,
        seats: i32,
    ) -> Result<Uuid> {
        let mut tx: Transaction<'_, Postgres> = self.db.get_connection().begin().await?;

        let event_id = Uuid::now_v7();
        sqlx::query!(
            r#"
        INSERT INTO events (id, title, description)
        VALUES ($1, $2, $3)
        "#,
            event_id,
            name,
            description
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
        INSERT INTO seats (id, event_id, seat_number)
        SELECT
            gen_random_uuid(),
            $1,
            gs
        FROM generate_series(1, $2) AS gs
        "#,
            event_id,
            seats
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(event_id)
    }

    pub async fn list(&self, page: i64, limit: i64, greather_than: i64) -> Result<Vec<Event>> {
        let created_at = Utc.timestamp_opt(greather_than, 0).single().unwrap();

        let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM events");
        qb.push(" WHERE created_at > ").push_bind(created_at);
        qb.push(" ORDER BY id LIMIT ").push_bind(limit);
        qb.push(" OFFSET ").push_bind((page - 1) * limit);

        Ok(qb.build_query_as::<Event>().fetch_all(self.db.get_connection()).await?)
    }
}
