//! OpenAI Provider Implementation

use super::{ChatMessage, LLMError, LLMProvider, LLMResponse, TokenUsage};
use crate::scheduler::retry::{retry_with_backoff, RetryConfig};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

pub struct OpenAIProvider {
    api_key: String,
    model: String,
    base_url: String,
    client: Client,
    retry_config: RetryConfig,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self::with_model(api_key, "gpt-4o".to_string())
    }

    pub fn with_model(api_key: String, model: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            api_key,
            model,
            base_url: "https://api.openai.com/v1".to_string(),
            client,
            retry_config: RetryConfig::default(),
        }
    }

    pub fn with_custom_url(api_key: String, base_url: &str, model: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            api_key,
            model: model.to_string(),
            base_url: base_url.to_string(),
            client,
            retry_config: RetryConfig::default(),
        }
    }

    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f64,
    max_tokens: Option<u32>,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
    model: String,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageResponse,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAIMessageResponse {
    content: String,
}

#[derive(Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn provider_name(&self) -> &str {
        "openai"
    }

    fn default_model(&self) -> &str {
        &self.model
    }

    async fn complete(&self, prompt: &str) -> Result<LLMResponse, LLMError> {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];
        self.complete_with_messages(messages).await
    }

    async fn complete_with_messages(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<LLMResponse, LLMError> {
        let retry_config = self.retry_config.clone();

        let openai_messages: Vec<OpenAIMessage> = messages
            .iter()
            .map(|m| OpenAIMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: openai_messages,
            temperature: 0.7,
            max_tokens: Some(4096),
        };

        let url = format!("{}/chat/completions", self.base_url);
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        retry_with_backoff(
            retry_config,
            || async { Self::execute_request(&client, &url, &api_key, &request).await },
            |e: &LLMError| e.is_retryable(),
        )
        .await
    }
}

impl OpenAIProvider {
    async fn execute_request(
        client: &Client,
        url: &str,
        api_key: &str,
        request: &OpenAIRequest,
    ) -> Result<LLMResponse, LLMError> {
        debug!("Sending request to OpenAI API at {}", url);

        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LLMError::Timeout(120)
                } else if e.is_connect() {
                    LLMError::NetworkError(e.to_string())
                } else {
                    LLMError::RequestError(e.to_string())
                }
            })?;

        let status = response.status();

        debug!("Received response with status: {}", status);

        if status == 401 {
            return Err(LLMError::InvalidApiKey);
        }

        if status == 429 {
            return Err(LLMError::RateLimit);
        }

        if status.is_server_error() {
            return Err(LLMError::ServerError(status.to_string()));
        }

        if status.as_u16() == 400 {
            let body = response.text().await.unwrap_or_default();
            if body.contains("context_length_exceeded") {
                return Err(LLMError::ContextOverflow(body));
            } else if body.contains("content_policy") {
                return Err(LLMError::ContentPolicyBlocked(body));
            }
            return Err(LLMError::RequestError(format!("Status 400: {}", body)));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(LLMError::RequestError(format!(
                "Status {}: {}",
                status, body
            )));
        }

        let openai_resp: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| LLMError::ParseError(e.to_string()))?;

        let content = openai_resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let usage = openai_resp.usage.map(|u| TokenUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        let finish_reason = openai_resp
            .choices
            .first()
            .and_then(|c| c.finish_reason.clone());

        Ok(LLMResponse {
            content,
            model: openai_resp.model,
            provider: "openai".to_string(),
            usage,
            finish_reason,
        })
    }
}
