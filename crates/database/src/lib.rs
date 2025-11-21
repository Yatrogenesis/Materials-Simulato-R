//! Materials-Simulato-R Database Layer
//!
//! This crate provides database abstraction and implementations for:
//! - PostgreSQL (relational data)
//! - MongoDB (flexible properties)
//! - Neo4j (similarity networks)
//! - Redis (caching, sessions, queues)

#![allow(dead_code, unused_imports)]

pub mod postgres;
pub mod mongo;
pub mod neo4j_db;
pub mod redis_cache;
pub mod smart_cache;
pub mod error;

pub use error::{Error, Result};

use materials_core::Material;
use uuid::Uuid;

/// Database trait for materials storage
#[async_trait::async_trait]
pub trait MaterialDatabase: Send + Sync {
    /// Create a new material
    async fn create_material(&self, material: &Material) -> Result<Uuid>;

    /// Get a material by ID
    async fn get_material(&self, id: Uuid) -> Result<Option<Material>>;

    /// Update a material
    async fn update_material(&self, id: Uuid, material: &Material) -> Result<()>;

    /// Delete a material
    async fn delete_material(&self, id: Uuid) -> Result<()>;

    /// List materials with pagination
    async fn list_materials(&self, limit: i64, offset: i64) -> Result<Vec<Material>>;
}

/// Version of the database layer
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
