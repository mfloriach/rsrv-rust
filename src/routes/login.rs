use crate::errors::AppError;
use crate::hash::{hash_password, verify_password};
use crate::infrastructure::server::AppState;
use crate::jwt::generate_token;
use actix_web::{HttpResponse, post, web};
use actix_web_validator::Json;
use anyhow::Result;
use metrics::counter;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use validator::Validate;

#[derive(Deserialize, Validate, Debug, Serialize)]
#[cfg_attr(debug_assertions, derive(utoipa::ToSchema))]
pub struct SignInRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(
        min = 6,
        max = 20,
        message = "Password must be between 6 and 20 characters"
    ))]
    pub password: String,
}

#[derive(Deserialize, Validate, Debug, Serialize)]
#[cfg_attr(debug_assertions, derive(utoipa::ToSchema))]
pub struct SignUpRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(
        min = 3,
        max = 20,
        message = "Username must be between 3 and 20 characters"
    ))]
    pub username: String,

    #[validate(length(
        min = 6,
        max = 20,
        message = "Password must be between 6 and 20 characters"
    ))]
    pub password: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(utoipa::ToSchema))]
pub struct SignInResponse {
    pub email: String,
    pub token: String,
}

#[cfg_attr(debug_assertions, utoipa::path(
    post,
    path = "/api/v1/auth/sign_in",
    request_body = SignInRequest,
    responses(
        (status = 200, description = "User signed in", body = SignInResponse),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "auth"
))]
#[post("/sign_in")]
#[instrument(name = "auth.sign_in", skip_all, fields(authenticated = false))]
pub async fn sign_in(
    payload: Json<SignInRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = state
        .repositories
        .users
        .find_by_email(&payload.email, state.db_pool.get_connection())
        .await?
        .ok_or_else(|| {
            counter!("auth_sign_in_total", "outcome" => "failure").increment(1);
            AppError::Unauthorized
        })?;

    let valid = verify_password(&payload.password, &user.password.into_boxed_str())
        .map_err(AppError::Internal)?;
    if !valid {
        counter!("auth_sign_in_total", "outcome" => "failure").increment(1);
        return Err(AppError::Unauthorized);
    }

    let token = generate_token(&payload.email, user.id)?;
    counter!("auth_sign_in_total", "outcome" => "success").increment(1);
    tracing::Span::current().record("authenticated", true);
    let response = SignInResponse { email: payload.email.clone(), token };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg_attr(debug_assertions, utoipa::path(
    post,
    path = "/api/v1/auth/sign_up",
    request_body = SignUpRequest,
    responses(
        (status = 201, description = "User created"),
        (status = 400, description = "Invalid request")
    ),
    tag = "auth"
))]
#[post("/sign_up")]
#[instrument(name = "auth.sign_up", skip_all, fields(created = false))]
pub async fn sign_up(
    payload: Json<SignUpRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    state
        .repositories
        .users
        .create(
            &payload.username,
            &payload.email,
            &hash_password(&payload.password)?,
            state.db_pool.get_connection(),
        )
        .await?;

    counter!("auth_sign_up_total", "outcome" => "success").increment(1);
    tracing::Span::current().record("created", true);
    Ok(HttpResponse::Created().finish())
}

#[cfg(test)]
mod tests {
    use super::{SignInRequest, SignUpRequest};
    use validator::Validate;

    #[test]
    fn sign_in_request_rejects_invalid_email_and_short_password() {
        let request =
            SignInRequest { email: "not-an-email".to_owned(), password: "short".to_owned() };

        assert!(request.validate().is_err());
    }

    #[test]
    fn sign_up_request_accepts_valid_credentials() {
        let request = SignUpRequest {
            email: "user@example.com".to_owned(),
            username: "user".to_owned(),
            password: "correct-password".to_owned(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn sign_up_request_rejects_short_username() {
        let request = SignUpRequest {
            email: "user@example.com".to_owned(),
            username: "x".to_owned(),
            password: "correct-password".to_owned(),
        };

        assert!(request.validate().is_err());
    }
}
