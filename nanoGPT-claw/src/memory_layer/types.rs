use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub importance: f32,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub access_count: u64,
}

impl MemoryEntry {
    pub fn new(content: String, importance: f32) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            importance,
            category: None,
            tags: Vec::new(),
            created_at: now,
            accessed_at: now,
            access_count: 0,
        }
    }

    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn touch(&mut self) {
        self.accessed_at = Utc::now();
        self.access_count += 1;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryStats {
    pub total_entries: usize,
    pub avg_importance: f32,
    pub total_access_count: u64,
    pub short_term_count: usize,
    pub long_term_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    pub limit: usize,
    pub category: Option<String>,
    pub min_importance: Option<f32>,
    pub tags: Option<Vec<String>>,
    pub keyword: Option<String>,
}

impl Default for MemoryQuery {
    fn default() -> Self {
        Self {
            limit: 100,
            category: None,
            min_importance: None,
            tags: None,
            keyword: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub session_id: String,
    pub messages: Vec<ConversationEntry>,
    pub summary: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub tokens: Option<u32>,
}

impl ConversationContext {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            messages: Vec::new(),
            summary: None,
            created_at: Utc::now(),
        }
    }

    pub fn add_message(&mut self, role: String, content: String) {
        self.messages.push(ConversationEntry {
            role,
            content,
            timestamp: Utc::now(),
            tokens: None,
        });
    }
}
