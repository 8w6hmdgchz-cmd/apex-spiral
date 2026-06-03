//! # Lark Gateway - Official SDK Integration
//!
//! ✅ 使用 open-lark 官方 SDK，不再手写 HTTP 客户端
//! ✅ 完全兼容现有接口
//! ✅ 自动 token 刷新和管理
//! ✅ 类型安全的 API 调用

pub mod client;
pub mod webhook;
pub mod handlers;

use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{debug, info};

pub use client::LarkClient;
pub use webhook::LarkWebhookHandler;
pub use handlers::{LarkEventHandler, ProcessedMessage};

use crate::scheduler::Scheduler;

#[derive(Debug, Clone)]
pub struct LarkConfig {
    pub enabled: bool,
    pub app_id: String,
    pub app_secret: String,
    pub verify_token: String,
    pub encrypt_key: String,
    pub bot_name: String,
    pub auto_reply: bool,
}

impl Default for LarkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            app_id: String::new(),
            app_secret: String::new(),
            verify_token: String::new(),
            encrypt_key: String::new(),
            bot_name: "nanoGPT-Claw".to_string(),
            auto_reply: true,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct LarkGateway {
    config: LarkConfig,
    client: Option<Arc<LarkClient>>,
    webhook: LarkWebhookHandler,
    event_handler: LarkEventHandler,
}

impl LarkGateway {
    pub fn new(config: LarkConfig, _scheduler: Option<Arc<Scheduler>>) -> Result<Self> {
        if !config.enabled {
            info!("Lark gateway is disabled");
            return Ok(Self {
                config,
                client: None,
                webhook: LarkWebhookHandler::new("")?,
                event_handler: LarkEventHandler::new(),
            });
        }

        info!("Initializing Lark gateway with official SDK (app_id: {})...", 
              if config.app_id.len() > 8 { &config.app_id[..8] } else { &config.app_id });
        
        let client = LarkClient::new(&config.app_id, &config.app_secret)
            .context("Failed to create Lark client with official SDK")?;
        
        let webhook = LarkWebhookHandler::new(&config.verify_token)?;
        let event_handler = LarkEventHandler::new();

        info!("✅ Lark gateway initialized successfully with official SDK");

        Ok(Self {
            config,
            client: Some(Arc::new(client)),
            webhook,
            event_handler,
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub async fn send_message(&self, chat_id: &str, content: &str, _msg_type: &str) -> Result<String> {
        let client = self.client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Lark client not initialized"))?;

        info!("Sending message via official SDK to chat_id: {}", chat_id);
        
        let message_id = client.send_text_message(chat_id, content).await
            .context("Failed to send message via official Lark SDK")?;

        info!("✅ Message sent successfully, message_id: {}", message_id);
        Ok(message_id)
    }

    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            debug!("Lark gateway not enabled, skipping start");
            return Ok(());
        }

        info!("Starting Lark gateway with official SDK...");
        
        // 验证 token 获取是否正常
        if let Some(client) = &self.client {
            client.get_access_token().await
                .context("Failed to verify Lark SDK authentication")?;
            info!("✅ Lark SDK authentication verified");
        }

        info!("Lark gateway started successfully");
        Ok(())
    }
}

impl Drop for LarkGateway {
    fn drop(&mut self) {
        debug!("LarkGateway dropped (official SDK)");
    }
}
