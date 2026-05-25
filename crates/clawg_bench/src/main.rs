use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct TaskSpec {
    id: String,
    persona_intent: String,
    skill_grounding: String,
    workspace: PathBuf,
    required_file: String,
    required_contains: String,
}

fn ts() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

fn create_task(root: &Path) -> std::io::Result<TaskSpec> {
    let id = format!("clawg_{}", ts());
    let ws = root.join("workspaces").join(&id);
    fs::create_dir_all(&ws)?;
    fs::write(ws.join("notes.txt"), "Project: mock local file operation training\nStatus: draft\n")?;
    let spec = TaskSpec {
        id: id.clone(),
        persona_intent: "Organize a mock project note and create a summary file".into(),
        skill_grounding: "file read/write; concise summary; no destructive delete".into(),
        workspace: ws.clone(),
        required_file: "summary.md".into(),
        required_contains: "mock local file operation training".into(),
    };
    fs::create_dir_all(root.join("tasks"))?;
    fs::write(root.join("tasks").join(format!("{}.json", id)), format!(
        "{{\n  \"id\": \"{}\",\n  \"persona_intent\": \"{}\",\n  \"skill_grounding\": \"{}\",\n  \"workspace\": \"{}\",\n  \"required_file\": \"{}\",\n  \"required_contains\": \"{}\"\n}}\n",
        spec.id, spec.persona_intent, spec.skill_grounding, spec.workspace.display(), spec.required_file, spec.required_contains
    ))?;
    Ok(spec)
}

fn simulate_agent(spec: &TaskSpec) -> std::io::Result<()> {
    let src = fs::read_to_string(spec.workspace.join("notes.txt"))?;
    fs::write(spec.workspace.join(&spec.required_file), format!("# Summary\n\n{}\n", src.trim()))?;
    Ok(())
}

fn auto_verify(spec: &TaskSpec) -> f64 {
    let p = spec.workspace.join(&spec.required_file);
    match fs::read_to_string(p) {
        Ok(text) if text.contains(&spec.required_contains) => 1.0,
        Ok(_) => 0.5,
        Err(_) => 0.0,
    }
}

fn apex_score(auto: f64, llm_human: f64) -> f64 {
    0.60 * auto + 0.40 * llm_human
}

fn write_result(root: &Path, spec: &TaskSpec, auto: f64, llm_human: f64) -> std::io::Result<()> {
    fs::create_dir_all(root.join("results"))?;
    let score = apex_score(auto, llm_human);
    fs::write(root.join("results").join(format!("{}.json", spec.id)), format!(
        "{{\n  \"id\": \"{}\",\n  \"auto_verify\": {:.3},\n  \"llm_human_verify\": {:.3},\n  \"score_apex\": {:.3},\n  \"formula\": \"Score_APEX = AutoVerify(60%) + LLM_HumanVerify(40%)\"\n}}\n",
        spec.id, auto, llm_human, score
    ))
}

fn root_from_args() -> PathBuf {
    let args: Vec<String> = env::args().collect();
    args.windows(2)
        .find(|w| w[0] == "--root")
        .map(|w| PathBuf::from(&w[1]))
        .unwrap_or_else(|| PathBuf::from("bench/clawg"))
}

fn main() -> std::io::Result<()> {
    let root = root_from_args();
    fs::create_dir_all(&root)?;
    let spec = create_task(&root)?;
    simulate_agent(&spec)?;
    let auto = auto_verify(&spec);
    let llm_human = 0.90; // placeholder for assisted human/LLM rubric; explicit, not fake training.
    write_result(&root, &spec, auto, llm_human)?;
    println!("task_id={}", spec.id);
    println!("workspace={}", spec.workspace.display());
    println!("auto_verify={:.3}", auto);
    println!("score_apex={:.3}", apex_score(auto, llm_human));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn score_weights() {
        assert!((apex_score(1.0, 0.5) - 0.8).abs() < 1e-9);
    }
    #[test]
    fn verify_missing_zero() {
        let spec = TaskSpec { id:"x".into(), persona_intent:"".into(), skill_grounding:"".into(), workspace: PathBuf::from("/tmp/definitely_missing_clawg"), required_file:"nope".into(), required_contains:"x".into() };
        assert_eq!(auto_verify(&spec), 0.0);
    }
}
