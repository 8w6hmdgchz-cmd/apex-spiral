//! Integration tests for apex-skillflow-core.
//!
//! Tests flow matching, credit allocation across nodes, collapse detection,
//! multi-peak redundancy, and DAG schema loading.

use apex_skillflow_core::{
    credit_allocation, detect_collapse, flow_probability, multi_peak_redundancy,
    normalized_reward, FlowProbability, Schema, TrajectoryReward,
};

/// -----------------------------------------------------------------------
/// 1. Trajectory reward smoothing (normalized_reward)
/// -----------------------------------------------------------------------
#[test]
fn test_trajectory_reward_default_epsilon() {
    let t = TrajectoryReward {
        trajectory_id: "t1".into(),
        reward: 0.75,
        epsilon_min: 0.01,
    };
    let r_tilde = normalized_reward(&t);
    assert!(
        (r_tilde - 0.76).abs() < 1e-12,
        "Expected 0.76, got {}",
        r_tilde
    );
}

#[test]
fn test_trajectory_reward_custom_epsilon() {
    let t = TrajectoryReward {
        trajectory_id: "t2".into(),
        reward: 2.0,
        epsilon_min: 0.5,
    };
    let r_tilde = normalized_reward(&t);
    assert!(
        (r_tilde - 2.5).abs() < 1e-12,
        "Expected 2.5, got {}",
        r_tilde
    );
}

/// -----------------------------------------------------------------------
/// 2. Soft flow probability — π*(τ|q) ∝ R̃(τ)^β
/// -----------------------------------------------------------------------
#[test]
fn test_flow_probability_simple() {
    let flows = vec![
        FlowProbability { from_node: "a".into(), to_node: "b".into(), flow_value: 2.0, beta: 1.0 },
        FlowProbability { from_node: "a".into(), to_node: "c".into(), flow_value: 3.0, beta: 1.0 },
        FlowProbability { from_node: "b".into(), to_node: "c".into(), flow_value: 5.0, beta: 1.0 },
    ];
    let probs = flow_probability(&flows, 1.0);
    let sum: f64 = probs.iter().sum();
    assert!((sum - 1.0).abs() < 1e-12, "Probabilities must sum to 1");

    // With beta=1 and values [2,3,5], probs = [0.2, 0.3, 0.5]
    assert!((probs[0] - 0.2).abs() < 1e-12);
    assert!((probs[1] - 0.3).abs() < 1e-12);
    assert!((probs[2] - 0.5).abs() < 1e-12);
}

#[test]
fn test_flow_probability_all_zero() {
    let flows = vec![
        FlowProbability { from_node: "a".into(), to_node: "b".into(), flow_value: 0.0, beta: 1.0 },
        FlowProbability { from_node: "a".into(), to_node: "c".into(), flow_value: 0.0, beta: 1.0 },
    ];
    let probs = flow_probability(&flows, 1.0);
    let sum: f64 = probs.iter().sum();
    assert!((sum - 1.0).abs() < 1e-12, "Uniform fallback must sum to 1");
    assert!((probs[0] - 0.5).abs() < 1e-12);
}

#[test]
fn test_flow_probability_beta_sharpening() {
    let flows = vec![
        FlowProbability { from_node: "x".into(), to_node: "y".into(), flow_value: 1.0, beta: 1.0 },
        FlowProbability { from_node: "x".into(), to_node: "z".into(), flow_value: 1.0, beta: 1.0 },
    ];
    let b1 = flow_probability(&flows, 1.0);
    let b2 = flow_probability(&flows, 3.0);
    let b4 = flow_probability(&flows, 5.0);
    // Uniform values → all betas give uniform
    for (p1, p2) in b1.iter().zip(b2.iter()) {
        assert!((p1 - p2).abs() < 1e-12);
    }
    for (p1, p2) in b1.iter().zip(b4.iter()) {
        assert!((p1 - p2).abs() < 1e-12);
    }
}

/// -----------------------------------------------------------------------
/// 3. Transparent credit allocation
/// -----------------------------------------------------------------------
#[test]
fn test_credit_allocation_simple() {
    let trajs = vec![
        TrajectoryReward { trajectory_id: "n_A".into(), reward: 0.8, epsilon_min: 0.01 },
        TrajectoryReward { trajectory_id: "n_B".into(), reward: 0.4, epsilon_min: 0.01 },
    ];
    let flows = vec![
        FlowProbability { from_node: "n_A".into(), to_node: "n_B".into(), flow_value: 0.7, beta: 1.0 },
        FlowProbability { from_node: "n_A".into(), to_node: "n_C".into(), flow_value: 0.3, beta: 1.0 },
    ];
    let credit = credit_allocation(&trajs, &flows);

    // n_A→n_B gets r_tilde(n_A) / 2 = 0.81/2 = 0.405  (n_A appears in 2 flow edges)
    // n_A→n_C gets r_tilde(n_A) / 2 = 0.405
    // n_B→... nothing extra because n_B only appears in flows[0] which is from n_A, not from n_B
    // Wait, credit_allocation matches trajectory_id against from_node OR to_node
    // So for traj "n_A": matches flows[0].from_node and flows[1].from_node → 2 matches, share = 0.81/2 = 0.405
    // For traj "n_B": matches flows[0].to_node → 1 match, share = 0.41/1 = 0.41
    // So n_A→n_B gets 0.405 + 0.41 = 0.815
    // And n_A→n_C gets 0.405

    assert!(credit.contains_key("n_A→n_B"));
    assert!(credit.contains_key("n_A→n_C"));
    assert!((credit["n_A→n_B"] - 0.815).abs() < 1e-10);
    assert!((credit["n_A→n_C"] - 0.405).abs() < 1e-10);
}

#[test]
fn test_credit_allocation_no_related_flows() {
    let trajs = vec![TrajectoryReward {
        trajectory_id: "orphan".into(),
        reward: 1.0,
        epsilon_min: 0.01,
    }];
    let flows = vec![FlowProbability {
        from_node: "a".into(),
        to_node: "b".into(),
        flow_value: 1.0,
        beta: 1.0,
    }];
    let credit = credit_allocation(&trajs, &flows);
    assert!(credit.is_empty(), "Orphan trajectory produces no credit");
}

/// -----------------------------------------------------------------------
/// 4. Policy collapse detection
/// -----------------------------------------------------------------------
#[test]
fn test_detect_collapse_healthy() {
    // High variance → no collapse
    let flows = vec![
        FlowProbability { from_node: "a".into(), to_node: "b".into(), flow_value: 10.0, beta: 1.0 },
        FlowProbability { from_node: "c".into(), to_node: "d".into(), flow_value: 0.1, beta: 1.0 },
    ];
    assert!(!detect_collapse(&flows, 0.3));
}

#[test]
fn test_detect_collapse_degenerate_uniform() {
    // Perfectly uniform → CV=0 → collapsed
    let flows = vec![
        FlowProbability { from_node: "a".into(), to_node: "b".into(), flow_value: 1.0, beta: 1.0 },
        FlowProbability { from_node: "c".into(), to_node: "d".into(), flow_value: 1.0, beta: 1.0 },
    ];
    assert!(detect_collapse(&flows, 0.01));
}

#[test]
fn test_detect_collapse_all_zero() {
    let flows = vec![
        FlowProbability { from_node: "a".into(), to_node: "b".into(), flow_value: 0.0, beta: 1.0 },
        FlowProbability { from_node: "c".into(), to_node: "d".into(), flow_value: 0.0, beta: 1.0 },
    ];
    assert!(detect_collapse(&flows, 0.1));
}

#[test]
fn test_detect_collapse_deterministic_not_collapsed() {
    // Very deterministic (one edge dominates) — high CV, not flat → not collapsed
    let flows = vec![
        FlowProbability { from_node: "a".into(), to_node: "b".into(), flow_value: 100.0, beta: 1.0 },
        FlowProbability { from_node: "c".into(), to_node: "d".into(), flow_value: 0.001, beta: 1.0 },
        FlowProbability { from_node: "e".into(), to_node: "f".into(), flow_value: 0.001, beta: 1.0 },
    ];
    // High variance → not collapsed (just very selective)
    assert!(!detect_collapse(&flows, 0.01));
}

/// -----------------------------------------------------------------------
/// 5. Multi-peak redundancy
/// -----------------------------------------------------------------------
#[test]
fn test_multi_peak_redundancy_two_peaks() {
    // 12 flows: 10 baseline at 1.0 + 2 high peaks sharing "hub" (5.0).
    // mean ≈ 1.667, std ≈ 1.49, threshold ≈ 3.16 → both 5.0s are peaks.
    let mut flows: Vec<FlowProbability> = (0..10)
        .map(|i| FlowProbability {
            from_node: format!("base_{}", i),
            to_node: format!("base_{}", i + 1),
            flow_value: 1.0,
            beta: 1.0,
        })
        .collect();
    flows.push(FlowProbability { from_node: "hub".into(), to_node: "a".into(), flow_value: 5.0, beta: 1.0 });
    flows.push(FlowProbability { from_node: "hub".into(), to_node: "b".into(), flow_value: 5.0, beta: 1.0 });
    let redundant = multi_peak_redundancy(&flows);
    assert_eq!(redundant.len(), 2, "Both peak edges involve hub");
}

#[test]
fn test_multi_peak_no_redundancy() {
    // 12 flows: 10 baseline at 1.0 + 2 disjoint high peaks (5.0 each, different nodes).
    let mut flows: Vec<FlowProbability> = (0..10)
        .map(|i| FlowProbability {
            from_node: format!("base_{}", i),
            to_node: format!("base_{}", i + 1),
            flow_value: 1.0,
            beta: 1.0,
        })
        .collect();
    flows.push(FlowProbability { from_node: "a".into(), to_node: "b".into(), flow_value: 5.0, beta: 1.0 });
    flows.push(FlowProbability { from_node: "c".into(), to_node: "d".into(), flow_value: 5.0, beta: 1.0 });
    let redundant = multi_peak_redundancy(&flows);
    // "a" and "c" each appear in only 1 peak → no overlap
    assert!(redundant.is_empty(), "No node appears in >1 peak");
}

/// -----------------------------------------------------------------------
/// 6. Full end-to-end: flow matching pipeline
/// -----------------------------------------------------------------------
#[test]
fn test_end_to_end_flow_matching() {
    // Simulate a small DAG: four trajectories through three skill nodes
    let trajs = vec![
        TrajectoryReward { trajectory_id: "n1".into(), reward: 0.90, epsilon_min: 0.01 },
        TrajectoryReward { trajectory_id: "n2".into(), reward: 0.60, epsilon_min: 0.01 },
        TrajectoryReward { trajectory_id: "n3".into(), reward: 0.80, epsilon_min: 0.01 },
        TrajectoryReward { trajectory_id: "n4".into(), reward: 0.30, epsilon_min: 0.01 },
    ];

    let flows = vec![
        // Flows through trajectories — use differentiated values to avoid flat collapse
        FlowProbability { from_node: "n1".into(), to_node: "n2".into(), flow_value: 9.0, beta: 1.0 },
        FlowProbability { from_node: "n2".into(), to_node: "n3".into(), flow_value: 7.0, beta: 1.0 },
        FlowProbability { from_node: "n1".into(), to_node: "n3".into(), flow_value: 3.0, beta: 1.0 },
        FlowProbability { from_node: "n3".into(), to_node: "n4".into(), flow_value: 5.0, beta: 1.0 },
    ];

    // 1. Flow probabilities sum to 1
    let probs = flow_probability(&flows, 2.0); // beta=2 for sharper weighting
    assert!((probs.iter().sum::<f64>() - 1.0).abs() < 1e-10);

    // 2. Credit allocation produces identifiable edge-level scores
    let credit = credit_allocation(&trajs, &flows);
    assert_eq!(credit.len(), 4);
    for (edge, val) in &credit {
        assert!(*val > 0.0, "Edge {} must have positive credit", edge);
    }

    // 3. No collapse detected for well-differentiated flows
    assert!(!detect_collapse(&flows, 0.3));

    // 4. High-beta flow probabilities distinguish edges better
    let flat_probs = flow_probability(&flows, 1.0);
    let sharp_probs = flow_probability(&flows, 4.0);
    let spread_flat: f64 = flat_probs.iter().map(|p| (p - 0.25).powi(2)).sum();
    let spread_sharp: f64 = sharp_probs.iter().map(|p| (p - 0.25).powi(2)).sum();
    assert!(spread_sharp > spread_flat, "Higher beta should sharpen distribution");
}

/// -----------------------------------------------------------------------
/// 7. DAG schema loading via include_str!
/// -----------------------------------------------------------------------
#[test]
fn test_schema_loads_all_datasets() {
    let schema_json = include_str!("../schema.json");
    let schema: Schema = serde_json::from_str(schema_json).expect("schema.json should deserialize");
    assert_eq!(schema.datasets.len(), 14, "Schema must contain exactly 14 datasets");
}

#[test]
fn test_schema_all_domains_present() {
    let schema_json = include_str!("../schema.json");
    let schema: Schema = serde_json::from_str(schema_json).unwrap();
    let domains: std::collections::HashSet<&str> =
        schema.datasets.iter().map(|d| d.domain.as_str()).collect();
    for expected in &[
        "reasoning", "code", "math", "agentic", "nlp", "science", "swe",
        "ethics", "embodied", "security", "finance", "biomed", "safety",
    ] {
        assert!(domains.contains(expected), "Domain '{}' is missing", expected);
    }
}

/// -----------------------------------------------------------------------
/// 8. Edge cases
/// -----------------------------------------------------------------------
#[test]
fn test_single_flow_probability() {
    let flows = vec![FlowProbability {
        from_node: "a".into(),
        to_node: "b".into(),
        flow_value: 42.0,
        beta: 1.0,
    }];
    let probs = flow_probability(&flows, 1.0);
    assert!((probs[0] - 1.0).abs() < 1e-12);
}

#[test]
fn test_trajectory_with_negative_reward() {
    let t = TrajectoryReward {
        trajectory_id: "neg".into(),
        reward: -0.5,
        epsilon_min: 0.01,
    };
    let r_tilde = normalized_reward(&t);
    assert!((r_tilde - (-0.49)).abs() < 1e-12);
}
