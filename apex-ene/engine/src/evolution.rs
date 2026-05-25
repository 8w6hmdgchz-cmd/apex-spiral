/// ∇Θ Evolution Loop
///
/// Replaces mechanical cron-based scheduling with dynamic evolution cycles.
/// Cycle period is adaptive: success shortens it, failure lengthens it.
/// The loop is self-driven, not externally scheduled.

use crate::apexe::{ApexDeltaE, ApexDimensions};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionState {
    pub version: u64,
    pub total_runs: u64,
    pub successful_runs: u64,
    pub failed_runs: u64,
    pub consecutive_failures: u64,
    pub current_cycle_hours: f64,
    pub min_cycle_hours: f64,
    pub max_cycle_hours: f64,
    pub last_run: String,
    pub next_run: String,
    pub history: Vec<ApexDeltaE>,
    pub bottleneck_focus: String,
}

impl EvolutionState {
    pub fn new() -> Self {
        Self {
            version: 1,
            total_runs: 0,
            successful_runs: 0,
            failed_runs: 0,
            consecutive_failures: 0,
            current_cycle_hours: 24.0,  // Start at daily cycle
            min_cycle_hours: 1.0,       // Can go as fast as 1 hour
            max_cycle_hours: 168.0,     // Can go as slow as 1 week
            last_run: String::new(),
            next_run: String::new(),
            history: Vec::new(),
            bottleneck_focus: "αΨ".to_string(),
        }
    }

    /// Dynamic cycle adjustment based on success/failure
    pub fn adapt_cycle(&mut self, success: bool) {
        self.total_runs += 1;
        if success {
            self.successful_runs += 1;
            self.consecutive_failures = 0;
            // Success: shorten the cycle (accelerate evolution)
            self.current_cycle_hours = (self.current_cycle_hours * 0.85)
                .max(self.min_cycle_hours);
        } else {
            self.failed_runs += 1;
            self.consecutive_failures += 1;
            // Failure: lengthen the cycle (stabilize)
            let penalty = 1.0 + (self.consecutive_failures as f64 * 0.5);
            self.current_cycle_hours = (self.current_cycle_hours * penalty)
                .min(self.max_cycle_hours);
        }
    }

    /// Get success rate over recent runs
    pub fn success_rate(&self, recent_n: usize) -> f64 {
        let relevant = self.history.iter().rev().take(recent_n);
        let total = relevant.len() as f64;
        if total == 0.0 {
            return 0.0;
        }
        let successes = relevant.filter(|h| h.delta_from_previous > 0.0).count() as f64;
        successes / total * 100.0
    }

    /// Identify which dimension needs the most attention
    pub fn focus_bottleneck(&mut self, current: &ApexDimensions) -> String {
        let focus = current.bottleneck().to_string();
        self.bottleneck_focus = focus.clone();
        focus
    }

    /// Generate next focus area as machine-readable instruction
    pub fn evolution_directive(&mut self, current: &ApexDimensions) -> String {
        let focus = self.focus_bottleneck(current);
        match focus.as_str() {
            "αΨ" => "IMPROVE_LLM_ROUTING: optimize model selection, reduce latency, add fallbacks",
            "βΩ" => "REFACTOR_CODE: restructure core architecture, fix vulnerabilities, optimize performance",
            "λΦ" => "EXPAND_KNOWLEDGE: scavenge new sources, absorb recent papers, refresh stale knowledge",
            "∇Θ" => "ACCELERATE_ITERATION: increase evolution frequency, reduce cycle time, push harder deltas",
            "Evol_code" => "ENHANCE_SELF_MODIFICATION: improve code generation quality, increase test coverage",
            _ => "MAINTAIN: all dimensions stable, continue current trajectory",
        }
        .to_string()
    }
}

/// Evolution Controller
pub struct EvolutionController {
    pub state: EvolutionState,
    state_path: PathBuf,
    version_counter: AtomicU64,
}

impl EvolutionController {
    pub fn new(state_path: PathBuf) -> Self {
        let state = if state_path.exists() {
            let content = fs::read_to_string(&state_path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_else(|_| EvolutionState::new())
        } else {
            EvolutionState::new()
        };

        let version = state.version;
        Self {
            state,
            state_path,
            version_counter: AtomicU64::new(version),
        }
    }

    /// Run one evolution cycle
    pub fn run_cycle(&mut self, current_dims: ApexDimensions) -> ApexDeltaE {
        let version = self.version_counter.fetch_add(1, Ordering::SeqCst);

        // Get previous total for delta calculation
        let previous_total = self.state.history.last()
            .map(|h| h.total)
            .unwrap_or(0.0);

        let result = ApexDeltaE::new(current_dims, version, previous_total);

        // Adapt cycle based on whether total improved
        let improved = result.delta_from_previous > 0.0;
        self.state.adapt_cycle(improved);

        // Update focus bottleneck
        self.state.focus_bottleneck(&result.dimensions);

        // Store in history
        self.state.history.push(result.clone());
        if self.state.history.len() > 100 {
            self.state.history.remove(0);
        }

        self.state.version = version;
        self.state.last_run = chrono::Utc::now().to_rfc3339();

        // Calculate next run time
        let next = chrono::Utc::now()
            + chrono::Duration::hours(self.state.current_cycle_hours as i64);
        self.state.next_run = next.to_rfc3339();

        // Persist state
        self.save_state();

        result
    }

    /// Get evolution directive for current cycle
    pub fn get_directive(&mut self, current: &ApexDimensions) -> String {
        self.state.evolution_directive(current)
    }

    fn save_state(&self) {
        if let Some(parent) = self.state_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(&self.state).unwrap_or_default();
        let _ = fs::write(&self.state_path, content);
    }
}
