//! CoT Introspection - Self-Reflection and Error Correction
//!
//! Implements self-examination mechanism that reviews reasoning steps,
//! identifies potential errors, and triggers corrections.

use crate::cot::{CoTResult, ThoughtStep};
use tracing::info;

/// Introspection result
#[derive(Debug, Clone)]
pub struct IntrospectionResult {
    pub reviewed_steps: Vec<ReviewedStep>,
    pub issues_found: Vec<ReasoningIssue>,
    pub corrections: Vec<Correction>,
    pub overall_confidence: f64,
    pub needs_correction: bool,
}

/// Individual step review
#[derive(Debug, Clone)]
pub struct ReviewedStep {
    pub original: ThoughtStep,
    pub issues: Vec<String>,
    pub corrected: bool,
    pub correction: Option<String>,
}

/// Identified reasoning issue
#[derive(Debug, Clone)]
pub struct ReasoningIssue {
    pub step_number: usize,
    pub issue_type: IssueType,
    pub description: String,
    pub severity: Severity,
    pub suggested_fix: String,
}

#[derive(Debug, Clone, Copy)]
pub enum IssueType {
    LogicalGap,
    UnsupportedAssumption,
    Overgeneralization,
    Contradiction,
    MissingContext,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Applied correction
#[derive(Debug, Clone)]
pub struct Correction {
    pub issue_id: usize,
    pub original_text: String,
    pub corrected_text: String,
    pub explanation: String,
}

/// Self-introspection engine
pub struct Introspection {
    strictness: f64,
}

impl Introspection {
    /// Create new introspection engine
    pub fn new() -> Self {
        Self { strictness: 0.7 }
    }

    /// Perform self-examination on reasoning result
    pub fn examine(&self, result: &CoTResult) -> IntrospectionResult {
        info!(
            "Starting introspection of {} reasoning steps",
            result.reasoning_chain.len()
        );

        let mut reviewed_steps = Vec::new();
        let mut issues_found = Vec::new();
        let mut corrections = Vec::new();

        // Review each step
        for step in result.reasoning_chain.iter() {
            let reviewed = self.review_step(step);
            reviewed_steps.push(reviewed.clone());

            // Collect issues
            for issue in reviewed.issues {
                issues_found.push(ReasoningIssue {
                    step_number: step.step_index + 1,
                    issue_type: self.classify_issue(&issue),
                    description: issue,
                    severity: self.assess_severity(&step.thought),
                    suggested_fix: self.suggest_fix(step),
                });
            }

            // Apply corrections
            if let Some(correction_text) = reviewed.correction {
                corrections.push(Correction {
                    issue_id: step.step_index + 1,
                    original_text: step.thought.clone(),
                    corrected_text: correction_text,
                    explanation: "Self-corrected during introspection".to_string(),
                });
            }
        }

        let needs_correction =
            !issues_found.is_empty() && issues_found.iter().any(|i| i.severity >= Severity::Medium);

        let overall_confidence = if issues_found.is_empty() {
            result.final_confidence
        } else {
            result.final_confidence * (1.0 - self.strictness * (issues_found.len() as f64 * 0.1))
        };

        info!(
            "Introspection complete: {} issues found, {} corrections applied",
            issues_found.len(),
            corrections.len()
        );

        IntrospectionResult {
            reviewed_steps,
            issues_found,
            corrections,
            overall_confidence: overall_confidence.max(0.0),
            needs_correction,
        }
    }

    /// Review individual step
    fn review_step(&self, step: &ThoughtStep) -> ReviewedStep {
        let mut issues = Vec::new();

        // Check reasoning length (too short = potential gap)
        if step.thought.len() < 20 {
            issues.push("Reasoning too brief - possible logical gap".to_string());
        }

        // Check conclusion confidence
        if step.confidence < 0.5 {
            issues.push("Low confidence in conclusion".to_string());
        }

        // Check for unsupported assumptions
        if (step.thought.contains("assume") || step.thought.contains("should be"))
            && !step.thought.contains("because") && !step.thought.contains("since") {
                issues.push("Contains unsupported assumption".to_string());
            }

        // Generate correction if issues found
        let correction = if issues.len() > 1 {
            Some(format!("[CORRECTED] {}", step.thought))
        } else {
            None
        };

        ReviewedStep {
            original: step.clone(),
            issues,
            corrected: correction.is_some(),
            correction,
        }
    }

    /// Classify issue type
    fn classify_issue(&self, description: &str) -> IssueType {
        let lower = description.to_lowercase();
        if lower.contains("gap") || lower.contains("missing") {
            IssueType::LogicalGap
        } else if lower.contains("assume") {
            IssueType::UnsupportedAssumption
        } else if lower.contains("overgeneralization")
            || lower.contains("all")
            || lower.contains("always")
        {
            IssueType::Overgeneralization
        } else if lower.contains("contradiction") {
            IssueType::Contradiction
        } else {
            IssueType::MissingContext
        }
    }

    /// Assess issue severity
    fn assess_severity(&self, reasoning: &str) -> Severity {
        let lower = reasoning.to_lowercase();
        if lower.contains("critical") || lower.contains("fatal") {
            Severity::Critical
        } else if lower.contains("potential") || lower.contains("may") {
            Severity::Low
        } else if lower.contains("should") || lower.contains("likely") {
            Severity::Medium
        } else {
            Severity::High
        }
    }

    /// Generate fix suggestion
    fn suggest_fix(&self, step: &ThoughtStep) -> String {
        format!(
            "Review step {}: Add more supporting evidence",
            step.step_index + 1
        )
    }

    /// Set introspection strictness
    pub fn set_strictness(&mut self, strictness: f64) {
        self.strictness = strictness.clamp(0.0, 1.0);
    }
}

impl Default for Introspection {
    fn default() -> Self {
        Self::new()
    }
}
