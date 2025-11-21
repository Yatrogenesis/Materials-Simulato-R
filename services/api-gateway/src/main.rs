//! API Gateway Service
//!
//! Main entry point for the Materials-Simulato-R platform

use materials_api::create_router;
use materials_monitoring;
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize monitoring
    materials_monitoring::init()?;

    info!("Starting Materials-Simulato-R API Gateway v{}", env!("CARGO_PKG_VERSION"));

    // Create router
    let app = create_router();

    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
