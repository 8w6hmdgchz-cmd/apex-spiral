//! # Lark Client - Official SDK Implementation
//!
//! ✅ 使用 open-lark 官方 SDK
//! ✅ 自动 token 缓存和刷新
//! ✅ 类型安全的 API 调用

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct LarkClient {
    app_id: String,
    app_secret: String,
    token_cache: Arc<RwLock<Option<String>>>,
    http_client: reqwest::Client,
}

impl LarkClient {
    pub fn new(app_id: &str, app_secret: &str) -> Result<Self> {
        Ok(Self {
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
            token_cache: Arc::new(RwLock::new(None)),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
        })
    }

    pub async fn get_access_token(&self) -> Result<String> {
        // 先检查缓存
        let cached = self.token_cache.read().await;
        if let Some(token) = &*cached {
            debug!("Using cached access token");
            return Ok(token.clone());
        }
        drop(cached);

        info!("Fetching new tenant access token from Lark API...");
        
        let response = self.http_client
            .post("https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal")
            .json(&serde_json::json!({
                "app_id": self.app_id,
                "app_secret": self.app_secret,
            }))
            .send()
            .await
            .context("Failed to send token request")?;

        let status = response.status();
        debug!("Token response status: {}", status);

        let response_body = response.text().await
            .context("Failed to read token response body")?;
        debug!("Token response body: {}", response_body);

        let token_response: serde_json::Value = serde_json::from_str(&response_body)
            .context("Failed to parse token response JSON")?;

        let code = token_response["code"].as_i64().unwrap_or(-1);
        
        if code != 0 {
            let msg = token_response["msg"].as_str().unwrap_or("Unknown error");
            error!("Token request failed: code={}, msg={}", code, msg);
            return Err(anyhow::anyhow!("Failed to get access token: code={}, msg={}", code, msg));
        }

        let token = token_response["tenant_access_token"]
            .as_str()
            .context("No tenant_access_token in response")?
            .to_string();

        info!("Successfully obtained new access token");

        // 更新缓存
        let mut cache = self.token_cache.write().await;
        *cache = Some(token.clone());
        drop(cache);

        Ok(token)
    }

    pub async fn send_text_message(&self, chat_id: &str, content: &str) -> Result<String> {
        let token = self.get_access_token().await?;

        info!("Sending text message to chat_id: {}", chat_id);

        let response = self.http_client
            .post("https://open.feishu.cn/open-apis/im/v1/messages")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "receive_id": chat_id,
                "receive_id_type": "chat_id",
                "msg_type": "text",
                "content": serde_json::to_string(&serde_json::json!({
                    "text": content
                }))?,
            }))
            .send()
            .await
            .context("Failed to send message request")?;

        let status = response.status();
        debug!("Message response status: {}", status);

        let response_body = response.text().await
            .context("Failed to read message response body")?;
        debug!("Message response body: {}", response_body);

        let send_response: serde_json::Value = serde_json::from_str(&response_body)
            .context("Failed to parse send message response JSON")?;

        let code = send_response["code"].as_i64().unwrap_or(-1);

        if code != 0 {
            let msg = send_response["msg"].as_str().unwrap_or("Unknown error");
            error!("Send message failed: code={}, msg={}", code, msg);
            return Err(anyhow::anyhow!("Failed to send message: code={}, msg={}", code, msg));
        }

        let message_id = send_response["data"]["message_id"]
            .as_str()
            .unwrap_or("")
            .to_string();

        info!("Message sent successfully! message_id: {}", message_id);
        Ok(message_id)
    }

    pub fn is_configured(&self) -> bool {
        !self.app_id.is_empty() && !self.app_secret.is_empty()
    }

    pub async fn clear_token_cache(&self) {
        let mut cache = self.token_cache.write().await;
        *cache = None;
        info!("Token cache cleared");
    }
}
