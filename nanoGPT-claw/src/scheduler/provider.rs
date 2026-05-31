//! LLM Provider Module - Multi-Provider Support
//!
//! Provides unified interface for different LLM providers:
//! - OpenAI (GPT-4, GPT-4o)
//! - Anthropic (Claude 3)
//! - Ollama (Local models)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub model: String,
    pub provider: String,
    pub usage: Option<TokenUsage>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f64,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FailoverStrategy {
    Retry,
    RotateKey,
    CompressContext,
    FallbackModel,
    Abort,
}

#[derive(Error, Debug)]
pub enum LLMError {
    #[error("API request failed: {0}")]
    RequestError(String),
    #[error("API response parsing failed: {0}")]
    ParseError(String),
    #[error("Authentication failed: {0}")]
    AuthError(String),
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Timeout after {0} seconds")]
    Timeout(u64),
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("Provider not configured: {0}")]
    NotConfigured(String),
    #[error("Context overflow: {0}")]
    ContextOverflow(String),
    #[error("Content policy blocked: {0}")]
    ContentPolicyBlocked(String),
    #[error("Server error: {0}")]
    ServerError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

impl LLMError {
    pub fn failover_strategy(&self) -> FailoverStrategy {
        match self {
            LLMError::RequestError(_) => FailoverStrategy::Retry,
            LLMError::ParseError(_) => FailoverStrategy::Abort,
            LLMError::AuthError(_) => FailoverStrategy::RotateKey,
            LLMError::RateLimit => FailoverStrategy::Retry,
            LLMError::ModelNotFound(_) => FailoverStrategy::FallbackModel,
            LLMError::Timeout(_) => FailoverStrategy::Retry,
            LLMError::InvalidApiKey => FailoverStrategy::RotateKey,
            LLMError::NotConfigured(_) => FailoverStrategy::Abort,
            LLMError::ContextOverflow(_) => FailoverStrategy::CompressContext,
            LLMError::ContentPolicyBlocked(_) => FailoverStrategy::Abort,
            LLMError::ServerError(_) => FailoverStrategy::Retry,
            LLMError::NetworkError(_) => FailoverStrategy::Retry,
        }
    }
    
    pub fn is_retryable(&self) -> bool {
        matches!(self.failover_strategy(), FailoverStrategy::Retry)
    }
}

pub type LLMResult<T> = Result<T, LLMError>;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn default_model(&self) -> &str;
    async fn complete(&self, prompt: &str) -> LLMResult<LLMResponse>;
    async fn complete_with_messages(&self, messages: Vec<ChatMessage>) -> LLMResult<LLMResponse>;
}

// pub mod openai; // not downloaded
// pub mod anthropic; // not downloaded
// pub mod ollama; // not downloaded

// pub use openai::OpenAIProvider;
// pub use anthropic::AnthropicProvider;
// pub use ollama::OllamaProvider;

use std::collections::HashMap;

pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn LLMProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, provider: Arc<dyn LLMProvider>) {
        self.providers.insert(name.to_string(), provider);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn LLMProvider>> {
        self.providers.get(name).cloned()
    }

    pub fn names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub fn create_from_env() -> Self {
        // Note: OpenAI, Anthropic, Ollama providers require separate module files
        // This is a placeholder that logs a warning
        let registry = Self::new();
        tracing::warn!("LLM providers not fully configured - provider modules not downloaded");
        registry
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
