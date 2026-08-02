use rsv::configuration::get_configuration;
use rsv::infrastructure::logger::init_logger;
use rsv::jwt::initialize_jwt_secret;
use rsv::server::{AppState, run};
use secrecy::ExposeSecret;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    let configuration = get_configuration().expect("Failed to read configuration.");
    initialize_jwt_secret(configuration.jwt_secret.clone())
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
