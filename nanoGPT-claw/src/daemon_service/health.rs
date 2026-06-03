use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub overall_status: HealthStatus,
    pub timestamp: DateTime<Utc>,
    pub uptime_secs: u64,
    pub restart_count: u32,
    pub components: ComponentHealth,
    pub metrics: HealthMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub memory_layer: ComponentStatus,
    pub llm_scheduler: ComponentStatus,
    pub think_engine: ComponentStatus,
    pub message_middleware: ComponentStatus,
    pub lark_gateway: ComponentStatus,
    pub github_gateway: ComponentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComponentStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub memory_usage_mb: Option<f64>,
    pub cpu_usage_percent: Option<f64>,
}

#[allow(dead_code)]
pub struct HealthMonitor {
    check_interval_secs: u64,
    start_time: chrono::DateTime<Utc>,
    restart_count: u32,
    total_requests: Arc<AtomicU64>,
    successful_requests: Arc<AtomicU64>,
    failed_requests: Arc<AtomicU64>,
}

impl HealthMonitor {
    pub fn new(check_interval_secs: u64) -> Self {
        Self {
            check_interval_secs,
            start_time: Utc::now(),
            restart_count: 0,
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_request(&self, success: bool) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn get_health_report(&self) -> HealthReport {
        let uptime_secs = (Utc::now() - self.start_time).num_seconds() as u64;
        let total = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);

        let error_rate = if total > 0 {
            failed as f64 / total as f64
        } else {
            0.0
        };

        let overall_status = if error_rate > 0.1 {
            HealthStatus::Unhealthy
        } else if error_rate > 0.05 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        HealthReport {
            overall_status,
            timestamp: Utc::now(),
            uptime_secs,
            restart_count: self.restart_count,
            components: ComponentHealth {
                memory_layer: ComponentStatus::Healthy,
                llm_scheduler: ComponentStatus::Healthy,
                think_engine: ComponentStatus::Healthy,
                message_middleware: ComponentStatus::Healthy,
                lark_gateway: ComponentStatus::Disabled,
                github_gateway: ComponentStatus::Disabled,
            },
            metrics: HealthMetrics {
                total_requests: total,
                successful_requests: successful,
                failed_requests: failed,
                avg_response_time_ms: 100.0,
                memory_usage_mb: None,
                cpu_usage_percent: None,
            },
        }
    }

    pub fn record_restart(&mut self) {
        self.restart_count += 1;
        self.start_time = Utc::now();
    }

    pub fn get_uptime(&self) -> chrono::Duration {
        Utc::now() - self.start_time
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new(60)
    }
}

impl HealthReport {
    pub fn is_healthy(&self) -> bool {
        self.overall_status == HealthStatus::Healthy
    }

    pub fn summary(&self) -> String {
        format!(
            "Status: {:?} | Uptime: {}s | Requests: {} ({} failed) | Components: memory={:?}, llm={:?}, think={:?}",
            self.overall_status,
            self.uptime_secs,
            self.metrics.total_requests,
            self.metrics.failed_requests,
            self.components.memory_layer,
            self.components.llm_scheduler,
            self.components.think_engine,
        )
    }
}
