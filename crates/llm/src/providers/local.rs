//! Local model provider implementation

use crate::{Completion, CompletionParams, Error, LLMProvider, Result};
use async_trait::async_trait;

pub struct LocalProvider {
    model_name: String,
    model_path: std::path::PathBuf,
}

impl LocalProvider {
    pub fn new(model_name: impl Into<String>, model_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            model_name: model_name.into(),
            model_path: model_path.into(),
        }
    }
}

#[async_trait]
impl LLMProvider for LocalProvider {
    async fn complete(&self, _prompt: &str, _params: CompletionParams) -> Result<Completion> {
        // TODO: Implement with llm crate or candle
        Err(Error::Other("Not yet implemented".to_string()))
    }

    fn cost_per_token(&self) -> f64 {
        0.0  // Local models are free
    }

    fn max_tokens(&self) -> usize {
        2048  // Typical for small models
    }

    fn supports_streaming(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        &self.model_name
    }
}
