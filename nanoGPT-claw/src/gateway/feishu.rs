//! Feishu Gateway placeholder

#[derive(Debug, Clone, Default)]
pub struct FeishuConfig {
    pub enabled: bool,
    pub app_id: String,
    pub app_secret: String,
}

#[derive(Debug, Clone)]
pub struct FeishuGateway;

impl FeishuGateway {
    pub fn new(config: FeishuConfig) -> Self {
        Self
    }
    
    pub async fn send_message(&self, _chat_id: &str, _content: &str, _msg_type: &str) -> crate::gateway::GatewayResult<String> {
        Err(crate::gateway::GatewayError::GatewayNotEnabled("feishu".to_string()))
    }
    
    pub async fn start(&self) -> crate::gateway::GatewayResult<()> {
        Err(crate::gateway::GatewayError::GatewayNotEnabled("feishu".to_string()))
    }
    
    pub async fn status(&self) -> crate::gateway::GatewayStatus {
        crate::gateway::GatewayStatus::Disconnected
    }
}
