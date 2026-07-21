use rsv::database::Database;
pub mod configuration;
use configuration::get_configuration;
use secrecy::ExposeSecret;
pub mod cache;
use rsv::cache::CacherRedis;
use rsv::{AppStates, run};
use std::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool =
        Database::new(configuration.database.get_connection_string().expose_secret()).await;
    let cacher =
        CacherRedis::new(configuration.redis.get_connection_string().expose_secret()).await;

    let app_states = AppStates { db_pool: connection_pool, redis_client: cacher };
    let listener = TcpListener::bind(format!("{}:{}", "localhost", 8080))?;

    run(listener, app_states)?.await
}

fn init_logger() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            "info,actix_web=info,tracing_actix_web=info,sqlx=debug",
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_target(true)
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE),
        )
        .init();
}
