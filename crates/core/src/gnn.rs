//! Graph Neural Networks (GNNs) for Materials
//!
//! Advanced graph-based deep learning for atomic structure representations.
//! Implements message passing neural networks for property prediction.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use nalgebra as na;

/// Node features for atoms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomNode {
    pub atom_id: usize,
    pub element: String,
    pub atomic_number: u8,
    pub features: Vec<f64>,  // Atomic features
    pub coordinates: [f64; 3],  // 3D position
}

/// Edge features for bonds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondEdge {
    pub from_atom: usize,
    pub to_atom: usize,
    pub bond_type: BondType,
    pub distance: f64,
    pub features: Vec<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BondType {
    Covalent,
    Ionic,
    Metallic,
    VanDerWaals,
    Hydrogen,
}

/// Molecular graph representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MolecularGraph {
    pub material_id: Uuid,
    pub formula: String,
    pub nodes: Vec<AtomNode>,
    pub edges: Vec<BondEdge>,
    pub adjacency: HashMap<usize, Vec<usize>>,
    pub global_features: Vec<f64>,  // Graph-level features
}

/// GNN Layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GNNConfig {
    pub node_feature_dim: usize,
    pub edge_feature_dim: usize,
    pub hidden_dim: usize,
    pub output_dim: usize,
    pub num_layers: usize,
    pub dropout: f64,
    pub aggregation: AggregationType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Mean,
    Max,
    Attention,
}

impl Default for GNNConfig {
    fn default() -> Self {
        Self {
            node_feature_dim: 64,
            edge_feature_dim: 32,
            hidden_dim: 128,
            output_dim: 64,
            num_layers: 3,
            dropout: 0.1,
            aggregation: AggregationType::Mean,
        }
    }
}

/// Message Passing Neural Network Layer
pub struct MPNNLayer {
    config: GNNConfig,
    // Weight matrices (simplified - in production would use proper tensor library)
    node_weights: na::DMatrix<f64>,
    edge_weights: na::DMatrix<f64>,
    message_weights: na::DMatrix<f64>,
    update_weights: na::DMatrix<f64>,
}

impl MPNNLayer {
    pub fn new(config: GNNConfig) -> Self {
        let node_dim = config.node_feature_dim;
        let hidden_dim = config.hidden_dim;
        let edge_dim = config.edge_feature_dim;

        Self {
            config,
            node_weights: na::DMatrix::from_fn(hidden_dim, node_dim, |_, _| rand::random::<f64>() * 0.1),
            edge_weights: na::DMatrix::from_fn(hidden_dim, edge_dim, |_, _| rand::random::<f64>() * 0.1),
            message_weights: na::DMatrix::from_fn(hidden_dim, hidden_dim * 2, |_, _| rand::random::<f64>() * 0.1),
            update_weights: na::DMatrix::from_fn(hidden_dim, hidden_dim * 2, |_, _| rand::random::<f64>() * 0.1),
        }
    }

    /// Message passing: compute messages from neighbors
    pub fn message_passing(
        &self,
        node_features: &HashMap<usize, na::DVector<f64>>,
        edge_features: &HashMap<(usize, usize), na::DVector<f64>>,
        adjacency: &HashMap<usize, Vec<usize>>,
    ) -> HashMap<usize, na::DVector<f64>> {
        let mut messages = HashMap::new();

        for (node_id, neighbors) in adjacency {
            let mut aggregated_message = na::DVector::zeros(self.config.hidden_dim);

            for &neighbor_id in neighbors {
                if let Some(neighbor_features) = node_features.get(&neighbor_id) {
                    // Get edge features if available
                    let edge_key = (*node_id, neighbor_id);
                    let edge_feat = edge_features.get(&edge_key)
                        .cloned()
                        .unwrap_or_else(|| na::DVector::zeros(self.config.edge_feature_dim));

                    // Compute message: f(h_neighbor, e_ij)
                    let message = self.compute_message(neighbor_features, &edge_feat);

                    // Aggregate
                    aggregated_message += message;
                }
            }

            // Apply aggregation function
            let final_message = match self.config.aggregation {
                AggregationType::Sum => aggregated_message,
                AggregationType::Mean => {
                    if neighbors.is_empty() {
                        aggregated_message
                    } else {
                        aggregated_message / neighbors.len() as f64
                    }
                }
                AggregationType::Max => aggregated_message,  // Simplified
                AggregationType::Attention => aggregated_message,  // Simplified
            };

            messages.insert(*node_id, final_message);
        }

        messages
    }

    /// Compute message from neighbor
    fn compute_message(
        &self,
        neighbor_features: &na::DVector<f64>,
        edge_features: &na::DVector<f64>,
    ) -> na::DVector<f64> {
        // Simple linear transformation (in practice would use MLP)
        let node_transformed = &self.node_weights * neighbor_features;
        let edge_transformed = &self.edge_weights * edge_features;

        // Combine and apply activation (ReLU)
        let combined = node_transformed + edge_transformed;
        combined.map(|x| x.max(0.0))  // ReLU
    }

    /// Update node features with messages
    pub fn update_nodes(
        &self,
        node_features: &HashMap<usize, na::DVector<f64>>,
        messages: &HashMap<usize, na::DVector<f64>>,
    ) -> HashMap<usize, na::DVector<f64>> {
        let mut updated = HashMap::new();

        for (node_id, features) in node_features {
            let message = messages.get(node_id)
                .cloned()
                .unwrap_or_else(|| na::DVector::zeros(self.config.hidden_dim));

            // Concatenate node features and message
            let mut combined_vec = features.as_slice().to_vec();
            combined_vec.extend(message.as_slice());
            let combined = na::DVector::from_vec(combined_vec);

            // Update: h_new = f(h_old, message)
            let resized = if combined.len() > self.update_weights.ncols() {
                na::DVector::from_iterator(
                    self.update_weights.ncols(),
                    combined.iter().take(self.update_weights.ncols()).copied()
                )
            } else {
                // Pad with zeros if too small
                let mut vec = combined.as_slice().to_vec();
                vec.resize(self.update_weights.ncols(), 0.0);
                na::DVector::from_vec(vec)
            };

            let new_features = &self.update_weights * &resized;
            let activated = new_features.map(|x| x.max(0.0));  // ReLU

            updated.insert(*node_id, activated);
        }

        updated
    }
}

/// Graph Neural Network Model
pub struct GNNModel {
    config: GNNConfig,
    layers: Vec<MPNNLayer>,
    readout_weights: na::DMatrix<f64>,
}

impl GNNModel {
    pub fn new(config: GNNConfig) -> Self {
        let mut layers = Vec::new();
        for _ in 0..config.num_layers {
            layers.push(MPNNLayer::new(config.clone()));
        }

        let readout_weights = na::DMatrix::from_fn(
            config.output_dim,
            config.hidden_dim,
            |_, _| rand::random::<f64>() * 0.1
        );

        Self {
            config,
            layers,
            readout_weights,
        }
    }

    /// Forward pass through GNN
    pub fn forward(&self, graph: &MolecularGraph) -> na::DVector<f64> {
        // Initialize node features
        let mut node_features = HashMap::new();
        for node in &graph.nodes {
            let features = na::DVector::from_vec(node.features.clone());
            node_features.insert(node.atom_id, features);
        }

        // Initialize edge features
        let mut edge_features = HashMap::new();
        for edge in &graph.edges {
            let features = na::DVector::from_vec(edge.features.clone());
            edge_features.insert((edge.from_atom, edge.to_atom), features);
        }

        // Message passing through layers
        for layer in &self.layers {
            let messages = layer.message_passing(&node_features, &edge_features, &graph.adjacency);
            node_features = layer.update_nodes(&node_features, &messages);
        }

        // Readout: aggregate node features to graph-level representation
        self.readout(&node_features)
    }

    /// Readout layer: aggregate node features
    fn readout(&self, node_features: &HashMap<usize, na::DVector<f64>>) -> na::DVector<f64> {
        if node_features.is_empty() {
            return na::DVector::zeros(self.config.output_dim);
        }

        // Mean pooling
        let mut sum = na::DVector::zeros(self.config.hidden_dim);
        for features in node_features.values() {
            sum += features;
        }
        let mean = sum / node_features.len() as f64;

        // Linear transformation to output dimension
        &self.readout_weights * &mean
    }
}

/// GNN Engine for materials
pub struct GNNEngine {
    models: Arc<RwLock<HashMap<String, GNNModel>>>,
    graphs: Arc<RwLock<HashMap<Uuid, MolecularGraph>>>,
    config: GNNConfig,
}

impl GNNEngine {
    pub fn new() -> Self {
        Self::with_config(GNNConfig::default())
    }

    pub fn with_config(config: GNNConfig) -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            graphs: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create graph from material
    pub async fn create_graph(
        &self,
        material_id: Uuid,
        formula: &str,
        atoms: Vec<AtomNode>,
        bonds: Vec<BondEdge>,
    ) -> Result<MolecularGraph, String> {
        // Build adjacency list
        let mut adjacency = HashMap::new();
        for bond in &bonds {
            adjacency.entry(bond.from_atom)
                .or_insert_with(Vec::new)
                .push(bond.to_atom);
            adjacency.entry(bond.to_atom)
                .or_insert_with(Vec::new)
                .push(bond.from_atom);
        }

        let graph = MolecularGraph {
            material_id,
            formula: formula.to_string(),
            nodes: atoms,
            edges: bonds,
            adjacency,
            global_features: Vec::new(),
        };

        self.graphs.write().await.insert(material_id, graph.clone());

        Ok(graph)
    }

    /// Predict property using GNN
    pub async fn predict_property(
        &self,
        material_id: Uuid,
        property_name: &str,
    ) -> Result<f64, String> {
        let graphs = self.graphs.read().await;
        let graph = graphs.get(&material_id)
            .ok_or("Graph not found")?;

        let models = self.models.read().await;
        let model = models.get(property_name)
            .ok_or("Model not found")?;

        let embedding = model.forward(graph);

        // Simple prediction: sum of embedding (in practice would use separate predictor)
        Ok(embedding.sum() / embedding.len() as f64)
    }

    /// Train GNN model (simplified - gradient descent not implemented)
    pub async fn train_model(
        &self,
        property_name: String,
        training_data: Vec<(Uuid, f64)>,
    ) -> Result<(), String> {
        // Create new model
        let model = GNNModel::new(self.config.clone());

        // In production: implement backpropagation and gradient descent
        // For now, just store the initialized model

        self.models.write().await.insert(property_name, model);

        Ok(())
    }

    /// Get graph embedding
    pub async fn get_embedding(&self, material_id: Uuid) -> Result<Vec<f64>, String> {
        let graphs = self.graphs.read().await;
        let graph = graphs.get(&material_id)
            .ok_or("Graph not found")?;

        // Use a default model or create one
        let model = GNNModel::new(self.config.clone());
        let embedding = model.forward(graph);

        Ok(embedding.as_slice().to_vec())
    }

    /// Get statistics
    pub async fn get_statistics(&self) -> GNNStats {
        let graphs = self.graphs.read().await;
        let models = self.models.read().await;

        GNNStats {
            total_graphs: graphs.len(),
            total_models: models.len(),
            config: self.config.clone(),
        }
    }
}

impl Default for GNNEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GNNStats {
    pub total_graphs: usize,
    pub total_models: usize,
    pub config: GNNConfig,
}

/// Utility: Create atom node from element
pub fn create_atom_node(
    atom_id: usize,
    element: &str,
    position: [f64; 3],
) -> AtomNode {
    let atomic_number = get_atomic_number(element);
    let features = generate_atom_features(element, atomic_number);

    AtomNode {
        atom_id,
        element: element.to_string(),
        atomic_number,
        features,
        coordinates: position,
    }
}

/// Get atomic number from element symbol
fn get_atomic_number(element: &str) -> u8 {
    match element {
        "H" => 1, "He" => 2, "Li" => 3, "Be" => 4, "B" => 5, "C" => 6,
        "N" => 7, "O" => 8, "F" => 9, "Ne" => 10, "Na" => 11, "Mg" => 12,
        "Al" => 13, "Si" => 14, "P" => 15, "S" => 16, "Cl" => 17, "Ar" => 18,
        "K" => 19, "Ca" => 20, "Ti" => 22, "Fe" => 26, "Co" => 27, "Ni" => 28,
        "Cu" => 29, "Zn" => 30, "Ga" => 31, "Ge" => 32, "As" => 33, "Se" => 34,
        "Br" => 35, "Mo" => 42, "Ag" => 47, "Sn" => 50, "I" => 53, "Xe" => 54,
        "Au" => 79, "Pb" => 82,
        _ => 0,
    }
}

/// Generate atomic features
fn generate_atom_features(element: &str, atomic_number: u8) -> Vec<f64> {
    let mut features = vec![0.0; 64];

    // One-hot encoding for common elements (first 20 positions)
    if let Some(idx) = ["H", "C", "N", "O", "F", "Si", "P", "S", "Cl", "Fe",
                         "Cu", "Ag", "Au", "Ti", "Al", "Li", "Na", "K", "Mg", "Ca"]
        .iter().position(|&e| e == element) {
        features[idx] = 1.0;
    }

    // Atomic properties
    features[20] = atomic_number as f64 / 100.0;  // Normalized atomic number
    features[21] = get_electronegativity(element);
    features[22] = get_atomic_radius(element);
    features[23] = get_ionization_energy(element);

    // Additional features (simplified)
    for i in 24..64 {
        features[i] = (atomic_number as f64 * (i as f64)) / 1000.0;
    }

    features
}

fn get_electronegativity(element: &str) -> f64 {
    match element {
        "H" => 2.20, "C" => 2.55, "N" => 3.04, "O" => 3.44, "F" => 3.98,
        "Si" => 1.90, "P" => 2.19, "S" => 2.58, "Cl" => 3.16, "Fe" => 1.83,
        _ => 2.0,
    }
}

fn get_atomic_radius(element: &str) -> f64 {
    match element {
        "H" => 0.37, "C" => 0.77, "N" => 0.75, "O" => 0.73, "F" => 0.71,
        "Si" => 1.18, "P" => 1.10, "S" => 1.04, "Cl" => 0.99, "Fe" => 1.26,
        _ => 1.0,
    }
}

fn get_ionization_energy(element: &str) -> f64 {
    match element {
        "H" => 13.6, "C" => 11.3, "N" => 14.5, "O" => 13.6, "F" => 17.4,
        "Si" => 8.2, "P" => 10.5, "S" => 10.4, "Cl" => 13.0, "Fe" => 7.9,
        _ => 10.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gnn_creation() {
        let engine = GNNEngine::new();
        let stats = engine.get_statistics().await;

        assert_eq!(stats.total_graphs, 0);
        assert_eq!(stats.total_models, 0);
    }

    #[tokio::test]
    async fn test_graph_creation() {
        let engine = GNNEngine::new();
        let material_id = Uuid::new_v4();

        let atoms = vec![
            create_atom_node(0, "Fe", [0.0, 0.0, 0.0]),
            create_atom_node(1, "O", [1.0, 0.0, 0.0]),
            create_atom_node(2, "O", [0.0, 1.0, 0.0]),
        ];

        let bonds = vec![
            BondEdge {
                from_atom: 0,
                to_atom: 1,
                bond_type: BondType::Ionic,
                distance: 1.0,
                features: vec![1.0, 0.0, 0.0],
            },
            BondEdge {
                from_atom: 0,
                to_atom: 2,
                bond_type: BondType::Ionic,
                distance: 1.0,
                features: vec![1.0, 0.0, 0.0],
            },
        ];

        let graph = engine.create_graph(material_id, "FeO2", atoms, bonds).await.unwrap();

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
    }

    #[tokio::test]
    async fn test_gnn_embedding() {
        let engine = GNNEngine::new();
        let material_id = Uuid::new_v4();

        let atoms = vec![
            create_atom_node(0, "Fe", [0.0, 0.0, 0.0]),
            create_atom_node(1, "O", [1.0, 0.0, 0.0]),
        ];

        let bonds = vec![
            BondEdge {
                from_atom: 0,
                to_atom: 1,
                bond_type: BondType::Ionic,
                distance: 1.0,
                features: vec![1.0; 32],
            },
        ];

        engine.create_graph(material_id, "FeO", atoms, bonds).await.unwrap();

        let embedding = engine.get_embedding(material_id).await.unwrap();

        assert_eq!(embedding.len(), 64);  // output_dim from default config
    }
}
