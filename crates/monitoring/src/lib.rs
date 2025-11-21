//! Materials-Simulato-R Monitoring & Observability
//!
//! Provides:
//! - Prometheus metrics
//! - Distributed tracing
//! - Health checks

#![allow(dead_code, unused_imports)]

pub mod metrics;
pub mod tracing_setup;
pub mod health;
pub mod error;

pub use error::{Error, Result};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize monitoring
pub fn init() -> Result<()> {
    tracing_subscriber::fmt::init();
    Ok(())
}
