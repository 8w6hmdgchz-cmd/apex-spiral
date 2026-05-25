use serde::{Deserialize, Serialize};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Phase {
    Observe,
    Act,
    Verify,
    Repair,
    Consolidate,
}

#[derive(Debug, Serialize, Deserialize)]
struct EvolverInput {
    objective: String,
    last_status: Option<String>,
    verification_score: Option<f64>,
    secret_hit_count: Option<u64>,
}

#[derive(Debug, Serialize)]
struct EvolverOutput {
    status: String,
    phase: Phase,
    next_phase: Phase,
    action: String,
    fitness: f64,
    promotion: String,
    reason: String,
    timestamp_ms: u128,
}

fn now_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}

fn next_phase(phase: Phase, input: &EvolverInput) -> Phase {
    let status = input.last_status.as_deref().unwrap_or("ok");
    let score = input.verification_score.unwrap_or(0.0);
    let secrets = input.secret_hit_count.unwrap_or(0);
    if secrets > 0 || status == "failed" || status == "blocked" {
        return Phase::Repair;
    }
    match phase {
        Phase::Observe => Phase::Act,
        Phase::Act => Phase::Verify,
        Phase::Verify if score >= 0.7 => Phase::Consolidate,
        Phase::Verify => Phase::Repair,
        Phase::Repair => Phase::Act,
        Phase::Consolidate => Phase::Observe,
    }
}

fn action_for(phase: Phase, next: Phase) -> &'static str {
    match (phase, next) {
        (Phase::Observe, Phase::Act) => "collect current state and execute bounded next step",
        (Phase::Act, Phase::Verify) => "run verification gate after action",
        (Phase::Verify, Phase::Consolidate) => "promote verified artifact into evolution memory",
        (Phase::Verify, Phase::Repair) => "repair failed or low-confidence artifact before promotion",
        (Phase::Repair, Phase::Act) => "apply minimal repair and retry action",
        (Phase::Consolidate, Phase::Observe) => "start next observation cycle",
        (_, Phase::Repair) => "enter repair due to safety or execution failure",
        _ => "continue deterministic evolution cycle",
    }
}

fn fitness(input: &EvolverInput) -> f64 {
    let score = input.verification_score.unwrap_or(0.0).clamp(0.0, 1.0);
    let status_score = match input.last_status.as_deref().unwrap_or("ok") {
        "ok" => 1.0,
        "warn" => 0.7,
        "failed" | "blocked" => 0.0,
        _ => 0.5,
    };
    let secret_score = if input.secret_hit_count.unwrap_or(0) == 0 { 1.0 } else { 0.0 };
    ((score + status_score + secret_score) / 3.0 * 10000.0).round() / 10000.0
}

fn parse_phase(s: &str) -> Option<Phase> {
    match s {
        "observe" => Some(Phase::Observe),
        "act" => Some(Phase::Act),
        "verify" => Some(Phase::Verify),
        "repair" => Some(Phase::Repair),
        "consolidate" => Some(Phase::Consolidate),
        _ => None,
    }
}

fn emit(output: EvolverOutput) {
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 2 {
        emit(EvolverOutput {
            status: "blocked".into(),
            phase: Phase::Observe,
            next_phase: Phase::Repair,
            action: "usage: tiangong_evolver_rs <phase> <json-input>".into(),
            fitness: 0.0,
            promotion: "blocked".into(),
            reason: "missing arguments".into(),
            timestamp_ms: now_ms(),
        });
        std::process::exit(2);
    }
    let phase = match parse_phase(&args[0]) {
        Some(p) => p,
        None => {
            emit(EvolverOutput {
                status: "blocked".into(),
                phase: Phase::Observe,
                next_phase: Phase::Repair,
                action: "use a valid phase".into(),
                fitness: 0.0,
                promotion: "blocked".into(),
                reason: "invalid phase".into(),
                timestamp_ms: now_ms(),
            });
            std::process::exit(3);
        }
    };
    let input: EvolverInput = match serde_json::from_str(&args[1]) {
        Ok(i) => i,
        Err(err) => {
            emit(EvolverOutput {
                status: "blocked".into(),
                phase,
                next_phase: Phase::Repair,
                action: "repair invalid input".into(),
                fitness: 0.0,
                promotion: "blocked".into(),
                reason: format!("invalid json: {}", err),
                timestamp_ms: now_ms(),
            });
            std::process::exit(4);
        }
    };
    let fit = fitness(&input);
    let next = next_phase(phase, &input);
    let promotion = if phase == Phase::Verify && next == Phase::Consolidate && fit >= 0.7 {
        "pass"
    } else if next == Phase::Repair {
        "hold"
    } else {
        "continue"
    };
    emit(EvolverOutput {
        status: "ok".into(),
        phase,
        next_phase: next,
        action: action_for(phase, next).into(),
        fitness: fit,
        promotion: promotion.into(),
        reason: format!("objective='{}'", input.objective),
        timestamp_ms: now_ms(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(score: f64, status: &str, secrets: u64) -> EvolverInput {
        EvolverInput {
            objective: "test".into(),
            last_status: Some(status.into()),
            verification_score: Some(score),
            secret_hit_count: Some(secrets),
        }
    }

    #[test]
    fn verify_promotes_when_score_ok() {
        assert_eq!(next_phase(Phase::Verify, &sample(0.9, "ok", 0)), Phase::Consolidate);
    }

    #[test]
    fn verify_repairs_when_score_low() {
        assert_eq!(next_phase(Phase::Verify, &sample(0.5, "ok", 0)), Phase::Repair);
    }

    #[test]
    fn secrets_force_repair() {
        assert_eq!(next_phase(Phase::Act, &sample(0.9, "ok", 1)), Phase::Repair);
    }

    #[test]
    fn fitness_penalizes_failure() {
        assert!(fitness(&sample(0.9, "failed", 0)) < 0.7);
    }
}
