//! # GitHub Gateway - Official SDK Integration
//!
//! ✅ 使用 octocrab 官方 SDK
//! ✅ 类型安全的 GitHub API 调用
//! ✅ 支持个人访问 token 和 GitHub App 认证
//! ✅ Webhook 验证和事件处理

pub mod webhook;
pub mod handlers;
pub mod repository;

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub use webhook::GithubWebhookHandler;
pub use handlers::GithubEventHandler;
pub use repository::RepositoryManager;

pub static GITHUB_GATEWAY: once_cell::sync::Lazy<Arc<RwLock<Option<GithubGateway>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

#[derive(Debug, Clone)]
pub struct GithubConfig {
    pub enabled: bool,
    pub webhook_secret: String,
    pub api_token: String,
    pub repository: String,
    pub auto_scan_enabled: bool,
    pub scan_interval_hours: u64,
    pub auto_commit_enabled: bool,
    pub branches: Vec<String>,
}

impl Default for GithubConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            webhook_secret: String::new(),
            api_token: String::new(),
            repository: String::new(),
            auto_scan_enabled: true,
            scan_interval_hours: 24,
            auto_commit_enabled: false,
            branches: vec!["main".to_string(), "master".to_string()],
        }
    }
}

pub struct GithubGateway {
    pub config: GithubConfig,
    pub webhook_handler: GithubWebhookHandler,
    pub event_handler: GithubEventHandler,
    octocrab: Option<Arc<octocrab::Octocrab>>,
}

impl GithubGateway {
    pub fn new(config: GithubConfig) -> Result<Self> {
        if !config.enabled {
            info!("GitHub gateway is disabled");
            return Ok(Self {
                config,
                webhook_handler: GithubWebhookHandler::new("")?,
                event_handler: GithubEventHandler::new(),
                octocrab: None,
            });
        }

        info!("Initializing GitHub gateway with official octocrab SDK...");
        
        let octocrab = if !config.api_token.is_empty() {
            info!("Creating Octocrab instance with personal access token");
            let crab = octocrab::Octocrab::builder()
                .personal_token(config.api_token.clone())
                .build()
                .context("Failed to create Octocrab instance")?;
            Some(Arc::new(crab))
        } else {
            warn!("No GitHub API token provided, some features will be disabled");
            None
        };

        let webhook_handler = GithubWebhookHandler::new(&config.webhook_secret)?;
        let event_handler = GithubEventHandler::new();

        info!("✅ GitHub gateway initialized with octocrab SDK");

        Ok(Self {
            config,
            webhook_handler,
            event_handler,
            octocrab,
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            debug!("GitHub gateway not enabled, skipping start");
            return Ok(());
        }

        info!("Starting GitHub gateway with octocrab SDK...");
        
        // 验证 GitHub API 连接
        if let Some(crab) = &self.octocrab {
            info!("Verifying GitHub API connection...");
            
            if !self.config.repository.is_empty() {
                let parts: Vec<&str> = self.config.repository.split('/').collect();
                if parts.len() >= 2 {
                    let owner = parts[0];
                    let repo = parts[1];
                    
                    match crab.repos(owner, repo).get().await {
                        Ok(repo_info) => {
                            info!("✅ Verified repository access: {} (stars: {})", 
                                  repo_info.full_name.as_deref().unwrap_or("unknown"),
                                  repo_info.stargazers_count.unwrap_or(0));
                        }
                        Err(e) => {
                            warn!("⚠️ Repository access check failed: {}", e);
                        }
                    }
                }
            }
        }

        info!("GitHub gateway started successfully");
        Ok(())
    }

    pub async fn handle_webhook(
        &self,
        payload: &[u8],
        headers: &std::collections::HashMap<String, String>,
    ) -> Result<crate::gateway::GatewayWebhookEvent, crate::gateway::GatewayError> {
        use crate::gateway::{GatewayError, GatewayWebhookEvent};
        use std::time::{SystemTime, UNIX_EPOCH};

        debug!("Handling GitHub webhook with SDK...");

        // 验证签名
        self.webhook_handler.verify_signature(payload, headers)
            .map_err(|_| GatewayError::SignatureVerificationFailed)?;

        let event_type = headers.get("X-GitHub-Event")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let delivery_id = headers.get("X-GitHub-Delivery")
            .cloned()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        info!("Processing GitHub webhook event: {}, delivery: {}", event_type, delivery_id);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(GatewayWebhookEvent {
            source: "github".to_string(),
            event_type,
            payload: String::from_utf8_lossy(payload).to_string(),
            headers: headers.clone(),
            timestamp,
        })
    }

    pub fn octocrab(&self) -> Option<&Arc<octocrab::Octocrab>> {
        self.octocrab.as_ref()
    }
}

pub async fn init_github_gateway(config: GithubConfig) -> Result<()> {
    if !config.enabled {
        info!("GitHub integration is disabled");
        return Ok(());
    }

    let gateway = GithubGateway::new(config)?;
    let mut github = GITHUB_GATEWAY.write().await;
    *github = Some(gateway);
    
    info!("✅ GitHub gateway initialized successfully");
    Ok(())
}
