//! LLM provider implementations

pub mod openai;
pub mod local;

pub use openai::OpenAIProvider;
pub use local::LocalProvider;
