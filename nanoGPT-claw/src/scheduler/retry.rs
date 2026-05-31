//! Retry Utilities with Jittered Exponential Backoff

use std::time::Duration;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

impl RetryConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}

pub struct RetryBackoff {
    config: RetryConfig,
    attempt: u32,
}

impl RetryBackoff {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            attempt: 0,
        }
    }

    pub fn new_default() -> Self {
        Self::new(RetryConfig::default())
    }

    pub fn next_delay(&mut self) -> Option<Duration> {
        if self.attempt >= self.config.max_retries {
            return None;
        }

        let exponential_delay = self.config.initial_delay_ms as f64
            * self.config.backoff_multiplier.powi(self.attempt as i32);
        
        let capped_delay = exponential_delay.min(self.config.max_delay_ms as f64);
        
        // Simple deterministic jitter (no RNG needed)
        let jitter_range = capped_delay * self.config.jitter_factor;
        let jitter = (self.attempt as f64 * 0.13 + 0.07) * jitter_range - jitter_range / 2.0;
        
        let final_delay_ms = (capped_delay + jitter).max(0.0) as u64;
        
        self.attempt += 1;
        
        Some(Duration::from_millis(final_delay_ms))
    }

    pub fn reset(&mut self) {
        self.attempt = 0;
    }

    pub fn attempt(&self) -> u32 {
        self.attempt
    }
}

pub async fn retry_with_backoff<F, Fut, T, E, ShouldRetry>(
    config: RetryConfig,
    operation: F,
    should_retry: ShouldRetry,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    ShouldRetry: Fn(&E) -> bool,
{
    let mut backoff = RetryBackoff::new(config);

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if !should_retry(&error) {
                    debug!("Error not retryable, aborting");
                    return Err(error);
                }

                if let Some(delay) = backoff.next_delay() {
                    warn!(
                        "Attempt {} failed, retrying in {:?}",
                        backoff.attempt(),
                        delay
                    );
                    tokio::time::sleep(delay).await;
                } else {
                    warn!("Max retries reached, aborting");
                    return Err(error);
                }
            }
        }
    }
}
