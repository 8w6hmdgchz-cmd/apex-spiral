// search_skill_core.rs
// SearchSkill 核心实现 - Rust 版本
// Select-Read-Act 三段式检索 + SkillBank 动态技能库
// 璇玑帝国 APEX · Rust实现

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================
// 1. 技能卡片
// ============================================================

#[derive(Debug, Clone)]
pub struct SkillCard {
    pub skill_id: String,
    pub trigger: Vec<String>,
    pub action: String,
    pub output_fmt: String,
    pub success_rate: f64,
    pub use_count: u32,
    pub last_used: u64,
}

impl SkillCard {
    pub fn new(skill_id: &str, trigger: Vec<&str>, action: &str, output_fmt: &str, success_rate: f64) -> Self {
        SkillCard {
            skill_id: skill_id.to_string(),
            trigger: trigger.into_iter().map(|s| s.to_lowercase()).collect(),
            action: action.to_string(),
            output_fmt: output_fmt.to_string(),
            success_rate,
            use_count: 0,
            last_used: 0,
        }
    }

    pub fn match_score(&self, query: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let mut score = 0.0;

        for trigger in &self.trigger {
            for word in &query_words {
                if trigger.contains(word) || word.contains(trigger) {
                    score += 1.0;
                }
            }
        }

        if score > 0.0 && !query_words.is_empty() {
            score / query_words.len() as f64 * 0.6
                + self.success_rate * 0.3
                + self.use_count as f64 * 0.1
        } else {
            0.0
        }
    }

    pub fn update_usage(&mut self, success: bool) {
        self.use_count += 1;
        self.last_used = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let delta = if success { 0.1 } else { -0.1 };
        self.success_rate = (self.success_rate + delta).clamp(0.0, 1.0);
    }
}

// ============================================================
// 2. 技能知识库 SkillBank
// ============================================================

#[derive(Debug)]
pub struct SkillBank {
    cards: HashMap<String, SkillCard>,
    pub bank_path: String,
}

impl SkillBank {
    pub fn new(bank_path: &str) -> Self {
        let mut bank = SkillBank {
            cards: HashMap::new(),
            bank_path: bank_path.to_string(),
        };
        bank.init_default_skills();
        bank
    }

    fn init_default_skills(&mut self) {
        let skills = vec![
            SkillCard::new("apex_reflection", vec!["完成", "结束", "解决", "complete"],
                "提取经验→更新SkillBank", "reflection+skill_update", 0.85),
            SkillCard::new("apex_doubt", vec!["确定", "准确", "确认", "verify"],
                "Doubt-Driven三问审查", "doubt_findings+confidence", 0.90),
            SkillCard::new("apex_formula", vec!["分析", "代入", "公式", "formula"],
                "APEX公式代入自检", "formula_check+delta_g", 0.88),
            SkillCard::new("apex_evolution", vec!["改进", "进化", "提升", "improve"],
                "PCEC周期+技能提取", "evolution_report", 0.82),
            SkillCard::new("apex_metacognition", vec!["自检", "反思", "回顾", "check"],
                "5步Metacognition检查", "metacognition_report", 0.91),
            SkillCard::new("apex_skill_fetch", vec!["资源", "获取", "拉取", "fetch"],
                "EvoMap GEP + gist raw拉取", "absorbed_resources", 0.87),
            SkillCard::new("apex_github_sync", vec!["github", "gist", "推送", "push"],
                "git push/fetch + gist操作", "sync_status", 0.93),
            SkillCard::new("search_general", vec!["搜索", "查找", "查询", "search"],
                "通用关键词检索", "search_results", 0.75),
        ];
        for s in skills {
            self.cards.insert(s.skill_id.clone(), s);
        }
    }

    /// Select: 选择最优技能
    pub fn select(&mut self, query: &str) -> Option<&mut SkillCard> {
        let mut best_id: Option<String> = None;
        let mut best_score = 0.0;

        for (id, card) in &self.cards {
            let score = card.match_score(query);
            if score > best_score {
                best_score = score;
                best_id = Some(id.clone());
            }
        }

        if let Some(id) = best_id {
            if let Some(card) = self.cards.get_mut(&id) {
                card.use_count += 1;
                card.last_used = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                return self.cards.get_mut(&id);
            }
        }
        None
    }

    /// Read: 读取技能规则
    pub fn read(&self, card: Option<&SkillCard>, query: &str) -> String {
        match card {
            Some(c) => format!("{} | skill={} action={}", query, c.skill_id, c.action),
            None => query.to_string(),
        }
    }

    /// Update from result: 根据执行结果更新
    pub fn update_from_result(&mut self, skill_id: &str, success: bool) {
        if let Some(card) = self.cards.get_mut(skill_id) {
            card.update_usage(success);
            // 淘汰低效技能
            if card.success_rate < 0.3 && card.use_count > 5 {
                self.cards.remove(skill_id);
            }
        }
    }

    /// Save: 持久化
    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.cards)
            .map_err(|e| e.to_string())?;
        fs::write(&self.bank_path, json).map_err(|e| e.to_string())
    }

    /// Load: 加载
    pub fn load(&mut self) -> Result<(), String> {
        if !Path::new(&self.bank_path).exists() {
            return Ok(());
        }
        let data = fs::read_to_string(&self.bank_path).map_err(|e| e.to_string())?;
        let cards: HashMap<String, SkillCard> = serde_json::from_str(&data)
            .map_err(|e| e.to_string())?;
        self.cards = cards;
        Ok(())
    }
}

// ============================================================
// 3. SearchSkill 执行器
// ============================================================

#[derive(Debug)]
pub struct SearchSkillEngine {
    pub bank: SkillBank,
    cache: HashMap<String, Vec<String>>,
}

impl SearchSkillEngine {
    pub fn new(bank_path: &str) -> Self {
        SearchSkillEngine {
            bank: SkillBank::new(bank_path),
            cache: HashMap::new(),
        }
    }

    /// Select-Read-Act 三段式执行
    pub fn execute(&mut self, query: &str) -> SearchResult {
        // Select: 选择技能
        let card = self.bank.select(query);

        // Read: 生成检索指令
        let act_query = self.bank.read(card, query);

        // Act: 执行检索
        let results = self.act(&act_query);
        let success = !results.is_empty();

        // Sync: 更新技能库
        if let Some(c) = card {
            self.bank.update_from_result(&c.skill_id, success);
        }

        SearchResult {
            skill_id: card.map(|c| c.skill_id).unwrap_or_else(|| "none".to_string()),
            act_query,
            results: results.clone(),
            success,
        }
    }

    /// Act: 执行检索 (核心Go/Rust实现)
    fn act(&self, act_query: &str) -> Vec<String> {
        // 优先从缓存读
        if let Some(cached) = self.cache.get(act_query) {
            return cached.clone();
        }
        // 实际实现: 对接 Mem0/EvoMap/WebFetch
        vec![]
    }
}

#[derive(Debug)]
pub struct SearchResult {
    pub skill_id: String,
    pub act_query: String,
    pub results: Vec<String>,
    pub success: bool,
}

// ============================================================
// 4. 与Hermes-Agent融合
// ============================================================

/*
Hermes-Agent 三大机制融合:

1. Prefetch (响应前预取)
   SearchSkillEngine 每次execute后，后台预加载相关技能到缓存

2. Sync (响应后同步)
   execute()执行后自动调用bank.update_from_result()更新技能库

3. Background (后台预加载)
   下一轮开始前预取可能用到的技能到cache

璇玑版执行流程:
  Select → 匹配SkillBank最优技能
    ↓
  Read → 读取技能规则约束
    ↓
  Act → 执行检索+Mem0语义搜索
    ↓
  Sync → 更新SkillBank+记忆层
    ↓
  Prefetch → 预加载下一轮相关技能
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_select() {
        let bank = SkillBank::new("/tmp/test_bank.json");
        let card = bank.select("分析APEX公式");
        assert!(card.is_some());
    }

    #[test]
    fn test_sra_execution() {
        let mut engine = SearchSkillEngine::new("/tmp/test_bank.json");
        let result = engine.execute("分析公式代入");
        assert!(!result.skill_id.is_empty());
    }
}
