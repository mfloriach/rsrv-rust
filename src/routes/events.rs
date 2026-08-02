use crate::errors::AppError;
use crate::middlewares::UserId;
use crate::routes::List;
use crate::server::AppState;
use actix_web::{HttpResponse, Result, get, post, web};
use actix_web_validator::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use validator::Validate;

#[derive(Deserialize, Debug, Validate, Clone)]
#[cfg_attr(debug_assertions, derive(utoipa::ToSchema))]
pub struct CreateEventRequest {
    #[validate(length(min = 1, max = 255))]
    name: String,
    #[validate(length(max = 5_000))]
    description: Option<String>,
    #[validate(range(min = 1))]
    capacity: i32,
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

#[cfg_attr(debug_assertions, utoipa::path(
    get,
    path = "/api/v1/events",
    params(crate::routes::events::Meta),
    responses((status = 200, description = "List events")),
    tag = "events"
))]
#[instrument(skip_all, fields(id = %*user_id))]
#[get("/")]
pub async fn get_events(
    user_id: web::ReqData<UserId>,
    query: web::Query<Meta>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let events =
        state.repositories.events.list(query.page, query.limit, query.event_at_gte).await?;

    Ok(HttpResponse::Ok().json(List { meta: query.0, data: events }))
}

#[cfg_attr(debug_assertions, utoipa::path(
    post,
    path = "/api/v1/events",
    request_body = CreateEventRequest,
    responses((status = 201, description = "Event created")),
    tag = "events"
))]
#[instrument(skip_all, fields(id = %*user_id))]
#[post("/")]
pub async fn create_event(
    user_id: web::ReqData<UserId>,
    payload: Json<CreateEventRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let event_id = state
        .repositories
        .events
        .create(payload.name.clone(), payload.description.clone(), payload.capacity)
        .await?;

    Ok(HttpResponse::Created().body(event_id.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{CreateEventRequest, Meta};
    use validator::Validate;

    #[test]
    fn create_event_request_requires_positive_capacity() {
        let request =
            CreateEventRequest { name: "concert".to_owned(), description: None, capacity: 0 };

        assert!(request.validate().is_err());
    }

    #[test]
    fn create_event_request_rejects_invalid_text_lengths() {
        let request = CreateEventRequest { name: String::new(), description: None, capacity: 1 };

        assert!(request.validate().is_err());
    }

    #[test]
    fn event_list_query_requires_positive_pagination() {
        let query = Meta { page: 0, limit: 0, event_at_gte: 0 };

        assert!(query.validate().is_err());
    }
}
