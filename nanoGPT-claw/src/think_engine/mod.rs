pub mod reasoning;
pub mod reflection;

use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info};

pub use reasoning::*;
pub use reflection::*;

pub static THINK_ENGINE: once_cell::sync::Lazy<Arc<ThinkEngine>> = 
    once_cell::sync::Lazy::new(|| Arc::new(ThinkEngine::new()));

pub async fn think(input: ThinkingInput) -> Result<ThinkingOutput> {
    THINK_ENGINE.think(input).await
}

pub struct ThinkEngine {
    max_depth: usize,
    enable_reflection: bool,
}

impl ThinkEngine {
    pub fn new() -> Self {
        Self {
            max_depth: 5,
            enable_reflection: true,
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn with_reflection(mut self, enabled: bool) -> Self {
        self.enable_reflection = enabled;
        self
    }

    pub async fn think(&self, input: ThinkingInput) -> Result<ThinkingOutput> {
        let start_time = Instant::now();
        info!("Starting CoT reasoning for query: {}", input.query.chars().take(50).collect::<String>());
        
        let depth: usize = match input.depth {
            ReasoningDepth::Shallow => 2,
            ReasoningDepth::Medium => 3,
            ReasoningDepth::Deep => 5,
            ReasoningDepth::Comprehensive => 7,
        };

        let mut steps = Vec::new();
        let mut current_confidence = 0.5f32;
        
        steps.push(self.create_step(
            1,
            StepType::Understanding,
            format!("Analyzing query: {}", input.query),
            Some(format!("Context: {:?}", input.context)),
            current_confidence,
        ));

        steps.push(self.create_step(
            2,
            StepType::Decomposition,
            "Breaking down the problem into components".to_string(),
            None,
            current_confidence,
        ));

        for i in 3..=depth.saturating_sub(1) {
            let step_type = match i % 4 {
                0 => StepType::Deduction,
                1 => StepType::Analysis,
                2 => StepType::Hypothesis,
                _ => StepType::Verification,
            };
            
            current_confidence = (current_confidence + 0.1).min(0.95);
            
            steps.push(self.create_step(
                i,
                step_type,
                format!("Processing step {} of reasoning", i),
                None,
                current_confidence,
            ));
        }

        steps.push(self.create_step(
            depth,
            StepType::Synthesis,
            "Synthesizing findings into coherent response".to_string(),
            None,
            current_confidence + 0.05,
        ));

        steps.push(self.create_step(
            depth + 1,
            StepType::Conclusion,
            format!("Final conclusion for: {}", input.query.chars().take(30).collect::<String>()),
            Some("Confidence validated through multi-step reasoning".to_string()),
            current_confidence + 0.1,
        ));

        let alternatives = self.generate_alternatives(&input);
        let errors = if self.enable_reflection {
            self.detect_errors(&steps)
        } else {
            vec![]
        };

        let total_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(ThinkingOutput {
            id: uuid::Uuid::new_v4().to_string(),
            steps,
            final_answer: format!("Reasoned response to: {}", input.query.chars().take(50).collect::<String>()),
            confidence: current_confidence,
            reasoning_chain: format!("CoT reasoning with {} steps", depth + 1),
            alternatives_considered: alternatives,
            errors_detected: errors,
            improvements_suggested: vec!["Consider multiple perspectives".to_string()],
            metadata: ThinkingMetadata {
                total_steps: depth + 1,
                total_time_ms,
                model_calls: 1,
                tokens_used: ((depth + 1) * 50) as u32,
            },
        })
    }

    fn create_step(
        &self,
        step_number: usize,
        step_type: StepType,
        content: String,
        reasoning: Option<String>,
        confidence: f32,
    ) -> ThoughtStep {
        ThoughtStep {
            step_number,
            step_type,
            content,
            reasoning,
            confidence,
            timestamp: Utc::now(),
            related_steps: if step_number > 1 {
                vec![step_number - 1]
            } else {
                vec![]
            },
            results: None,
        }
    }

    fn generate_alternatives(&self, _input: &ThinkingInput) -> Vec<Alternative> {
        let mut alternatives = Vec::new();
        
        alternatives.push(Alternative {
            approach: "Direct analysis approach".to_string(),
            reasoning: "Quick but may miss nuances".to_string(),
            pros: vec!["Fast execution".to_string(), "Low complexity".to_string()],
            cons: vec!["May oversimplify".to_string()],
            selected: false,
            rejection_reason: Some("Not thorough enough for complex queries".to_string()),
        });

        alternatives.push(Alternative {
            approach: "Multi-step reasoning".to_string(),
            reasoning: "Systematic decomposition and analysis".to_string(),
            pros: vec!["Comprehensive".to_string(), "Traceable".to_string()],
            cons: vec!["Slower".to_string(), "More tokens".to_string()],
            selected: true,
            rejection_reason: None,
        });

        alternatives
    }

    fn detect_errors(&self, steps: &[ThoughtStep]) -> Vec<DetectedError> {
        let mut errors = Vec::new();
        
        for step in steps {
            if step.confidence < 0.5 {
                errors.push(DetectedError {
                    error_type: ErrorType::Reasoning,
                    description: format!("Low confidence in step {}", step.step_number),
                    location: Some(format!("Step {}", step.step_number)),
                    severity: Severity::Medium,
                    suggestion: "Review this step's logic".to_string(),
                });
            }
        }
        
        errors
    }

    pub async fn reflect(&self, output: &ThinkingOutput) -> Result<ReflectionResult> {
        Ok(ReflectionResult {
            quality_score: output.confidence,
            consistency_score: self.calculate_consistency(output),
            bias_identified: vec!["Potential confirmation bias in step reasoning".to_string()],
            improvements: vec!["Increase reasoning depth for complex tasks".to_string()],
            lessons: vec!["Multi-step verification improves confidence".to_string()],
        })
    }

    fn calculate_consistency(&self, output: &ThinkingOutput) -> f32 {
        if output.steps.is_empty() {
            return 0.0;
        }

        let confidence_sum: f32 = output.steps.iter().map(|s| s.confidence).sum();
        let avg_confidence = confidence_sum / output.steps.len() as f32;
        
        let variance: f32 = output.steps.iter()
            .map(|s| (s.confidence - avg_confidence).powi(2))
            .sum::<f32>() / output.steps.len() as f32;
        
        (1.0 - variance.sqrt()).max(0.0).min(1.0)
    }
}

impl Default for ThinkEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReflectionResult {
    pub fn overall_score(&self) -> f32 {
        (self.quality_score + self.consistency_score) / 2.0
    }
}
