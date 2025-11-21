//! API Gateway Service
//!
//! Main entry point for the Materials-Simulato-R platform

use materials_api::{create_router, AppState};
use materials_database::postgres::PostgresDatabase;
use materials_monitoring;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize monitoring
    materials_monitoring::init()?;

    info!("Starting Materials-Simulato-R API Gateway v{}", env!("CARGO_PKG_VERSION"));

    // Get database connection string from environment
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/materials".to_string());

    // Initialize database
    info!("Connecting to database...");
    let db = PostgresDatabase::new(&db_url).await?;
    let state = AppState::new(Arc::new(db));

    // Create router with state
    let app = create_router(state);

    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
