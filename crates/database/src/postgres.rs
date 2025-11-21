//! PostgreSQL database implementation with SQLx

use crate::{Error, MaterialDatabase, Result};
use materials_core::Material;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::time::Duration;
use uuid::Uuid;

/// PostgreSQL database connection pool
pub struct PostgresDatabase {
    pool: PgPool,
}

impl PostgresDatabase {
    /// Create a new PostgreSQL database connection
    pub async fn new(connection_string: impl Into<String>) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(50)
            .acquire_timeout(Duration::from_secs(30))
            .connect(&connection_string.into())
            .await
            .map_err(|e| Error::postgres(format!("Failed to connect to PostgreSQL: {}", e)))?;

        Ok(Self { pool })
    }

    /// Get the underlying connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> Result<()> {
        // TODO: Add migrations directory and run them
        tracing::info!("Migrations would run here");
        Ok(())
    }

    /// Health check - verify database is accessible
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Connection(format!("Health check failed: {}", e)))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl MaterialDatabase for PostgresDatabase {
    async fn create_material(&self, material: &Material) -> Result<Uuid> {
        let id = material.id;
        let formula = &material.formula;
        let structure_json = serde_json::to_value(&material.structure)?;
        let properties_json = serde_json::to_value(&material.properties)?;
        let metadata_json = serde_json::to_value(&material.metadata)?;

        sqlx::query(
            r#"
            INSERT INTO materials (id, formula, structure, properties, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(id)
        .bind(formula)
        .bind(structure_json)
        .bind(properties_json)
        .bind(metadata_json)
        .bind(material.created_at)
        .bind(material.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::postgres(format!("Failed to insert material: {}", e)))?;

        tracing::info!(material_id = %id, formula = %formula, "Material created");
        Ok(id)
    }

    async fn get_material(&self, id: Uuid) -> Result<Option<Material>> {
        let row = sqlx::query(
            r#"
            SELECT id, formula, structure, properties, metadata, created_at, updated_at
            FROM materials
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::postgres(format!("Failed to fetch material: {}", e)))?;

        match row {
            Some(row) => {
                let material = Material {
                    id: row.try_get("id")?,
                    formula: row.try_get("formula")?,
                    structure: serde_json::from_value(row.try_get("structure")?)?,
                    properties: serde_json::from_value(row.try_get("properties")?)?,
                    metadata: serde_json::from_value(row.try_get("metadata")?)?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                };
                Ok(Some(material))
            }
            None => Ok(None),
        }
    }

    async fn update_material(&self, id: Uuid, material: &Material) -> Result<()> {
        let structure_json = serde_json::to_value(&material.structure)?;
        let properties_json = serde_json::to_value(&material.properties)?;
        let metadata_json = serde_json::to_value(&material.metadata)?;

        let result = sqlx::query(
            r#"
            UPDATE materials
            SET formula = $2, structure = $3, properties = $4, metadata = $5, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&material.formula)
        .bind(structure_json)
        .bind(properties_json)
        .bind(metadata_json)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::postgres(format!("Failed to update material: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(Error::not_found(format!("Material {} not found", id)));
        }

        tracing::info!(material_id = %id, "Material updated");
        Ok(())
    }

    async fn delete_material(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM materials WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::postgres(format!("Failed to delete material: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(Error::not_found(format!("Material {} not found", id)));
        }

        tracing::info!(material_id = %id, "Material deleted");
        Ok(())
    }

    async fn list_materials(&self, limit: i64, offset: i64) -> Result<Vec<Material>> {
        let rows = sqlx::query(
            r#"
            SELECT id, formula, structure, properties, metadata, created_at, updated_at
            FROM materials
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::postgres(format!("Failed to list materials: {}", e)))?;

        let materials = rows
            .iter()
            .map(|row| {
                Ok(Material {
                    id: row.try_get("id")?,
                    formula: row.try_get("formula")?,
                    structure: serde_json::from_value(row.try_get("structure")?)?,
                    properties: serde_json::from_value(row.try_get("properties")?)?,
                    metadata: serde_json::from_value(row.try_get("metadata")?)?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(materials)
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::Postgres(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_postgres_health_check() {
        let db = PostgresDatabase::new("postgresql://postgres:postgres@localhost/materials_test")
            .await
            .expect("Failed to connect");

        db.health_check().await.expect("Health check failed");
    }
}
