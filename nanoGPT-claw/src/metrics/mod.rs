//! # Metrics Module - System Monitoring and Observability
//!
//! ✅ Prometheus 格式导出（简化版）
//! ✅ 关键业务指标追踪
//! ✅ 性能指标监控

use std::sync::Arc;
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct Metrics {
    requests_total: Arc<std::sync::atomic::AtomicU64>,
    requests_success: Arc<std::sync::atomic::AtomicU64>,
    requests_failed: Arc<std::sync::atomic::AtomicU64>,
    active_requests: Arc<std::sync::atomic::AtomicU64>,
    cache_hits: Arc<std::sync::atomic::AtomicU64>,
    cache_misses: Arc<std::sync::atomic::AtomicU64>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            requests_total: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            requests_success: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            requests_failed: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            active_requests: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            cache_hits: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            cache_misses: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub fn record_request(&self) {
        self.requests_total.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.active_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn record_request_success(&self) {
        self.requests_success.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.active_requests.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn record_request_failed(&self) {
        self.requests_failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.active_requests.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn requests_total(&self) -> u64 {
        self.requests_total.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn requests_success(&self) -> u64 {
        self.requests_success.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn requests_failed(&self) -> u64 {
        self.requests_failed.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn active_requests(&self) -> u64 {
        self.active_requests.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn cache_misses(&self) -> u64 {
        self.cache_misses.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits();
        let misses = self.cache_misses();
        if hits + misses == 0 {
            0.0
        } else {
            hits as f64 / (hits + misses) as f64
        }
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.requests_total();
        if total == 0 {
            0.0
        } else {
            self.requests_success() as f64 / total as f64
        }
    }

    pub fn to_prometheus(&self) -> String {
        format!(
            "# HELP nano_requests_total Total HTTP requests\n\
             # TYPE nano_requests_total counter\n\
             nano_requests_total {}\n\n\
             # HELP nano_requests_success Successful HTTP requests\n\
             # TYPE nano_requests_success counter\n\
             nano_requests_success {}\n\n\
             # HELP nano_requests_failed Failed HTTP requests\n\
             # TYPE nano_requests_failed counter\n\
             nano_requests_failed {}\n\n\
             # HELP nano_active_requests Currently active HTTP requests\n\
             # TYPE nano_active_requests gauge\n\
             nano_active_requests {}\n\n\
             # HELP nano_cache_hits Cache hits\n\
             # TYPE nano_cache_hits counter\n\
             nano_cache_hits {}\n\n\
             # HELP nano_cache_misses Cache misses\n\
             # TYPE nano_cache_misses counter\n\
             nano_cache_misses {}",
            self.requests_total(),
            self.requests_success(),
            self.requests_failed(),
            self.active_requests(),
            self.cache_hits(),
            self.cache_misses()
        )
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

pub static GLOBAL_METRICS: Lazy<Metrics> = Lazy::new(Metrics::new);
