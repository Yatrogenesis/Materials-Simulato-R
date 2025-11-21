//! LLM error types

use thiserror::Error;

/// Result type alias for LLM operations
pub type Result<T> = std::result::Result<T, Error>;

/// LLM error types
#[derive(Error, Debug)]
pub enum Error {
    /// Provider error
    #[error("Provider error ({provider}): {message}")]
    Provider { provider: String, message: String },

    /// API error
    #[error("API error: {0}")]
    Api(String),

    /// Timeout error
    #[error("Timeout after {0}s")]
    Timeout(u64),

    /// Rate limit error
    #[error("Rate limit exceeded")]
    RateLimit,

    /// Circuit breaker open
    #[error("Circuit breaker open for provider: {0}")]
    CircuitBreakerOpen(String),

    /// All providers failed
    #[error("All providers failed")]
    AllProvidersFailed,

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// HTTP error
    #[error("HTTP error: {0}")]
    Http(String),

    /// Core error
    #[error("Core error: {0}")]
    Core(#[from] materials_core::Error),

    /// Other error
    #[error("{0}")]
    Other(String),
}

impl Error {
    pub fn provider(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Provider {
            provider: provider.into(),
            message: message.into(),
        }
    }
}
