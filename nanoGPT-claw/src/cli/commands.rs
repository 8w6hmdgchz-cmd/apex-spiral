//! CLI Commands - Command Processing and Execution

use std::sync::Arc;
use std::collections::HashMap;
use crate::memory::{MemoryManager, MemoryConfig, MemoryStats as MemStats};
use crate::scheduler::Scheduler;
use crate::middleware::{MessageMiddleware, MessageContext, MessageSource};
use crate::daemon_service::{TaskQueue, TaskWorker, Task, TaskType, TaskStatus};
use crate::skill::SkillRegistry;
use crate::skill::built_in::{
    EchoSkill, HelpSkill, StatusSkill,
    CargoCheckSkill, CargoTestSkill, CargoClippySkill, CodeFixSkill
};
use crate::skill::auto_fix::AutoFixSkill;
use crate::skill::github_api::GitHubApiSkill;
use tracing::{info, warn};

static TASK_QUEUE: once_cell::sync::Lazy<Arc<TaskQueue>> = 
    once_cell::sync::Lazy::new(|| Arc::new(TaskQueue::new(4)));

static SKILL_REGISTRY: once_cell::sync::Lazy<Arc<SkillRegistry>> = 
    once_cell::sync::Lazy::new(|| {
        let registry = Arc::new(SkillRegistry::new());
        registry.register(Arc::new(EchoSkill::new()));
        registry.register(Arc::new(HelpSkill::new()));
        registry.register(Arc::new(StatusSkill::new()));
        registry.register(Arc::new(CargoCheckSkill::new()));
        registry.register(Arc::new(CargoTestSkill::new()));
        registry.register(Arc::new(CargoClippySkill::new()));
        registry.register(Arc::new(CodeFixSkill::new()));
        registry.register(Arc::new(AutoFixSkill::new()));
        registry.register(Arc::new(GitHubApiSkill::new()));
        registry
    });

pub async fn process_message(message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    if message.trim().is_empty() {
        warn!("Empty message received");
        return Ok(());
    }

    info!("Processing message: {}", message);

    let ctx = MessageContext {
        content: message.to_string(),
        source: MessageSource::Cli,
        user_id: "cli_user".to_string(),
        session_id: uuid_v4(),
        timestamp: chrono_now(),
        metadata: Default::default(),
    };

    let scheduler = Arc::new(Scheduler::new());
    let middleware = MessageMiddleware::new(scheduler);
    let response = middleware.process(ctx).await?;

    info!("Response: {}", response.content);
    println!("\n[Agent] {}", response.content);

    Ok(())
}

pub async fn manage_memory(subcmd: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let config = MemoryConfig::default();
    let memory = MemoryManager::new(config).await?;

    match subcmd {
        "show" | "stats" => {
            let stats = memory.stats().await;
            println!("\n=== Memory Statistics ===");
            println!("Session entries: {}", stats.session_entries);
            println!("Persistent entries: {}", stats.persistent_entries);
        }
        "clear" => {
            info!("Clearing session memory...");
            memory.clear_session().await;
            println!("Session memory cleared.");
        }
        _ => {
            println!("Unknown memory command: {}", subcmd);
            println!("Available: show, clear, stats");
        }
    }

    Ok(())
}

pub async fn get_system_status() -> Result<SystemStatus, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let config = MemoryConfig::default();
    let memory = MemoryManager::new(config).await?;
    let mem_stats = memory.stats().await;

    let scheduler = Scheduler::new();

    Ok(SystemStatus {
        version: "0.1.0".to_string(),
        uptime_seconds: get_uptime(),
        memory_stats: mem_stats,
        daemon_running: is_daemon_running(),
        scheduler_active: scheduler.is_active(),
    })
}

#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub version: String,
    pub uptime_seconds: u64,
    pub memory_stats: MemStats,
    pub daemon_running: bool,
    pub scheduler_active: bool,
}

pub type MemoryStats = MemStats;

fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn get_uptime() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn is_daemon_running() -> bool {
    std::path::Path::new("/tmp/nano-gpt-claw.pid").exists()
}

pub async fn add_task(task_type: TaskType, description: String) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let task = Task::new(task_type, description);
    let task_id = TASK_QUEUE.add_task(task).await?;
    println!("✅ Task created: [{}]", task_id);
    Ok(task_id)
}

pub async fn list_tasks() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let tasks = TASK_QUEUE.list_tasks().await;
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  Background Tasks                                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    if tasks.is_empty() {
        println!("  📭 No tasks found.");
        return Ok(());
    }
    
    for task in tasks {
        let status_icon = match task.status {
            TaskStatus::Pending => "⏳",
            TaskStatus::Running => "🚀",
            TaskStatus::Completed => "✅",
            TaskStatus::Failed => "❌",
            TaskStatus::Cancelled => "🚫",
        };
        
        println!("\n  {} Task [{}]", status_icon, task.id);
        println!("    Type:    {:?}", task.task_type);
        println!("    Status:  {:?}", task.status);
        println!("    Description: {}", task.description);
        println!("    Progress: {:.1}%", task.progress);
        if let Some(result) = &task.result {
            println!("    Result: {}", result);
        }
        if let Some(error) = &task.error {
            println!("    Error:  {}", error);
        }
    }
    
    Ok(())
}

pub async fn get_task(task_id: String) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    if let Some(task) = TASK_QUEUE.get_task(&task_id).await {
        let status_icon = match task.status {
            TaskStatus::Pending => "⏳",
            TaskStatus::Running => "🚀",
            TaskStatus::Completed => "✅",
            TaskStatus::Failed => "❌",
            TaskStatus::Cancelled => "🚫",
        };
        
        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║  Task Details                                               ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        
        println!("\n  {} Task [{}]", status_icon, task.id);
        println!("  Type:         {:?}", task.task_type);
        println!("  Description:  {}", task.description);
        println!("  Status:       {:?}", task.status);
        println!("  Created:      {}", task.created_at);
        println!("  Started:      {}", task.started_at.as_ref().map(|t| t.to_string()).unwrap_or_else(|| "N/A".to_string()));
        println!("  Completed:    {}", task.completed_at.as_ref().map(|t| t.to_string()).unwrap_or_else(|| "N/A".to_string()));
        println!("  Progress:     {:.1}%", task.progress);
        
        if let Some(result) = &task.result {
            println!("  Result:       {}", result);
        }
        if let Some(error) = &task.error {
            println!("  Error:        {}", error);
        }
    } else {
        println!("❌ Task not found: {}", task_id);
    }
    
    Ok(())
}

pub async fn cancel_task(task_id: String) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    TASK_QUEUE.cancel_task(&task_id).await?;
    println!("✅ Task cancelled: [{}]", task_id);
    Ok(())
}

pub async fn start_task_worker() {
    let worker = TaskWorker::new(TASK_QUEUE.clone());
    worker.start().await;
    println!("✅ Background task worker started!");
}

pub async fn list_skills() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let skills = SKILL_REGISTRY.list_skills();
    
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  Available Skills                                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    if skills.is_empty() {
        println!("  📭 No skills registered.");
        return Ok(());
    }
    
    println!("\nTotal skills: {}\n", skills.len());
    
    for skill in skills {
        println!("  🛠️  {}", skill.id);
        println!("     Name:    {}", skill.name);
        println!("     Version: {}", skill.version);
        println!("     Desc:    {}", skill.description);
        println!("     Category: {:?}", skill.category);
        println!("     Enabled: {}", if skill.enabled { "✅" } else { "❌" });
        println!();
    }
    
    println!("\n💡 Usage: skill run <skill_id>");
    println!("   Example: skill run cargo-check");
    
    Ok(())
}

pub async fn run_skill(skill_id: String) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    println!("\n🚀 Running skill: {}", skill_id);
    println!("═══════════════════════════════════════════════════");
    
    match SKILL_REGISTRY.execute(&skill_id, HashMap::new()).await {
        Ok(result) => {
            if result.success {
                println!("✅ Skill executed successfully!");
                println!("\n📤 Output:");
                println!("{}", result.output);
                
                if !result.metadata.is_empty() {
                    println!("\n📊 Metadata:");
                    for (key, value) in &result.metadata {
                        println!("  {}: {}", key, value);
                    }
                }
                
                println!("\n⏱️  Execution time: {}ms", result.execution_time_ms);
            } else {
                println!("❌ Skill execution failed!");
                println!("\n📤 Output:");
                println!("{}", result.output);
            }
        }
        Err(e) => {
            println!("❌ Failed to execute skill: {}", e);
            println!("\n💡 Try: skill list  (to see available skills)");
        }
    }
    
    println!("═══════════════════════════════════════════════════\n");
    
    Ok(())
}
