use crate::message_middle::types::UnifiedMessage;
use anyhow::Result;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use regex::Regex;
use tracing::{debug, warn};

type HmacSha256 = Hmac<Sha256>;

pub struct SecurityFilter {
    blocked_patterns: Vec<String>,
    max_content_length: usize,
}

impl SecurityFilter {
    pub fn new() -> Self {
        Self {
            blocked_patterns: vec![
                r"(?i)(sql|inject)".to_string(),
                r"(?i)(exec|eval)".to_string(),
                r"(?i)(rm\s+-rf|mkfs)".to_string(),
            ],
            max_content_length: 100_000,
        }
    }

    pub fn verify_signature(&self, payload: &str, signature: &str, secret: &str) -> bool {
        let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return false,
        };
        
        mac.update(payload.as_bytes());
        
        let expected = hex::encode(mac.finalize().into_bytes());
        let signature_clean = signature.trim_start_matches("sha256=");
        
        timing_safe_eq(&expected, signature_clean)
    }

    pub fn verify_github_signature(&self, payload: &[u8], signature: &str, secret: &str) -> bool {
        let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return false,
        };
        
        mac.update(payload);
        
        let expected = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        
        debug!("Verifying GitHub signature");
        timing_safe_eq(&expected, signature)
    }

    pub fn verify_lark_signature(&self, encrypt_key: &str, timestamp: &str, nonce: &str, signature: &str) -> bool {
        if encrypt_key.is_empty() || signature.is_empty() {
            warn!("Empty encrypt_key or signature provided");
            return false;
        }

        let message = format!("{}{}{}", timestamp, nonce, encrypt_key);
        
        let mut mac = match HmacSha256::new_from_slice(b"lark") {
            Ok(mac) => mac,
            Err(_) => return false,
        };
        
        mac.update(message.as_bytes());
        
        let expected = hex::encode(mac.finalize().into_bytes());
        
        debug!("Verifying Lark signature");
        timing_safe_eq(&expected, signature)
    }

    pub fn filter_message(&self, message: &UnifiedMessage) -> Result<FilterResult> {
        if message.content.len() > self.max_content_length {
            return Ok(FilterResult::Blocked { 
                reason: format!("Content exceeds max length: {} > {}", 
                    message.content.len(), self.max_content_length) 
            });
        }

        for pattern in &self.blocked_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(&message.content) {
                    return Ok(FilterResult::Blocked {
                        reason: format!("Content matches blocked pattern: {}", pattern),
                    });
                }
            }
        }

        Ok(FilterResult::Allowed)
    }
}

fn timing_safe_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    
    let mut result = 0u8;
    for i in 0..a_bytes.len() {
        result |= a_bytes[i] ^ b_bytes[i];
    }
    
    result == 0
}

pub struct RateLimiter {
    requests: Mutex<HashMap<String, VecDeque<Instant>>>,
    max_requests: usize,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            max_requests,
            window_secs,
        }
    }

    pub fn check_rate_limit(&self, key: &str) -> RateLimitResult {
        let mut requests = self.requests.lock();
        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);
        
        let timestamps = requests.entry(key.to_string()).or_insert_with(VecDeque::new);
        
        while let Some(oldest) = timestamps.front() {
            if now.duration_since(*oldest) > window {
                timestamps.pop_front();
            } else {
                break;
            }
        }
        
        let current_count = timestamps.len();
        
        if current_count >= self.max_requests {
            let retry_after = if let Some(oldest) = timestamps.front() {
                let remaining = window.saturating_sub(now.duration_since(*oldest));
                remaining.as_secs()
            } else {
                self.window_secs
            };
            
            debug!("Rate limit exceeded for key: {}", key);
            return RateLimitResult::Limited { 
                retry_after_secs: retry_after 
            };
        }
        
        timestamps.push_back(now);
        
        RateLimitResult::Allowed {
            remaining: self.max_requests - timestamps.len(),
            reset_in_secs: self.window_secs,
        }
    }

    pub fn cleanup(&self) {
        let mut requests = self.requests.lock();
        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);
        
        for timestamps in requests.values_mut() {
            while let Some(oldest) = timestamps.front() {
                if now.duration_since(*oldest) > window {
                    timestamps.pop_front();
                } else {
                    break;
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum FilterResult {
    Allowed,
    Blocked { reason: String },
}

#[derive(Debug)]
pub enum RateLimitResult {
    Allowed { remaining: usize, reset_in_secs: u64 },
    Limited { retry_after_secs: u64 },
}

impl Default for SecurityFilter {
    fn default() -> Self {
        Self::new()
    }
}
