//! Materials-Simulato-R CLI Binary

use materials_cli::{Cli, Commands};
use clap::Parser;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute command
    match cli.command {
        Commands::Query { formula } => {
            materials_cli::commands::query::execute(formula).await?;
        }
        Commands::Simulate { material_id } => {
            materials_cli::commands::simulate::execute(&material_id).await?;
        }
        Commands::Ingest { limit, source } => {
            materials_cli::commands::ingest::run(limit, &source).await?;
        }
        Commands::Version => {
            materials_cli::commands::version::execute();
        }
    }

    Ok(())
}
