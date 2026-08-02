use crate::jwt::verify_token;
use actix_web::HttpMessage;
use actix_web::{
    Error, HttpResponse,
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header::HeaderValue,
    middleware::Next,
};
use std::fmt;
use uuid::Uuid;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct UserId(pub Uuid);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Parses a bearer token without allocating an intermediate collection.
fn parse_bearer_token(header: &HeaderValue) -> Option<&str> {
    header
        .to_str()
        .ok()
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|token| !token.is_empty())
}

pub async fn auth(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let token = match req.headers().get("Authorization").and_then(parse_bearer_token) {
        Some(token) => token,
        None => return Ok(req.into_response(HttpResponse::Unauthorized().finish())),
    };

    match verify_token(token) {
        Ok(sub) => {
            req.extensions_mut().insert(UserId(sub));
            next.call(req).await
        }
        Err(_) => Ok(req.into_response(HttpResponse::Unauthorized().finish())),
    }
}

#[cfg(test)]
mod tests {
    use super::{UserId, auth, parse_bearer_token};
    use crate::jwt::{generate_token, initialize_jwt_secret};
    use actix_web::{
        App, Error, HttpMessage, HttpRequest, HttpResponse,
        body::BoxBody,
        dev::{Service, ServiceResponse},
        http::{StatusCode, header::HeaderValue},
        middleware, test, web,
    };

    #[actix_web::test]
    async fn bearer_parser_borrows_valid_token_without_allocating() {
        let header = HeaderValue::from_static("Bearer token");

        assert_eq!(parse_bearer_token(&header), Some("token"));
    }

    #[actix_web::test]
    async fn bearer_parser_rejects_malformed_headers() {
        for value in ["", "Basic token", "Bearer ", "bearer token"] {
            let header = HeaderValue::from_static(value);
            assert_eq!(parse_bearer_token(&header), None);
        }
    }

    async fn protected(req: HttpRequest) -> HttpResponse {
        let user_id = req.extensions().get::<UserId>().copied().expect("user id should be set");

        HttpResponse::Ok().body(user_id.to_string())
    }

    async fn spawn_app()
    -> impl Service<actix_http::Request, Response = ServiceResponse<BoxBody>, Error = Error> {
        test::init_service(
            App::new()
                .wrap(middleware::from_fn(auth))
                .route("/protected", web::get().to(protected)),
        )
        .await
    }

    #[actix_web::test]
    async fn rejects_requests_without_an_authorization_header() {
        let app = spawn_app().await;

        let response =
            test::call_service(&app, test::TestRequest::get().uri("/protected").to_request()).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn rejects_requests_with_an_invalid_token() {
        let app = spawn_app().await;

        let request = test::TestRequest::get()
            .uri("/protected")
            .insert_header(("Authorization", "Bearer invalid-token"))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn rejects_requests_with_an_empty_bearer_token() {
        let app = spawn_app().await;

        let request = test::TestRequest::get()
            .uri("/protected")
            .insert_header(("Authorization", "Bearer "))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn rejects_requests_with_a_non_utf8_authorization_header() {
        let app = spawn_app().await;
        let header_value = HeaderValue::from_bytes(b"Bearer \xFF")
            .expect("header value should allow non-UTF-8 bytes");

        let request = test::TestRequest::get()
            .uri("/protected")
            .insert_header(("Authorization", header_value))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn allows_requests_with_a_valid_token_and_sets_the_user_id() {
        let user_id = uuid::Uuid::now_v7();
        initialize_jwt_secret(secrecy::SecretString::new("test-secret".to_owned()))
            .expect("JWT secret should be initialized");
        let token = generate_token("user@example.com".to_string(), user_id)
            .expect("token should be generated");
        let app = spawn_app().await;

        let request = test::TestRequest::get()
            .uri("/protected")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let response = test::call_service(&app, request).await;
        let body = test::read_body(response).await;

        assert_eq!(body.as_ref(), user_id.to_string().as_bytes());
    }
}
