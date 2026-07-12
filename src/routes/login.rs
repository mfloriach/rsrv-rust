use crate::jwt::{generate_token, hash_password, verify_password};
use crate::{database::Database, domain::User};
use actix_web::{HttpResponse, web};
use actix_web_validator::Json;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct SignInRequest {
    #[validate(email(message = "Invalid email format"))]
    email: String,

    #[validate(length(
        min = 6,
        max = 20,
        message = "Username must be between 3 and 20 characters"
    ))]
    password: String,
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

#[derive(Serialize)]
pub struct SignInResponse {
    email: String,
    token: String,
}

pub async fn sign_in(payload: Json<SignInRequest>, db: web::Data<Database>) -> HttpResponse {
    let result = match sqlx::query_as!(
        User,
        "SELECT id, email, name, password FROM users WHERE email = $1",
        &payload.email
    )
    .fetch_one(db.get_connection())
    .await
    {
        Ok(user) => user,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid email or password"),
    };

    if let Err(_err) = verify_password(&payload.password, result.password.expose_secret()) {
        return HttpResponse::Unauthorized().body("Invalid email or password");
    }

    let token = match generate_token(payload.email.clone(), result.id) {
        Ok(token) => token,
        Err(_err) => return HttpResponse::InternalServerError().body("Failed to generate token"),
    };

    let response = SignInResponse { email: payload.email.clone(), token };

    HttpResponse::Ok().json(response)
}

pub async fn sign_up(payload: Json<SignUpRequest>, db: web::Data<Database>) -> HttpResponse {
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
    .execute(db.get_connection())
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
