use serde::{Deserialize, Serialize};
use std::env;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
struct GateInput {
    objective: String,
    artifacts: Vec<String>,
    tests_passed: bool,
    secret_hit_count: u64,
    risk: String,
}

#[derive(Debug, Serialize)]
struct GateOutput {
    status: String,
    gate: String,
    passed: bool,
    score: f64,
    checklist: Vec<CheckItem>,
    recommendation: String,
    timestamp_ms: u128,
}

#[derive(Debug, Serialize)]
struct CheckItem {
    name: String,
    passed: bool,
    reason: String,
}

fn now_ms() -> u128 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
}

fn check_requirements(input: &GateInput) -> Vec<CheckItem> {
    vec![
        CheckItem { name: "objective_present".into(), passed: !input.objective.trim().is_empty(), reason: "task has explicit objective".into() },
        CheckItem { name: "risk_declared".into(), passed: matches!(input.risk.as_str(), "low" | "medium" | "high"), reason: "risk must be low/medium/high".into() },
    ]
}

fn check_architecture(input: &GateInput) -> Vec<CheckItem> {
    vec![
        CheckItem { name: "artifacts_tracked".into(), passed: !input.artifacts.is_empty(), reason: "architecture requires traceable artifacts".into() },
        CheckItem { name: "local_first".into(), passed: true, reason: "TianGong native loop is local-first".into() },
    ]
}

fn check_test_plan(input: &GateInput) -> Vec<CheckItem> {
    vec![
        CheckItem { name: "tests_passed".into(), passed: input.tests_passed, reason: "verification gate must pass before promotion".into() },
        CheckItem { name: "secrets_clear".into(), passed: input.secret_hit_count == 0, reason: "secret hits block external sync and promotion".into() },
    ]
}

fn check_review(input: &GateInput) -> Vec<CheckItem> {
    vec![
        CheckItem { name: "high_risk_hold".into(), passed: input.risk != "high", reason: "high-risk tasks require explicit human approval".into() },
        CheckItem { name: "clean_room".into(), passed: true, reason: "capability is represented as native abstraction".into() },
    ]
}

fn checklist(gate: &str, input: &GateInput) -> Vec<CheckItem> {
    match gate {
        "requirements" => check_requirements(input),
        "architecture" => check_architecture(input),
        "test_plan" => check_test_plan(input),
        "review" => check_review(input),
        "full" => {
            let mut all = vec![];
            all.extend(check_requirements(input));
            all.extend(check_architecture(input));
            all.extend(check_test_plan(input));
            all.extend(check_review(input));
            all
        }
        _ => vec![CheckItem { name: "valid_gate".into(), passed: false, reason: "gate must be requirements|architecture|test_plan|review|full".into() }],
    }
}

fn score(items: &[CheckItem]) -> f64 {
    if items.is_empty() { return 0.0; }
    let passed = items.iter().filter(|i| i.passed).count() as f64;
    ((passed / items.len() as f64) * 10000.0).round() / 10000.0
}

fn emit(out: GateOutput) {
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 2 {
        emit(GateOutput {
            status: "blocked".into(),
            gate: "usage".into(),
            passed: false,
            score: 0.0,
            checklist: vec![],
            recommendation: "usage: tiangong_superpowers_rs <gate> <json-input>".into(),
            timestamp_ms: now_ms(),
        });
        std::process::exit(2);
    }
    let gate = args[0].clone();
    let input: GateInput = match serde_json::from_str(&args[1]) {
        Ok(i) => i,
        Err(err) => {
            emit(GateOutput {
                status: "blocked".into(),
                gate,
                passed: false,
                score: 0.0,
                checklist: vec![],
                recommendation: format!("invalid json: {}", err),
                timestamp_ms: now_ms(),
            });
            std::process::exit(3);
        }
    };
    let items = checklist(&gate, &input);
    let sc = score(&items);
    let passed = sc >= 0.9999;
    let recommendation = if passed {
        "gate passed; continue".to_string()
    } else if input.secret_hit_count > 0 {
        "blocked: repair secret hits before promotion".to_string()
    } else if !input.tests_passed {
        "hold: tests must pass before promotion".to_string()
    } else {
        "hold: repair failed checklist items".to_string()
    };
    emit(GateOutput {
        status: "ok".into(),
        gate,
        passed,
        score: sc,
        checklist: items,
        recommendation,
        timestamp_ms: now_ms(),
    });
    std::process::exit(if passed { 0 } else { 1 });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> GateInput {
        GateInput {
            objective: "ship native core".into(),
            artifacts: vec!["artifact.json".into()],
            tests_passed: true,
            secret_hit_count: 0,
            risk: "low".into(),
        }
    }

    #[test]
    fn full_gate_passes_good_input() {
        let items = checklist("full", &good());
        assert_eq!(score(&items), 1.0);
    }

    #[test]
    fn secrets_fail_test_plan() {
        let mut input = good();
        input.secret_hit_count = 1;
        let items = checklist("test_plan", &input);
        assert!(score(&items) < 1.0);
    }

    #[test]
    fn high_risk_fails_review() {
        let mut input = good();
        input.risk = "high".into();
        let items = checklist("review", &input);
        assert!(score(&items) < 1.0);
    }
}
