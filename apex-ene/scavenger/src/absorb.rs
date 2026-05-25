/// λΦ 知识吸收引擎
///
/// 将猎食到的资源过滤、提纯、蒸馏为可用知识。
/// 冗余剔除：不吸收重复或低质量内容。

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A knowledge fragment ready for absorption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeFragment {
    pub id: String,
    pub source: String,
    pub source_type: SourceType,
    pub title: String,
    pub content_summary: String,
    pub key_insights: Vec<String>,
    pub tags: Vec<String>,
    pub quality_score: f64,
    pub novelty_score: f64,
    pub absorbed: bool,
    pub absorbed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    GitHub,
    Paper,
    Docs,
    Skill,
    XVArchitecture,
}

/// Absorption engine
pub struct AbsorptionEngine {
    pub fragments: Vec<KnowledgeFragment>,
    absorbed_ids: HashSet<String>,
    quality_threshold: f64,
    novelty_threshold: f64,
}

impl AbsorptionEngine {
    pub fn new(quality_threshold: f64, novelty_threshold: f64) -> Self {
        Self {
            fragments: Vec::new(),
            absorbed_ids: HashSet::new(),
            quality_threshold,
            novelty_threshold,
        }
    }

    /// Evaluate a fragment for absorption
    pub fn evaluate(&self, fragment: &KnowledgeFragment) -> AbsorptionVerdict {
        let mut reasons = Vec::new();

        // Duplicate check
        if self.absorbed_ids.contains(&fragment.id) {
            return AbsorptionVerdict::Reject("Already absorbed (duplicate)".to_string());
        }

        // Quality gate
        if fragment.quality_score < self.quality_threshold {
            reasons.push(format!(
                "Quality {:.2} < threshold {:.2}",
                fragment.quality_score, self.quality_threshold
            ));
        }

        // Novelty gate
        if fragment.novelty_score < self.novelty_threshold {
            reasons.push(format!(
                "Novelty {:.2} < threshold {:.2}",
                fragment.novelty_score, self.novelty_threshold
            ));
        }

        if reasons.is_empty() {
            AbsorptionVerdict::Absorb
        } else {
            AbsorptionVerdict::Reject(reasons.join("; "))
        }
    }

    /// Absorb a fragment if it passes all gates
    pub fn absorb(&mut self, mut fragment: KnowledgeFragment) -> AbsorptionResult {
        let verdict = self.evaluate(&fragment);

        match verdict {
            AbsorptionVerdict::Absorb => {
                fragment.absorbed = true;
                fragment.absorbed_at = Some(chrono::Utc::now().to_rfc3339());
                self.absorbed_ids.insert(fragment.id.clone());
                self.fragments.push(fragment);
                AbsorptionResult::Success
            }
            AbsorptionVerdict::Reject(reason) => {
                AbsorptionResult::Rejected(reason)
            }
        }
    }

    /// Batch absorb multiple fragments
    pub fn absorb_batch(&mut self, fragments: Vec<KnowledgeFragment>) -> BatchAbsorptionReport {
        let mut accepted = 0;
        let mut rejected = 0;
        let mut rejections = Vec::new();

        for fragment in fragments {
            match self.absorb(fragment) {
                AbsorptionResult::Success => accepted += 1,
                AbsorptionResult::Rejected(reason) => {
                    rejected += 1;
                    rejections.push(reason);
                }
            }
        }

        BatchAbsorptionReport {
            total: accepted + rejected,
            accepted,
            rejected,
            rejections,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Remove redundant fragments (keep highest quality per tag)
    pub fn deduplicate(&mut self) -> usize {
        let mut removed = 0;
        let mut tag_groups: std::collections::HashMap<String, Vec<usize>> = 
            std::collections::HashMap::new();

        for (i, fragment) in self.fragments.iter().enumerate() {
            for tag in &fragment.tags {
                tag_groups.entry(tag.clone()).or_default().push(i);
            }
        }

        // Keep only the best per tag group
        let mut to_remove = HashSet::new();
        for (_tag, indices) in tag_groups {
            if indices.len() > 1 {
                // Keep the one with highest quality
                let mut best_idx = indices[0];
                for &idx in &indices {
                    if self.fragments[idx].quality_score > self.fragments[best_idx].quality_score {
                        best_idx = idx;
                    }
                }
                for &idx in &indices {
                    if idx != best_idx {
                        to_remove.insert(idx);
                    }
                }
            }
        }

        // Remove duplicates (in reverse order to preserve indices)
        let mut sorted: Vec<usize> = to_remove.into_iter().collect();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        for idx in sorted {
            self.fragments.remove(idx);
            removed += 1;
        }

        removed
    }

    /// Generate an absorption report
    pub fn report(&self) -> AbsorptionReport {
        AbsorptionReport {
            total_fragments: self.fragments.len(),
            absorbed_count: self.fragments.iter().filter(|f| f.absorbed).count(),
            unique_ids: self.absorbed_ids.len(),
            quality_stats: if self.fragments.is_empty() {
                QualityStats::default()
            } else {
                let scores: Vec<f64> = self.fragments.iter().map(|f| f.quality_score).collect();
                let avg = scores.iter().sum::<f64>() / scores.len() as f64;
                let min = scores.iter().cloned().fold(f64::MAX, f64::min);
                let max = scores.iter().cloned().fold(f64::MIN, f64::max);
                QualityStats { avg, min, max }
            },
            tags: {
                let mut tags: Vec<String> = self.fragments.iter()
                    .flat_map(|f| f.tags.clone())
                    .collect();
                tags.sort_unstable();
                tags.dedup();
                tags
            },
        }
    }
}

#[derive(Debug)]
pub enum AbsorptionVerdict {
    Absorb,
    Reject(String),
}

#[derive(Debug)]
pub enum AbsorptionResult {
    Success,
    Rejected(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchAbsorptionReport {
    pub total: usize,
    pub accepted: usize,
    pub rejected: usize,
    pub rejections: Vec<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AbsorptionReport {
    pub total_fragments: usize,
    pub absorbed_count: usize,
    pub unique_ids: usize,
    pub quality_stats: QualityStats,
    pub tags: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct QualityStats {
    pub avg: f64,
    pub min: f64,
    pub max: f64,
}
