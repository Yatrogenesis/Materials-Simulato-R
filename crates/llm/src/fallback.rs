//! Fallback logic for LLM providers

pub struct FallbackConfig {
    pub max_retries: u32,
    pub timeout_seconds: u64,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            timeout_seconds: 30,
        }
    }
}
