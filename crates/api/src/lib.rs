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

pub use error::{Error, Result};

use axum::{Router, routing::get};

/// Create the API router
pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
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
