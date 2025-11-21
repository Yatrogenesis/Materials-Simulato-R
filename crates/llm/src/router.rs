//! LLM router with intelligent fallback

use crate::{Completion, CompletionParams, Error, LLMProvider, Result};
use std::collections::HashMap;
use std::sync::Arc;

pub struct LLMRouter {
    providers: HashMap<String, Arc<dyn LLMProvider>>,
    fallback_chain: Vec<String>,
}

impl LLMRouter {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            fallback_chain: Vec::new(),
        }
    }

    pub fn add_provider(&mut self, name: impl Into<String>, provider: Arc<dyn LLMProvider>) {
        self.providers.insert(name.into(), provider);
    }

    pub fn set_fallback_chain(&mut self, chain: Vec<String>) {
        self.fallback_chain = chain;
    }

    pub async fn complete(&self, prompt: &str, params: CompletionParams) -> Result<Completion> {
        for provider_name in &self.fallback_chain {
            if let Some(provider) = self.providers.get(provider_name) {
                match provider.complete(prompt, params.clone()).await {
                    Ok(completion) => return Ok(completion),
                    Err(e) => {
                        tracing::warn!(
                            provider = provider_name,
                            error = ?e,
                            "Provider failed, trying fallback"
                        );
                        continue;
                    }
                }
            }
        }

        Err(Error::AllProvidersFailed)
    }
}

impl Default for LLMRouter {
    fn default() -> Self {
        Self::new()
    }
}
