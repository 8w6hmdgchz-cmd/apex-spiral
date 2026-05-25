/// λΦ GitHub 资源猎食器
///
/// Uses SSH to clone/fetch GitHub repos (bypassing HTTPS firewall).
/// Discovers trending repos, forks with high activity, and fresh commits.

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::{Path, PathBuf};

/// A discovered GitHub resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubResource {
    pub repo: String,
    pub ssh_url: String,
    pub description: String,
    pub stars_estimate: u32,
    pub topics: Vec<String>,
    pub language: String,
    pub discovered_at: String,
    pub last_commit: Option<String>,
    pub absorption_status: AbsorptionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbsorptionStatus {
    Pending,
    Cloned,
    Analyzed,
    Absorbed,
    Failed(String),
}

/// Core GitHub scavenger
pub struct GitHubScavenger {
    pub cache_dir: PathBuf,
    pub resources: Vec<GitHubResource>,
}

impl GitHubScavenger {
    pub fn new(cache_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&cache_dir);
        Self {
            cache_dir,
            resources: Vec::new(),
        }
    }

    /// Scavenge a GitHub repo via SSH
    pub fn scavenge_repo(&mut self, org: &str, repo: &str) -> Result<GitHubResource, String> {
        let ssh_url = format!("git@github.com:{}/{}.git", org, repo);

        // First, check reachability via SSH
        let reachable = Command::new("git")
            .args(["ls-remote", &ssh_url, "HEAD"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !reachable {
            return Err(format!("{} 不可达 (SSH)", ssh_url));
        }

        // Get the latest commit hash
        let commit = Command::new("git")
            .args(["ls-remote", &ssh_url, "HEAD"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok()
                        .and_then(|s| s.split_whitespace().next().map(String::from))
                } else {
                    None
                }
            });

        // Try to infer language from repo name and known mappings
        let language = self.infer_language(repo);
        let topics = self.infer_topics(repo);

        let resource = GitHubResource {
            repo: format!("{}/{}", org, repo),
            ssh_url,
            description: format!("{} - 自动猎食", repo),
            stars_estimate: 0, // Can't get stars via SSH
            topics,
            language,
            discovered_at: chrono::Utc::now().to_rfc3339(),
            last_commit: commit,
            absorption_status: AbsorptionStatus::Pending,
        };

        self.resources.push(resource.clone());
        Ok(resource)
    }

    /// Clone a repo for deeper analysis
    pub fn clone_for_analysis(&mut self, idx: usize) -> Result<(), String> {
        if idx >= self.resources.len() {
            return Err("Index out of range".to_string());
        }

        let resource = &self.resources[idx];
        let repo_dir = self.cache_dir.join(
            resource.repo.replace('/', "__")
        );

        if repo_dir.exists() {
            // Already cloned, just pull
            let output = Command::new("git")
                .args(["-C", &repo_dir.to_string_lossy(), "pull", "--ff-only"])
                .output()
                .map_err(|e| format!("Git pull failed: {}", e))?;

            if !output.status.success() {
                return Err(format!("Pull failed: {}", String::from_utf8_lossy(&output.stderr)));
            }
        } else {
            // Fresh clone via SSH
            let output = Command::new("git")
                .args(["clone", &resource.ssh_url, &repo_dir.to_string_lossy()])
                .output()
                .map_err(|e| format!("Git clone failed: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                self.resources[idx].absorption_status = 
                    AbsorptionStatus::Failed(format!("Clone failed: {}", stderr));
                return Err(format!("Clone failed: {}", stderr));
            }
        }

        self.resources[idx].absorption_status = AbsorptionStatus::Cloned;
        Ok(())
    }

    /// Analyze clond repo for absorbable content
    pub fn analyze_clone(&self, idx: usize) -> Result<Vec<String>, String> {
        if idx >= self.resources.len() {
            return Err("Index out of range".to_string());
        }

        let resource = &self.resources[idx];
        let repo_dir = self.cache_dir.join(
            resource.repo.replace('/', "__")
        );

        if !repo_dir.exists() {
            return Err("Repo not cloned yet".to_string());
        }

        let mut findings = Vec::new();

        // Check for SKILL.md / README
        for filename in &["SKILL.md", "README.md", "README", "README.rst"] {
            let path = repo_dir.join(filename);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    findings.push(format!("Found {} ({} chars)", filename, content.len()));
                }
            }
        }

        // Check for Rust/Cargo.toml
        let cargo_path = repo_dir.join("Cargo.toml");
        if cargo_path.exists() {
            findings.push("Rust project detected (Cargo.toml)".to_string());
        }

        // Check for Go modules
        let go_path = repo_dir.join("go.mod");
        if go_path.exists() {
            findings.push("Go project detected (go.mod)".to_string());
        }

        // Check for Python
        let py_files: Vec<_> = std::fs::read_dir(&repo_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "py").unwrap_or(false))
            .collect();
        if !py_files.is_empty() {
            findings.push(format!("Python project ({} .py files)", py_files.len()));
        }

        // List top-level structure
        if let Ok(entries) = std::fs::read_dir(&repo_dir) {
            let names: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .take(10)
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            if !names.is_empty() {
                findings.push(format!("Subdirs: {}", names.join(", ")));
            }
        }

        Ok(findings)
    }

    /// Mark a resource as absorbed
    pub fn mark_absorbed(&mut self, idx: usize) {
        if idx < self.resources.len() {
            self.resources[idx].absorption_status = AbsorptionStatus::Absorbed;
        }
    }

    /// Get pending resources
    pub fn pending(&self) -> Vec<&GitHubResource> {
        self.resources.iter()
            .filter(|r| matches!(r.absorption_status, AbsorptionStatus::Pending))
            .collect()
    }

    /// Get absorbed resources
    pub fn absorbed(&self) -> Vec<&GitHubResource> {
        self.resources.iter()
            .filter(|r| matches!(r.absorption_status, AbsorptionStatus::Absorbed))
            .collect()
    }

    // ========================================
    // 内置猎食清单
    // ========================================

    /// Scavenge the built-in priority list
    pub fn scavenge_priority_list(&mut self) -> Vec<Result<GitHubResource, String>> {
        let targets = vec![
            ("google-a2a", "a2a"),
            ("openai", "openai-agents-python"),
            ("langchain-ai", "langgraph"),
            ("mem0ai", "mem0"),
            ("microsoft", "autogen"),
            ("crewAIInc", "crewAI"),
            ("getzep", "zep"),
            ("ComposioHQ", "composio"),
        ];

        targets.into_iter()
            .map(|(org, repo)| self.scavenge_repo(org, repo))
            .collect()
    }

    /// Scavenge trending topics from the cached resource list
    pub fn scavenge_trending(&mut self) -> Vec<Result<GitHubResource, String>> {
        let topics = vec![
            ("THUDM", "AgentBench"),
            ("noahshinn", "reflexion"),
            ("langchain-ai", "langmem"),
            ("guardrails-ai", "guardrails"),
            ("truera", "trulens"),
            ("Arize-ai", "phoenix"),
            ("e2b-dev", "e2b"),
            ("letta-ai", "letta"),
        ];

        topics.into_iter()
            .map(|(org, repo)| self.scavenge_repo(org, repo))
            .collect()
    }

    /// Save scavenger state
    pub fn save_state(&self, path: &Path) -> Result<(), String> {
        let content = serde_json::to_string_pretty(&self.resources)
            .map_err(|e| format!("Serialize error: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| format!("Write error: {}", e))
    }

    /// Load scavenger state
    pub fn load_state(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Read error: {}", e))?;
        let resources: Vec<GitHubResource> = serde_json::from_str(&content)
            .map_err(|e| format!("Deserialize error: {}", e))?;
        Ok(Self {
            cache_dir: PathBuf::from("."),
            resources,
        })
    }

    // ========================================
    // Heuristics
    // ========================================

    fn infer_language(&self, repo: &str) -> String {
        let r = repo.to_lowercase();
        if r.contains("agent") || r.contains("lang") || r.contains("py") {
            "Python".to_string()
        } else if r.contains("rust") || r.contains("rs") {
            "Rust".to_string()
        } else if r.contains("go") || r.contains("golang") {
            "Go".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    fn infer_topics(&self, repo: &str) -> Vec<String> {
        let r = repo.to_lowercase();
        let mut topics = Vec::new();
        if r.contains("agent") { topics.push("agent".to_string()); }
        if r.contains("memory") || r.contains("mem") { topics.push("memory".to_string()); }
        if r.contains("lang") || r.contains("llm") { topics.push("llm".to_string()); }
        if r.contains("eval") || r.contains("bench") { topics.push("benchmark".to_string()); }
        if r.contains("guard") || r.contains("safe") { topics.push("safety".to_string()); }
        if topics.is_empty() { topics.push("general".to_string()); }
        topics
    }
}
