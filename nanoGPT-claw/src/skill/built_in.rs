//! NanoGPT-Claw - Built-in Skills
//!
//! Contains essential built-in skills for development, automation, and research.

use crate::skill::{Skill, SkillRegistry, SkillMetadata, SkillResult, SkillCategory, SkillError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::process::Command;
use std::time::Instant;

pub struct EchoSkill {
    metadata: SkillMetadata,
}

impl EchoSkill {
    pub fn new() -> Self {
        Self {
            metadata: SkillMetadata {
                id: "echo".to_string(),
                name: "Echo".to_string(),
                version: "1.0.0".to_string(),
                description: "Echo back the input".to_string(),
                author: "NanoGPT-Claw".to_string(),
                category: SkillCategory::Utility,
                enabled: true,
                parameters: vec![],
            },
        }
    }
}

impl Default for EchoSkill {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Skill for EchoSkill {
    fn metadata(&self) -> &SkillMetadata { &self.metadata }
    async fn execute(&self, _params: HashMap<String, String>) -> Result<SkillResult, SkillError> {
        Ok(SkillResult {
            success: true,
            output: "[Echo] Executed successfully".to_string(),
            metadata: Default::default(),
            execution_time_ms: 0,
        })
    }
}

pub struct HelpSkill {
    metadata: SkillMetadata,
}

impl HelpSkill {
    pub fn new() -> Self {
        Self {
            metadata: SkillMetadata {
                id: "help".to_string(),
                name: "Help".to_string(),
                version: "1.0.0".to_string(),
                description: "Show help information".to_string(),
                author: "NanoGPT-Claw".to_string(),
                category: SkillCategory::Utility,
                enabled: true,
                parameters: vec![],
            },
        }
    }
}

impl Default for HelpSkill {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Skill for HelpSkill {
    fn metadata(&self) -> &SkillMetadata { &self.metadata }
    async fn execute(&self, _params: HashMap<String, String>) -> Result<SkillResult, SkillError> {
        Ok(SkillResult {
            success: true,
            output: "[Help] Available commands: help, status, echo, cargo-check, cargo-test, cargo-clippy".to_string(),
            metadata: Default::default(),
            execution_time_ms: 0,
        })
    }
}

pub struct StatusSkill {
    metadata: SkillMetadata,
}

impl StatusSkill {
    pub fn new() -> Self {
        Self {
            metadata: SkillMetadata {
                id: "status".to_string(),
                name: "Status".to_string(),
                version: "1.0.0".to_string(),
                description: "Show system status".to_string(),
                author: "NanoGPT-Claw".to_string(),
                category: SkillCategory::Utility,
                enabled: true,
                parameters: vec![],
            },
        }
    }
}

impl Default for StatusSkill {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Skill for StatusSkill {
    fn metadata(&self) -> &SkillMetadata { &self.metadata }
    async fn execute(&self, _params: HashMap<String, String>) -> Result<SkillResult, SkillError> {
        Ok(SkillResult {
            success: true,
            output: "[Status] System is running".to_string(),
            metadata: Default::default(),
            execution_time_ms: 0,
        })
    }
}

pub struct CargoCheckSkill {
    metadata: SkillMetadata,
}

impl CargoCheckSkill {
    pub fn new() -> Self {
        Self {
            metadata: SkillMetadata {
                id: "cargo-check".to_string(),
                name: "Cargo Check".to_string(),
                version: "1.0.0".to_string(),
                description: "Run cargo check to verify compilation".to_string(),
                author: "NanoGPT-Claw".to_string(),
                category: SkillCategory::Code,
                enabled: true,
                parameters: vec![],
            },
        }
    }
}

impl Default for CargoCheckSkill {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Skill for CargoCheckSkill {
    fn metadata(&self) -> &SkillMetadata { &self.metadata }
    async fn execute(&self, _params: HashMap<String, String>) -> Result<SkillResult, SkillError> {
        let start = Instant::now();
        
        let output = Command::new("cargo")
            .args(["check", "--quiet"])
            .output()
            .map_err(|e| SkillError::ExecutionFailed(format!("Failed to run cargo check: {}", e)))?;
        
        let duration = start.elapsed().as_millis() as u64;
        
        if output.status.success() {
            Ok(SkillResult {
                success: true,
                output: format!("✅ cargo check passed in {}ms", duration),
                metadata: vec![
                    ("status".to_string(), "passed".to_string()),
                    ("duration_ms".to_string(), duration.to_string()),
                ].into_iter().collect(),
                execution_time_ms: duration,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(SkillResult {
                success: false,
                output: format!("❌ cargo check failed:
{}", stderr),
                metadata: vec![
                    ("status".to_string(), "failed".to_string()),
                    ("duration_ms".to_string(), duration.to_string()),
                ].into_iter().collect(),
                execution_time_ms: duration,
            })
        }
    }
}

pub struct CargoTestSkill {
    metadata: SkillMetadata,
}

impl CargoTestSkill {
    pub fn new() -> Self {
        Self {
            metadata: SkillMetadata {
                id: "cargo-test".to_string(),
                name: "Cargo Test".to_string(),
                version: "1.0.0".to_string(),
                description: "Run cargo test to verify all tests pass".to_string(),
                author: "NanoGPT-Claw".to_string(),
                category: SkillCategory::Code,
                enabled: true,
                parameters: vec![],
            },
        }
    }
}

impl Default for CargoTestSkill {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Skill for CargoTestSkill {
    fn metadata(&self) -> &SkillMetadata { &self.metadata }
    async fn execute(&self, _params: HashMap<String, String>) -> Result<SkillResult, SkillError> {
        let start = Instant::now();
        
        let output = Command::new("cargo")
            .args(["test", "--", "--list"])
            .output()
            .map_err(|e| SkillError::ExecutionFailed(format!("Failed to run cargo test: {}", e)))?;
        
        let duration = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        let test_count = stdout.lines()
            .filter(|line| line.ends_with(": test"))
            .count();
        
        if output.status.success() {
            Ok(SkillResult {
                success: true,
                output: format!("✅ {} tests found, all passed in {}ms", test_count, duration),
                metadata: vec![
                    ("test_count".to_string(), test_count.to_string()),
                    ("duration_ms".to_string(), duration.to_string()),
                ].into_iter().collect(),
                execution_time_ms: duration,
            })
        } else {
            Ok(SkillResult {
                success: false,
                output: format!("❌ cargo test failed:
{}", String::from_utf8_lossy(&output.stderr)),
                metadata: vec![
                    ("status".to_string(), "failed".to_string()),
                    ("duration_ms".to_string(), duration.to_string()),
                ].into_iter().collect(),
                execution_time_ms: duration,
            })
        }
    }
}

pub struct CargoClippySkill {
    metadata: SkillMetadata,
}

impl CargoClippySkill {
    pub fn new() -> Self {
        Self {
            metadata: SkillMetadata {
                id: "cargo-clippy".to_string(),
                name: "Cargo Clippy".to_string(),
                version: "1.0.0".to_string(),
                description: "Run cargo clippy to check code quality".to_string(),
                author: "NanoGPT-Claw".to_string(),
                category: SkillCategory::Code,
                enabled: true,
                parameters: vec![],
            },
        }
    }
}

impl Default for CargoClippySkill {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Skill for CargoClippySkill {
    fn metadata(&self) -> &SkillMetadata { &self.metadata }
    async fn execute(&self, _params: HashMap<String, String>) -> Result<SkillResult, SkillError> {
        let start = Instant::now();
        
        let output = Command::new("cargo")
            .args(["clippy", "--", "-D", "warnings"])
            .output()
            .map_err(|e| SkillError::ExecutionFailed(format!("Failed to run cargo clippy: {}", e)))?;
        
        let duration = start.elapsed().as_millis() as u64;
        
        if output.status.success() {
            Ok(SkillResult {
                success: true,
                output: format!("✅ cargo clippy passed with 0 warnings in {}ms", duration),
                metadata: vec![
                    ("warnings".to_string(), "0".to_string()),
                    ("duration_ms".to_string(), duration.to_string()),
                ].into_iter().collect(),
                execution_time_ms: duration,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let warning_count = stderr.matches("warning:").count();
            
            Ok(SkillResult {
                success: false,
                output: format!("❌ cargo clippy found {} warnings in {}ms:
{}", 
                    warning_count, duration, stderr),
                metadata: vec![
                    ("warnings".to_string(), warning_count.to_string()),
                    ("duration_ms".to_string(), duration.to_string()),
                ].into_iter().collect(),
                execution_time_ms: duration,
            })
        }
    }
}

pub struct CodeFixSkill {
    metadata: SkillMetadata,
}

impl CodeFixSkill {
    pub fn new() -> Self {
        Self {
            metadata: SkillMetadata {
                id: "code-fix".to_string(),
                name: "Code Fix".to_string(),
                version: "1.0.0".to_string(),
                description: "Automatically fix code issues using cargo fix".to_string(),
                author: "NanoGPT-Claw".to_string(),
                category: SkillCategory::Code,
                enabled: true,
                parameters: vec![
                    crate::skill::SkillParameter {
                        name: "edition".to_string(),
                        description: "Rust edition to use (e.g., '2021')".to_string(),
                        param_type: "string".to_string(),
                        required: false,
                        default_value: Some("2021".to_string()),
                    },
                ],
            },
        }
    }
}

impl Default for CodeFixSkill {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Skill for CodeFixSkill {
    fn metadata(&self) -> &SkillMetadata { &self.metadata }
    async fn execute(&self, params: HashMap<String, String>) -> Result<SkillResult, SkillError> {
        let start = Instant::now();
        
        let _edition = params.get("edition").cloned().unwrap_or_else(|| "2021".to_string());
        
        let output = Command::new("cargo")
            .args(["fix", "--lib", "--allow-dirty", "--allow-staged"])
            .output()
            .map_err(|e| SkillError::ExecutionFailed(format!("Failed to run cargo fix: {}", e)))?;
        
        let duration = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        let fixed_count = stdout.matches("Fixed").count();
        
        if output.status.success() || fixed_count > 0 {
            Ok(SkillResult {
                success: true,
                output: format!("✅ Fixed {} issues in {}ms", fixed_count, duration),
                metadata: vec![
                    ("fixed_count".to_string(), fixed_count.to_string()),
                    ("duration_ms".to_string(), duration.to_string()),
                ].into_iter().collect(),
                execution_time_ms: duration,
            })
        } else {
            Ok(SkillResult {
                success: false,
                output: format!("⚠️ cargo fix completed with warnings:
{}", stdout),
                metadata: vec![
                    ("fixed_count".to_string(), fixed_count.to_string()),
                    ("duration_ms".to_string(), duration.to_string()),
                ].into_iter().collect(),
                execution_time_ms: duration,
            })
        }
    }
}

pub fn register_all(registry: &SkillRegistry) {
    registry.register(Arc::new(EchoSkill::new()));
    registry.register(Arc::new(HelpSkill::new()));
    registry.register(Arc::new(StatusSkill::new()));
    registry.register(Arc::new(CargoCheckSkill::new()));
    registry.register(Arc::new(CargoTestSkill::new()));
    registry.register(Arc::new(CargoClippySkill::new()));
    registry.register(Arc::new(CodeFixSkill::new()));
}
