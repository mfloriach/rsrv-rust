use crate::infrastructure::cache::CacherRedis;
use crate::infrastructure::database::Database;
#[cfg(debug_assertions)]
use crate::openapi::ApiDoc;
use crate::repositories::{EventRepository, ReservationRepository};
use crate::routes::{api_config, auth_config, health_check, paid_reservation_webhook};
use crate::services::ReservationService;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
#[cfg(debug_assertions)]
use utoipa::OpenApi;
#[cfg(debug_assertions)]
use utoipa_scalar::{Scalar, Servable};

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
pub struct AppStates {
    pub db_pool: Database,
    pub redis_client: CacherRedis,
    pub services: Services,
    pub repositories: Repositories,
}

impl AppStates {
    /// Closes shared database resources during application shutdown.
    pub async fn shutdown(self) {
        self.db_pool.disconnect().await;
        drop(self.redis_client);
    }
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
        repositories: Repositories {
            events: event_repository,
            reservations: reservation_repository.clone(),
        },
    }
}

pub fn run(listener: TcpListener, app_states: AppStates) -> Result<Server, std::io::Error> {
    let app_states = web::Data::new(app_states);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(app_states.clone())
            .configure(configure_app)
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub fn configure_app(app: &mut web::ServiceConfig) {
    #[cfg(debug_assertions)]
    app.service(Scalar::with_url("/scalar", ApiDoc::openapi()));

    app.configure(auth_config)
        .service(paid_reservation_webhook)
        .service(health_check)
        .configure(api_config);
}
