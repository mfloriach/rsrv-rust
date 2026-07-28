use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use std::net::TcpListener;
pub mod cache;
pub mod configuration;
pub mod database;
pub mod distributed_lock;
pub mod errors;
pub mod hash;
pub mod jwt;
pub mod kafka;
pub mod logger;
pub mod middlewares;
pub mod models;
pub mod queues;
pub mod repositories;
pub mod routes;
pub mod services;
use cache::CacherRedis;
use database::Database;
use middlewares::auth;
use routes::{
    auth_config, events_config, health_check, paid_reservation_webhook, reservations_config,
};
use services::ReservationService;
use tracing_actix_web::TracingLogger;

#[derive(Clone)]
pub struct Services {
    pub reservations: ReservationService,
}

#[derive(Clone)]
pub struct AppStates {
    pub db_pool: Database,
    pub redis_client: CacherRedis,
    pub services: Services,
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
