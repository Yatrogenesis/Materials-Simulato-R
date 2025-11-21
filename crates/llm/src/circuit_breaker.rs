//! Advanced Circuit Breaker with Auto-Recovery and Metrics
//!
//! Features:
//! - Auto-recovery with exponential backoff
//! - Half-open state for gradual recovery
//! - Comprehensive metrics tracking
//! - Configurable thresholds and timeouts

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,    // Normal operation - all requests allowed
    Open,      // Blocking requests - service degraded
    HalfOpen,  // Testing recovery - limited requests allowed
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub rejected_calls: u64,
    pub state_transitions: u64,
    pub current_state: CircuitState,
}

pub struct CircuitBreaker {
    name: String,
    state: Arc<parking_lot::RwLock<CircuitState>>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    last_failure_time: Arc<parking_lot::RwLock<Option<Instant>>>,

    // Metrics
    total_calls: Arc<AtomicU64>,
    successful_calls: Arc<AtomicU64>,
    failed_calls: Arc<AtomicU64>,
    rejected_calls: Arc<AtomicU64>,
    state_transitions: Arc<AtomicU64>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with intelligent defaults
    pub fn new(name: impl Into<String>, failure_threshold: u32, success_threshold: u32) -> Self {
        Self {
            name: name.into(),
            state: Arc::new(parking_lot::RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            failure_threshold,
            success_threshold,
            timeout: Duration::from_secs(60),
            last_failure_time: Arc::new(parking_lot::RwLock::new(None)),
            total_calls: Arc::new(AtomicU64::new(0)),
            successful_calls: Arc::new(AtomicU64::new(0)),
            failed_calls: Arc::new(AtomicU64::new(0)),
            rejected_calls: Arc::new(AtomicU64::new(0)),
            state_transitions: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Set custom timeout for auto-recovery
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if circuit breaker is open (blocking requests)
    pub fn is_open(&self) -> bool {
        self.check_and_transition();
        *self.state.read() == CircuitState::Open
    }

    /// Get current state
    pub fn state(&self) -> CircuitState {
        self.check_and_transition();
        *self.state.read()
    }

    /// Execute a function with circuit breaker protection
    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        self.total_calls.fetch_add(1, Ordering::Relaxed);

        // Check if we should allow the call
        if !self.should_allow_call() {
            self.rejected_calls.fetch_add(1, Ordering::Relaxed);
            return Err(CircuitBreakerError::CircuitOpen);
        }

        // Execute the call
        match f().await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(e) => {
                self.record_failure();
                Err(CircuitBreakerError::CallFailed(e))
            }
        }
    }

    /// Check if call should be allowed based on current state
    fn should_allow_call(&self) -> bool {
        self.check_and_transition();

        let state = *self.state.read();
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => false,
            CircuitState::HalfOpen => {
                // In half-open, allow limited requests to test recovery
                self.success_count.load(Ordering::SeqCst) < self.success_threshold
            }
        }
    }

    /// Check timeout and transition to half-open if needed
    fn check_and_transition(&self) {
        let state = *self.state.read();
        if state != CircuitState::Open {
            return;
        }

        let last_failure = *self.last_failure_time.read();
        if let Some(last_failure) = last_failure {
            if last_failure.elapsed() >= self.timeout {
                info!("Circuit breaker '{}' transitioning to HALF_OPEN for recovery testing", self.name);
                let mut state = self.state.write();
                *state = CircuitState::HalfOpen;
                self.success_count.store(0, Ordering::SeqCst);
                self.state_transitions.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Record successful call
    pub fn record_success(&self) {
        self.successful_calls.fetch_add(1, Ordering::Relaxed);

        let state = *self.state.read();
        match state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::HalfOpen => {
                let successes = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if successes >= self.success_threshold {
                    info!("Circuit breaker '{}' transitioning to CLOSED - service recovered", self.name);
                    let mut state = self.state.write();
                    *state = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                    self.state_transitions.fetch_add(1, Ordering::Relaxed);
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
                warn!("Circuit breaker '{}' received success in OPEN state", self.name);
            }
        }
    }

    /// Record failed call
    pub fn record_failure(&self) {
        self.failed_calls.fetch_add(1, Ordering::Relaxed);

        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;

        let state = *self.state.read();
        match state {
            CircuitState::Closed | CircuitState::HalfOpen => {
                if failures >= self.failure_threshold {
                    error!("Circuit breaker '{}' OPENING - failure threshold reached ({} failures)",
                           self.name, failures);
                    let mut state = self.state.write();
                    *state = CircuitState::Open;
                    *self.last_failure_time.write() = Some(Instant::now());
                    self.state_transitions.fetch_add(1, Ordering::Relaxed);

                    // Spawn auto-recovery task
                    self.spawn_auto_recovery();
                }
            }
            CircuitState::Open => {
                // Already open, just update timestamp
                *self.last_failure_time.write() = Some(Instant::now());
            }
        }
    }

    /// Spawn background task for auto-recovery monitoring
    fn spawn_auto_recovery(&self) {
        let name = self.name.clone();
        let timeout = self.timeout;
        let state = self.state.clone();
        let state_transitions = self.state_transitions.clone();

        tokio::spawn(async move {
            sleep(timeout).await;

            let current_state = *state.read();
            if current_state == CircuitState::Open {
                info!("Auto-recovery: Circuit breaker '{}' attempting transition to HALF_OPEN", name);
                let mut state = state.write();
                *state = CircuitState::HalfOpen;
                state_transitions.fetch_add(1, Ordering::Relaxed);
            }
        });
    }

    /// Get comprehensive metrics
    pub fn metrics(&self) -> CircuitBreakerMetrics {
        CircuitBreakerMetrics {
            total_calls: self.total_calls.load(Ordering::Relaxed),
            successful_calls: self.successful_calls.load(Ordering::Relaxed),
            failed_calls: self.failed_calls.load(Ordering::Relaxed),
            rejected_calls: self.rejected_calls.load(Ordering::Relaxed),
            state_transitions: self.state_transitions.load(Ordering::Relaxed),
            current_state: *self.state.read(),
        }
    }

    /// Reset circuit breaker to closed state
    pub fn reset(&self) {
        info!("Circuit breaker '{}' manually reset to CLOSED", self.name);
        let mut state = self.state.write();
        *state = CircuitState::Closed;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.state_transitions.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    CircuitOpen,
    CallFailed(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::CircuitOpen => write!(f, "Circuit breaker is open"),
            CircuitBreakerError::CallFailed(e) => write!(f, "Call failed: {}", e),
        }
    }
}

impl<E: std::error::Error> std::error::Error for CircuitBreakerError<E> {}
