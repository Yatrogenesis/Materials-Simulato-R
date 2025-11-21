//! Materials-Simulato-R API Layer
//!
//! Provides:
//! - REST API (Axum)
//! - GraphQL API (future)
//! - gRPC services (future)
//! - WebSocket support (future)

#![allow(dead_code, unused_imports)]

pub mod rest;
pub mod handlers;
pub mod middleware;
pub mod error;
pub mod rate_limiter;

pub use error::{Error, Result};

use axum::{Router, routing::{get, post}, extract::Extension};
use std::sync::Arc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn materials_database::MaterialDatabase>,
}

impl AppState {
    pub fn new(db: Arc<dyn materials_database::MaterialDatabase>) -> Self {
        Self { db }
    }
}

/// Create the API router with state
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/materials", post(rest::materials::create_material))
        .route("/api/v1/materials/:id", get(rest::materials::get_material))
        .route("/api/v1/materials", get(rest::materials::list_materials))
        .layer(Extension(state))
}

async fn health_check() -> &'static str {
    "OK"
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
