//! # APEX SkillFlow Core
//!
//! Flow matching engine for trajectory-based skill composition.
//!
//! ## Core formula
//!
//! ```text
//! π*(τ|q) ∝ R̃(τ)^β       R̃(τ) = R(τ) + ε_min
//! Flow(s→a) ∝ Reward(trajectory through s→a)
//! ```
//!
//! Provides: trajectory reward normalization, flow probability calculation,
//! transparent credit allocation across nodes, policy collapse detection,
//! and multi-peak redundancy analysis.

mod schema;

use std::collections::HashMap;

// Re-export schema types so callers can work with DAG nodes.
pub use schema::{DagNode, DatasetBinding, Schema};

/// A trajectory's accumulated reward.
///
/// `trajectory_id` uniquely identifies a path through the skill DAG.
/// `reward` is the raw accumulated score (f64).
/// `epsilon_min` is the additive smoothing constant (default 0.01).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryReward {
    pub trajectory_id: String,
    pub reward: f64,
    #[serde(default = "default_epsilon")]
    pub epsilon_min: f64,
}

fn default_epsilon() -> f64 {
    0.01
}

/// A directed flow edge between two DAG nodes.
///
/// `from_node` / `to_node` identify the edge.
/// `flow_value` is the raw flow magnitude.
/// `beta` is the exponent for the normalized flow probability (default 1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowProbability {
    pub from_node: String,
    pub to_node: String,
    pub flow_value: f64,
    #[serde(default = "default_beta")]
    pub beta: f64,
}

fn default_beta() -> f64 {
    1.0
}

/// Compute the smoothed (regularised) reward:  R̃ = R + ε_min
pub fn normalized_reward(t: &TrajectoryReward) -> f64 {
    t.reward + t.epsilon_min
}

/// Compute the soft flow distribution over edges:
///
/// ```text
/// π*(edge_i | q) = (flow_i ^ β) / Σ_j (flow_j ^ β)
/// ```
///
/// Returns a vector of probabilities in the same order as `flows`.
/// If all flows are zero, returns a uniform distribution.
pub fn flow_probability(flows: &[FlowProbability], beta: f64) -> Vec<f64> {
    let total: f64 = flows.iter().map(|f| f.flow_value.powf(beta)).sum();

    if total <= 0.0 {
        // Degenerate case — uniform over all edges
        let n = flows.len() as f64;
        return flows.iter().map(|_| 1.0 / n).collect();
    }

    flows
        .iter()
        .map(|f| f.flow_value.powf(beta) / total)
        .collect()
}

/// Allocate credit transparently from trajectory rewards to flow edges.
///
/// The reward of each trajectory is distributed equally across its constituent
/// edges.  The output map is keyed by `"from→to"`.
pub fn credit_allocation(
    trajectories: &[TrajectoryReward],
    flows: &[FlowProbability],
) -> HashMap<String, f64> {
    let mut credit: HashMap<String, f64> = HashMap::new();

    for t in trajectories {
        let r_tilde = normalized_reward(t);
        // Find all flows that share this trajectory's ID (a trajectory
        // touches multiple edges).  We split the reward equally.
        let related: Vec<&FlowProbability> =
            flows.iter().filter(|f| f.from_node == t.trajectory_id || f.to_node == t.trajectory_id).collect();

        if related.is_empty() {
            continue;
        }

        let share = r_tilde / related.len() as f64;
        for f in &related {
            let key = format!("{}→{}", f.from_node, f.to_node);
            *credit.entry(key).or_insert(0.0) += share;
        }
    }

    credit
}

/// Detect policy collapse.
///
/// A collapsed policy produces a near-uniform flow distribution where all
/// edges carry roughly the same flow — i.e. the model has failed to learn
/// meaningful differentiation.  We detect this via the **coefficient of
/// variation** (CV = std / mean).  When CV falls below `threshold` the
/// distribution is too flat and we flag collapse.
///
/// Empty or all-zero flow vectors are always considered collapsed.
pub fn detect_collapse(flows: &[FlowProbability], threshold: f64) -> bool {
    let n = flows.len();
    if n == 0 {
        return true;
    }

    let values: Vec<f64> = flows.iter().map(|f| f.flow_value).collect();
    let mean = values.iter().sum::<f64>() / n as f64;

    if mean <= 0.0 {
        return true; // degenerate
    }

    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let std_dev = variance.sqrt();
    let cv = std_dev / mean; // coefficient of variation

    cv < threshold
}

/// Identify nodes that carry redundant flow load across multiple peaks.
///
/// A "peak" is a flow edge whose value is above the mean + 1 std.
/// Nodes that appear in more than one peak edge are flagged as
/// potential redundant load-bearing nodes.  Returns the indices
/// (into `flows`) of those peak edges.
pub fn multi_peak_redundancy(flows: &[FlowProbability]) -> Vec<usize> {
    let n = flows.len();
    if n == 0 {
        return vec![];
    }

    let values: Vec<f64> = flows.iter().map(|f| f.flow_value).collect();
    let mean = values.iter().sum::<f64>() / n as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let std_dev = variance.sqrt();
    let peak_threshold = mean + std_dev;

    // Collect peak edge indices
    let peak_indices: Vec<usize> = (0..n)
        .filter(|&i| flows[i].flow_value > peak_threshold)
        .collect();

    // Count node frequency across peaks
    let mut node_count: HashMap<&str, usize> = HashMap::new();
    for &i in &peak_indices {
        *node_count.entry(&flows[i].from_node).or_insert(0) += 1;
        *node_count.entry(&flows[i].to_node).or_insert(0) += 1;
    }

    // A node is "redundant" if it appears in >1 peak
    let redundant_nodes: Vec<&str> = node_count
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(node, _)| node)
        .collect();

    // Return peak edge indices whose either endpoint is a redundant node
    peak_indices
        .into_iter()
        .filter(|&i| {
            let f = &flows[i];
            redundant_nodes.contains(&f.from_node.as_str())
                || redundant_nodes.contains(&f.to_node.as_str())
        })
        .collect()
}

// Ensure serde derive macros are available in the crate root.
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_trajectories() -> Vec<TrajectoryReward> {
        vec![
            TrajectoryReward {
                trajectory_id: "traj_1".into(),
                reward: 0.85,
                epsilon_min: 0.01,
            },
            TrajectoryReward {
                trajectory_id: "traj_2".into(),
                reward: 0.42,
                epsilon_min: 0.01,
            },
            TrajectoryReward {
                trajectory_id: "traj_3".into(),
                reward: 0.91,
                epsilon_min: 0.01,
            },
        ]
    }

    fn sample_flows() -> Vec<FlowProbability> {
        vec![
            FlowProbability {
                from_node: "traj_1".into(),
                to_node: "node_a".into(),
                flow_value: 0.8,
                beta: 1.0,
            },
            FlowProbability {
                from_node: "traj_1".into(),
                to_node: "node_b".into(),
                flow_value: 0.6,
                beta: 1.0,
            },
            FlowProbability {
                from_node: "traj_2".into(),
                to_node: "node_b".into(),
                flow_value: 0.3,
                beta: 1.0,
            },
            FlowProbability {
                from_node: "traj_3".into(),
                to_node: "node_a".into(),
                flow_value: 0.9,
                beta: 1.0,
            },
            FlowProbability {
                from_node: "traj_3".into(),
                to_node: "node_c".into(),
                flow_value: 0.7,
                beta: 1.0,
            },
        ]
    }

    #[test]
    fn test_normalized_reward() {
        let t = TrajectoryReward {
            trajectory_id: "test".into(),
            reward: 0.5,
            epsilon_min: 0.01,
        };
        let r_tilde = normalized_reward(&t);
        assert!((r_tilde - 0.51).abs() < 1e-12);
    }

    #[test]
    fn test_flow_probability_sum_one() {
        let flows = sample_flows();
        let probs = flow_probability(&flows, 1.0);
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_flow_probability_with_beta() {
        let flows = sample_flows();
        let probs_beta1 = flow_probability(&flows, 1.0);
        let probs_beta2 = flow_probability(&flows, 2.0);
        // Higher beta sharpens the distribution
        let var1: f64 = probs_beta1
            .iter()
            .map(|p| (p - probs_beta1.iter().sum::<f64>() / 5.0).powi(2))
            .sum();
        let var2: f64 = probs_beta2
            .iter()
            .map(|p| (p - probs_beta2.iter().sum::<f64>() / 5.0).powi(2))
            .sum();
        assert!(var2 > var1);
    }

    #[test]
    fn test_credit_allocation() {
        let trajs = sample_trajectories();
        let flows = sample_flows();
        let credit = credit_allocation(&trajs, &flows);

        // The map should contain entries for edges that belong to
        // trajectories sharing a node ID with the flow from_node or to_node
        assert!(!credit.is_empty());
        for (edge, value) in &credit {
            assert!(*value > 0.0, "credit for {} must be positive", edge);
        }
    }

    #[test]
    fn test_detect_collapse_normal() {
        let flows = sample_flows();
        assert!(!detect_collapse(&flows, 0.3));
    }

    #[test]
    fn test_detect_collapse_uniform() {
        let flows = vec![
            FlowProbability {
                from_node: "a".into(),
                to_node: "b".into(),
                flow_value: 1.0,
                beta: 1.0,
            },
            FlowProbability {
                from_node: "a".into(),
                to_node: "c".into(),
                flow_value: 1.0,
                beta: 1.0,
            },
        ];
        // Uniform distribution → CV=0, should flag collapse
        assert!(detect_collapse(&flows, 0.01));
    }

    #[test]
    fn test_multi_peak_redundancy() {
        // 12 flows: 10 at baseline (1.0) + 2 high peaks sharing node "a" (5.0).
        // mean ≈ 1.667, std ≈ 1.49, threshold ≈ 3.16
        // → 5.0 values are peaks, both involve "a" → redundant.
        let mut flows: Vec<FlowProbability> = (0..10)
            .map(|i| FlowProbability {
                from_node: format!("base_{}", i),
                to_node: format!("base_{}", i + 1),
                flow_value: 1.0,
                beta: 1.0,
            })
            .collect();
        flows.push(FlowProbability {
            from_node: "a".into(),
            to_node: "b".into(),
            flow_value: 5.0,
            beta: 1.0,
        });
        flows.push(FlowProbability {
            from_node: "a".into(),
            to_node: "c".into(),
            flow_value: 5.0,
            beta: 1.0,
        });
        let redundant = multi_peak_redundancy(&flows);
        // Node "a" appears in both peak edges → should detect redundancy
        assert!(!redundant.is_empty(), "expected redundant peak edges");
        assert_eq!(redundant.len(), 2, "both peak edges (a→b, a→c) should be flagged");
    }

    #[test]
    fn test_empty_flows_collapse() {
        let flows: Vec<FlowProbability> = vec![];
        assert!(detect_collapse(&flows, 0.3));
    }

    #[test]
    fn test_empty_flows_multi_peak() {
        let flows: Vec<FlowProbability> = vec![];
        assert!(multi_peak_redundancy(&flows).is_empty());
    }
}
