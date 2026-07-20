use actix_http;
use actix_web::http::header::ContentType;
use actix_web::{App, http::StatusCode, test, web};
use anyhow::Context;
use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use zero2prod::database::Database;
use zero2prod::routes::SignInResponse;
use zero2prod::routes::{SignInRequest, sign_in};

#[actix_web::test]
async fn test_login() {
    let (app, container) = spawn_app().await;

    let request =
        SignInRequest { email: "test@gmail.com".into(), password: "asdfgbvc123458hh".into() };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/sign_in")
        .insert_header(ContentType::json())
        .set_json(&request)
        .to_request();

    let resp: SignInResponse = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.email, request.email);
    assert!(!resp.token.is_empty());
}

#[actix_web::test]
async fn login_fails_with_invalid_password() {
    let (app, container) = spawn_app().await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/sign_in")
        .insert_header(ContentType::json())
        .set_json(&SignInRequest {
            email: "mfloriach@example.com".into(),
            password: "wrong-password".into(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

async fn spawn_app() -> (
    impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<
            tracing_actix_web::StreamSpan<actix_http::body::BoxBody>,
        >,
        Error = actix_web::Error,
    >,
    testcontainers::ContainerAsync<Postgres>,
) {
    let container = Postgres::default().start().await.expect("could not start postgress container");

    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        container.get_host_port_ipv4(5432).await.expect("could not get database url")
    );

    let pool = PgPool::connect(&connection_string).await.expect("could not get pool");
    let connection_pool = Database::new(&connection_string).await;

    sqlx::migrate!("./migrations").run(&pool).await.expect("could not migrate");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    // insert_user(&pool.clone(), "test@gmail.com", "asdfgbvc123458hh").await;

    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(connection_pool))
            .route("/api/v1/auth/sign_in", web::post().to(sign_in)),
    )
    .await;

    (app, container)
}

// async fn insert_user(pool: &PgPool, email: &str, password: &str) {
//     let _ = sqlx::query!(
//         r#"
//         INSERT INTO users (id, name, email, password)
//         VALUES ($1, $2, $3, $4)
//         "#,
//         Uuid::now_v7(),
//         "asdsa",
//         email,
//         hash_password(&password.to_string()).expect("could not hash password")
//     )
//     .execute(pool)
//     .await;
// }

async fn delete_user(pool: &PgPool, email: &str) {
    let _ = sqlx::query!(
        r#"
        DELETE FROM users WHERE email = $1;
        "#,
        email,
    )
    .execute(pool)
    .await
    .context("could not create user");
}
