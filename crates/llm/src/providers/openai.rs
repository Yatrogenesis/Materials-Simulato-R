//! OpenAI provider implementation with async-openai

use crate::{Completion, CompletionParams, Error, LLMProvider, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{CreateChatCompletionRequestArgs, ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs}
};
use async_trait::async_trait;

pub struct OpenAIProvider {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key.into());
        let client = Client::with_config(config);
        Self {
            client,
            model: model.into(),
        }
    }

    pub fn gpt4_turbo(api_key: impl Into<String>) -> Self {
        Self::new(api_key, "gpt-4-turbo-preview")
    }

    pub fn gpt35_turbo(api_key: impl Into<String>) -> Self {
        Self::new(api_key, "gpt-3.5-turbo")
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, prompt: &str, params: CompletionParams) -> Result<Completion> {
        let message = ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()
            .map_err(|e| Error::provider("openai", e.to_string()))?;

        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder
            .model(&self.model)
            .max_tokens(params.max_tokens as u16)
            .temperature(params.temperature as f32)
            .messages(vec![ChatCompletionRequestMessage::User(message)]);

        if let Some(top_p) = params.top_p {
            request_builder.top_p(top_p as f32);
        }

        if !params.stop_sequences.is_empty() {
            request_builder.stop(params.stop_sequences.clone());
        }

        let request = request_builder
            .build()
            .map_err(|e| Error::provider("openai", e.to_string()))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| Error::Api(e.to_string()))?;

        let choice = response
            .choices
            .first()
            .ok_or_else(|| Error::provider("openai", "No completion returned"))?;

        let text = choice
            .message
            .content
            .clone()
            .unwrap_or_default();

        let finish_reason = choice
            .finish_reason
            .as_ref()
            .map(|r| format!("{:?}", r))
            .unwrap_or_else(|| "unknown".to_string());

        let tokens_used = response
            .usage
            .as_ref()
            .map(|u| u.total_tokens as usize)
            .unwrap_or(0);

        Ok(Completion {
            text,
            tokens_used,
            model: self.model.clone(),
            finish_reason,
        })
    }

    fn cost_per_token(&self) -> f64 {
        match self.model.as_str() {
            "gpt-4-turbo" | "gpt-4-turbo-preview" => 0.00001,  // $10 per 1M tokens
            "gpt-3.5-turbo" => 0.0000005,  // $0.50 per 1M tokens
            _ => 0.00001,
        }
    }

    fn max_tokens(&self) -> usize {
        match self.model.as_str() {
            "gpt-4-turbo" | "gpt-4-turbo-preview" => 128000,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let provider = OpenAIProvider::gpt35_turbo("fake-api-key");
        assert_eq!(provider.name(), "gpt-3.5-turbo");
        assert_eq!(provider.max_tokens(), 16385);
    }

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_openai_completion() {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap();
        let provider = OpenAIProvider::gpt35_turbo(api_key);

        let result = provider.complete(
            "Say hello in one word",
            CompletionParams::default(),
        ).await;

        assert!(result.is_ok());
    }
}
