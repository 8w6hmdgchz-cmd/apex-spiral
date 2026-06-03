pub mod health;
pub mod task_queue;
pub mod watchdog;

pub use task_queue::{Task, TaskQueue, TaskStatus, TaskType, TaskWorker};

use anyhow::Result;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonState {
    pub running: bool,
    pub start_time: Option<DateTime<Utc>>,
    pub restart_count: u32,
    pub last_error: Option<String>,
    pub uptime_secs: u64,
}

impl DaemonState {
    pub fn new() -> Self {
        Self {
            running: false,
            start_time: None,
            restart_count: 0,
            last_error: None,
            uptime_secs: 0,
        }
    }

    pub fn update_uptime(&mut self) {
        if let Some(start) = self.start_time {
            self.uptime_secs = (Utc::now() - start).num_seconds() as u64;
        }
    }
}

impl Default for DaemonState {
    fn default() -> Self {
        Self::new()
    }
}

static DAEMON_STATE: once_cell::sync::Lazy<Arc<RwLock<DaemonState>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(DaemonState::new())));

static DAEMON_RUNNING: AtomicBool = AtomicBool::new(false);

pub fn get_daemon_state() -> DaemonState {
    let mut state = DAEMON_STATE.read().clone();
    state.update_uptime();
    state
}

pub fn start_daemon() -> Result<()> {
    info!("Starting NanoGPT-Claw daemon...");
    let mut state = DAEMON_STATE.write();
    state.running = true;
    state.start_time = Some(Utc::now());
    state.uptime_secs = 0;
    DAEMON_RUNNING.store(true, Ordering::SeqCst);
    info!("Daemon started successfully");
    Ok(())
}

pub fn stop_daemon() -> Result<()> {
    info!("Stopping NanoGPT-Claw daemon...");
    let mut state = DAEMON_STATE.write();
    state.running = false;
    DAEMON_RUNNING.store(false, Ordering::SeqCst);
    info!("Daemon stopped");
    Ok(())
}

pub fn is_daemon_running() -> bool {
    DAEMON_RUNNING.load(Ordering::SeqCst)
}

pub fn record_restart() {
    let mut state = DAEMON_STATE.write();
    state.restart_count += 1;
    state.start_time = Some(Utc::now());
    info!("Daemon restarted (count: {})", state.restart_count);
}

pub fn record_error(error: String) {
    let mut state = DAEMON_STATE.write();
    state.last_error = Some(error);
}

pub fn get_uptime() -> std::time::Duration {
    let state = DAEMON_STATE.read();
    if let Some(start) = state.start_time {
        let duration = Utc::now() - start;
        std::time::Duration::from_secs(duration.num_seconds() as u64)
    } else {
        std::time::Duration::from_secs(0)
    }
}

pub fn format_uptime() -> String {
    let duration = get_uptime();
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
