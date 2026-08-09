use crate::types::EventId;
use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use serde::Serialize;
use sqlx::{Executor, FromRow, Postgres, QueryBuilder};
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
pub struct EventRepository;

impl EventRepository {
    pub async fn create<'e>(
        &self,
        name: &str,
        description: Option<&str>,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<EventId> {
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
        .execute(executor)
        .await?;

        Ok(EventId(event_id))
    }

    pub async fn list<'e>(
        &self,
        page: i64,
        limit: i64,
        greather_than: i64,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<Vec<Event>> {
        let created_at = Utc.timestamp_opt(greather_than, 0).single().unwrap();

        let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM events");
        qb.push(" WHERE created_at > ").push_bind(created_at);
        qb.push(" ORDER BY id LIMIT ").push_bind(limit);
        qb.push(" OFFSET ").push_bind((page - 1) * limit);

        Ok(qb.build_query_as::<Event>().fetch_all(executor).await?)
    }
}
