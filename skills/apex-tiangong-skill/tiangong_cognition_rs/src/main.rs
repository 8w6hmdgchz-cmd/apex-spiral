use serde::{Deserialize, Serialize};
use std::env;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
struct Candidate {
    gene: String,
    evidence: f64,
    feasibility: f64,
    risk: f64,
    reversibility: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CognitionInput {
    objective: String,
    constraints: Vec<String>,
    candidates: Vec<Candidate>,
}

#[derive(Debug, Serialize)]
struct RankedCandidate {
    gene: String,
    score: f64,
    reason: String,
}

#[derive(Debug, Serialize)]
struct CognitionOutput {
    status: String,
    mode: String,
    objective: String,
    roles: Vec<String>,
    decomposition: Vec<String>,
    ranked_options: Vec<RankedCandidate>,
    critique: String,
    falsification_path: String,
    timestamp_ms: u128,
}

fn now_ms() -> u128 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
}

fn clamp01(v: f64) -> f64 {
    if v < 0.0 { 0.0 } else if v > 1.0 { 1.0 } else { v }
}

fn score(c: &Candidate) -> f64 {
    let raw = (0.35 * clamp01(c.evidence))
        + (0.30 * clamp01(c.feasibility))
        + (0.20 * clamp01(c.reversibility))
        + (0.15 * (1.0 - clamp01(c.risk)));
    (raw * 10000.0).round() / 10000.0
}

fn default_candidates() -> Vec<Candidate> {
    vec![
        Candidate { gene: "sandbox adapter".into(), evidence: 0.90, feasibility: 0.92, risk: 0.20, reversibility: 0.85 },
        Candidate { gene: "evidence ledger".into(), evidence: 0.86, feasibility: 0.88, risk: 0.10, reversibility: 0.90 },
        Candidate { gene: "role router".into(), evidence: 0.84, feasibility: 0.86, risk: 0.15, reversibility: 0.88 },
        Candidate { gene: "repair loop".into(), evidence: 0.82, feasibility: 0.84, risk: 0.25, reversibility: 0.80 },
    ]
}

fn roles_for(objective: &str) -> Vec<String> {
    let mut roles = vec!["researcher", "architect", "executor", "reviewer", "evolver"];
    let lower = objective.to_ascii_lowercase();
    if lower.contains("medical") || objective.contains("医学") || objective.contains("科研") {
        roles.push("scientific_critic");
    }
    if lower.contains("rust") || lower.contains("go") || lower.contains("c ") {
        roles.push("systems_engineer");
    }
    roles.into_iter().map(String::from).collect()
}

fn decomposition_for(objective: &str) -> Vec<String> {
    vec![
        format!("clarify boundary for: {}", objective),
        "collect evidence and constraints".into(),
        "rank candidate genes".into(),
        "select reversible implementation path".into(),
        "define verification gate".into(),
        "consolidate only after tests pass".into(),
    ]
}

fn emit(out: CognitionOutput) {
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        emit(CognitionOutput {
            status: "blocked".into(),
            mode: "usage".into(),
            objective: "".into(),
            roles: vec![],
            decomposition: vec![],
            ranked_options: vec![],
            critique: "missing json input".into(),
            falsification_path: "provide valid CognitionInput".into(),
            timestamp_ms: now_ms(),
        });
        std::process::exit(2);
    }
    let mut input: CognitionInput = match serde_json::from_str(&args[0]) {
        Ok(i) => i,
        Err(err) => {
            emit(CognitionOutput {
                status: "blocked".into(),
                mode: "parse".into(),
                objective: "".into(),
                roles: vec![],
                decomposition: vec![],
                ranked_options: vec![],
                critique: format!("invalid json: {}", err),
                falsification_path: "retry with valid JSON".into(),
                timestamp_ms: now_ms(),
            });
            std::process::exit(3);
        }
    };
    if input.candidates.is_empty() {
        input.candidates = default_candidates();
    }
    let mut ranked: Vec<RankedCandidate> = input.candidates.iter().map(|c| RankedCandidate {
        gene: c.gene.clone(),
        score: score(c),
        reason: format!("evidence={:.2}, feasibility={:.2}, risk={:.2}, reversibility={:.2}", c.evidence, c.feasibility, c.risk, c.reversibility),
    }).collect();
    ranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    emit(CognitionOutput {
        status: "ok".into(),
        mode: "multi_role_router".into(),
        objective: input.objective.clone(),
        roles: roles_for(&input.objective),
        decomposition: decomposition_for(&input.objective),
        ranked_options: ranked,
        critique: "Hypotheses are not findings; require verification before promotion.".into(),
        falsification_path: "If tests fail, evidence is missing, or risk exceeds benefit, route to repair/hold.".into(),
        timestamp_ms: now_ms(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_risk_scores_higher_when_other_factors_equal() {
        let a = Candidate { gene: "a".into(), evidence: 0.8, feasibility: 0.8, risk: 0.1, reversibility: 0.8 };
        let b = Candidate { gene: "b".into(), evidence: 0.8, feasibility: 0.8, risk: 0.9, reversibility: 0.8 };
        assert!(score(&a) > score(&b));
    }

    #[test]
    fn systems_role_added_for_rust_objective() {
        let roles = roles_for("implement rust sandbox");
        assert!(roles.contains(&"systems_engineer".to_string()));
    }

    #[test]
    fn default_candidates_not_empty() {
        assert!(!default_candidates().is_empty());
    }
}
