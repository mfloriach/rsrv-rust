use anyhow::{Result, bail};
use redis::Client;
use std::mem::ManuallyDrop;
use std::{marker::PhantomData, time::Duration};
use thiserror::Error;

use crate::types::UserId;

#[derive(Debug, Error)]
pub enum DistributedLockError {
    #[error("Failed to acquire lock")]
    AcquisitionFailed,

    #[error("Lock not held by this owner")]
    NotOwner,

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
}

pub trait LockState {
    const RELEASE_ON_DROP: bool;
}

pub struct Locked;

pub struct Unlocked;

impl LockState for Locked {
    const RELEASE_ON_DROP: bool = true;
}

impl LockState for Unlocked {
    const RELEASE_ON_DROP: bool = false;
}

pub struct DistributedLock<State: LockState = Unlocked> {
    client: Client,
    lock_key: String,
    owner_id: UserId,
    ttl: Duration,

    _state: PhantomData<State>,
}

impl DistributedLock<Unlocked> {
    pub fn new(client: Client, owner_id: UserId, lock_key: String, ttl: Duration) -> Self {
        Self { client, lock_key, owner_id, ttl, _state: PhantomData }
    }
}

impl<S: LockState> DistributedLock<S> {
    fn transition<T: LockState>(self) -> DistributedLock<T> {
        DistributedLock {
            client: self.client.clone(),
            lock_key: self.lock_key.clone(),
            owner_id: self.owner_id,
            ttl: self.ttl,
            _state: PhantomData,
        }
    }
}

impl DistributedLock<Unlocked> {
    pub async fn acquire(self) -> Result<DistributedLock<Locked>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        tracing::debug!("Locked aquire");

        let result: Option<String> = redis::cmd("SET")
            .arg(&self.lock_key)
            .arg(self.owner_id.to_string())
            .arg("NX")
            .arg("PX")
            .arg(self.ttl.as_millis() as u64)
            .query_async(&mut conn)
            .await?;

        if result.is_none() {
            bail!("could not aquire lock");
        }

        Ok(self.transition())
    }
}

impl DistributedLock<Locked> {
    async fn release_inner(client: &Client, key: &str, owner_id: UserId) -> Result<()> {
        let mut conn = client.get_multiplexed_async_connection().await?;

        tracing::debug!("locked released");

        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
        "#;

        let result: i32 = redis::Script::new(script)
            .key(key)
            .arg(owner_id.to_string())
            .invoke_async(&mut conn)
            .await?;

        if result == 0 {
            bail!("it is not the owner");
        }

        Ok(())
    }

    pub async fn release(self) -> Result<DistributedLock<Unlocked>> {
        let this = ManuallyDrop::new(self);

        Self::release_inner(&this.client, &this.lock_key, this.owner_id).await?;

        Ok(DistributedLock {
            client: this.client.clone(),
            lock_key: this.lock_key.clone(),
            owner_id: this.owner_id,
            ttl: this.ttl,
            _state: PhantomData,
        })
    }
}

impl<S: LockState> Drop for DistributedLock<S> {
    fn drop(&mut self) {
        if !S::RELEASE_ON_DROP {
            return;
        }

        let client = self.client.clone();
        let key = self.lock_key.clone();
        let owner = self.owner_id;

        actix_web::rt::spawn(async move {
            tracing::warn!("locked release");
            if let Err(error) = DistributedLock::<Locked>::release_inner(&client, &key, owner).await
            {
                tracing::error!(
                    error = ?error,
                    "failed to release distributed lock"
                );
            }
        });
    }
}
