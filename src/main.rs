use zero2prod::database::Database;
mod configuration;
use configuration::get_configuration;
use secrecy::ExposeSecret;
use zero2prod::email_client::EmailClient;
pub mod cache;
use zero2prod::cache::CacherRedis;
mod queues;
use zero2prod::{DependyInjections, run};
// use crate::queues::QueueKafka;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // send_to_queue().await;
    // consume_from_queue().await;

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool =
        Database::new(configuration.database.get_connection_string().expose_secret()).await;

    // let queue_kafka = QueueKafka::new("localhost:9092");
    let email_client = EmailClient::new("https://pokeapi.co");
    let cacher =
        CacherRedis::new(configuration.redis.get_connection_string().expose_secret()).await;

    let dependency_injection =
        DependyInjections { db_pool: connection_pool, email_client, redis_client: cacher };

    run(dependency_injection)?.await
}
