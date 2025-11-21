//! OpenAI provider implementation

use crate::{Completion, CompletionParams, Error, LLMProvider, Result};
use async_trait::async_trait;

pub struct OpenAIProvider {
    api_key: String,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
        }
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, _prompt: &str, _params: CompletionParams) -> Result<Completion> {
        // TODO: Implement with async-openai
        Err(Error::Other("Not yet implemented".to_string()))
    }

    fn cost_per_token(&self) -> f64 {
        match self.model.as_str() {
            "gpt-4-turbo" => 0.00001,  // $10 per 1M tokens
            "gpt-3.5-turbo" => 0.0000005,  // $0.50 per 1M tokens
            _ => 0.00001,
        }
    }

    fn max_tokens(&self) -> usize {
        match self.model.as_str() {
            "gpt-4-turbo" => 128000,
            "gpt-3.5-turbo" => 16385,
            _ => 4096,
        }
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        &self.model
    }
}
