//! MongoDB database implementation
//!
//! MongoDB is used for flexible property storage that doesn't fit the relational schema

use crate::{Error, MaterialDatabase, Result};
use materials_core::Material;
use mongodb::{
    Client, Collection,
    bson::{doc, to_document, from_document, Bson},
    options::{ClientOptions, FindOptions},
};
use uuid::Uuid;
use std::time::Duration;

/// MongoDB database for flexible material properties
pub struct MongoDatabase {
    client: Client,
    collection: Collection<mongodb::bson::Document>,
}

impl MongoDatabase {
    pub async fn new(connection_string: impl Into<String>) -> Result<Self> {
        let mut client_options = ClientOptions::parse(&connection_string.into())
            .await
            .map_err(|e| Error::mongo(format!("Failed to parse MongoDB URI: {}", e)))?;

        client_options.app_name = Some("materials-simulator".to_string());
        client_options.connect_timeout = Some(Duration::from_secs(10));
        client_options.server_selection_timeout = Some(Duration::from_secs(10));

        let client = Client::with_options(client_options)
            .map_err(|e| Error::mongo(format!("Failed to create MongoDB client: {}", e)))?;

        // Ping to verify connection
        client
            .database("admin")
            .run_command(doc! { "ping": 1 }, None)
            .await
            .map_err(|e| Error::mongo(format!("Failed to connect to MongoDB: {}", e)))?;

        let collection = client
            .database("materials")
            .collection("materials");

        Ok(Self { client, collection })
    }

    /// Check if MongoDB is healthy
    pub async fn health_check(&self) -> Result<()> {
        self.client
            .database("admin")
            .run_command(doc! { "ping": 1 }, None)
            .await
            .map_err(|e| Error::mongo(format!("Health check failed: {}", e)))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl MaterialDatabase for MongoDatabase {
    async fn create_material(&self, material: &Material) -> Result<Uuid> {
        let doc = to_document(material)
            .map_err(|e| Error::mongo(format!("Failed to serialize material: {}", e)))?;

        self.collection
            .insert_one(doc, None)
            .await
            .map_err(|e| Error::mongo(format!("Failed to insert material: {}", e)))?;

        Ok(material.id)
    }

    async fn get_material(&self, id: Uuid) -> Result<Option<Material>> {
        let filter = doc! { "id": id.to_string() };

        match self.collection.find_one(filter, None).await {
            Ok(Some(doc)) => {
                let material = from_document(doc)
                    .map_err(|e| Error::mongo(format!("Failed to deserialize material: {}", e)))?;
                Ok(Some(material))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(Error::mongo(format!("Failed to query material: {}", e))),
        }
    }

    async fn update_material(&self, id: Uuid, material: &Material) -> Result<()> {
        let filter = doc! { "id": id.to_string() };
        let update_doc = to_document(material)
            .map_err(|e| Error::mongo(format!("Failed to serialize material: {}", e)))?;

        let update = doc! { "$set": update_doc };

        let result = self.collection
            .update_one(filter, update, None)
            .await
            .map_err(|e| Error::mongo(format!("Failed to update material: {}", e)))?;

        if result.matched_count == 0 {
            return Err(Error::mongo("Material not found".to_string()));
        }

        Ok(())
    }

    async fn delete_material(&self, id: Uuid) -> Result<()> {
        let filter = doc! { "id": id.to_string() };

        let result = self.collection
            .delete_one(filter, None)
            .await
            .map_err(|e| Error::mongo(format!("Failed to delete material: {}", e)))?;

        if result.deleted_count == 0 {
            return Err(Error::mongo("Material not found".to_string()));
        }

        Ok(())
    }

    async fn list_materials(&self, limit: i64, offset: i64) -> Result<Vec<Material>> {
        let options = FindOptions::builder()
            .limit(limit)
            .skip(offset as u64)
            .sort(doc! { "created_at": -1 })
            .build();

        let mut cursor = self.collection
            .find(doc! {}, Some(options))
            .await
            .map_err(|e| Error::mongo(format!("Failed to list materials: {}", e)))?;

        let mut materials = Vec::new();

        use futures::stream::StreamExt;
        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => {
                    match from_document(doc) {
                        Ok(material) => materials.push(material),
                        Err(e) => eprintln!("Warning: Failed to deserialize material: {}", e),
                    }
                }
                Err(e) => eprintln!("Warning: Failed to fetch document: {}", e),
            }
        }

        Ok(materials)
    }
}
