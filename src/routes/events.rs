use crate::database::Database;
use crate::middlewares::UserId;
use actix_web::{HttpResponse, web};
use actix_web_validator::Json;
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct CreateEventRequest {
    name: String,
    description: Option<String>,
    capacity: i32,
}

#[instrument(skip_all, fields(id = %*user_id))]
pub async fn create_event(
    user_id: web::ReqData<UserId>,
    payload: Json<CreateEventRequest>,
    db: web::Data<Database>,
) -> HttpResponse {
    tracing::info!("start aking event");

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
    .execute(db.get_connection())
    .await
    .expect("insert event");

    HttpResponse::Created().finish()
}
