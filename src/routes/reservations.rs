use crate::errors::AppError;
use crate::infrastructure::distributed_lock::DistributedLock;
use crate::infrastructure::server::AppState;
use crate::repositories::ReservationStatus;
use crate::routes::List;
use crate::types::{EventId, PaymentId, ReservationId, UserId};
use actix_web::{HttpResponse, get, post, web};
use actix_web_validator::Json;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};
use std::num::NonZeroU16;
use std::time::Duration;
use tracing::instrument;
use validator::{Validate, ValidationError};

#[derive(Deserialize, Debug, Validate)]
#[cfg_attr(debug_assertions, derive(utoipa::ToSchema))]
pub struct CreateReservationRequest {
    #[validate(custom(function = "validate_seats"))]
    #[cfg_attr(debug_assertions, schema(value_type = u16))]
    seats: NonZeroU16,
}

fn validate_seats(value: &NonZeroU16) -> Result<(), ValidationError> {
    if value.get() <= 65_000 { Ok(()) } else { Err(ValidationError::new("max_seats")) }
}

#[derive(Debug, Validate, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(utoipa::IntoParams, utoipa::ToSchema))]
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

#[cfg_attr(debug_assertions, utoipa::path(
    post,
    path = "/api/v1/events/{event_id}/reservations",
    params(("event_id" = EventId, Path, description = "Event ID")),
    request_body = CreateReservationRequest,
    responses((status = 201, description = "Reservation created")),
    tag = "reservations"
))]
#[instrument(skip_all, fields(user_id = %*user_id, event_id = %event_id))]
#[post("/{event_id}/reservations")]
pub async fn create_reservation(
    user_id: web::ReqData<UserId>,
    event_id: web::Path<EventId>,
    payload: Json<CreateReservationRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    // TODO: fix this clone
    let mut user_id_0 = user_id.clone();

    let ttl = Duration::from_millis(5000);
    let key = format!("{}{}", user_id.0, event_id.0);
    let lock =
        DistributedLock::new(state.redis_client.client.clone(), user_id.into_inner(), key, ttl)
            .acquire()
            .await?;

    let mut tx = state.db_pool.get_connection().begin().await?;

    let seats_id = state
        .repositories
        .seats
        .find_available(*event_id, payload.seats.get() as i64, &mut *tx)
        .await?;

    let reservation_id = state
        .repositories
        .reservations
        .create(*event_id, user_id_0.into_inner(), &seats_id, &mut tx)
        .await?;

    state.repositories.seats.lock(*event_id, &seats_id, &mut *tx).await?;

    tx.commit().await?;

    lock.release().await?;

    Ok(HttpResponse::Created().finish())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub enum PaymentStatus {
    Succeeded,
    Failed,
}

#[derive(Deserialize, Debug, Validate, Serialize)]
#[cfg_attr(debug_assertions, derive(utoipa::ToSchema))]
pub struct PaymentIntentRequest {
    pub reservation_id: ReservationId,
    pub user_id: UserId,
    pub payment_id: PaymentId,
    pub status: PaymentStatus,
}

#[cfg_attr(debug_assertions, utoipa::path(
    post,
    path = "/api/v1/reservations/paied",
    request_body = PaymentIntentRequest,
    responses((status = 200, description = "Payment webhook accepted")),
    tag = "reservations"
))]
#[instrument(skip_all, fields(user_id = %payload.user_id, reservation = %payload.reservation_id))]
#[post("/api/v1/reservations/paied")]
pub async fn paid_reservation_webhook(
    payload: Json<PaymentIntentRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    if payload.status == PaymentStatus::Failed {
        return Err(AppError::BadRequest("has failed".to_string()));
    }

    if state
        .repositories
        .idempotency
        .find_by_aggregate::<PaymentIntentRequest>(
            payload.payment_id.0,
            state.db_pool.get_connection(),
        )
        .await?
        .is_some()
    {
        return Ok(HttpResponse::Ok().finish());
    }

    let mut tx: Transaction<'_, Postgres> = state.db_pool.get_connection().begin().await?;

    state.repositories.idempotency.create(payload.payment_id.0, &payload.0, &mut *tx).await?;

    state
        .repositories
        .reservations
        .update_status(payload.reservation_id, ReservationStatus::Paied, &mut *tx)
        .await?;

    state.repositories.seats.reserved(payload.reservation_id, &mut *tx).await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

#[cfg_attr(debug_assertions, utoipa::path(
    get,
    path = "/api/v1/reservations",
    params(crate::routes::reservations::Meta),
    responses((status = 200, description = "List reservations")),
    tag = "reservations"
))]
#[instrument(skip_all, fields(id = %*user_id))]
#[get("/")]
pub async fn get_reservations(
    user_id: web::ReqData<UserId>,
    query: web::Query<Meta>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let reservations = state
        .repositories
        .reservations
        .list(&user_id, query.page, query.limit, &query.status, state.db_pool.get_connection())
        .await?;

    Ok(HttpResponse::Ok().json(List { meta: query.0, data: reservations }))
}

#[cfg(test)]
mod tests {
    use super::{CreateReservationRequest, Meta, validate_seats};
    use std::num::NonZeroU16;
    use validator::Validate;

    #[test]
    fn create_reservation_request_accepts_a_valid_seat_count() {
        let request =
            CreateReservationRequest { seats: NonZeroU16::new(2).expect("non-zero test value") };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn create_reservation_request_rejects_excessive_seat_count() {
        let seats = NonZeroU16::new(u16::MAX).expect("non-zero test value");

        assert!(validate_seats(&seats).is_err());
    }

    #[test]
    fn reservation_list_query_requires_positive_pagination() {
        let query = Meta { page: 0, limit: 0, status: "all".to_owned() };

        assert!(query.validate().is_err());
    }
}
