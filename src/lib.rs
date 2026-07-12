use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use std::net::TcpListener;
pub mod cache;
pub mod database;
pub mod distributed_lock;
pub mod domain;
pub mod email_client;
pub mod jwt;
pub mod middlewares;
pub mod routes;
use cache::CacherRedis;
use database::Database;
use email_client::EmailClient;
use middlewares::auth;
use routes::{events_config, health_check, reservations_config, sign_in, sign_up};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct DependyInjections {
    pub db_pool: Database,
    pub email_client: EmailClient,
    pub redis_client: CacherRedis,
}

pub fn run(conf: DependyInjections) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(conf.db_pool);
    let email_client = web::Data::new(conf.email_client);
    let redis_client = web::Data::new(conf.redis_client);

    init_logger();

    let listener =
        TcpListener::bind(format!("{}:{}", "localhost", 8080)).expect("Failed to bind address");

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(redis_client.clone())
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

fn init_logger() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json().with_span_list(false))
        .init();
}
