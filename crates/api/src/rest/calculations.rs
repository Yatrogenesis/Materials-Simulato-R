//! Calculations REST API endpoints

use axum::{
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCalculationRequest {
    pub material_id: Uuid,
    pub method: String,
}

#[derive(Debug, Serialize)]
pub struct CalculationResponse {
    pub id: Uuid,
    pub material_id: Uuid,
    pub status: String,
}

/// Create a new calculation
pub async fn create_calculation(
    Json(_payload): Json<CreateCalculationRequest>,
) -> Result<Json<CalculationResponse>, StatusCode> {
    // TODO: Implement calculation creation
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Get calculation status
pub async fn get_calculation_status() -> Result<Json<CalculationResponse>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}
