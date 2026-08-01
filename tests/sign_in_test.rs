use actix_web::{http::StatusCode, test};
use rsv::routes::{SignInRequest, SignInResponse, SignUpRequest};

mod helper;
use helper::{post_json, spawn_app};
use helper::{sign_in, sign_up};

#[actix_web::test]
async fn test_sign_in_success() {
    let (app, _container) = spawn_app().await;

    let email = "signin@gmail.com";
    let password = "asdfgbvc123458hh";

    let sign_up_request = SignUpRequest {
        email: email.into(),
        password: password.into(),
        username: "signin_user".into(),
    };
    let status_code = sign_up(&app, &sign_up_request).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let sign_in_request = SignInRequest { email: email.into(), password: password.into() };
    let response: SignInResponse = sign_in(&app, &sign_in_request).await;

    assert_eq!(sign_in_request.email, response.email);
    assert!(!response.token.is_empty());
}

#[actix_web::test]
async fn test_sign_in_validation_fails() {
    let (app, _container) = spawn_app().await;

    let request = serde_json::json!({
        "email": "not-an-email",
        "password": "123"
    });

    let req = post_json("/api/v1/auth/sign_in", &request, None);
    let response = test::call_service(&app, req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_sign_in_email_not_found() {
    let (app, _container) = spawn_app().await;

    let request =
        SignInRequest { email: "missing@example.com".into(), password: "valid-password".into() };

    let req = post_json("/api/v1/auth/sign_in", &request, None);
    let response = test::call_service(&app, req).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
