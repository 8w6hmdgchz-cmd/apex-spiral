//! Configuration module
pub mod settings;
pub mod env;

use anyhow::Result;
use dirs;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::path::PathBuf;
use tracing::info;

pub use settings::*;
pub use env::{interpolate_env_vars, EnvInterpolate};

#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    pub fn load_from_file(_path: &str) -> Result<Self> {
        Ok(Self::default())
    }
}

pub static APP_CONFIG: Lazy<RwLock<AppConfig>> = Lazy::new(|| {
    RwLock::new(AppConfig::default())
});

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nano-gpt-claw")
}

pub fn init_config() -> Result<AppConfig> {
    let config_path = get_config_dir().join("config.toml");
    info!("Config path: {:?}", config_path);
    
    let config = if config_path.exists() {
        AppConfig::load_from_file(config_path.to_str().unwrap_or(""))?
    } else {
        AppConfig::default()
    };
    
    *APP_CONFIG.write() = config.clone();
    Ok(config)
}

pub fn reload_config() -> Result<AppConfig> {
    init_config()
}

pub fn get_config() -> AppConfig {
    APP_CONFIG.read().clone()
}
