use crate::errors::AppError;
use crate::middlewares::UserId;
use crate::routes::List;
use crate::server::AppStates;
use actix_web::{HttpResponse, web};
use actix_web_validator::Json;
use anyhow::Result;
use serde;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU16;
use tracing::instrument;
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Deserialize, Debug, Validate)]
pub struct CreateReservationRequest {
    #[validate(custom(function = "validate_seats"))]
    seats: NonZeroU16,
}

fn validate_seats(value: &NonZeroU16) -> Result<(), ValidationError> {
    if value.get() <= 65_000 { Ok(()) } else { Err(ValidationError::new("max_seats")) }
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
        .create_reservation(user_id.0, event_id.into_inner(), payload.seats.get())
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

#[instrument(skip_all, fields(user_id = %payload.user_id, reservation = %payload.reservation_id))]
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

#[instrument(skip_all, fields(id = %*user_id))]
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
