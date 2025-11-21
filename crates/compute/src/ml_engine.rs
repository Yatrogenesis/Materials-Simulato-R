//! Machine Learning engine (Candle-based)

use crate::{ComputationMethod, Error, Result};
use materials_core::Material;
use async_trait::async_trait;

pub struct MLEngine {
    model_name: String,
}

impl MLEngine {
    pub fn new(model_name: impl Into<String>) -> Self {
        Self {
            model_name: model_name.into(),
        }
    }
}

#[async_trait]
impl ComputationMethod for MLEngine {
    async fn calculate_energy(&self, _material: &Material) -> Result<f64> {
        // TODO: Implement with Candle
        Err(Error::Other("Not yet implemented".to_string()))
    }

    async fn calculate_forces(&self, _material: &Material) -> Result<Vec<[f64; 3]>> {
        // TODO: Implement with Candle
        Err(Error::Other("Not yet implemented".to_string()))
    }

    fn cost_estimate(&self, material: &Material) -> f64 {
        // Estimate: ~0.01s per atom
        material.num_atoms() as f64 * 0.01
    }

    fn name(&self) -> &str {
        &self.model_name
    }
}
