use anyhow::Result;
use serde::Serialize;
use sqlx::FromRow;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

#[derive(FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Clone)]
pub struct UserRepository;

impl UserRepository {
    pub async fn create<'e>(
        &self,
        username: &str,
        email: &str,
        hashed_password: &str,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, name, email, password)
            VALUES ($1, $2, $3, $4)
            "#,
            Uuid::now_v7(),
            username,
            email,
            hashed_password
        )
        .execute(executor)
        .await?;

        Ok(())
    }

    pub async fn find_by_email<'e>(
        &self,
        email: &str,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<Option<User>> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
            .fetch_optional(executor)
            .await?;

        Ok(user)
    }
}
