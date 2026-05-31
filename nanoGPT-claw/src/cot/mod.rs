//! # Chain-of-Thought Engine Module
//!
//! This module implements a self-reflective Chain-of-Thought (CoT) reasoning engine
//! inspired by cognitive architectures. It provides structured reasoning capabilities
//! with introspection, self-correction, and multi-step thought chains.
//!
//! ## Architecture Overview
//!
//! The CoT engine consists of three interconnected components:
//!
//! - **[`Reasoner`](reasoner::Reasoner)**: Core reasoning engine that executes thought chains
//!   with support for deduction, abduction, and analogy-based inference.
//!
//! - **[`Introspection`](introspection::IntrospectionEngine)**: Self-examination mechanism that
//!   monitors reasoning quality, detects errors, and triggers reflection when confidence
//!   falls below thresholds.
//!
//! ## Key Features
//!
//! - **Dual-mode reasoning**: Supports both linear (fast) and recursive (deep) reasoning modes
//! - **Confidence tracking**: Per-step confidence scores with automatic escalation
//! - **Self-correction**: Introspection-triggered backtracking and replanning
//! - **Thought scaffolding**: Structured intermediate representations for complex reasoning
//! - **Async-first design**: Fully async for high-throughput concurrent reasoning sessions
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use nanoGPT_claw::cot::{Reasoner, CoTConfig, ReasoningMode};
//!
//! let config = CoTConfig::default();
//! let reasoner = Reasoner::new(config);
//!
//! let result = reasoner
//!     .solve("Should I implement caching for this API endpoint?")
//!     .await?;
//! ```
//!
//! ## Design Principles
//!
//! 1. **Monotonic improvement**: Reasoning quality should never degrade across iterations
//! 2. **Transparent auditing**: Every reasoning step is logged and inspectable
//! 3. **Graceful degradation**: If introspection fails, fall back to linear reasoning
//! 4. **Configurable depth**: Maximum recursion/depth is user-configurable
//!
//! ## Performance Characteristics
//!
//! - Linear reasoning: O(n) where n = number of thought steps
//! - Recursive reasoning: O(d) where d = maximum recursion depth
//! - Memory usage: Proportional to max_depth × max_steps_per_depth
//!
//! ## References
//!
//! - [Chain-of-Thought Prompting](https://arxiv.org/abs/2201.11903)
//! - [Self-Consistency](https://arxiv.org/abs/2203.11171)
//! - [Reflexion](https://arxiv.org/abs/2303.11366)

pub mod introspection;
pub mod reasoner;

pub use reasoner::Reasoner;

use serde::{Deserialize, Serialize};

/// Reasoning mode determines how the engine processes thought steps.
///
/// Each mode has different performance and quality trade-offs:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningMode {
    /// Linear mode: Process each thought step sequentially without recursion.
    /// Fastest but no self-correction. Best for simple, well-defined tasks.
    Linear,

    /// Recursive mode: Allow self-referential thoughts that can re-examine
    /// previous conclusions. Slower but enables deep self-correction.
    Recursive,

    /// Beam search mode: Maintain multiple candidate reasoning paths
    /// and select the best at the end. Good balance of quality and speed.
    BeamSearch,
}

impl Default for ReasoningMode {
    fn default() -> Self {
        ReasoningMode::Linear
    }
}

/// Configuration for the CoT engine.
///
/// All parameters can be tuned to balance reasoning quality against
/// latency and resource consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CoTConfig {
    /// Maximum number of thought steps in a single reasoning chain.
    /// Exceeding this triggers forced termination.
    pub max_steps: usize,

    /// Maximum recursion depth for recursive reasoning mode.
    /// Only applies when `mode == ReasoningMode::Recursive`.
    pub max_depth: usize,

    /// Minimum confidence score (0.0–1.0) below which introspection is triggered.
    /// Lower values = more tolerant, fewer corrections.
    pub confidence_threshold: f64,

    /// Whether to emit verbose reasoning traces for debugging/auditing.
    pub trace_enabled: bool,

    /// Maximum number of candidate paths for beam search mode.
    /// Only applies when `mode == ReasoningMode::BeamSearch`.
    pub beam_width: usize,

    /// Timeout for a single reasoning step in milliseconds.
    /// Steps exceeding this are terminated and marked as uncertain.
    pub step_timeout_ms: u64,

    /// Whether to enable self-correction via introspection.
    pub enable_self_correction: bool,

    /// Reasoning mode selection.
    pub reasoning_mode: ReasoningMode,

    /// Temperature for LLM sampling during reasoning (0.0 = deterministic).
    pub temperature: f64,

    /// Enable parallel step evaluation when dependencies allow.
    pub parallel_steps: bool,
}

impl Default for CoTConfig {
    fn default() -> Self {
        Self {
            max_steps: 20,
            max_depth: 5,
            confidence_threshold: 0.7,
            trace_enabled: false,
            beam_width: 4,
            step_timeout_ms: 30_000,
            enable_self_correction: true,
            reasoning_mode: ReasoningMode::Linear,
            temperature: 0.7,
            parallel_steps: true,
        }
    }
}

/// A single step in a reasoning chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThoughtStep {
    /// Unique step index within the chain (0-based).
    pub step_index: usize,

    /// Natural language description of this reasoning step.
    pub thought: String,

    /// Confidence in this step's correctness (0.0–1.0).
    pub confidence: f64,

    /// Reasoning type used to produce this step.
    pub reasoning_type: ReasoningType,

    /// Optional: ID of the step this one depends on (for recursive mode).
    pub parent_step: Option<usize>,

    /// Whether introspection marked this step as needing correction.
    pub flagged: bool,

    /// Serialized intermediate reasoning state (model-specific).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scratchpad: Option<String>,
}

/// Type of reasoning applied at a given step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningType {
    /// Direct logical deduction from premises.
    Deduction,
    /// Inference to the best explanation.
    Abduction,
    /// Reasoning by structural analogy.
    Analogy,
    /// Commonsense inference.
    Commonsense,
    /// Self-referential reflection.
    Reflection,
    /// Planning / goal decomposition.
    Planning,
    /// Verification / sanity check.
    Verification,
}

impl std::fmt::Display for ReasoningType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReasoningType::Deduction => write!(f, "deduction"),
            ReasoningType::Abduction => write!(f, "abduction"),
            ReasoningType::Analogy => write!(f, "analogy"),
            ReasoningType::Commonsense => write!(f, "commonsense"),
            ReasoningType::Reflection => write!(f, "reflection"),
            ReasoningType::Planning => write!(f, "planning"),
            ReasoningType::Verification => write!(f, "verification"),
        }
    }
}

/// Complete result of a CoT reasoning session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoTResult {
    /// The final conclusion or answer.
    pub conclusion: String,

    /// All thought steps taken to reach the conclusion.
    pub reasoning_chain: Vec<ThoughtStep>,

    /// Aggregate confidence in the final result (0.0–1.0).
    pub final_confidence: f64,

    /// Whether any step was flagged for correction.
    pub had_corrections: bool,

    /// Total reasoning steps executed.
    pub total_steps: usize,

    /// Mode used for this reasoning session.
    pub mode: ReasoningMode,

    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
}

/// Errors that can occur during CoT reasoning.
#[derive(Debug, thiserror::Error)]
pub enum CoTError {
    #[error("Reasoning timed out after {0}ms")]
    Timeout(u64),

    #[error("Maximum steps ({0}) exceeded")]
    MaxStepsExceeded(usize),

    #[error("Maximum depth ({0}) exceeded")]
    MaxDepthExceeded(usize),

    #[error("Empty reasoning chain — no thoughts generated")]
    EmptyChain,

    #[error("LLM inference failed: {0}")]
    InferenceFailed(String),

    #[error("Configuration error: {0}")]
    InvalidConfig(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<CoTError> for String {
    fn from(e: CoTError) -> String {
        e.to_string()
    }
}
