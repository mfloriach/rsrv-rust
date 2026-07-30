use crate::server::AppStates;
use actix_web::{HttpResponse, web};
use serde::Serialize;
use strum_macros::{Display, IntoStaticStr};
use tracing::instrument;

#[derive(Debug, PartialEq, Display, IntoStaticStr, Serialize)]
#[strum(serialize_all = "snake_case")]
enum Status {
    Up,
    Down,
}

#[derive(serde::Serialize)]
struct Service {
    name: String,
    status: Status,
}

#[derive(serde::Serialize)]
struct HealthCheckResponse {
    status: String,
    services: Vec<Service>,
}

#[instrument(
    name = "secure_login_flow",
    level = "debug",
    skip(state),
    fields(attempt_status = "pending")
)]
pub async fn health_check(state: web::Data<AppStates>) -> HttpResponse {
    let db_status = match state.db_pool.ping().await {
        Ok(()) => Status::Up,
        Err(_) => Status::Down,
    };

    let redis_status = match state.redis_client.ping().await {
        true => Status::Up,
        false => Status::Down,
    };

    let response = HealthCheckResponse {
        status: "healthy".into(),
        services: vec![
            Service { name: "API".into(), status: Status::Up },
            Service { name: "Database".into(), status: db_status },
            Service { name: "Redis".into(), status: redis_status },
            Service { name: "Kafka".into(), status: Status::Down },
        ],
    };
    HttpResponse::Ok().json(response)
}
