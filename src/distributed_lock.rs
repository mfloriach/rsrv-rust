use redis::Client;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum LockError {
    #[error("Failed to acquire lock")]
    AcquisitionFailed,

    #[error("Lock not held by this owner")]
    NotOwner,

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
}

pub struct DistributedLock {
    client: Client,
    lock_key: String,
    owner_id: Uuid,
    ttl: Duration,
}

impl DistributedLock {
    pub fn new(client: Client, owner_id: Uuid, lock_key: String, ttl: Duration) -> Self {
        Self { client, lock_key, owner_id, ttl }
    }

    pub async fn acquire(&self) -> Result<(), LockError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let result: Option<String> = redis::cmd("SET")
            .arg(&self.lock_key)
            .arg(self.owner_id.to_string())
            .arg("NX")
            .arg("PX")
            .arg(self.ttl.as_millis() as u64)
            .query_async(&mut conn)
            .await?;

        if result.is_some() { Ok(()) } else { Err(LockError::AcquisitionFailed) }
    }

    pub async fn release(&self) -> Result<(), LockError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
        "#;

        let result: i32 = redis::Script::new(script)
            .key(&self.lock_key)
            .arg(self.owner_id.to_string())
            .invoke_async(&mut conn)
            .await?;

        if result == 0 { Err(LockError::NotOwner) } else { Ok(()) }
    }
}
