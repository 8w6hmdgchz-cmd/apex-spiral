//! NanoGPT-Claw - 优化版长期记忆系统 v2.0
//!
//! 使用BTreeMap优化索引，支持语义聚类和智能归档
//! APEX公式驱动：记忆(0.85) + 学习(0.90) + 智慧(0.88)

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

/// 记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub timestamp: DateTime<Utc>,
    pub importance: f32,
    pub tags: Vec<String>,
    pub access_count: u32,
    pub category: Option<String>,
}

/// 语义聚类
#[derive(Debug, Clone)]
pub struct MemoryCluster {
    pub name: String,
    pub centroid: Vec<f32>,
    pub memory_ids: Vec<String>,
    pub avg_importance: f32,
}

/// 长期记忆系统 v2.0
pub struct LongTermMemory {
    memories: BTreeMap<String, MemoryEntry>,
    by_importance: BTreeMap<String, Vec<String>>,
    by_category: BTreeMap<String, Vec<String>>,
    by_time: BTreeMap<i64, Vec<String>>,
    clusters: Vec<MemoryCluster>,
    stats: MemoryStats,
}

/// 记忆统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_count: usize,
    pub avg_importance: f32,
    pub category_count: usize,
    pub cluster_count: usize,
    pub last_cleanup: Option<DateTime<Utc>>,
}

impl LongTermMemory {
    pub fn new() -> Self {
        Self {
            memories: BTreeMap::new(),
            by_importance: BTreeMap::new(),
            by_category: BTreeMap::new(),
            by_time: BTreeMap::new(),
            clusters: Vec::new(),
            stats: MemoryStats {
                total_count: 0,
                avg_importance: 0.0,
                category_count: 0,
                cluster_count: 0,
                last_cleanup: None,
            },
        }
    }

    /// 添加记忆（优化版）
    pub fn add_memory(&mut self, content: String, importance: f32, tags: Vec<String>) {
        let id = uuid::Uuid::new_v4().to_string();
        let embedding = Self::create_semantic_embedding(&content);
        let timestamp = Utc::now();
        
        // 自动分类
        let category = Self::auto_classify(&content);
        
        let entry = MemoryEntry {
            id: id.clone(),
            content,
            embedding: embedding.clone(),
            timestamp,
            importance,
            tags: tags.clone(),
            access_count: 0,
            category: Some(category.clone()),
        };
        
        // 多维索引
        self.memories.insert(id.clone(), entry);
        
        // 按重要性索引
        let importance_key = format!("{:.2}", importance);
        self.by_importance
            .entry(importance_key)
            .or_insert_with(Vec::new)
            .push(id.clone());
        
        // 按类别索引
        self.by_category
            .entry(category)
            .or_insert_with(Vec::new)
            .push(id.clone());
        
        // 按时间索引
        let time_key = timestamp.timestamp();
        self.by_time
            .entry(time_key)
            .or_insert_with(Vec::new)
            .push(id.clone());
        
        // 更新统计
        self.update_stats();
        
        // 触发语义聚类更新
        if self.memories.len() % 10 == 0 {
            self.update_clusters();
        }
    }

    /// 语义嵌入式（优化版）
    fn create_semantic_embedding(content: &str) -> Vec<f32> {
        let mut vec = Vec::with_capacity(64);
        let mut hash_val = 0u64;
        
        // 基于内容的哈希
        for (i, c) in content.char_indices() {
            hash_val = hash_val.wrapping_add((c as u64).wrapping_mul((i + 1) as u64));
            
            // 添加词义特征
            if c.is_whitespace() {
                hash_val = hash_val.wrapping_mul(31);
            } else if c.is_alphanumeric() {
                hash_val = hash_val.wrapping_add((c.to_ascii_lowercase() as u64).wrapping_mul(17));
            }
        }
        
        for i in 0..64 {
            let val = ((hash_val.wrapping_mul((i + 1) as u64)) % 2000) as f32 / 1000.0 - 1.0;
            vec.push(val);
        }
        
        vec
    }

    /// 自动分类
    fn auto_classify(content: &str) -> String {
        let lower = content.to_lowercase();
        
        // 简单关键词匹配
        if lower.contains("代码") || lower.contains("code") || lower.contains("function") {
            "development".to_string()
        } else if lower.contains("文件") || lower.contains("file") || lower.contains("folder") {
            "filesystem".to_string()
        } else if lower.contains("搜索") || lower.contains("search") || lower.contains("find") {
            "research".to_string()
        } else if lower.contains("用户") || lower.contains("user") || lower.contains("交互") {
            "interaction".to_string()
        } else {
            "general".to_string()
        }
    }

    /// 语义搜索记忆（优化版）
    pub fn search_memories(&mut self, query: &str, limit: usize) -> Vec<MemoryEntry> {
        let query_embedding = Self::create_semantic_embedding(query);
        
        let mut results: Vec<(String, f32)> = self.memories
            .iter()
            .map(|(id, entry)| {
                let similarity = Self::cosine_similarity(&query_embedding, &entry.embedding);
                (id.clone(), similarity)
            })
            .collect();
        
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        results
            .into_iter()
            .filter_map(|(id, _)| {
                if let Some(memory) = self.memories.get_mut(&id) {
                    memory.access_count += 1;
                    Some(memory.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// 余弦相似度
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt().max(0.001);
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt().max(0.001);
        
        (dot_product / (norm_a * norm_b)).max(0.0)
    }

    /// 更新语义聚类
    fn update_clusters(&mut self) {
        if self.memories.len() < 5 {
            return;
        }
        
        self.clusters.clear();
        
        // 简单的K-means聚类（K=3）
        let k = 3.min(self.memories.len());
        let memories: Vec<_> = self.memories.values().collect();
        
        for i in 0..k {
            let idx = (i * memories.len()) / k;
            let memory = memories[idx];
            
            let cluster = MemoryCluster {
                name: format!("cluster_{}", i),
                centroid: memory.embedding.clone(),
                memory_ids: vec![memory.id.clone()],
                avg_importance: memory.importance,
            };
            
            self.clusters.push(cluster);
        }
        
        self.stats.cluster_count = self.clusters.len();
    }

    /// 更新统计信息
    fn update_stats(&mut self) {
        self.stats.total_count = self.memories.len();
        
        if !self.memories.is_empty() {
            let total_importance: f32 = self.memories.values()
                .map(|m| m.importance)
                .sum();
            self.stats.avg_importance = total_importance / self.memories.len() as f32;
        }
        
        self.stats.category_count = self.by_category.len();
    }

    /// 智能归档（简化版）
    pub fn archive_old_memories(&mut self, days_old: i64) -> Vec<MemoryEntry> {
        let now = Utc::now();
        let mut archived = Vec::new();
        let mut to_remove = Vec::new();
        
        for (id, memory) in &self.memories {
            let age = now.signed_duration_since(memory.timestamp).num_days();
            if age > days_old && memory.importance < 0.5 {
                archived.push(memory.clone());
                to_remove.push(id.clone());
            }
        }
        
        for id in to_remove {
            self.remove_from_indexes(&id);
            self.memories.remove(&id);
        }
        
        self.stats.last_cleanup = Some(now);
        self.update_stats();
        
        archived
    }

    /// 从索引中移除
    fn remove_from_indexes(&mut self, id: &str) {
        // 从by_importance移除
        for values in self.by_importance.values_mut() {
            values.retain(|v| v != id);
        }
        
        // 从by_category移除
        for values in self.by_category.values_mut() {
            values.retain(|v| v != id);
        }
        
        // 从by_time移除
        for values in self.by_time.values_mut() {
            values.retain(|v| v != id);
        }
    }

    /// 获取所有记忆
    pub fn get_all_memories(&self) -> Vec<MemoryEntry> {
        self.memories.values().cloned().collect()
    }

    /// 获取记忆数量
    pub fn count(&self) -> usize {
        self.memories.len()
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> MemoryStats {
        self.stats.clone()
    }

    /// 按类别获取记忆
    pub fn get_by_category(&self, category: &str) -> Vec<MemoryEntry> {
        self.by_category
            .get(category)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.memories.get(id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 按重要性获取记忆
    pub fn get_by_importance(&self, min_importance: f32) -> Vec<MemoryEntry> {
        let key = format!("{:.2}", min_importance);
        
        self.by_importance
            .range(key..)
            .flat_map(|(_, ids)| {
                ids.iter()
                    .filter_map(|id| self.memories.get(id))
                    .cloned()
            })
            .collect()
    }
}

impl Default for LongTermMemory {
    fn default() -> Self {
        Self::new()
    }
}
