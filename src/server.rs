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

pub fn run(listener: TcpListener, app_states: AppStates) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(app_states.clone()))
            .configure(auth_config)
            .route("/api/v1/reservations/paied", web::post().to(paid_reservation_webhook))
            .route("/api/v1/health", web::get().to(health_check))
            .service(
                web::scope("/api/v1")
                    .wrap(actix_web::middleware::from_fn(auth))
                    .configure(reservations_config)
                    .configure(events_config),
            )
    })
    .listen(listener)?
    .run();

    Ok(server)
}
