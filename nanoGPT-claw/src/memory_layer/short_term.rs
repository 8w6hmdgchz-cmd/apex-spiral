use crate::memory_layer::{MemoryEntry, MemoryQuery, MemoryStats};
use anyhow::Result;
use chrono::{Duration, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

pub struct ShortTermMemory {
    entries: RwLock<HashMap<String, MemoryEntry>>,
    ttl_hours: u64,
    max_entries: usize,
}

impl ShortTermMemory {
    pub fn new(ttl_hours: u64, max_entries: usize) -> Result<Self> {
        Ok(Self {
            entries: RwLock::new(HashMap::new()),
            ttl_hours,
            max_entries,
        })
    }

    pub fn init(&self) -> Result<()> {
        info!("Initializing short-term memory with TTL {}h, max {} entries", 
              self.ttl_hours, self.max_entries);
        Ok(())
    }

    pub fn store(&self, entry: MemoryEntry) -> Result<String> {
        let mut entries = self.entries.write();
        
        if entries.len() >= self.max_entries {
            self.evict_lowest_priority(&mut entries);
        }
        
        let id = entry.id.clone();
        entries.insert(id.clone(), entry);
        debug!("Stored entry in short-term memory: {}", id);
        Ok(id)
    }

    pub fn get_by_id(&self, id: &str) -> Option<MemoryEntry> {
        let mut entries = self.entries.write();
        if let Some(mut entry) = entries.get_mut(id) {
            entry.touch();
            Some(entry.clone())
        } else {
            None
        }
    }

    pub fn query(&self, query: &MemoryQuery) -> Result<Vec<MemoryEntry>> {
        let entries = self.entries.read();
        let now = Utc::now();
        
        let mut results: Vec<MemoryEntry> = entries.values()
            .filter(|e| {
                if let Some(min_imp) = query.min_importance {
                    if e.importance < min_imp {
                        return false;
                    }
                }
                if let Some(ref cat) = query.category {
                    if e.category.as_ref() != Some(cat) {
                        return false;
                    }
                }
                if let Some(ref kw) = query.keyword {
                    if !e.content.to_lowercase().contains(&kw.to_lowercase()) {
                        return false;
                    }
                }
                let age = now.signed_duration_since(e.created_at);
                if age > Duration::hours(self.ttl_hours as i64) {
                    return false;
                }
                true
            })
            .cloned()
            .collect();
        
        results.sort_by(|a, b| {
            b.importance.partial_cmp(&a.importance).unwrap()
        });
        
        results.truncate(query.limit);
        Ok(results)
    }

    pub fn delete(&self, id: &str) -> bool {
        let mut entries = self.entries.write();
        entries.remove(id).is_some()
    }

    pub fn cleanup_expired(&self) -> Result<usize> {
        let now = Utc::now();
        let ttl = Duration::hours(self.ttl_hours as i64);
        let mut entries = self.entries.write();
        let mut removed = 0;
        
        entries.retain(|_, entry| {
            let should_keep = now.signed_duration_since(entry.created_at) <= ttl;
            if !should_keep {
                removed += 1;
            }
            should_keep
        });
        
        if removed > 0 {
            info!("Cleaned up {} expired entries from short-term memory", removed);
        }
        Ok(removed)
    }

    pub fn get_stats(&self) -> MemoryStats {
        let entries = self.entries.read();
        let total = entries.len();
        let total_access: u64 = entries.values().map(|e| e.access_count).sum();
        let avg_importance = if total > 0 {
            entries.values().map(|e| e.importance).sum::<f32>() / total as f32
        } else {
            0.0
        };
        
        MemoryStats {
            total_entries: total,
            avg_importance,
            total_access_count: total_access,
            short_term_count: total,
            long_term_count: 0,
        }
    }

    pub fn compact(&self) -> Result<()> {
        let mut entries = self.entries.write();
        let capacity = entries.capacity();
        
        if capacity > self.max_entries * 2 {
            entries.reserve(self.max_entries);
            info!("Compacted short-term memory: capacity {} -> {}", 
                  capacity, entries.capacity());
        }
        Ok(())
    }

    fn evict_lowest_priority(&self, entries: &mut HashMap<String, MemoryEntry>) {
        if let Some((id, _)) = entries.iter()
            .min_by(|a, b| a.1.importance.partial_cmp(&b.1.importance).unwrap())
            .map(|(id, entry)| (id.clone(), entry.clone()))
        {
            entries.remove(&id);
            debug!("Evicted low priority entry: {}", id);
        }
    }
}

impl Default for ShortTermMemory {
    fn default() -> Self {
        Self::new(24, 1000).expect("Failed to create default short-term memory")
    }
}
