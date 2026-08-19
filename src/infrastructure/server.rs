use crate::infrastructure::cache::CacherRedis;
use crate::infrastructure::database::{Database, DatabaseOptions};
use crate::repositories::{
    EventRepository, IdempotencyRepository, OutboxRepository, ReservationRepository,
    SeatsRepository, UserRepository,
};
use crate::routes::configure_app;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use anyhow::Result;
use std::net::TcpListener;
use std::time::Duration;
use tracing_actix_web::TracingLogger;

#[derive(Clone)]
pub struct Repositories {
    pub users: UserRepository,
    pub events: EventRepository,
    pub reservations: ReservationRepository,
    pub outbox: OutboxRepository,
    pub seats: SeatsRepository,
    pub idempotency: IdempotencyRepository,
}

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Database,
    pub redis_client: CacherRedis,
    pub repositories: Repositories,
}

impl AppState {
    /// Builds the application state and establishes its shared infrastructure.
    pub async fn new(database_url: &str, redis_url: &str) -> Result<Self> {
        let db_options = DatabaseOptions::builder()
            .min_connections(5)
            .idle_timeout(Duration::from_secs(10 * 60))
            .build()
            .expect("all database options have defaults");
        let db_pool = Database::connect(database_url, db_options).await?;

        let redis_client = CacherRedis::new(redis_url).await;

        Ok(Self {
            db_pool,
            redis_client,
            repositories: Repositories {
                users: UserRepository,
                events: EventRepository,
                reservations: ReservationRepository,
                outbox: OutboxRepository,
                seats: SeatsRepository,
                idempotency: IdempotencyRepository,
            },
        })
    }

    pub async fn shutdown(self) {
        self.db_pool.disconnect().await;
        drop(self.redis_client);
    }
}

pub fn run(listener: TcpListener, app_state: AppState) -> Result<Server, std::io::Error> {
    let app_state = web::Data::new(app_state);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(app_state.clone())
            .configure(configure_app)
    })
    .listen(listener)?
    .run();

    Ok(server)
}
