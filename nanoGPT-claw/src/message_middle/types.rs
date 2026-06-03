use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMessage {
    pub id: String,
    pub source: MessageSource,
    pub message_type: MessageType,
    pub sender: Sender,
    pub content: String,
    pub raw_content: Option<serde_json::Value>,
    pub session_id: Option<String>,
    pub thread_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: MessageMetadata,
    pub context: Option<MessageContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageSource {
    Cli,
    Lark,
    Github,
    System,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Command,
    Query,
    Event,
    Callback,
    Webhook,
    Notification,
    Error,
    Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sender {
    pub sender_type: SenderType,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub channel_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SenderType {
    User,
    Bot,
    System,
    Webhook,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageMetadata {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub flags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContext {
    pub conversation_history: Vec<ConversationEntry>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RouteDestination {
    LlmScheduler,
    ThinkEngine,
    EvolveEngine,
    MemoryLayer,
    LarkGateway,
    GithubGateway,
    CliOutput,
    SystemCommand,
    Drop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    pub message: UnifiedMessage,
    pub destination: RouteDestination,
    pub handlers: Vec<String>,
    pub priority: u32,
    pub requires_auth: bool,
    pub rate_limited: bool,
}
