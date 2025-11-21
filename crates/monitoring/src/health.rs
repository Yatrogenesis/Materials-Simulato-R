//! Health check endpoints

use serde::{Deserialize, Serialize};

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
