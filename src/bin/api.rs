use rsv::cache::CacherRedis;
use rsv::configuration::get_configuration;
use rsv::database::Database;
use rsv::repositories::ReservationRepository;
use rsv::services::ReservationService;
use rsv::{AppStates, Services, run};
use secrecy::ExposeSecret;
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

    let reservation_repository = ReservationRepository::new(connection_pool.clone());

    let app_states = AppStates {
        db_pool: connection_pool.clone(),
        redis_client: cacher.clone(),
        services: Services {
            reservations: ReservationService::new(cacher.client.clone(), reservation_repository),
        },
    };
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
