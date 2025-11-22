//! Materials-Simulato-R CLI Interface

#![allow(dead_code, unused_imports)]

pub mod commands;
pub mod output;
pub mod error;

pub use error::{Error, Result};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "materials-cli")]
#[command(about = "Materials-Simulato-R CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
    /// Query materials from database
    Query {
        #[arg(short, long)]
        formula: Option<String>,
    },

    /// Run a simulation
    Simulate {
        #[arg(short, long)]
        material_id: String,
    },

    /// Ingest data from worldwide scientific databases
    Ingest {
        /// Limit the number of records to ingest (for testing)
        #[arg(short, long)]
        limit: Option<u64>,

        /// Data source to ingest from (all, materials-project, pubchem, icsd)
        #[arg(short, long, default_value = "all")]
        source: String,
    },

    /// Get version information
    Version,
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
