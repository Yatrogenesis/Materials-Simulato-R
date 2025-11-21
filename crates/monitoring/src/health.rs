//! Comprehensive Health Check System
//!
//! Multi-dimensional health monitoring for all system components

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentHealthStatus {
    Healthy,      // All systems operational
    Degraded,     // Some issues but functional
    Unhealthy,    // Critical issues
    Unknown,      // Status not determined
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: ComponentHealthStatus,
    pub message: Option<String>,
    pub last_check: Option<String>,
    pub response_time_ms: Option<u64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_status: ComponentHealthStatus,
    pub components: HashMap<String, ComponentHealth>,
    pub timestamp: String,
    pub uptime_seconds: u64,
    pub version: String,
}

pub struct HealthChecker {
    checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck>>>>,
    results: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    start_time: Instant,
}

#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform health check and return status
    async fn check(&self) -> ComponentHealth;

    /// Get check name
    fn name(&self) -> &str;

    /// Get check interval
    fn interval(&self) -> Duration {
        Duration::from_secs(30)
    }
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Register a new health check
    pub async fn register(&self, check: Box<dyn HealthCheck>) {
        let name = check.name().to_string();
        info!("Registering health check: {}", name);
        self.checks.write().await.insert(name.clone(), check);

        // Initialize with unknown status
        self.results.write().await.insert(name.clone(), ComponentHealth {
            name: name.clone(),
            status: ComponentHealthStatus::Unknown,
            message: Some("Not yet checked".to_string()),
            last_check: None,
            response_time_ms: None,
            metadata: HashMap::new(),
        });
    }

    /// Run all health checks
    pub async fn check_all(&self) -> SystemHealth {
        let checks = self.checks.read().await;
        let mut results = HashMap::new();
        let mut overall_healthy = true;
        let mut overall_degraded = false;

        for (name, check) in checks.iter() {
            let start = Instant::now();
            let result = check.check().await;
            let elapsed = start.elapsed();

            info!("Health check '{}': {:?} ({:.2}ms)",
                  name, result.status, elapsed.as_secs_f64() * 1000.0);

            match result.status {
                ComponentHealthStatus::Healthy => {},
                ComponentHealthStatus::Degraded => overall_degraded = true,
                ComponentHealthStatus::Unhealthy => overall_healthy = false,
                ComponentHealthStatus::Unknown => overall_degraded = true,
            }

            results.insert(name.clone(), result);
        }

        // Cache results
        *self.results.write().await = results.clone();

        let overall_status = if !overall_healthy {
            ComponentHealthStatus::Unhealthy
        } else if overall_degraded {
            ComponentHealthStatus::Degraded
        } else {
            ComponentHealthStatus::Healthy
        };

        SystemHealth {
            overall_status,
            components: results,
            timestamp: chrono::Utc::now().to_rfc3339(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            version: crate::VERSION.to_string(),
        }
    }

    /// Get cached health status (fast, doesn't re-check)
    pub async fn get_cached_health(&self) -> SystemHealth {
        let results = self.results.read().await.clone();
        let mut overall_healthy = true;
        let mut overall_degraded = false;

        for health in results.values() {
            match health.status {
                ComponentHealthStatus::Healthy => {},
                ComponentHealthStatus::Degraded => overall_degraded = true,
                ComponentHealthStatus::Unhealthy => overall_healthy = false,
                ComponentHealthStatus::Unknown => overall_degraded = true,
            }
        }

        let overall_status = if !overall_healthy {
            ComponentHealthStatus::Unhealthy
        } else if overall_degraded {
            ComponentHealthStatus::Degraded
        } else {
            ComponentHealthStatus::Healthy
        };

        SystemHealth {
            overall_status,
            components: results,
            timestamp: chrono::Utc::now().to_rfc3339(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            version: crate::VERSION.to_string(),
        }
    }

    /// Start background health checking
    pub fn start_background_checks(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;

                let health = self.check_all().await;

                match health.overall_status {
                    ComponentHealthStatus::Healthy => {
                        info!("✅ System health: All systems operational");
                    }
                    ComponentHealthStatus::Degraded => {
                        warn!("⚠️  System health: Some components degraded");
                    }
                    ComponentHealthStatus::Unhealthy => {
                        error!("❌ System health: Critical issues detected!");
                    }
                    ComponentHealthStatus::Unknown => {
                        warn!("❓ System health: Status unknown");
                    }
                }
            }
        });
    }
}

/// Simple health check - always returns healthy (for testing)
pub struct SimpleHealthCheck {
    name: String,
}

impl SimpleHealthCheck {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait::async_trait]
impl HealthCheck for SimpleHealthCheck {
    async fn check(&self) -> ComponentHealth {
        ComponentHealth {
            name: self.name.clone(),
            status: ComponentHealthStatus::Healthy,
            message: Some("Service operational".to_string()),
            last_check: Some(chrono::Utc::now().to_rfc3339()),
            response_time_ms: Some(1),
            metadata: HashMap::new(),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// Legacy compatibility
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub version: String,
    pub uptime_seconds: u64,
}

impl HealthStatus {
    pub fn new(uptime_seconds: u64) -> Self {
        Self {
            healthy: true,
            version: crate::VERSION.to_string(),
            uptime_seconds,
        }
    }
}
