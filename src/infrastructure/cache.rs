use redis::Client;
use redis::ConnectionLike;

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
        let mut con = self.client.get_connection().expect("could not get connection");
        con.check_connection()
    }
}
