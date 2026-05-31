use crate::skill::{Skill, SkillMetadata, SkillResult, SkillCategory, SkillError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

pub struct AutoFixSkill {
    metadata: SkillMetadata,
    max_iterations: usize,
}

impl AutoFixSkill {
    pub fn new() -> Self {
        Self {
            metadata: SkillMetadata {
                id: "auto-fix".to_string(),
                name: "Auto Fix".to_string(),
                version: "1.0.0".to_string(),
                description: "真正闭环自修复 - 检测编译错误并循环修复直到通过".to_string(),
                author: "NanoGPT-Claw".to_string(),
                category: SkillCategory::Code,
                enabled: true,
                parameters: vec![
                    crate::skill::SkillParameter {
                        name: "max_iterations".to_string(),
                        description: "最大修复迭代次数".to_string(),
                        param_type: "number".to_string(),
                        required: false,
                        default_value: Some("10".to_string()),
                    },
                ],
            },
            max_iterations: 10,
        }
    }

    pub fn with_max_iterations(max: usize) -> Self {
        let mut skill = Self::new();
        skill.max_iterations = max;
        skill
    }

    fn parse_errors(stderr: &str) -> Vec<CompileError> {
        let mut errors = Vec::new();
        let lines: Vec<&str> = stderr.lines().collect();
        
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            
            if line.contains("error[E") || line.contains("error:") {
                let file = if let Some(pos) = line.find("-->") {
                    let path_part = &line[pos..];
                    path_part.split(':').nth(1).unwrap_or("unknown").trim().to_string()
                } else {
                    "unknown".to_string()
                };
                
                let error_code = if let Some(start) = line.find("error[E") {
                    let end = line[start..].find(']').map(|p| start + p + 1).unwrap_or(line.len());
                    line[start..end].to_string()
                } else {
                    "E0000".to_string()
                };
                
                let mut message = line.to_string();
                i += 1;
                while i < lines.len() && !lines[i].contains("error[E") && !lines[i].contains("error -->") && !lines[i].is_empty() {
                    message.push_str(&format!("
{}", lines[i]));
                    i += 1;
                }
                
                errors.push(CompileError {
                    file,
                    code: error_code,
                    message,
                });
            } else {
                i += 1;
            }
        }
        errors
    }

    fn run_check(&self) -> Result<(bool, String), String> {
        let output = Command::new("cargo")
            .args(["check", "--message-format=short"])
            .output()
            .map_err(|e| format!("Failed to run cargo check: {}", e))?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        Ok((output.status.success(), format!("{}{}", stdout, stderr)))
    }

    fn run_fix(&self) -> Result<usize, String> {
        let output = Command::new("cargo")
            .args(["fix", "--lib", "--allow-dirty", "--allow-staged", "--message-format=short"])
            .output()
            .map_err(|e| format!("Failed to run cargo fix: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let fixed = stdout.matches("Fixed").count();

        Ok(fixed)
    }

    async fn run_auto_fix_loop(&self) -> Result<(bool, usize, usize), String> {
        let mut iteration = 0;
        let mut total_fixed = 0;
        let mut last_error_count = usize::MAX;

        loop {
            iteration += 1;
            if iteration > self.max_iterations {
                return Err(format!("达到最大迭代次数 {}，仍有未修复的错误", self.max_iterations));
            }

            let (success, output) = self.run_check()?;
            
            if success {
                return Ok((true, iteration, total_fixed));
            }

            let errors = Self::parse_errors(&output);
            let current_error_count = errors.len();
            
            if current_error_count >= last_error_count && iteration > 1 {
                return Err(format!(
                    "无法自动修复 {} 个错误（可能是手动代码问题）", 
                    current_error_count
                ));
            }
            last_error_count = current_error_count;

            println!("  🔧 第 {} 轮: 检测到 {} 个错误，尝试修复...", iteration, current_error_count);

            let fixed = self.run_fix()?;
            total_fixed += fixed;

            if fixed == 0 {
                return Err(format!(
                    "第 {} 轮修复后仍有 {} 个错误无法自动修复", 
                    iteration, current_error_count
                ));
            }

            println!("  ✅ 第 {} 轮: 修复了 {} 个问题", iteration, fixed);
        }
    }
}

impl Default for AutoFixSkill {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone)]
struct CompileError {
    #[allow(dead_code)]
    file: String,
    code: String,
    message: String,
}

#[async_trait]
impl Skill for AutoFixSkill {
    fn metadata(&self) -> &SkillMetadata { &self.metadata }
    
    async fn execute(&self, params: HashMap<String, String>) -> Result<SkillResult, SkillError> {
        let start = Instant::now();
        
        let max_iter = params.get("max_iterations")
            .and_then(|s| s.parse().ok())
            .unwrap_or(self.max_iterations);

        println!("
{}", "═".repeat(70));
        println!("  🔄 AUTO-FIX 真正闭环自修复引擎");
        println!("  ⏱️  最大迭代: {}", max_iter);
        println!("{}", "═".repeat(70));

        let initial_check = self.run_check();
        
        let (initial_success, initial_output) = match initial_check {
            Ok((success, output)) => (success, output),
            Err(e) => {
                return Ok(SkillResult {
                    success: false,
                    output: format!("❌ 初始化检查失败: {}", e),
                    metadata: Default::default(),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                });
            }
        };

        if initial_success {
            println!("
  ✨ 系统已无错误，无需修复！");
            println!("{}", "═".repeat(70));
            return Ok(SkillResult {
                success: true,
                output: "✅ 系统无编译错误".to_string(),
                metadata: vec![
                    ("status".to_string(), "clean".to_string()),
                    ("iterations".to_string(), "0".to_string()),
                ].into_iter().collect(),
                execution_time_ms: start.elapsed().as_millis() as u64,
            });
        }

        let initial_errors = Self::parse_errors(&initial_output);
        println!("
  📋 初始检测: {} 个错误", initial_errors.len());
        for (i, err) in initial_errors.iter().take(3).enumerate() {
            println!("    {}. [{}] {}", i + 1, err.code, err.message.lines().next().unwrap_or(""));
        }
        if initial_errors.len() > 3 {
            println!("    ... 还有 {} 个错误", initial_errors.len() - 3);
        }

        match self.run_auto_fix_loop().await {
            Ok((_success, iterations, total_fixed)) => {
                let duration = start.elapsed().as_millis() as u64;
                let output = format!(
                    "✅ 自修复完成！
  迭代次数: {}
  总修复数: {}
  耗时: {}ms",
                    iterations, total_fixed, duration
                );
                println!("
{}", "═".repeat(70));
                println!("  {}", output);
                println!("{}", "═".repeat(70));
                
                Ok(SkillResult {
                    success: true,
                    output,
                    metadata: vec![
                        ("status".to_string(), "fixed".to_string()),
                        ("iterations".to_string(), iterations.to_string()),
                        ("total_fixed".to_string(), total_fixed.to_string()),
                        ("duration_ms".to_string(), duration.to_string()),
                    ].into_iter().collect(),
                    execution_time_ms: duration,
                })
            }
            Err(e) => {
                let duration = start.elapsed().as_millis() as u64;
                println!("
  ❌ {}", e);
                println!("{}", "═".repeat(70));
                
                Ok(SkillResult {
                    success: false,
                    output: format!("❌ {}
  提示: 请手动检查代码问题", e),
                    metadata: vec![
                        ("status".to_string(), "failed".to_string()),
                        ("error".to_string(), e),
                    ].into_iter().collect(),
                    execution_time_ms: duration,
                })
            }
        }
    }
}
