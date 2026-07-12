use crate::cache::CacherRedis;
use crate::database::Database;
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
    name = "secure_login_flow",          // Changes the span name
    level = "debug",                     // Emits span at DEBUG level instead of INFO
    skip(pool, redis_client),                      // Excludes sensitive fields from logs
    fields(attempt_status = "pending")   // Adds custom extra fields
)]
pub async fn health_check(
    pool: web::Data<Database>,
    redis_client: web::Data<CacherRedis>,
) -> HttpResponse {
    let db_status = match pool.ping().await {
        Ok(()) => Status::Up,
        Err(_) => Status::Down,
    };

    let redis_status = match redis_client.ping().await {
        true => Status::Up,
        false => Status::Down,
    };

    tracing::info!("HELLO LOG");
    logs();
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

pub fn logs() {
    tracing::info!("HELLO LOG2");
}
