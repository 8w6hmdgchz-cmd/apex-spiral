use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub id: String,
    pub model_id: String,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub top_p: Option<f32>,
    pub stream: bool,
    pub metadata: RequestMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestMetadata {
    pub task_id: Option<String>,
    pub session_id: Option<String>,
    pub source: Option<String>,
    pub priority: u32,
    pub timeout_secs: Option<u64>,
    pub retry_count: u32,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub id: String,
    pub model_id: String,
    pub content: String,
    pub reasoning: Option<String>,
    pub finish_reason: FinishReason,
    pub usage: TokenUsage,
    pub latency_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ModelOverload,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
