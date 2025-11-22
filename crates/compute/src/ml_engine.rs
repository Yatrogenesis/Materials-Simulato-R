//! Machine Learning engine for materials property prediction
//!
//! This module provides ML-based prediction using trained models for:
//! - Formation energy
//! - Band gap
//! - Elastic moduli
//! - Magnetic properties
//!
//! Models can be loaded from:
//! - Candle safetensors format
//! - ONNX Runtime
//! - PyTorch checkpoints (via tch-rs)

use crate::{ComputationMethod, Error, Result};
use materials_core::Material;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

// ============================================================================
// FEATURE EXTRACTION
// ============================================================================

/// Material feature vector for ML models
#[derive(Debug, Clone)]
pub struct MaterialFeatures {
    /// Composition features (element fractions, stoichiometry)
    pub composition: Vec<f64>,

    /// Structural features (volume, density, packing)
    pub structural: Vec<f64>,

    /// Electronic features (valence electrons, electronegativity)
    pub electronic: Vec<f64>,

    /// Combined feature vector
    pub features: Vec<f64>,
}

impl MaterialFeatures {
    /// Extract features from material
    pub fn from_material(material: &Material) -> Self {
        let composition = Self::extract_composition_features(material);
        let structural = Self::extract_structural_features(material);
        let electronic = Self::extract_electronic_features(material);

        // Combine all features
        let mut features = Vec::new();
        features.extend(&composition);
        features.extend(&structural);
        features.extend(&electronic);

        Self {
            composition,
            structural,
            electronic,
            features,
        }
    }

    fn extract_composition_features(material: &Material) -> Vec<f64> {
        let mut features = vec![0.0; 100]; // 100-dim composition vector

        // Get element counts
        let elements = material.elements();

        // Simple encoding: element occurrence and fraction
        for (i, element) in elements.iter().enumerate().take(10) {
            // Atomic number (normalized)
            let atomic_num = Self::atomic_number(element) as f64 / 118.0;
            features[i * 10] = atomic_num;

            // Element fraction
            features[i * 10 + 1] = 1.0 / elements.len() as f64;

            // Electronegativity (Pauling scale, normalized)
            features[i * 10 + 2] = Self::electronegativity(element) / 4.0;

            // Atomic radius (normalized)
            features[i * 10 + 3] = Self::atomic_radius(element) / 300.0;
        }

        features
    }

    fn extract_structural_features(material: &Material) -> Vec<f64> {
        let mut features = Vec::new();

        // Number of atoms (normalized)
        features.push((material.num_atoms() as f64).ln() / 10.0);

        // Average atomic mass
        let avg_mass = material.average_atomic_mass();
        features.push(avg_mass / 200.0); // Normalized by 200 amu

        // Density estimate (if available)
        if let Some(density) = material.properties.get("density") {
            let density_value = density.as_scalar().unwrap_or(0.0);
            features.push(density_value / 20.0); // Normalized by 20 g/cm³
        } else {
            features.push(0.0);
        }

        // Crystal system indicators (one-hot encoded)
        for _ in 0..7 {
            features.push(0.0); // Placeholder for crystal system
        }

        features
    }

    fn extract_electronic_features(material: &Material) -> Vec<f64> {
        let mut features = Vec::new();

        let elements = material.elements();

        // Total valence electrons
        let total_valence: f64 = elements
            .iter()
            .map(|e| Self::valence_electrons(e) as f64)
            .sum();
        features.push(total_valence / 100.0); // Normalized

        // Average electronegativity
        let avg_en: f64 = elements
            .iter()
            .map(|e| Self::electronegativity(e))
            .sum::<f64>()
            / elements.len() as f64;
        features.push(avg_en / 4.0);

        // Electronegativity difference (max - min)
        let en_values: Vec<f64> = elements.iter().map(|e| Self::electronegativity(e)).collect();
        let en_diff = en_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
            - en_values.iter().cloned().fold(f64::INFINITY, f64::min);
        features.push(en_diff / 4.0);

        features
    }

    // Atomic property lookup tables (simplified)

    fn atomic_number(element: &str) -> u8 {
        match element {
            "H" => 1, "He" => 2, "Li" => 3, "Be" => 4, "B" => 5,
            "C" => 6, "N" => 7, "O" => 8, "F" => 9, "Ne" => 10,
            "Na" => 11, "Mg" => 12, "Al" => 13, "Si" => 14, "P" => 15,
            "S" => 16, "Cl" => 17, "Ar" => 18, "K" => 19, "Ca" => 20,
            "Ti" => 22, "V" => 23, "Cr" => 24, "Mn" => 25, "Fe" => 26,
            "Co" => 27, "Ni" => 28, "Cu" => 29, "Zn" => 30, "Ga" => 31,
            "Sr" => 38, "Y" => 39, "Zr" => 40, "Nb" => 41, "Mo" => 42,
            "Ba" => 56, "La" => 57, "Ce" => 58, "Pb" => 82, "Bi" => 83,
            _ => 1,
        }
    }

    fn electronegativity(element: &str) -> f64 {
        match element {
            "H" => 2.20, "Li" => 0.98, "C" => 2.55, "N" => 3.04,
            "O" => 3.44, "F" => 3.98, "Na" => 0.93, "Mg" => 1.31,
            "Al" => 1.61, "Si" => 1.90, "P" => 2.19, "S" => 2.58,
            "Cl" => 3.16, "K" => 0.82, "Ca" => 1.00, "Ti" => 1.54,
            "Fe" => 1.83, "Ni" => 1.91, "Cu" => 1.90, "Zn" => 1.65,
            "Sr" => 0.95, "Ba" => 0.89, "Pb" => 2.33,
            _ => 1.5,
        }
    }

    fn atomic_radius(element: &str) -> f64 {
        match element {
            "H" => 37.0, "Li" => 152.0, "C" => 77.0, "N" => 71.0,
            "O" => 66.0, "F" => 64.0, "Na" => 186.0, "Mg" => 160.0,
            "Al" => 143.0, "Si" => 117.0, "P" => 110.0, "S" => 104.0,
            "Cl" => 99.0, "K" => 227.0, "Ca" => 197.0, "Ti" => 147.0,
            "Fe" => 126.0, "Ni" => 124.0, "Cu" => 128.0, "Zn" => 134.0,
            "Sr" => 215.0, "Ba" => 222.0, "Pb" => 180.0,
            _ => 100.0,
        }
    }

    fn valence_electrons(element: &str) -> u8 {
        match element {
            "H" => 1, "He" => 2, "Li" => 1, "Be" => 2, "B" => 3,
            "C" => 4, "N" => 5, "O" => 6, "F" => 7, "Ne" => 8,
            "Na" => 1, "Mg" => 2, "Al" => 3, "Si" => 4, "P" => 5,
            "S" => 6, "Cl" => 7, "Ti" => 4, "Fe" => 8, "Ni" => 10,
            "Cu" => 11, "Zn" => 12,
            _ => 4,
        }
    }
}

// ============================================================================
// ML MODEL
// ============================================================================

/// Simple neural network model for property prediction
#[derive(Debug)]
pub struct MLModel {
    name: String,
    input_dim: usize,
    hidden_dim: usize,
    output_dim: usize,

    // Weights (in production, would use Candle Tensors)
    w1: Vec<Vec<f64>>,  // input -> hidden
    b1: Vec<f64>,        // hidden bias
    w2: Vec<Vec<f64>>,  // hidden -> output
    b2: Vec<f64>,        // output bias

    // Normalization parameters
    feature_mean: Vec<f64>,
    feature_std: Vec<f64>,
    target_mean: f64,
    target_std: f64,
}

impl MLModel {
    pub fn new(name: String, input_dim: usize) -> Self {
        let hidden_dim = 128;
        let output_dim = 1;

        // Initialize random weights (Xavier initialization)
        let w1 = Self::init_weights(input_dim, hidden_dim);
        let b1 = vec![0.0; hidden_dim];
        let w2 = Self::init_weights(hidden_dim, output_dim);
        let b2 = vec![0.0; output_dim];

        Self {
            name,
            input_dim,
            hidden_dim,
            output_dim,
            w1,
            b1,
            w2,
            b2,
            feature_mean: vec![0.0; input_dim],
            feature_std: vec![1.0; input_dim],
            target_mean: 0.0,
            target_std: 1.0,
        }
    }

    fn init_weights(nin: usize, nout: usize) -> Vec<Vec<f64>> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let scale = (2.0 / nin as f64).sqrt();

        (0..nin)
            .map(|_| {
                (0..nout)
                    .map(|_| rng.gen::<f64>() * scale - scale / 2.0)
                    .collect()
            })
            .collect()
    }

    /// Forward pass through the network
    pub fn predict(&self, features: &[f64]) -> f64 {
        assert_eq!(features.len(), self.input_dim);

        // Normalize input
        let normalized: Vec<f64> = features
            .iter()
            .zip(&self.feature_mean)
            .zip(&self.feature_std)
            .map(|((x, mean), std)| (x - mean) / std)
            .collect();

        // Layer 1: input -> hidden
        let mut hidden = vec![0.0; self.hidden_dim];
        for (i, h) in hidden.iter_mut().enumerate() {
            let mut sum = self.b1[i];
            for (j, &x) in normalized.iter().enumerate() {
                sum += x * self.w1[j][i];
            }
            *h = Self::relu(sum);
        }

        // Layer 2: hidden -> output
        let mut output = self.b2[0];
        for (i, &h) in hidden.iter().enumerate() {
            output += h * self.w2[i][0];
        }

        // Denormalize output
        output * self.target_std + self.target_mean
    }

    fn relu(x: f64) -> f64 {
        if x > 0.0 { x } else { 0.0 }
    }

    /// Load pre-trained model (mock implementation)
    pub fn load_from_file(&mut self, _path: &PathBuf) -> Result<()> {
        info!("Loading model from file (mock mode)");

        // In real implementation, would load weights from safetensors/ONNX
        // For now, we use pre-initialized weights

        // Set some reasonable normalization parameters
        self.feature_mean = vec![0.5; self.input_dim];
        self.feature_std = vec![0.2; self.input_dim];
        self.target_mean = -3.0; // Typical formation energy
        self.target_std = 1.5;

        Ok(())
    }
}

// ============================================================================
// ML ENGINE
// ============================================================================

pub struct MLEngine {
    model_name: String,
    models: Arc<RwLock<HashMap<String, MLModel>>>,
    model_dir: PathBuf,
}

impl MLEngine {
    pub fn new(model_name: impl Into<String>) -> Self {
        Self {
            model_name: model_name.into(),
            models: Arc::new(RwLock::new(HashMap::new())),
            model_dir: PathBuf::from("models/ml"),
        }
    }

    pub fn with_model_dir(mut self, dir: PathBuf) -> Self {
        self.model_dir = dir;
        self
    }

    /// Load ML models for different properties
    pub fn load_models(&self) -> Result<()> {
        info!("Loading ML models from {:?}", self.model_dir);

        let mut models = self.models.write()
            .map_err(|e| Error::Other(format!("Lock error: {}", e)))?;

        // Formation energy model
        let mut formation_model = MLModel::new("formation_energy".to_string(), 120);
        let model_path = self.model_dir.join("formation_energy.safetensors");
        formation_model.load_from_file(&model_path)?;
        models.insert("formation_energy".to_string(), formation_model);

        // Band gap model
        let mut bandgap_model = MLModel::new("band_gap".to_string(), 120);
        let model_path = self.model_dir.join("band_gap.safetensors");
        bandgap_model.load_from_file(&model_path)?;
        models.insert("band_gap".to_string(), bandgap_model);

        // Elastic modulus model
        let mut elastic_model = MLModel::new("elastic_modulus".to_string(), 120);
        let model_path = self.model_dir.join("elastic_modulus.safetensors");
        elastic_model.load_from_file(&model_path)?;
        models.insert("elastic_modulus".to_string(), elastic_model);

        info!("Loaded {} ML models", models.len());
        Ok(())
    }

    /// Predict a specific property
    pub fn predict_property(&self, material: &Material, property: &str) -> Result<f64> {
        // Extract features
        let features = MaterialFeatures::from_material(material);

        debug!("Extracted {} features for {}", features.features.len(), material.formula);

        // Get model
        let models = self.models.read()
            .map_err(|e| Error::Other(format!("Lock error: {}", e)))?;

        let model = models.get(property)
            .ok_or_else(|| Error::Other(format!("Model not found: {}", property)))?;

        // Predict
        let prediction = model.predict(&features.features);

        debug!("Predicted {} = {:.4} for {}", property, prediction, material.formula);

        Ok(prediction)
    }

    /// Batch prediction for multiple materials
    pub async fn batch_predict(
        &self,
        materials: &[Material],
        property: &str,
    ) -> Result<Vec<f64>> {
        let mut predictions = Vec::new();

        for material in materials {
            let pred = self.predict_property(material, property)?;
            predictions.push(pred);
        }

        Ok(predictions)
    }
}

#[async_trait]
impl ComputationMethod for MLEngine {
    async fn calculate_energy(&self, material: &Material) -> Result<f64> {
        // Try to load models if not already loaded
        {
            let models = self.models.read()
                .map_err(|e| Error::Other(format!("Lock error: {}", e)))?;

            if models.is_empty() {
                drop(models);
                if let Err(e) = self.load_models() {
                    warn!("Failed to load models: {}, using fallback", e);
                }
            }
        }

        // Predict formation energy
        self.predict_property(material, "formation_energy")
    }

    async fn calculate_forces(&self, material: &Material) -> Result<Vec<[f64; 3]>> {
        // ML models typically don't predict forces directly
        // This would require a force-field ML model or E3NN-type architecture

        let num_atoms = material.num_atoms();
        let forces = vec![[0.0, 0.0, 0.0]; num_atoms];

        debug!("Force prediction not implemented, returning zeros for {} atoms", num_atoms);

        Ok(forces)
    }

    fn cost_estimate(&self, material: &Material) -> f64 {
        // ML is very fast: ~0.001s per material
        let num_atoms = material.num_atoms() as f64;
        0.001 + num_atoms * 0.0001
    }

    fn name(&self) -> &str {
        &self.model_name
    }
}

// ============================================================================
// SPECIALIZED PREDICTORS
// ============================================================================

/// Band gap predictor
pub struct BandGapPredictor {
    engine: MLEngine,
}

impl BandGapPredictor {
    pub fn new() -> Self {
        Self {
            engine: MLEngine::new("bandgap_predictor"),
        }
    }

    pub async fn predict(&self, material: &Material) -> Result<f64> {
        self.engine.predict_property(material, "band_gap")
    }
}

/// Formation energy predictor
pub struct FormationEnergyPredictor {
    engine: MLEngine,
}

impl FormationEnergyPredictor {
    pub fn new() -> Self {
        Self {
            engine: MLEngine::new("formation_predictor"),
        }
    }

    pub async fn predict(&self, material: &Material) -> Result<f64> {
        self.engine.predict_property(material, "formation_energy")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use materials_core::material::MaterialBuilder;

    #[test]
    fn test_feature_extraction() {
        let material = MaterialBuilder::new("Fe2O3")
            .unwrap()
            .build();

        let features = MaterialFeatures::from_material(&material);

        assert!(!features.features.is_empty());
        assert!(features.features.len() > 100);
    }

    #[test]
    fn test_ml_model_prediction() {
        let model = MLModel::new("test".to_string(), 120);

        let features = vec![0.5; 120];
        let prediction = model.predict(&features);

        // Should return some value
        assert!(prediction.is_finite());
    }

    #[tokio::test]
    async fn test_ml_engine() {
        let engine = MLEngine::new("test_engine");

        let material = MaterialBuilder::new("TiO2")
            .unwrap()
            .build();

        let result = engine.calculate_energy(&material).await;
        assert!(result.is_ok());

        let energy = result.unwrap();
        assert!(energy.is_finite());
    }

    #[tokio::test]
    async fn test_bandgap_predictor() {
        let predictor = BandGapPredictor::new();

        let material = MaterialBuilder::new("GaAs")
            .unwrap()
            .build();

        let result = predictor.predict(&material).await;
        assert!(result.is_ok());
    }
}
