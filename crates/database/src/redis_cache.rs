//! Redis cache implementation

#[allow(dead_code)]
pub struct RedisCache {
    // TODO: Add redis client
    connection_string: String,
}

impl RedisCache {
    pub async fn new(connection_string: impl Into<String>) -> crate::Result<Self> {
        Ok(Self {
            connection_string: connection_string.into(),
        })
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, _key: &str) -> crate::Result<Option<T>> {
        // TODO: Implement
        Ok(None)
    }

    pub async fn set<T: serde::Serialize>(&self, _key: &str, _value: &T, _ttl: Option<usize>) -> crate::Result<()> {
        // TODO: Implement
        Ok(())
    }
}
