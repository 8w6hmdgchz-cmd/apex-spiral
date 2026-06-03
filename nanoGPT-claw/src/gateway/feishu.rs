//! # Feishu Gateway Module
//!
//! 使用 gateway_lark 官方 SDK 实现飞书网关

use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::gateway_lark::client::LarkClient;
use crate::gateway_lark::webhook::{LarkChallengeRequest, LarkEventPayload, LarkWebhookHandler};
use crate::gateway::{GatewayError, GatewayStatus};
use crate::memory::MemoryEntry;

#[derive(Debug, Clone)]
pub struct FeishuConfig {
    pub app_id: String,
    pub app_secret: String,
    pub verification_token: String,
    pub encrypt_key: Option<String>,
    pub api_base_url: String,
    pub enabled: bool,
}

impl Default for FeishuConfig {
    fn default() -> Self {
        Self {
            app_id: String::new(),
            app_secret: String::new(),
            verification_token: String::new(),
            encrypt_key: None,
            api_base_url: "https://open.feishu.cn".to_string(),
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub struct FeishuGateway {
    config: FeishuConfig,
    lark_client: Arc<LarkClient>,
    webhook_handler: Arc<LarkWebhookHandler>,
    memory: Arc<RwLock<Option<MemoryEntry>>>,
    status: Arc<RwLock<GatewayStatus>>,
}

impl FeishuGateway {
    pub fn new(config: FeishuConfig) -> Self {
        info!("Creating FeishuGateway - enabled={}, app_id={}", 
            config.enabled, 
            if config.app_id.is_empty() { "<empty>" } else { &config.app_id[..8.min(config.app_id.len())] }
        );

        let lark_client = if config.enabled && !config.app_id.is_empty() && !config.app_secret.is_empty() {
            match LarkClient::new(&config.app_id, &config.app_secret) {
                Ok(client) => {
                    info!("LarkClient created successfully");
                    client
                }
                Err(e) => {
                    warn!("Failed to create LarkClient: {:?}", e);
                    LarkClient::new("", "").expect("Failed to create dummy client")
                }
            }
        } else {
            info!("LarkClient not created: enabled={}, app_id empty={}, app_secret empty={}", 
                config.enabled, 
                config.app_id.is_empty(), 
                config.app_secret.is_empty()
            );
            LarkClient::new("", "").expect("Failed to create dummy client")
        };

        let webhook_handler = if !config.verification_token.is_empty() {
            match LarkWebhookHandler::new(&config.verification_token) {
                Ok(handler) => {
                    info!("LarkWebhookHandler created successfully");
                    handler
                }
                Err(e) => {
                    warn!("Failed to create LarkWebhookHandler: {:?}", e);
                    LarkWebhookHandler::new("").expect("Failed to create dummy handler")
                }
            }
        } else {
            info!("LarkWebhookHandler not created: verification_token is empty");
            LarkWebhookHandler::new("").expect("Failed to create dummy handler")
        };

        Self {
            config,
            lark_client: Arc::new(lark_client),
            webhook_handler: Arc::new(webhook_handler),
            memory: Arc::new(RwLock::new(None)),
            status: Arc::new(RwLock::new(GatewayStatus::Disconnected)),
        }
    }

    pub async fn start(&self) -> Result<(), GatewayError> {
        info!("FeishuGateway starting...");

        {
            let mut status = self.status.write().await;
            *status = GatewayStatus::Connecting;
        }

        if self.config.enabled && !self.config.app_id.is_empty() && !self.config.app_secret.is_empty() {
            info!("FeishuGateway enabled, testing connection...");

            let client = (*self.lark_client).clone();
            match client.get_access_token().await {
                Ok(token) => {
                    info!("FeishuGateway access token obtained successfully!");
                    let token_len = token.len();
                    debug!("Token length: {} chars", token_len);
                }
                Err(e) => {
                    warn!("FeishuGateway failed to get access token: {:?}", e);
                }
            }
        } else {
            info!("FeishuGateway is disabled or not configured");
        }

        {
            let mut status = self.status.write().await;
            *status = if self.config.enabled && !self.config.app_id.is_empty() {
                GatewayStatus::Connected
            } else {
                GatewayStatus::Disconnected
            };
        }

        info!("FeishuGateway started successfully");
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), GatewayError> {
        info!("FeishuGateway stopping...");
        let mut status = self.status.write().await;
        *status = GatewayStatus::Disconnected;
        info!("FeishuGateway stopped");
        Ok(())
    }

    pub async fn status(&self) -> GatewayStatus {
        self.status.read().await.clone()
    }

    pub async fn get_access_token(&self) -> Result<String, GatewayError> {
        if !self.config.enabled || self.config.app_id.is_empty() {
            return Err(GatewayError::GatewayNotEnabled("feishu".to_string()));
        }

        let client = (*self.lark_client).clone();
        client.get_access_token()
            .await
            .map_err(|e| GatewayError::Authentication(format!("Failed to get access token: {:?}", e)))
    }

    pub async fn send_message(
        &self,
        receive_id: &str,
        content: &str,
        _msg_type: &str,
    ) -> Result<String, GatewayError> {
        debug!("Sending Feishu message to {}", receive_id);

        if !self.config.enabled || self.config.app_id.is_empty() {
            return Err(GatewayError::GatewayNotEnabled("feishu".to_string()));
        }

        let client = &*self.lark_client;
        client.send_text_message(receive_id, content)
            .await
            .map_err(|e| {
                error!("Failed to send message: {:?}", e);
                GatewayError::Feishu(format!("Failed to send message: {:?}", e))
            })
    }

    pub async fn handle_webhook_url_verification(&self, body: &[u8]) -> Result<Vec<u8>, GatewayError> {
        debug!("Handling URL verification webhook");

        let handler = &*self.webhook_handler;

        let request: LarkChallengeRequest = serde_json::from_slice(body)
            .map_err(|e| GatewayError::Parse(format!("Failed to parse challenge request: {}", e)))?;

        if request.is_url_verification() {
            let challenge = request.challenge.as_ref()
                .ok_or_else(|| GatewayError::InvalidPayload("Missing challenge".to_string()))?;

            info!("URL verification challenge: {}", challenge);

            let response = handler.verify_url(challenge)
                .map_err(|e| GatewayError::Parse(format!("Failed to create challenge response: {:?}", e)))?;

            serde_json::to_vec(&response)
                .map_err(|e| GatewayError::Parse(format!("Failed to serialize response: {}", e)))
        } else {
            Err(GatewayError::InvalidPayload("Not a URL verification request".to_string()))
        }
    }

    pub async fn handle_webhook_event(&self, body: &[u8]) -> Result<String, GatewayError> {
        debug!("Handling event webhook");

        let handler = &*self.webhook_handler;

        let event_payload: LarkEventPayload = serde_json::from_slice(body)
            .map_err(|e| GatewayError::Parse(format!("Failed to parse event payload: {}", e)))?;

        match handler.parse_event(&event_payload) {
            Some(event) => {
                info!("Received event: {:?}", event);
                Ok(format!("{{\"event_type\": \"{}\"}}", match &event {
                    crate::gateway_lark::webhook::LarkEvent::MessageReceive(e) => &e.event_type,
                    crate::gateway_lark::webhook::LarkEvent::Unknown(t) => t,
                }))
            }
            None => {
                Err(GatewayError::InvalidPayload("Failed to parse event".to_string()))
            }
        }
    }

    pub fn verify_webhook(&self, signature: &str, timestamp: &str, encrypted: bool) -> bool {
        if !encrypted {
            return true;
        }

        let handler = &*self.webhook_handler;
        let encrypt_key = self.config.encrypt_key.as_deref().unwrap_or("");
        handler.verify_signature(timestamp, "", signature, encrypt_key)
    }
}

impl Clone for FeishuGateway {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            lark_client: self.lark_client.clone(),
            webhook_handler: self.webhook_handler.clone(),
            memory: self.memory.clone(),
            status: self.status.clone(),
        }
    }
}
