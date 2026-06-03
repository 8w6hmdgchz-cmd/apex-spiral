pub mod short_term;
pub mod long_term;
pub mod types;
pub mod storage;

use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tracing::info;

pub use types::*;
pub use storage::MemoryStorage;

pub static MEMORY_STORAGE: once_cell::sync::Lazy<Arc<MemoryStorage>> = 
    once_cell::sync::Lazy::new(|| {
        Arc::new(MemoryStorage::new().expect("Failed to create memory storage"))
    });

pub fn init_memory() -> Result<()> {
    info!("Initializing memory layer...");
    MEMORY_STORAGE.init()?;
    info!("Memory layer initialized successfully");
    Ok(())
}
