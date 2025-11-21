//! Redis cache implementation

use redis::{AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};

pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    pub async fn new(connection_string: impl Into<String>) -> crate::Result<Self> {
        let client = Client::open(connection_string.into())
            .map_err(|e| crate::Error::redis(format!("Failed to create Redis client: {}", e)))?;

        // Test connection
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::Error::Connection(format!("Failed to connect to Redis: {}", e)))?;

        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .map_err(|e| crate::Error::Connection(format!("Redis ping failed: {}", e)))?;

        Ok(Self { client })
    }

    /// Get a value from cache
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> crate::Result<Option<T>> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::Error::redis(e.to_string()))?;

        let value: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| crate::Error::redis(e.to_string()))?;

        match value {
            Some(json) => {
                let parsed = serde_json::from_str(&json)?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    /// Set a value in cache with optional TTL (in seconds)
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Option<usize>) -> crate::Result<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::Error::redis(e.to_string()))?;

        let json = serde_json::to_string(value)?;

        if let Some(seconds) = ttl {
            let _: () = conn.set_ex(key, json, seconds as u64)
                .await
                .map_err(|e| crate::Error::redis(e.to_string()))?;
        } else {
            let _: () = conn.set(key, json)
                .await
                .map_err(|e| crate::Error::redis(e.to_string()))?;
        }

        tracing::debug!(key = %key, ttl = ?ttl, "Value cached");
        Ok(())
    }

    /// Delete a key from cache
    pub async fn delete(&self, key: &str) -> crate::Result<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::Error::redis(e.to_string()))?;

        let _: () = conn.del(key)
            .await
            .map_err(|e| crate::Error::redis(e.to_string()))?;

        tracing::debug!(key = %key, "Key deleted from cache");
        Ok(())
    }

    /// Check if key exists
    pub async fn exists(&self, key: &str) -> crate::Result<bool> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::Error::redis(e.to_string()))?;

        let exists: bool = conn
            .exists(key)
            .await
            .map_err(|e| crate::Error::redis(e.to_string()))?;

        Ok(exists)
    }

    /// Health check
    pub async fn health_check(&self) -> crate::Result<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::Error::Connection(format!("Health check failed: {}", e)))?;

        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .map_err(|e| crate::Error::Connection(format!("Health check failed: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis connection
    async fn test_redis_cache() {
        let cache = RedisCache::new("redis://localhost:6379")
            .await
            .expect("Failed to connect to Redis");

        cache.set("test_key", &"test_value", Some(60)).await.unwrap();

        let value: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        cache.delete("test_key").await.unwrap();
    }
}
