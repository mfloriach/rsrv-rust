use anyhow::{Context, Result, ensure};
use dotenvy::dotenv;
use secrecy::ExposeSecret;
use secrecy::Secret;
use serde::Deserialize;
use validator::{Validate, ValidationError};

/// Runtime configuration loaded from environment variables.
#[derive(Debug)]
pub struct Configuration {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
}

#[derive(Deserialize, Debug, Validate)]
pub struct DatabaseConfig {
    #[validate(custom(function = "validate_non_blank"))]
    pub username: String,
    pub password: Secret<String>,
    #[validate(range(min = 1))]
    pub port: u16,
    #[validate(custom(function = "validate_non_blank"))]
    pub host: String,
    #[validate(custom(function = "validate_non_blank"))]
    pub database: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct RedisConfig {
    #[validate(custom(function = "validate_non_blank"))]
    host: String,
    #[validate(range(min = 1))]
    port: u16,
}

#[derive(Deserialize, Debug, Validate)]
pub struct JwtConfig {
    pub secret: Secret<String>,
    #[validate(range(min = 1))]
    pub expiration_seconds: u64,
}

fn validate_non_blank(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() { Err(ValidationError::new("blank")) } else { Ok(()) }
}

impl Configuration {
    /// Loads and validates service configuration from the environment.
    ///
    /// # Errors
    ///
    /// Returns an error when a required environment variable is missing, malformed,
    /// or when a required credential is blank.
    pub fn new() -> Result<Self> {
        dotenv().ok();

        let database = envy::prefixed("POSTGRES_")
            .from_env::<DatabaseConfig>()
            .context("failed to load POSTGRES_ configuration")?;
        let redis = envy::prefixed("REDIS_")
            .from_env::<RedisConfig>()
            .context("failed to load REDIS_ configuration")?;
        let jwt = envy::prefixed("JWT_")
            .from_env::<JwtConfig>()
            .context("failed to load JWT_ configuration")?;

        database.validate().context("invalid POSTGRES_ configuration")?;
        ensure!(
            !database.password.expose_secret().trim().is_empty(),
            "POSTGRES_PASSWORD must not be blank"
        );
        redis.validate().context("invalid REDIS_ configuration")?;
        jwt.validate().context("invalid JWT_ configuration")?;
        ensure!(!jwt.secret.expose_secret().trim().is_empty(), "JWT_SECRET must not be blank");

        Ok(Self { database, redis, jwt })
    }
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
    use super::{DatabaseConfig, JwtConfig, RedisConfig};
    use secrecy::{ExposeSecret, Secret};
    use validator::Validate;

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

    #[test]
    fn database_config_rejects_blank_required_values() {
        let config = DatabaseConfig {
            username: " ".to_owned(),
            password: Secret::new("secret".to_owned()),
            host: "localhost".to_owned(),
            port: 5432,
            database: "reservations".to_owned(),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn jwt_config_rejects_zero_expiration() {
        let config = JwtConfig { secret: Secret::new(" ".to_owned()), expiration_seconds: 0 };

        assert!(config.validate().is_err());
    }

    #[test]
    fn required_text_validation_rejects_blank_values() {
        assert!(super::validate_non_blank(" ").is_err());
    }

    #[test]
    fn redis_config_rejects_zero_port() {
        let config = RedisConfig { host: "localhost".to_owned(), port: 0 };

        assert!(config.validate().is_err());
    }
}
