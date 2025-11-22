//! Materials-Simulato-R Core Library
//!
//! This crate provides the core types, traits, and utilities used throughout
//! the Materials-Simulato-R platform.

pub mod error;
pub mod material;
pub mod property;
pub mod config;
pub mod auto_optimizer;
pub mod feature_flags;

// 🧠 Advanced Intelligence Modules
pub mod embeddings;
pub mod ml_predictor;
pub mod knowledge_graph;
pub mod discovery;
pub mod recommendations;

// 🔬 LIRS - LISP In Rust for Science
pub mod lirs;

// 🧬 Graph Neural Networks
pub mod gnn;

// 🗣️ Natural Language Interface
pub mod nli;

pub use error::{Error, Result};
pub use material::Material;
pub use property::Property;
pub use config::Config;

/// Version of the Materials-Simulato-R platform
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Minimum Supported Rust Version
pub const MSRV: &str = "1.75.0";
