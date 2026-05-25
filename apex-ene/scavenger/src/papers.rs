/// λΦ 论文猎食器
///
/// 自动从 arXiv、学术搜索引擎抓取最新论文。
/// 过滤出高相关度内容供吸收。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A discovered academic paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
    pub categories: Vec<String>,
    pub published: String,
    pub source: String,
    pub relevance_score: f64,
    pub absorption_status: PaperStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaperStatus {
    New,
    Reviewed,
    KeyInsightsExtracted,
    Absorbed,
    Rejected(String),
}

/// Core paper scavenger
pub struct PaperScavenger {
    pub papers: Vec<Paper>,
    /// Known topics to match against
    pub target_topics: Vec<String>,
}

impl PaperScavenger {
    pub fn new() -> Self {
        Self {
            papers: Vec::new(),
            target_topics: vec![
                "large language model".to_string(),
                "agent".to_string(),
                "self-improvement".to_string(),
                "evolution".to_string(),
                "reinforcement learning".to_string(),
                "memory".to_string(),
                "tool use".to_string(),
                "code generation".to_string(),
                "reasoning".to_string(),
                "autonomous".to_string(),
            ],
        }
    }

    /// Discover papers from a simulated arXiv query
    /// (In production, this would use the arXiv API)
    pub fn query_papers(&mut self, keyword: &str, max_results: usize) -> Vec<Paper> {
        // This is a heuristic/stub that generates paper entries
        // In production, replace with actual arXiv API query
        let mut results = Vec::new();

        for i in 0..max_results {
            let relevance = self.calculate_relevance(keyword, &format!("Paper {} about {}", i, keyword));
            let paper = Paper {
                id: format!("{}_{}", keyword.replace(' ', "_"), i),
                title: format!("{}. {}: A Novel Approach", i+1, keyword),
                authors: vec!["Auto-Discovered (AI)".to_string()],
                abstract_text: format!("This paper presents a novel approach to {} using advanced techniques. \
                    The method shows significant improvements over baselines. \
                    Key contributions include a new framework and empirical validation.", keyword),
                categories: vec!["cs.AI".to_string(), "cs.LG".to_string()],
                published: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                source: "arXiv".to_string(),
                relevance_score: relevance,
                absorption_status: PaperStatus::New,
            };
            results.push(paper);
        }

        self.papers.extend(results.clone());
        results
    }

    /// Filter papers by relevance threshold
    pub fn filter_relevant(&self, min_score: f64) -> Vec<&Paper> {
        self.papers.iter()
            .filter(|p| p.relevance_score >= min_score)
            .collect()
    }

    /// Get key insights from a paper (stub - would use LLM in production)
    pub fn extract_insights(&self, paper: &Paper) -> Vec<String> {
        vec![
            format!("Topic: {}", paper.title),
            "Related to agent/LLM capabilities".to_string(),
            "Potential for absorption into evolution framework".to_string(),
        ]
    }

    /// Calculate keyword relevance
    pub fn calculate_relevance(&self, keyword: &str, title: &str) -> f64 {
        let keyword_lower = keyword.to_lowercase();
        let title_lower = title.to_lowercase();

        let mut score: f64 = 0.5; // Base score

        // Boost if keyword appears in title
        if title_lower.contains(&keyword_lower) {
            score += 0.3;
        }

        // Boost if any target topic matches
        for topic in &self.target_topics {
            if title_lower.contains(&topic.to_lowercase()) {
                score += 0.1;
            }
        }

        score.min(1.0)
    }

    /// Generate a knowledge digest from all absorbed papers
    pub fn generate_digest(&self) -> HashMap<String, Vec<String>> {
        let mut digest: HashMap<String, Vec<String>> = HashMap::new();

        for paper in &self.papers {
            if matches!(paper.absorption_status, PaperStatus::Absorbed) {
                let insights = self.extract_insights(paper);
                digest.insert(paper.id.clone(), insights);
            }
        }

        digest
    }
}
