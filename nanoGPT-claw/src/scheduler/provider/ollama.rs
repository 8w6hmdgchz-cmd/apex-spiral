//! Ollama Local Provider Implementation

use super::{ChatMessage, LLMError, LLMProvider, LLMResponse};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct OllamaProvider {
    base_url: String,
    model: String,
    client: Client,
}

impl OllamaProvider {
    pub fn new(base_url: &str) -> Self {
        Self::with_model(base_url, "llama3".to_string())
    }

    pub fn with_model(base_url: &str, model: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            base_url: base_url.to_string().trim_end_matches('/').to_string(),
            model,
            client,
        }
    }

    pub fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaMessageRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f64,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    model: Option<String>,
    done: bool,
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    fn provider_name(&self) -> &str {
        "ollama"
    }

    fn default_model(&self) -> &str {
        &self.model
    }

    async fn complete(&self, prompt: &str) -> Result<LLMResponse, LLMError> {
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaOptions { temperature: 0.7 },
        };

        let url = format!("{}/api/generate", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LLMError::RequestError(e.to_string()))?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(LLMError::RequestError(format!(
                "Status {}: {}",
                status, body
            )));
        }

        let ollama_resp: OllamaResponse = response
            .json()
            .await
            .map_err(|e| LLMError::ParseError(e.to_string()))?;

        Ok(LLMResponse {
            content: ollama_resp.response,
            model: ollama_resp.model.unwrap_or(self.model.clone()),
            provider: "ollama".to_string(),
            usage: None,
            finish_reason: if ollama_resp.done {
                Some("stop".to_string())
            } else {
                None
            },
        })
    }

    async fn complete_with_messages(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<LLMResponse, LLMError> {
        let ollama_messages: Vec<OllamaMessage> = messages
            .into_iter()
            .map(|m| OllamaMessage {
                role: m.role,
                content: m.content,
            })
            .collect();

        let request = OllamaMessageRequest {
            model: self.model.clone(),
            messages: ollama_messages,
            stream: false,
        };

        let url = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LLMError::RequestError(e.to_string()))?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(LLMError::RequestError(format!(
                "Status {}: {}",
                status, body
            )));
        }

        #[derive(Deserialize)]
        struct OllamaChatResponse {
            message: OllamaMessage,
            model: Option<String>,
        }

        let chat_resp: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| LLMError::ParseError(e.to_string()))?;

        Ok(LLMResponse {
            content: chat_resp.message.content,
            model: chat_resp.model.unwrap_or(self.model.clone()),
            provider: "ollama".to_string(),
            usage: None,
            finish_reason: Some("stop".to_string()),
        })
    }
}
