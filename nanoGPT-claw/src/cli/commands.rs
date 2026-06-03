//! CLI Commands - FULLY INTEGRATED WITH MEMORY & APEX
//!
//! ✅ 真的初始化完整记忆层
//! ✅ 真的初始化带 APEX 进化的 Scheduler
//! ✅ 真的调用完整闭环流程
//!
//! 不再造假！

use crate::daemon_service::{Task, TaskQueue, TaskStatus, TaskType, TaskWorker};
use crate::memory::{MemoryConfig, MemoryLayer, MemoryStats};
use crate::middleware::{MessageContext, MessageMiddleware, MessageSource};
use crate::scheduler::Scheduler;
use crate::skill::auto_fix::AutoFixSkill;
use crate::skill::built_in::{
    CargoCheckSkill, CargoClippySkill, CargoTestSkill, CodeFixSkill, EchoSkill, HelpSkill,
    StatusSkill,
};
use crate::skill::github_api::GitHubApiSkill;
use crate::skill::SkillRegistry;
use std::collections::HashMap;
use std::sync::Arc;
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

/// Global app config storage
static APP_CONFIG: once_cell::sync::Lazy<
    parking_lot::RwLock<Option<crate::config::settings::AppConfig>>,
> = once_cell::sync::Lazy::new(|| parking_lot::RwLock::new(None));

/// Initialize app config with environment variables
pub fn init_app_config() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let app_config = crate::config::init_config()?;
    let env_config = crate::config::EnvConfig::load();
    let merged_config = crate::config::merge_env_config(app_config, &env_config);
    *APP_CONFIG.write() = Some(merged_config);
    Ok(())
}

/// Get global config
pub fn get_app_config() -> Option<crate::config::settings::AppConfig> {
    APP_CONFIG.read().clone()
}

/// 真的完整处理，初始化所有系统！
pub async fn process_message(
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    if message.trim().is_empty() {
        warn!("Empty message received");
        return Ok(());
    }

    info!("Processing message with FULL INTEGRATION: {}", message);

    let ctx = MessageContext {
        content: message.to_string(),
        source: MessageSource::Cli,
        user_id: "cli_user".to_string(),
        session_id: "cli_session".to_string(),
        timestamp: chrono_now(),
        metadata: Default::default(),
    };

    // 1. 真的初始化完整记忆层！
    info!("Initializing full MemoryLayer...");
    let memory_config = MemoryConfig::default();
    let memory_layer = MemoryLayer::new(memory_config).await?;
    info!("✅ MemoryLayer initialized!");

    // 2. 真的初始化带 APEX 进化的 Scheduler！
    info!("Initializing full Scheduler with APEX evolution...");
    let scheduler: Arc<Scheduler> = if let Some(config) = get_app_config() {
        info!("✅ Using app config for Scheduler initialization");
        let mut sched = Scheduler::from_app_config(&config);
        sched.memory = Some(Arc::new(memory_layer));
        Arc::new(sched)
    } else {
        info!("✅ Using default Scheduler with memory");
        Arc::new(Scheduler::with_memory(memory_layer))
    };

    info!("✅ Full Scheduler with APEX evolution initialized!");

    // 3. 创建中间件处理
    let middleware = MessageMiddleware::new(scheduler);
    let response = middleware.process(ctx).await?;

    info!("✅ Response received, length: {}", response.content.len());
    println!("\n{}", response.content);

    Ok(())
}

pub async fn manage_memory(
    subcmd: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let config = MemoryConfig::default();
    let memory = MemoryLayer::new(config).await?;

    match subcmd {
        "show" | "stats" => {
            let stats = memory.stats().await;
            println!("\n=== 记忆统计 ===");
            println!("会话条目: {}", stats.session_entries);
            println!("持久化条目: {}", stats.persistent_entries);
        }
        "clear" => {
            info!("Clearing session memory");
            memory.clear_session().await;
            println!("✅ 会话记忆已清空");
        }
        _ => {
            println!("未知的记忆命令: {}", subcmd);
            println!("可用命令: show, clear, stats");
        }
    }

    Ok(())
}

pub async fn get_system_status(
) -> Result<SystemStatus, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let config = MemoryConfig::default();
    let memory = MemoryLayer::new(config).await?;
    let mem_stats = memory.stats().await;

    let scheduler = Scheduler::with_memory(memory);
    let scheduler_stats = scheduler.get_stats().await;

    Ok(SystemStatus {
        version: "0.9.1".to_string(),
        uptime_seconds: get_uptime(),
        memory_stats: mem_stats,
        daemon_running: is_daemon_running(),
        scheduler_active: scheduler.is_active(),
        apex_score: scheduler_stats.apex_score,
    })
}

#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub version: String,
    pub uptime_seconds: u64,
    pub memory_stats: MemoryStats,
    pub daemon_running: bool,
    pub scheduler_active: bool,
    pub apex_score: f64,
}

pub type MemoryStatsAlias = MemoryStats;



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

pub async fn add_task(
    task_type: TaskType,
    description: String,
) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let task = Task::new(task_type, description);
    let task_id = TASK_QUEUE.add_task(task).await?;
    println!("✅ 任务创建: [{}]", task_id);
    Ok(task_id)
}

pub async fn list_tasks() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let tasks = TASK_QUEUE.list_tasks().await;
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  后台任务列表                                                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    if tasks.is_empty() {
        println!("  📭 暂无任务");
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

        println!("\n  {} 任务 [{}]", status_icon, task.id);
        println!("    类型:     {:?}", task.task_type);
        println!("    状态:     {:?}", task.status);
        println!("    描述:     {}", task.description);
        println!("    进度:     {:.1}%", task.progress);
        if let Some(result) = &task.result {
            println!("    结果:     {}", result);
        }
        if let Some(error) = &task.error {
            println!("    错误:     {}", error);
        }
    }

    Ok(())
}

pub async fn get_task(
    task_id: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    if let Some(task) = TASK_QUEUE.get_task(&task_id).await {
        let status_icon = match task.status {
            TaskStatus::Pending => "⏳",
            TaskStatus::Running => "🚀",
            TaskStatus::Completed => "✅",
            TaskStatus::Failed => "❌",
            TaskStatus::Cancelled => "🚫",
        };

        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║  任务详情                                                     ║");
        println!("╚══════════════════════════════════════════════════════════════╝");

        println!("\n  {} 任务 [{}]", status_icon, task.id);
        println!("  类型:         {:?}", task.task_type);
        println!("  描述:         {}", task.description);
        println!("  状态:         {:?}", task.status);
        println!("  创建时间:     {}", task.created_at);
        println!(
            "  开始时间:     {}",
            task.started_at
                .as_ref()
                .map(|t| t.to_string())
                .unwrap_or_else(|| "N/A".to_string())
        );
        println!(
            "  完成时间:     {}",
            task.completed_at
                .as_ref()
                .map(|t| t.to_string())
                .unwrap_or_else(|| "N/A".to_string())
        );
        println!("  进度:         {:.1}%", task.progress);

        if let Some(result) = &task.result {
            println!("  结果:         {}", result);
        }
        if let Some(error) = &task.error {
            println!("  错误:         {}", error);
        }
    } else {
        println!("❌ 未找到任务: {}", task_id);
    }

    Ok(())
}

pub async fn cancel_task(
    task_id: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    TASK_QUEUE.cancel_task(&task_id).await?;
    println!("✅ 任务已取消: [{}]", task_id);
    Ok(())
}

pub async fn start_task_worker() {
    let worker = TaskWorker::new(TASK_QUEUE.clone());
    worker.start().await;
    println!("✅ 后台任务工作进程已启动！");
}

pub async fn list_skills() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let skills = SKILL_REGISTRY.list_skills();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  可用技能列表                                                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    if skills.is_empty() {
        println!("  📭 暂无可使用的技能");
        return Ok(());
    }

    println!("\n总技能数: {}\n", skills.len());

    for skill in skills {
        println!("  🛠️  {}", skill.id);
        println!("    名称:     {}", skill.name);
        println!("    版本:     {}", skill.version);
        println!("    描述:     {}", skill.description);
        println!("    分类:     {:?}", skill.category);
        println!("    启用:     {}", if skill.enabled { "✅" } else { "❌" });
        println!();
    }

    println!("\n💡 使用方法: skill run <技能ID>");
    println!("   示例: skill run cargo-check");

    Ok(())
}

pub async fn run_skill(
    skill_id: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    println!("\n🚀 执行技能: {}", skill_id);
    println!("═══════════════════════════════════════════════════");

    match SKILL_REGISTRY.execute(&skill_id, HashMap::new()).await {
        Ok(result) => {
            if result.success {
                println!("✅ 技能执行成功！");
                println!("\n📤 输出:");
                println!("{}", result.output);

                if !result.metadata.is_empty() {
                    println!("\n📊 元数据:");
                    for (key, value) in &result.metadata {
                        println!("  {}: {}", key, value);
                    }
                }

                println!("\n⏱️  执行耗时: {}ms", result.execution_time_ms);
            } else {
                println!("❌ 技能执行失败！");
                println!("\n📤 输出:");
                println!("{}", result.output);
            }
        }
        Err(e) => {
            println!("❌ 执行技能时出错: {}", e);
            println!("\n💡 请执行 'skill list' 查看可用技能");
        }
    }

    println!("═══════════════════════════════════════════════════\n");

    Ok(())
}
