use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::gateway_github::webhook::{GithubEventType, WebhookPayload};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct GithubEventHandler {
    pub auto_reply: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedGithubEvent {
    pub event_type: String,
    pub repository: String,
    pub action: Option<String>,
    pub sender: Option<String>,
    pub content: String,
    pub should_process: bool,
}

impl GithubEventHandler {
    pub fn new() -> Self {
        Self {
            auto_reply: false,
        }
    }

    pub fn handle_event(&self, event_type: &str, payload: &WebhookPayload) -> Result<ProcessedGithubEvent> {
        let event = match event_type {
            "push" => self.handle_push(payload),
            "pull_request" => self.handle_pull_request(payload),
            "issues" => self.handle_issues(payload),
            _ => self.handle_generic(event_type, payload),
        };

        debug!("Processed GitHub event: {:?}", event.event_type);
        Ok(event)
    }

    fn handle_push(&self, payload: &WebhookPayload) -> ProcessedGithubEvent {
        let branch = payload.git_ref.as_ref()
            .map(|r| r.replace("refs/heads/", ""))
            .unwrap_or_else(|| "unknown".to_string());
        
        let commit_count = payload.commits.as_ref().map(|c| c.len()).unwrap_or(0);
        
        ProcessedGithubEvent {
            event_type: "push".to_string(),
            repository: payload.repository.as_ref().map(|r| r.full_name.clone()).unwrap_or_default(),
            action: Some(format!("pushed {} commits to {}", commit_count, branch)),
            sender: payload.sender.as_ref().map(|s| s.login.clone()),
            content: format!(
                "Push to {}/{}: {} commit(s)", 
                payload.repository.as_ref().map(|r| r.full_name.as_str()).unwrap_or("?"),
                branch,
                commit_count
            ),
            should_process: true,
        }
    }

    fn handle_pull_request(&self, payload: &WebhookPayload) -> ProcessedGithubEvent {
        let pr = payload.pull_request.as_ref();
        let action = payload.action.clone().unwrap_or_else(|| "updated".to_string());
        
        ProcessedGithubEvent {
            event_type: "pull_request".to_string(),
            repository: payload.repository.as_ref().map(|r| r.full_name.clone()).unwrap_or_default(),
            action: Some(action.clone()),
            sender: payload.sender.as_ref().map(|s| s.login.clone()),
            content: format!(
                "Pull Request #{}: {} - {}",
                pr.map(|p| p.number).unwrap_or(0),
                action,
                pr.map(|p| p.title.as_str()).unwrap_or("unknown")
            ),
            should_process: action == "opened" || action == "synchronize",
        }
    }

    fn handle_issues(&self, payload: &WebhookPayload) -> ProcessedGithubEvent {
        let issue = payload.issue.as_ref();
        let action = payload.action.clone().unwrap_or_else(|| "updated".to_string());
        
        ProcessedGithubEvent {
            event_type: "issues".to_string(),
            repository: payload.repository.as_ref().map(|r| r.full_name.clone()).unwrap_or_default(),
            action: Some(action.clone()),
            sender: payload.sender.as_ref().map(|s| s.login.clone()),
            content: format!(
                "Issue #{}: {} - {}",
                issue.map(|i| i.number).unwrap_or(0),
                action,
                issue.map(|i| i.title.as_str()).unwrap_or("unknown")
            ),
            should_process: action == "opened" || action == "labeled",
        }
    }

    fn handle_generic(&self, event_type: &str, payload: &WebhookPayload) -> ProcessedGithubEvent {
        ProcessedGithubEvent {
            event_type: event_type.to_string(),
            repository: payload.repository.as_ref().map(|r| r.full_name.clone()).unwrap_or_default(),
            action: payload.action.clone(),
            sender: payload.sender.as_ref().map(|s| s.login.clone()),
            content: format!("GitHub event: {} on {}", event_type, 
                payload.repository.as_ref().map(|r| r.full_name.as_str()).unwrap_or("?")),
            should_process: true,
        }
    }
}

impl Default for GithubEventHandler {
    fn default() -> Self {
        Self::new()
    }
}
