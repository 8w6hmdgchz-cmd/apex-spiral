//! # Gateway Layer - Main Module

pub mod feishu;
pub mod github;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use feishu::FeishuGateway;
use feishu::FeishuConfig;
use github::GitHubGateway;
use github::GitHubConfig;

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
pub struct GatewayConfig {
    pub feishu: FeishuConfig,
    pub github: GitHubConfig,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            feishu: FeishuConfig::default(),
            github: GitHubConfig::default(),
        }
    }
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

impl GatewayManager {
    pub async fn new(config: GatewayConfig) -> Result<Self, GatewayError> {
        let (event_sender, event_receiver) = tokio::sync::mpsc::channel(100);
        let event_sender_clone = event_sender.clone();

        let feishu = if config.feishu.enabled {
            let gateway = FeishuGateway::new(config.feishu.clone());
            Some(gateway)
        } else {
            None
        };

        let github = if config.github.enabled {
            let gateway = GitHubGateway::new(config.github.clone());
            Some(gateway)
        } else {
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
        if let Some(feishu) = &*self.feishu.read().await {
            info!("Starting Feishu gateway...");
            feishu.start().await?;
        }
        
        // 启动 GitHub Gateway
        if let Some(github) = &*self.github.read().await {
            info!("Starting GitHub gateway...");
            github.start().await?;
        }
        
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
            Some(gateway) => gateway.send_message(chat_id, content, msg_type).await,
            None => Err(GatewayError::GatewayNotEnabled("feishu".to_string())),
        }
    }

    pub async fn handle_github_webhook(
        &self,
        payload: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<GatewayWebhookEvent, GatewayError> {
        let github = self.github.read().await;
        match github.as_ref() {
            Some(gateway) => gateway.handle_webhook(payload, headers).await,
            None => Err(GatewayError::GatewayNotEnabled("github".to_string())),
        }
    }

    pub async fn status(&self) -> HashMap<String, GatewayStatus> {
        let mut statuses = HashMap::new();
        
        if let Some(ref feishu) = *self.feishu.read().await {
            statuses.insert("feishu".to_string(), feishu.status().await);
        } else {
            statuses.insert("feishu".to_string(), GatewayStatus::Disconnected);
        }

        if let Some(ref github) = *self.github.read().await {
            statuses.insert("github".to_string(), github.status());
        }
        statuses
    }

    pub async fn is_ready(&self) -> bool {
        let feishu_status = if let Some(ref feishu) = *self.feishu.read().await {
            feishu.status().await == GatewayStatus::Connected
        } else {
            false
        };
        feishu_status || self.github.read().await.is_some()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("Feishu gateway error: {0}")]
    Feishu(String),

    #[error("GitHub gateway error: {0}")]
    GitHub(String),

    #[error("Gateway not enabled: {0}")]
    GatewayNotEnabled(String),

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Invalid payload: {0}")]
    InvalidPayload(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

pub type GatewayResult<T> = Result<T, GatewayError>;

pub trait Gateway: Send + Sync {
    fn start(&self) -> impl std::future::Future<Output = Result<(), GatewayError>> + Send;
    fn stop(&self) -> impl std::future::Future<Output = Result<(), GatewayError>> + Send;
    fn status(&self) -> GatewayStatus;
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_gateway_manager_creation() {
        let config = super::GatewayConfig::default();
        let manager = super::GatewayManager::new(config).await;
        assert!(manager.is_ok());
    }
}
