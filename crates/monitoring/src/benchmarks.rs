//! Automated Benchmarking with Regression Detection
//!
//! Features:
//! - Automatic performance regression detection
//! - Historical benchmark comparison
//! - Statistical analysis of performance trends
//! - Alerting on performance degradation

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration_ms: f64,
    pub timestamp: String,
    pub iterations: u32,
    pub throughput: Option<f64>,  // Operations per second
    pub memory_mb: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkStats {
    pub name: String,
    pub mean_ms: f64,
    pub median_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub std_dev_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAlert {
    pub benchmark_name: String,
    pub baseline_ms: f64,
    pub current_ms: f64,
    pub degradation_percent: f64,
    pub timestamp: String,
}

pub struct BenchmarkTracker {
    results: Arc<RwLock<std::collections::HashMap<String, VecDeque<BenchmarkResult>>>>,
    max_history: usize,
    regression_threshold: f64,  // Percentage degradation to trigger alert
}

impl BenchmarkTracker {
    pub fn new(max_history: usize, regression_threshold: f64) -> Self {
        Self {
            results: Arc::new(RwLock::new(std::collections::HashMap::new())),
            max_history,
            regression_threshold,
        }
    }

    /// Run a benchmark and record results
    pub async fn benchmark<F, T>(&self, name: &str, iterations: u32, f: F) -> BenchmarkResult
    where
        F: Fn() -> T,
    {
        info!("Running benchmark: {} ({} iterations)", name, iterations);

        let start = Instant::now();

        for _ in 0..iterations {
            let _ = f();
        }

        let total_duration = start.elapsed();
        let avg_duration_ms = total_duration.as_secs_f64() * 1000.0 / iterations as f64;
        let throughput = iterations as f64 / total_duration.as_secs_f64();

        let result = BenchmarkResult {
            name: name.to_string(),
            duration_ms: avg_duration_ms,
            timestamp: chrono::Utc::now().to_rfc3339(),
            iterations,
            throughput: Some(throughput),
            memory_mb: None,
        };

        self.record_result(result.clone()).await;

        info!("Benchmark complete: {} - {:.2}ms avg ({:.0} ops/s)",
              name, avg_duration_ms, throughput);

        result
    }

    /// Run an async benchmark
    pub async fn benchmark_async<F, Fut, T>(&self, name: &str, iterations: u32, f: F) -> BenchmarkResult
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        info!("Running async benchmark: {} ({} iterations)", name, iterations);

        let start = Instant::now();

        for _ in 0..iterations {
            let _ = f().await;
        }

        let total_duration = start.elapsed();
        let avg_duration_ms = total_duration.as_secs_f64() * 1000.0 / iterations as f64;
        let throughput = iterations as f64 / total_duration.as_secs_f64();

        let result = BenchmarkResult {
            name: name.to_string(),
            duration_ms: avg_duration_ms,
            timestamp: chrono::Utc::now().to_rfc3339(),
            iterations,
            throughput: Some(throughput),
            memory_mb: None,
        };

        self.record_result(result.clone()).await;

        info!("Async benchmark complete: {} - {:.2}ms avg ({:.0} ops/s)",
              name, avg_duration_ms, throughput);

        result
    }

    /// Record a benchmark result
    async fn record_result(&self, result: BenchmarkResult) {
        let mut results = self.results.write().await;
        let history = results.entry(result.name.clone()).or_insert_with(VecDeque::new);

        history.push_back(result);

        // Keep only max_history results
        while history.len() > self.max_history {
            history.pop_front();
        }
    }

    /// Get statistics for a benchmark
    pub async fn get_stats(&self, name: &str) -> Option<BenchmarkStats> {
        let results = self.results.read().await;
        let history = results.get(name)?;

        if history.is_empty() {
            return None;
        }

        let mut durations: Vec<f64> = history.iter().map(|r| r.duration_ms).collect();
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mean = durations.iter().sum::<f64>() / durations.len() as f64;
        let median = durations[durations.len() / 2];
        let p95 = durations[(durations.len() as f64 * 0.95) as usize];
        let p99 = durations[(durations.len() as f64 * 0.99) as usize];
        let min = durations[0];
        let max = durations[durations.len() - 1];

        // Calculate standard deviation
        let variance = durations.iter()
            .map(|d| {
                let diff = d - mean;
                diff * diff
            })
            .sum::<f64>() / durations.len() as f64;
        let std_dev = variance.sqrt();

        Some(BenchmarkStats {
            name: name.to_string(),
            mean_ms: mean,
            median_ms: median,
            p95_ms: p95,
            p99_ms: p99,
            std_dev_ms: std_dev,
            min_ms: min,
            max_ms: max,
            sample_count: durations.len(),
        })
    }

    /// Check for performance regressions
    pub async fn check_regression(&self, name: &str) -> Option<RegressionAlert> {
        let results = self.results.read().await;
        let history = results.get(name)?;

        if history.len() < 2 {
            return None;
        }

        // Compare latest result with baseline (median of previous results)
        let latest = history.back()?;
        let previous: Vec<f64> = history.iter()
            .rev()
            .skip(1)
            .take(10)  // Use last 10 results as baseline
            .map(|r| r.duration_ms)
            .collect();

        if previous.is_empty() {
            return None;
        }

        let baseline = previous.iter().sum::<f64>() / previous.len() as f64;
        let current = latest.duration_ms;

        let degradation_percent = ((current - baseline) / baseline) * 100.0;

        if degradation_percent > self.regression_threshold {
            let alert = RegressionAlert {
                benchmark_name: name.to_string(),
                baseline_ms: baseline,
                current_ms: current,
                degradation_percent,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            warn!("⚠️  Performance regression detected in '{}': {:.2}ms → {:.2}ms ({:+.1}%)",
                  name, baseline, current, degradation_percent);

            Some(alert)
        } else if degradation_percent < -10.0 {
            // Significant improvement
            info!("✨ Performance improvement in '{}': {:.2}ms → {:.2}ms ({:+.1}%)",
                  name, baseline, current, degradation_percent);
            None
        } else {
            None
        }
    }

    /// Get all benchmark stats
    pub async fn get_all_stats(&self) -> Vec<BenchmarkStats> {
        let results = self.results.read().await;
        let mut stats = Vec::new();

        for name in results.keys() {
            if let Some(stat) = self.get_stats(name).await {
                stats.push(stat);
            }
        }

        stats
    }

    /// Compare two benchmarks
    pub async fn compare(&self, name1: &str, name2: &str) -> Option<BenchmarkComparison> {
        let stats1 = self.get_stats(name1).await?;
        let stats2 = self.get_stats(name2).await?;

        let diff_percent = ((stats2.mean_ms - stats1.mean_ms) / stats1.mean_ms) * 100.0;

        Some(BenchmarkComparison {
            benchmark1: name1.to_string(),
            benchmark2: name2.to_string(),
            mean1_ms: stats1.mean_ms,
            mean2_ms: stats2.mean_ms,
            difference_ms: stats2.mean_ms - stats1.mean_ms,
            difference_percent: diff_percent,
        })
    }

    /// Start background regression monitoring
    pub fn start_monitoring(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(300)).await;  // Check every 5 minutes

                // Get all benchmark names first
                let names: Vec<String> = {
                    let results = self.results.read().await;
                    results.keys().cloned().collect()
                };

                // Check each benchmark for regressions
                for name in names {
                    if let Some(alert) = self.check_regression(&name).await {
                        error!("🔴 REGRESSION ALERT: {} degraded by {:.1}%",
                               alert.benchmark_name, alert.degradation_percent);
                    }
                }
            }
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub benchmark1: String,
    pub benchmark2: String,
    pub mean1_ms: f64,
    pub mean2_ms: f64,
    pub difference_ms: f64,
    pub difference_percent: f64,
}
