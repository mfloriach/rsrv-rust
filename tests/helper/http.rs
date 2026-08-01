use actix_web::http::header::{AUTHORIZATION, ContentType};
use actix_web::{http::StatusCode, test};
use rsv::routes::{SignInRequest, SignInResponse, SignUpRequest};

pub fn post_json<T: serde::Serialize>(
    uri: &str,
    body: &T,
    bearer_token: Option<&str>,
) -> actix_http::Request {
    let mut request = test::TestRequest::post().uri(uri).insert_header(ContentType::json());

    if let Some(token) = bearer_token {
        request = request.insert_header((AUTHORIZATION, format!("Bearer {token}")));
    }

    request.set_json(body).to_request()
}

pub async fn sign_in<S>(app: &S, request: &SignInRequest) -> SignInResponse
where
    S: actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse<
                tracing_actix_web::StreamSpan<actix_http::body::BoxBody>,
            >,
            Error = actix_web::Error,
        >,
{
    let req = post_json("/api/v1/auth/sign_in", request, None);

    test::call_and_read_body_json(app, req).await
}

pub async fn sign_up<S>(app: &S, request: &SignUpRequest) -> StatusCode
where
    S: actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse<
                tracing_actix_web::StreamSpan<actix_http::body::BoxBody>,
            >,
            Error = actix_web::Error,
        >,
{
    let req = post_json("/api/v1/auth/sign_up", request, None);
    let resp = test::call_service(app, req).await;

    resp.status()
}

pub async fn create_user<S>(app: &S) -> String
where
    S: actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse<
                tracing_actix_web::StreamSpan<actix_http::body::BoxBody>,
            >,
            Error = actix_web::Error,
        >,
{
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
    response.token
}
