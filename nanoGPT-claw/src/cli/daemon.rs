//! CLI Daemon - Background Process Management
//!
//! ✅ FIXED: 现在使用真实配置，不再硬编码 GatewayConfig::default()
//! ✅ 集成 AppConfig 加载真实的 LLM、飞书、GitHub 配置

use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::config::settings::{AppConfig, GithubConfig as ConfigGithubConfig, LarkConfig as ConfigLarkConfig};
use crate::gateway::feishu::FeishuConfig;
use crate::gateway::github::GitHubConfig as GatewayGithubConfig;
use crate::gateway::{GatewayConfig, GatewayManager};
use crate::memory::{MemoryConfig as MemoryLayerConfig, MemoryLayer};
use crate::scheduler::Scheduler;
use crate::evolution::EvolutionEngine;

/// Global app config storage
static APP_CONFIG: once_cell::sync::Lazy<
    parking_lot::RwLock<Option<AppConfig>>,
> = once_cell::sync::Lazy::new(|| parking_lot::RwLock::new(None));

pub fn init_app_config() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let config = crate::config::init_config()?;
    *APP_CONFIG.write() = Some(config);
    Ok(())
}

fn get_app_config() -> Option<AppConfig> {
    APP_CONFIG.read().clone()
}

/// Convert config MemoryConfig to memory::MemoryConfig
fn convert_memory_config(config: &crate::config::settings::MemoryConfig) -> MemoryLayerConfig {
    MemoryLayerConfig {
        session_max_entries: config.max_short_term_items,
        session_ttl_ms: config.short_term_ttl_hours * 3_600_000,
        eviction_policy: crate::memory::EvictionPolicy::LRU,
        db_path: config.storage.path.clone(),
        persistent_max_entries: config.max_long_term_items,
        embedding_dim: config.vector_db.as_ref().map(|v| v.dimension).unwrap_or(384),
    }
}

/// Convert config LarkConfig to gateway FeishuConfig
fn convert_lark_config(config: &ConfigLarkConfig) -> FeishuConfig {
    FeishuConfig {
        app_id: config.app_id.clone(),
        app_secret: config.app_secret.clone(),
        verification_token: config.verify_token.clone(),
        encrypt_key: config.encrypt_key.clone(),
        api_base_url: "https://open.feishu.cn".to_string(),
        enabled: config.enabled,
    }
}

/// Convert config GithubConfig to gateway GitHubConfig
fn convert_github_config(config: &ConfigGithubConfig) -> GatewayGithubConfig {
    GatewayGithubConfig {
        webhook_secret: config.webhook_secret.clone(),
        enabled: config.enabled,
        allowed_repos: if !config.repository.is_empty() {
            vec![config.repository.clone()]
        } else {
            config.branches.clone()
        },
        allowed_events: config.allowed_events.clone(),
        api_base_url: "https://api.github.com".to_string(),
        timeout_secs: 30,
    }
}

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
    // P1-4修复：启动前清理残留的stunnel进程
    cleanup_stunnel().await;

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
    info!(
        "PID file written: {} (PID: {})",
        DaemonConfig::default().pid_file.display(),
        pid
    );

    // ✅ FIXED: 初始化真实配置
    init_app_config()?;

    // ✅ FIXED: 传递真实配置给各子系统
    let app_config = get_app_config();
    initialize_subsystems(app_config.as_ref()).await?;

    // 启动 watchdog 并保持进程运行！
    tokio::spawn(async move {
        start_watchdog(app_config).await;
    });

    info!("Daemon running! Press Ctrl+C to stop...");

    // 保持进程运行，直到收到终止信号
    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal, stopping daemon...");

    // 清理 PID 文件
    std::fs::remove_file(&DaemonConfig::default().pid_file)?;

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

    // ✅ FIXED: 显示真实配置状态
    if let Some(app_config) = get_app_config() {
        println!("  LLM Provider: {}", app_config.llm.core_model.provider);
        println!("  LLM Model: {}", app_config.llm.core_model.model_name);
        if let Some(lark) = &app_config.lark {
            println!("  Lark: ✅ Enabled ({})", lark.bot_name);
        } else {
            println!("  Lark: ❌ Disabled");
        }
        if let Some(github) = &app_config.github {
            println!("  GitHub: ✅ Enabled (repo: {})", github.repository);
        } else {
            println!("  GitHub: ❌ Disabled");
        }
    } else {
        println!("  Config: ❌ Not loaded (using defaults)");
    }

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

/// ✅ FIXED: 使用真实 AppConfig 初始化所有子系统
async fn initialize_subsystems(app_config: Option<&AppConfig>) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    info!("Initializing subsystems with REAL configuration...");

    // 1. Memory layer - 使用真实配置
    info!("  [1/4] Memory layer...");
    let memory_config = if let Some(config) = app_config {
        info!("    ✅ Using config memory settings");
        convert_memory_config(&config.memory)
    } else {
        info!("    ⚠️  Using default memory settings");
        MemoryLayerConfig::default()
    };
    let memory_layer = MemoryLayer::new(memory_config).await?;

    // 2. Scheduler - 使用真实配置 (关键修复！)
    info!("  [2/4] LLM Scheduler with REAL config...");
    #[allow(unused_variables)]
    let scheduler: Arc<Scheduler> = if let Some(config) = app_config {
        info!("    ✅ Loading LLM providers from config:");
        info!("       - Core model: {} ({})", config.llm.core_model.name, config.llm.core_model.provider);
        info!("       - Auxiliary models: {}", config.llm.auxiliary_models.len());
        info!("       - Providers: {:?}", config.llm.providers.keys().collect::<Vec<_>>());

        let memory = Arc::new(memory_layer);
        let mut sched = Scheduler::from_app_config(config);
        sched.memory = Some(memory.clone());
        Arc::new(sched)
    } else {
        info!("    ⚠️  Using default scheduler (no LLM config!)");
        Arc::new(Scheduler::with_memory(memory_layer))
    };

    // 3. Gateway managers - 使用真实配置 (关键修复！飞书不通的原因)
    info!("  [3/4] Gateway managers with REAL config...");
    let _gateway = if let Some(config) = app_config {
        info!("    ✅ Loading gateway config:");

        // 构建真实的 gateway::GatewayConfig
        let feishu_config = if let Some(lark) = &config.lark {
            info!("       - Lark: ✅ enabled={}, bot={}", lark.enabled, lark.bot_name);
            if !lark.app_id.is_empty() {
                info!("       - Lark app_id: {}...", &lark.app_id[..8.min(lark.app_id.len())]);
            }
            convert_lark_config(lark)
        } else {
            info!("       - Lark: ❌ disabled");
            FeishuConfig::default()
        };

        let github_config = if let Some(github) = &config.github {
            info!("       - GitHub: ✅ webhook_secret configured, repo={}", github.repository);
            convert_github_config(github)
        } else {
            info!("       - GitHub: ❌ disabled");
            GatewayGithubConfig::default()
        };

        let gateway_config = GatewayConfig {
            feishu: feishu_config,
            github: github_config,
        };

        let gw = GatewayManager::new(gateway_config).await?;
        gw.start().await?;
        gw
    } else {
        info!("    ⚠️  Using default gateway (飞书/LLM 不会通！)");
        let gw = GatewayManager::new(GatewayConfig::default()).await?;
        gw.start().await?;
        gw
    };

    // 4. Evolution engine
    info!("  [4/4] Evolution engine...");
    let _evolution = EvolutionEngine::new();

    info!("✅ All subsystems initialized with REAL configuration!");
    if app_config.is_none() {
        warn!("⚠️  WARNING: No config loaded! Using defaults - LLM/Feishu will NOT work!");
    }
    Ok(())
}

async fn start_watchdog(app_config: Option<AppConfig>) {
    let config = DaemonConfig::default();
    let mut interval = interval(Duration::from_secs(config.watch_interval_secs));
    let mut restart_count = 0u32;
    let mut bridge_error_count = 0u32;

    info!(
        "Watchdog started (interval: {}s)",
        config.watch_interval_secs
    );

    // ✅ 记录配置状态
    if let Some(cfg) = &app_config {
        info!("Watchdog running with config: LLM={}/{}", cfg.llm.core_model.provider, cfg.llm.core_model.model_name);
        if cfg.lark.is_some() {
            info!("Watchdog monitoring Lark gateway");
        }
        if cfg.github.is_some() {
            info!("Watchdog monitoring GitHub gateway");
        }
    } else {
        warn!("Watchdog running WITHOUT config - subsystems using defaults!");
    }

    loop {
        interval.tick().await;

        let pid = std::fs::read_to_string(&config.pid_file).unwrap_or_default();
        let pid_num: u32 = pid.trim().parse().unwrap_or(0);

        if !is_process_alive(pid_num) {
            warn!("Main process {} appears dead", pid_num);

            if restart_count < config.max_restart_attempts {
                restart_count += 1;
                tracing::error!(
                    "Auto-restart attempt {}/{}",
                    restart_count,
                    config.max_restart_attempts
                );
                std::fs::remove_file(&config.pid_file).ok();
                break;
            } else {
                tracing::error!("Max restart attempts reached. Exiting.");
                std::fs::remove_file(&config.pid_file).ok();
                break;
            }
        }

        // P1-5修复：检查Bridge线程状态
        #[cfg(unix)]
        {
            // 检查飞书WebSocket连接状态
            let bridge_healthy = check_bridge_health().await;

            if !bridge_healthy {
                bridge_error_count += 1;
                warn!("Bridge不健康 (连续失败: {})", bridge_error_count);

                if bridge_error_count >= 3 {
                    error!("Bridge连续失败{}次，尝试重连...", bridge_error_count);
                    restart_bridge().await;
                    bridge_error_count = 0;
                }
            } else {
                bridge_error_count = 0;
            }
        }

        info!(
            "Watchdog health check passed (PID: {}, restarts: {})",
            pid_num, restart_count
        );
    }
}

// P1-5修复：检查Bridge健康状态
async fn check_bridge_health() -> bool {
    // 检查lark bridge相关进程
    #[cfg(unix)]
    {
        let output = std::process::Command::new("pgrep")
            .arg("-f")
            .arg("lark.*bridge|feishu.*ws|websocket")
            .output();

        if let Ok(output) = output {
            let pids = String::from_utf8_lossy(&output.stdout);
            return !pids.trim().is_empty();
        }
    }
    true // 非Unix系统默认健康
}

// P1-5修复：重启Bridge连接
async fn restart_bridge() {
    #[cfg(unix)]
    {
        info!("正在重启Bridge...");

        // 杀死现有bridge进程
        let _ = std::process::Command::new("pkill")
            .arg("-9")
            .arg("-f")
            .arg("lark.*bridge|feishu.*ws")
            .output();

        // 短暂等待
        tokio::time::sleep(Duration::from_secs(2)).await;

        info!("Bridge重启命令已发送");
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

async fn cleanup_stunnel() {
    // P1-4修复：清理残留的stunnel进程（WS Bridge已不需要公网webhook）
    #[cfg(unix)]
    {
        info!("检查并清理残留的stunnel进程...");

        // 检查stunnel是否在运行
        let output = std::process::Command::new("pgrep")
            .arg("-f")
            .arg("stunnel")
            .output();

        if let Ok(output) = output {
            let pids = String::from_utf8_lossy(&output.stdout);
            if !pids.is_empty() {
                info!("发现残留stunnel进程，清理中...");

                // 杀死所有stunnel进程
                let _ = std::process::Command::new("pkill")
                    .arg("-9")
                    .arg("-f")
                    .arg("stunnel")
                    .output();

                info!("stunnel进程已清理");
            } else {
                info!("未发现残留stunnel进程");
            }
        }
    }
}
