//! Materials-Simulato-R Computation Engine
//!
//! Multi-fidelity computation methods:
//! - ML methods (Candle, PyTorch)
//! - Molecular Dynamics
//! - DFT bridges

#![allow(dead_code, unused_imports)]

pub mod ml_engine;
pub mod md_engine;
pub mod dft_bridge;
pub mod properties;
pub mod error;

pub use error::{Error, Result};

use materials_core::Material;
use async_trait::async_trait;

/// Computation method trait
#[async_trait]
pub trait ComputationMethod: Send + Sync {
    /// Calculate energy for a material
    async fn calculate_energy(&self, material: &Material) -> Result<f64>;

    /// Calculate forces on atoms
    async fn calculate_forces(&self, material: &Material) -> Result<Vec<[f64; 3]>>;

    /// Estimate computation cost (in seconds)
    fn cost_estimate(&self, material: &Material) -> f64;

    /// Get method name
    fn name(&self) -> &str;
}

/// Version of the compute layer
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
