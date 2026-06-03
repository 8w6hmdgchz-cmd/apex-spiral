use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingInput {
    pub query: String,
    pub context: Option<String>,
    pub task_type: TaskType,
    pub constraints: Vec<String>,
    pub depth: ReasoningDepth,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskType {
    Analysis,
    CodeGeneration,
    ProblemSolving,
    CreativeWriting,
    QuestionAnswering,
    TaskPlanning,
    Review,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReasoningDepth {
    Shallow,
    Medium,
    Deep,
    Comprehensive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingOutput {
    pub id: String,
    pub steps: Vec<ThoughtStep>,
    pub final_answer: String,
    pub confidence: f32,
    pub reasoning_chain: String,
    pub alternatives_considered: Vec<Alternative>,
    pub errors_detected: Vec<DetectedError>,
    pub improvements_suggested: Vec<String>,
    pub metadata: ThinkingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtStep {
    pub step_number: usize,
    pub step_type: StepType,
    pub content: String,
    pub reasoning: Option<String>,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
    pub related_steps: Vec<usize>,
    pub results: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepType {
    Understanding,
    Decomposition,
    Analysis,
    Deduction,
    Hypothesis,
    Verification,
    Synthesis,
    Conclusion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub approach: String,
    pub reasoning: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub selected: bool,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedError {
    pub error_type: ErrorType,
    pub description: String,
    pub location: Option<String>,
    pub severity: Severity,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorType {
    Logic,
    Syntax,
    Factual,
    Reasoning,
    Assumption,
    Calculation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingMetadata {
    pub total_steps: usize,
    pub total_time_ms: u64,
    pub model_calls: usize,
    pub tokens_used: u32,
}
