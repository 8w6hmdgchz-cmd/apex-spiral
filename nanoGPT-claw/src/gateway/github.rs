//! GitHub Gateway placeholder

#[derive(Debug, Clone, Default)]
pub struct GitHubConfig {
    pub enabled: bool,
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct GitHubGateway;

impl GitHubGateway {
    pub fn new(config: GitHubConfig) -> Self {
        Self
    }
    
    pub async fn handle_webhook(&self, _payload: &[u8], _headers: &std::collections::HashMap<String, String>) -> crate::gateway::GatewayResult<crate::gateway::GatewayWebhookEvent> {
        Err(crate::gateway::GatewayError::GatewayNotEnabled("github".to_string()))
    }
    
    pub async fn start(&self) -> crate::gateway::GatewayResult<()> {
        Err(crate::gateway::GatewayError::GatewayNotEnabled("github".to_string()))
    }
    
    pub fn status(&self) -> crate::gateway::GatewayStatus {
        crate::gateway::GatewayStatus::Disconnected
    }
}
