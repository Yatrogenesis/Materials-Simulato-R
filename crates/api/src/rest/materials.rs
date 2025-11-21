//! Materials REST API endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use materials_core::Material;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// Create a new material
pub async fn create_material(
    Json(payload): Json<CreateMaterialRequest>,
) -> Result<Json<MaterialResponse>, StatusCode> {
    let material = Material::new(payload.formula);

    let response = MaterialResponse {
        id: material.id,
        formula: material.formula.clone(),
        created_at: material.created_at.to_rfc3339(),
    };

    Ok(Json(response))
}

/// Get a material by ID
pub async fn get_material(
    Path(id): Path<Uuid>,
) -> Result<Json<MaterialResponse>, StatusCode> {
    // TODO: Implement database lookup
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// List all materials
pub async fn list_materials() -> Result<Json<Vec<MaterialResponse>>, StatusCode> {
    // TODO: Implement database listing
    Ok(Json(vec![]))
}
