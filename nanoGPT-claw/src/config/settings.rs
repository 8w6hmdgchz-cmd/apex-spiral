use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, warn};

use super::env::interpolate_env_vars;

/// Main application configuration - aligned with Hermes-Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub system: SystemConfig,
    pub cli: CliConfig,
    pub llm: LlmConfig,
    pub memory: MemoryConfig,
    pub skills: SkillsConfig,
    pub lark: Option<LarkConfig>,
    pub github: Option<GithubConfig>,
    pub gateway: GatewayConfig,
    pub daemon: DaemonConfig,
    pub logging: LoggingConfig,
    pub optimization: OptimizationConfig,
    pub safety: SafetyConfig,
    pub plugins: PluginsConfig,
}

/// System configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub name: String,
    pub version: String,
    pub mode: String,
    pub max_concurrent_tasks: usize,
    pub language: String,
    pub timezone: String,
}

/// CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub prompt: String,
    pub history_size: usize,
    pub echo: bool,
    pub color_output: bool,
    pub auto_save: bool,
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub core_model: ModelConfig,
    pub auxiliary_models: Vec<ModelConfig>,
    pub providers: HashMap<String, ProviderConfig>,
    pub request_timeout_secs: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub retry_backoff_multiplier: f64,
    pub retry_jitter_factor: f64,
    pub fallback_strategy: FallbackStrategy,
    pub cache_enabled: bool,
    pub cache_ttl_secs: u64,
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub enabled: bool,
    pub api_key: String,
    pub base_url: String,
    pub default_model: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub timeout_secs: Option<u64>,
    pub priority: u32,
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub role: ModelRole,
    pub provider: String,
    pub api_key: String,
    pub base_url: String,
    pub model_name: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub system_prompt: Option<String>,
}

/// Model role enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelRole {
    Core,
    CodeAudit,
    LogicReview,
    VulnerabilityDetect,
    FrameworkBenchmark,
    KnowledgeRetrieval,
    Planning,
    Reflection,
}

/// Fallback strategy for LLM errors
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackStrategy {
    Retry,
    FallbackModel,
    FallbackProvider,
    GracefulDegradation,
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub short_term_ttl_hours: u64,
    pub long_term_enabled: bool,
    pub similarity_threshold: f32,
    pub max_short_term_items: usize,
    pub max_long_term_items: usize,
    pub cleanup_interval_hours: u64,
    pub storage: StorageConfig,
    pub vector_db: Option<VectorDbConfig>,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend: String,
    pub path: String,
    pub encryption_enabled: bool,
}

/// Vector database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDbConfig {
    pub enabled: bool,
    pub provider: String,
    pub url: String,
    pub api_key: String,
    pub index_name: String,
    pub dimension: usize,
}

/// Skills configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsConfig {
    pub enabled: bool,
    pub auto_discover: bool,
    pub skill_paths: Vec<String>,
    pub enabled_skills: Vec<String>,
    pub skill_settings: HashMap<String, SkillSetting>,
}

/// Individual skill setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSetting {
    pub enabled: bool,
    pub parameters: HashMap<String, String>,
}

/// Gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub http_enabled: bool,
    pub http_port: u16,
    pub http_host: String,
    pub websocket_enabled: bool,
    pub websocket_port: u16,
    pub cors_origins: Vec<String>,
    pub rate_limit_enabled: bool,
    pub rate_limit_requests: u32,
    pub rate_limit_window_secs: u64,
}

/// Lark/Feishu configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LarkConfig {
    pub enabled: bool,
    pub app_id: String,
    pub app_secret: String,
    pub bot_name: String,
    pub webhook_url: String,
    pub verify_token: String,
    pub encrypt_key: Option<String>,
    pub event_callback_url: String,
    pub auto_reply: bool,
    pub allowed_groups: Vec<String>,
}

/// GitHub configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubConfig {
    pub enabled: bool,
    pub webhook_secret: String,
    pub app_id: String,
    pub private_key: String,
    pub repository: String,
    pub auto_scan_enabled: bool,
    pub scan_interval_hours: u64,
    pub auto_commit_enabled: bool,
    pub auto_pr_enabled: bool,
    pub branches: Vec<String>,
    pub allowed_events: Vec<String>,
}

/// Daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub pid_file: String,
    pub restart_delay_secs: u64,
    pub max_restart_attempts: u32,
    pub health_check_interval_secs: u64,
    pub graceful_shutdown_timeout_secs: u64,
    pub watch_dog_enabled: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_enabled: bool,
    pub file_path: String,
    pub console_enabled: bool,
    pub max_file_size_mb: u64,
    pub max_files: usize,
    pub format: String,
    pub include_timestamps: bool,
    pub include_module_path: bool,
}

/// Optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub auto_evolution_enabled: bool,
    pub evolution_interval_hours: u64,
    pub performance_monitoring_enabled: bool,
    pub benchmark_on_startup: bool,
    pub auto_optimize_prompts: bool,
    pub prompt_learning_rate: f32,
}

/// Safety configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub content_filter_enabled: bool,
    pub content_filter_level: String,
    pub pii_detection_enabled: bool,
    pub pii_redaction_enabled: bool,
    pub rate_limiting_enabled: bool,
    pub max_session_length_minutes: u64,
    pub allowed_domains: Vec<String>,
}

/// Plugins configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginsConfig {
    pub enabled: bool,
    pub plugin_dir: String,
    pub auto_load: bool,
    pub enabled_plugins: Vec<String>,
    pub plugin_settings: HashMap<String, HashMap<String, String>>,
}

impl Default for AppConfig {
    fn default() -> Self {
        use std::collections::HashMap;
        
        Self {
            system: SystemConfig {
                name: "NanoGPT-Claw".to_string(),
                version: "0.9.0".to_string(),
                mode: "daemon".to_string(),
                max_concurrent_tasks: 5,
                language: "en".to_string(),
                timezone: "UTC".to_string(),
            },
            cli: CliConfig {
                prompt: "nanogpt> ".to_string(),
                history_size: 100,
                echo: true,
                color_output: true,
                auto_save: true,
            },
            llm: LlmConfig {
                core_model: ModelConfig {
                    id: "core-1".to_string(),
                    name: "Core Main LLM".to_string(),
                    role: ModelRole::Core,
                    provider: "openai".to_string(),
                    api_key: "".to_string(),
                    base_url: "https://api.openai.com/v1".to_string(),
                    model_name: "gpt-4o".to_string(),
                    max_tokens: 4096,
                    temperature: 0.7,
                    top_p: 0.9,
                    system_prompt: None,
                },
                auxiliary_models: vec![
                    ModelConfig {
                        id: "aux-code-1".to_string(),
                        name: "Code Auditor".to_string(),
                        role: ModelRole::CodeAudit,
                        provider: "openai".to_string(),
                        api_key: "".to_string(),
                        base_url: "https://api.openai.com/v1".to_string(),
                        model_name: "gpt-3.5-turbo".to_string(),
                        max_tokens: 2048,
                        temperature: 0.3,
                        top_p: 0.85,
                        system_prompt: None,
                    },
                ],
                providers: {
                    let mut providers = HashMap::new();
                    providers.insert("openai".to_string(), ProviderConfig {
                        enabled: true,
                        api_key: "${OPENAI_API_KEY}".to_string(),
                        base_url: "https://api.openai.com/v1".to_string(),
                        default_model: "gpt-4o".to_string(),
                        max_tokens: Some(4096),
                        temperature: Some(0.7),
                        top_p: Some(0.9),
                        timeout_secs: Some(120),
                        priority: 1,
                    });
                    providers.insert("anthropic".to_string(), ProviderConfig {
                        enabled: false,
                        api_key: "${ANTHROPIC_API_KEY}".to_string(),
                        base_url: "https://api.anthropic.com/v1".to_string(),
                        default_model: "claude-3-opus".to_string(),
                        max_tokens: Some(4096),
                        temperature: Some(0.7),
                        top_p: Some(0.9),
                        timeout_secs: Some(120),
                        priority: 2,
                    });
                    providers
                },
                request_timeout_secs: 120,
                max_retries: 3,
                retry_delay_ms: 1000,
                retry_backoff_multiplier: 2.0,
                retry_jitter_factor: 0.1,
                fallback_strategy: FallbackStrategy::FallbackProvider,
                cache_enabled: true,
                cache_ttl_secs: 3600,
            },
            memory: MemoryConfig {
                short_term_ttl_hours: 24,
                long_term_enabled: true,
                similarity_threshold: 0.75,
                max_short_term_items: 1000,
                max_long_term_items: 10000,
                cleanup_interval_hours: 6,
                storage: StorageConfig {
                    backend: "sqlite".to_string(),
                    path: "./data/memory.db".to_string(),
                    encryption_enabled: false,
                },
                vector_db: None,
            },
            skills: SkillsConfig {
                enabled: true,
                auto_discover: true,
                skill_paths: vec!["./skills".to_string()],
                enabled_skills: vec!["echo".to_string()],
                skill_settings: HashMap::new(),
            },
            lark: Some(LarkConfig {
                enabled: false,
                app_id: "${FEISHU_APP_ID}".to_string(),
                app_secret: "${FEISHU_APP_SECRET}".to_string(),
                bot_name: "NanoGPT-Claw Bot".to_string(),
                webhook_url: "".to_string(),
                verify_token: "${FEISHU_VERIFY_TOKEN}".to_string(),
                encrypt_key: None,
                event_callback_url: "/webhook/lark".to_string(),
                auto_reply: true,
                allowed_groups: vec![],
            }),
            github: Some(GithubConfig {
                enabled: false,
                webhook_secret: "${GITHUB_WEBHOOK_SECRET}".to_string(),
                app_id: "".to_string(),
                private_key: "".to_string(),
                repository: "".to_string(),
                auto_scan_enabled: true,
                scan_interval_hours: 24,
                auto_commit_enabled: false,
                auto_pr_enabled: false,
                branches: vec!["main".to_string(), "master".to_string()],
                allowed_events: vec!["push".to_string(), "pull_request".to_string(), "issues".to_string()],
            }),
            gateway: GatewayConfig {
                http_enabled: true,
                http_port: 8080,
                http_host: "0.0.0.0".to_string(),
                websocket_enabled: false,
                websocket_port: 8081,
                cors_origins: vec!["*".to_string()],
                rate_limit_enabled: true,
                rate_limit_requests: 100,
                rate_limit_window_secs: 60,
            },
            daemon: DaemonConfig {
                pid_file: "/tmp/nanogpt-claw.pid".to_string(),
                restart_delay_secs: 5,
                max_restart_attempts: 10,
                health_check_interval_secs: 60,
                graceful_shutdown_timeout_secs: 30,
                watch_dog_enabled: true,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_enabled: true,
                file_path: "./logs/nanogpt-claw.log".to_string(),
                console_enabled: true,
                max_file_size_mb: 100,
                max_files: 10,
                format: "json".to_string(),
                include_timestamps: true,
                include_module_path: true,
            },
            optimization: OptimizationConfig {
                auto_evolution_enabled: true,
                evolution_interval_hours: 24,
                performance_monitoring_enabled: true,
                benchmark_on_startup: false,
                auto_optimize_prompts: true,
                prompt_learning_rate: 0.01,
            },
            safety: SafetyConfig {
                content_filter_enabled: false,
                content_filter_level: "medium".to_string(),
                pii_detection_enabled: false,
                pii_redaction_enabled: false,
                rate_limiting_enabled: true,
                max_session_length_minutes: 480,
                allowed_domains: vec![],
            },
            plugins: PluginsConfig {
                enabled: false,
                plugin_dir: "./plugins".to_string(),
                auto_load: false,
                enabled_plugins: vec![],
                plugin_settings: HashMap::new(),
            },
        }
    }
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .context(format!("Failed to read config file: {:?}", path.as_ref()))?;
        
        debug!("Interpolating environment variables in config");
        let interpolated_content = interpolate_env_vars(&content);
        
        let config: AppConfig = serde_yaml::from_str(&interpolated_content)
            .context("Failed to parse config YAML")?;
        
        Ok(config)
    }
    
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_yaml::to_string(self)
            .context("Failed to serialize config to YAML")?;
        
        std::fs::write(path.as_ref(), content)
            .context(format!("Failed to write config file: {:?}", path.as_ref()))?;
        
        Ok(())
    }
}

pub fn load_env_overrides(config: &mut AppConfig) {
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        if !api_key.is_empty() {
            config.llm.core_model.api_key = api_key;
            warn!("API key loaded from OPENAI_API_KEY environment variable");
        }
    }
}
