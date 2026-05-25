// emv_skill/src/skill_selector.rs
// SkillSelector API - 基于特征向量的技能路由选择器
// 璇玑帝国 APEX · Rust实现
//
// 核心机制:
// - 基于特征向量 [f32] 输入，选择最优技能
// - 计算 delta_gini (基尼增益)
// - 投票分布 + 置信度输出
// - 软路由：返回完整决策结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{gini_gain, SkillGene};

/// 技能决策结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDecision {
    /// 选中的技能ID
    pub skill_id: String,
    /// 置信度 [0.0, 1.0]
    pub confidence: f32,
    /// 各技能的投票计数 {skill_id: vote_count}
    pub vote_distribution: HashMap<String, u32>,
    /// 基尼增益 ΔGini
    pub delta_gini: f32,
}

/// SkillSelector - 多模型投票 + Gini增益选择
pub struct SkillSelector {
    /// 注册的技能列表
    skills: Vec<SkillGene>,
    /// 特征维度数量（与输入特征向量长度对应）
    feature_dim: usize,
    /// 投票阈值
    _vote_threshold: u32,
}

impl SkillSelector {
    /// 新建选择器
    pub fn new() -> Self {
        Self {
            skills: Vec::new(),
            feature_dim: 0,
            _vote_threshold: 1,
        }
    }

    /// 从 SkillGene 列表构建选择器
    pub fn from_genes(genes: Vec<SkillGene>) -> Self {
        let feature_dim = if genes.is_empty() { 0 } else { 4 }; // 默认4维特征
        Self {
            skills: genes,
            feature_dim,
            _vote_threshold: 1,
        }
    }

    /// 注册技能
    pub fn register(&mut self, gene: SkillGene) {
        self.feature_dim = 4; // success_rate, fitness, generation, total_reward
        self.skills.push(gene);
    }

    /// 注册多个技能
    pub fn register_all(&mut self, genes: Vec<SkillGene>) {
        for gene in genes {
            self.register(gene);
        }
    }

    /// 提取技能的特征向量
    fn extract_features(&self, gene: &SkillGene) -> Vec<f32> {
        vec![
            gene.success_rate() as f32,
            gene.fitness() as f32,
            gene.generation as f32,
            gene.total_reward as f32,
        ]
    }

    /// 余弦相似度
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }
        dot / (mag_a * mag_b)
    }

    /// 欧几里得距离
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// 归一化特征向量
    /// 提取输入特征向量的统计特征（用于与技能基因比较）
    fn extract_input_features(&self, input: &[f32]) -> Vec<f32> {
        if input.is_empty() {
            return vec![0.0, 0.0, 0.0, 0.0];
        }
        let n = input.len() as f32;
        let sum: f32 = input.iter().sum();
        let mean = sum / n;
        let variance = input.iter().map(|x| { let d = x - mean; d * d }).sum::<f32>() / n;
        let max_val = input.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min_val = input.iter().cloned().fold(f32::INFINITY, f32::min);
        let range = max_val - min_val;
        let energy: f32 = input.iter().map(|x| x * x).sum::<f32>() / n;
        vec![
            mean,
            variance.sqrt(),
            energy,
            range,
        ]
    }

    /// 计算候选技能与输入特征的 delta_gini
    fn compute_delta_gini(&self, gene: &SkillGene, input_features: &[f32]) -> f32 {
        // 使用统计特征（维度匹配）
        let input_stats = self.extract_input_features(input_features);
        let gene_features = self.extract_features(gene);
        
        // 余弦相似度
        let similarity = self.cosine_similarity(&input_stats, &gene_features);
        
        // 额外考虑intensity差异
        let input_energy = input_stats[2];
        let gene_energy = gene_features[2]; // total_reward 作为能量代理
        let energy_ratio = (input_energy / (gene_energy.max(0.01))).min(2.0);
        
        // 综合delta_gini
        let base = (similarity + 1.0) / 2.0;
        let adjusted = base * (0.7 + 0.3 * energy_ratio);
        adjusted.min(1.0).max(0.0)
    }

    /// 多策略投票
    fn multi_strategy_vote(&self, input_features: &[f32]) -> HashMap<String, u32> {
        let mut votes: HashMap<String, u32> = HashMap::new();
        
        // 使用统计特征提取代替归一化（保留幅值信息）
        let input_stats = self.extract_input_features(input_features);
        let input_mean = input_stats[0];
        let input_intensity = input_stats[2]; // energy

        for gene in &self.skills {
            let gene_features = self.extract_features(gene);

            // 策略1: 余弦相似度（基于统计特征）
            let cos_sim = self.cosine_similarity(&input_stats, &gene_features);
            if cos_sim > 0.3 {
                *votes.entry(gene.gene_id.clone()).or_insert(0) += 1;
            }

            // 策略2: 幅值感知投票 - 高强度偏好Explore
            if input_intensity > 0.5 {
                if gene.gene_id.contains("replay") || gene.gene_id.contains("explore") {
                    *votes.entry(gene.gene_id.clone()).or_insert(0) += 1;
                }
            }

            // 策略3: fitness 加权投票
            let fitness_score = gene.fitness() as f32;
            if fitness_score > 0.5 {
                *votes.entry(gene.gene_id.clone()).or_insert(0) += 1;
            }

            // 策略4: 基于intensity的差异化投票
            if input_mean > 0.7 {
                // 高mean偏好特定技能
                if gene.gene_id.contains("apex") || gene.gene_id.contains("repair") {
                    *votes.entry(gene.gene_id.clone()).or_insert(0) += 1;
                }
            } else if input_mean > 0.4 {
                // 中等mean偏好gini
                if gene.gene_id.contains("gini") {
                    *votes.entry(gene.gene_id.clone()).or_insert(0) += 1;
                }
            }
        }

        votes
    }

    /// 核心API: 从特征向量选择技能
    pub fn select_skill(&self, features: &[f32]) -> SkillDecision {
        if self.skills.is_empty() {
            return SkillDecision {
                skill_id: String::new(),
                confidence: 0.0,
                vote_distribution: HashMap::new(),
                delta_gini: 0.0,
            };
        }

        let vote_distribution = self.multi_strategy_vote(features);

        // 找出得票最高的技能
        let (best_skill_id, max_votes) = vote_distribution
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.clone(), *v))
            .unwrap_or_else(|| (self.skills[0].gene_id.clone(), 0));

        // 计算置信度：基于得票率
        let total_voters = vote_distribution.values().sum::<u32>();
        let confidence = if total_voters > 0 {
            max_votes as f32 / total_voters as f32
        } else {
            0.0
        };

        // 计算 delta_gini
        let best_gene = self.skills
            .iter()
            .find(|g| g.gene_id == best_skill_id)
            .unwrap_or(&self.skills[0]);
        let delta_gini = self.compute_delta_gini(best_gene, features);

        SkillDecision {
            skill_id: best_skill_id,
            confidence,
            vote_distribution,
            delta_gini,
        }
    }

    /// 基于 Gini CART 树的分裂选择
    pub fn cart_select(&self, features: &[f32], threshold: f32) -> SkillDecision {
        // CART 风格：基于 success_rate 分裂
        let mut candidates: Vec<&SkillGene> = self.skills.iter().collect();

        // 按 success_rate 排序
        candidates.sort_by(|a, b| {
            a.success_rate()
                .partial_cmp(&b.success_rate())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let n = candidates.len();
        if n == 0 {
            return self.select_skill(features);
        }

        // 找最优分裂点
        let mut best_gain = f32::MIN;
        let mut best_idx = 0;

        for i in 1..n {
            let left: Vec<f64> = candidates[..i]
                .iter()
                .flat_map(|g| {
                    if g.success_count > 0 {
                        vec![1.0]
                    } else {
                        vec![0.0]
                    }
                })
                .collect();
            let right: Vec<f64> = candidates[i..]
                .iter()
                .flat_map(|g| {
                    if g.success_count > 0 {
                        vec![1.0]
                    } else {
                        vec![0.0]
                    }
                })
                .collect();
            let parent: Vec<f64> = candidates
                .iter()
                .flat_map(|g| {
                    if g.success_count > 0 {
                        vec![1.0]
                    } else {
                        vec![0.0]
                    }
                })
                .collect();

            let gain = gini_gain(
                &parent,
                &left,
                &right,
            ) as f32;

            if gain > best_gain {
                best_gain = gain;
                best_idx = i;
            }
        }

        // 取左子集或右子集（基于阈值和特征比较）
        let split_gene = if features.is_empty() || features[0] < threshold {
            &candidates[best_idx.min(candidates.len() - 1)]
        } else {
            candidates.last().unwrap_or(&candidates[0])
        };

        let _gene_features = self.extract_features(split_gene);
        let delta_gini = self.compute_delta_gini(split_gene, features);

        SkillDecision {
            skill_id: split_gene.gene_id.clone(),
            confidence: best_gain.max(0.0).min(1.0),
            vote_distribution: HashMap::from([(split_gene.gene_id.clone(), 1)]),
            delta_gini,
        }
    }

    /// 获取所有技能
    pub fn all_skills(&self) -> &[SkillGene] {
        &self.skills
    }

    /// 获取技能数量
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// 判断是否为空
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

impl Default for SkillSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// 顶级 API: 根据特征向量选择最优技能
///
/// # Arguments
/// * `features` - 特征向量 [success_rate, fitness, generation, total_reward]
///
/// # Returns
/// * `SkillDecision` - 包含 skill_id, confidence, vote_distribution, delta_gini
///
/// # Example
/// ```
/// use emv_skill::{select_skill, SkillGene, SkillSelector};
/// let features = vec![0.8, 0.7, 1.0, 10.0];
/// let decision = select_skill(&features);
/// ```
pub fn select_skill(features: &[f32]) -> SkillDecision {
    // 使用默认/空的 selector（在没有注册技能时返回空决策）
    let selector = SkillSelector::new();
    selector.select_skill(features)
}

/// 带技能库的选择器 API
pub fn select_skill_with_genes(features: &[f32], genes: Vec<SkillGene>) -> SkillDecision {
    let selector = SkillSelector::from_genes(genes);
    selector.select_skill(features)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SkillGene;

    #[test]
    fn test_select_skill_empty() {
        let selector = SkillSelector::new();
        let decision = selector.select_skill(&[0.5, 0.5, 0.0, 0.0]);
        assert_eq!(decision.skill_id, "");
        assert_eq!(decision.confidence, 0.0);
    }

    #[test]
    fn test_select_skill_with_genes() {
        let genes = vec![
            SkillGene::new("skill_a", "测试技能A", "action_a"),
            SkillGene::new("skill_b", "测试技能B", "action_b"),
        ];
        let mut selector = SkillSelector::new();
        selector.register_all(genes);

        let features = vec![0.8, 0.7, 1.0, 10.0];
        let decision = selector.select_skill(&features);
        assert!(!decision.skill_id.is_empty());
        assert!(decision.confidence >= 0.0 && decision.confidence <= 1.0);
    }

    #[test]
    fn test_cosine_similarity() {
        let genes = vec![SkillGene::new("s1", "desc", "act")];
        let mut selector = SkillSelector::from_genes(genes);
        let sim = selector.cosine_similarity(&[1.0, 0.0], &[1.0, 0.0]);
        assert!((sim - 1.0).abs() < 0.001);

        let sim2 = selector.cosine_similarity(&[1.0, 0.0], &[-1.0, 0.0]);
        assert!((sim2 - (-1.0)).abs() < 0.001);
    }
}
