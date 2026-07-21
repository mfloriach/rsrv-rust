use anyhow::Ok;
use anyhow::Result;
use dotenvy::dotenv;
use secrecy::ExposeSecret;
use secrecy::Secret;
use serde::Deserialize;

pub struct Configuration {
    pub database: Database,
    pub redis: Redis,
}

#[derive(Deserialize, Debug)]
pub struct Database {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database: String,
}

#[derive(Deserialize, Debug)]
pub struct Redis {
    host: String,
    port: u16,
}

pub fn get_configuration() -> Result<Configuration> {
    dotenv().ok();

    let database = envy::prefixed("POSTGRES_").from_env::<Database>()?;
    let redis = envy::prefixed("REDIS_").from_env::<Redis>()?;

    Ok(Configuration { database, redis })
}

impl Database {
    pub fn get_connection_string(&self) -> Secret<String> {
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

impl Redis {
    pub fn get_connection_string(&self) -> Secret<String> {
        Secret::new(format!("redis://{}:{}/", self.host, self.port))
    }
}
