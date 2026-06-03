mod types;

use anyhow::{Context, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

pub use types::*;

pub static LLM_SCHEDULER: once_cell::sync::Lazy<Arc<LlmScheduler>> = 
    once_cell::sync::Lazy::new(|| Arc::new(LlmScheduler::new()));

pub async fn schedule_task(request: LlmRequest) -> Result<LlmResponse> {
    LLM_SCHEDULER.schedule(request).await
}

pub struct LlmScheduler {
    stats: RwLock<SchedulerStats>,
    http_client: reqwest::Client,
    model_configs: RwLock<HashMap<String, ModelEndpoint>>,
}

#[derive(Clone)]
pub struct ModelEndpoint {
    pub base_url: String,
    pub api_key: String,
    pub model_name: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
}

#[derive(Debug, Clone, Default)]
pub struct SchedulerStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_latency_ms: f64,
    pub model_usage: HashMap<String, ModelUsageStats>,
}

#[derive(Debug, Clone, Default)]
pub struct ModelUsageStats {
    pub request_count: u64,
    pub total_tokens: u64,
    pub avg_latency_ms: f64,
    pub error_count: u64,
}

impl LlmScheduler {
    pub fn new() -> Self {
        Self {
            stats: RwLock::new(SchedulerStats::default()),
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client"),
            model_configs: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_model(&self, model_id: String, endpoint: ModelEndpoint) {
        info!("Registering model: {} -> {}", model_id, endpoint.base_url);
        self.model_configs.write().insert(model_id, endpoint);
    }

    pub async fn schedule(&self, request: LlmRequest) -> Result<LlmResponse> {
        let start_time = Instant::now();
        debug!("Scheduling request to model: {}", request.model_id);
        
        let config = self.model_configs.read()
            .get(&request.model_id)
            .cloned();

        let response = match config {
            Some(endpoint) => {
                self.call_model(&request, &endpoint).await
            }
            None => {
                warn!("Model {} not found, using mock response", request.model_id);
                Ok(self.mock_response(&request))
            }
        };

        let elapsed = start_time.elapsed().as_millis() as u64;
        
        {
            let mut stats = self.stats.write();
            stats.total_requests += 1;
            
            if response.is_ok() {
                stats.successful_requests += 1;
            } else {
                stats.failed_requests += 1;
            }
            
            let usage = response.as_ref().map(|r| r.usage.total_tokens).unwrap_or(0);
            let model_stats = stats.model_usage.entry(request.model_id.clone()).or_default();
            model_stats.request_count += 1;
            model_stats.total_tokens += usage as u64;
            model_stats.avg_latency_ms = 
                (model_stats.avg_latency_ms * (model_stats.request_count - 1) as f64 + elapsed as f64) 
                / model_stats.request_count as f64;
        }

        response
    }

    async fn call_model(&self, request: &LlmRequest, endpoint: &ModelEndpoint) -> Result<LlmResponse> {
        let url = format!("{}/chat/completions", endpoint.base_url.trim_end_matches('/'));
        
        let messages: Vec<serde_json::Value> = {
            let mut msgs = Vec::new();
            if let Some(ref sys) = request.system_prompt {
                msgs.push(serde_json::json!({
                    "role": "system",
                    "content": sys
                }));
            }
            for msg in &request.messages {
                msgs.push(serde_json::json!({
                    "role": msg.role,
                    "content": msg.content,
                    "name": msg.name
                }));
            }
            if request.prompt.is_empty() && msgs.is_empty() {
                msgs.push(serde_json::json!({
                    "role": "user", 
                    "content": request.prompt
                }));
            } else if !request.prompt.is_empty() && msgs.is_empty() {
                msgs.push(serde_json::json!({
                    "role": "user",
                    "content": request.prompt
                }));
            }
            msgs
        };

        let body = serde_json::json!({
            "model": endpoint.model_name,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(endpoint.max_tokens),
            "temperature": request.temperature.unwrap_or(endpoint.temperature),
            "top_p": request.top_p.unwrap_or(endpoint.top_p),
            "stream": request.stream,
        });

        let start = Instant::now();
        
        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", endpoint.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send request to LLM")?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("LLM request failed: {} - {}", status, error_text);
            return Err(anyhow::anyhow!("LLM request failed: {} - {}", status, error_text));
        }

        let llm_response: serde_json::Value = response.json().await
            .context("Failed to parse LLM response")?;

        let content = llm_response["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let prompt_tokens = llm_response["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let completion_tokens = llm_response["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;
        let total_tokens = llm_response["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(LlmResponse {
            id: llm_response["id"].as_str().unwrap_or(&request.id).to_string(),
            model_id: request.model_id.clone(),
            content,
            reasoning: None,
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
            },
            latency_ms,
            timestamp: chrono::Utc::now(),
            error: None,
        })
    }

    fn mock_response(&self, request: &LlmRequest) -> LlmResponse {
        LlmResponse {
            id: uuid::Uuid::new_v4().to_string(),
            model_id: request.model_id.clone(),
            content: format!("[Mock] Processed: {}", request.prompt.chars().take(100).collect::<String>()),
            reasoning: None,
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
            latency_ms: 100,
            timestamp: chrono::Utc::now(),
            error: None,
        }
    }

    pub fn get_stats(&self) -> SchedulerStats {
        self.stats.read().clone()
    }
}

impl LlmRequest {
    pub fn new(prompt: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            model_id: "core-1".to_string(),
            prompt,
            system_prompt: None,
            messages: Vec::new(),
            temperature: None,
            max_tokens: None,
            top_p: None,
            stream: false,
            metadata: RequestMetadata::default(),
        }
    }

    pub fn for_model(mut self, model_id: String) -> Self {
        self.model_id = model_id;
        self
    }

    pub fn with_system_prompt(mut self, system_prompt: String) -> Self {
        self.system_prompt = Some(system_prompt);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_messages(mut self, messages: Vec<ChatMessage>) -> Self {
        self.messages = messages;
        self
    }
}

impl LlmResponse {
    pub fn error(message: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            model_id: "unknown".to_string(),
            content: String::new(),
            reasoning: None,
            finish_reason: FinishReason::Unknown,
            usage: TokenUsage::default(),
            latency_ms: 0,
            timestamp: chrono::Utc::now(),
            error: Some(message),
        }
    }
}
