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

pub static APP_CONFIG: Lazy<RwLock<AppConfig>> = Lazy::new(|| {
    RwLock::new(AppConfig::default())
});

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
    
    *APP_CONFIG.write() = config.clone();
    Ok(config)
}

pub fn reload_config() -> Result<AppConfig> {
    let config_dir = get_config_dir();
    let config_path = config_dir.join("config.yaml");
    let config = AppConfig::load_from_file(&config_path)?;
    *APP_CONFIG.write() = config.clone();
    Ok(config)
}
