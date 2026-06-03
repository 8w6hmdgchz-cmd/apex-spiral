use crate::memory_layer::{long_term, short_term, MemoryEntry, MemoryQuery, MemoryStats};
use anyhow::Result;
use dirs;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info};

pub struct MemoryStorage {
    short_term: Arc<short_term::ShortTermMemory>,
    long_term: Arc<long_term::LongTermMemory>,
    similarity_threshold: f32,
}

impl MemoryStorage {
    pub fn new() -> Result<Self> {
        let base_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nanoGPT-claw")
            .join("memory");
        
        let short_term = Arc::new(short_term::ShortTermMemory::new(24, 1000)?);
        let long_term = Arc::new(long_term::LongTermMemory::new(&base_dir)?);
        
        Ok(Self {
            short_term,
            long_term,
            similarity_threshold: 0.75,
        })
    }

    pub fn init(&self) -> Result<()> {
        self.short_term.init()?;
        info!("Memory storage initialized successfully");
        Ok(())
    }

    pub fn store(&self, entry: MemoryEntry) -> Result<String> {
        let short_id = self.short_term.store(entry.clone())?;
        
        if entry.importance >= self.similarity_threshold {
            if let Err(e) = self.long_term.store(&entry) {
                debug!("Failed to store in long-term memory: {}", e);
            }
        }
        
        Ok(short_id)
    }

    pub fn query(&self, query: &MemoryQuery) -> Result<Vec<MemoryEntry>> {
        let mut results = self.short_term.query(query)?;
        
        if let Some(ref kw) = query.keyword {
            let long_results = self.long_term.query(&MemoryQuery {
                keyword: Some(kw.clone()),
                limit: query.limit.saturating_sub(results.len()),
                ..Default::default()
            })?;
            
            if let Some(long_entries) = long_results {
                for entry in long_entries {
                    if !results.iter().any(|e| e.id == entry.id) {
                        results.push(entry);
                    }
                }
            }
        }
        
        results.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
        results.truncate(query.limit);
        
        Ok(results)
    }

    pub fn get_by_id(&self, id: &str) -> Option<MemoryEntry> {
        self.short_term.get_by_id(id)
            .or_else(|| {
                match self.long_term.get_by_id(id) {
                    Ok(Some(entry)) => Some(entry),
                    _ => None,
                }
            })
    }

    pub fn delete(&self, id: &str) -> Result<bool> {
        let short_deleted = self.short_term.delete(id);
        let long_deleted = self.long_term.delete(id).unwrap_or(false);
        Ok(short_deleted || long_deleted)
    }

    pub fn get_stats(&self) -> MemoryStats {
        let mut stats = self.short_term.get_stats();
        
        if let Ok(long_stats) = self.long_term.get_stats() {
            let count: Option<i64> = long_stats.get("total_entries").and_then(|v: &serde_json::Value| v.as_i64());
            if let Some(count_val) = count {
                stats.long_term_count = count_val as usize;
            }
            stats.total_entries = stats.short_term_count + stats.long_term_count;
        }
        
        stats
    }

    pub fn cleanup_expired(&self) -> Result<usize> {
        let removed = self.short_term.cleanup_expired()?;
        Ok(removed)
    }

    pub fn compact(&self) -> Result<()> {
        self.short_term.compact()?;
        self.long_term.compact()?;
        Ok(())
    }

    pub fn sync_to_long_term(&self) -> Result<usize> {
        let entries = self.short_term.query(&MemoryQuery {
            min_importance: Some(self.similarity_threshold),
            limit: 1000,
            ..Default::default()
        })?;
        
        let mut synced = 0;
        for entry in entries {
            if let Err(e) = self.long_term.store(&entry) {
                debug!("Failed to sync entry {} to long-term: {}", entry.id, e);
            } else {
                synced += 1;
            }
        }
        
        info!("Synced {} high-importance entries to long-term memory", synced);
        Ok(synced)
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create default memory storage")
    }
}
