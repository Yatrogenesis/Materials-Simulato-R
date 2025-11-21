//! Neo4j graph database implementation

#[allow(dead_code)]
pub struct Neo4jDatabase {
    // TODO: Add neo4rs client
    connection_string: String,
}

impl Neo4jDatabase {
    pub async fn new(connection_string: impl Into<String>) -> crate::Result<Self> {
        Ok(Self {
            connection_string: connection_string.into(),
        })
    }
}
