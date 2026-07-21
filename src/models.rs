use chrono::{DateTime, Utc};
use secrecy::Secret;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password: Secret<String>,
}

#[derive(FromRow, Serialize)]
pub struct Reservation {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub seats: i32,
    pub reserved_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub capacity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
