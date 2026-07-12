use sqlx::Error;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

#[derive(Debug)]
pub struct Database {
    conn: PgPool,
}

pub struct DatabaseConfig {
    pub max_connection: i32,
    pub min_connection: i32,
    pub timeout: Duration,
}

impl Database {
    pub async fn new(url: &str) -> Self {
        let connection_pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(Duration::from_secs(2))
            .connect(url)
            .await
            .expect("Failed to connect to Postgres.");

        Self { conn: connection_pool }
    }

    pub async fn ping(&self) -> Result<(), Error> {
        sqlx::query("SELECT 1").execute(&self.conn).await?;

        Ok(())
    }

    pub fn get_connection(&self) -> &PgPool {
        &self.conn
    }

    pub async fn disconnect(&self) {
        self.conn.close().await;
    }
}
