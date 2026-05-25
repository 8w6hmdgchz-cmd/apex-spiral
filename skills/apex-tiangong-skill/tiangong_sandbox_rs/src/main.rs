use serde::Serialize;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[derive(Serialize)]
struct AuditReport {
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

fn workspace_root() -> PathBuf {
    env::var("APEX_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/Users/lihongxin/.openclaw/workspace"))
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

fn emit(report: AuditReport) {
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}

fn main() {
    let started = Instant::now();
    let args: Vec<String> = env::args().skip(1).collect();
    let root = workspace_root();
    let cwd = env::current_dir().unwrap_or_else(|_| root.clone());

    if args.is_empty() {
        emit(AuditReport {
            status: "blocked".into(),
            reason: "empty command".into(),
            command: args,
            cwd: cwd.display().to_string(),
            code: None,
            latency_ms: started.elapsed().as_millis(),
            stdout: String::new(),
            stderr: String::new(),
            external_side_effects: false,
        });
        std::process::exit(2);
    }

    if !within_workspace(&cwd, &root) {
        emit(AuditReport {
            status: "blocked".into(),
            reason: "cwd outside workspace".into(),
            command: args,
            cwd: cwd.display().to_string(),
            code: None,
            latency_ms: started.elapsed().as_millis(),
            stdout: String::new(),
            stderr: String::new(),
            external_side_effects: false,
        });
        std::process::exit(3);
    }

    let program = &args[0];
    if !allowed_program(program) || args.iter().any(|a| dangerous_arg(a)) {
        emit(AuditReport {
            status: "blocked".into(),
            reason: "command not allowed by TianGong sandbox policy".into(),
            command: args,
            cwd: cwd.display().to_string(),
            code: None,
            latency_ms: started.elapsed().as_millis(),
            stdout: String::new(),
            stderr: String::new(),
            external_side_effects: false,
        });
        std::process::exit(4);
    }

    let output = Command::new(program).args(&args[1..]).current_dir(&cwd).output();
    match output {
        Ok(out) => {
            let code = out.status.code().unwrap_or(-1);
            let status = if code == 0 { "ok" } else { "failed" };
            emit(AuditReport {
                status: status.into(),
                reason: "executed under TianGong local sandbox policy".into(),
                command: args,
                cwd: cwd.display().to_string(),
                code: Some(code),
                latency_ms: started.elapsed().as_millis(),
                stdout: String::from_utf8_lossy(&out.stdout).chars().take(2000).collect(),
                stderr: String::from_utf8_lossy(&out.stderr).chars().take(2000).collect(),
                external_side_effects: false,
            });
            std::process::exit(if code == 0 { 0 } else { 1 });
        }
        Err(err) => {
            emit(AuditReport {
                status: "failed".into(),
                reason: format!("spawn failed: {}", err),
                command: args,
                cwd: cwd.display().to_string(),
                code: None,
                latency_ms: started.elapsed().as_millis(),
                stdout: String::new(),
                stderr: String::new(),
                external_side_effects: false,
            });
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_expected_programs() {
        assert!(allowed_program("python3"));
        assert!(allowed_program("cargo"));
        assert!(!allowed_program("bash"));
    }

    #[test]
    fn block_dangerous_args() {
        assert!(dangerous_arg("rm"));
        assert!(dangerous_arg("sudo"));
        assert!(dangerous_arg("~/.ssh/id_ed25519"));
        assert!(!dangerous_arg("--version"));
    }
}
