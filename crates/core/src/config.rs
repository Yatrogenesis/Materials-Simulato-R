//! Configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::Result;

/// Main configuration struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Environment (development, staging, production)
    pub environment: String,

    /// Log level
    pub log_level: String,

    /// Database configuration
    pub database: DatabaseConfig,

    /// LLM configuration
    pub llm: LlmConfig,

    /// API configuration
    pub api: ApiConfig,

    /// Compute configuration
    pub compute: ComputeConfig,

    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub postgres: PostgresConfig,
    pub mongodb: MongoConfig,
    pub neo4j: Neo4jConfig,
    pub redis: RedisConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    pub url: String,
    pub pool_size: u32,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoConfig {
    pub url: String,
    pub database: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub max_connections: u32,
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub routing: LlmRoutingConfig,
    pub providers: LlmProvidersConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRoutingConfig {
    pub strategy: String,
    pub primary_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProvidersConfig {
    pub openai: Option<OpenAiConfig>,
    pub anthropic: Option<AnthropicConfig>,
    pub local: Option<LocalLlmConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub enabled: bool,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub enabled: bool,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalLlmConfig {
    pub enabled: bool,
    pub models_path: PathBuf,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
}

/// Compute configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeConfig {
    pub workers: usize,
    pub gpu_enabled: bool,
    pub ml_backend: String,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
}

impl Config {
    /// Load configuration from file
    pub fn from_file(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let content = std::fs::read_to_string(&path)?;

        let config = if path.extension().and_then(|s| s.to_str()) == Some("yaml")
            || path.extension().and_then(|s| s.to_str()) == Some("yml")
        {
            serde_yaml::from_str(&content)
                .map_err(|e| crate::error::Error::config(e.to_string()))?
        } else if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::from_str(&content)
                .map_err(|e| crate::error::Error::config(e.to_string()))?
        } else {
            serde_json::from_str(&content)?
        };

        Ok(config)
    }

    /// Load configuration from environment-specific file
    pub fn from_env(env: &str) -> Result<Self> {
        let path = format!("config/{}.yml", env);
        Self::from_file(path)
    }
}
