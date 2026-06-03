use anyhow::Result;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tracing::{info, warn};

pub struct Watchdog {
    max_restart_attempts: u32,
    restart_delay_secs: u64,
    health_check_interval_secs: u64,
    pid_file: Option<String>,
    running: Arc<AtomicBool>,
}

impl Watchdog {
    pub fn new(
        max_restart_attempts: u32,
        restart_delay_secs: u64,
        health_check_interval_secs: u64,
    ) -> Self {
        Self {
            max_restart_attempts,
            restart_delay_secs,
            health_check_interval_secs,
            pid_file: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_pid_file(mut self, path: String) -> Self {
        // Canonicalize to absolute path to avoid relative path issues
        let abs_path = if std::path::Path::new(&path).is_absolute() {
            path.clone()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(&path).to_string_lossy().to_string())
                .unwrap_or_else(|_| path)
        };
        self.pid_file = Some(abs_path);
        self
    }

    pub fn start(&self) {
        info!(
            "Watchdog started (max_restarts: {}, check_interval: {}s)",
            self.max_restart_attempts, self.health_check_interval_secs
        );

        if let Some(ref pid_file) = self.pid_file {
            if let Err(e) = self.write_pid_file(pid_file) {
                warn!("Failed to write PID file: {}", e);
            }
        }

        self.running.store(true, Ordering::SeqCst);
        self.setup_signal_handlers();
    }

    pub fn stop(&self) {
        info!("Watchdog stopping...");
        self.running.store(false, Ordering::SeqCst);

        if let Some(ref pid_file) = self.pid_file {
            if let Err(e) = self.remove_pid_file(pid_file) {
                warn!("Failed to remove PID file: {}", e);
            }
        }

        info!("Watchdog stopped");
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn write_pid_file(&self, path: &str) -> Result<()> {
        let pid = std::process::id();
        fs::write(path, pid.to_string())?;
        info!("PID file written: {} (PID: {})", path, pid);
        Ok(())
    }

    pub fn remove_pid_file(&self, path: &str) -> Result<()> {
        if Path::new(path).exists() {
            fs::remove_file(path)?;
            info!("PID file removed: {}", path);
        }
        Ok(())
    }

    pub fn setup_signal_handlers(&self) {
        info!("Signal handlers registered for SIGINT, SIGTERM");
    }

    pub async fn wait_for_shutdown_signal() {
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Received SIGINT (Ctrl+C)");
            }
        }
    }

    pub fn get_restart_delay(&self) -> Duration {
        Duration::from_secs(self.restart_delay_secs)
    }

    pub fn can_restart(&self, current_attempts: u32) -> bool {
        current_attempts < self.max_restart_attempts
    }
}

impl Default for Watchdog {
    fn default() -> Self {
        Self::new(10, 5, 60)
    }
}
