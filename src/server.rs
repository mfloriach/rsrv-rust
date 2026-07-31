use crate::infrastructure::cache::CacherRedis;
use crate::infrastructure::database::Database;
use crate::middlewares::auth;
use crate::repositories::{EventRepository, ReservationRepository};
use crate::routes::{
    auth_config, events_config, health_check, paid_reservation_webhook, reservations_config,
};
use crate::services::ReservationService;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

#[derive(Clone)]
pub struct Services {
    pub reservations: ReservationService,
}

#[derive(Clone)]
pub struct Repositores {
    pub events: EventRepository,
    pub reservations: ReservationRepository,
}

#[derive(Clone)]
pub struct AppStates {
    pub db_pool: Database,
    pub redis_client: CacherRedis,
    pub services: Services,
    pub repositories: Repositores,
}

pub async fn generate_states(database_url: &str, redis_url: &str) -> AppStates {
    let connection_pool = Database::new(database_url).await;
    let cacher = CacherRedis::new(redis_url).await;

    let reservation_repository = ReservationRepository::new(connection_pool.clone());
    let event_repository = EventRepository::new(connection_pool.clone());

    AppStates {
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
    }
}

pub fn run(listener: TcpListener, app_states: AppStates) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(app_states.clone()))
            .configure(configure_app)
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub fn configure_app(app: &mut web::ServiceConfig) {
    app.configure(auth_config)
        .route("/api/v1/reservations/paied", web::post().to(paid_reservation_webhook))
        .route("/api/v1/health", web::get().to(health_check))
        .service(
            web::scope("/api/v1")
                .wrap(actix_web::middleware::from_fn(auth))
                .configure(reservations_config)
                .configure(events_config),
        );
}
