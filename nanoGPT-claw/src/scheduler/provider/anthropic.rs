//! Anthropic Claude Provider Implementation

use super::{ChatMessage, LLMError, LLMProvider, LLMResponse, TokenUsage};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    base_url: String,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self::with_model(api_key, "claude-3-opus-20240229".to_string())
    }

    pub fn with_model(api_key: String, model: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(180))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            api_key,
            model,
            base_url: "https://api.anthropic.com/v1".to_string(),
            client,
        }
    }
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    temperature: f64,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    model: String,
    usage: AnthropicUsage,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    #[serde(rename = "input_tokens")]
    input_tokens: u32,
    #[serde(rename = "output_tokens")]
    output_tokens: u32,
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    fn provider_name(&self) -> &str {
        "anthropic"
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
        let anthropic_messages: Vec<AnthropicMessage> = messages
            .into_iter()
            .map(|m| AnthropicMessage {
                role: if m.role == "assistant" {
                    "assistant"
                } else {
                    "user"
                }
                .to_string(),
                content: m.content,
            })
            .collect();

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: anthropic_messages,
            max_tokens: 4096,
            temperature: 0.7,
        };

        let url = format!("{}/messages", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LLMError::RequestError(e.to_string()))?;

        let status = response.status();

        if status == 401 {
            return Err(LLMError::InvalidApiKey);
        }

        if status == 429 {
            return Err(LLMError::RateLimit);
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(LLMError::RequestError(format!(
                "Status {}: {}",
                status, body
            )));
        }

        let anthropic_resp: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| LLMError::ParseError(e.to_string()))?;

        let content = anthropic_resp
            .content
            .iter()
            .filter_map(|c| c.text.clone())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(LLMResponse {
            content,
            model: anthropic_resp.model,
            provider: "anthropic".to_string(),
            usage: Some(TokenUsage {
                prompt_tokens: anthropic_resp.usage.input_tokens,
                completion_tokens: anthropic_resp.usage.output_tokens,
                total_tokens: anthropic_resp.usage.input_tokens
                    + anthropic_resp.usage.output_tokens,
            }),
            finish_reason: Some("stop".to_string()),
        })
    }
}
