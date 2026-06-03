use anyhow::Result;
use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct LarkEventHandler {
    auto_reply: bool,
}

#[derive(Debug, Deserialize)]
pub struct ReceivedMessage {
    pub sender: Option<SenderInfo>,
    pub message: Option<MessageInfo>,
    pub chat_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SenderInfo {
    pub sender_id: Option<SenderId>,
    pub sender_type: Option<String>,
    pub tenant_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SenderId {
    pub open_id: Option<String>,
    pub user_id: Option<String>,
    pub union_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MessageInfo {
    pub message_id: Option<String>,
    pub create_time: Option<String>,
    pub chat_id: Option<String>,
    pub chat_type: Option<String>,
    pub message_type: Option<String>,
    pub content: Option<String>,
}

impl LarkEventHandler {
    pub fn new() -> Self {
        Self {
            auto_reply: true,
        }
    }

    pub fn with_auto_reply(mut self, auto_reply: bool) -> Self {
        self.auto_reply = auto_reply;
        self
    }

    pub fn handle_message(&self, message: &ReceivedMessage) -> Result<Option<ProcessedMessage>> {
        let sender_id = message.sender
            .as_ref()
            .and_then(|s| s.sender_id.as_ref())
            .and_then(|id| id.open_id.clone().or(id.user_id.clone()))
            .unwrap_or_else(|| "unknown".to_string());

        let content = message.message
            .as_ref()
            .and_then(|m| m.content.clone())
            .unwrap_or_default();

        let message_type = message.message
            .as_ref()
            .and_then(|m| m.message_type.clone())
            .unwrap_or_else(|| "text".to_string());

        debug!("Processing message from {}: type={}", sender_id, message_type);

        let processed = ProcessedMessage {
            sender_id,
            chat_id: message.chat_id.clone().unwrap_or_default(),
            message_type,
            content: content.clone(),
            should_reply: self.auto_reply && !content.is_empty(),
        };

        Ok(Some(processed))
    }

    pub fn extract_text(&self, content: &str, message_type: &str) -> Result<String> {
        if message_type == "text" {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
                return Ok(parsed["text"].as_str().unwrap_or(content).to_string());
            }
        }
        Ok(content.to_string())
    }
}

#[derive(Debug)]
pub struct ProcessedMessage {
    pub sender_id: String,
    pub chat_id: String,
    pub message_type: String,
    pub content: String,
    pub should_reply: bool,
}

impl Default for LarkEventHandler {
    fn default() -> Self {
        Self::new()
    }
}
