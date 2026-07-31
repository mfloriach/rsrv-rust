use actix_web::{http::StatusCode, test};
use rsv::routes::SignInResponse;
use rsv::routes::{SignInRequest, SignUpRequest};
mod helper;
use helper::{post_json, spawn_app};

#[actix_web::test]
async fn test_login_success() {
    let (app, _container) = spawn_app().await;

    let email = "test@gmail.com";
    let password = "asdfgbvc123458hh";

    let req_signup =
        SignUpRequest { email: email.into(), password: password.into(), username: "test".into() };
    let status_code = test_sign_up(&app, &req_signup).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let req_signin = SignInRequest { email: email.into(), password: password.into() };
    let resp_signin: SignInResponse = test_sign_in(&app, &req_signin).await;
    assert_eq!(req_signin.email, resp_signin.email);
    assert!(!resp_signin.token.is_empty());
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

    test::call_and_read_body_json(&app, req).await
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
