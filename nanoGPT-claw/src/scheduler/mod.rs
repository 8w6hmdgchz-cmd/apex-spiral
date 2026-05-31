//! Scheduler Module - Multi-LLM Cluster Dispatch Kernel
//!
//! Manages 1 Core Main LLM + Multiple Auxiliary LLM cluster scheduling,
//! asynchronous task distribution, and result convergence.

pub mod llm_client;
pub mod task;
pub mod provider;
pub mod retry;

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, warn};

use task::Task;
use provider::ProviderRegistry;
use crate::evolution::apex_akashic::{ApexAkashicCalculator, ApexAkashicResult, ApexDimensions, ApexPenalties};

#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub main_provider: String,
    pub main_model: String,
    pub aux_providers: Vec<AuxModelConfig>,
    pub max_concurrent: usize,
    pub timeout_secs: u64,
    /// ARS评分开关：converge后是否启用Φ公式裁判
    pub ars_enabled: bool,
    /// ARS接受阈值：final_score >= 此值则接受输出
    pub ars_threshold: f64,
    /// ARS重试阈值：final_score >= 此值但 < ars_threshold 时允许重试
    pub ars_retry_threshold: f64,
    /// ARS最大重试次数
    pub ars_max_retries: usize,
}

#[derive(Debug, Clone)]
pub struct AuxModelConfig {
    pub name: String,
    pub provider: String,
    pub model: String,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            main_provider: "openai".to_string(),
            main_model: "gpt-4o".to_string(),
            aux_providers: vec![
                AuxModelConfig {
                    name: "code".to_string(),
                    provider: "openai".to_string(),
                    model: "gpt-4o".to_string(),
                },
                AuxModelConfig {
                    name: "logic".to_string(),
                    provider: "openai".to_string(),
                    model: "gpt-4o".to_string(),
                },
                AuxModelConfig {
                    name: "review".to_string(),
                    provider: "openai".to_string(),
                    model: "gpt-4o".to_string(),
                },
            ],
            max_concurrent: 4,
            timeout_secs: 120,
            ars_enabled: true,
            ars_threshold: 0.65,
            ars_retry_threshold: 0.40,
            ars_max_retries: 2,
        }
    }
}

pub struct Scheduler {
    config: SchedulerConfig,
    providers: ProviderRegistry,
    active_tasks: Arc<RwLock<HashMap<String, Task>>>,
    /// ARS评分计算器：converge后Φ公式裁判
    ars_calculator: ApexAkashicCalculator,
    /// ARS重试计数（每次pipeline重置）
    ars_retry_count: usize,
}

/// ARS决策结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArsDecision {
    Accept,
    Reject,
    Retry,
}

impl Scheduler {
    pub fn new() -> Self {
        Self::with_config(SchedulerConfig::default())
    }

    pub fn with_config(config: SchedulerConfig) -> Self {
        let providers = Self::create_providers();
        Self {
            config,
            providers,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            ars_calculator: ApexAkashicCalculator::new(),
            ars_retry_count: 0,
        }
    }

    fn create_providers() -> ProviderRegistry {
        ProviderRegistry::create_from_env()
    }

    pub fn is_active(&self) -> bool {
        self.providers.get(&self.config.main_provider).is_some()
    }

    fn truncate_for_log(s: &str, max_chars: usize) -> String {
        if s.chars().count() <= max_chars {
            s.to_string()
        } else {
            s.chars().take(max_chars).collect::<String>() + "..."
        }
    }

    pub async fn submit_to_main(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        info!("Submitting to main LLM ({}): {}...", self.config.main_provider, Self::truncate_for_log(prompt, 50));

        let provider = self.providers.get(&self.config.main_provider)
            .ok_or_else(|| format!("Main provider '{}' not configured", self.config.main_provider))?;

        let response = provider.complete(prompt).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(response.content)
    }

    pub async fn submit_to_aux(&self, model_name: &str, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        info!("Submitting to aux LLM [{}]: {}...", model_name, Self::truncate_for_log(prompt, 30));

        let aux_config = self.config.aux_providers.iter()
            .find(|c| c.name == model_name)
            .ok_or_else(|| format!("Aux model '{}' not found", model_name))?;

        let provider = self.providers.get(&aux_config.provider)
            .ok_or_else(|| format!("Provider '{}' not configured", aux_config.provider))?;

        let response = provider.complete(prompt).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(response.content)
    }

    pub async fn dispatch_parallel(&self, prompt: &str) -> HashMap<String, String> {
        let mut handles = Vec::new();

        for aux_config in &self.config.aux_providers {
            let provider = match self.providers.get(&aux_config.provider) {
                Some(p) => p,
                None => {
                    warn!("Provider '{}' not configured, skipping {}", aux_config.provider, aux_config.name);
                    continue;
                }
            };

            let p = prompt.to_string();
            let n = aux_config.name.clone();
            let provider = Arc::clone(&provider);

            let handle = tokio::spawn(async move {
                let result = provider.complete(&p).await;
                (n, result)
            });
            handles.push(handle);
        }

        let mut results = HashMap::new();
        for handle in handles {
            match handle.await {
                Ok((name, Ok(resp))) => {
                    results.insert(name, resp.content);
                }
                Ok((name, Err(e))) => {
                    warn!("Aux LLM {} failed: {}", name, e);
                }
                Err(e) => {
                    warn!("Task join failed: {}", e);
                }
            }
        }

        results
    }

    pub fn converge_results(&self, main_result: &str, aux_results: &HashMap<String, String>) -> String {
        let mut output = format!("[MAIN LLM DECISION]
{}

", main_result);

        for (model, result) in aux_results {
            output.push_str(&format!("[AUX:{}]
{}

", model, result));
        }

        output
    }

    pub async fn process_full_pipeline(&mut self, user_input: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting full pipeline for: {}...", Self::truncate_for_log(user_input, 40));

        self.ars_retry_count = 0;

        let main_prompt = format!(
            "Analyze this request and break it into subtasks for specialized agents:

{}",
            user_input
        );

        loop {
            let main_result = self.submit_to_main(&main_prompt).await?;
            let aux_results = self.dispatch_parallel(&main_result).await;
            let output = self.converge_results(&main_result, &aux_results);

            // ARS/Φ公式评分层
            if !self.config.ars_enabled {
                info!("Pipeline complete (ARS disabled), output length: {}", output.len());
                return Ok(output);
            }

            let (decision, score) = self.ars_evaluate(&output)?;
            info!("ARS decision: {:?}, score: {:.4}, threshold: {:.2}", decision, score.final_score, self.config.ars_threshold);

            match decision {
                ArsDecision::Accept => {
                    info!("Pipeline complete (ARS accepted), output length: {}", output.len());
                    return Ok(output);
                }
                ArsDecision::Retry if self.ars_retry_count < self.config.ars_max_retries => {
                    self.ars_retry_count += 1;
                    warn!("ARS retry {}/{}", self.ars_retry_count, self.config.ars_max_retries);
                    continue;
                }
                _ => {
                    return Err(format!(
                        "ARS rejected: score={:.4} < threshold={:.2}, decisions exhausted",
                        score.final_score, self.config.ars_threshold
                    ).into());
                }
            }
        }
    }

    /// ARS评分：将converge后的输出映射为APEX维度，调用Φ公式计算final_score。
    /// 输入为converge后的完整输出字符串，通过文本统计特征构建评分维度。
    fn ars_score(&self, output: &str) -> ApexAkashicResult {
        let chars: Vec<char> = output.chars().collect();
        let len = chars.len();

        // 基础统计特征
        let word_count = output.split_whitespace().count();
        let line_count = output.lines().count();
        let unique_word_ratio = if word_count > 0 {
            output.split_whitespace().collect::<std::collections::HashSet<_>>().len() as f64 / word_count as f64
        } else {
            0.0
        };

        // E - Evolution：基于辅助LLB覆盖度（aux结果数量作为进化多样性代理）
        let evolution = 0.5_f64.min(1.0);

        // V - Value：词数效率（适中为佳，过长/过短都差）
        let value = if len < 100 { len as f64 / 100.0 }
        else if len > 5000 { (5000.0 / len as f64).max(0.3) }
        else { 0.7 + 0.3 * unique_word_ratio.min(1.0) };

        // M - Memory：唯一词比例（信息密度）
        let memory = unique_word_ratio.clamp(0.3, 1.0);

        // A - Autonomy：行数/字数比（结构化程度）
        let autonomy = if word_count > 0 { (line_count as f64 / word_count as f64 * 10.0).clamp(0.3, 1.0) } else { 0.5 };

        // B - Benchmark：收敛质量（唯一词*结构化，适中）
        let benchmark = (unique_word_ratio * 0.5 + (1.0 - (len as f64 / 2000.0).abs()) * 0.5).clamp(0.3, 1.0);

        // T - Thinking：句子完整度（有无截断）
        let thinking = if output.lines().last().map(|l| l.trim().is_empty()).unwrap_or(true) { 0.9 } else { 0.7 };

        // D - Decision：收敛信号（有无MAIN/AUX标记）
        let decision = if output.contains("[MAIN LLM DECISION]") && output.contains("[AUX:") { 0.9 } else { 0.6 };

        // H - Harmony：格式一致性
        let harmony = (line_count as f64 / 50.0).clamp(0.4, 1.0);

        // L - Learning：增长潜力估算
        let learning = (unique_word_ratio * value).clamp(0.3, 1.0);

        // G - Growth：扩展性（字数适中）
        let growth = if len < 500 { len as f64 / 500.0 } else { (2000.0 / len as f64).clamp(0.4, 1.0) };

        // W - Wisdom：知识密度
        let wisdom = (word_count as f64 / len.max(1) as f64).clamp(0.3, 1.0);

        // B2 - Balance：整体均衡
        let balance = ((evolution + value + memory + autonomy) / 4.0).clamp(0.3, 1.0);

        let dimensions = ApexDimensions {
            evolution, value, memory, autonomy, benchmark,
            thinking, decision, harmony, learning, growth, wisdom, balance,
        };

        // 惩罚项
        let penalties = ApexPenalties {
            token: 0.0,   // scheduler层不追踪token
            claw: 0.0,
            agent: 0.0,
            panic: 0.0,
            prune: 0.0,
            soul: 0.0,
            runtime: ((len as f64 / 10000.0) * 0.05).min(0.05),
            network: 0.0,
            error: 0.0,
            memory: ((len as f64 / 100000.0) * 0.02).min(0.02),
            resource: 0.0,
            log: 0.0,
        };

        let calc = ApexAkashicCalculator::new()
            .with_dimensions(dimensions)
            .with_penalties(penalties);
        calc.calculate()
    }

    /// ARS决策：根据Φ公式final_score决定accept/reject/retry。
    /// - score >= ars_threshold → Accept
    /// - score >= ars_retry_threshold → Retry（可修复）
    /// - score < ars_retry_threshold → Reject（质量太差）
    fn ars_evaluate(&self, output: &str) -> Result<(ArsDecision, ApexAkashicResult), Box<dyn std::error::Error + Send + Sync>> {
        let score = self.ars_score(output);
        let decision = if score.final_score >= self.config.ars_threshold {
            ArsDecision::Accept
        } else if score.final_score >= self.config.ars_retry_threshold {
            ArsDecision::Retry
        } else {
            ArsDecision::Reject
        };
        Ok((decision, score))
    }

    pub fn get_stats(&self) -> SchedulerStats {
        let active = self.active_tasks.read();
        SchedulerStats {
            active_tasks: active.len(),
            max_concurrent: self.config.max_concurrent,
            aux_model_count: self.config.aux_providers.len(),
            available_providers: self.providers.names(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub active_tasks: usize,
    pub max_concurrent: usize,
    pub aux_model_count: usize,
    pub available_providers: Vec<String>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
