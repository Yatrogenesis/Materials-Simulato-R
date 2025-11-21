//! Database error types

use thiserror::Error;

/// Result type alias for database operations
pub type Result<T> = std::result::Result<T, Error>;

/// Database error types
#[derive(Error, Debug)]
pub enum Error {
    /// PostgreSQL error
    #[error("PostgreSQL error: {0}")]
    Postgres(String),

    /// MongoDB error
    #[error("MongoDB error: {0}")]
    Mongo(String),

    /// Neo4j error
    #[error("Neo4j error: {0}")]
    Neo4j(String),

    /// Redis error
    #[error("Redis error: {0}")]
    Redis(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Query error
    #[error("Query error: {0}")]
    Query(String),

    /// Not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Core error
    #[error("Core error: {0}")]
    Core(#[from] materials_core::Error),

    /// Other error
    #[error("{0}")]
    Other(String),
}

impl Error {
    pub fn postgres(msg: impl Into<String>) -> Self {
        Self::Postgres(msg.into())
    }

    pub fn mongo(msg: impl Into<String>) -> Self {
        Self::Mongo(msg.into())
    }

    pub fn neo4j(msg: impl Into<String>) -> Self {
        Self::Neo4j(msg.into())
    }

    pub fn redis(msg: impl Into<String>) -> Self {
        Self::Redis(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
}
