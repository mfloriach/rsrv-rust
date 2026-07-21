use crate::AppStates;
use crate::errors::AppError;
use crate::middlewares::UserId;
use crate::models::Event;
use crate::routes::List;
use actix_web::Result;
use actix_web::{HttpResponse, web};
use actix_web_validator::Json;
use chrono::TimeZone;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
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
    // event_at_lte: i64,
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
    let created_at = Utc.timestamp_opt(query.event_at_gte, 0).single().unwrap();

    let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM events");
    qb.push(" WHERE created_at > ").push_bind(created_at);
    qb.push(" ORDER BY id LIMIT ").push_bind(query.limit);
    qb.push(" OFFSET ").push_bind((query.page - 1) * query.limit);

    let rows = qb.build_query_as::<Event>().fetch_all(state.db_pool.get_connection()).await?;

    Ok(HttpResponse::Ok().json(List { meta: query.0, data: rows }))
}

#[instrument(skip_all, fields(id = %*user_id))]
pub async fn create_event(
    user_id: web::ReqData<UserId>,
    payload: Json<CreateEventRequest>,
    state: web::Data<AppStates>,
) -> Result<HttpResponse, AppError> {
    sqlx::query!(
        r#"
        INSERT INTO events (id, title, capacity, description)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::now_v7(),
        payload.name,
        payload.capacity,
        payload.description
    )
    .execute(state.db_pool.get_connection())
    .await?;

    Ok(HttpResponse::Created().finish())
}
