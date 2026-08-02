use rsv::configuration::get_configuration;
use rsv::infrastructure::logger::init_logger;
use rsv::jwt::initialize_jwt_secret;
use rsv::server::{generate_states, run};
use secrecy::ExposeSecret;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    let configuration = get_configuration().expect("Failed to read configuration.");
    initialize_jwt_secret(configuration.jwt_secret.clone())
        .expect("Failed to initialize JWT configuration.");

    let app_states = generate_states(
        configuration.database.get_connection_string().expose_secret(),
        configuration.redis.get_connection_string().expose_secret(),
    )
    .await;

    let listener = TcpListener::bind(format!("{}:{}", "localhost", 8080))?;

    run(listener, app_states)?.await
}
