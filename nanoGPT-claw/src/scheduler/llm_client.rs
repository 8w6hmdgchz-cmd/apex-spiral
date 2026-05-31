//! LLM Client - HTTP Client for LLM API Communication
//!
//! Handles HTTP communication with various LLM providers (OpenAI, iAMHC, etc.)
//! with retry logic, timeout handling, and error recovery.

use serde::{Deserialize, Serialize};
use tracing::{warn, error};

/// LLM configuration
#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub name: String,
    pub model: String,
    pub api_url: String,
    pub api_key: String,
    pub max_retries: u32,
    pub timeout_secs: u64,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            model: "gpt-4o".to_string(),
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: String::new(),
            max_retries: 3,
            timeout_secs: 120,
        }
    }
}

/// LLM response structure
#[derive(Debug, Clone, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

/// Chat message structure
#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Chat completion request
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

/// HTTP client for LLM API using ureq
#[derive(Clone)]
pub struct LLMClient {
    config: LLMConfig,
}

impl LLMClient {
    /// Create new LLM client
    pub fn new(config: LLMConfig) -> Self {
        Self { config }
    }

    /// Send completion request with retry
    pub async fn complete(&self, prompt: &str) -> Result<LLMResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut last_error = None;
        let mut backoff = 1u64;

        for attempt in 0..self.config.max_retries {
            match self.send_request(prompt).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retries - 1 {
                        warn!("LLM request failed (attempt {}/{}), retrying in {}s...",
                            attempt + 1, self.config.max_retries, backoff);
                        std::thread::sleep(std::time::Duration::from_secs(backoff));
                        backoff *= 2;
                    }
                }
            }
        }

        error!("LLM request failed after {} attempts: {:?}", self.config.max_retries, last_error);
        Err(last_error.unwrap_or_else(|| "Unknown error".into()))
    }

    /// Internal HTTP request sender
    async fn send_request(&self, prompt: &str) -> Result<LLMResponse, Box<dyn std::error::Error + Send + Sync>> {
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.7,
            max_tokens: Some(2048),
        };

        let json_body = serde_json::to_string(&request)?;

        let client = reqwest::Client::new();
        let response = client
            .post(&self.config.api_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .header("User-Agent", "NanoGPT-Claw/0.1.0")
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .body(json_body)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {}", e))?;

        let status = response.status();
        if status != 200 {
            let body = response.text().await.unwrap_or_default();
            return Err(format!("LLM API error: {} - {}", status, body).into());
        }

        #[derive(Deserialize)]
        struct ApiResponse {
            choices: Vec<ApiChoice>,
            usage: Option<TokenUsage>,
        }

        #[derive(Deserialize)]
        struct ApiChoice {
            message: ApiMessage,
            finish_reason: Option<String>,
        }

        #[derive(Deserialize)]
        struct ApiMessage {
            content: String,
        }

        let api_resp: ApiResponse = response
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;

        let content = api_resp.choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(LLMResponse {
            content,
            model: self.config.model.clone(),
            usage: api_resp.usage,
            finish_reason: api_resp.choices.first().and_then(|c| c.finish_reason.clone()),
        })
    }

    /// Get client info
    pub fn info(&self) -> (&str, &str) {
        (&self.config.name, &self.config.model)
    }
}
