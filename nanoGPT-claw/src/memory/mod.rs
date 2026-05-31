//! # Memory Layer - Main Module

pub mod persistent;
pub mod session;
pub mod long_term;

use std::sync::Arc;
use tokio::sync::RwLock;

pub use session::{SessionMemory, EvictionPolicy};
pub use persistent::{PersistentMemory, DbConfig};

#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub value: String,
    pub created_at: u64,
    pub last_accessed: u64,
    pub embedding: Option<Vec<f32>>,
    pub tags: Vec<String>,
}

impl MemoryEntry {
    pub fn new(value: String) -> Self {
        let now = current_time_millis();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            embedding: None,
            tags: Vec::new(),
        }
    }

    pub fn with_embedding(value: String, embedding: Vec<f32>) -> Self {
        let mut entry = Self::new(value);
        entry.embedding = Some(embedding);
        entry
    }

    pub fn with_tags(value: String, tags: Vec<String>) -> Self {
        let mut entry = Self::new(value);
        entry.tags = tags;
        entry
    }

    pub fn touch(&mut self) {
        self.last_accessed = current_time_millis();
    }

    pub fn age(&self) -> u64 {
        current_time_millis().saturating_sub(self.created_at)
    }

    pub fn idle_time(&self) -> u64 {
        current_time_millis().saturating_sub(self.last_accessed)
    }
}

fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub session_max_entries: usize,
    pub session_ttl_ms: u64,
    pub eviction_policy: EvictionPolicy,
    pub db_path: String,
    pub persistent_max_entries: usize,
    pub embedding_dim: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            session_max_entries: 10_000,
            session_ttl_ms: 3_600_000,
            eviction_policy: EvictionPolicy::LRU,
            db_path: "nanoGPT-claw.memory.db".to_string(),
            persistent_max_entries: 100_000,
            embedding_dim: 384,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryQuery {
    pub prefix: Option<String>,
    pub tags: Vec<String>,
    pub min_age_ms: Option<u64>,
    pub limit: usize,
    pub sort_by: MemorySortKey,
}

impl Default for MemoryQuery {
    fn default() -> Self {
        Self {
            prefix: None,
            tags: Vec::new(),
            min_age_ms: None,
            limit: 100,
            sort_by: MemorySortKey::LastAccessed,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MemorySortKey {
    LastAccessed,
    CreatedAt,
    KeyName,
}

pub struct MemoryLayer {
    session: Arc<RwLock<SessionMemory>>,
    persistent: Arc<RwLock<PersistentMemory>>,
    config: MemoryConfig,
}

pub type MemoryManager = MemoryLayer;

impl MemoryLayer {
    pub async fn new(config: MemoryConfig) -> Result<Self, MemoryError> {
        let persistent = PersistentMemory::new(
            DbConfig {
                path: config.db_path.clone(),
                max_entries: config.persistent_max_entries,
                embedding_dim: config.embedding_dim,
            }
        ).await?;

        let session = SessionMemory::new(
            config.session_max_entries,
            config.session_ttl_ms,
            config.eviction_policy.clone(),
        );

        Ok(Self {
            session: Arc::new(RwLock::new(session)),
            persistent: Arc::new(RwLock::new(persistent)),
            config,
        })
    }

    pub async fn store_session(&self, key: &str, entry: MemoryEntry) -> Result<(), MemoryError> {
        let mut session = self.session.write().await;
        session.insert(key.to_string(), entry);
        Ok(())
    }

    pub async fn retrieve_session(&self, key: &str) -> Option<MemoryEntry> {
        let mut session = self.session.write().await;
        let result = session.get(key).cloned();
        if result.is_some() {
            if let Some(entry) = session.get_mut(key) {
                entry.touch();
            }
        }
        result
    }

    pub async fn remove_session(&self, key: &str) -> bool {
        let mut session = self.session.write().await;
        session.remove(key)
    }

    pub async fn store_persistent(
        &self,
        key: &str,
        entry: MemoryEntry,
    ) -> Result<(), MemoryError> {
        let mut persistent = self.persistent.write().await;
        persistent.insert(key, entry).await
    }

    pub async fn retrieve_persistent(&self, key: &str) -> Option<MemoryEntry> {
        let persistent = self.persistent.write().await;
        persistent.get(key).await
    }

    pub async fn semantic_search(
        &self,
        query_embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<(String, MemoryEntry, f32)>, MemoryError> {
        let persistent = self.persistent.write().await;
        persistent.semantic_search(query_embedding, top_k).await
    }

    pub async fn store_both(
        &self,
        key: &str,
        entry: MemoryEntry,
        immediate_persist: bool,
    ) -> Result<(), MemoryError> {
        {
            let mut session = self.session.write().await;
            session.insert(key.to_string(), entry.clone());
        }
        {
            let mut persistent = self.persistent.write().await;
            if immediate_persist {
                persistent.insert(key, entry).await?;
            }
        }
        Ok(())
    }

    pub async fn query_session(&self, query: MemoryQuery) -> Vec<(String, MemoryEntry)> {
        let session = self.session.read().await;
        let mut results: Vec<(String, MemoryEntry)> = session
            .iter()
            .filter(|(key, entry)| {
                if let Some(ref prefix) = query.prefix {
                    if !key.starts_with(prefix) {
                        return false;
                    }
                }
                if !query.tags.is_empty() {
                    if !query.tags.iter().all(|t| entry.tags.contains(t)) {
                        return false;
                    }
                }
                if let Some(min_age) = query.min_age_ms {
                    if entry.age() < min_age {
                        return false;
                    }
                }
                true
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        match query.sort_by {
            MemorySortKey::LastAccessed => {
                results.sort_by(|a, b| b.1.last_accessed.cmp(&a.1.last_accessed));
            }
            MemorySortKey::CreatedAt => {
                results.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));
            }
            MemorySortKey::KeyName => {
                results.sort_by(|a, b| a.0.cmp(&b.0));
            }
        }

        results.truncate(query.limit);
        results
    }

    pub async fn stats(&self) -> MemoryStats {
        let session = self.session.read().await;
        let persistent = self.persistent.read().await;

        MemoryStats {
            session_entries: session.len(),
            session_capacity: self.config.session_max_entries,
            persistent_entries: persistent.len().await,
            persistent_capacity: self.config.persistent_max_entries,
        }
    }

    pub async fn evict_expired(&self) -> usize {
        let mut session = self.session.write().await;
        session.evict_expired()
    }

    pub async fn clear_session(&self) {
        let mut session = self.session.write().await;
        session.clear();
    }

    pub async fn shutdown(&self) -> Result<(), MemoryError> {
        let persistent = self.persistent.write().await;
        persistent.flush().await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub session_entries: usize,
    pub session_capacity: usize,
    pub persistent_entries: usize,
    pub persistent_capacity: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Session memory error: {0}")]
    Session(String),

    #[error("Persistent storage error: {0}")]
    Persistent(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Embedding error: {0}")]
    Embedding(String),
}

pub type MemoryResult<T> = Result<T, MemoryError>;
