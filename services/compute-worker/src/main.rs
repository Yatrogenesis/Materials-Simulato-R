//! Compute Worker Service
//!
//! Distributed compute worker for materials simulations

use materials_monitoring;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize monitoring
    materials_monitoring::init()?;

    info!("Starting Materials-Simulato-R Compute Worker v{}", env!("CARGO_PKG_VERSION"));

    // TODO: Connect to Redis queue
    // TODO: Process computation jobs

    // Keep alive
    tokio::signal::ctrl_c().await?;
    info!("Shutting down compute worker");

    Ok(())
}
