use crate::errors::AppError;
use crate::middlewares::UserId;
use crate::routes::List;
use crate::server::AppStates;
use actix_web::{HttpResponse, get, post, web};
use actix_web_validator::Json;
use anyhow::Result;
use serde;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU16;
use tracing::instrument;
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Deserialize, Debug, Validate, utoipa::ToSchema)]
pub struct CreateReservationRequest {
    #[validate(custom(function = "validate_seats"))]
    #[schema(value_type = u16)]
    seats: NonZeroU16,
}

fn validate_seats(value: &NonZeroU16) -> Result<(), ValidationError> {
    if value.get() <= 65_000 { Ok(()) } else { Err(ValidationError::new("max_seats")) }
}

#[derive(Debug, Validate, Serialize, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
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

#[utoipa::path(
    post,
    path = "/api/v1/events/{event_id}/reservations",
    params(("event_id" = Uuid, Path, description = "Event ID")),
    request_body = CreateReservationRequest,
    responses((status = 201, description = "Reservation created")),
    tag = "reservations"
)]
#[instrument(skip_all, fields(user_id = %*user_id, event_id = %event_id))]
#[post("/{event_id}/reservations")]
pub async fn create_reservation(
    user_id: web::ReqData<UserId>,
    event_id: web::Path<Uuid>,
    payload: Json<CreateReservationRequest>,
    state: web::Data<AppStates>,
) -> Result<HttpResponse, AppError> {
    state
        .services
        .reservations
        .create_reservation(user_id.0, event_id.into_inner(), payload.seats.get())
        .await?;

    Ok(HttpResponse::Created().finish())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum PaymentStatus {
    Succeeded,
    Failed,
}

#[derive(Deserialize, Debug, Validate, utoipa::ToSchema)]
pub struct PaymentIntentRequest {
    pub reservation_id: Uuid,
    pub user_id: Uuid,
    pub payment_id: Uuid,
    pub status: PaymentStatus,
}

#[utoipa::path(
    post,
    path = "/api/v1/reservations/paied",
    request_body = PaymentIntentRequest,
    responses((status = 200, description = "Payment webhook accepted")),
    tag = "reservations"
)]
#[instrument(skip_all, fields(user_id = %payload.user_id, reservation = %payload.reservation_id))]
#[post("/api/v1/reservations/paied")]
pub async fn paid_reservation_webhook(
    payload: Json<PaymentIntentRequest>,
    state: web::Data<AppStates>,
) -> Result<HttpResponse, AppError> {
    if payload.status == PaymentStatus::Failed {
        return Err(AppError::BadRequest("has failed".to_string()));
    }

    state.repositories.reservations.paid(payload.reservation_id).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    get,
    path = "/api/v1/reservations",
    params(crate::routes::reservations::Meta),
    responses((status = 200, description = "List reservations")),
    tag = "reservations"
)]
#[instrument(skip_all, fields(id = %*user_id))]
#[get("/")]
pub async fn get_reservations(
    user_id: web::ReqData<UserId>,
    query: web::Query<Meta>,
    state: web::Data<AppStates>,
) -> Result<HttpResponse, AppError> {
    let reservations = state
        .repositories
        .reservations
        .list(user_id.0, query.page, query.limit, query.status.clone())
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
