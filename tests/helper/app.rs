use actix_web::{App, test, web};
use rsv::infrastructure::database::Database;
use rsv::infrastructure::logger::init_logger;
use rsv::infrastructure::server::AppState;
use rsv::jwt::initialize_jwt_config;
use rsv::routes::configure_app;
use secrecy::SecretString;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;
use tracing_actix_web::TracingLogger;

pub async fn spawn_app() -> (
    impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<
            tracing_actix_web::StreamSpan<actix_http::body::BoxBody>,
        >,
        Error = actix_web::Error,
    >,
    testcontainers::ContainerAsync<Postgres>,
) {
    let (db, connection_string) = start_postgres().await;
    let (_redis, redis_url) = start_redis().await;

    initialize_jwt_config(SecretString::new("test-secret".to_owned()), 86_400)
        .expect("JWT secret should be initialized");

    let state =
        AppState::new(&connection_string, &redis_url).await.expect("could not connect to postgres");

    migrate(&connection_string).await;
    init_logger();

    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(state))
            .configure(configure_app),
    )
    .await;

    (app, db)
}

async fn migrate(connection_string: &str) {
    let connection_pool =
        Database::new(connection_string).await.expect("could not connect to postgres");
    sqlx::query("CREATE EXTENSION IF NOT EXISTS pgcrypto")
        .execute(connection_pool.get_connection())
        .await
        .expect("could not apply pgcrypto extension");

    sqlx::migrate!("./migrations")
        .run(connection_pool.get_connection())
        .await
        .expect("could not migrate");
}

async fn start_postgres() -> (testcontainers::ContainerAsync<Postgres>, String) {
    let db = Postgres::default().start().await.expect("could not start postgres");

    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        db.get_host_port_ipv4(5432).await.expect("could not get db port")
    );

    (db, connection_string)
}

async fn start_redis() -> (testcontainers::ContainerAsync<Redis>, String) {
    let redis = Redis::default().start().await.expect("could not start redis");

    let host = redis.get_host().await.expect("could not get redis host");
    let port = redis.get_host_port_ipv4(6379).await.expect("could not get redis port");

    (redis, format!("redis://{}:{}/", host, port))
}
