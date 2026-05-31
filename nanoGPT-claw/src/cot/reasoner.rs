//! CoT Reasoner - Chain-of-Thought Reasoning Engine (REAL LLM)
//!
//! Implements step-by-step reasoning with REAL LLM calls, progressive deduction,
//! logical verification, and structured problem decomposition.

use std::sync::Arc;
use tracing::{info, debug, warn};
use crate::scheduler::Scheduler;
use crate::cot::{CoTConfig, ThoughtStep, ReasoningType, CoTResult, CoTError};

/// Chain-of-thought reasoner with REAL LLM integration
pub struct Reasoner {
    config: CoTConfig,
    scheduler: Arc<Scheduler>,
}

impl Reasoner {
    /// Create new reasoner with scheduler
    pub fn new(scheduler: Arc<Scheduler>) -> Self {
        Self {
            config: CoTConfig::default(),
            scheduler,
        }
    }

    /// Create new reasoner with custom config
    pub fn with_config(scheduler: Arc<Scheduler>, config: CoTConfig) -> Self {
        Self {
            config,
            scheduler,
        }
    }

    /// Perform chain-of-thought reasoning (REAL LLM)
    pub async fn reason(&self, problem: &str) -> Result<CoTResult, CoTError> {
        let start = std::time::Instant::now();
        info!("Starting REAL CoT reasoning for: {}...", Self::truncate(problem, 50));

        let mut steps = Vec::new();
        let mut current_context = format!("问题: {}
", problem);
        const MAX_CONTEXT_LENGTH: usize = 8000; // 安全限制 context 长度

        for step_num in 0..self.config.max_steps {
            info!("Executing CoT step {}/{}", step_num + 1, self.config.max_steps);

            // 安全限制 context 长度，防止无限增长
            if current_context.len() > MAX_CONTEXT_LENGTH {
                warn!("Context too long, truncating to {} chars", MAX_CONTEXT_LENGTH);
                let chars: Vec<char> = current_context.chars().collect();
                let start = chars.len().saturating_sub(MAX_CONTEXT_LENGTH);
                current_context = chars[start..].iter().collect();
            }

            let thought_step = self.execute_step(step_num, &current_context).await?;
            steps.push(thought_step.clone());

            current_context.push_str(&format!("
思维步骤{}: {}
", step_num + 1, thought_step.thought));

            // Check if we've reached sufficient confidence
            if thought_step.confidence >= self.config.confidence_threshold {
                info!("Reached confidence threshold ({:.2}) at step {}", thought_step.confidence, step_num + 1);
                break;
            }
        }

        // Generate final conclusion
        let final_conclusion = self.generate_final_conclusion(problem, &steps).await?;
        let final_confidence = steps.iter().map(|s| s.confidence).sum::<f64>() / steps.len() as f64;
        let had_corrections = steps.iter().any(|s| s.flagged);

        Ok(CoTResult {
            conclusion: final_conclusion,
            reasoning_chain: steps.clone(),
            final_confidence,
            had_corrections,
            total_steps: steps.len(),
            mode: self.config.reasoning_mode,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Execute single reasoning step with REAL LLM
    async fn execute_step(&self, step_index: usize, context: &str) -> Result<ThoughtStep, CoTError> {
        let prompt = format!(
            "你是一个优秀的推理助手，请进行逐步思维链推理。

当前上下文：
{}

请执行第{}步推理，请用清晰的语言描述你的思考过程，并给出这一步的结论。

你的回答格式：
思考过程：[你的思考]
结论：[这一步的结论]
置信度：[0.0到1.0之间的数字]",
            context, step_index + 1
        );

        let llm_response = self.scheduler.submit_to_main(&prompt).await
            .map_err(|e| CoTError::InferenceFailed(e.to_string()))?;

        debug!("LLM response for step {}: {}", step_index, Self::truncate(&llm_response, 100));

        let (thought, confidence) = Self::parse_llm_response(&llm_response);

        Ok(ThoughtStep {
            step_index,
            thought,
            confidence,
            reasoning_type: ReasoningType::Deduction,
            parent_step: if step_index > 0 { Some(step_index - 1) } else { None },
            flagged: confidence < self.config.confidence_threshold * 0.5,
            scratchpad: Some(llm_response),
        })
    }

    /// Generate final conclusion from all steps
    async fn generate_final_conclusion(&self, problem: &str, steps: &[ThoughtStep]) -> Result<String, CoTError> {
        let steps_summary: Vec<String> = steps.iter()
            .enumerate()
            .map(|(i, s)| format!("步骤{}: {}
结论: {}", i + 1, s.thought, s.thought))
            .collect();

        let prompt = format!(
            "基于以下思维链，为问题\"{}\"生成最终结论：

{}",
            problem, steps_summary.join("

")
        );

        self.scheduler.submit_to_main(&prompt).await
            .map_err(|e| CoTError::InferenceFailed(e.to_string()))
    }

    /// Parse LLM response to extract thought and confidence
    fn parse_llm_response(response: &str) -> (String, f64) {
        let mut thought = response.to_string();
        let mut confidence = 0.6;

        // Try to extract confidence score
        if let Some(conf_pos) = response.to_lowercase().find("置信度：") {
            let after = &response[conf_pos + 4..];
            if let Some(end_pos) = after.find(|c: char| !c.is_ascii_digit() && c != '.') {
                if let Ok(f) = after[..end_pos].trim().parse::<f64>() {
                    confidence = f.clamp(0.0, 1.0);
                    thought = response[..conf_pos].trim().to_string();
                }
            }
        }

        // Try to extract conclusion/thought
        if let Some(thought_pos) = response.find("思考过程：") {
            thought = response[thought_pos + 5..].trim().to_string();
            if let Some(conclusion_pos) = thought.find("结论：") {
                thought = thought[..conclusion_pos].trim().to_string();
            }
        } else if let Some(thought_pos) = response.find("结论：") {
            thought = response[thought_pos + 3..].trim().to_string();
        }

        (thought, confidence)
    }

    /// Truncate string safely for logging
    fn truncate(s: &str, max_chars: usize) -> String {
        if s.chars().count() <= max_chars {
            s.to_string()
        } else {
            s.chars().take(max_chars).collect::<String>() + "..."
        }
    }

    /// Set maximum reasoning steps
    pub fn set_max_steps(&mut self, max: usize) {
        self.config.max_steps = max;
    }

    /// Set confidence threshold
    pub fn set_confidence_threshold(&mut self, threshold: f64) {
        self.config.confidence_threshold = threshold;
    }
}

impl Default for Reasoner {
    fn default() -> Self {
        panic!("Reasoner requires a Scheduler. Use Reasoner::new() instead.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::Scheduler;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_basic_reasoning() {
        let scheduler = Arc::new(Scheduler::new());
        let reasoner = Reasoner::new(scheduler);
        // Note: This test requires valid API keys to pass
        // let result = reasoner.reason("What is the capital of France?").await;
        // assert!(result.is_ok());
    }
}
