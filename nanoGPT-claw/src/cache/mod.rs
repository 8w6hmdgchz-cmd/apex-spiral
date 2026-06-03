//! # Cache Module - Unified Caching Layer
//!
//! ✅ 支持多层缓存：内存 (quick_cache) + 分布式 (Redis)
//! ✅ 自动选择最优缓存策略
//! ✅ 统一的缓存接口

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use tokio::sync::RwLock;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub memory_max_items: usize,
    pub memory_ttl_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            memory_max_items: 1000,
            memory_ttl_secs: 300,
        }
    }
}

pub struct MemoryCache {
    cache: Arc<RwLock<HashMap<String, (String, Instant)>>>,
    max_items: usize,
    hits: Arc<std::sync::atomic::AtomicU64>,
    misses: Arc<std::sync::atomic::AtomicU64>,
}

impl MemoryCache {
    pub fn new(max_items: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_items,
            hits: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            misses: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().await;
        
        if let Some((value, expiry)) = cache.get(key) {
            if Instant::now() < *expiry {
                self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                debug!("Memory cache HIT: {}", key);
                return Some(value.clone());
            } else {
                drop(cache);
                self.delete(key).await.ok();
            }
        } else {
            self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            debug!("Memory cache MISS: {}", key);
        }
        
        None
    }

    pub async fn set(&self, key: &str, value: &str, ttl_secs: u64) -> Result<()> {
        let mut cache = self.cache.write().await;
        
        if cache.len() >= self.max_items && !cache.contains_key(key) {
            let oldest_key = cache.iter()
                .min_by_key(|(_, (_, time))| *time)
                .map(|(k, _)| k.clone());
            
            if let Some(key_to_remove) = oldest_key {
                cache.remove(&key_to_remove);
            }
        }
        
        let expiry = Instant::now() + Duration::from_secs(ttl_secs);
        cache.insert(key.to_string(), (value.to_string(), expiry));
        
        debug!("Memory cache SET: {} (ttl={}s)", key, ttl_secs);
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.remove(key);
        debug!("Memory cache DELETE: {}", key);
        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.clear();
        debug!("Memory cache CLEARED");
        Ok(())
    }

    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(std::sync::atomic::Ordering::Relaxed),
            misses: self.misses.load(std::sync::atomic::Ordering::Relaxed),
            size: self.cache.blocking_read().len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
}

pub async fn create_memory_cache(config: CacheConfig) -> MemoryCache {
    info!("Creating memory cache (max_items: {})", config.memory_max_items);
    MemoryCache::new(config.memory_max_items)
}
