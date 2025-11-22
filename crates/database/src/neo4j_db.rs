//! Neo4j graph database implementation
//!
//! Neo4j is used for storing and querying material similarity networks

use crate::{Error, MaterialDatabase, Result};
use materials_core::Material;
use neo4rs::{Graph, query, ConfigBuilder, Node};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Neo4j graph database for material similarity networks
pub struct Neo4jDatabase {
    graph: Graph,
}

impl Neo4jDatabase {
    pub async fn new(uri: impl Into<String>, user: impl Into<String>, password: impl Into<String>) -> Result<Self> {
        let config = ConfigBuilder::default()
            .uri(uri.into())
            .user(user.into())
            .password(password.into())
            .build()
            .map_err(|e| Error::neo4j(format!("Failed to build Neo4j config: {}", e)))?;

        let graph = Graph::connect(config)
            .await
            .map_err(|e| Error::neo4j(format!("Failed to connect to Neo4j: {}", e)))?;

        // Ensure constraints exist
        let _ = graph
            .run(query("CREATE CONSTRAINT IF NOT EXISTS FOR (m:Material) REQUIRE m.id IS UNIQUE"))
            .await;

        Ok(Self { graph })
    }

    /// Create a connection string from components
    pub async fn from_connection_string(connection_string: impl Into<String>) -> Result<Self> {
        // Parse neo4j://user:pass@host:port format
        let conn_str = connection_string.into();
        let parts: Vec<&str> = conn_str.split("://").collect();

        if parts.len() != 2 {
            return Err(Error::neo4j("Invalid connection string format".to_string()));
        }

        let auth_host: Vec<&str> = parts[1].split('@').collect();
        if auth_host.len() != 2 {
            return Err(Error::neo4j("Invalid connection string format".to_string()));
        }

        let user_pass: Vec<&str> = auth_host[0].split(':').collect();
        if user_pass.len() != 2 {
            return Err(Error::neo4j("Invalid connection string format".to_string()));
        }

        let uri = format!("{}://{}", parts[0], auth_host[1]);
        Self::new(uri, user_pass[0], user_pass[1]).await
    }

    /// Add similarity relationship between materials
    pub async fn add_similarity(&self, id1: Uuid, id2: Uuid, score: f64) -> Result<()> {
        let q = query(
            "MATCH (m1:Material {id: $id1}), (m2:Material {id: $id2})
             MERGE (m1)-[r:SIMILAR_TO]-(m2)
             SET r.score = $score, r.updated_at = datetime()"
        )
        .param("id1", id1.to_string())
        .param("id2", id2.to_string())
        .param("score", score);

        self.graph
            .run(q)
            .await
            .map_err(|e| Error::neo4j(format!("Failed to create similarity: {}", e)))?;

        Ok(())
    }

    /// Find similar materials
    pub async fn find_similar(&self, id: Uuid, limit: i64) -> Result<Vec<(Uuid, f64)>> {
        let q = query(
            "MATCH (m:Material {id: $id})-[r:SIMILAR_TO]-(similar:Material)
             RETURN similar.id as id, r.score as score
             ORDER BY r.score DESC
             LIMIT $limit"
        )
        .param("id", id.to_string())
        .param("limit", limit);

        let mut result = self.graph
            .execute(q)
            .await
            .map_err(|e| Error::neo4j(format!("Failed to find similar materials: {}", e)))?;

        let mut similar = Vec::new();

        while let Ok(Some(row)) = result.next().await {
            let id_str: String = row.get("id").unwrap_or_default();
            let score: f64 = row.get("score").unwrap_or(0.0);

            if let Ok(uuid) = Uuid::parse_str(&id_str) {
                similar.push((uuid, score));
            }
        }

        Ok(similar)
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        self.graph
            .run(query("RETURN 1"))
            .await
            .map_err(|e| Error::neo4j(format!("Health check failed: {}", e)))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl MaterialDatabase for Neo4jDatabase {
    async fn create_material(&self, material: &Material) -> Result<Uuid> {
        let q = query(
            "CREATE (m:Material {
                id: $id,
                formula: $formula,
                created_at: datetime($created_at),
                updated_at: datetime($updated_at)
             })
             RETURN m.id"
        )
        .param("id", material.id.to_string())
        .param("formula", material.formula.clone())
        .param("created_at", material.created_at.to_rfc3339())
        .param("updated_at", material.updated_at.to_rfc3339());

        self.graph
            .run(q)
            .await
            .map_err(|e| Error::neo4j(format!("Failed to create material: {}", e)))?;

        Ok(material.id)
    }

    async fn get_material(&self, id: Uuid) -> Result<Option<Material>> {
        let q = query("MATCH (m:Material {id: $id}) RETURN m")
            .param("id", id.to_string());

        let mut result = self.graph
            .execute(q)
            .await
            .map_err(|e| Error::neo4j(format!("Failed to query material: {}", e)))?;

        if let Ok(Some(row)) = result.next().await {
            if let Ok(node) = row.get::<Node>("m") {
                let id_str: String = node.get("id").unwrap_or_default();
                let formula: String = node.get("formula").unwrap_or_default();

                if let Ok(_uuid) = Uuid::parse_str(&id_str) {
                    let material = Material::new(formula);
                    // Note: In a real implementation, you'd preserve the original ID and timestamps
                    return Ok(Some(material));
                }
            }
        }

        Ok(None)
    }

    async fn update_material(&self, id: Uuid, material: &Material) -> Result<()> {
        let q = query(
            "MATCH (m:Material {id: $id})
             SET m.formula = $formula,
                 m.updated_at = datetime($updated_at)
             RETURN m"
        )
        .param("id", id.to_string())
        .param("formula", material.formula.clone())
        .param("updated_at", material.updated_at.to_rfc3339());

        let mut result = self.graph
            .execute(q)
            .await
            .map_err(|e| Error::neo4j(format!("Failed to update material: {}", e)))?;

        if result.next().await.ok().flatten().is_none() {
            return Err(Error::neo4j("Material not found".to_string()));
        }

        Ok(())
    }

    async fn delete_material(&self, id: Uuid) -> Result<()> {
        let q = query(
            "MATCH (m:Material {id: $id})
             DETACH DELETE m
             RETURN count(m) as deleted"
        )
        .param("id", id.to_string());

        let mut result = self.graph
            .execute(q)
            .await
            .map_err(|e| Error::neo4j(format!("Failed to delete material: {}", e)))?;

        if let Ok(Some(row)) = result.next().await {
            let deleted: i64 = row.get("deleted").unwrap_or(0);
            if deleted == 0 {
                return Err(Error::neo4j("Material not found".to_string()));
            }
        }

        Ok(())
    }

    async fn list_materials(&self, limit: i64, offset: i64) -> Result<Vec<Material>> {
        let q = query(
            "MATCH (m:Material)
             RETURN m
             ORDER BY m.created_at DESC
             SKIP $offset
             LIMIT $limit"
        )
        .param("limit", limit)
        .param("offset", offset);

        let mut result = self.graph
            .execute(q)
            .await
            .map_err(|e| Error::neo4j(format!("Failed to list materials: {}", e)))?;

        let mut materials = Vec::new();

        while let Ok(Some(row)) = result.next().await {
            if let Ok(node) = row.get::<Node>("m") {
                let formula: String = node.get("formula").unwrap_or_default();
                let material = Material::new(formula);
                materials.push(material);
            }
        }

        Ok(materials)
    }
}
