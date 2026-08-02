use crate::infrastructure::cache::CacherRedis;
use crate::infrastructure::database::{Database, DatabaseOptions};
use crate::repositories::{EventRepository, ReservationRepository};
use crate::routes::configure_app;
use crate::services::ReservationService;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use anyhow::Result;
use std::net::TcpListener;
use std::time::Duration;
use tracing_actix_web::TracingLogger;

#[derive(Clone)]
pub struct Services {
    pub reservations: ReservationService,
}

#[derive(Clone)]
pub struct Repositories {
    pub events: EventRepository,
    pub reservations: ReservationRepository,
}

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Database,
    pub redis_client: CacherRedis,
    pub services: Services,
    pub repositories: Repositories,
}

impl AppState {
    /// Closes shared database resources during application shutdown.
    pub async fn shutdown(self) {
        self.db_pool.disconnect().await;
        drop(self.redis_client);
    }
}

impl AppState {
    /// Builds the application state and establishes its shared infrastructure.
    pub async fn new(database_url: &str, redis_url: &str) -> Result<Self> {
        let db_options = DatabaseOptions::builder()
            .min_connections(5)
            .idle_timeout(Duration::from_secs(10 * 60))
            .build();
        let db_pool = Database::connect(database_url, db_options).await?;

        let cacher = CacherRedis::new(redis_url).await;

        let reservation_repository = ReservationRepository::new(db_pool.clone());
        let event_repository = EventRepository::new(db_pool.clone());

        Ok(Self {
            db_pool,
            redis_client: cacher.clone(),
            services: Services {
                reservations: ReservationService::new(
                    cacher.client.clone(),
                    reservation_repository.clone(),
                ),
            },
            repositories: Repositories {
                events: event_repository,
                reservations: reservation_repository,
            },
        })
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
