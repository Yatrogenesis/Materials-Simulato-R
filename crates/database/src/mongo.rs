//! MongoDB database implementation

#[allow(dead_code)]
pub struct MongoDatabase {
    // TODO: Add MongoDB client
    connection_string: String,
}

impl MongoDatabase {
    pub async fn new(connection_string: impl Into<String>) -> crate::Result<Self> {
        Ok(Self {
            connection_string: connection_string.into(),
        })
    }
}
