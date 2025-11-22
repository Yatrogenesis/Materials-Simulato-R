//! Intelligent Material Recommendation System
//!
//! Provides personalized recommendations for materials based on:
//! - User preferences and history
//! - Application requirements
//! - Property constraints
//! - Collaborative filtering

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Recommendation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationContext {
    /// User ID (optional)
    pub user_id: Option<Uuid>,

    /// Application domain
    pub application: String,

    /// Required properties
    pub required_properties: HashMap<String, f64>,

    /// Preferences
    pub preferences: UserPreferences,

    /// Budget constraints
    pub budget: Option<BudgetConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub prefer_common_elements: bool,
    pub prefer_environmentally_friendly: bool,
    pub prefer_cost_effective: bool,
    pub preferred_crystal_systems: Vec<String>,
    pub avoid_toxic_elements: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            prefer_common_elements: true,
            prefer_environmentally_friendly: true,
            prefer_cost_effective: true,
            preferred_crystal_systems: Vec::new(),
            avoid_toxic_elements: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConstraint {
    pub max_cost_per_kg: f64,
    pub synthesis_complexity_max: f64,
}

/// Material recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialRecommendation {
    pub material_id: Uuid,
    pub formula: String,
    pub recommendation_score: f64,
    pub confidence: f64,
    pub reasons: Vec<String>,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub estimated_cost: Option<f64>,
    pub availability: AvailabilityScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AvailabilityScore {
    High,
    Medium,
    Low,
    Unknown,
}

/// Recommendation engine
pub struct RecommendationEngine {
    /// User interaction history
    user_history: Arc<RwLock<HashMap<Uuid, Vec<MaterialInteraction>>>>,

    /// Material metadata cache
    material_metadata: Arc<RwLock<HashMap<Uuid, MaterialMetadata>>>,

    /// Collaborative filtering model
    similarity_matrix: Arc<RwLock<HashMap<(Uuid, Uuid), f64>>>,
}

#[derive(Debug, Clone)]
struct MaterialInteraction {
    material_id: Uuid,
    interaction_type: InteractionType,
    timestamp: chrono::DateTime<chrono::Utc>,
    rating: Option<f64>,
}

#[derive(Debug, Clone)]
enum InteractionType {
    View,
    Download,
    Favorite,
    Used,
    Rated(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialMetadata {
    pub material_id: Uuid,
    pub formula: String,
    pub popularity_score: f64,
    pub average_rating: Option<f64>,
    pub applications: Vec<String>,
    pub cost_estimate: Option<f64>,
    pub environmental_impact: f64,
    pub toxicity_score: f64,
}

impl RecommendationEngine {
    pub fn new() -> Self {
        Self {
            user_history: Arc::new(RwLock::new(HashMap::new())),
            material_metadata: Arc::new(RwLock::new(HashMap::new())),
            similarity_matrix: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get personalized recommendations
    pub async fn get_recommendations(
        &self,
        context: RecommendationContext,
        candidates: Vec<Uuid>,
        top_k: usize,
    ) -> Result<Vec<MaterialRecommendation>, String> {
        let mut recommendations = Vec::new();

        for material_id in candidates {
            if let Some(recommendation) = self.evaluate_material(
                material_id,
                &context,
            ).await {
                recommendations.push(recommendation);
            }
        }

        // Sort by recommendation score
        recommendations.sort_by(|a, b| {
            b.recommendation_score.partial_cmp(&a.recommendation_score).unwrap()
        });

        // Return top-k
        Ok(recommendations.into_iter().take(top_k).collect())
    }

    /// Evaluate a single material for recommendation
    async fn evaluate_material(
        &self,
        material_id: Uuid,
        context: &RecommendationContext,
    ) -> Option<MaterialRecommendation> {
        let metadata = self.material_metadata.read().await;
        let material = metadata.get(&material_id)?;

        let mut score = 0.0;
        let mut reasons = Vec::new();
        let mut pros = Vec::new();
        let mut cons = Vec::new();

        // Application match
        if material.applications.contains(&context.application) {
            score += 2.0;
            reasons.push(format!("Suitable for {} application", context.application));
            pros.push("Application match".to_string());
        }

        // Popularity score
        score += material.popularity_score * 0.5;
        if material.popularity_score > 0.7 {
            pros.push("Widely used material".to_string());
        }

        // User preferences
        if context.preferences.prefer_environmentally_friendly {
            let env_score = 1.0 - material.environmental_impact;
            score += env_score;
            if env_score > 0.7 {
                pros.push("Environmentally friendly".to_string());
                reasons.push("Low environmental impact".to_string());
            }
        }

        if context.preferences.avoid_toxic_elements {
            if material.toxicity_score > 0.5 {
                score -= 1.0;
                cons.push("Contains toxic elements".to_string());
            } else {
                pros.push("Low toxicity".to_string());
            }
        }

        // Cost consideration
        if let Some(ref budget) = context.budget {
            if let Some(cost) = material.cost_estimate {
                if cost <= budget.max_cost_per_kg {
                    score += 1.0;
                    pros.push("Within budget".to_string());
                } else {
                    score -= 0.5;
                    cons.push("Above budget".to_string());
                }
            }
        }

        // Collaborative filtering
        if let Some(user_id) = context.user_id {
            let cf_score = self.collaborative_filtering_score(user_id, material_id).await;
            score += cf_score;
            if cf_score > 0.5 {
                reasons.push("Similar users liked this material".to_string());
            }
        }

        // Normalize score
        let recommendation_score = (score / 5.0).min(1.0).max(0.0);

        // Determine availability
        let availability = if material.popularity_score > 0.7 {
            AvailabilityScore::High
        } else if material.popularity_score > 0.4 {
            AvailabilityScore::Medium
        } else {
            AvailabilityScore::Low
        };

        Some(MaterialRecommendation {
            material_id,
            formula: material.formula.clone(),
            recommendation_score,
            confidence: 0.85,
            reasons,
            pros,
            cons,
            estimated_cost: material.cost_estimate,
            availability,
        })
    }

    /// Calculate collaborative filtering score
    async fn collaborative_filtering_score(
        &self,
        user_id: Uuid,
        material_id: Uuid,
    ) -> f64 {
        let history = self.user_history.read().await;

        // Check if user has interacted with this material
        if let Some(interactions) = history.get(&user_id) {
            for interaction in interactions {
                if interaction.material_id == material_id {
                    match interaction.interaction_type {
                        InteractionType::Favorite => return 1.0,
                        InteractionType::Used => return 0.8,
                        InteractionType::Rated(rating) => return rating / 5.0,
                        InteractionType::Download => return 0.5,
                        InteractionType::View => return 0.2,
                    }
                }
            }
        }

        // Find similar users and their preferences
        let similarity_matrix = self.similarity_matrix.read().await;
        let mut total_similarity = 0.0;
        let mut weighted_score = 0.0;

        for (&(u1, u2), &similarity) in similarity_matrix.iter() {
            if u1 == user_id {
                if let Some(other_interactions) = history.get(&u2) {
                    for interaction in other_interactions {
                        if interaction.material_id == material_id {
                            total_similarity += similarity;
                            weighted_score += similarity * 0.7; // Assume positive
                        }
                    }
                }
            }
        }

        if total_similarity > 0.0 {
            weighted_score / total_similarity
        } else {
            0.0
        }
    }

    /// Record user interaction
    pub async fn record_interaction(
        &self,
        user_id: Uuid,
        material_id: Uuid,
        interaction_type: String,
        rating: Option<f64>,
    ) -> Result<(), String> {
        let mut history = self.user_history.write().await;

        let interaction = MaterialInteraction {
            material_id,
            interaction_type: match interaction_type.as_str() {
                "view" => InteractionType::View,
                "download" => InteractionType::Download,
                "favorite" => InteractionType::Favorite,
                "used" => InteractionType::Used,
                "rated" => InteractionType::Rated(rating.unwrap_or(0.0)),
                _ => InteractionType::View,
            },
            timestamp: chrono::Utc::now(),
            rating,
        };

        history.entry(user_id)
            .or_insert_with(Vec::new)
            .push(interaction);

        Ok(())
    }

    /// Add material metadata
    pub async fn add_material_metadata(
        &self,
        metadata: MaterialMetadata,
    ) -> Result<(), String> {
        let mut materials = self.material_metadata.write().await;
        materials.insert(metadata.material_id, metadata);
        Ok(())
    }

    /// Update similarity matrix
    pub async fn update_similarity_matrix(&self) -> Result<(), String> {
        let history = self.user_history.read().await;
        let user_ids: Vec<Uuid> = history.keys().copied().collect();

        let mut similarity_matrix = self.similarity_matrix.write().await;
        similarity_matrix.clear();

        // Calculate user-user similarity
        for i in 0..user_ids.len() {
            for j in (i + 1)..user_ids.len() {
                let user1 = user_ids[i];
                let user2 = user_ids[j];

                let interactions1 = history.get(&user1).unwrap();
                let interactions2 = history.get(&user2).unwrap();

                let similarity = Self::calculate_user_similarity(interactions1, interactions2);

                similarity_matrix.insert((user1, user2), similarity);
                similarity_matrix.insert((user2, user1), similarity);
            }
        }

        Ok(())
    }

    fn calculate_user_similarity(
        interactions1: &[MaterialInteraction],
        interactions2: &[MaterialInteraction],
    ) -> f64 {
        let materials1: std::collections::HashSet<_> = interactions1.iter()
            .map(|i| i.material_id)
            .collect();

        let materials2: std::collections::HashSet<_> = interactions2.iter()
            .map(|i| i.material_id)
            .collect();

        let intersection = materials1.intersection(&materials2).count();
        let union = materials1.union(&materials2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Get trending materials
    pub async fn get_trending_materials(&self, top_k: usize) -> Vec<MaterialMetadata> {
        let materials = self.material_metadata.read().await;

        let mut sorted: Vec<_> = materials.values().cloned().collect();
        sorted.sort_by(|a, b| {
            b.popularity_score.partial_cmp(&a.popularity_score).unwrap()
        });

        sorted.into_iter().take(top_k).collect()
    }

    /// Get user's favorite materials
    pub async fn get_user_favorites(&self, user_id: Uuid) -> Vec<Uuid> {
        let history = self.user_history.read().await;

        if let Some(interactions) = history.get(&user_id) {
            interactions.iter()
                .filter(|i| matches!(i.interaction_type, InteractionType::Favorite))
                .map(|i| i.material_id)
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_recommendations() {
        let engine = RecommendationEngine::new();

        // Add material metadata
        let material_id = Uuid::new_v4();
        engine.add_material_metadata(MaterialMetadata {
            material_id,
            formula: "Fe2O3".to_string(),
            popularity_score: 0.8,
            average_rating: Some(4.5),
            applications: vec!["battery".to_string()],
            cost_estimate: Some(10.0),
            environmental_impact: 0.2,
            toxicity_score: 0.1,
        }).await.unwrap();

        // Get recommendations
        let context = RecommendationContext {
            user_id: Some(Uuid::new_v4()),
            application: "battery".to_string(),
            required_properties: HashMap::new(),
            preferences: UserPreferences::default(),
            budget: Some(BudgetConstraint {
                max_cost_per_kg: 50.0,
                synthesis_complexity_max: 0.7,
            }),
        };

        let recommendations = engine.get_recommendations(
            context,
            vec![material_id],
            10,
        ).await.unwrap();

        assert!(!recommendations.is_empty());
        assert!(recommendations[0].recommendation_score > 0.0);
    }
}
