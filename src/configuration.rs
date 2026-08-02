use anyhow::{Context, Result, ensure};
use dotenvy::dotenv;
use secrecy::ExposeSecret;
use secrecy::Secret;
use serde::Deserialize;

/// Runtime configuration loaded from environment variables.
#[derive(Debug)]
pub struct Configuration {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt_secret: Secret<String>,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database: String,
}

#[derive(Deserialize, Debug)]
pub struct RedisConfig {
    host: String,
    port: u16,
}

/// Loads and validates the service configuration.
///
/// # Errors
///
/// Returns an error when a required environment variable is missing, malformed,
/// or when `JWT_SECRET` is blank.
pub fn get_configuration() -> Result<Configuration> {
    dotenv().ok();

    let database = envy::prefixed("POSTGRES_")
        .from_env::<DatabaseConfig>()
        .context("failed to load POSTGRES_ configuration")?;
    let redis = envy::prefixed("REDIS_")
        .from_env::<RedisConfig>()
        .context("failed to load REDIS_ configuration")?;
    let jwt_secret = envy::prefixed("JWT_")
        .from_env::<JwtConfig>()
        .context("failed to load JWT_ configuration")?
        .secret;
    ensure!(!jwt_secret.expose_secret().trim().is_empty(), "JWT_SECRET must not be blank");

    Ok(Configuration { database, redis, jwt_secret })
}

#[derive(Deserialize, Debug)]
struct JwtConfig {
    secret: Secret<String>,
}

impl DatabaseConfig {
    /// Builds the PostgreSQL URL without exposing its password to callers.
    #[must_use]
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database
        ))
    }
}

impl RedisConfig {
    /// Builds the Redis URL.
    #[must_use]
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!("redis://{}:{}/", self.host, self.port))
    }
}

#[cfg(test)]
mod tests {
    use super::{DatabaseConfig, RedisConfig};
    use secrecy::{ExposeSecret, Secret};

    #[test]
    fn database_connection_string_uses_the_configured_values() {
        let config = DatabaseConfig {
            username: "rsv".to_owned(),
            password: Secret::new("secret".to_owned()),
            host: "localhost".to_owned(),
            port: 5432,
            database: "reservations".to_owned(),
        };

        assert_eq!(
            config.connection_string().expose_secret(),
            "postgres://rsv:secret@localhost:5432/reservations"
        );
    }

    #[test]
    fn redis_connection_string_uses_the_configured_values() {
        let config = RedisConfig { host: "localhost".to_owned(), port: 6379 };

        assert_eq!(config.connection_string().expose_secret(), "redis://localhost:6379/");
    }
}
