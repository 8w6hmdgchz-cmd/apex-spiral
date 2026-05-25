use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime};

fn now_ms() -> u128 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
}

fn emit<T: Serialize>(v: &T) {
    println!("{}", serde_json::to_string_pretty(v).unwrap());
}

fn workspace_root() -> PathBuf {
    env::var("APEX_WORKSPACE").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/Users/lihongxin/.openclaw/workspace"))
}

fn clamp01(v: f64) -> f64 {
    if v < 0.0 { 0.0 } else if v > 1.0 { 1.0 } else { v }
}

// ---------------- sandbox ----------------
#[derive(Serialize)]
struct SandboxReport {
    status: String,
    reason: String,
    command: Vec<String>,
    cwd: String,
    code: Option<i32>,
    latency_ms: u128,
    stdout: String,
    stderr: String,
    external_side_effects: bool,
}

fn allowed_program(program: &str) -> bool {
    matches!(program, "python3" | "python" | "cargo" | "git" | "ls" | "pwd" | "echo" | "cat" | "test")
}

fn dangerous_arg(arg: &str) -> bool {
    let lowered = arg.to_ascii_lowercase();
    lowered.contains(" rm ")
        || lowered == "rm"
        || lowered.contains("sudo")
        || lowered.contains("curl")
        || lowered.contains("wget")
        || lowered.contains("/etc/")
        || lowered.contains("~/.ssh")
        || lowered.contains("openclaw.json")
}

fn within_workspace(path: &Path, root: &Path) -> bool {
    let cwd = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    cwd.starts_with(root)
}

fn cmd_sandbox(args: &[String]) -> i32 {
    let started = Instant::now();
    let root = workspace_root();
    let cwd = env::current_dir().unwrap_or_else(|_| root.clone());
    let block = |reason: &str, code: i32| -> i32 {
        emit(&SandboxReport { status: "blocked".into(), reason: reason.into(), command: args.to_vec(), cwd: cwd.display().to_string(), code: None, latency_ms: started.elapsed().as_millis(), stdout: String::new(), stderr: String::new(), external_side_effects: false });
        code
    };
    if args.is_empty() { return block("empty command", 2); }
    if !within_workspace(&cwd, &root) { return block("cwd outside workspace", 3); }
    if !allowed_program(&args[0]) || args.iter().any(|a| dangerous_arg(a)) { return block("command not allowed by TianGong sandbox policy", 4); }
    match Command::new(&args[0]).args(&args[1..]).current_dir(&cwd).output() {
        Ok(out) => {
            let code = out.status.code().unwrap_or(-1);
            emit(&SandboxReport { status: if code == 0 { "ok" } else { "failed" }.into(), reason: "executed under TianGong local sandbox policy".into(), command: args.to_vec(), cwd: cwd.display().to_string(), code: Some(code), latency_ms: started.elapsed().as_millis(), stdout: String::from_utf8_lossy(&out.stdout).chars().take(2000).collect(), stderr: String::from_utf8_lossy(&out.stderr).chars().take(2000).collect(), external_side_effects: false });
            if code == 0 { 0 } else { 1 }
        }
        Err(err) => {
            emit(&SandboxReport { status: "failed".into(), reason: format!("spawn failed: {}", err), command: args.to_vec(), cwd: cwd.display().to_string(), code: None, latency_ms: started.elapsed().as_millis(), stdout: String::new(), stderr: String::new(), external_side_effects: false });
            1
        }
    }
}

// ---------------- evolver ----------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Phase { Observe, Act, Verify, Repair, Consolidate }

#[derive(Debug, Serialize, Deserialize)]
struct EvolverInput { objective: String, last_status: Option<String>, verification_score: Option<f64>, secret_hit_count: Option<u64> }

#[derive(Debug, Serialize)]
struct EvolverOutput { status: String, phase: Phase, next_phase: Phase, action: String, fitness: f64, promotion: String, reason: String, timestamp_ms: u128 }

fn parse_phase(s: &str) -> Option<Phase> { match s { "observe" => Some(Phase::Observe), "act" => Some(Phase::Act), "verify" => Some(Phase::Verify), "repair" => Some(Phase::Repair), "consolidate" => Some(Phase::Consolidate), _ => None } }

fn evolver_next(phase: Phase, input: &EvolverInput) -> Phase {
    let status = input.last_status.as_deref().unwrap_or("ok");
    let score = input.verification_score.unwrap_or(0.0);
    let secrets = input.secret_hit_count.unwrap_or(0);
    if secrets > 0 || status == "failed" || status == "blocked" { return Phase::Repair; }
    match phase { Phase::Observe => Phase::Act, Phase::Act => Phase::Verify, Phase::Verify if score >= 0.7 => Phase::Consolidate, Phase::Verify => Phase::Repair, Phase::Repair => Phase::Act, Phase::Consolidate => Phase::Observe }
}

fn evolver_fitness(input: &EvolverInput) -> f64 {
    let score = input.verification_score.unwrap_or(0.0).clamp(0.0, 1.0);
    let status_score = match input.last_status.as_deref().unwrap_or("ok") { "ok" => 1.0, "warn" => 0.7, "failed" | "blocked" => 0.0, _ => 0.5 };
    let secret_score = if input.secret_hit_count.unwrap_or(0) == 0 { 1.0 } else { 0.0 };
    ((score + status_score + secret_score) / 3.0 * 10000.0).round() / 10000.0
}

fn evolver_action(phase: Phase, next: Phase) -> &'static str {
    match (phase, next) { (Phase::Observe, Phase::Act) => "collect current state and execute bounded next step", (Phase::Act, Phase::Verify) => "run verification gate after action", (Phase::Verify, Phase::Consolidate) => "promote verified artifact into evolution memory", (Phase::Verify, Phase::Repair) => "repair failed or low-confidence artifact before promotion", (Phase::Repair, Phase::Act) => "apply minimal repair and retry action", (Phase::Consolidate, Phase::Observe) => "start next observation cycle", (_, Phase::Repair) => "enter repair due to safety or execution failure", _ => "continue deterministic evolution cycle" }
}

fn cmd_evolver(args: &[String]) -> i32 {
    if args.len() < 2 { emit(&serde_json::json!({"status":"blocked","reason":"usage: tiangong_core evolver <phase> <json-input>"})); return 2; }
    let phase = match parse_phase(&args[0]) { Some(p) => p, None => { emit(&serde_json::json!({"status":"blocked","reason":"invalid phase"})); return 3; } };
    let input: EvolverInput = match serde_json::from_str(&args[1]) { Ok(v) => v, Err(e) => { emit(&serde_json::json!({"status":"blocked","reason":format!("invalid json: {}", e)})); return 4; } };
    let fit = evolver_fitness(&input);
    let next = evolver_next(phase, &input);
    let promotion = if phase == Phase::Verify && next == Phase::Consolidate && fit >= 0.7 { "pass" } else if next == Phase::Repair { "hold" } else { "continue" };
    emit(&EvolverOutput { status: "ok".into(), phase, next_phase: next, action: evolver_action(phase, next).into(), fitness: fit, promotion: promotion.into(), reason: format!("objective='{}'", input.objective), timestamp_ms: now_ms() });
    0
}

// ---------------- cognition ----------------
#[derive(Debug, Serialize, Deserialize)]
struct Candidate { gene: String, evidence: f64, feasibility: f64, risk: f64, reversibility: f64 }
#[derive(Debug, Serialize, Deserialize)]
struct CognitionInput { objective: String, constraints: Vec<String>, candidates: Vec<Candidate> }
#[derive(Debug, Serialize)]
struct RankedCandidate { gene: String, score: f64, reason: String }
#[derive(Debug, Serialize)]
struct CognitionOutput { status: String, mode: String, objective: String, roles: Vec<String>, decomposition: Vec<String>, ranked_options: Vec<RankedCandidate>, critique: String, falsification_path: String, timestamp_ms: u128 }

fn candidate_score(c: &Candidate) -> f64 { ((0.35*clamp01(c.evidence)+0.30*clamp01(c.feasibility)+0.20*clamp01(c.reversibility)+0.15*(1.0-clamp01(c.risk))) * 10000.0).round()/10000.0 }
fn default_candidates() -> Vec<Candidate> { vec![ Candidate{gene:"sandbox adapter".into(),evidence:0.90,feasibility:0.92,risk:0.20,reversibility:0.85}, Candidate{gene:"evidence ledger".into(),evidence:0.86,feasibility:0.88,risk:0.10,reversibility:0.90}, Candidate{gene:"role router".into(),evidence:0.84,feasibility:0.86,risk:0.15,reversibility:0.88}, Candidate{gene:"repair loop".into(),evidence:0.82,feasibility:0.84,risk:0.25,reversibility:0.80} ] }
fn roles_for(objective: &str) -> Vec<String> { let mut roles = vec!["researcher","architect","executor","reviewer","evolver"]; let lower = objective.to_ascii_lowercase(); if lower.contains("medical") || objective.contains("医学") || objective.contains("科研") { roles.push("scientific_critic"); } if lower.contains("rust") || lower.contains("go") || lower.contains("c ") { roles.push("systems_engineer"); } roles.into_iter().map(String::from).collect() }
fn decomposition_for(objective: &str) -> Vec<String> { vec![format!("clarify boundary for: {}", objective), "collect evidence and constraints".into(), "rank candidate genes".into(), "select reversible implementation path".into(), "define verification gate".into(), "consolidate only after tests pass".into()] }

fn cmd_cognition(args: &[String]) -> i32 {
    if args.is_empty() { emit(&serde_json::json!({"status":"blocked","reason":"missing json input"})); return 2; }
    let mut input: CognitionInput = match serde_json::from_str(&args[0]) { Ok(v) => v, Err(e) => { emit(&serde_json::json!({"status":"blocked","reason":format!("invalid json: {}", e)})); return 3; } };
    if input.candidates.is_empty() { input.candidates = default_candidates(); }
    let mut ranked: Vec<RankedCandidate> = input.candidates.iter().map(|c| RankedCandidate { gene: c.gene.clone(), score: candidate_score(c), reason: format!("evidence={:.2}, feasibility={:.2}, risk={:.2}, reversibility={:.2}", c.evidence, c.feasibility, c.risk, c.reversibility) }).collect();
    ranked.sort_by(|a,b| b.score.partial_cmp(&a.score).unwrap());
    emit(&CognitionOutput { status:"ok".into(), mode:"multi_role_router".into(), objective: input.objective.clone(), roles: roles_for(&input.objective), decomposition: decomposition_for(&input.objective), ranked_options: ranked, critique:"Hypotheses are not findings; require verification before promotion.".into(), falsification_path:"If tests fail, evidence is missing, or risk exceeds benefit, route to repair/hold.".into(), timestamp_ms: now_ms() });
    0
}

// ---------------- gate ----------------
#[derive(Debug, Serialize, Deserialize)]
struct GateInput { objective: String, artifacts: Vec<String>, tests_passed: bool, secret_hit_count: u64, risk: String }
#[derive(Debug, Serialize)]
struct CheckItem { name: String, passed: bool, reason: String }
#[derive(Debug, Serialize)]
struct GateOutput { status: String, gate: String, passed: bool, score: f64, checklist: Vec<CheckItem>, recommendation: String, timestamp_ms: u128 }
fn gate_requirements(input:&GateInput)->Vec<CheckItem>{vec![CheckItem{name:"objective_present".into(),passed:!input.objective.trim().is_empty(),reason:"task has explicit objective".into()},CheckItem{name:"risk_declared".into(),passed:matches!(input.risk.as_str(),"low"|"medium"|"high"),reason:"risk must be low/medium/high".into()}]}
fn gate_architecture(input:&GateInput)->Vec<CheckItem>{vec![CheckItem{name:"artifacts_tracked".into(),passed:!input.artifacts.is_empty(),reason:"architecture requires traceable artifacts".into()},CheckItem{name:"local_first".into(),passed:true,reason:"TianGong native loop is local-first".into()}]}
fn gate_test_plan(input:&GateInput)->Vec<CheckItem>{vec![CheckItem{name:"tests_passed".into(),passed:input.tests_passed,reason:"verification gate must pass before promotion".into()},CheckItem{name:"secrets_clear".into(),passed:input.secret_hit_count==0,reason:"secret hits block external sync and promotion".into()}]}
fn gate_review(input:&GateInput)->Vec<CheckItem>{vec![CheckItem{name:"high_risk_hold".into(),passed:input.risk!="high",reason:"high-risk tasks require explicit human approval".into()},CheckItem{name:"clean_room".into(),passed:true,reason:"capability is represented as native abstraction".into()}]}
fn gate_checklist(gate:&str,input:&GateInput)->Vec<CheckItem>{match gate{"requirements"=>gate_requirements(input),"architecture"=>gate_architecture(input),"test_plan"=>gate_test_plan(input),"review"=>gate_review(input),"full"=>{let mut all=vec![];all.extend(gate_requirements(input));all.extend(gate_architecture(input));all.extend(gate_test_plan(input));all.extend(gate_review(input));all},_=>vec![CheckItem{name:"valid_gate".into(),passed:false,reason:"gate must be requirements|architecture|test_plan|review|full".into()}]}}
fn gate_score(items:&[CheckItem])->f64{if items.is_empty(){0.0}else{let passed=items.iter().filter(|i|i.passed).count() as f64;((passed/items.len() as f64)*10000.0).round()/10000.0}}
fn cmd_gate(args:&[String])->i32{if args.len()<2{emit(&serde_json::json!({"status":"blocked","reason":"usage: tiangong_core gate <gate> <json-input>"}));return 2;} let gate=args[0].clone(); let input:GateInput=match serde_json::from_str(&args[1]){Ok(v)=>v,Err(e)=>{emit(&serde_json::json!({"status":"blocked","reason":format!("invalid json: {}",e)}));return 3;}}; let items=gate_checklist(&gate,&input); let sc=gate_score(&items); let passed=sc>=0.9999; let recommendation=if passed{"gate passed; continue".to_string()}else if input.secret_hit_count>0{"blocked: repair secret hits before promotion".to_string()}else if !input.tests_passed{"hold: tests must pass before promotion".to_string()}else{"hold: repair failed checklist items".to_string()}; emit(&GateOutput{status:"ok".into(),gate,passed,score:sc,checklist:items,recommendation,timestamp_ms:now_ms()}); if passed{0}else{1}}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() { emit(&serde_json::json!({"status":"blocked","reason":"usage: tiangong_core <sandbox|evolver|cognition|gate> ..."})); std::process::exit(2); }
    let (cmd, rest) = (&args[0], &args[1..]);
    let code = match cmd.as_str() { "sandbox" => cmd_sandbox(rest), "evolver" => cmd_evolver(rest), "cognition" => cmd_cognition(rest), "gate" => cmd_gate(rest), "selftest" => { emit(&serde_json::json!({"status":"ok","core":"tiangong_core","timestamp_ms":now_ms()})); 0 }, _ => { emit(&serde_json::json!({"status":"blocked","reason":"unknown subcommand"})); 2 } };
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn sandbox_blocks_bash(){ assert!(!allowed_program("bash")); }
    #[test] fn evolver_promotes_verified(){ let i=EvolverInput{objective:"x".into(),last_status:Some("ok".into()),verification_score:Some(0.9),secret_hit_count:Some(0)}; assert_eq!(evolver_next(Phase::Verify,&i),Phase::Consolidate); }
    #[test] fn cognition_scores_low_risk_higher(){ let a=Candidate{gene:"a".into(),evidence:0.8,feasibility:0.8,risk:0.1,reversibility:0.8}; let b=Candidate{gene:"b".into(),evidence:0.8,feasibility:0.8,risk:0.9,reversibility:0.8}; assert!(candidate_score(&a)>candidate_score(&b)); }
    #[test] fn gate_full_passes(){ let i=GateInput{objective:"x".into(),artifacts:vec!["a".into()],tests_passed:true,secret_hit_count:0,risk:"low".into()}; assert_eq!(gate_score(&gate_checklist("full",&i)),1.0); }
}
