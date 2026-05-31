//! # Persistent Memory Module
//!
//! Implements long-term SQLite-backed storage with semantic search capabilities.
//! Unlike session memory, data in this layer persists across restarts.
//!
//! ## Features
//!
//! - **Persistence**: SQLite database for durable storage
//! - **Semantic Search**: Vector embedding storage with cosine similarity search
//! - **Tag-based Filtering**: Categorical organization via tags
//! - **Batched Writes**: Optimized for high-throughput workloads
//!
//! ## Database Schema
//!
//! ```sql
//! CREATE TABLE memories (
//!     key TEXT PRIMARY KEY,
//!     value TEXT NOT NULL,
//!     embedding BLOB,           -- Serialized f32 vector
//!     tags TEXT,                -- JSON array of tags
//!     created_at INTEGER,
//!     last_accessed INTEGER
//! );
//!
//! CREATE INDEX idx_created_at ON memories(created_at);
//! CREATE INDEX idx_last_accessed ON memories(last_accessed);
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use nano_gpt_claw::memory::persistent::{PersistentMemory, DbConfig};
//!
//! let config = DbConfig {
//!     path: "memory.db".to_string(),
//!     max_entries: 100_000,
//!     embedding_dim: 384,
//! };
//!
//! let memory = PersistentMemory::new(config).await.unwrap();
//! ```

use rusqlite::{Connection, params};
use std::sync::RwLock;

use crate::memory::MemoryEntry;

/// Configuration for the persistent memory database.
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Path to the SQLite database file
    pub path: String,
    /// Maximum number of entries (0 = unlimited)
    pub max_entries: usize,
    /// Dimension of embedding vectors (must match model)
    pub embedding_dim: usize,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            path: "nanoGPT-claw.memory.db".to_string(),
            max_entries: 100_000,
            embedding_dim: 384,
        }
    }
}

/// PersistentMemory provides SQLite-backed long-term storage.
///
/// # Example
/// ```rust,ignore
/// use nano_gpt_claw::memory::persistent::{PersistentMemory, DbConfig};
///
/// let config = DbConfig::default();
/// let memory = PersistentMemory::new(config).await.unwrap();
///
/// memory.insert("key1", MemoryEntry::new("value1".to_string())).await;
/// ```
pub struct PersistentMemory {
    /// SQLite database connection (thread-safe via RwLock for read/write separation)
    conn: RwLock<Connection>,
    /// Configuration
    config: DbConfig,
    /// Track connection state for debugging
    is_open: bool,
}

impl PersistentMemory {
    /// Creates a new PersistentMemory instance and initializes the database.
    ///
    /// # Arguments
    /// * `config` - Database configuration (path, max entries, embedding dim)
    ///
    /// # Returns
    /// * `Result<Self>` - New instance or error on database failure
    ///
    /// # Errors
    /// Returns error if SQLite cannot create/open the database file,
    /// or if schema initialization fails.
    pub async fn new(config: DbConfig) -> Result<Self, crate::memory::MemoryError> {
        let conn = Self::init_db(&config)?;
        
        Ok(Self {
            conn: RwLock::new(conn),
            config,
            is_open: true,
        })
    }

    /// Initializes the SQLite database and creates schema if needed.
    fn init_db(config: &DbConfig) -> Result<Connection, crate::memory::MemoryError> {
        let conn = Connection::open(&config.path)
            .map_err(|e| crate::memory::MemoryError::Database(e))?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| crate::memory::MemoryError::Database(e))?;

        // Create the memories table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS memories (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                embedding BLOB,
                tags TEXT,
                created_at INTEGER NOT NULL,
                last_accessed INTEGER NOT NULL
            )",
            [],
        ).map_err(|e| crate::memory::MemoryError::Database(e))?;

        // Create indexes for efficient querying
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON memories(created_at)",
            [],
        ).map_err(|e| crate::memory::MemoryError::Database(e))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_last_accessed ON memories(last_accessed)",
            [],
        ).map_err(|e| crate::memory::MemoryError::Database(e))?;

        Ok(conn)
    }

    /// Inserts or updates a memory entry in persistent storage.
    ///
    /// # Arguments
    /// * `key` - Unique identifier for this entry
    /// * `entry` - MemoryEntry containing value, optional embedding, and tags
    ///
    /// # Note
    /// If embedding is provided, it's serialized as a blob of f32 values.
    /// Tags are stored as a JSON array string.
    pub async fn insert(&mut self, key: &str, entry: MemoryEntry) 
        -> Result<(), crate::memory::MemoryError> 
    {
        let conn = self.conn.write().unwrap();
        
        // Serialize embedding if present
        let embedding_blob: Option<Vec<u8>> = entry.embedding.as_ref().map(|e| {
            let mut bytes = Vec::with_capacity(e.len() * 4);
            for f in e {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            bytes
        });

        // Serialize tags as JSON array
        let tags_json = if entry.tags.is_empty() {
            String::new()
        } else {
            serde_json::to_string(&entry.tags).unwrap_or_default()
        };

        conn.execute(
            "INSERT OR REPLACE INTO memories (key, value, embedding, tags, created_at, last_accessed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                key,
                entry.value,
                embedding_blob,
                tags_json,
                entry.created_at as i64,
                entry.last_accessed as i64,
            ],
        ).map_err(|e| crate::memory::MemoryError::Database(e))?;

        // Check if we need to evict old entries
        if self.config.max_entries > 0 {
            Self::evict_if_needed(&conn, self.config.max_entries)?;
        }

        Ok(())
    }

    /// Retrieves a memory entry by key.
    ///
    /// # Arguments
    /// * `key` - The key to look up
    ///
    /// # Returns
    /// * `Option<MemoryEntry>` - The entry if found, None otherwise
    pub async fn get(&self, key: &str) -> Option<MemoryEntry> {
        let conn = self.conn.read().unwrap();
        
        let result = conn.query_row(
            "SELECT value, embedding, tags, created_at, last_accessed 
             FROM memories WHERE key = ?1",
            params![key],
            |row| {
                let value: String = row.get(0)?;
                let embedding_blob: Option<Vec<u8>> = row.get(1)?;
                let tags_json: Option<String> = row.get(2)?;
                let created_at: i64 = row.get(3)?;
                let last_accessed: i64 = row.get(4)?;

                // Deserialize embedding
                let embedding = embedding_blob.map(|bytes| {
                    let float_count = bytes.len() / 4;
                    let mut vec = Vec::with_capacity(float_count);
                    for i in 0..float_count {
                        let f = f32::from_le_bytes([
                            bytes[i * 4],
                            bytes[i * 4 + 1],
                            bytes[i * 4 + 2],
                            bytes[i * 4 + 3],
                        ]);
                        vec.push(f);
                    }
                    vec
                });

                // Deserialize tags
                let tags: Vec<String> = tags_json
                    .as_ref()
                    .and_then(|j| serde_json::from_str(j).ok())
                    .unwrap_or_default();

                Ok(MemoryEntry {
                    value,
                    created_at: created_at as u64,
                    last_accessed: last_accessed as u64,
                    embedding,
                    tags,
                })
            },
        );

        match result {
            Ok(entry) => {
                // Update last_accessed time
                let now = current_time_millis() as i64;
                conn.execute(
                    "UPDATE memories SET last_accessed = ?1 WHERE key = ?2",
                    params![now, key],
                ).ok();
                Some(entry)
            }
            Err(_) => None,
        }
    }

    /// Removes a memory entry by key.
    ///
    /// # Arguments
    /// * `key` - The key to remove
    ///
    /// # Returns
    /// * `bool` - True if the key existed and was removed
    pub async fn remove(&self, key: &str) -> bool {
        let conn = self.conn.write().unwrap();
        
        let affected = conn.execute(
            "DELETE FROM memories WHERE key = ?1",
            params![key],
        ).unwrap_or(0);

        affected > 0
    }

    /// Performs semantic search using cosine similarity on embeddings.
    ///
    /// # Arguments
    /// * `query_embedding` - The embedding vector to search with
    /// * `top_k` - Number of results to return
    ///
    /// # Returns
    /// * `Vec<(String, MemoryEntry, f32)>` - List of (key, entry, similarity_score)
    ///   sorted by similarity (highest first)
    ///
    /// # Note
    /// Only entries with pre-computed embeddings are considered in the search.
    /// Entries without embeddings are skipped.
    pub async fn semantic_search(
        &self,
        query_embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<(String, MemoryEntry, f32)>, crate::memory::MemoryError> 
    {
        let conn = self.conn.read().unwrap();
        
        // Get all entries with embeddings
        let mut stmt = conn.prepare(
            "SELECT key, value, embedding, tags, created_at, last_accessed 
             FROM memories WHERE embedding IS NOT NULL"
        ).map_err(|e| crate::memory::MemoryError::Database(e))?;

        let rows = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            let embedding_blob: Vec<u8> = row.get(2)?;
            let tags_json: Option<String> = row.get(3)?;
            let created_at: i64 = row.get(4)?;
            let last_accessed: i64 = row.get(5)?;

            // Deserialize embedding
            let float_count = embedding_blob.len() / 4;
            let mut embedding = Vec::with_capacity(float_count);
            for i in 0..float_count {
                let f = f32::from_le_bytes([
                    embedding_blob[i * 4],
                    embedding_blob[i * 4 + 1],
                    embedding_blob[i * 4 + 2],
                    embedding_blob[i * 4 + 3],
                ]);
                embedding.push(f);
            }

            let tags: Vec<String> = tags_json
                .as_ref()
                .and_then(|j| serde_json::from_str(j).ok())
                .unwrap_or_default();

            Ok((key, value, embedding, tags, created_at, last_accessed))
        }).map_err(|e| crate::memory::MemoryError::Database(e))?;

        let mut results: Vec<(String, MemoryEntry, f32)> = Vec::new();

        for row_result in rows {
            if let Ok((key, value, embedding, tags, created_at, last_accessed)) = row_result {
                // Calculate cosine similarity
                let similarity = cosine_similarity(query_embedding, &embedding);
                
                let entry = MemoryEntry {
                    value,
                    created_at: created_at as u64,
                    last_accessed: last_accessed as u64,
                    embedding: Some(embedding),
                    tags,
                };
                
                results.push((key, entry, similarity));
            }
        }

        // Sort by similarity descending
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        results.truncate(top_k);
        Ok(results)
    }

    /// Returns the number of entries in persistent storage.
    pub async fn len(&self) -> usize {
        let conn = self.conn.read().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM memories",
            [],
            |row| row.get(0),
        ).unwrap_or(0)
    }

    /// Returns true if persistent storage is empty.
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    /// Clears all entries from persistent storage.
    pub async fn clear(&self) -> Result<(), crate::memory::MemoryError> {
        let conn = self.conn.write().unwrap();
        conn.execute("DELETE FROM memories", [])
            .map_err(|e| crate::memory::MemoryError::Database(e))?;
        Ok(())
    }

    /// Flushes pending writes to disk (checkpoint WAL).
    pub async fn flush(&self) -> Result<(), crate::memory::MemoryError> {
        let conn = self.conn.write().unwrap();
        conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")
            .map_err(|e| crate::memory::MemoryError::Database(e))?;
        Ok(())
    }

    /// Returns iterator over all entries (for batch processing).
    pub async fn iter(&self) -> Result<impl Iterator<Item = (String, MemoryEntry)>, crate::memory::MemoryError> {
        let conn = self.conn.read().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT key, value, embedding, tags, created_at, last_accessed FROM memories"
        ).map_err(|e| crate::memory::MemoryError::Database(e))?;

        let entries: Vec<(String, MemoryEntry)> = stmt
            .query_map([], |row| {
                let key: String = row.get(0)?;
                let value: String = row.get(1)?;
                let embedding_blob: Option<Vec<u8>> = row.get(2)?;
                let tags_json: Option<String> = row.get(3)?;
                let created_at: i64 = row.get(4)?;
                let last_accessed: i64 = row.get(5)?;

                let embedding = embedding_blob.map(|bytes| {
                    let float_count = bytes.len() / 4;
                    let mut vec = Vec::with_capacity(float_count);
                    for i in 0..float_count {
                        let f = f32::from_le_bytes([
                            bytes[i * 4],
                            bytes[i * 4 + 1],
                            bytes[i * 4 + 2],
                            bytes[i * 4 + 3],
                        ]);
                        vec.push(f);
                    }
                    vec
                });

                let tags: Vec<String> = tags_json
                    .as_ref()
                    .and_then(|j| serde_json::from_str(j).ok())
                    .unwrap_or_default();

                Ok((key, MemoryEntry {
                    value,
                    created_at: created_at as u64,
                    last_accessed: last_accessed as u64,
                    embedding,
                    tags,
                }))
            })
            .map_err(|e| crate::memory::MemoryError::Database(e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries.into_iter())
    }

    /// Evicts oldest entries if max_entries is exceeded.
    fn evict_if_needed(conn: &Connection, max_entries: usize) 
        -> Result<(), crate::memory::MemoryError> 
    {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memories",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        if count > max_entries as i64 {
            // Delete oldest entries (by created_at) to get down to max_entries
            let to_delete = count - max_entries as i64;
            conn.execute(
                "DELETE FROM memories WHERE key IN 
                 (SELECT key FROM memories ORDER BY created_at ASC LIMIT ?1)",
                params![to_delete],
            ).map_err(|e| crate::memory::MemoryError::Database(e))?;
        }

        Ok(())
    }

    /// Checks if the database connection is open.
    pub fn is_connected(&self) -> bool {
        self.is_open
    }
}

impl Drop for PersistentMemory {
    fn drop(&mut self) {
        // Ensure WAL is checkpointed on drop
        if let Ok(conn) = self.conn.write() {
            let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        }
        self.is_open = false;
    }
}

/// Calculates cosine similarity between two vectors.
///
/// # Arguments
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Returns
/// * `f32` - Cosine similarity between -1.0 and 1.0
///
/// # Note
/// Returns 0.0 if vectors have different lengths or zero magnitude.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

/// Returns the current Unix timestamp in milliseconds.
fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(value: &str) -> MemoryEntry {
        MemoryEntry::new(value.to_string())
    }

    #[tokio::test]
    async fn test_insert_get() {
        let config = DbConfig {
            path: ":memory:".to_string(),
            max_entries: 1000,
            embedding_dim: 4,
        };

        let mut memory = PersistentMemory::new(config).await.unwrap();
        memory.insert("key1", make_entry("value1")).await.unwrap();

        let retrieved = memory.get("key1").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, "value1");
    }

    #[tokio::test]
    async fn test_remove() {
        let config = DbConfig {
            path: ":memory:".to_string(),
            max_entries: 1000,
            embedding_dim: 4,
        };

        let mut memory = PersistentMemory::new(config).await.unwrap();
        memory.insert("key1", make_entry("value1")).await.unwrap();
        
        let removed = memory.remove("key1").await;
        assert!(removed);
        
        let retrieved = memory.get("key1").await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_len() {
        let config = DbConfig {
            path: ":memory:".to_string(),
            max_entries: 1000,
            embedding_dim: 4,
        };

        let mut memory = PersistentMemory::new(config).await.unwrap();
        assert_eq!(memory.len().await, 0);

        memory.insert("k1", make_entry("v1")).await.unwrap();
        memory.insert("k2", make_entry("v2")).await.unwrap();
        
        assert_eq!(memory.len().await, 2);
    }

    #[tokio::test]
    async fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 1.0);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        assert_eq!(cosine_similarity(&c, &d), 0.0);

        let e = vec![1.0, 0.0, 0.0];
        let f = vec![-1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&e, &f), -1.0);
    }
}
