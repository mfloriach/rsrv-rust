use secrecy::Secret;
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub password: Secret<String>,
}
