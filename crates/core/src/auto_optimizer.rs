//! Auto-Optimization Engine - Self-Adaptive System
//!
//! Cognitive system that automatically adjusts parameters based on performance metrics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationMetrics {
    pub throughput: f64,          // Requests per second
    pub latency_p50: f64,          // Median latency in ms
    pub latency_p95: f64,          // 95th percentile latency
    pub error_rate: f64,           // Error rate (0.0 to 1.0)
    pub cpu_usage: f64,            // CPU usage (0.0 to 1.0)
    pub memory_usage_mb: f64,      // Memory usage in MB
    pub cache_hit_rate: f64,       // Cache hit rate (0.0 to 1.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizableParameters {
    pub worker_pool_size: usize,
    pub connection_pool_size: usize,
    pub cache_size_mb: usize,
    pub request_timeout_ms: u64,
    pub batch_size: usize,
    pub prefetch_enabled: bool,
}

impl Default for OptimizableParameters {
    fn default() -> Self {
        Self {
            worker_pool_size: 4,
            connection_pool_size: 10,
            cache_size_mb: 100,
            request_timeout_ms: 5000,
            batch_size: 10,
            prefetch_enabled: false,
        }
    }
}

pub struct AutoOptimizer {
    current_params: Arc<RwLock<OptimizableParameters>>,
    metrics_history: Arc<RwLock<Vec<(Instant, OptimizationMetrics)>>>,
    optimization_enabled: Arc<RwLock<bool>>,
    last_optimization: Arc<RwLock<Instant>>,
    optimization_interval: Duration,
}

impl AutoOptimizer {
    pub fn new() -> Self {
        Self {
            current_params: Arc::new(RwLock::new(OptimizableParameters::default())),
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            optimization_enabled: Arc::new(RwLock::new(true)),
            last_optimization: Arc::new(RwLock::new(Instant::now())),
            optimization_interval: Duration::from_secs(60), // Optimize every minute
        }
    }

    /// Record current system metrics
    pub async fn record_metrics(&self, metrics: OptimizationMetrics) {
        let mut history = self.metrics_history.write().await;
        history.push((Instant::now(), metrics));

        // Keep only last hour of metrics
        let cutoff = Instant::now() - Duration::from_secs(3600);
        history.retain(|(time, _)| *time > cutoff);
    }

    /// Get current parameters
    pub async fn get_parameters(&self) -> OptimizableParameters {
        self.current_params.read().await.clone()
    }

    /// Run optimization cycle
    pub async fn optimize(&self) -> OptimizationResult {
        if !*self.optimization_enabled.read().await {
            return OptimizationResult {
                optimized: false,
                changes: HashMap::new(),
                reason: "Optimization disabled".to_string(),
            };
        }

        let last_opt = *self.last_optimization.read().await;
        if last_opt.elapsed() < self.optimization_interval {
            return OptimizationResult {
                optimized: false,
                changes: HashMap::new(),
                reason: "Too soon since last optimization".to_string(),
            };
        }

        // Analyze recent metrics
        let history = self.metrics_history.read().await;
        if history.is_empty() {
            return OptimizationResult {
                optimized: false,
                changes: HashMap::new(),
                reason: "No metrics available".to_string(),
            };
        }

        let recent_metrics: Vec<&OptimizationMetrics> = history.iter()
            .filter(|(time, _)| time.elapsed() < Duration::from_secs(300))
            .map(|(_, m)| m)
            .collect();

        if recent_metrics.is_empty() {
            return OptimizationResult {
                optimized: false,
                changes: HashMap::new(),
                reason: "No recent metrics".to_string(),
            };
        }

        // Calculate average metrics
        let avg_latency = recent_metrics.iter().map(|m| m.latency_p95).sum::<f64>() / recent_metrics.len() as f64;
        let avg_error_rate = recent_metrics.iter().map(|m| m.error_rate).sum::<f64>() / recent_metrics.len() as f64;
        let avg_cache_hit = recent_metrics.iter().map(|m| m.cache_hit_rate).sum::<f64>() / recent_metrics.len() as f64;
        let avg_cpu = recent_metrics.iter().map(|m| m.cpu_usage).sum::<f64>() / recent_metrics.len() as f64;

        let mut params = self.current_params.write().await;
        let mut changes = HashMap::new();

        // Rule-based optimization
        // High latency → increase workers
        if avg_latency > 1000.0 && params.worker_pool_size < 16 {
            let old = params.worker_pool_size;
            params.worker_pool_size = (params.worker_pool_size * 2).min(16);
            changes.insert("worker_pool_size".to_string(), format!("{} → {}", old, params.worker_pool_size));
            info!("🔧 Auto-optimization: Increased workers {} → {} (high latency)", old, params.worker_pool_size);
        }

        // High error rate → increase timeout
        if avg_error_rate > 0.05 && params.request_timeout_ms < 30000 {
            let old = params.request_timeout_ms;
            params.request_timeout_ms = (params.request_timeout_ms as f64 * 1.5) as u64;
            changes.insert("request_timeout_ms".to_string(), format!("{} → {}", old, params.request_timeout_ms));
            info!("🔧 Auto-optimization: Increased timeout {} → {} (high error rate)", old, params.request_timeout_ms);
        }

        // Low cache hit → increase cache size
        if avg_cache_hit < 0.7 && params.cache_size_mb < 500 {
            let old = params.cache_size_mb;
            params.cache_size_mb = (params.cache_size_mb as f64 * 1.5) as usize;
            changes.insert("cache_size_mb".to_string(), format!("{} → {}", old, params.cache_size_mb));
            info!("🔧 Auto-optimization: Increased cache {} → {} MB (low hit rate)", old, params.cache_size_mb);
        }

        // High CPU but low latency → can reduce workers to save resources
        if avg_cpu > 0.8 && avg_latency < 100.0 && params.worker_pool_size > 2 {
            let old = params.worker_pool_size;
            params.worker_pool_size = (params.worker_pool_size / 2).max(2);
            changes.insert("worker_pool_size".to_string(), format!("{} → {}", old, params.worker_pool_size));
            info!("🔧 Auto-optimization: Reduced workers {} → {} (over-provisioned)", old, params.worker_pool_size);
        }

        // Good performance → enable prefetching
        if avg_latency < 100.0 && avg_cache_hit > 0.8 && !params.prefetch_enabled {
            params.prefetch_enabled = true;
            changes.insert("prefetch_enabled".to_string(), "false → true".to_string());
            info!("🔧 Auto-optimization: Enabled prefetching (stable performance)");
        }

        *self.last_optimization.write().await = Instant::now();

        OptimizationResult {
            optimized: !changes.is_empty(),
            changes,
            reason: format!("Optimized based on: latency={:.1}ms, errors={:.1}%, cache_hit={:.1}%, cpu={:.1}%",
                          avg_latency, avg_error_rate * 100.0, avg_cache_hit * 100.0, avg_cpu * 100.0),
        }
    }

    /// Start background optimization loop
    pub fn start_background_optimization(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;

                match self.optimize().await {
                    OptimizationResult { optimized: true, changes, reason } => {
                        info!("✨ Auto-optimization completed: {} changes", changes.len());
                        info!("   Reason: {}", reason);
                        for (param, change) in changes {
                            info!("   • {}: {}", param, change);
                        }
                    }
                    _ => {
                        // No optimization needed
                    }
                }
            }
        });
    }

    /// Enable/disable optimization
    pub async fn set_enabled(&self, enabled: bool) {
        *self.optimization_enabled.write().await = enabled;
        if enabled {
            info!("✅ Auto-optimization enabled");
        } else {
            warn!("⚠️  Auto-optimization disabled");
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub optimized: bool,
    pub changes: HashMap<String, String>,
    pub reason: String,
}
