use derive_builder::Builder;
use sqlx::Error;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

const DEFAULT_MAX_CONNECTIONS: u32 = 10;
const DEFAULT_MIN_CONNECTIONS: u32 = 2;
const DEFAULT_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Debug, Clone)]
pub struct Database {
    conn: PgPool,
}

/// Options used to construct a PostgreSQL connection pool.
#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "String"))]
pub struct DatabaseOptions {
    #[builder(default = "DEFAULT_MAX_CONNECTIONS")]
    max_connections: u32,
    #[builder(default = "DEFAULT_MIN_CONNECTIONS")]
    min_connections: u32,
    #[builder(default = "DEFAULT_ACQUIRE_TIMEOUT")]
    acquire_timeout: Duration,
    #[builder(default = "Some(DEFAULT_IDLE_TIMEOUT)", setter(strip_option))]
    idle_timeout: Option<Duration>,
}

impl DatabaseOptions {
    #[must_use]
    pub fn builder() -> DatabaseOptionsBuilder {
        DatabaseOptionsBuilder::default()
    }

    fn pool_options(&self) -> PgPoolOptions {
        let options = PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(self.acquire_timeout);

        match self.idle_timeout {
            Some(timeout) => options.idle_timeout(timeout),
            None => options,
        }
    }
}

impl Default for DatabaseOptions {
    fn default() -> Self {
        Self {
            max_connections: DEFAULT_MAX_CONNECTIONS,
            min_connections: DEFAULT_MIN_CONNECTIONS,
            acquire_timeout: DEFAULT_ACQUIRE_TIMEOUT,
            idle_timeout: Some(DEFAULT_IDLE_TIMEOUT),
        }
    }
}

impl Database {
    /// Connects to PostgreSQL with the default pool options.
    pub async fn new(url: &str) -> Result<Self, Error> {
        Self::connect(url, DatabaseOptions::default()).await
    }

    /// Connects to PostgreSQL with caller-provided pool options.
    pub async fn connect(url: &str, options: DatabaseOptions) -> Result<Self, Error> {
        let connection_pool = options.pool_options().connect(url).await?;

        Ok(Self { conn: connection_pool })
    }

    pub async fn ping(&self) -> Result<(), Error> {
        if self.conn.is_closed() {
            return Err(Error::PoolClosed);
        }

        sqlx::query("SELECT 1").execute(&self.conn).await.map(|_| ())
    }

    pub fn get_connection(&self) -> &PgPool {
        &self.conn
    }

    pub async fn disconnect(&self) {
        if !self.conn.is_closed() {
            self.conn.close().await;
            tracing::info!("database disconnect");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_ACQUIRE_TIMEOUT, DEFAULT_IDLE_TIMEOUT, DatabaseOptions};
    use std::time::Duration;

    #[test]
    fn options_builder_starts_with_safe_defaults() {
        let options =
            DatabaseOptions::builder().build().expect("all database options have defaults");

        assert_eq!(options.max_connections, 10);
        assert_eq!(options.min_connections, 2);
        assert_eq!(options.acquire_timeout, DEFAULT_ACQUIRE_TIMEOUT);
        assert_eq!(options.idle_timeout, Some(DEFAULT_IDLE_TIMEOUT));
    }

    #[test]
    fn options_builder_overrides_pool_settings() {
        let options = DatabaseOptions::builder()
            .max_connections(20)
            .min_connections(4)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(600))
            .build()
            .expect("all database options have defaults");

        assert_eq!(options.max_connections, 20);
        assert_eq!(options.min_connections, 4);
        assert_eq!(options.acquire_timeout, Duration::from_secs(5));
        assert_eq!(options.idle_timeout, Some(Duration::from_secs(600)));
    }
}
