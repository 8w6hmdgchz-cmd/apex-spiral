//! # Session Memory Module
//!
//! Implements short-term in-memory storage with LRU (Least Recently Used) eviction
//! and optional TTL (Time-To-Live) expiry. This layer is volatile — data is lost
//! on process restart but offers sub-millisecond access times.
//!
//! ## Design Goals
//!
//! - **Speed**: O(1) lookup and insertion via HashMap
//! - **Efficiency**: Automatic LRU eviction when capacity is reached
//! - **Predictability**: Configurable TTL for time-based expiry
//! - **Transparency**: Automatic access time tracking for eviction decisions
//!
//! ## Architecture
//!
//! ```text
//! +----------------------------------+
//! |         SessionMemory            |
//! +----------------------------------+
//! |  entries: HashMap<String, Entry> |
//! |  access_order: VecDeque<Key>     |  <- O(1) LRU ordering
//! |  max_entries: usize             |
//! |  ttl_ms: u64                     |
//! +----------------------------------+
//! ```
//!
//! ## Eviction Policies
//!
//! - [`EvictionPolicy::LRU`] - Evict least recently accessed entries first
//! - [`EvictionPolicy::FIFO`] - Evict oldest entries by creation time
//! - [`EvictionPolicy::LFU`] - Evict least frequently accessed entries
//!
//! # Example
//!
//! ```rust,ignore
//! use nano_gpt_claw::memory::session::{SessionMemory, EvictionPolicy};
//!
//! let memory = SessionMemory::new(1000, 3600000, EvictionPolicy::LRU);
//! memory.insert("key1".to_string(), MemoryEntry::new("value1".to_string()));
//! ```

use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns current Unix timestamp in milliseconds.
fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Eviction policy determines which entry is selected for removal
/// when the session memory reaches capacity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvictionPolicy {
    /// Least Recently Used - evict entry with oldest last_accessed time
    LRU,
    /// First In First Out - evict entry with oldest created_at time
    FIFO,
    /// Least Frequently Used - evict entry with lowest access count
    LFU,
}

/// A session memory entry with access tracking metadata.
/// Re-exports from the parent module via type alias.
pub type MemoryEntry = crate::memory::MemoryEntry;

/// SessionMemory provides fast, volatile, in-memory storage for session data.
///
/// # Example
/// ```rust,ignore
/// use nano_gpt_claw::memory::session::{SessionMemory, EvictionPolicy};
///
/// let mut memory = SessionMemory::new(100, 60000, EvictionPolicy::LRU);
/// memory.insert("session:abc".to_string(), MemoryEntry::new("data".to_string()));
/// assert!(memory.get("session:abc").is_some());
/// ```
pub struct SessionMemory {
    /// Core storage: key -> entry
    entries: HashMap<String, MemoryEntry>,
    /// LRU access order tracking (front = most recent)
    access_order: VecDeque<String>,
    /// LFU frequency counter: key -> access count
    access_counts: HashMap<String, u64>,
    /// Maximum entries before eviction
    max_entries: usize,
    /// Time-to-live in milliseconds (0 = no expiry)
    ttl_ms: u64,
    /// Current eviction policy
    eviction_policy: EvictionPolicy,
    /// Statistics for monitoring
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl SessionMemory {
    /// Creates a new SessionMemory instance.
    ///
    /// # Arguments
    /// * `max_entries` - Maximum number of entries before LRU eviction triggers
    /// * `ttl_ms` - Time-to-live in milliseconds (0 = never expire)
    /// * `eviction_policy` - Strategy for selecting entries to evict
    pub fn new(max_entries: usize, ttl_ms: u64, eviction_policy: EvictionPolicy) -> Self {
        Self {
            entries: HashMap::with_capacity(max_entries),
            access_order: VecDeque::new(),
            access_counts: HashMap::new(),
            max_entries,
            ttl_ms,
            eviction_policy,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    /// Inserts or updates a memory entry.
    ///
    /// # Arguments
    /// * `key` - Unique identifier for this entry
    /// * `entry` - MemoryEntry containing the value
    pub fn insert(&mut self, key: String, entry: MemoryEntry) {
        if !self.entries.contains_key(&key) && self.entries.len() >= self.max_entries {
            self.evict_one();
        }

        if self.entries.contains_key(&key) {
            self.remove_from_access_order(&key);
        }

        self.entries.insert(key.clone(), entry);
        self.access_order.push_front(key.clone());
        *self.access_counts.entry(key).or_insert(0) += 1;
    }

    /// Retrieves a memory entry by key.
    pub fn get(&mut self, key: &str) -> Option<&MemoryEntry> {
        match self.entries.get(key) {
            Some(_) => {
                self.hits += 1;
                self.remove_from_access_order(key);
                self.access_order.push_front(key.to_string());
                *self.access_counts.entry(key.to_string()).or_insert(1) += 1;
                let e = self.entries.get_mut(key).unwrap();
                e.last_accessed = now_millis();
                // Return reference through a unsafe block to avoid borrow issues
                // In practice, we'd restructure this, but for now:
                self.entries.get(key)
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    /// Retrieves a mutable memory entry by key.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut MemoryEntry> {
        // First update hit/miss stats and access order using indices
        let key_owned = key.to_string();
        let key_exists = self.entries.contains_key(key);
        
        if key_exists {
            self.hits += 1;
            self.remove_from_access_order(&key_owned);
            self.access_order.push_front(key_owned.clone());
            *self.access_counts.entry(key_owned.clone()).or_insert(1) += 1;
        } else {
            self.misses += 1;
        }
        
        // Now get mutable access to the entry
        if let Some(entry) = self.entries.get_mut(key) {
            entry.last_accessed = now_millis();
            Some(entry)
        } else {
            None
        }
    }

    /// Removes a memory entry by key.
    ///
    /// # Returns
    /// * `bool` - True if the key existed and was removed
    pub fn remove(&mut self, key: &str) -> bool {
        if self.entries.remove(key).is_some() {
            self.remove_from_access_order(key);
            self.access_counts.remove(key);
            true
        } else {
            false
        }
    }

    /// Checks if a key exists in the session memory.
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Returns the number of entries currently stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the session memory contains no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns true if the session memory is at capacity.
    pub fn is_full(&self) -> bool {
        self.entries.len() >= self.max_entries
    }

    /// Clears all entries from session memory.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.access_counts.clear();
    }

    /// Returns an iterator over all (key, entry) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &MemoryEntry)> {
        self.entries.iter()
    }

    /// Returns an iterator over entries sorted by the current eviction policy.
    ///
    /// # Arguments
    /// * `policy` - Override the default eviction policy for sorting
    pub fn iter_sorted(&self, policy: &EvictionPolicy) -> Vec<(&String, &MemoryEntry)> {
        let mut items: Vec<_> = self.entries.iter().collect();
        match policy {
            EvictionPolicy::LRU => {
                items.sort_by(|a, b| a.1.last_accessed.cmp(&b.1.last_accessed));
            }
            EvictionPolicy::FIFO => {
                items.sort_by(|a, b| a.1.created_at.cmp(&b.1.created_at));
            }
            EvictionPolicy::LFU => {
                items.sort_by(|a, b| {
                    let count_a = self.access_counts.get(a.0).copied().unwrap_or(0);
                    let count_b = self.access_counts.get(b.0).copied().unwrap_or(0);
                    count_a.cmp(&count_b)
                });
            }
        }
        items
    }

    /// Evicts all entries that have exceeded their TTL.
    ///
    /// # Returns
    /// * `usize` - Number of entries evicted
    pub fn evict_expired(&mut self) -> usize {
        if self.ttl_ms == 0 {
            return 0;
        }

        let now = now_millis();
        let mut evicted: usize = 0;
        let keys_to_evict: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, entry)| now.saturating_sub(entry.created_at) > self.ttl_ms)
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_evict {
            self.remove(&key);
            evicted += 1;
        }

        self.evictions += evicted as u64;
        evicted
    }

    /// Returns cache hit ratio as a percentage (0.0 - 100.0).
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Returns memory statistics.
    pub fn stats(&self) -> SessionStats {
        SessionStats {
            entries: self.entries.len(),
            capacity: self.max_entries,
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
            hit_ratio: self.hit_ratio(),
        }
    }

    /// Removes a key from the access order tracking.
    fn remove_from_access_order(&mut self, key: &str) {
        let pos = self.access_order.iter().position(|k| k == key);
        if let Some(idx) = pos {
            self.access_order.remove(idx);
        }
    }

    /// Evicts one entry based on the configured eviction policy.
    fn evict_one(&mut self) {
        let victim = match self.eviction_policy {
            EvictionPolicy::LRU => {
                self.access_order.pop_back()
            }
            EvictionPolicy::FIFO => {
                self.entries
                    .iter()
                    .min_by_key(|(_, e)| e.created_at)
                    .map(|(k, _)| k.clone())
            }
            EvictionPolicy::LFU => {
                self.access_counts
                    .iter()
                    .min_by_key(|(_, count)| *count)
                    .map(|(k, _)| k.clone())
            }
        };

        if let Some(key) = victim {
            self.remove(&key);
            self.evictions += 1;
        }
    }
}

/// Session memory statistics for monitoring and debugging.
#[derive(Debug, Clone)]
pub struct SessionStats {
    /// Current number of entries
    pub entries: usize,
    /// Maximum capacity
    pub capacity: usize,
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Total number of evictions performed
    pub evictions: u64,
    /// Cache hit ratio as percentage
    pub hit_ratio: f64,
}

impl std::fmt::Display for SessionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SessionMemory(stats: {}/{} entries, hit_ratio: {:.1}%, evictions: {})",
            self.entries, self.capacity, self.hit_ratio, self.evictions
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(value: &str) -> MemoryEntry {
        MemoryEntry::new(value.to_string())
    }

    #[test]
    fn test_basic_insert_get() {
        let mut memory = SessionMemory::new(10, 0, EvictionPolicy::LRU);
        memory.insert("key1".to_string(), make_entry("value1"));
        assert!(memory.get("key1").is_some());
        assert_eq!(memory.get("key1").unwrap().value, "value1");
    }

    #[test]
    fn test_missing_key() {
        let mut memory = SessionMemory::new(10, 0, EvictionPolicy::LRU);
        assert!(memory.get("nonexistent").is_none());
    }

    #[test]
    fn test_update_existing() {
        let mut memory = SessionMemory::new(10, 0, EvictionPolicy::LRU);
        memory.insert("key1".to_string(), make_entry("value1"));
        memory.insert("key1".to_string(), make_entry("value2"));
        assert_eq!(memory.get("key1").unwrap().value, "value2");
        assert_eq!(memory.len(), 1);
    }

    #[test]
    fn test_lru_eviction() {
        let mut memory = SessionMemory::new(3, 0, EvictionPolicy::LRU);
        memory.insert("a".to_string(), make_entry("1"));
        memory.insert("b".to_string(), make_entry("2"));
        memory.insert("c".to_string(), make_entry("3"));

        memory.insert("d".to_string(), make_entry("4"));

        assert!(memory.get("a").is_none());
        assert!(memory.get("b").is_some());
        assert!(memory.get("c").is_some());
        assert!(memory.get("d").is_some());
    }

    #[test]
    fn test_remove() {
        let mut memory = SessionMemory::new(10, 0, EvictionPolicy::LRU);
        memory.insert("key1".to_string(), make_entry("value1"));
        assert!(memory.remove("key1"));
        assert!(memory.get("key1").is_none());
        assert!(!memory.remove("key1"));
    }

    #[test]
    fn test_clear() {
        let mut memory = SessionMemory::new(10, 0, EvictionPolicy::LRU);
        memory.insert("a".to_string(), make_entry("1"));
        memory.insert("b".to_string(), make_entry("2"));
        assert_eq!(memory.len(), 2);
        memory.clear();
        assert!(memory.is_empty());
    }
}
