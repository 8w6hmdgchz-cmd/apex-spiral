use crate::skill::{Skill, SkillMetadata, SkillResult, SkillCategory, SkillError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

pub struct GitHubApiSkill {
    metadata: SkillMetadata,
}

impl GitHubApiSkill {
    pub fn new() -> Self {
        Self {
            metadata: SkillMetadata {
                id: "github-api".to_string(),
                name: "GitHub API".to_string(),
                version: "1.0.0".to_string(),
                description: "调用GitHub API创建Issue、PR、触发Actions".to_string(),
                author: "NanoGPT-Claw".to_string(),
                category: SkillCategory::Automation,
                enabled: true,
                parameters: vec![
                    crate::skill::SkillParameter {
                        name: "action".to_string(),
                        description: "操作类型: create-issue, list-issues, trigger-actions, create-pr".to_string(),
                        param_type: "string".to_string(),
                        required: true,
                        default_value: None,
                    },
                    crate::skill::SkillParameter {
                        name: "title".to_string(),
                        description: "Issue或PR的标题".to_string(),
                        param_type: "string".to_string(),
                        required: false,
                        default_value: None,
                    },
                    crate::skill::SkillParameter {
                        name: "body".to_string(),
                        description: "Issue或PR的内容".to_string(),
                        param_type: "string".to_string(),
                        required: false,
                        default_value: None,
                    },
                    crate::skill::SkillParameter {
                        name: "labels".to_string(),
                        description: "标签，逗号分隔".to_string(),
                        param_type: "string".to_string(),
                        required: false,
                        default_value: None,
                    },
                ],
            },
        }
    }
}

impl Default for GitHubApiSkill {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Skill for GitHubApiSkill {
    fn metadata(&self) -> &SkillMetadata { &self.metadata }
    
    async fn execute(&self, params: HashMap<String, String>) -> Result<SkillResult, SkillError> {
        let start = Instant::now();
        
        let action = params.get("action")
            .ok_or_else(|| SkillError::MissingParameter("action".to_string()))?;
        
        println!("
{}", "═".repeat(70));
        println!("  🔗 GitHub API Skill - n+n 自动进化");
        println!("  ⚡ Action: {}", action);
        println!("{}", "═".repeat(70));
        
        let result = match action.as_str() {
            "create-issue" => self.create_issue(&params).await,
            "list-issues" => self.list_issues().await,
            "trigger-actions" => self.trigger_actions(&params).await,
            "create-pr" => self.create_pr(&params).await,
            "get-repo-status" => self.get_repo_status().await,
            "auto-evolve" => self.auto_evolve().await,
            _ => Err(format!("未知action: {}", action)),
        };
        
        let duration = start.elapsed().as_millis() as u64;
        
        match result {
            Ok(output) => {
                println!("
{}", "═".repeat(70));
                Ok(SkillResult {
                    success: true,
                    output: output.clone(),
                    metadata: vec![
                        ("action".to_string(), action.clone()),
                        ("duration_ms".to_string(), duration.to_string()),
                    ].into_iter().collect(),
                    execution_time_ms: duration,
                })
            }
            Err(e) => {
                println!("
  ❌ Error: {}", e);
                println!("{}", "═".repeat(70));
                Ok(SkillResult {
                    success: false,
                    output: format!("❌ {}", e),
                    metadata: vec![
                        ("action".to_string(), action.clone()),
                        ("error".to_string(), e),
                    ].into_iter().collect(),
                    execution_time_ms: duration,
                })
            }
        }
    }
}

impl GitHubApiSkill {
    fn get_token(&self) -> Result<String, String> {
        std::env::var("GITHUB_TOKEN")
            .map_err(|_| "GITHUB_TOKEN 环境变量未设置".to_string())
    }
    
    fn get_repo_info(&self) -> Result<(String, String), String> {
        // Primary: use environment variables (works in CI/deployed environments)
        if let (Ok(owner), Ok(repo)) = (
            std::env::var("GITHUB_REPO_OWNER"),
            std::env::var("GITHUB_REPO_NAME"),
        ) {
            return Ok((owner, repo));
        }
        
        // Fallback: parse git remote URL
        let remote = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .output()
            .map_err(|e| format!("Failed to get remote: {}", e))?;
        
        let url = String::from_utf8_lossy(&remote.stdout);
        
        let cleaned = url.trim()
            .replace("https://", "")
            .replace("git@github.com:", "")
            .replace(".git", "");
        let parts: Vec<&str> = cleaned.split('/').collect();
        
        if parts.len() >= 2 {
            Ok((parts[0].to_string(), parts[1].to_string()))
        } else {
            Err("无法解析仓库信息: 请设置 GITHUB_REPO_OWNER 和 GITHUB_REPO_NAME 环境变量".to_string())
        }
    }
    
    fn api_call(&self, method: &str, endpoint: &str, body: Option<&str>) -> Result<String, String> {
        let token = self.get_token()?;
        let (owner, repo) = self.get_repo_info()?;
        let url = format!("https://api.github.com/repos/{}/{}{}", owner, repo, endpoint);
        
        let auth_header = format!("Authorization: token {}", token);
        let mut args = vec!["-s", "-X", method];
        args.extend(["-H", &auth_header]);
        args.extend(["-H", "Accept: application/vnd.github.v3+json"]);
        args.extend(["-H", "Content-Type: application/json"]);
        
        if let Some(b) = body {
            args.extend(["-d", b]);
        }
        
        let output = Command::new("curl")
            .args(&args)
            .arg(&url)
            .output()
            .map_err(|e| format!("curl failed: {}", e))?;
        
        let response = String::from_utf8_lossy(&output.stdout);
        Ok(response.to_string())
    }
    
    async fn create_issue(&self, params: &HashMap<String, String>) -> Result<String, String> {
        let title = params.get("title")
            .ok_or_else(|| "缺少title参数".to_string())?;
        let body = params.get("body").cloned().unwrap_or_default();
        let labels = params.get("labels")
            .map(|s| s.split(',').map(|l| l.trim()).collect::<Vec<_>>())
            .unwrap_or_default();
        
        let mut labels_json = String::from("[");
        for (i, label) in labels.iter().enumerate() {
            if i > 0 { labels_json.push(','); }
            labels_json.push_str(&format!("\"{}\"", label));
        }
        labels_json.push(']');
        
        let payload = format!(
            r#"{{"title":"{}","body":"{}","labels":{}}}"#,
            title.replace('"', "\""),
            body.replace('"', "\""),
            labels_json
        );
        
        let result = self.api_call("POST", "/issues", Some(&payload))?;
        
        if result.contains("\"html_url\"") {
            if let Some(url) = result.lines().find(|l| l.contains("html_url"))
                .and_then(|l| l.split(':').nth(1)) 
            {
                return Ok(format!("✅ Issue创建成功!\n  URL: {}", url.trim()));
            }
        }
        
        Ok(format!("✅ Issue API调用完成
{}", &result[..result.len().min(500)]))
    }
    
    async fn list_issues(&self) -> Result<String, String> {
        let result = self.api_call("GET", "/issues?state=open", None)?;
        
        let mut output = String::from("📋 开放Issues:

");
        
        if let Some(start) = result.find("[{\"url\"") {
            let json_part = &result[start..];
            for (i, line) in json_part.lines().enumerate() {
                if i > 20 { break; }
                if line.contains("\"title\"") {
                    let title = line.split("\"title\":").nth(1)
                        .map(|s| s.split(',').next().unwrap_or(s))
                        .map(|s| s.trim().trim_matches('"'))
                        .unwrap_or("");
                    output.push_str(&format!("  • {}
", title));
                }
            }
        } else {
            output.push_str("  (无开放Issue)");
        }
        
        Ok(output)
    }
    
    async fn trigger_actions(&self, params: &HashMap<String, String>) -> Result<String, String> {
        let workflow = params.get("workflow")
            .ok_or_else(|| "缺少workflow参数".to_string())?;
        
        let token = self.get_token()?;
        let (owner, repo) = self.get_repo_info()?;
        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/workflows/{}/runs",
            owner, repo, workflow
        );
        
        let output = Command::new("curl")
            .args([
                "-s", "-X", "POST",
                "-H", &format!("Authorization: token {}", token),
                "-H", "Accept: application/vnd.github.v3+json",
                &url,
            ])
            .output()
            .map_err(|e| format!("curl failed: {}", e))?;
        
        let response = String::from_utf8_lossy(&output.stdout);
        
        Ok(format!("⚡ Actions触发结果:
{}", &response[..response.len().min(300)]))
    }
    
    async fn create_pr(&self, params: &HashMap<String, String>) -> Result<String, String> {
        let title = params.get("title")
            .ok_or_else(|| "缺少title参数".to_string())?;
        let body = params.get("body").cloned().unwrap_or_default();
        let head = params.get("head").cloned().unwrap_or_else(|| "master".to_string());
        let base = params.get("base").cloned().unwrap_or_else(|| "main".to_string());
        
        let payload = format!(
            r#"{{"title":"{}","body":"{}","head":"{}","base":"{}"}}"#,
            title.replace('"', "\""),
            body.replace('"', "\""),
            head,
            base
        );
        
        let result = self.api_call("POST", "/pulls", Some(&payload))?;
        
        Ok(format!("✅ PR创建结果:
{}", &result[..result.len().min(400)]))
    }
    
    async fn get_repo_status(&self) -> Result<String, String> {
        let result = self.api_call("GET", "", None)?;
        
        let mut output = String::from("📊 仓库状态:
");
        
        let fields = ["full_name", "stargazers_count", "forks_count", "open_issues_count"];
        for field in &fields {
            if let Some(line) = result.lines().find(|l| l.contains(field)) {
                output.push_str(&format!("  {}: {}
", field, line.split(':').nth(1).unwrap_or("")));
            }
        }
        
        Ok(output)
    }
    
    async fn auto_evolve(&self) -> Result<String, String> {
        println!("
  🌀 启动自动进化循环...");
        
        let stats = Command::new("cargo")
            .args(["test", "--", "--list"])
            .output();
        
        let test_count = stats.as_ref()
            .map(|s| String::from_utf8_lossy(&s.stdout))
            .map(|o| o.lines().filter(|l| l.ends_with(": test")).count())
            .unwrap_or(0);
        
        let build_status = Command::new("cargo")
            .args(["check", "--quiet"])
            .output()
            .map(|o| if o.status.success() { "✅ 通过" } else { "❌ 失败" })
            .unwrap_or("❌ 失败");
        
        let issue_body = format!(
            "## 🔄 GPT自动进化报告

### 系统状态
- 测试数量: {}
- 编译状态: {}
- 时间: {}

### 进化建议
(由GPT分析后补充)

---
*Auto-evolved by nanoGPT-Claw + GitHub API*",
            test_count,
            build_status,
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        );
        
        let mut params = HashMap::new();
        params.insert("action".to_string(), "create-issue".to_string());
        params.insert("title".to_string(), format!("[Auto] 进化报告 #{}", chrono::Utc::now().timestamp()));
        params.insert("body".to_string(), issue_body);
        params.insert("labels".to_string(), "auto-evolve,gpt-generated".to_string());
        
        let result = self.create_issue(&params).await?;
        
        Ok(format!(
            "🔄 自动进化完成!

  测试: {}个
  编译: {}
  Issue: {}",
            test_count, build_status, result
        ))
    }
}
