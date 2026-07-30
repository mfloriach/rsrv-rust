use crate::errors::AppError;
use crate::middlewares::UserId;
use crate::routes::List;
use crate::server::AppStates;
use actix_web::{HttpResponse, Result, web};
use actix_web_validator::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use validator::Validate;

#[derive(Deserialize, Debug, Validate, Clone)]
pub struct CreateEventRequest {
    name: String,
    description: Option<String>,
    capacity: i32,
}

#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct Meta {
    #[validate(range(min = 1))]
    #[serde(default = "default_page")]
    page: i64,

    #[validate(range(min = 1))]
    #[serde(default = "default_limit")]
    limit: i64,

    #[serde(default = "default_event_at_gte")]
    event_at_gte: i64,
}

fn default_page() -> i64 {
    1
}
fn default_limit() -> i64 {
    20
}
fn default_event_at_gte() -> i64 {
    Utc::now().timestamp()
}

#[instrument(skip_all, fields(id = %*user_id))]
pub async fn get_events(
    user_id: web::ReqData<UserId>,
    query: web::Query<Meta>,
    state: web::Data<AppStates>,
) -> Result<HttpResponse, AppError> {
    let events =
        state.repositories.events.list(query.page, query.limit, query.event_at_gte).await?;

    Ok(HttpResponse::Ok().json(List { meta: query.0, data: events }))
}

#[instrument(skip_all, fields(id = %*user_id))]
pub async fn create_event(
    user_id: web::ReqData<UserId>,
    payload: Json<CreateEventRequest>,
    state: web::Data<AppStates>,
) -> Result<HttpResponse, AppError> {
    let event_id = state
        .repositories
        .events
        .create(payload.name.clone(), payload.description.clone(), payload.capacity)
        .await?;

    Ok(HttpResponse::Created().body(event_id.to_string()))
}
