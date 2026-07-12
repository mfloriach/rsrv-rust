use std::time::Duration;

use crate::database::Database;
use crate::middlewares::UserId;
use crate::{cache::CacherRedis, distributed_lock::DistributedLock};
use actix_web::{HttpResponse, web};
use actix_web_validator::Json;
use anyhow::{Result, bail};
use serde::Deserialize;
use sqlx::{Postgres, Transaction};
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct CreateReservationRequest {
    event_id: Uuid,
    seats: i32,
}

#[instrument(skip_all, fields(id = %*user_id))]
pub async fn create_reservation(
    user_id: web::ReqData<UserId>,
    payload: Json<CreateReservationRequest>,
    db: web::Data<Database>,
    cacher: web::Data<CacherRedis>,
) -> HttpResponse {
    tracing::info!("start reservation for user_id: {}", user_id.0);

    let dlock = DistributedLock::new(
        cacher.client.clone(),
        user_id.0,
        "".to_string(),
        Duration::from_secs(5),
    );
    dlock.acquire().await.expect("could not acquire lock");

    match reserve(user_id, payload, db).await {
        Ok(_) => {
            tracing::info!("reservation created successfully");
        }
        Err(e) => {
            tracing::error!("failed to create reservation: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    dlock.release().await.expect("could not release lock");

    HttpResponse::Created().finish()
}

pub async fn reserve(
    user_id: web::ReqData<UserId>,
    payload: Json<CreateReservationRequest>,
    db: web::Data<Database>,
) -> Result<()> {
    tracing::info!("start reservation for user_id: {}", user_id.0);

    let mut tx: Transaction<'_, Postgres> =
        db.get_connection().begin().await.expect("could not get transaction");

    sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
        .execute(&mut *tx)
        .await
        .expect("cloud not make it serialize");

    let event_seats = sqlx::query_scalar!(
        "SELECT capacity FROM events WHERE id = $1 FOR UPDATE;",
        payload.event_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("check seats");

    if event_seats < payload.seats {
        tx.rollback().await.expect("rollback");
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
    .await
    .expect("insert reservation");

    sqlx::query!(
        r#"
        UPDATE events SET capacity = capacity - $1 WHERE id = $2
        "#,
        payload.seats,
        payload.event_id
    )
    .execute(&mut *tx)
    .await
    .expect("update event capacity");

    tx.commit().await.expect("sdfdfds");

    Ok(())
}
