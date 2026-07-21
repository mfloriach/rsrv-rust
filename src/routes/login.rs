use crate::AppStates;
use crate::errors::AppError;
use crate::hash::{hash_password, verify_password};
use crate::jwt::generate_token;
use crate::models::User;
use actix_web::{HttpResponse, web};
use actix_web_validator::Json;
use anyhow::Result;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, Debug, Serialize)]
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

#[derive(Deserialize, Validate, Debug)]
pub struct SignUpRequest {
    #[validate(email(message = "Invalid email format"))]
    email: String,

    #[validate(length(
        min = 3,
        max = 20,
        message = "Username must be between 3 and 20 characters"
    ))]
    username: String,

    #[validate(length(
        min = 6,
        max = 20,
        message = "Password must be between 6 and 20 characters"
    ))]
    password: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignInResponse {
    pub email: String,
    pub token: String,
}

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

pub async fn sign_up(payload: Json<SignUpRequest>, state: web::Data<AppStates>) -> HttpResponse {
    match sqlx::query!(
        r#"
        INSERT INTO users (id, name, email, password)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::now_v7(),
        payload.username,
        payload.email,
        hash_password(&payload.password).expect("dfds")
    )
    .execute(state.db_pool.get_connection())
    .await
    {
        Ok(_) => {
            tracing::info!("Subscription saved successfully");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn hello() -> HttpResponse {
    HttpResponse::Ok().into()
}
