//! PostgreSQL database implementation

use crate::{Error, MaterialDatabase, Result};
use materials_core::Material;
use uuid::Uuid;

/// PostgreSQL database connection pool
pub struct PostgresDatabase {
    // TODO: Add sqlx pool
    #[allow(dead_code)]
    connection_string: String,
}

impl PostgresDatabase {
    /// Create a new PostgreSQL database connection
    pub async fn new(connection_string: impl Into<String>) -> Result<Self> {
        Ok(Self {
            connection_string: connection_string.into(),
        })
    }
}

#[async_trait::async_trait]
impl MaterialDatabase for PostgresDatabase {
    async fn create_material(&self, _material: &Material) -> Result<Uuid> {
        // TODO: Implement with SQLx
        Err(Error::Other("Not yet implemented".to_string()))
    }

    async fn get_material(&self, _id: Uuid) -> Result<Option<Material>> {
        // TODO: Implement with SQLx
        Err(Error::Other("Not yet implemented".to_string()))
    }

    async fn update_material(&self, _id: Uuid, _material: &Material) -> Result<()> {
        // TODO: Implement with SQLx
        Err(Error::Other("Not yet implemented".to_string()))
    }

    async fn delete_material(&self, _id: Uuid) -> Result<()> {
        // TODO: Implement with SQLx
        Err(Error::Other("Not yet implemented".to_string()))
    }

    async fn list_materials(&self, _limit: i64, _offset: i64) -> Result<Vec<Material>> {
        // TODO: Implement with SQLx
        Err(Error::Other("Not yet implemented".to_string()))
    }
}
