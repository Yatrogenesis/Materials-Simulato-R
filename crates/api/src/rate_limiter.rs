//! Adaptive Rate Limiter - Self-Adjusting Traffic Control
//!
//! Features:
//! - Token bucket algorithm with dynamic refill
//! - Auto-adjusts limits based on system load
//! - Per-user and global rate limiting
//! - Burst handling with intelligent backpressure
//! - Metrics and monitoring integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
    pub per_user_limit: Option<u32>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 150,
            per_user_limit: Some(10),
        }
    }
}

/// Token bucket for rate limiting
#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64,  // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            tokens: capacity as f64,
            capacity: capacity as f64,
            refill_rate: refill_rate as f64,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        // Add tokens based on elapsed time
        let new_tokens = elapsed * self.refill_rate;
        self.tokens = (self.tokens + new_tokens).min(self.capacity);
        self.last_refill = now;
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn available_tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
    }
}

/// Adaptive rate limiter with auto-adjustment
pub struct AdaptiveRateLimiter {
    global_bucket: Arc<RwLock<TokenBucket>>,
    user_buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    config: Arc<RwLock<RateLimitConfig>>,

    // Metrics
    total_requests: Arc<std::sync::atomic::AtomicU64>,
    allowed_requests: Arc<std::sync::atomic::AtomicU64>,
    rejected_requests: Arc<std::sync::atomic::AtomicU64>,

    // Adaptive behavior
    last_adjustment: Arc<RwLock<Instant>>,
    adjustment_interval: Duration,
}

impl AdaptiveRateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        let global_bucket = TokenBucket::new(
            config.burst_size,
            config.requests_per_second,
        );

        Self {
            global_bucket: Arc::new(RwLock::new(global_bucket)),
            user_buckets: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(config)),
            total_requests: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            allowed_requests: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            rejected_requests: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            last_adjustment: Arc::new(RwLock::new(Instant::now())),
            adjustment_interval: Duration::from_secs(60),
        }
    }

    /// Check if a request should be allowed (global limit)
    pub async fn check_global(&self) -> bool {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let mut bucket = self.global_bucket.write().await;
        if bucket.try_consume(1.0) {
            self.allowed_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            debug!("Rate limit: Global request allowed");
            true
        } else {
            self.rejected_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            warn!("Rate limit: Global request rejected");
            false
        }
    }

    /// Check if a request should be allowed for a specific user
    pub async fn check_user(&self, user_id: &str) -> bool {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Check global limit first
        {
            let mut global_bucket = self.global_bucket.write().await;
            if !global_bucket.try_consume(1.0) {
                self.rejected_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                warn!("Rate limit: User {} rejected (global limit)", user_id);
                return false;
            }
        }

        // Check per-user limit if configured
        let config = self.config.read().await;
        if let Some(per_user_limit) = config.per_user_limit {
            let mut buckets = self.user_buckets.write().await;
            let bucket = buckets.entry(user_id.to_string()).or_insert_with(|| {
                TokenBucket::new(per_user_limit * 2, per_user_limit)
            });

            if bucket.try_consume(1.0) {
                self.allowed_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                debug!("Rate limit: User {} request allowed", user_id);
                true
            } else {
                self.rejected_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                warn!("Rate limit: User {} rejected (user limit)", user_id);
                false
            }
        } else {
            // No per-user limit, already checked global
            self.allowed_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            true
        }
    }

    /// Auto-adjust rate limits based on system load
    pub async fn auto_adjust(&self, system_load: f64) {
        let last_adj = *self.last_adjustment.read().await;
        if last_adj.elapsed() < self.adjustment_interval {
            return;
        }

        let mut config = self.config.write().await;
        let old_rate = config.requests_per_second;

        // Adjust based on system load (0.0 to 1.0)
        let new_rate = if system_load > 0.9 {
            // System overloaded, reduce rate significantly
            (old_rate as f64 * 0.7) as u32
        } else if system_load > 0.7 {
            // System under pressure, reduce rate moderately
            (old_rate as f64 * 0.85) as u32
        } else if system_load < 0.3 {
            // System underutilized, increase rate
            (old_rate as f64 * 1.2) as u32
        } else {
            // System fine, keep current rate
            old_rate
        };

        if new_rate != old_rate {
            config.requests_per_second = new_rate.clamp(10, 1000);
            config.burst_size = (config.requests_per_second as f64 * 1.5) as u32;

            // Update global bucket
            let mut bucket = self.global_bucket.write().await;
            bucket.refill_rate = config.requests_per_second as f64;
            bucket.capacity = config.burst_size as f64;

            info!("🎛️  Rate limiter auto-adjusted: {} → {} req/s (load: {:.1}%)",
                  old_rate, config.requests_per_second, system_load * 100.0);
        }

        *self.last_adjustment.write().await = Instant::now();
    }

    /// Get current rate limit configuration
    pub async fn get_config(&self) -> RateLimitConfig {
        self.config.read().await.clone()
    }

    /// Get rate limiting metrics
    pub fn metrics(&self) -> RateLimitMetrics {
        let total = self.total_requests.load(std::sync::atomic::Ordering::Relaxed);
        let allowed = self.allowed_requests.load(std::sync::atomic::Ordering::Relaxed);
        let rejected = self.rejected_requests.load(std::sync::atomic::Ordering::Relaxed);

        let rejection_rate = if total > 0 {
            rejected as f64 / total as f64
        } else {
            0.0
        };

        RateLimitMetrics {
            total_requests: total,
            allowed_requests: allowed,
            rejected_requests: rejected,
            rejection_rate,
        }
    }

    /// Reset rate limiter (for testing)
    pub async fn reset(&self) {
        let config = self.config.read().await;
        let mut global_bucket = self.global_bucket.write().await;
        *global_bucket = TokenBucket::new(config.burst_size, config.requests_per_second);

        let mut user_buckets = self.user_buckets.write().await;
        user_buckets.clear();

        info!("Rate limiter reset");
    }

    /// Start background auto-adjustment based on metrics
    pub fn start_auto_adjustment(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;

                // Calculate system load based on rejection rate
                let metrics = self.metrics();
                let load = if metrics.total_requests > 100 {
                    // Use rejection rate as proxy for load
                    metrics.rejection_rate.clamp(0.0, 1.0)
                } else {
                    0.5 // Default moderate load if not enough data
                };

                self.auto_adjust(load).await;
            }
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitMetrics {
    pub total_requests: u64,
    pub allowed_requests: u64,
    pub rejected_requests: u64,
    pub rejection_rate: f64,
}
