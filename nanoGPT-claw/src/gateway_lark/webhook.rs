use anyhow::Result;
use serde::{Deserialize, Serialize};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{debug, warn};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct LarkWebhookHandler {
    verify_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LarkChallengeRequest {
    #[serde(rename = "challenge")]
    pub challenge: Option<String>,
    #[serde(rename = "token")]
    pub token: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LarkChallengeResponse {
    #[serde(rename = "challenge")]
    pub challenge: String,
}

#[derive(Debug, Deserialize)]
pub struct LarkEventPayload {
    #[serde(rename = "schema")]
    pub schema: Option<String>,
    #[serde(rename = "header")]
    pub header: Option<EventHeader>,
    #[serde(rename = "event")]
    pub event: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct EventHeader {
    pub event_id: Option<String>,
    pub event_type: Option<String>,
    pub create_time: Option<String>,
    pub token: Option<String>,
    pub app_id: Option<String>,
    pub tenant_key: Option<String>,
}

impl LarkWebhookHandler {
    pub fn new(verify_token: &str) -> Result<Self> {
        Ok(Self {
            verify_token: verify_token.to_string(),
        })
    }

    pub fn verify_url(&self, challenge: &str) -> Result<LarkChallengeResponse> {
        debug!("Verifying webhook URL with challenge: {}", challenge);
        Ok(LarkChallengeResponse {
            challenge: challenge.to_string(),
        })
    }

    pub fn verify_token(&self, token: &str) -> bool {
        if self.verify_token.is_empty() {
            warn!("Verify token is not configured");
            return true;
        }
        self.verify_token == token
    }

    pub fn verify_signature(&self, timestamp: &str, nonce: &str, signature: &str, encrypt_key: &str) -> bool {
        if encrypt_key.is_empty() {
            debug!("No encrypt key configured, skipping signature verification");
            return true;
        }

        let message = format!("{}{}{}", timestamp, nonce, encrypt_key);
        
        let mut mac = match HmacSha256::new_from_slice(encrypt_key.as_bytes()) {
            Ok(m) => m,
            Err(_) => return false, // Hmac accepts any key size, this should never fail
        };
        mac.update(message.as_bytes());
        
        let expected = hex::encode(mac.finalize().into_bytes());
        
        debug!("Verifying webhook signature");
        expected == signature
    }

    pub fn parse_event(&self, payload: &LarkEventPayload) -> Option<LarkEvent> {
        let event_type = payload.header.as_ref()?.event_type.as_ref()?.clone();
        
        match event_type.as_str() {
            "im.message.receive_v1" => {
                Some(LarkEvent::MessageReceive(MessageReceiveEvent {
                    event_type,
                    event_id: payload.header.as_ref()?.event_id.clone(),
                    content: payload.event.as_ref()?.to_string(),
                }))
            }
            _ => {
                debug!("Received unhandled event type: {}", event_type);
                Some(LarkEvent::Unknown(event_type))
            }
        }
    }
}

#[derive(Debug)]
pub enum LarkEvent {
    MessageReceive(MessageReceiveEvent),
    Unknown(String),
}

#[derive(Debug)]
pub struct MessageReceiveEvent {
    pub event_type: String,
    pub event_id: Option<String>,
    pub content: String,
}

impl LarkChallengeRequest {
    pub fn is_url_verification(&self) -> bool {
        self.challenge.is_some() && self.event_type.as_ref().map(|t| t == "url_verification").unwrap_or(false)
    }
}
