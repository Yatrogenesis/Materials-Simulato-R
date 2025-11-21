//! Request handlers

use axum::{Json, response::IntoResponse};
use serde_json::json;

pub async fn not_found() -> impl IntoResponse {
    Json(json!({
        "error": "Not Found"
    }))
}
