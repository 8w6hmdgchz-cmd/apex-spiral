use crate::memory_layer::{MemoryEntry, MemoryQuery};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use parking_lot::Mutex;
use tracing::{debug, info};

pub struct LongTermMemory {
    conn: Arc<Mutex<Connection>>,
    base_dir: std::path::PathBuf,
}

impl LongTermMemory {
    pub fn new(base_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(base_dir)?;
        let db_path = base_dir.join("memory.db");
        
        let conn = Connection::open(&db_path)
            .context("Failed to open SQLite database")?;

        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;
             PRAGMA busy_timeout=5000;"
        )?;

        let memory = Self {
            conn: Arc::new(Mutex::new(conn)),
            base_dir: base_dir.to_path_buf(),
        };
        
        memory.init()?;
        Ok(memory)
    }

    pub fn init(&self) -> Result<()> {
        let conn = self.conn.lock();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS memory_entries (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                importance REAL NOT NULL,
                category TEXT,
                tags TEXT,
                created_at TEXT NOT NULL,
                accessed_at TEXT NOT NULL,
                access_count INTEGER DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_importance ON memory_entries(importance DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_category ON memory_entries(category)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON memory_entries(created_at)",
            [],
        )?;

        info!("Long-term memory database initialized at {:?}", self.base_dir);
        Ok(())
    }

    pub fn store(&self, entry: &MemoryEntry) -> Result<()> {
        let conn = self.conn.lock();
        
        let tags_json = serde_json::to_string(&entry.tags)?;
        
        conn.execute(
            "INSERT OR REPLACE INTO memory_entries 
             (id, content, importance, category, tags, created_at, accessed_at, access_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.id,
                entry.content,
                entry.importance,
                entry.category,
                tags_json,
                entry.created_at.to_rfc3339(),
                entry.accessed_at.to_rfc3339(),
                entry.access_count as i64,
            ],
        )?;

        debug!("Stored entry in long-term memory: {}", entry.id);
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<MemoryEntry>> {
        let conn = self.conn.lock();
        
        let mut stmt = conn.prepare(
            "SELECT id, content, importance, category, tags, created_at, accessed_at, access_count
             FROM memory_entries WHERE id = ?1"
        )?;

        let entry_opt = stmt.query_row(params![id], |row| {
            Ok(Self::row_to_entry(row))
        }).optional()?;
        
        drop(stmt);
        drop(conn);

        if let Some(mut entry) = entry_opt {
            entry.touch();
            let _ = self.store(&entry);
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    pub fn query(&self, query: &MemoryQuery) -> Result<Option<Vec<MemoryEntry>>> {
        let conn = self.conn.lock();
        
        let mut sql = String::from(
            "SELECT id, content, importance, category, tags, created_at, accessed_at, access_count
             FROM memory_entries WHERE 1=1"
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref cat) = query.category {
            sql.push_str(" AND category = ?");
            params_vec.push(Box::new(cat.clone()));
        }
        
        if let Some(min_imp) = query.min_importance {
            sql.push_str(" AND importance >= ?");
            params_vec.push(Box::new(min_imp));
        }
        
        if let Some(ref kw) = query.keyword {
            sql.push_str(" AND content LIKE ?");
            params_vec.push(Box::new(format!("%{}%", kw)));
        }

        sql.push_str(" ORDER BY importance DESC, accessed_at DESC");
        sql.push_str(&format!(" LIMIT {}", query.limit));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        
        let mut stmt = conn.prepare(&sql)?;
        let entries = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(Self::row_to_entry(row))
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(Some(entries))
    }

    pub fn delete(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock();
        let affected = conn.execute("DELETE FROM memory_entries WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn get_stats(&self) -> Result<HashMap<String, serde_json::Value>> {
        let conn = self.conn.lock();
        
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memory_entries", [], |row| row.get(0)
        )?;
        
        let avg_importance: f64 = conn.query_row(
            "SELECT COALESCE(AVG(importance), 0) FROM memory_entries", [], |row| row.get(0)
        )?;

        let total_access: i64 = conn.query_row(
            "SELECT COALESCE(SUM(access_count), 0) FROM memory_entries", [], |row| row.get(0)
        )?;

        let mut stats = HashMap::new();
        stats.insert("total_entries".to_string(), serde_json::json!(total));
        stats.insert("avg_importance".to_string(), serde_json::json!(avg_importance));
        stats.insert("total_access_count".to_string(), serde_json::json!(total_access));
        
        Ok(stats)
    }

    pub fn compact(&self) -> Result<()> {
        let conn = self.conn.lock();
        
        conn.execute("VACUUM", [])?;
        conn.execute("ANALYZE", [])?;
        
        info!("Long-term memory database compacted");
        Ok(())
    }

    fn row_to_entry(row: &rusqlite::Row) -> MemoryEntry {
        let tags_str: String = row.get(4).unwrap_or_default();
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        
        let created_str: String = row.get(5).unwrap_or_default();
        let accessed_str: String = row.get(6).unwrap_or_default();
        
        MemoryEntry {
            id: row.get(0).unwrap_or_default(),
            content: row.get(1).unwrap_or_default(),
            importance: row.get(2).unwrap_or(0.5),
            category: row.get(3).ok(),
            tags,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            accessed_at: DateTime::parse_from_rfc3339(&accessed_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            access_count: row.get(7).unwrap_or(0) as u64,
        }
    }
}

impl Default for LongTermMemory {
    fn default() -> Self {
        Self::new(std::path::Path::new("./data/memory")).expect("Failed to create default long-term memory")
    }
}