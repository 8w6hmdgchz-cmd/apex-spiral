//! # Gateway Layer - Main Module

pub mod feishu;
pub mod github;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

pub use feishu::{FeishuConfig, FeishuGateway};
pub use github::{GitHubConfig, GitHubGateway};

#[derive(Debug, Clone)]
pub enum GatewayEvent {
    Message(GatewayMessage),
    Webhook(GatewayWebhookEvent),
    Callback(CallbackEvent),
    Status(StatusEvent),
}

#[derive(Debug, Clone)]
pub struct GatewayMessage {
    pub message_id: String,
    pub channel: String,
    pub sender_id: String,
    pub chat_id: String,
    pub content: String,
    pub msg_type: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct GatewayWebhookEvent {
    pub source: String,
    pub event_type: String,
    pub payload: String,
    pub headers: HashMap<String, String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct CallbackEvent {
    pub callback_id: String,
    pub message_id: String,
    pub user_id: String,
    pub data: HashMap<String, String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct StatusEvent {
    pub gateway: String,
    pub status: GatewayStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayStatus {
    Connected,
    Connecting,
    Disconnected,
    Error,
}

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct GatewayConfig {
    pub feishu: FeishuConfig,
    pub github: GitHubConfig,
}

#[allow(dead_code)]
pub struct GatewayManager {
    feishu: Arc<RwLock<Option<FeishuGateway>>>,
    github: Arc<RwLock<Option<GitHubGateway>>>,
    config: GatewayConfig,
    event_receiver: tokio::sync::mpsc::Receiver<GatewayEvent>,
    event_sender: tokio::sync::mpsc::Sender<GatewayEvent>,
    shutdown: Arc<RwLock<bool>>,
}

#[derive(Debug)]
pub enum GatewayError {
    GatewayNotEnabled(String),
    GatewayNotInitialized(String),
    SendError,
    Network(String),
    Authentication(String),
    Feishu(String),
    Parse(String),
    InvalidPayload(String),
    SignatureVerificationFailed,
}

impl std::fmt::Display for GatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GatewayNotEnabled(gw) => write!(f, "Gateway {} not enabled", gw),
            Self::GatewayNotInitialized(gw) => write!(f, "Gateway {} not initialized", gw),
            Self::SendError => write!(f, "Failed to send message"),
            Self::Network(e) => write!(f, "Network error: {}", e),
            Self::Authentication(e) => write!(f, "Authentication error: {}", e),
            Self::Feishu(e) => write!(f, "Feishu API error: {}", e),
            Self::Parse(e) => write!(f, "Parse error: {}", e),
            Self::InvalidPayload(e) => write!(f, "Invalid payload: {}", e),
            Self::SignatureVerificationFailed => write!(f, "Signature verification failed"),
        }
    }
}

impl std::error::Error for GatewayError {}

impl GatewayManager {
    pub async fn new(config: GatewayConfig) -> Result<Self, GatewayError> {
        let (event_sender, event_receiver) = tokio::sync::mpsc::channel(100);
        let event_sender_clone = event_sender.clone();

        info!("Initializing GatewayManager...");
        debug!("Feishu config: enabled={}", config.feishu.enabled);
        debug!("GitHub config: enabled={}", config.github.enabled);

        let feishu = if config.feishu.enabled {
            info!("Feishu gateway enabled, creating FeishuGateway...");
            Some(FeishuGateway::new(config.feishu.clone()))
        } else {
            info!("Feishu gateway disabled");
            None
        };

        let github = if config.github.enabled {
            info!("GitHub gateway enabled, creating GitHubGateway...");
            Some(GitHubGateway::new(config.github.clone()))
        } else {
            info!("GitHub gateway disabled");
            None
        };

        Ok(Self {
            feishu: Arc::new(RwLock::new(feishu)),
            github: Arc::new(RwLock::new(github)),
            config,
            event_receiver,
            event_sender: event_sender_clone,
            shutdown: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn start(&self) -> Result<(), GatewayError> {
        info!("Starting gateway manager...");

        // 启动 Feishu Gateway
        let feishu_guard = self.feishu.read().await;
        if let Some(feishu) = &*feishu_guard {
            info!("Starting Feishu gateway...");
            feishu.start().await?;
            info!("Feishu gateway started successfully");
        } else {
            debug!("Feishu gateway not configured, skipping");
        }
        drop(feishu_guard);

        // 启动 GitHub Gateway
        let github_guard = self.github.read().await;
        if let Some(github) = &*github_guard {
            info!("Starting GitHub gateway...");
            github.start().await?;
            info!("GitHub gateway started successfully");
        } else {
            debug!("GitHub gateway not configured, skipping");
        }
        drop(github_guard);

        info!("Gateway manager started successfully");
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), GatewayError> {
        *self.shutdown.write().await = true;
        Ok(())
    }

    pub async fn next_event(&mut self) -> Option<GatewayEvent> {
        self.event_receiver.recv().await
    }

    pub async fn send_feishu_message(
        &self,
        chat_id: &str,
        content: &str,
        msg_type: &str,
    ) -> Result<String, GatewayError> {
        let feishu = self.feishu.read().await;
        match feishu.as_ref() {
            Some(gateway) => {
                debug!("Sending Feishu message to chat_id={}", chat_id);
                gateway.send_message(chat_id, content, msg_type).await
            }
            None => {
                warn!("Feishu gateway not enabled or not initialized");
                Err(GatewayError::GatewayNotEnabled("feishu".to_string()))
            }
        }
    }

    pub async fn handle_github_webhook(
        &self,
        payload: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<GatewayWebhookEvent, GatewayError> {
        let github = self.github.read().await;
        match github.as_ref() {
            Some(gateway) => {
                debug!("Handling GitHub webhook...");
                gateway.handle_webhook(payload, headers).await
            }
            None => {
                warn!("GitHub gateway not enabled or not initialized");
                Err(GatewayError::GatewayNotEnabled("github".to_string()))
            }
        }
    }
}
