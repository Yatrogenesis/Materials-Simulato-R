//! Knowledge Graph - Semantic relationships between materials
//!
//! Automatically constructs and maintains a knowledge graph of materials,
//! discovering relationships, patterns, and insights.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Relationship types in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationType {
    /// Materials with similar composition
    SimilarComposition,
    /// Materials with similar properties
    SimilarProperties,
    /// Materials in same crystal system
    SameCrystalSystem,
    /// Materials containing same element
    ContainsElement(String),
    /// Materials with parent-child relationship (substitution)
    Substitution,
    /// Materials in same application domain
    SameApplication,
    /// Materials discovered by same method
    SameMethod,
    /// Custom relationship
    Custom(String),
}

/// Edge in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub relation_type: RelationType,
    pub weight: f64,
    pub confidence: f64,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Node in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub material_id: Uuid,
    pub formula: String,
    pub properties: HashMap<String, f64>,
    pub tags: Vec<String>,
    pub importance_score: f64,
}

/// Knowledge Graph Engine
pub struct KnowledgeGraph {
    /// Nodes (materials)
    nodes: Arc<RwLock<HashMap<Uuid, KnowledgeNode>>>,

    /// Edges (relationships)
    edges: Arc<RwLock<Vec<KnowledgeEdge>>>,

    /// Adjacency list for fast traversal
    adjacency: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,

    /// Community detection cache
    communities: Arc<RwLock<Vec<Vec<Uuid>>>>,

    /// Centrality scores
    centrality_scores: Arc<RwLock<HashMap<Uuid, f64>>>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            edges: Arc::new(RwLock::new(Vec::new())),
            adjacency: Arc::new(RwLock::new(HashMap::new())),
            communities: Arc::new(RwLock::new(Vec::new())),
            centrality_scores: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a material node to the graph
    pub async fn add_node(&self, node: KnowledgeNode) -> Result<(), String> {
        let mut nodes = self.nodes.write().await;
        nodes.insert(node.material_id, node);
        Ok(())
    }

    /// Add a relationship edge
    pub async fn add_edge(&self, edge: KnowledgeEdge) -> Result<(), String> {
        let mut edges = self.edges.write().await;
        let mut adjacency = self.adjacency.write().await;

        // Add to edges
        edges.push(edge.clone());

        // Update adjacency list
        adjacency.entry(edge.from)
            .or_insert_with(Vec::new)
            .push(edge.to);

        adjacency.entry(edge.to)
            .or_insert_with(Vec::new)
            .push(edge.from);

        Ok(())
    }

    /// Auto-discover relationships between materials
    pub async fn auto_discover_relationships(&self) -> Result<usize, String> {
        let nodes = self.nodes.read().await;
        let node_list: Vec<_> = nodes.values().cloned().collect();
        drop(nodes);

        let mut discovered = 0;

        // Discover composition similarity
        for i in 0..node_list.len() {
            for j in (i + 1)..node_list.len() {
                let node_i = &node_list[i];
                let node_j = &node_list[j];

                // Check composition similarity
                let composition_sim = Self::composition_similarity(
                    &node_i.formula,
                    &node_j.formula,
                );

                if composition_sim > 0.7 {
                    self.add_edge(KnowledgeEdge {
                        from: node_i.material_id,
                        to: node_j.material_id,
                        relation_type: RelationType::SimilarComposition,
                        weight: composition_sim,
                        confidence: 0.9,
                        metadata: HashMap::new(),
                        created_at: chrono::Utc::now(),
                    }).await?;
                    discovered += 1;
                }

                // Check property similarity
                let property_sim = Self::property_similarity(
                    &node_i.properties,
                    &node_j.properties,
                );

                if property_sim > 0.8 {
                    self.add_edge(KnowledgeEdge {
                        from: node_i.material_id,
                        to: node_j.material_id,
                        relation_type: RelationType::SimilarProperties,
                        weight: property_sim,
                        confidence: 0.85,
                        metadata: HashMap::new(),
                        created_at: chrono::Utc::now(),
                    }).await?;
                    discovered += 1;
                }

                // Check common elements
                let common_elements = Self::common_elements(
                    &node_i.formula,
                    &node_j.formula,
                );

                for element in common_elements {
                    self.add_edge(KnowledgeEdge {
                        from: node_i.material_id,
                        to: node_j.material_id,
                        relation_type: RelationType::ContainsElement(element.clone()),
                        weight: 0.5,
                        confidence: 1.0,
                        metadata: HashMap::new(),
                        created_at: chrono::Utc::now(),
                    }).await?;
                    discovered += 1;
                }
            }
        }

        Ok(discovered)
    }

    /// Find path between two materials
    pub async fn find_path(
        &self,
        from: Uuid,
        to: Uuid,
    ) -> Option<Vec<Uuid>> {
        let adjacency = self.adjacency.read().await;

        // BFS to find shortest path
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(from);
        visited.insert(from);

        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = to;
                path.push(node);

                while let Some(&p) = parent.get(&node) {
                    path.push(p);
                    node = p;
                }

                path.reverse();
                return Some(path);
            }

            if let Some(neighbors) = adjacency.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        parent.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        None
    }

    /// Get neighbors of a material
    pub async fn get_neighbors(
        &self,
        material_id: Uuid,
        relation_type: Option<RelationType>,
    ) -> Vec<KnowledgeNode> {
        let edges = self.edges.read().await;
        let nodes = self.nodes.read().await;

        let mut neighbors = Vec::new();

        for edge in edges.iter() {
            if edge.from == material_id {
                if let Some(ref rel_type) = relation_type {
                    if &edge.relation_type != rel_type {
                        continue;
                    }
                }

                if let Some(node) = nodes.get(&edge.to) {
                    neighbors.push(node.clone());
                }
            } else if edge.to == material_id {
                if let Some(ref rel_type) = relation_type {
                    if &edge.relation_type != rel_type {
                        continue;
                    }
                }

                if let Some(node) = nodes.get(&edge.from) {
                    neighbors.push(node.clone());
                }
            }
        }

        neighbors
    }

    /// Detect communities using Louvain algorithm (simplified)
    pub async fn detect_communities(&self) -> Vec<Vec<Uuid>> {
        let nodes = self.nodes.read().await;
        let edges = self.edges.read().await;

        let node_ids: Vec<Uuid> = nodes.keys().copied().collect();
        let n = node_ids.len();

        if n == 0 {
            return Vec::new();
        }

        // Build adjacency matrix
        let mut adj_matrix = vec![vec![0.0; n]; n];
        let id_to_idx: HashMap<Uuid, usize> = node_ids.iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();

        for edge in edges.iter() {
            if let (Some(&i), Some(&j)) = (id_to_idx.get(&edge.from), id_to_idx.get(&edge.to)) {
                adj_matrix[i][j] = edge.weight;
                adj_matrix[j][i] = edge.weight;
            }
        }

        // Simple community detection: connected components
        let mut visited = vec![false; n];
        let mut communities = Vec::new();

        for i in 0..n {
            if !visited[i] {
                let mut community = Vec::new();
                let mut stack = vec![i];

                while let Some(node) = stack.pop() {
                    if visited[node] {
                        continue;
                    }

                    visited[node] = true;
                    community.push(node_ids[node]);

                    for j in 0..n {
                        if !visited[j] && adj_matrix[node][j] > 0.0 {
                            stack.push(j);
                        }
                    }
                }

                if !community.is_empty() {
                    communities.push(community);
                }
            }
        }

        // Cache communities
        let mut cached = self.communities.write().await;
        *cached = communities.clone();

        communities
    }

    /// Calculate centrality scores (PageRank-like)
    pub async fn calculate_centrality(&self) -> HashMap<Uuid, f64> {
        let nodes = self.nodes.read().await;
        let adjacency = self.adjacency.read().await;

        let mut scores: HashMap<Uuid, f64> = nodes.keys()
            .map(|&id| (id, 1.0))
            .collect();

        let damping = 0.85;
        let iterations = 20;

        for _ in 0..iterations {
            let mut new_scores = HashMap::new();

            for (&node, _) in nodes.iter() {
                let mut score = (1.0 - damping) / nodes.len() as f64;

                // Get incoming edges
                for (&other_node, neighbors) in adjacency.iter() {
                    if neighbors.contains(&node) && other_node != node {
                        let out_degree = neighbors.len() as f64;
                        if out_degree > 0.0 {
                            score += damping * scores.get(&other_node).unwrap_or(&1.0) / out_degree;
                        }
                    }
                }

                new_scores.insert(node, score);
            }

            scores = new_scores;
        }

        // Cache centrality scores
        let mut cached = self.centrality_scores.write().await;
        *cached = scores.clone();

        scores
    }

    /// Get most important materials (by centrality)
    pub async fn get_most_important(&self, top_k: usize) -> Vec<(Uuid, f64)> {
        let centrality = self.centrality_scores.read().await;

        let mut sorted: Vec<_> = centrality.iter()
            .map(|(&id, &score)| (id, score))
            .collect();

        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        sorted.into_iter().take(top_k).collect()
    }

    /// Get graph statistics
    pub async fn get_statistics(&self) -> GraphStats {
        let nodes = self.nodes.read().await;
        let edges = self.edges.read().await;
        let communities = self.communities.read().await;

        GraphStats {
            node_count: nodes.len(),
            edge_count: edges.len(),
            community_count: communities.len(),
            average_degree: if nodes.is_empty() {
                0.0
            } else {
                (edges.len() * 2) as f64 / nodes.len() as f64
            },
            density: if nodes.len() > 1 {
                edges.len() as f64 / (nodes.len() * (nodes.len() - 1) / 2) as f64
            } else {
                0.0
            },
        }
    }

    // === Private Helper Methods ===

    fn composition_similarity(formula1: &str, formula2: &str) -> f64 {
        let elements1 = Self::extract_elements(formula1);
        let elements2 = Self::extract_elements(formula2);

        let common: HashSet<_> = elements1.intersection(&elements2).collect();
        let union: HashSet<_> = elements1.union(&elements2).collect();

        if union.is_empty() {
            return 0.0;
        }

        common.len() as f64 / union.len() as f64
    }

    fn property_similarity(props1: &HashMap<String, f64>, props2: &HashMap<String, f64>) -> f64 {
        if props1.is_empty() || props2.is_empty() {
            return 0.0;
        }

        let mut similarity = 0.0;
        let mut count = 0;

        for (key, &val1) in props1 {
            if let Some(&val2) = props2.get(key) {
                let diff = (val1 - val2).abs();
                let avg = (val1.abs() + val2.abs()) / 2.0;

                if avg > 1e-10 {
                    similarity += 1.0 - (diff / avg).min(1.0);
                    count += 1;
                }
            }
        }

        if count == 0 {
            0.0
        } else {
            similarity / count as f64
        }
    }

    fn common_elements(formula1: &str, formula2: &str) -> Vec<String> {
        let elements1 = Self::extract_elements(formula1);
        let elements2 = Self::extract_elements(formula2);

        elements1.intersection(&elements2)
            .map(|s| s.to_string())
            .collect()
    }

    fn extract_elements(formula: &str) -> HashSet<String> {
        let mut elements = HashSet::new();
        let mut current = String::new();

        for ch in formula.chars() {
            if ch.is_uppercase() {
                if !current.is_empty() {
                    elements.insert(current.clone());
                }
                current = ch.to_string();
            } else if ch.is_lowercase() {
                current.push(ch);
            } else if ch.is_numeric() {
                // Skip numbers
            }
        }

        if !current.is_empty() {
            elements.insert(current);
        }

        elements
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub community_count: usize,
    pub average_degree: f64,
    pub density: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_graph_creation() {
        let kg = KnowledgeGraph::new();

        let node = KnowledgeNode {
            material_id: Uuid::new_v4(),
            formula: "Fe2O3".to_string(),
            properties: HashMap::new(),
            tags: vec!["oxide".to_string()],
            importance_score: 0.8,
        };

        kg.add_node(node).await.unwrap();

        let stats = kg.get_statistics().await;
        assert_eq!(stats.node_count, 1);
    }

    #[tokio::test]
    async fn test_relationship_discovery() {
        let kg = KnowledgeGraph::new();

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        kg.add_node(KnowledgeNode {
            material_id: id1,
            formula: "Fe2O3".to_string(),
            properties: HashMap::new(),
            tags: Vec::new(),
            importance_score: 0.8,
        }).await.unwrap();

        kg.add_node(KnowledgeNode {
            material_id: id2,
            formula: "Fe3O4".to_string(),
            properties: HashMap::new(),
            tags: Vec::new(),
            importance_score: 0.7,
        }).await.unwrap();

        let discovered = kg.auto_discover_relationships().await.unwrap();
        assert!(discovered > 0);
    }
}
