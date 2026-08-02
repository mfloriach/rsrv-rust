use redis::Client;

#[derive(Debug, Clone)]
pub struct CacherRedis {
    pub client: Client,
}

impl CacherRedis {
    pub async fn new(address: &str) -> Self {
        let client = Client::open(address).expect("Failed to open Redis connection");
        Self { client }
    }
}

impl CacherRedis {
    pub async fn ping(&self) -> bool {
        let Ok(mut connection) = self.client.get_multiplexed_async_connection().await else {
            return false;
        };

        redis::cmd("PING").query_async::<String>(&mut connection).await.is_ok()
    }
}
