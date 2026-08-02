use crate::server::AppState;
use actix_web::{HttpResponse, get, web};
use metrics::counter;
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
    name = "health.check",
    level = "debug",
    skip(state),
    fields(database_status = tracing::field::Empty, redis_status = tracing::field::Empty)
)]
#[cfg_attr(debug_assertions, utoipa::path(
    get,
    path = "/api/v1/health",
    responses((status = 200, description = "Service health")),
    tag = "health"
))]
#[get("/api/v1/health")]
pub async fn health_check(state: web::Data<AppState>) -> HttpResponse {
    let db_status = match state.db_pool.ping().await {
        Ok(()) => Status::Up,
        Err(_) => Status::Down,
    };

    let redis_status = match state.redis_client.ping().await {
        true => Status::Up,
        false => Status::Down,
    };

    tracing::Span::current()
        .record("database_status", tracing::field::display(&db_status))
        .record("redis_status", tracing::field::display(&redis_status));
    counter!("health_checks_total").increment(1);

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
