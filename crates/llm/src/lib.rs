//! Materials-Simulato-R LLM Integration Layer
//!
//! Multi-LLM orchestration with intelligent fallback:
//! - OpenAI (GPT-4, GPT-3.5)
//! - Anthropic (Claude-3.5, Claude-3)
//! - Google (Gemini)
//! - Mistral (API + local)
//! - Local models (Phi2, TinyLlama)

#![allow(dead_code, unused_imports)]

pub mod providers;
pub mod router;
pub mod fallback;
pub mod circuit_breaker;
pub mod error;

pub use error::{Error, Result};
pub use router::LLMRouter;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Completion request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionParams {
    pub max_tokens: usize,
    pub temperature: f64,
    pub top_p: Option<f64>,
    pub stop_sequences: Vec<String>,
}

impl Default for CompletionParams {
    fn default() -> Self {
        Self {
            max_tokens: 1000,
            temperature: 0.7,
            top_p: Some(0.95),
            stop_sequences: Vec::new(),
        }
    }
}

/// Completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completion {
    pub text: String,
    pub tokens_used: usize,
    pub model: String,
    pub finish_reason: String,
}

/// LLM Provider trait
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Complete a prompt
    async fn complete(&self, prompt: &str, params: CompletionParams) -> Result<Completion>;

    /// Get cost per token (in USD)
    fn cost_per_token(&self) -> f64;

    /// Get maximum tokens supported
    fn max_tokens(&self) -> usize;

    /// Check if streaming is supported
    fn supports_streaming(&self) -> bool;

    /// Get provider name
    fn name(&self) -> &str;
}

/// Version of the LLM layer
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_completion_params_default() {
        let params = CompletionParams::default();
        assert_eq!(params.max_tokens, 1000);
        assert_eq!(params.temperature, 0.7);
    }
}
