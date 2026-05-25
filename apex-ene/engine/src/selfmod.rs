/// Evol_code - 原生代码演化引擎
///
/// Self-modification engine that can:
/// 1. Analyze its own codebase
/// 2. Generate patches based on evolution directives
/// 3. Apply patches safely
/// 4. Rollback on failure
/// 5. Track modification history

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePatch {
    pub id: String,
    pub target_file: String,
    pub patch_type: PatchType,
    pub content_before: String,
    pub content_after: String,
    pub status: PatchStatus,
    pub timestamp: String,
    pub directive_source: String,
    pub verification_result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatchType {
    BugFix,
    Optimization,
    Feature,
    Refactor,
    RedundancyRemoval,
    ArchitectureChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatchStatus {
    Proposed,
    Applied,
    Verified,
    RolledBack,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModEngine {
    pub workspace: PathBuf,
    pub patches: Vec<CodePatch>,
    pub total_applied: u64,
    pub total_failed: u64,
    pub total_rolled_back: u64,
    pub success_rate: f64,
}

impl SelfModEngine {
    pub fn new(workspace: PathBuf) -> Self {
        Self {
            workspace,
            patches: Vec::new(),
            total_applied: 0,
            total_failed: 0,
            total_rolled_back: 0,
            success_rate: 0.0,
        }
    }

    /// Analyze codebase for potential improvements
    pub fn analyze_codebase(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let engine_dir = self.workspace.join("apex-ene");

        // Check for hardcoded paths
        if let Ok(entries) = fs::read_dir(&engine_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "rs" || e == "go").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        // Check for common issues
                        if content.contains("hardcoded") || content.contains("TODO") {
                            issues.push(format!("{}: contains TODOs or hardcoded values", 
                                path.file_name().unwrap_or_default().to_string_lossy()));
                        }
                    }
                }
            }
        }

        if issues.is_empty() {
            issues.push("Codebase clean - no immediate issues found".to_string());
        }
        issues
    }

    /// Generate a patch for a specific file
    pub fn generate_patch(
        &mut self,
        target_file: &str,
        patch_type: PatchType,
        content_before: &str,
        content_after: &str,
        directive: &str,
    ) -> CodePatch {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}", target_file, chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)).as_bytes());
        let id = format!("patch-{:x}", hasher.finalize())[..16].to_string();

        let patch = CodePatch {
            id,
            target_file: target_file.to_string(),
            patch_type,
            content_before: content_before.to_string(),
            content_after: content_after.to_string(),
            status: PatchStatus::Proposed,
            timestamp: chrono::Utc::now().to_rfc3339(),
            directive_source: directive.to_string(),
            verification_result: None,
        };

        self.patches.push(patch.clone());
        patch
    }

    /// Apply a patch to the filesystem
    pub fn apply_patch(&mut self, patch_id: &str) -> Result<String, String> {
        let patch_idx = self.patches.iter().position(|p| p.id == patch_id)
            .ok_or_else(|| format!("Patch {} not found", patch_id))?;

        let patch = &self.patches[patch_idx];
        let file_path = self.workspace.join(&patch.target_file);

        // Verify the file content matches `content_before` (safety check)
        let current = fs::read_to_string(&file_path)
            .map_err(|e| format!("Cannot read {}: {}", file_path.display(), e))?;

        if current != patch.content_before {
            return Err(format!(
                "File {} has changed since patch was generated. Cannot apply safely.",
                file_path.display()
            ));
        }

        // Write the new content
        fs::write(&file_path, &patch.content_after)
            .map_err(|e| format!("Cannot write {}: {}", file_path.display(), e))?;

        // Update patch status
        let patch = &mut self.patches[patch_idx];
        patch.status = PatchStatus::Applied;
        self.total_applied += 1;

        Ok(format!("✅ Patch {} applied to {}", patch_id, file_path.display()))
    }

    /// Verify a patch by compiling the project
    pub fn verify_patch(&mut self, patch_id: &str) -> Result<String, String> {
        let patch_idx = self.patches.iter().position(|p| p.id == patch_id)
            .ok_or_else(|| format!("Patch {} not found", patch_id))?;

        // Try to compile the engine
        let engine_dir = self.workspace.join("apex-ene").join("engine");
        let output = Command::new("cargo")
            .args(["check", "--manifest-path", &engine_dir.join("Cargo.toml").to_string_lossy()])
            .output()
            .map_err(|e| format!("Cannot run cargo check: {}", e))?;

        let result = if output.status.success() {
            self.patches[patch_idx].status = PatchStatus::Verified;
            self.patches[patch_idx].verification_result = Some("Compilation OK".to_string());
            format!("✅ Patch {} verified - compilation passed", patch_id)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            self.patches[patch_idx].status = PatchStatus::Failed;
            self.patches[patch_idx].verification_result = Some(format!("Compilation failed: {}", &stderr[..200]));
            self.total_failed += 1;
            format!("❌ Patch {} failed verification:\n{}", patch_id, stderr)
        };

        // Save state
        self.save_state();
        Ok(result)
    }

    /// Rollback a patch
    pub fn rollback_patch(&mut self, patch_id: &str) -> Result<String, String> {
        let patch_idx = self.patches.iter().position(|p| p.id == patch_id)
            .ok_or_else(|| format!("Patch {} not found", patch_id))?;

        let patch = &self.patches[patch_idx];
        let file_path = self.workspace.join(&patch.target_file);

        fs::write(&file_path, &patch.content_before)
            .map_err(|e| format!("Cannot rollback {}: {}", file_path.display(), e))?;

        self.patches[patch_idx].status = PatchStatus::RolledBack;
        self.total_rolled_back += 1;

        Ok(format!("↩️ Patch {} rolled back", patch_id))
    }

    /// Self-diagnosis: identify code-level issues
    pub fn code_diagnosis(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let rate = if (self.total_applied + self.total_failed) > 0 {
            self.total_applied as f64 / (self.total_applied + self.total_failed) as f64 * 100.0
        } else {
            0.0
        };

        issues.push(format!("Patch success rate: {:.1}%", rate));
        issues.push(format!("Total patches applied: {}", self.total_applied));
        issues.push(format!("Total rolled back: {}", self.total_rolled_back));

        if self.total_failed > self.total_applied {
            issues.push("⚠️ More failures than successes - consider slowing evolution cycle".to_string());
        }
        if self.total_rolled_back > 5 {
            issues.push("⚠️ High rollback rate - patches need better pre-verification".to_string());
        }

        issues
    }

    fn save_state(&self) {
        // Future: persist patch history
    }
}
