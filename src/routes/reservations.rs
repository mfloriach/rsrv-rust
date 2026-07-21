use crate::distributed_lock::DistributedLock;
use crate::errors::AppError;
use crate::middlewares::UserId;
use crate::models::Reservation;
use crate::routes::List;
use crate::{AppStates, database::Database};
use actix_web::{HttpResponse, web};
use actix_web_validator::Json;
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder, Transaction};
use std::time::Duration;
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct CreateReservationRequest {
    event_id: Uuid,
    seats: i32,
}

#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct Meta {
    #[validate(range(min = 1))]
    #[serde(default = "default_page")]
    page: i64,

    #[validate(range(min = 1))]
    #[serde(default = "default_limit")]
    limit: i64,

    #[serde(default = "default_status")]
    status: String,
}

fn default_page() -> i64 {
    1
}
fn default_limit() -> i64 {
    20
}

fn default_status() -> String {
    "all".to_string()
}

#[instrument(skip_all, fields(id = %*user_id))]
pub async fn create_reservation(
    user_id: web::ReqData<UserId>,
    payload: Json<CreateReservationRequest>,
    state: web::Data<AppStates>,
) -> Result<HttpResponse, AppError> {
    let ttl = Duration::from_millis(5000);
    let key = format!("{}{}", user_id.0, payload.event_id);
    DistributedLock::new(state.redis_client.client.clone(), user_id.0, key, ttl).acquire().await?;

    reserve(user_id, payload, state.db_pool.clone()).await?;

    Ok(HttpResponse::Created().finish())
}

#[instrument(skip_all, fields(id = %*user_id))]
pub async fn get_reservations(
    user_id: web::ReqData<UserId>,
    query: web::Query<Meta>,
    state: web::Data<AppStates>,
) -> HttpResponse {
    let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM reservations");

    qb.push(" WHERE user_id = ").push_bind(user_id.0);

    if query.status != "all" {
        qb.push(" AND status = ").push_bind(query.status.clone());
    }

    qb.push(" ORDER BY id LIMIT ").push_bind(query.limit);
    qb.push(" OFFSET ").push_bind((query.page - 1) * query.limit);

    let rows = qb
        .build_query_as::<Reservation>()
        .fetch_all(state.db_pool.get_connection())
        .await
        .expect("sdfds");

    HttpResponse::Ok().json(List { meta: query.0, data: rows })
}

pub async fn reserve(
    user_id: web::ReqData<UserId>,
    payload: Json<CreateReservationRequest>,
    db: Database,
) -> Result<()> {
    let mut tx: Transaction<'_, Postgres> = db.get_connection().begin().await?;

    sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE").execute(&mut *tx).await?;

    let event_seats = sqlx::query_scalar!(
        "SELECT capacity FROM events WHERE id = $1 FOR UPDATE;",
        payload.event_id
    )
    .fetch_one(&mut *tx)
    .await?;

    if event_seats < payload.seats {
        tx.rollback().await?;
        bail!("Not enough seats available");
    }

    sqlx::query!(
        r#"
        INSERT INTO reservations (id, event_id, user_id, seats)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::now_v7(),
        payload.event_id,
        user_id.0,
        payload.seats
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        UPDATE events SET capacity = capacity - $1 WHERE id = $2
        "#,
        payload.seats,
        payload.event_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
