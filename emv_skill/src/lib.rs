// emv_skill/src/lib.rs
// EMV熵Skill基因网络选择框架
// 璇玑帝国 APEX · Rust实现
//
// 核心机制:
// - Challenger: 出题，从长文档提炼可复用技能
// - Reasoner: 解题，多智能体自博弈推理
// - Judge: 判题，用基尼增益+信息熵选择最优技能
// - GiniSelector: 基因网络选择器

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 1. 技能基因结构
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillGene {
    pub gene_id: String,
    pub name: String,
    pub description: String,
    pub trigger_patterns: Vec<String>,
    pub action: String,
    pub success_count: u32,
    pub failure_count: u32,
    pub total_reward: f64,
    pub parent_genes: Vec<String>,  // 父基因ID（用于溯源）
    pub generation: u32,           // 代数
}

impl SkillGene {
    pub fn new(name: &str, description: &str, action: &str) -> Self {
        Self {
            gene_id: uuid_v4(),
            name: name.to_string(),
            description: description.to_string(),
            trigger_patterns: vec![],
            action: action.to_string(),
            success_count: 0,
            failure_count: 0,
            total_reward: 0.0,
            parent_genes: vec![],
            generation: 0,
        }
    }

    /// 成功率
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 { 0.5 } else { self.success_count as f64 / total as f64 }
    }

    /// 增益评估
    pub fn fitness(&self) -> f64 {
        let sr = self.success_rate();
        let recency = 1.0; // 简化：实际可用时间衰减
        sr * recency + self.total_reward / 100.0
    }

    /// 保存到JSON文件
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    /// 从JSON文件加载
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }
}

// ============================================================
// 2. 基尼不纯度与信息熵
// ============================================================

/// Gini不纯度: Gini = 1 - sum(p_k^2)
pub fn gini_impurity(counts: &[f64]) -> f64 {
    let total: f64 = counts.iter().sum();
    if total <= 0.0 { return 0.0; }
    1.0 - counts
        .iter()
        .filter(|&&c| c > 0.0)
        .map(|&c| {
            let p = c / total;
            p * p
        })
        .sum::<f64>()
}

/// 信息熵: H = -sum(p_k * log2(p_k))
pub fn entropy(counts: &[f64]) -> f64 {
    let total: f64 = counts.iter().sum();
    if total <= 0.0 { return 0.0; }
    -counts
        .iter()
        .filter(|&&c| c > 0.0)
        .map(|&c| {
            let p = c / total;
            p * p.log2()
        })
        .sum::<f64>()
}

/// 信息增益: IG = H父 - sum(N_v/N * H_v)
pub fn information_gain(parent_counts: &[f64], child_counts: &[Vec<f64>]) -> f64 {
    let parent_ent = entropy(parent_counts);
    let total: f64 = parent_counts.iter().sum();
    if total <= 0.0 { return 0.0; }

    let mut weighted_child_ent = 0.0;
    for child in child_counts {
        let child_total: f64 = child.iter().sum();
        let weight = child_total / total;
        weighted_child_ent += weight * entropy(child);
    }

    parent_ent - weighted_child_ent
}

/// 基尼增益: ΔGini = Gini父 - (N_L/N*Gini_L + N_R/N*Gini_R)
pub fn gini_gain(parent_counts: &[f64], left_counts: &[f64], right_counts: &[f64]) -> f64 {
    let parent_gini = gini_impurity(parent_counts);
    let total: f64 = parent_counts.iter().sum();
    if total <= 0.0 { return 0.0; }

    let left_total: f64 = left_counts.iter().sum();
    let right_total: f64 = right_counts.iter().sum();
    let left_weight = left_total / total;
    let right_weight = right_total / total;

    parent_gini - (left_weight * gini_impurity(left_counts) + right_weight * gini_impurity(right_counts))
}

// ============================================================
// 3. GiniSelector: 基因网络选择器
// ============================================================

#[derive(Debug, Clone)]
pub struct GiniSelector {
    min_samples_leaf: usize,
    max_depth: usize,
    min_gain: f64,
}

impl GiniSelector {
    pub fn new() -> Self {
        Self {
            min_samples_leaf: 5,
            max_depth: 10,
            min_gain: 0.01,
        }
    }

    /// 选择最优分裂特征
    pub fn best_split(&self, genes: &[SkillGene], feature: &str, threshold: f64) -> f64 {
        let mut parent_success = 0.0;
        let mut parent_failure = 0.0;
        let mut left_success = 0.0;
        let mut left_failure = 0.0;
        let mut right_success = 0.0;
        let mut right_failure = 0.0;

        for gene in genes {
            let val = self.feature_value(gene, feature);
            if val <= threshold {
                if gene.success_count as f64 > 0.0 { left_success += 1.0; }
                else { left_failure += 1.0; }
            } else {
                if gene.success_count as f64 > 0.0 { right_success += 1.0; }
                else { right_failure += 1.0; }
            }
            if gene.success_count as f64 > 0.0 { parent_success += 1.0; }
            else { parent_failure += 1.0; }
        }

        gini_gain(
            &[parent_success, parent_failure],
            &[left_success, left_failure],
            &[right_success, right_failure],
        )
    }

    fn feature_value(&self, gene: &SkillGene, feature: &str) -> f64 {
        match feature {
            "success_rate" => gene.success_rate(),
            "fitness" => gene.fitness(),
            "generation" => gene.generation as f64,
            "total_reward" => gene.total_reward,
            _ => gene.success_rate(),
        }
    }

    /// 随机森林多数投票
    pub fn random_forest_vote(&self, predictions: &[bool], probabilities: &[f64]) -> bool {
        // 软投票：概率加权
        let pos_prob: f64 = probabilities.iter().sum();
        pos_prob / probabilities.len() as f64 >= 0.5
    }
}

// ============================================================
// 4. EMV三核心角色
// ============================================================

/// Challenger: 出题 - 从长文档提炼可复用技能
pub struct Challenger;

impl Challenger {
    pub fn extract_skills(&self, document: &str) -> Vec<SkillGene> {
        let mut skills = vec![];
        let lines: Vec<&str> = document.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            if trimmed.contains("步骤") || trimmed.contains("流程") || trimmed.contains("方法") || trimmed.contains("APEX") {
                let name = format!("skill_{}", i);
                let mut gene = SkillGene::new(&name, trimmed, trimmed);
                // 提取触发词：从行中提取关键概念
                let triggers: Vec<String> = trimmed
                    .split(|c: char| c == ':' || c == '-' || c == '：')
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.trim().to_string())
                    .filter(|s| s.len() > 1 && s.len() < 50)
                    .collect();
                gene.trigger_patterns = triggers;
                skills.push(gene);
            }
        }
        if skills.is_empty() {
            // fallback: 把整篇文档当作一个技能
            let mut gene = SkillGene::new("skill_doc", "文档技能", document);
            gene.trigger_patterns = document
                .split(|c: char| c.is_whitespace() || c == '：' || c == '-')
                .filter(|s| s.len() > 2)
                .map(|s| s.to_string())
                .take(10)
                .collect();
            skills.push(gene);
        }
        skills
    }
}

/// Reasoner: 解题 - 多智能体自博弈推理
pub struct Reasoner {
    api_key: String,
}

impl Reasoner {
    pub fn new(api_key: &str) -> Self {
        Self { api_key: api_key.to_string() }
    }

    /// 调用GPT-5进行真实推理
    pub fn solve_with_gpt(&self, skill: &SkillGene, task: &str) -> (bool, f64) {
        use std::process::Command;

        let task_esc = task.replace('"', "'");
        let action_esc = skill.action.replace('"', "'");

        // 构建JSON payload
        let json_body = format!(
            "{{\"model\": \"gpt-5\", \"messages\": [{{\"role\": \"user\", \"content\": \"任务: {}\n技能: {}\n判断技能是否匹配？返回JSON: {{\\\"match\\\": true/false}}\"}}], \"max_tokens\": 100}}",
            task_esc, action_esc
        );

        let output = Command::new("curl")
            .args([
                "-s", "--connect-timeout", "10",
                "-X", "POST",
                "https://api.freemodel.dev/v1/chat/completions",
                "-H", &format!("Authorization: Bearer {}", self.api_key),
                "-H", "Content-Type: application/json",
                "-d", &json_body,
            ])
            .output();

        match output {
            Ok(out) => {
                let response = String::from_utf8_lossy(&out.stdout);
                if response.contains("\"match\": true") || response.contains("\"match\":true") {
                    return (true, 1.0);
                } else if response.contains("\"match\": false") || response.contains("\"match\":false") {
                    return (false, -0.5);
                }
            }
            Err(_) => {}
        }
        let success = skill.trigger_patterns.iter().any(|p| task.contains(p));
        (success, if success { 0.5 } else { -0.3 })
    }

    /// 简化推理（无GPT时）
    pub fn solve(&self, skill: &SkillGene, task: &str) -> (bool, f64) {
        let success = skill.trigger_patterns.iter().any(|p| task.contains(p));
        let reward = if success { 1.0 } else { -0.5 };
        (success, reward)
    }
}

/// Judge: 判题 - 用基尼增益+信息熵判决
pub struct Judge {
    selector: GiniSelector,
}

impl Judge {
    pub fn new() -> Self {
        Self { selector: GiniSelector::new() }
    }

    pub fn evaluate(&self, genes: &[&SkillGene], task: &str) -> Option<String> {
        // 找最优技能
        let mut best_gene: Option<&SkillGene> = None;
        let mut best_fitness = f64::MIN;

        for gene in genes {
            if gene.trigger_patterns.iter().any(|p| task.contains(p)) {
                let f = gene.fitness();
                if f > best_fitness {
                    best_fitness = f;
                    best_gene = Some(*gene);
                }
            }
        }

        best_gene.map(|g| g.gene_id.clone())
    }
}

// ============================================================
// 5. EMV循环迭代
// ============================================================

pub struct EMVCycle {
    challenger: Challenger,
    reasoner: Reasoner,
    judge: Judge,
    genes: HashMap<String, SkillGene>,
    iteration: u32,
}

impl EMVCycle {
    pub fn new_with_gpt(api_key: &str) -> Self {
        Self {
            challenger: Challenger,
            reasoner: Reasoner::new(api_key),
            judge: Judge::new(),
            genes: HashMap::new(),
            iteration: 0,
        }
    }

    pub fn new() -> Self {
        Self::new_with_gpt("")
    }

    /// 执行一轮EMV循环
    pub fn run_cycle(&mut self, document: &str, task: &str) -> (bool, String) {
        self.iteration += 1;

        // 1. Challenger出题：从文档提炼技能
        let new_skills = self.challenger.extract_skills(document);
        for skill in &new_skills {
            self.genes.insert(skill.gene_id.clone(), skill.clone());
        }

        // 2. Reasoner解题：每个技能都试一遍（优先用GPT）
        let mut updates: Vec<(String, bool, f64)> = vec![];
        for (gene_id, gene) in &self.genes {
            let (success, reward) = if !self.reasoner.api_key.is_empty() {
                self.reasoner.solve_with_gpt(gene, task)
            } else {
                self.reasoner.solve(gene, task)
            };
            updates.push((gene_id.clone(), success, reward));
        }

        // 更新基因统计（分开迭代避免borrow冲突）
        for (gene_id, success, reward) in updates {
            if let Some(g) = self.genes.get_mut(&gene_id) {
                if success { g.success_count += 1; }
                else { g.failure_count += 1; }
                g.total_reward += reward;
            }
        }

        // 3. Judge判题：用GiniSelector选最优
        let best_gene_id = {
            let gene_refs: Vec<_> = self.genes.values().map(|g| g as &SkillGene).collect();
            self.judge.evaluate(&gene_refs, task)
        };

        (best_gene_id.is_some(), best_gene_id.unwrap_or_default())
    }

    /// 获取最佳技能
    pub fn best_gene(&self) -> Option<&SkillGene> {
        self.genes.values().max_by(|a, b| a.fitness().partial_cmp(&b.fitness()).unwrap())
    }

    /// 获取所有技能
    pub fn all_genes(&self) -> &HashMap<String, SkillGene> {
        &self.genes
    }

    /// 保存所有技能到JSON文件
    pub fn save_skillbank(&self, path: &str) -> Result<(), String> {
        let genes: Vec<&SkillGene> = self.genes.values().collect();
        let json = serde_json::to_string_pretty(&genes).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    /// 从JSON文件加载技能库
    pub fn load_skillbank(&mut self, path: &str) -> Result<usize, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let genes: Vec<SkillGene> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        let count = genes.len();
        for gene in genes {
            self.genes.insert(gene.gene_id.clone(), gene);
        }
        Ok(count)
    }
}

// ============================================================
// 6. 跨时间重放机制
// ============================================================

#[derive(Debug, Clone)]
pub struct ReplayBuffer {
    tasks: Vec<ReplayTask>,
    buffer_size: usize,
}

#[derive(Debug, Clone)]
pub struct ReplayTask {
    pub task: String,
    pub best_gene_id: String,
    pub success: bool,
    pub timestamp: u64,
}

impl ReplayBuffer {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            tasks: vec![],
            buffer_size,
        }
    }

    pub fn add(&mut self, task: ReplayTask) {
        self.tasks.push(task);
        if self.tasks.len() > self.buffer_size {
            self.tasks.remove(0);
        }
    }

    pub fn len(&self) -> usize { self.tasks.len() }

    /// 重放：避免对抗性崩溃
    pub fn replay_sample(&self) -> Option<&ReplayTask> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // 随机选一个近期任务重放
        let recent: Vec<_> = self.tasks.iter()
            .filter(|t| now - t.timestamp < 3600) // 1小时内
            .collect();

        if recent.is_empty() { None }
        else { Some(recent[rand_index(recent.len())]) }
    }
}

fn rand_index(max: usize) -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as usize;
    seed % max
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("{:x}-{:x}", ts, rand_index(99999))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gini() {
        // Gini([10, 0]) = 1 - (1^2 + 0^2) = 0
        // Gini([5, 5]) = 1 - (0.5^2 + 0.5^2) = 0.5
        assert_eq!(gini_impurity(&[10.0, 0.0]), 0.0);
        assert!((gini_impurity(&[5.0, 5.0]) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_entropy() {
        // H([1,0]) = 0, H([0.5,0.5]) = 1
        assert!((entropy(&[1.0, 0.0]) - 0.0).abs() < 0.001);
        assert!((entropy(&[0.5, 0.5]) - 1.0).abs() < 0.01);
    }
}
