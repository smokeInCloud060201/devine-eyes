use anyhow::Result;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct CacheService {
    client: Option<redis::Client>,
}

impl CacheService {
    pub fn new(redis_url: Option<String>) -> Result<Self> {
        let client = if let Some(url) = redis_url {
            Some(redis::Client::open(url)?)
        } else {
            None
        };

        Ok(Self { client })
    }

    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if let Some(ref client) = self.client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let value: Option<String> = conn.get(key).await?;
            
            if let Some(v) = value {
                let deserialized: T = serde_json::from_str(&v)?;
                return Ok(Some(deserialized));
            }
        }
        Ok(None)
    }

    pub async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize,
    {
        if let Some(ref client) = self.client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let serialized = serde_json::to_string(value)?;
            
            if let Some(ttl_duration) = ttl {
                let _: () = conn.set_ex(key, serialized, ttl_duration.as_secs()).await?;
            } else {
                let _: () = conn.set(key, serialized).await?;
            }
        }
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        if let Some(ref client) = self.client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let _: () = conn.del(key).await?;
        }
        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.client.is_some()
    }
}

