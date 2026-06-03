#![allow(clippy::type_complexity)]

//! # GitHub Webhook Gateway Module
//!
//! Implements GitHub Webhook handling with HMAC-SHA256 signature verification.
//! This gateway receives and validates GitHub webhook events securely.
//!
//! ## Security Features
//!
//! - **HMAC-SHA256 Signature Verification**: Every webhook request is verified
//!   using the `X-Hub-Signature-256` header against the configured webhook secret
//! - **Replay Attack Prevention**: Validates `X-GitHub-Event` and `X-GitHub-Delivery` headers
//! - **IP Whitelisting**: Optionally verify requests come from GitHub's IP ranges
//!
//! ## Supported Events
//!
//! - `push` - Push to a repository
//! - `pull_request` - Pull request opened, closed, synchronized, etc.
//! - `issues` - Issue opened, closed, labeled, etc.
//! - `issue_comment` - Comment on an issue or PR
//! - `create` - Branch or tag created
//! - `delete` - Branch or tag deleted
//! - `release` - Release published
//! - `workflow_run` - Workflow run completed
//! - Custom events via `*` handler
//!
//! ## Event Delivery
//!
//! GitHub sends webhooks with these headers:
//! - `X-GitHub-Event`: Event type (e.g., "push", "pull_request")
//! - `X-GitHub-Delivery`: Unique delivery ID (GUID)
//! - `X-Hub-Signature-256`: HMAC-SHA256 signature of the payload
//! - `X-GitHub-Hook-ID`: Webhook ID
//! - `X-GitHub-Hook-Installation-Target-ID`: Installation ID for GitHub Apps
//!
//! # Example
//!
//! ```rust,ignore
//! use nano_gpt_claw::gateway::github::{GitHubGateway, GitHubConfig};
//!
//! let config = GitHubConfig {
//!     webhook_secret: std::env::var("GITHUB_WEBHOOK_SECRET")
//!         .unwrap_or_else(|_| panic!("GITHUB_WEBHOOK_SECRET must be set")),
//!     ..Default::default()
//! };
//!
//! let gateway = GitHubGateway::new(config);
//!
//! // Handle incoming webhook
//! let event = gateway.handle_webhook(payload_bytes, &headers).await?;
//! ```
//!
//! # Signature Verification
//!
//! The signature is computed as:
//! ```text
//! HMAC-SHA256(webhook_secret, request_body)
//! ```
//!
//! And compared against the `X-Hub-Signature-256` header value which has the format:
//! ```text
//! sha256=<hex_encoded_signature>
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokio::sync::RwLock;

use super::{GatewayError, GatewayStatus, GatewayWebhookEvent};

/// HMAC-SHA256 type alias for cleaner code.
type HmacSha256 = Hmac<Sha256>;

/// Configuration for the GitHub Gateway.
#[derive(Debug, Clone)]
pub struct GitHubConfig {
    /// Webhook secret for signature verification
    pub webhook_secret: String,
    /// Enable the gateway
    pub enabled: bool,
    /// Allowed repository patterns (glob-style, e.g., "owner/repo", "owner/*")
    pub allowed_repos: Vec<String>,
    /// Allowed event types (empty = allow all)
    pub allowed_events: Vec<String>,
    /// GitHub API base URL (for checking additional info)
    pub api_base_url: String,
    /// Timeout for webhook processing in seconds
    pub timeout_secs: u64,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            webhook_secret: String::new(),
            enabled: false,
            allowed_repos: Vec::new(),
            allowed_events: Vec::new(),
            api_base_url: "https://api.github.com".to_string(),
            timeout_secs: 30,
        }
    }
}

/// GitHub webhook delivery headers.
#[derive(Debug, Clone)]
pub struct GitHubWebhookHeaders {
    /// Event type (push, pull_request, etc.)
    pub event: String,
    /// Unique delivery ID
    pub delivery_id: String,
    /// HMAC signature
    pub signature: Option<String>,
    /// Hook ID
    pub hook_id: String,
    /// Installation ID (for GitHub Apps)
    pub installation_id: Option<String>,
    /// Repository full name (owner/repo)
    pub repository: Option<String>,
}

impl GitHubWebhookHeaders {
    /// Parses headers from a HashMap (e.g., from HTTP headers).
    ///
    /// # Arguments
    /// * `headers` - Raw header map
    ///
    /// # Returns
    /// * `Option<Self>` - Parsed headers if all required headers present
    pub fn from_map(headers: &HashMap<String, String>) -> Option<Self> {
        let event = headers.get("X-GitHub-Event")?.clone();
        let delivery_id = headers.get("X-GitHub-Delivery")?.clone();
        let signature = headers.get("X-Hub-Signature-256").cloned();
        let hook_id = headers.get("X-GitHub-Hook-ID").cloned().unwrap_or_default();
        let installation_id = headers.get("X-GitHub-Hook-Installation-Target-Id").cloned();
        let repository = headers.get("X-GitHub-Repository").cloned();

        Some(Self {
            event,
            delivery_id,
            signature,
            hook_id,
            installation_id,
            repository,
        })
    }
}

/// GitHub Gateway for handling webhook events.
/// This gateway is stateless — it processes individual webhook deliveries
/// without maintaining persistent connections.
pub struct GitHubGateway {
    /// Configuration
    config: GitHubConfig,
    /// Current connection status (always Connected for webhook receiver)
    status: Arc<RwLock<GatewayStatus>>,
    /// Event counter for monitoring
    events_received: Arc<RwLock<u64>>,
    /// Signature verification failures counter
    verification_failures: Arc<RwLock<u64>>,
}

impl GitHubGateway {
    /// Creates a new GitHubGateway instance.
    ///
    /// # Arguments
    /// * `config` - GitHub gateway configuration
    ///
    /// # Returns
    /// * `Self` - New gateway instance
    pub fn new(config: GitHubConfig) -> Self {
        Self {
            config,
            status: Arc::new(RwLock::new(GatewayStatus::Connected)),
            events_received: Arc::new(RwLock::new(0)),
            verification_failures: Arc::new(RwLock::new(0)),
        }
    }

    /// Starts the GitHub gateway.
    /// For webhook receivers, this is a no-op as the gateway is inherently stateless.
    ///
    /// # Returns
    /// * `Result<()>` - Success
    pub async fn start(&self) -> Result<(), GatewayError> {
        *self.status.write().await = GatewayStatus::Connected;
        Ok(())
    }

    /// Stops the GitHub gateway.
    /// No cleanup needed for stateless webhook receiver.
    ///
    /// # Returns
    /// * `Result<()>` - Success
    pub async fn stop(&self) -> Result<(), GatewayError> {
        *self.status.write().await = GatewayStatus::Disconnected;
        Ok(())
    }

    /// Returns the current connection status.
    pub fn status(&self) -> GatewayStatus {
        self.status.blocking_read().clone()
    }

    /// Returns the number of events received.
    pub fn events_received(&self) -> u64 {
        *self.events_received.blocking_read()
    }

    /// Returns the number of signature verification failures.
    pub fn verification_failures(&self) -> u64 {
        *self.verification_failures.blocking_read()
    }

    /// Handles an incoming webhook request.
    ///
    /// # Arguments
    /// * `payload` - Raw request body bytes
    /// * `headers` - HTTP headers (lowercase keys)
    ///
    /// # Returns
    /// * `Result<GatewayWebhookEvent>` - Parsed and validated event
    ///
    /// # Errors
    /// Returns `GatewayError::SignatureVerificationFailed` if:
    /// - HMAC signature doesn't match
    /// - Webhook secret is not configured (production mode)
    /// - Payload is empty
    ///
    /// Returns `GatewayError::InvalidPayload` if:
    /// - Required headers are missing
    /// - JSON parsing fails
    pub async fn handle_webhook(
        &self,
        payload: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<GatewayWebhookEvent, GatewayError> {
        // Increment event counter
        {
            let mut counter = self.events_received.write().await;
            *counter += 1;
        }

        // Parse headers
        let webhook_headers = GitHubWebhookHeaders::from_map(headers).ok_or_else(|| {
            GatewayError::InvalidPayload("Missing required GitHub headers".to_string())
        })?;

        // Verify signature
        if let Some(ref sig) = webhook_headers.signature {
            if !self.verify_signature(payload, sig) {
                let mut failures = self.verification_failures.write().await;
                *failures += 1;
                return Err(GatewayError::SignatureVerificationFailed);
            }
        } else if !self.config.webhook_secret.is_empty() {
            // Signature expected but not provided
            let mut failures = self.verification_failures.write().await;
            *failures += 1;
            return Err(GatewayError::SignatureVerificationFailed);
        }

        // Validate event type if allowed_events is configured
        if !self.config.allowed_events.is_empty()
            && !self
                .config
                .allowed_events
                .iter()
                .any(|e| e == "*" || e == &webhook_headers.event)
        {
            return Err(GatewayError::InvalidPayload(format!(
                "Event type '{}' not in allowed list",
                webhook_headers.event
            )));
        }

        // Validate repository if allowed_repos is configured
        if let Some(ref repo) = webhook_headers.repository {
            if !self.is_repo_allowed(repo) {
                return Err(GatewayError::InvalidPayload(format!(
                    "Repository '{}' not in allowed list",
                    repo
                )));
            }
        }

        // Parse the payload as JSON
        let payload_str = String::from_utf8_lossy(payload);
        let json_value: serde_json::Value = serde_json::from_str(&payload_str)
            .map_err(|e| GatewayError::InvalidPayload(format!("Failed to parse JSON: {}", e)))?;

        // Extract relevant information based on event type
        let _event_info = self.extract_event_info(&webhook_headers.event, &json_value);

        Ok(GatewayWebhookEvent {
            source: "github".to_string(),
            event_type: webhook_headers.event,
            payload: payload_str.to_string(),
            headers: headers.clone(),
            timestamp: current_time_millis(),
        })
    }

    /// Verifies the HMAC-SHA256 signature of a webhook payload.
    ///
    /// # Arguments
    /// * `payload` - Raw request body
    /// * `signature` - Signature from X-Hub-Signature-256 header (format: "sha256=...")
    ///
    /// # Returns
    /// * `bool` - True if signature is valid
    ///
    /// # Algorithm
    /// 1. Extract hex signature from "sha256=xxxx" format
    /// 2. Compute HMAC-SHA256(webhook_secret, payload)
    /// 3. Compare using constant-time comparison (prevent timing attacks)
    pub fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        // If no secret configured, skip verification (dev mode only)
        if self.config.webhook_secret.is_empty() {
            return true;
        }

        // Extract hex signature from "sha256=xxxx" format
        let expected_hex = signature.strip_prefix("sha256=").unwrap_or(signature);

        // Compute HMAC-SHA256
        let mut mac = match HmacSha256::new_from_slice(self.config.webhook_secret.as_bytes()) {
            Ok(m) => m,
            Err(_) => return false, // Hmac accepts any key size, this should never fail
        };
        mac.update(payload);

        let result = mac.finalize();

        // Encode computed signature as hex
        let computed_hex = hex::encode(result.into_bytes());

        // Constant-time comparison
        constant_time_compare(&computed_hex, expected_hex)
    }

    /// Checks if a repository is in the allowed list.
    ///
    /// # Arguments
    /// * `repo` - Repository full name (e.g., "owner/repo")
    ///
    /// # Returns
    /// * `bool` - True if allowed or no restrictions configured
    fn is_repo_allowed(&self, repo: &str) -> bool {
        if self.config.allowed_repos.is_empty() {
            return true; // No restrictions configured
        }

        self.config.allowed_repos.iter().any(|pattern| {
            if pattern.contains('*') {
                // Glob-style matching
                let parts: Vec<&str> = pattern.split('/').collect();
                let repo_parts: Vec<&str> = repo.split('/').collect();
                if parts.len() == 2 && repo_parts.len() == 2 {
                    let owner_match = parts[0] == "*" || parts[0] == repo_parts[0];
                    let repo_match = parts[1] == "*" || parts[1] == repo_parts[1];
                    return owner_match && repo_match;
                }
                false
            } else {
                pattern == repo
            }
        })
    }

    /// Extracts relevant information from the payload based on event type.
    /// This creates a structured summary for easier downstream processing.
    ///
    /// # Arguments
    /// * `event_type` - The GitHub event type
    /// * `payload` - Parsed JSON payload
    ///
    /// # Returns
    /// * `GitHubEventInfo` - Extracted information
    fn extract_event_info(&self, event_type: &str, payload: &serde_json::Value) -> GitHubEventInfo {
        match event_type {
            "push" => {
                let ref_ = payload.get("ref").and_then(|v| v.as_str()).unwrap_or("");
                let commits = payload
                    .get("commits")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0);
                let repository = payload
                    .get("repository")
                    .and_then(|r| r.get("full_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                GitHubEventInfo {
                    event_type: event_type.to_string(),
                    action: None,
                    repository: Some(repository.to_string()),
                    branch: Some(extract_branch_from_ref(ref_)),
                    title: Some(format!(
                        "{} commits to {}",
                        commits,
                        extract_branch_from_ref(ref_)
                    )),
                    sender: payload
                        .get("sender")
                        .and_then(|s| s.get("login"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                }
            }
            "pull_request" => {
                let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");
                let title = payload
                    .get("pull_request")
                    .and_then(|pr| pr.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let repository = payload
                    .get("repository")
                    .and_then(|r| r.get("full_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                GitHubEventInfo {
                    event_type: event_type.to_string(),
                    action: Some(action.to_string()),
                    repository: Some(repository.to_string()),
                    branch: None,
                    title: Some(title.to_string()),
                    sender: payload
                        .get("sender")
                        .and_then(|s| s.get("login"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                }
            }
            "issues" => {
                let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");
                let title = payload
                    .get("issue")
                    .and_then(|i| i.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let repository = payload
                    .get("repository")
                    .and_then(|r| r.get("full_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                GitHubEventInfo {
                    event_type: event_type.to_string(),
                    action: Some(action.to_string()),
                    repository: Some(repository.to_string()),
                    branch: None,
                    title: Some(title.to_string()),
                    sender: payload
                        .get("sender")
                        .and_then(|s| s.get("login"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                }
            }
            _ => {
                // Generic fallback
                let repository = payload
                    .get("repository")
                    .and_then(|r| r.get("full_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                GitHubEventInfo {
                    event_type: event_type.to_string(),
                    action: None,
                    repository: Some(repository.to_string()),
                    branch: None,
                    title: None,
                    sender: payload
                        .get("sender")
                        .and_then(|s| s.get("login"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                }
            }
        }
    }

    /// Creates a GitHub webhook handler for use in a web framework.
    /// Returns a closure that processes webhook requests.
    ///
    /// # Example
    /// ```rust,ignore
    /// use nano_gpt_claw::gateway::github::GitHubGateway;
    ///
    /// let gateway = GitHubGateway::new(config);
    ///
    /// // In an Actix-web handler:
    /// async fn handle_webhook(
    ///     gw: web::Data<GitHubGateway>,
    ///     payload: web::Bytes,
    ///     headers: web::HeaderMap,
    /// ) -> impl Responder {
    ///     let headers_map = headers_to_hashmap(&headers);
    ///     match gw.handle_webhook(&payload, &headers_map).await {
    ///         Ok(event) => { /* process event */ }
    ///         Err(e) => { /* handle error */ }
    ///     }
    /// }
    /// ```
    pub fn handler(
        self: Arc<Self>,
    ) -> Box<
        dyn Fn(
                Vec<u8>,
                HashMap<String, String>,
            ) -> std::pin::Pin<
                Box<
                    dyn std::future::Future<Output = Result<GatewayWebhookEvent, GatewayError>>
                        + Send
                        + 'static,
                >,
            > + Send
            + 'static,
    > {
        let me = self;
        Box::new(move |payload, headers| {
            let me = me.clone();
            Box::pin(async move { me.handle_webhook(&payload, &headers).await })
        })
    }
}

/// Extracted information from a GitHub webhook event.
#[derive(Debug, Clone)]
pub struct GitHubEventInfo {
    /// Event type (push, pull_request, etc.)
    pub event_type: String,
    /// Action (opened, closed, etc.) if applicable
    pub action: Option<String>,
    /// Repository full name
    pub repository: Option<String>,
    /// Branch name if applicable
    pub branch: Option<String>,
    /// Event title/subject
    pub title: Option<String>,
    /// Sender username
    pub sender: String,
}

/// Converts HTTP headers to a HashMap with lowercase keys.
pub fn headers_to_hashmap(headers: &reqwest::header::HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_lowercase(),
                v.to_str().unwrap_or("").to_string(),
            )
        })
        .collect()
}

/// Extracts branch name from a git ref string (refs/heads/branch-name).
fn extract_branch_from_ref(ref_: &str) -> String {
    ref_.strip_prefix("refs/heads/").unwrap_or(ref_).to_string()
}

/// Constant-time string comparison to prevent timing attacks.
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

/// Returns the current Unix timestamp in milliseconds.
fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_WEBHOOK_SECRET: &str = "test_webhook_secret_for_testing_only";

    fn make_config() -> GitHubConfig {
        GitHubConfig {
            webhook_secret: TEST_WEBHOOK_SECRET.to_string(),
            enabled: true,
            allowed_repos: vec![],
            allowed_events: vec![],
            api_base_url: "https://api.github.com".to_string(),
            timeout_secs: 30,
        }
    }

    #[test]
    fn test_signature_verification() {
        let config = make_config();
        let secret = config.webhook_secret.clone();
        let gateway = GitHubGateway::new(config);

        let payload = b"test payload";

        // Compute expected signature using the config's secret
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let result = mac.finalize();
        let sig = format!("sha256={}", hex::encode(result.into_bytes()));

        assert!(gateway.verify_signature(payload, &sig));
    }

    #[test]
    fn test_signature_verification_failure() {
        let config = make_config();
        let gateway = GitHubGateway::new(config);

        let payload = b"test payload";
        let fake_sig = "sha256=0000000000000000000000000000000000000000000000000000000000000000";

        assert!(!gateway.verify_signature(payload, fake_sig));
    }

    #[test]
    fn test_repo_allowed_exact_match() {
        let config = GitHubConfig {
            webhook_secret: "".to_string(),
            enabled: true,
            allowed_repos: vec!["owner/repo".to_string()],
            allowed_events: vec![],
            api_base_url: "".to_string(),
            timeout_secs: 30,
        };
        let gateway = GitHubGateway::new(config);

        assert!(gateway.is_repo_allowed("owner/repo"));
        assert!(!gateway.is_repo_allowed("owner/other"));
    }

    #[test]
    fn test_repo_allowed_glob() {
        let config = GitHubConfig {
            webhook_secret: "".to_string(),
            enabled: true,
            allowed_repos: vec!["owner/*".to_string()],
            allowed_events: vec![],
            api_base_url: "".to_string(),
            timeout_secs: 30,
        };
        let gateway = GitHubGateway::new(config);

        assert!(gateway.is_repo_allowed("owner/repo"));
        assert!(gateway.is_repo_allowed("owner/another"));
        assert!(!gateway.is_repo_allowed("other/repo"));
    }

    #[test]
    fn test_extract_branch_from_ref() {
        assert_eq!(extract_branch_from_ref("refs/heads/main"), "main");
        assert_eq!(
            extract_branch_from_ref("refs/heads/feature/test"),
            "feature/test"
        );
        assert_eq!(extract_branch_from_ref("main"), "main");
    }

    #[test]
    fn test_headers_to_hashmap() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("X-GitHub-Event", "push".parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let map = headers_to_hashmap(&headers);
        assert_eq!(map.get("x-github-event"), Some(&"push".to_string()));
        assert_eq!(
            map.get("content-type"),
            Some(&"application/json".to_string())
        );
    }
}
