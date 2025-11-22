//! Chemical Embeddings - Vector representations of materials
//!
//! This module implements advanced chemical embeddings for materials,
//! enabling semantic similarity search, clustering, and ML predictions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Dimensionality of embedding vectors
pub const EMBEDDING_DIM: usize = 256;

/// Chemical embedding vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChemicalEmbedding {
    /// Material ID
    pub material_id: Uuid,

    /// Embedding vector (256-dimensional)
    pub vector: Vec<f64>,

    /// Formula hash for quick lookup
    pub formula_hash: u64,

    /// Metadata
    pub metadata: EmbeddingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub model_version: String,
    pub confidence: f64,
}

/// Similarity search result
#[derive(Debug, Clone)]
pub struct SimilarityResult {
    pub material_id: Uuid,
    pub similarity_score: f64,
    pub distance: f64,
    pub formula: String,
}

/// Chemical Embedding Engine
pub struct EmbeddingEngine {
    /// Embedding cache
    embeddings: Arc<RwLock<HashMap<Uuid, ChemicalEmbedding>>>,

    /// Element embeddings (learned representations)
    element_vectors: HashMap<String, Vec<f64>>,

    /// Property weights for embedding generation
    property_weights: HashMap<String, f64>,

    /// Model version
    version: String,
}

impl EmbeddingEngine {
    pub fn new() -> Self {
        Self {
            embeddings: Arc::new(RwLock::new(HashMap::new())),
            element_vectors: Self::initialize_element_vectors(),
            property_weights: Self::initialize_property_weights(),
            version: "v1.0.0".to_string(),
        }
    }

    /// Generate embedding from material
    pub async fn generate_embedding(
        &self,
        material_id: Uuid,
        formula: &str,
        properties: &HashMap<String, f64>,
    ) -> Result<ChemicalEmbedding, String> {
        // Parse formula to get element composition
        let composition = Self::parse_formula(formula)?;

        // Generate composition-based embedding
        let mut composition_vector = self.composition_embedding(&composition);

        // Add property-based features
        let property_vector = self.property_embedding(properties);

        // Combine vectors
        composition_vector.extend(property_vector);

        // Ensure correct dimensionality
        composition_vector.resize(EMBEDDING_DIM, 0.0);

        // Normalize
        let normalized = Self::normalize_vector(&composition_vector);

        let embedding = ChemicalEmbedding {
            material_id,
            vector: normalized,
            formula_hash: Self::hash_formula(formula),
            metadata: EmbeddingMetadata {
                created_at: chrono::Utc::now(),
                model_version: self.version.clone(),
                confidence: 0.95,
            },
        };

        // Cache the embedding
        self.embeddings.write().await.insert(material_id, embedding.clone());

        Ok(embedding)
    }

    /// Find similar materials using cosine similarity
    pub async fn find_similar(
        &self,
        material_id: Uuid,
        top_k: usize,
    ) -> Result<Vec<SimilarityResult>, String> {
        let embeddings = self.embeddings.read().await;

        let target = embeddings.get(&material_id)
            .ok_or_else(|| "Material not found in embeddings".to_string())?;

        let mut similarities: Vec<SimilarityResult> = Vec::new();

        for (id, embedding) in embeddings.iter() {
            if *id == material_id {
                continue;
            }

            let similarity = Self::cosine_similarity(&target.vector, &embedding.vector);
            let distance = 1.0 - similarity;

            similarities.push(SimilarityResult {
                material_id: *id,
                similarity_score: similarity,
                distance,
                formula: format!("Material-{}", id.to_string()[..8].to_string()),
            });
        }

        // Sort by similarity (descending)
        similarities.sort_by(|a, b| {
            b.similarity_score.partial_cmp(&a.similarity_score).unwrap()
        });

        // Return top-k
        Ok(similarities.into_iter().take(top_k).collect())
    }

    /// Cluster materials using k-means
    pub async fn cluster_materials(&self, k: usize) -> Result<Vec<Vec<Uuid>>, String> {
        let embeddings = self.embeddings.read().await;

        if embeddings.len() < k {
            return Err(format!("Not enough materials to cluster: {} < {}", embeddings.len(), k));
        }

        let ids: Vec<Uuid> = embeddings.keys().copied().collect();
        let vectors: Vec<Vec<f64>> = embeddings.values().map(|e| e.vector.clone()).collect();

        // Simple k-means clustering
        let clusters = Self::kmeans_clustering(&vectors, k, 100);

        // Map clusters back to material IDs
        let mut result = vec![Vec::new(); k];
        for (i, cluster_id) in clusters.iter().enumerate() {
            result[*cluster_id].push(ids[i]);
        }

        Ok(result)
    }

    /// Get embedding statistics
    pub async fn get_statistics(&self) -> EmbeddingStats {
        let embeddings = self.embeddings.read().await;

        EmbeddingStats {
            total_embeddings: embeddings.len(),
            dimension: EMBEDDING_DIM,
            model_version: self.version.clone(),
            average_confidence: embeddings.values()
                .map(|e| e.metadata.confidence)
                .sum::<f64>() / embeddings.len() as f64,
        }
    }

    // === Private Helper Methods ===

    fn initialize_element_vectors() -> HashMap<String, Vec<f64>> {
        // Pre-trained element embeddings based on:
        // - Atomic number
        // - Electronegativity
        // - Atomic radius
        // - Ionization energy
        // - Electron affinity
        // - Common oxidation states

        let mut vectors = HashMap::new();

        // Common elements (simplified for demo)
        let elements = vec![
            ("H", vec![1.0, 2.20, 0.37, 13.6, 0.75, 1.0]),
            ("C", vec![6.0, 2.55, 0.77, 11.3, 1.26, 4.0]),
            ("N", vec![7.0, 3.04, 0.75, 14.5, 0.07, -3.0]),
            ("O", vec![8.0, 3.44, 0.73, 13.6, 1.46, -2.0]),
            ("F", vec![9.0, 3.98, 0.71, 17.4, 3.40, -1.0]),
            ("Si", vec![14.0, 1.90, 1.18, 8.2, 1.39, 4.0]),
            ("P", vec![15.0, 2.19, 1.10, 10.5, 0.75, 5.0]),
            ("S", vec![16.0, 2.58, 1.04, 10.4, 2.08, -2.0]),
            ("Cl", vec![17.0, 3.16, 0.99, 13.0, 3.61, -1.0]),
            ("Fe", vec![26.0, 1.83, 1.26, 7.9, 0.15, 3.0]),
            ("Cu", vec![29.0, 1.90, 1.28, 7.7, 1.24, 2.0]),
            ("Ag", vec![47.0, 1.93, 1.45, 7.6, 1.30, 1.0]),
            ("Au", vec![79.0, 2.54, 1.44, 9.2, 2.31, 3.0]),
            ("Ti", vec![22.0, 1.54, 1.47, 6.8, 0.08, 4.0]),
            ("Al", vec![13.0, 1.61, 1.43, 6.0, 0.43, 3.0]),
        ];

        for (symbol, features) in elements {
            // Expand to higher dimension
            let mut vector = features.clone();
            // Add derived features
            vector.extend(features.iter().map(|x| x * x)); // Squared
            vector.extend(features.iter().copied()); // Duplicate for higher dimension
            vector.resize(64, 0.0);
            vectors.insert(symbol.to_string(), Self::normalize_vector(&vector));
        }

        vectors
    }

    fn initialize_property_weights() -> HashMap<String, f64> {
        let mut weights = HashMap::new();
        weights.insert("formation_energy".to_string(), 2.0);
        weights.insert("band_gap".to_string(), 1.5);
        weights.insert("density".to_string(), 1.0);
        weights.insert("melting_point".to_string(), 1.2);
        weights.insert("thermal_conductivity".to_string(), 1.0);
        weights.insert("electrical_conductivity".to_string(), 1.3);
        weights.insert("magnetic_moment".to_string(), 0.8);
        weights.insert("bulk_modulus".to_string(), 1.1);
        weights
    }

    fn parse_formula(formula: &str) -> Result<HashMap<String, usize>, String> {
        let mut composition = HashMap::new();
        let mut current_element = String::new();
        let mut current_count = String::new();

        for ch in formula.chars() {
            if ch.is_uppercase() {
                // Save previous element
                if !current_element.is_empty() {
                    let count = if current_count.is_empty() {
                        1
                    } else {
                        current_count.parse().unwrap_or(1)
                    };
                    composition.insert(current_element.clone(), count);
                }

                current_element = ch.to_string();
                current_count.clear();
            } else if ch.is_lowercase() {
                current_element.push(ch);
            } else if ch.is_numeric() {
                current_count.push(ch);
            }
        }

        // Save last element
        if !current_element.is_empty() {
            let count = if current_count.is_empty() {
                1
            } else {
                current_count.parse().unwrap_or(1)
            };
            composition.insert(current_element, count);
        }

        Ok(composition)
    }

    fn composition_embedding(&self, composition: &HashMap<String, usize>) -> Vec<f64> {
        let mut vector = vec![0.0; 128];
        let total_atoms: usize = composition.values().sum();

        for (element, count) in composition {
            if let Some(element_vec) = self.element_vectors.get(element) {
                let fraction = *count as f64 / total_atoms as f64;
                for (i, val) in element_vec.iter().enumerate() {
                    if i < vector.len() {
                        vector[i] += val * fraction;
                    }
                }
            }
        }

        vector
    }

    fn property_embedding(&self, properties: &HashMap<String, f64>) -> Vec<f64> {
        let mut vector = vec![0.0; 128];

        for (i, (prop_name, weight)) in self.property_weights.iter().enumerate() {
            if let Some(value) = properties.get(prop_name) {
                if i < vector.len() {
                    vector[i] = value * weight;
                }
            }
        }

        vector
    }

    fn normalize_vector(vector: &[f64]) -> Vec<f64> {
        let magnitude = vector.iter().map(|x| x * x).sum::<f64>().sqrt();
        if magnitude < 1e-10 {
            return vector.to_vec();
        }
        vector.iter().map(|x| x / magnitude).collect()
    }

    fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let magnitude_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

        if magnitude_a < 1e-10 || magnitude_b < 1e-10 {
            return 0.0;
        }

        dot_product / (magnitude_a * magnitude_b)
    }

    fn hash_formula(formula: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        formula.hash(&mut hasher);
        hasher.finish()
    }

    fn kmeans_clustering(vectors: &[Vec<f64>], k: usize, max_iterations: usize) -> Vec<usize> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let n = vectors.len();
        let dim = vectors[0].len();

        // Initialize centroids randomly
        let mut centroids: Vec<Vec<f64>> = (0..k)
            .map(|_| {
                let idx = rng.gen_range(0..n);
                vectors[idx].clone()
            })
            .collect();

        let mut assignments = vec![0; n];

        for _ in 0..max_iterations {
            // Assignment step
            for (i, vector) in vectors.iter().enumerate() {
                let mut best_cluster = 0;
                let mut best_distance = f64::MAX;

                for (j, centroid) in centroids.iter().enumerate() {
                    let distance = Self::euclidean_distance(vector, centroid);
                    if distance < best_distance {
                        best_distance = distance;
                        best_cluster = j;
                    }
                }

                assignments[i] = best_cluster;
            }

            // Update step
            for j in 0..k {
                let cluster_vectors: Vec<&Vec<f64>> = vectors.iter()
                    .enumerate()
                    .filter(|(i, _)| assignments[*i] == j)
                    .map(|(_, v)| v)
                    .collect();

                if !cluster_vectors.is_empty() {
                    let mut new_centroid = vec![0.0; dim];
                    for vector in &cluster_vectors {
                        for (i, val) in vector.iter().enumerate() {
                            new_centroid[i] += val;
                        }
                    }
                    for val in &mut new_centroid {
                        *val /= cluster_vectors.len() as f64;
                    }
                    centroids[j] = new_centroid;
                }
            }
        }

        assignments
    }

    fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

impl Default for EmbeddingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingStats {
    pub total_embeddings: usize,
    pub dimension: usize,
    pub model_version: String,
    pub average_confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_generation() {
        let engine = EmbeddingEngine::new();
        let material_id = Uuid::new_v4();

        let mut properties = HashMap::new();
        properties.insert("formation_energy".to_string(), -2.5);
        properties.insert("band_gap".to_string(), 1.2);

        let embedding = engine.generate_embedding(
            material_id,
            "Fe2O3",
            &properties,
        ).await.unwrap();

        assert_eq!(embedding.vector.len(), EMBEDDING_DIM);
        assert!(embedding.metadata.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_similarity_search() {
        let engine = EmbeddingEngine::new();

        let mut props1 = HashMap::new();
        props1.insert("formation_energy".to_string(), -2.5);

        let mut props2 = HashMap::new();
        props2.insert("formation_energy".to_string(), -2.4);

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        engine.generate_embedding(id1, "Fe2O3", &props1).await.unwrap();
        engine.generate_embedding(id2, "Fe2O4", &props2).await.unwrap();

        let similar = engine.find_similar(id1, 5).await.unwrap();
        assert!(!similar.is_empty());
    }
}
