//! Message Middleware Module - Cross-Terminal Unified Message Middleware
//!
//! Provides unified message routing, format normalization, and
//! cross-terminal context synchronization for CLI, Feishu, and GitHub.

pub mod router;

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use crate::scheduler::Scheduler;

/// Message source terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageSource {
    Cli,
    Feishu,
    GitHub,
}

impl std::fmt::Display for MessageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageSource::Cli => write!(f, "CLI"),
            MessageSource::Feishu => write!(f, "Feishu"),
            MessageSource::GitHub => write!(f, "GitHub"),
        }
    }
}

/// Message priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Message context structure
#[derive(Debug, Clone)]
pub struct MessageContext {
    pub content: String,
    pub source: MessageSource,
    pub user_id: String,
    pub session_id: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

/// Message response structure
#[derive(Debug, Clone)]
pub struct MessageResponse {
    pub content: String,
    pub session_id: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

/// Unified message middleware
#[allow(dead_code)]
pub struct MessageMiddleware {
    sessions: Arc<RwLock<HashMap<String, SessionContext>>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    router: router::MessageRouter,
    scheduler: Arc<Scheduler>,
}

struct SessionContext {
    source: MessageSource,
    last_message: i64,
    message_count: usize,
}

/// Rate limiter state
struct RateLimiter {
    max_per_minute: usize,
    counts: HashMap<String, Vec<i64>>,
}

impl RateLimiter {
    fn new(max_per_minute: usize) -> Self {
        Self {
            max_per_minute,
            counts: HashMap::new(),
        }
    }

    fn is_allowed(&mut self, user_id: &str) -> bool {
        let now = current_timestamp();
        let window = 60;

        let timestamps = self.counts.entry(user_id.to_string()).or_default();
        timestamps.retain(|&t| now - t < window);

        if timestamps.len() >= self.max_per_minute {
            return false;
        }

        timestamps.push(now);
        true
    }
}

impl MessageMiddleware {
    pub fn new(scheduler: Arc<Scheduler>) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new(60))),
            router: router::MessageRouter::new(scheduler.clone()),
            scheduler,
        }
    }

    pub async fn process(&self, ctx: MessageContext) -> Result<MessageResponse, MiddlewareError> {
        let user_id = ctx.user_id.clone();

        if !self.rate_limiter.write().is_allowed(&user_id) {
            warn!("Rate limit exceeded for user: {}", user_id);
            return Err(MiddlewareError::RateLimited);
        }

        self.update_session(&ctx);

        if let Err(e) = self.security_check(&ctx) {
            error!("Security check failed: {}", e);
            return Err(e);
        }

        let response = self.router.route(ctx).await?;

        Ok(response)
    }

    fn update_session(&self, ctx: &MessageContext) {
        let mut sessions = self.sessions.write();

        if let Some(session) = sessions.get_mut(&ctx.session_id) {
            session.last_message = ctx.timestamp;
            session.message_count += 1;
        } else {
            sessions.insert(ctx.session_id.clone(), SessionContext {
                source: ctx.source,
                last_message: ctx.timestamp,
                message_count: 1,
            });
        }
    }

    fn security_check(&self, ctx: &MessageContext) -> Result<(), MiddlewareError> {
        if ctx.content.trim().is_empty() {
            return Err(MiddlewareError::InvalidContent("Empty message".to_string()));
        }

        if ctx.content.len() > 100_000 {
            return Err(MiddlewareError::InvalidContent("Message too long".to_string()));
        }

        Ok(())
    }

    pub fn get_session(&self, session_id: &str) -> Option<SessionInfo> {
        let sessions = self.sessions.read();
        sessions.get(session_id).map(|s| SessionInfo {
            source: s.source,
            last_message: s.last_message,
            message_count: s.message_count,
        })
    }

    pub async fn sync_context(&self, from: MessageSource, to: MessageSource, session_id: &str) -> Result<(), MiddlewareError> {
        info!("Syncing context from {} to {} for session {}", from, to, session_id);
        Ok(())
    }
}

impl Default for MessageMiddleware {
    fn default() -> Self {
        Self::new(Arc::new(Scheduler::new()))
    }
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub source: MessageSource,
    pub last_message: i64,
    pub message_count: usize,
}

#[derive(Debug)]
pub enum MiddlewareError {
    RateLimited,
    InvalidContent(String),
    SecurityViolation,
    RoutingError(String),
    LLMError(String),
}

impl std::fmt::Display for MiddlewareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MiddlewareError::RateLimited => write!(f, "Rate limit exceeded"),
            MiddlewareError::InvalidContent(s) => write!(f, "Invalid content: {}", s),
            MiddlewareError::SecurityViolation => write!(f, "Security violation"),
            MiddlewareError::RoutingError(s) => write!(f, "Routing error: {}", s),
            MiddlewareError::LLMError(s) => write!(f, "LLM error: {}", s),
        }
    }
}

impl std::error::Error for MiddlewareError {}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
