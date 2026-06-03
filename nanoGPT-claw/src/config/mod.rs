pub mod env;
pub mod env_config;
pub mod settings;

use anyhow::Result;
use dirs;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::path::PathBuf;
use tracing::info;

pub use env::{interpolate_env_vars, EnvInterpolate};
pub use settings::*;
pub use env_config::{EnvConfig, LlmEnvConfig, LarkEnvConfig, GithubEnvConfig};
pub use settings::{LarkConfig, GithubConfig, ProviderConfig};

pub static APP_CONFIG: Lazy<RwLock<AppConfig>> = Lazy::new(|| RwLock::new(AppConfig::default()));

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nanoGPT-claw")
}

pub fn get_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nanoGPT-claw")
}

pub fn get_log_dir() -> PathBuf {
    get_config_dir().join("logs")
}

pub fn get_memory_dir() -> PathBuf {
    get_data_dir().join("memory")
}

pub fn init_config() -> Result<AppConfig> {
    let config_dir = get_config_dir();
    std::fs::create_dir_all(&config_dir)?;

    let config_path = config_dir.join("config.yaml");
    let config = if config_path.exists() {
        info!("Loading existing configuration from {:?}", config_path);
        AppConfig::load_from_file(&config_path)?
    } else {
        info!("Creating default configuration at {:?}", config_path);
        let default_config = AppConfig::default();
        default_config.save_to_file(&config_path)?;
        default_config
    };

    // ✅ 关键修复：合并环境变量配置
    let env_config = EnvConfig::load();
    let config = merge_env_config(config, &env_config);

    *APP_CONFIG.write() = config.clone();
    Ok(config)
}

pub fn merge_env_config(mut app_config: AppConfig, env_config: &EnvConfig) -> AppConfig {
    if let Some(ref api_key) = env_config.llm.openai_api_key {
        if let Some(provider) = app_config.llm.providers.get_mut("openai") {
            provider.api_key = api_key.clone();
        }
        app_config.llm.core_model.api_key = api_key.clone();
        info!("LLM: OpenAI API key loaded from environment");
    }

    if let Some(ref api_key) = env_config.llm.anthropic_api_key {
        if let Some(provider) = app_config.llm.providers.get_mut("anthropic") {
            provider.api_key = api_key.clone();
        }
        info!("LLM: Anthropic API key loaded from environment");
    }

    if let Some(ref base_url) = env_config.llm.ollama_base_url {
        app_config.llm.providers.insert("ollama".to_string(), ProviderConfig {
            enabled: true,
            api_key: String::new(),
            base_url: base_url.clone(),
            default_model: "llama2".to_string(),
            max_tokens: Some(2048),
            temperature: Some(0.7),
            top_p: Some(0.9),
            timeout_secs: Some(120),
            priority: 1,
        });
        info!("LLM: Ollama configured from environment");
    }

    if env_config.has_any_lark_config() {
        let lark = app_config.lark.take().unwrap_or_default();
        app_config.lark = Some(LarkConfig {
            enabled: true,
            app_id: env_config.lark.app_id.clone().unwrap_or_default(),
            app_secret: env_config.lark.app_secret.clone().unwrap_or_default(),
            bot_name: lark.bot_name.clone(),
            webhook_url: lark.webhook_url.clone(),
            verify_token: env_config.lark.verify_token.clone().unwrap_or_default(),
            encrypt_key: env_config.lark.encrypt_key.clone(),
            event_callback_url: lark.event_callback_url.clone(),
            auto_reply: lark.auto_reply,
            allowed_groups: lark.allowed_groups.clone(),
        });
        info!("Lark: Configuration loaded from environment");
    }

    if env_config.has_github_config() {
        let github = app_config.github.take().unwrap_or_default();
        app_config.github = Some(GithubConfig {
            enabled: true,
            webhook_secret: env_config.github.webhook_secret.clone().unwrap_or_default(),
            api_token: env_config.github.api_token.clone().unwrap_or_default(),
            app_id: github.app_id.clone(),
            private_key: github.private_key.clone(),
            repository: github.repository.clone(),
            auto_scan_enabled: github.auto_scan_enabled,
            scan_interval_hours: github.scan_interval_hours,
            auto_commit_enabled: github.auto_commit_enabled,
            auto_pr_enabled: github.auto_pr_enabled,
            branches: github.branches.clone(),
            allowed_events: github.allowed_events.clone(),
        });
        info!("GitHub: Configuration loaded from environment");
    }

    app_config
}

pub fn reload_config() -> Result<AppConfig> {
    let config_dir = get_config_dir();
    let config_path = config_dir.join("config.yaml");
    let config = AppConfig::load_from_file(&config_path)?;
    *APP_CONFIG.write() = config.clone();
    Ok(config)
}
