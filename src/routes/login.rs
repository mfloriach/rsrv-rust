use crate::errors::AppError;
use crate::hash::{hash_password, verify_password};
use crate::jwt::generate_token;
use crate::models::User;
use crate::server::AppStates;
use actix_web::{HttpResponse, post, web};
use actix_web_validator::Json;
use anyhow::Result;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, Debug, Serialize)]
#[cfg_attr(debug_assertions, derive(utoipa::ToSchema))]
pub struct SignInRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(
        min = 6,
        max = 20,
        message = "Username must be between 3 and 20 characters"
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
pub async fn sign_in(
    payload: Json<SignInRequest>,
    state: web::Data<AppStates>,
) -> Result<HttpResponse, AppError> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, email, name, password FROM users WHERE email = $1",
        &payload.email
    )
    .fetch_optional(state.db_pool.get_connection())
    .await?
    .ok_or(AppError::Unauthorized)?;

    let valid = verify_password(&payload.password, user.password.expose_secret())
        .map_err(AppError::Internal)?;
    if !valid {
        return Err(AppError::Unauthorized);
    }

    let token = generate_token(payload.email.clone(), user.id)?;
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
pub async fn sign_up(
    payload: Json<SignUpRequest>,
    state: web::Data<AppStates>,
) -> Result<HttpResponse, AppError> {
    sqlx::query!(
        r#"
        INSERT INTO users (id, name, email, password)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::now_v7(),
        payload.username,
        payload.email,
        hash_password(&payload.password)?
    )
    .execute(state.db_pool.get_connection())
    .await?;

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
