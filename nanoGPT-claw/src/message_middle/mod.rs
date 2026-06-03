pub mod types;
pub mod router;
pub mod filter;

use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use parking_lot::RwLock;
use tracing::info;

pub use types::*;
pub use router::MessageRouter;
pub use filter::{SecurityFilter, RateLimiter, FilterResult, RateLimitResult};

pub static MESSAGE_ROUTER: once_cell::sync::Lazy<Arc<MessageRouter>> = 
    once_cell::sync::Lazy::new(|| Arc::new(MessageRouter::new()));

pub static SECURITY_FILTER: once_cell::sync::Lazy<Arc<SecurityFilter>> =
    once_cell::sync::Lazy::new(|| Arc::new(SecurityFilter::new()));

pub static RATE_LIMITER: once_cell::sync::Lazy<Arc<RateLimiter>> =
    once_cell::sync::Lazy::new(|| Arc::new(RateLimiter::new(100, 60)));

pub fn init_middleware() -> Result<()> {
    info!("Initializing message middleware...");
    info!("Security filter initialized");
    info!("Rate limiter configured: 100 requests per 60 seconds");
    info!("Message router initialized");
    Ok(())
}

pub fn route_message(message: UnifiedMessage) -> Result<RouteResult> {
    MESSAGE_ROUTER.route(message)
}

pub fn verify_github_signature(payload: &[u8], signature: &str, secret: &str) -> bool {
    SECURITY_FILTER.verify_github_signature(payload, signature, secret)
}

pub fn verify_lark_signature(encrypt_key: &str, timestamp: &str, nonce: &str, signature: &str) -> bool {
    SECURITY_FILTER.verify_lark_signature(encrypt_key, timestamp, nonce, signature)
}

pub fn check_rate_limit(key: &str) -> RateLimitResult {
    RATE_LIMITER.check_rate_limit(key)
}
