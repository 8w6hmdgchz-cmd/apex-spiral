use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct EngineScores {
    coord_fix: f64,
    token_control: f64,
    task_syn: f64,
    train_readiness: f64,
    bench_verify: f64,
    era: f64,
    co_scientist: f64,
    robin: f64,
}

impl EngineScores {
    fn ui_control(&self) -> f64 {
        self.coord_fix * self.token_control
    }
    fn training_loop(&self) -> f64 {
        (self.task_syn + self.train_readiness + self.bench_verify) / 3.0
    }
    fn research_autonomy(&self) -> f64 {
        (self.era + self.co_scientist + self.robin) / 3.0
    }
    fn engine_apex(&self) -> f64 {
        self.ui_control() * self.training_loop() * self.research_autonomy()
    }
}

fn ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn arg_value(args: &[String], key: &str, default: &str) -> String {
    args.windows(2)
        .find(|w| w[0] == key)
        .map(|w| w[1].clone())
        .unwrap_or_else(|| default.to_string())
}

fn write_project(root: &Path, question: &str, scores: &EngineScores) -> std::io::Result<PathBuf> {
    let project_id = format!("apex_research_{}", ts());
    let dir = root.join("projects").join(&project_id);
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("protocol.md"), format!("# Protocol\n\nQuestion: {}\n\nBoundary: evidence-first; hypotheses are not findings until verified.\n", question))?;
    fs::write(dir.join("evidence-ledger.jsonl"), format!("{{\"id\":\"seed\",\"source_type\":\"user_request\",\"claim\":\"{}\",\"confidence\":0.5,\"limitations\":\"initial question only; no external evidence yet\"}}\n", question.replace('"', "'")))?;
    fs::write(dir.join("hypotheses.json"), "[\n  {\"id\":\"h1\",\"text\":\"Initial hypothesis placeholder pending evidence search\",\"status\":\"proposed\"}\n]\n")?;
    fs::write(dir.join("experiment-plan.md"), "# Experiment Plan\n\n1. Build evidence ledger.\n2. Generate/rank hypotheses.\n3. Run sandbox analysis on approved data.\n4. Analyze mechanism and limitations.\n")?;
    fs::write(dir.join("paper-outline.md"), "# Paper Outline\n\n- Abstract\n- Background\n- Methods\n- Results / Proposed Results\n- Discussion\n- Limitations\n")?;
    fs::write(dir.join("run.json"), format!(
        "{{\n  \"project_id\": \"{}\",\n  \"engine_apex\": {:.6},\n  \"ui_control\": {:.6},\n  \"training_loop\": {:.6},\n  \"research_autonomy\": {:.6},\n  \"formula\": \"Engine_APEX=(Coord_Fix*Token_Control)*(Task_Syn+Train+Bench)/3*(ERA+CoScientist+Robin)/3\"\n}}\n",
        project_id, scores.engine_apex(), scores.ui_control(), scores.training_loop(), scores.research_autonomy()
    ))?;
    Ok(dir)
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let root = PathBuf::from(arg_value(&args, "--root", "research/apex"));
    let question = arg_value(
        &args,
        "--question",
        "Build a reproducible APEX research pipeline",
    );
    let scores = EngineScores {
        coord_fix: 1.0,
        token_control: 0.95,
        task_syn: 0.90,
        train_readiness: 0.65,
        bench_verify: 0.96,
        era: 0.60,
        co_scientist: 0.62,
        robin: 0.58,
    };
    let dir = write_project(&root, &question, &scores)?;
    println!("project_dir={}", dir.display());
    println!("ui_control={:.3}", scores.ui_control());
    println!("training_loop={:.3}", scores.training_loop());
    println!("research_autonomy={:.3}", scores.research_autonomy());
    println!("engine_apex={:.6}", scores.engine_apex());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn engine_multiplies_modules() {
        let s = EngineScores {
            coord_fix: 1.0,
            token_control: 1.0,
            task_syn: 1.0,
            train_readiness: 1.0,
            bench_verify: 1.0,
            era: 1.0,
            co_scientist: 1.0,
            robin: 1.0,
        };
        assert!((s.engine_apex() - 1.0).abs() < 1e-9);
    }
    #[test]
    fn partial_scores_less_than_one() {
        let s = EngineScores {
            coord_fix: 1.0,
            token_control: 0.5,
            task_syn: 1.0,
            train_readiness: 1.0,
            bench_verify: 1.0,
            era: 1.0,
            co_scientist: 1.0,
            robin: 1.0,
        };
        assert!((s.engine_apex() - 0.5).abs() < 1e-9);
    }
}
