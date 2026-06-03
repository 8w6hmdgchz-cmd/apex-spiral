use anyhow::Result;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn, error};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct GithubWebhookHandler {
    secret: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebhookPayload {
    pub action: Option<String>,
    pub repository: Option<RepositoryInfo>,
    pub sender: Option<SenderInfo>,
    pub issue: Option<IssueInfo>,
    pub pull_request: Option<PRInfo>,
    pub commits: Option<Vec<CommitInfo>>,
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepositoryInfo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub clone_url: Option<String>,
    pub html_url: Option<String>,
    pub default_branch: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SenderInfo {
    pub login: String,
    pub id: u64,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IssueInfo {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: Option<String>,
    pub user: Option<SenderInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PRInfo {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: Option<String>,
    pub user: Option<SenderInfo>,
    pub merged: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author: CommitAuthor,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
}

impl GithubWebhookHandler {
    pub fn new(secret: &str) -> Result<Self> {
        Ok(Self {
            secret: secret.to_string(),
        })
    }

    pub fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        if self.secret.is_empty() {
            warn!("GitHub webhook secret is not configured, skipping verification");
            return true;
        }

        let signature_clean = signature.trim_start_matches("sha256=");
        
        let mut mac = match HmacSha256::new_from_slice(self.secret.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return false,
        };
        
        mac.update(payload);
        
        let expected = hex::encode(mac.finalize().into_bytes());
        
        debug!("Verifying GitHub webhook signature");
        
        if expected != signature_clean {
            error!("GitHub signature mismatch: expected={}, got={}", expected, signature_clean);
            return false;
        }
        
        true
    }

    pub fn parse_event_type(&self, event_type: &str) -> GithubEventType {
        match event_type {
            "push" => GithubEventType::Push,
            "pull_request" => GithubEventType::PullRequest,
            "issues" => GithubEventType::Issues,
            "issue_comment" => GithubEventType::IssueComment,
            "create" => GithubEventType::Create,
            "delete" => GithubEventType::Delete,
            "fork" => GithubEventType::Fork,
            "watch" => GithubEventType::Watch,
            "release" => GithubEventType::Release,
            _ => GithubEventType::Unknown(event_type.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum GithubEventType {
    Push,
    PullRequest,
    Issues,
    IssueComment,
    Create,
    Delete,
    Fork,
    Watch,
    Release,
    Unknown(String),
}

impl WebhookPayload {
    pub fn get_event_summary(&self) -> String {
        let repo = self.repository.as_ref()
            .map(|r| r.full_name.clone())
            .unwrap_or_else(|| "unknown".to_string());
        
        let action = self.action.clone().unwrap_or_else(|| "none".to_string());
        
        format!("{} - {}", repo, action)
    }
}
