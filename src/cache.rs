use redis::Client;
use redis::ConnectionLike;
// use redis::TypedCommands;
// use redis::{aio::MultiplexedConnection, AsyncCommands, Client, RedisResult, Connection};
// use std::option::Option;
// use std::string::String;

// pub trait Cacher {
//     fn ping(&self) -> impl std::future::Future<Output = bool> + Send;
//     async fn set(&self, key: &str, value: &str) -> Result<(), redis::RedisError>;
//     async fn get(&self, key: &str) -> Result<Option<String>, redis::RedisError>;
// }

#[derive(Debug)]
pub struct CacherRedis {
    pub client: Client,
}

impl CacherRedis {
    pub async fn new(address: &str) -> Self {
        let client = Client::open(address).expect("Failed to open Redis connection");
        // let conn = client.get_multiplexed_async_connection().await.expect("error getting connection");
        Self { client }
    }
}

impl CacherRedis {
    pub async fn ping(&self) -> bool {
        let mut con = self.client.get_connection().expect("could not get connection");
        con.check_connection()
    }

    // async fn set(&self, key: &str, value: &str) -> Result<(), redis::RedisError> {
    //     let mut con = self.client.get_connection().expect("could not get connection");
    //     match con.set(key, value) {
    //         Ok(_) => Ok(()),
    //         Err(e) => Err(e),
    //     }
    // }

    // async fn get(&self, key: &str) -> Result<Option<String>, redis::RedisError> {
    //     let mut con = self.client.get_connection().expect("could not get connection");
    //     match con.get(key) {
    //         Ok(value) => Ok(value),
    //         Err(e) => Err(e),
    //     }
    // }
}
