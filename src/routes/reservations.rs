use crate::AppStates;
use crate::errors::AppError;
use crate::middlewares::UserId;
use crate::models::Reservation;
use crate::routes::List;
use actix_web::{HttpResponse, web};
use actix_web_validator::Json;
use anyhow::Result;
use serde;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder, Transaction};
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct CreateReservationRequest {
    seats: i64,
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

#[instrument(skip_all, fields(user_id = %*user_id, event_id = %event_id))]
pub async fn create_reservation(
    user_id: web::ReqData<UserId>,
    event_id: web::Path<Uuid>,
    payload: Json<CreateReservationRequest>,
    state: web::Data<AppStates>,
) -> Result<HttpResponse, AppError> {
    state
        .services
        .reservations
        .create_reservation(user_id.0, event_id.into_inner(), payload.seats)
        .await?;

    Ok(HttpResponse::Created().finish())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PaymentStatus {
    Succeeded,
    Failed,
}

#[derive(Deserialize, Debug, Validate)]
pub struct PaymentIntentRequest {
    pub reservation_id: Uuid,
    pub user_id: Uuid,
    pub payment_id: Uuid,
    pub status: PaymentStatus,
}

// #[instrument(skip_all, fields(user_id = %payload.user_id, reservation = %payload.reservation_id))]
pub async fn paid_reservation_webhook(
    payload: Json<PaymentIntentRequest>,
    state: web::Data<AppStates>,
) -> Result<HttpResponse, AppError> {
    if payload.status == PaymentStatus::Failed {
        return Err(AppError::BadRequest("has failed".to_string()));
    }

    let mut tx: Transaction<'_, Postgres> = state.db_pool.get_connection().begin().await?;

    sqlx::query_as!(
        Reservation,
        "UPDATE reservations SET status = 'paied' WHERE id = $1 AND status = 'pending'",
        &payload.reservation_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;

    sqlx::query!(
        r#"
        UPDATE seats s
        SET status = 'reserved'
        FROM reservation_seats rs
        WHERE s.id = rs.seat_id
        AND rs.reservation_id = $1
        AND s.status = 'blocked'
        "#,
        payload.reservation_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

#[instrument(skip_all, fields(id = %*user_id))]
pub async fn get_reservations(
    user_id: web::ReqData<UserId>,
    query: web::Query<Meta>,
    state: web::Data<AppStates>,
) -> Result<HttpResponse, AppError> {
    let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM reservations");

    qb.push(" WHERE user_id = ").push_bind(user_id.0);

    if query.status != "all" {
        qb.push(" AND status = ").push_bind(query.status.clone());
    }

    qb.push(" ORDER BY id LIMIT ").push_bind(query.limit);
    qb.push(" OFFSET ").push_bind((query.page - 1) * query.limit);

    let rows = qb.build_query_as::<Reservation>().fetch_all(state.db_pool.get_connection()).await?;

    Ok(HttpResponse::Ok().json(List { meta: query.0, data: rows }))
}
