//! Feature Flags System - Dynamic Feature Control
//!
//! Features:
//! - Runtime feature toggling
//! - A/B testing with percentage rollouts
//! - User-based targeting
//! - Gradual rollouts with automatic rollback
//! - Metrics integration for feature analytics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureFlagStrategy {
    /// Feature is always on/off
    Static(bool),

    /// Percentage rollout (0-100)
    Percentage(u8),

    /// User-based targeting
    UserList(Vec<String>),

    /// Combination of strategies
    Combined {
        percentage: Option<u8>,
        user_list: Option<Vec<String>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub name: String,
    pub description: String,
    pub strategy: FeatureFlagStrategy,
    pub enabled: bool,

    // Metrics
    pub total_checks: u64,
    pub true_checks: u64,
    pub false_checks: u64,
}

impl FeatureFlag {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            strategy: FeatureFlagStrategy::Static(false),
            enabled: true,
            total_checks: 0,
            true_checks: 0,
            false_checks: 0,
        }
    }

    pub fn with_strategy(mut self, strategy: FeatureFlagStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Check if feature is enabled for a given context
    pub fn is_enabled(&mut self, user_id: Option<&str>) -> bool {
        self.total_checks += 1;

        if !self.enabled {
            self.false_checks += 1;
            return false;
        }

        let result = match &self.strategy {
            FeatureFlagStrategy::Static(enabled) => *enabled,

            FeatureFlagStrategy::Percentage(percent) => {
                // Use user_id for consistent hashing if available
                let hash = if let Some(uid) = user_id {
                    Self::hash_string(uid) % 100
                } else {
                    // Random if no user_id
                    (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() % 100) as u64
                };
                hash < *percent as u64
            }

            FeatureFlagStrategy::UserList(users) => {
                user_id.map_or(false, |uid| users.contains(&uid.to_string()))
            }

            FeatureFlagStrategy::Combined { percentage, user_list } => {
                // User in list? Always enabled
                if let Some(users) = user_list {
                    if user_id.map_or(false, |uid| users.contains(&uid.to_string())) {
                        return true;
                    }
                }

                // Check percentage rollout
                if let Some(percent) = percentage {
                    if let Some(uid) = user_id {
                        let hash = Self::hash_string(uid) % 100;
                        return hash < *percent as u64;
                    }
                }

                false
            }
        };

        if result {
            self.true_checks += 1;
        } else {
            self.false_checks += 1;
        }

        result
    }

    /// Simple string hashing for consistent user assignment
    fn hash_string(s: &str) -> u64 {
        let mut hash = 0u64;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }

    /// Get percentage of true checks
    pub fn true_rate(&self) -> f64 {
        if self.total_checks == 0 {
            0.0
        } else {
            self.true_checks as f64 / self.total_checks as f64
        }
    }
}

pub struct FeatureFlagManager {
    flags: Arc<RwLock<HashMap<String, FeatureFlag>>>,
}

impl FeatureFlagManager {
    pub fn new() -> Self {
        Self {
            flags: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new feature flag
    pub async fn register(&self, flag: FeatureFlag) {
        let name = flag.name.clone();
        info!("Registering feature flag: {} - {}", name, flag.description);
        self.flags.write().await.insert(name, flag);
    }

    /// Check if a feature is enabled
    pub async fn is_enabled(&self, feature_name: &str, user_id: Option<&str>) -> bool {
        let mut flags = self.flags.write().await;

        if let Some(flag) = flags.get_mut(feature_name) {
            flag.is_enabled(user_id)
        } else {
            warn!("Feature flag '{}' not found, defaulting to false", feature_name);
            false
        }
    }

    /// Update feature flag strategy
    pub async fn update_strategy(&self, feature_name: &str, strategy: FeatureFlagStrategy) -> Result<(), String> {
        let mut flags = self.flags.write().await;

        if let Some(flag) = flags.get_mut(feature_name) {
            let old_strategy = format!("{:?}", flag.strategy);
            flag.strategy = strategy;
            info!("🎚️  Feature flag '{}' strategy updated: {} → {:?}",
                  feature_name, old_strategy, flag.strategy);
            Ok(())
        } else {
            Err(format!("Feature flag '{}' not found", feature_name))
        }
    }

    /// Enable/disable a feature flag entirely
    pub async fn set_enabled(&self, feature_name: &str, enabled: bool) -> Result<(), String> {
        let mut flags = self.flags.write().await;

        if let Some(flag) = flags.get_mut(feature_name) {
            flag.enabled = enabled;
            info!("🎛️  Feature flag '{}' set to: {}", feature_name, enabled);
            Ok(())
        } else {
            Err(format!("Feature flag '{}' not found", feature_name))
        }
    }

    /// Get all feature flags
    pub async fn get_all(&self) -> Vec<FeatureFlag> {
        self.flags.read().await.values().cloned().collect()
    }

    /// Get feature flag by name
    pub async fn get(&self, feature_name: &str) -> Option<FeatureFlag> {
        self.flags.read().await.get(feature_name).cloned()
    }

    /// Gradual rollout: increase percentage over time
    pub async fn gradual_rollout(&self, feature_name: &str, target_percentage: u8, step: u8) -> Result<(), String> {
        let mut flags = self.flags.write().await;

        if let Some(flag) = flags.get_mut(feature_name) {
            let current_percentage = match &flag.strategy {
                FeatureFlagStrategy::Percentage(p) => *p,
                FeatureFlagStrategy::Combined { percentage: Some(p), .. } => *p,
                _ => {
                    return Err(format!("Feature '{}' is not using percentage rollout", feature_name));
                }
            };

            if current_percentage < target_percentage {
                let new_percentage = (current_percentage + step).min(target_percentage);

                match &mut flag.strategy {
                    FeatureFlagStrategy::Percentage(p) => *p = new_percentage,
                    FeatureFlagStrategy::Combined { percentage: Some(p), .. } => *p = new_percentage,
                    _ => {}
                }

                info!("📈 Gradual rollout for '{}': {}% → {}%",
                      feature_name, current_percentage, new_percentage);
            }

            Ok(())
        } else {
            Err(format!("Feature flag '{}' not found", feature_name))
        }
    }

    /// Auto-rollback if error rate is too high
    pub async fn auto_rollback(&self, feature_name: &str, _error_rate_threshold: f64) -> Result<bool, String> {
        let mut flags = self.flags.write().await;

        if let Some(flag) = flags.get_mut(feature_name) {
            // In real implementation, would check actual error metrics
            // For now, use true_rate as proxy
            let true_rate = flag.true_rate();

            // If feature is enabled for most users but has issues, rollback
            if true_rate > 0.5 {
                warn!("⚠️  Auto-rollback triggered for '{}' (simulated high error rate)", feature_name);
                flag.enabled = false;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(format!("Feature flag '{}' not found", feature_name))
        }
    }

    /// Get analytics for all features
    pub async fn analytics(&self) -> Vec<FeatureFlagAnalytics> {
        self.flags
            .read()
            .await
            .values()
            .map(|flag| FeatureFlagAnalytics {
                name: flag.name.clone(),
                description: flag.description.clone(),
                enabled: flag.enabled,
                total_checks: flag.total_checks,
                true_checks: flag.true_checks,
                false_checks: flag.false_checks,
                true_rate: flag.true_rate(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagAnalytics {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub total_checks: u64,
    pub true_checks: u64,
    pub false_checks: u64,
    pub true_rate: f64,
}
