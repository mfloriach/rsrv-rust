use rsv::configuration::get_configuration;
use rsv::infrastructure::cache::CacherRedis;
use rsv::infrastructure::database::Database;
use rsv::infrastructure::logger::init_logger;
use rsv::repositories::{EventRepository, ReservationRepository};
use rsv::server::{AppStates, Repositores, Services, run};
use rsv::services::ReservationService;
use secrecy::ExposeSecret;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool =
        Database::new(configuration.database.get_connection_string().expose_secret()).await;
    let cacher =
        CacherRedis::new(configuration.redis.get_connection_string().expose_secret()).await;

    let reservation_repository = ReservationRepository::new(connection_pool.clone());
    let event_repository = EventRepository::new(connection_pool.clone());

    let app_states = AppStates {
        db_pool: connection_pool.clone(),
        redis_client: cacher.clone(),
        services: Services {
            reservations: ReservationService::new(
                cacher.client.clone(),
                reservation_repository.clone(),
            ),
        },
        repositories: Repositores {
            events: event_repository,
            reservations: reservation_repository.clone(),
        },
    };
    let listener = TcpListener::bind(format!("{}:{}", "localhost", 8080))?;

    run(listener, app_states)?.await
}
