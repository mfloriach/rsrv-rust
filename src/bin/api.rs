use rsv::configuration::Configuration;
use rsv::infrastructure::logger::init_logger;
use rsv::infrastructure::server::{AppState, run};
use rsv::jwt::initialize_jwt_config;
use secrecy::ExposeSecret;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    let configuration = Configuration::new().expect("Failed to read configuration.");
    initialize_jwt_config(configuration.jwt.secret.clone(), configuration.jwt.expiration_seconds)
        .expect("Failed to initialize JWT configuration.");

    let app_state = AppState::new(
        configuration.database.connection_string().expose_secret(),
        configuration.redis.connection_string().expose_secret(),
    )
    .await
    .map_err(std::io::Error::other)?;
    let shutdown_state = app_state.clone();

    let listener = TcpListener::bind(format!("{}:{}", "localhost", 8080))?;

    let server = run(listener, app_state)?;
    let server_handle = server.handle();

    let result = tokio::select! {
        result = server => result,

        signal = tokio::signal::ctrl_c() => {
            signal.map_err(std::io::Error::other)?;
            tracing::info!("shutdown signal received");
            server_handle.stop(true).await;
            Ok(())
        }
    };

    shutdown_state.shutdown().await;
    result
}
