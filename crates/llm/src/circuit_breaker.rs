//! Circuit breaker implementation for fault tolerance

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Blocking requests
    HalfOpen,  // Testing if recovered
}

pub struct CircuitBreaker {
    state: Arc<parking_lot::RwLock<CircuitState>>,
    failure_count: Arc<AtomicU32>,
    failure_threshold: u32,
    success_threshold: u32,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, success_threshold: u32) -> Self {
        Self {
            state: Arc::new(parking_lot::RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicU32::new(0)),
            failure_threshold,
            success_threshold,
        }
    }

    pub fn is_open(&self) -> bool {
        *self.state.read() == CircuitState::Open
    }

    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        let mut state = self.state.write();
        *state = CircuitState::Closed;
    }

    pub fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        if failures >= self.failure_threshold {
            let mut state = self.state.write();
            *state = CircuitState::Open;
        }
    }
}
