//! Users REST API endpoints

use axum::{
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

/// User login
pub async fn login(
    Json(_payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // TODO: Implement authentication
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Get current user
pub async fn get_current_user() -> Result<Json<serde_json::Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}
