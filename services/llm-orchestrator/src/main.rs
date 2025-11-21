//! LLM Orchestrator Service
//!
//! Smart LLM routing and fallback service

use materials_llm::{
    providers::openai::OpenAIProvider,
    router::LLMRouter,
    CompletionParams, LLMProvider,
};
use materials_monitoring;
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Deserialize)]
struct CompletionRequest {
    prompt: String,
    #[serde(default)]
    params: CompletionParams,
}

#[derive(Debug, Serialize)]
struct CompletionResponse {
    text: String,
    tokens_used: usize,
    model: String,
    finish_reason: String,
    provider: String,
}

async fn complete_handler(
    Extension(router): Extension<Arc<LLMRouter>>,
    Json(req): Json<CompletionRequest>,
) -> Result<Json<CompletionResponse>, StatusCode> {
    let completion = router
        .complete(&req.prompt, req.params)
        .await
        .map_err(|e| {
            eprintln!("LLM completion error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(CompletionResponse {
        text: completion.text,
        tokens_used: completion.tokens_used,
        model: completion.model.clone(),
        finish_reason: completion.finish_reason,
        provider: completion.model,
    }))
}

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize monitoring
    materials_monitoring::init()?;

    info!("Starting Materials-Simulato-R LLM Orchestrator v{}", env!("CARGO_PKG_VERSION"));

    // Get API keys from environment
    let openai_key = std::env::var("OPENAI_API_KEY")
        .unwrap_or_else(|_| {
            tracing::warn!("OPENAI_API_KEY not set, using placeholder");
            "placeholder".to_string()
        });

    // Initialize LLM providers
    let openai_gpt4 = Arc::new(OpenAIProvider::gpt4_turbo(&openai_key)) as Arc<dyn LLMProvider>;
    let openai_gpt35 = Arc::new(OpenAIProvider::gpt35_turbo(&openai_key)) as Arc<dyn LLMProvider>;

    // Create LLM router with fallback chain
    let mut router = LLMRouter::new();
    router.add_provider("gpt-4-turbo", openai_gpt4);
    router.add_provider("gpt-3.5-turbo", openai_gpt35);

    let router = Arc::new(router);

    info!("LLM Router initialized with {} providers", 2);

    // Create API server
    let app = Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/v1/completions", post(complete_handler))
        .layer(Extension(router));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    info!("Listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
