//! Materials REST API endpoints

use axum::{
    extract::{Path, Extension, Query},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use materials_core::Material;
use materials_database::MaterialDatabase;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMaterialRequest {
    pub formula: String,
}

#[derive(Debug, Serialize)]
pub struct MaterialResponse {
    pub id: Uuid,
    pub formula: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

impl From<Material> for MaterialResponse {
    fn from(material: Material) -> Self {
        Self {
            id: material.id,
            formula: material.formula,
            created_at: material.created_at.to_rfc3339(),
        }
    }
}

/// Create a new material
pub async fn create_material(
    Extension(state): Extension<AppState>,
    Json(payload): Json<CreateMaterialRequest>,
) -> Result<Json<MaterialResponse>, StatusCode> {
    let material = Material::new(payload.formula);

    // Save to database
    state.db.create_material(&material)
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(material.into()))
}

/// Get a material by ID
pub async fn get_material(
    Extension(state): Extension<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MaterialResponse>, StatusCode> {
    match state.db.get_material(id).await {
        Ok(Some(material)) => Ok(Json(material.into())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List all materials
pub async fn list_materials(
    Extension(state): Extension<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<MaterialResponse>>, StatusCode> {
    match state.db.list_materials(query.limit, query.offset).await {
        Ok(materials) => {
            let responses: Vec<MaterialResponse> = materials
                .into_iter()
                .map(|m| m.into())
                .collect();
            Ok(Json(responses))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
