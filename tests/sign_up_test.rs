use actix_web::{http::StatusCode, test};
use rsv::routes::SignUpRequest;

mod helper;
use helper::{post_json, sign_up, spawn_app};

#[actix_web::test]
async fn test_sign_up_email_does_not_exist() {
    let (app, _container) = spawn_app().await;

    let request = SignUpRequest {
        email: "signup@gmail.com".into(),
        password: "asdfgbvc123458hh".into(),
        username: "signup_user".into(),
    };

    let status_code = sign_up(&app, &request).await;
    assert_eq!(status_code, StatusCode::CREATED);
}

#[actix_web::test]
async fn test_sign_up_email_already_exists() {
    let (app, _container) = spawn_app().await;

    let request = SignUpRequest {
        email: "existing@gmail.com".into(),
        password: "asdfgbvc123458hh".into(),
        username: "existing_user".into(),
    };

    assert_eq!(sign_up(&app, &request).await, StatusCode::CREATED);

    let duplicate_request = SignUpRequest {
        email: "existing@gmail.com".into(),
        password: "asdfgbvc123458hh".into(),
        username: "another_user".into(),
    };

    assert_eq!(sign_up(&app, &duplicate_request).await, StatusCode::INTERNAL_SERVER_ERROR);
}

#[actix_web::test]
async fn test_sign_up_validation_fails() {
    let (app, _container) = spawn_app().await;

    let request = serde_json::json!({
        "email": "not-an-email",
        "username": "ab",
        "password": "123"
    });

    let req = post_json("/api/v1/auth/sign_up", &request, None);
    let response = test::call_service(&app, req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
