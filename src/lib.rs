use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use std::net::TcpListener;
pub mod cache;
pub mod configuration;
pub mod database;
pub mod distributed_lock;
pub mod domain;
pub mod errors;
pub mod jwt;
pub mod middlewares;
pub mod routes;
use cache::CacherRedis;
use database::Database;
use middlewares::auth;
use routes::{events_config, health_check, reservations_config, sign_in, sign_up};
use tracing_actix_web::TracingLogger;

#[derive(Debug, Clone)]
pub struct AppStates {
    pub db_pool: Database,
    pub redis_client: CacherRedis,
}

pub fn run(listener: TcpListener, app_states: AppStates) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(app_states.clone()))
            .route("/api/v1/auth/sign_in", web::post().to(sign_in))
            .route("/api/v1/auth/sign_up", web::post().to(sign_up))
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
