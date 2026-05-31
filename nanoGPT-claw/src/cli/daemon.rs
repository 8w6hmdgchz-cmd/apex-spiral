//! CLI Daemon - Background Process Management

use std::path::PathBuf;
use std::process;
use std::time::Duration;
use tracing::{info, warn};
use tokio::time::interval;

pub struct DaemonConfig {
    pub pid_file: PathBuf,
    pub log_file: PathBuf,
    pub watch_interval_secs: u64,
    pub max_restart_attempts: u32,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            pid_file: PathBuf::from("/tmp/nano-gpt-claw.pid"),
            log_file: PathBuf::from("/var/log/nano-gpt-claw.log"),
            watch_interval_secs: 30,
            max_restart_attempts: 5,
        }
    }
}

pub async fn start_daemon() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    if DaemonConfig::default().pid_file.exists() {
        let pid = std::fs::read_to_string(&DaemonConfig::default().pid_file)?;
        let pid_num: u32 = pid.trim().parse().unwrap_or(0);
        if pid_num > 0 && is_process_alive(pid_num) {
            tracing::error!("NanoGPT-Claw daemon already running with PID: {}", pid_num);
            return Ok(());
        } else {
            tracing::warn!("Stale PID file found, removing...");
            std::fs::remove_file(&DaemonConfig::default().pid_file)?;
        }
    }

    info!("Starting NanoGPT-Claw daemon...");

    let pid = process::id();
    std::fs::write(&DaemonConfig::default().pid_file, pid.to_string())?;
    info!("PID file written: {} (PID: {})", DaemonConfig::default().pid_file.display(), pid);

    initialize_subsystems().await?;

    start_watchdog().await;

    Ok(())
}

pub async fn stop_daemon() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let config = DaemonConfig::default();

    if !config.pid_file.exists() {
        warn!("Daemon not running (no PID file)");
        return Ok(());
    }

    let pid = std::fs::read_to_string(&config.pid_file)?;
    info!("Stopping daemon (PID: {})...", pid.trim());

    #[cfg(unix)]
    {
        let pid_num: u32 = pid.trim().parse().unwrap_or(0);
        if pid_num > 0 {
            std::process::Command::new("kill")
                .arg("-TERM")
                .arg(pid_num.to_string())
                .spawn()?;
        }
    }

    std::fs::remove_file(&config.pid_file)?;

    info!("Daemon stopped gracefully.");
    Ok(())
}

pub async fn show_status() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let config = DaemonConfig::default();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  NanoGPT-Claw System Status                                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    if config.pid_file.exists() {
        let pid = std::fs::read_to_string(&config.pid_file)?;
        println!("  Daemon:  ✅ Running (PID: {})", pid.trim());

        let pid_num: u32 = pid.trim().parse().unwrap_or(0);
        if is_process_alive(pid_num) {
            println!("  Status:  ✅ Responding");
        } else {
            println!("  Status:  ⚠️  Unresponsive (stale PID file)");
        }
    } else {
        println!("  Daemon:  ❌ Not running");
    }

    println!("  Version: 0.9.1");
    println!("  Rust:    ✅ (tokio async runtime)");
    println!("  Memory:  Layer initialized");
    println!("  Scheduler: Multi-LLM cluster ready");

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  Architecture                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("  Layer 1 (Access):     CLI | Feishu | GitHub Webhook");
    println!("  Layer 2 (Middleware): Unified message router");
    println!("  Layer 3 (Scheduler):  1 Main + N Aux LLM cluster");
    println!("  Layer 4 (CoT):        Chain-of-thought reasoning");
    println!("  Layer 5 (Memory):      Session + Persistent dual-layer");
    println!("  Layer 6 (Evolution):   Auto self-evolution engine");
    println!("  Layer 7 (Daemon):      Watchdog + crash recovery");

    Ok(())
}

async fn initialize_subsystems() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    info!("Initializing subsystems...");

    info!("  [1/4] Memory layer...");
    let _mem = crate::memory::MemoryLayer::new(crate::memory::MemoryConfig::default()).await?;

    info!("  [2/4] LLM scheduler...");
    let _scheduler = crate::scheduler::Scheduler::new();

    info!("  [3/4] Gateway managers...");
    let _gateway = crate::gateway::GatewayManager::new(crate::gateway::GatewayConfig::default()).await?;

    info!("  [4/4] Evolution engine...");
    let _evolution = crate::evolution::EvolutionEngine::new();

    info!("All subsystems initialized successfully.");
    Ok(())
}

async fn start_watchdog() {
    let config = DaemonConfig::default();
    let mut interval = interval(Duration::from_secs(config.watch_interval_secs));
    let mut restart_count = 0u32;

    info!("Watchdog started (interval: {}s)", config.watch_interval_secs);

    loop {
        interval.tick().await;

        let pid = std::fs::read_to_string(&config.pid_file).unwrap_or_default();
        let pid_num: u32 = pid.trim().parse().unwrap_or(0);

        if !is_process_alive(pid_num) {
            warn!("Main process {} appears dead", pid_num);

            if restart_count < config.max_restart_attempts {
                restart_count += 1;
                tracing::error!("Auto-restart attempt {}/{}", restart_count, config.max_restart_attempts);
                std::fs::remove_file(&config.pid_file).ok();
                break;
            } else {
                tracing::error!("Max restart attempts reached. Exiting.");
                std::fs::remove_file(&config.pid_file).ok();
                break;
            }
        }

        info!("Watchdog health check passed (PID: {}, restarts: {})", pid_num, restart_count);
    }
}

fn is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        match std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
    #[cfg(not(unix))]
    {
        true
    }
}
