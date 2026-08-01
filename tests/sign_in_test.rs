use actix_web::{http::StatusCode, test};
use rsv::routes::{SignInRequest, SignInResponse, SignUpRequest};

mod helper;
use helper::{post_json, spawn_app};

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
    let status_code = test_sign_up(&app, &sign_up_request).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let sign_in_request = SignInRequest { email: email.into(), password: password.into() };
    let response: SignInResponse = test_sign_in(&app, &sign_in_request).await;

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

    let req = post_json("/api/v1/auth/sign_in", &request);
    let response = test::call_service(&app, req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_sign_in_email_not_found() {
    let (app, _container) = spawn_app().await;

    let request =
        SignInRequest { email: "missing@example.com".into(), password: "valid-password".into() };

    let req = post_json("/api/v1/auth/sign_in", &request);
    let response = test::call_service(&app, req).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

async fn test_sign_in<S>(app: &S, request: &SignInRequest) -> SignInResponse
where
    S: actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse<
                tracing_actix_web::StreamSpan<actix_http::body::BoxBody>,
            >,
            Error = actix_web::Error,
        >,
{
    let req = post_json("/api/v1/auth/sign_in", request);

    test::call_and_read_body_json(app, req).await
}

async fn test_sign_up<S>(app: &S, request: &SignUpRequest) -> StatusCode
where
    S: actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse<
                tracing_actix_web::StreamSpan<actix_http::body::BoxBody>,
            >,
            Error = actix_web::Error,
        >,
{
    let req = post_json("/api/v1/auth/sign_up", request);
    let resp = test::call_service(app, req).await;

    resp.status()
}
