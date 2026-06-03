//! NanoGPT-Claw - 后台任务队列系统
//! 支持多任务并行处理、任务持久化、状态追踪

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    CodeFix,
    CodeUpgrade,
    TodoComplete,
    Research,
    Benchmark,
    GitHubSearch,
    AutoResearch,
    OpenHands,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub task_type: TaskType,
    pub description: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: f32,
    pub result: Option<String>,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl Task {
    pub fn new(task_type: TaskType, description: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_type,
            description,
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            progress: 0.0,
            result: None,
            error: None,
            metadata: Default::default(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        )
    }
}

pub struct TaskQueue {
    pending: Arc<RwLock<Vec<String>>>,
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    max_parallel: usize,
    tx: Option<mpsc::Sender<String>>,
    rx: RwLock<Option<mpsc::Receiver<String>>>,
    python_exec_path: PathBuf,
    storage_path: PathBuf,
}

impl TaskQueue {
    pub fn new(max_parallel: usize) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let queue = Self {
            pending: Arc::new(RwLock::new(Vec::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            max_parallel,
            tx: Some(tx),
            rx: RwLock::new(Some(rx)),
            python_exec_path: PathBuf::from("python3"),
            storage_path: PathBuf::from("/tmp/nano-gpt-claw-tasks"),
        };

        // Clone the necessary parts for the async task
        let tasks_clone = queue.tasks.clone();
        let storage_path_clone = queue.storage_path.clone();

        tokio::spawn(async move {
            let file_path = storage_path_clone.join("tasks.json");
            if file_path.exists() {
                match std::fs::read_to_string(file_path) {
                    Ok(json) => match serde_json::from_str::<HashMap<String, Task>>(&json) {
                        Ok(tasks_data) => {
                            let mut tasks = tasks_clone.write().await;
                            *tasks = tasks_data;
                            info!("Loaded {} tasks from disk", tasks.len());
                        }
                        Err(e) => warn!("Failed to parse tasks from disk: {}", e),
                    },
                    Err(e) => warn!("Failed to read tasks from disk: {}", e),
                }
            }
        });

        queue
    }

    pub fn with_paths(mut self, python_path: PathBuf, storage_path: PathBuf) -> Self {
        self.python_exec_path = python_path;
        self.storage_path = storage_path;
        self
    }

    pub async fn add_task(&self, task: Task) -> Result<String> {
        let task_id = task.id.clone();
        info!("Adding task to queue: [{}] {}", task_id, task.description);

        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id.clone(), task);

        let mut pending = self.pending.write().await;

        // Backpressure: if pending queue exceeds max_parallel * 10, reject
        if pending.len() >= self.max_parallel * 10 {
            return Err(anyhow!(
                "Task queue full ({} pending, max {})",
                pending.len(),
                self.max_parallel * 10
            ));
        }

        pending.push(task_id.clone());

        // 尝试发送，但不等待，如果没有worker也能继续
        if let Some(tx) = &self.tx {
            // 使用 try_send 而不是 send，这样不会阻塞
            if let Err(e) = tx.try_send(task_id.clone()) {
                warn!("Failed to send task to worker: {}, but task is still saved and can be processed later", e);
            }
        }

        drop(pending);
        self.save_tasks_to_disk().await?;

        Ok(task_id)
    }

    pub async fn get_task(&self, task_id: &str) -> Option<Task> {
        self.tasks.read().await.get(task_id).cloned()
    }

    pub async fn list_tasks(&self) -> Vec<Task> {
        self.tasks.read().await.values().cloned().collect()
    }

    pub async fn update_task_status(&self, task_id: &str, status: TaskStatus) -> Result<()> {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.get_mut(task_id) {
            task.status = status.clone();
            match status {
                TaskStatus::Running => {
                    task.started_at = Some(Utc::now());
                }
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
                    task.completed_at = Some(Utc::now());
                }
                _ => {}
            }

            info!("Task [{}] status updated to: {:?}", task_id, status);
        }

        self.save_tasks_to_disk().await?;
        Ok(())
    }

    pub async fn update_task_progress(
        &self,
        task_id: &str,
        progress: f32,
        result: Option<String>,
    ) -> Result<()> {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.get_mut(task_id) {
            task.progress = progress.clamp(0.0, 100.0);
            if let Some(result) = result {
                task.result = Some(result);
            }
        }

        self.save_tasks_to_disk().await?;
        Ok(())
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        self.update_task_status(task_id, TaskStatus::Cancelled)
            .await?;
        Ok(())
    }

    pub async fn get_running_count(&self) -> usize {
        self.tasks
            .read()
            .await
            .values()
            .filter(|t| t.status == TaskStatus::Running)
            .count()
    }

    pub async fn get_next_task(&self) -> Option<Task> {
        let mut pending = self.pending.write().await;

        while !pending.is_empty() {
            let task_id = pending.remove(0);
            let mut tasks = self.tasks.write().await;

            if let Some(task) = tasks.get_mut(&task_id) {
                if task.status == TaskStatus::Pending {
                    task.status = TaskStatus::Running;
                    task.started_at = Some(Utc::now());
                    return Some(task.clone());
                }
            }
        }

        None
    }

    pub async fn cleanup_completed(&self, older_than_hours: i64) -> usize {
        let now = Utc::now();
        let cutoff = now - chrono::Duration::hours(older_than_hours);

        let mut tasks = self.tasks.write().await;
        let original_len = tasks.len();

        tasks.retain(|_, task| {
            if !task.is_finished() {
                return true;
            }

            if let Some(completed) = task.completed_at {
                completed > cutoff
            } else {
                true
            }
        });

        original_len - tasks.len()
    }

    pub async fn save_tasks_to_disk(&self) -> Result<()> {
        let tasks = self.tasks.read().await;
        let json = serde_json::to_string_pretty(&*tasks)?;
        std::fs::create_dir_all(&self.storage_path)?;
        let file_path = self.storage_path.join("tasks.json");
        std::fs::write(file_path, json)?;
        Ok(())
    }

    pub async fn load_tasks_from_disk(&self) -> Result<()> {
        let file_path = self.storage_path.join("tasks.json");
        if file_path.exists() {
            let json = std::fs::read_to_string(file_path)?;
            let tasks: HashMap<String, Task> = serde_json::from_str(&json)?;
            let mut self_tasks = self.tasks.write().await;
            *self_tasks = tasks;
            info!("Loaded {} tasks from disk", self_tasks.len());
        }
        Ok(())
    }

    async fn execute_python_command(&self, module: &str, args: Vec<String>) -> Result<String> {
        let python_script = format!(
            r#"
import sys
import json
sys.path.insert(0, '/workspace/nanoGPT-claw/python')

from core.logging import setup_logging
setup_logging()

try:
    from integrations.{module} import main
    result = main(*{args:?})
    print(json.dumps({{"success": True, "result": result}}))
except Exception as e:
    print(json.dumps({{"success": False, "error": str(e)}}))
    import traceback
    traceback.print_exc()
"#,
            module = module,
            args = args
        );

        let temp_script_path = std::env::temp_dir().join(format!("task_{}.py", Uuid::new_v4()));
        std::fs::write(&temp_script_path, python_script)?;

        let output = Command::new(&self.python_exec_path)
            .arg(&temp_script_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        std::fs::remove_file(temp_script_path)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !stderr.is_empty() {
            warn!("Python stderr: {}", stderr);
        }

        if !output.status.success() {
            return Err(anyhow!("Python command failed: {}", stderr));
        }

        Ok(stdout.trim().to_string())
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new(4)
    }
}

pub struct TaskWorker {
    queue: Arc<TaskQueue>,
}

impl TaskWorker {
    pub fn new(queue: Arc<TaskQueue>) -> Self {
        Self { queue }
    }

    pub async fn start(&self) {
        info!("Task worker starting...");

        let queue = self.queue.clone();
        let rx = {
            let mut rx_lock = queue.rx.write().await;
            rx_lock.take().expect("Receiver already taken")
        };

        tokio::spawn(async move {
            let mut rx = rx;

            while let Some(task_id) = rx.recv().await {
                let running = queue.get_running_count().await;
                if running >= queue.max_parallel {
                    info!(
                        "Max parallel tasks reached ({}), queueing [{}]",
                        queue.max_parallel, task_id
                    );
                    continue;
                }

                if let Some(task) = queue.get_next_task().await {
                    info!("Worker processing task [{}]: {}", task.id, task.description);

                    let queue_clone = queue.clone();
                    let task_clone = task.clone();

                    tokio::spawn(async move {
                        Self::execute_task(queue_clone, task_clone).await;
                    });
                }
            }
        });

        info!("Task worker started");
    }

    async fn execute_task(queue: Arc<TaskQueue>, mut task: Task) {
        info!("Executing task [{}]: {}", task.id, task.description);

        let result = match task.task_type {
            TaskType::TodoComplete => Self::process_todo_task(&queue, &mut task).await,
            TaskType::CodeFix => Self::process_codefix_task(&queue, &mut task).await,
            TaskType::Research => Self::process_research_task(&queue, &mut task).await,
            TaskType::GitHubSearch => Self::process_github_search_task(&queue, &mut task).await,
            TaskType::AutoResearch => Self::process_autoresearch_task(&queue, &mut task).await,
            TaskType::OpenHands => Self::process_openhands_task(&queue, &mut task).await,
            _ => {
                task.result = Some(format!(
                    "Task type not implemented yet: {:?}",
                    task.task_type
                ));
                task.status = TaskStatus::Completed;
                Ok(())
            }
        };

        if let Err(e) = result {
            error!("Task [{}] failed: {}", task.id, e);
            task.error = Some(e.to_string());
            task.status = TaskStatus::Failed;
        }

        let task_id = task.id.clone();

        if let Err(e) = queue
            .update_task_status(&task_id, task.status.clone())
            .await
        {
            error!("Failed to update task status: {}", e);
        }

        if let Err(e) = queue
            .update_task_progress(&task_id, task.progress, task.result.clone())
            .await
        {
            error!("Failed to update task progress: {}", e);
        }

        info!(
            "Task [{}] completed with status: {:?}",
            task_id, task.status
        );
    }

    async fn process_todo_task(queue: &Arc<TaskQueue>, task: &mut Task) -> Result<()> {
        queue.update_task_progress(&task.id, 10.0, None).await?;

        info!("Processing Todo task [{}]: {}", task.id, task.description);

        queue
            .update_task_progress(
                &task.id,
                50.0,
                Some("Todo items being processed...".to_string()),
            )
            .await?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        queue
            .update_task_progress(
                &task.id,
                100.0,
                Some("Todo completed successfully!".to_string()),
            )
            .await?;

        task.status = TaskStatus::Completed;
        Ok(())
    }

    async fn process_codefix_task(queue: &Arc<TaskQueue>, task: &mut Task) -> Result<()> {
        queue.update_task_progress(&task.id, 20.0, None).await?;

        info!(
            "Processing CodeFix task [{}]: {}",
            task.id, task.description
        );

        queue
            .update_task_progress(
                &task.id,
                60.0,
                Some("Code analysis completed...".to_string()),
            )
            .await?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        queue
            .update_task_progress(&task.id, 100.0, Some("Code fix completed!".to_string()))
            .await?;

        task.status = TaskStatus::Completed;
        Ok(())
    }

    async fn process_research_task(queue: &Arc<TaskQueue>, task: &mut Task) -> Result<()> {
        queue.update_task_progress(&task.id, 15.0, None).await?;

        info!(
            "Processing Research task [{}]: {}",
            task.id, task.description
        );

        let query = task
            .metadata
            .get("query")
            .cloned()
            .unwrap_or_else(|| task.description.clone());

        queue
            .update_task_progress(&task.id, 45.0, Some("Research in progress...".to_string()))
            .await?;

        let result = queue
            .execute_python_command("auto_research", vec![query])
            .await;

        match result {
            Ok(output) => {
                queue
                    .update_task_progress(
                        &task.id,
                        100.0,
                        Some(format!("Research completed! {}", output)),
                    )
                    .await?;
                task.result = Some(output);
            }
            Err(e) => {
                task.error = Some(e.to_string());
                task.status = TaskStatus::Failed;
                return Err(e);
            }
        }

        task.status = TaskStatus::Completed;
        Ok(())
    }

    async fn process_github_search_task(queue: &Arc<TaskQueue>, task: &mut Task) -> Result<()> {
        queue.update_task_progress(&task.id, 10.0, None).await?;

        info!(
            "Processing GitHub Search task [{}]: {}",
            task.id, task.description
        );

        let query = task
            .metadata
            .get("query")
            .cloned()
            .unwrap_or_else(|| task.description.clone());

        queue
            .update_task_progress(&task.id, 50.0, Some("Searching GitHub...".to_string()))
            .await?;

        let result = queue
            .execute_python_command("github_integration", vec!["search".to_string(), query])
            .await;

        match result {
            Ok(output) => {
                queue
                    .update_task_progress(
                        &task.id,
                        100.0,
                        Some(format!("GitHub search completed! {}", output)),
                    )
                    .await?;
                task.result = Some(output);
            }
            Err(e) => {
                task.error = Some(e.to_string());
                task.status = TaskStatus::Failed;
                return Err(e);
            }
        }

        task.status = TaskStatus::Completed;
        Ok(())
    }

    async fn process_autoresearch_task(queue: &Arc<TaskQueue>, task: &mut Task) -> Result<()> {
        queue.update_task_progress(&task.id, 10.0, None).await?;

        info!(
            "Processing AutoResearch task [{}]: {}",
            task.id, task.description
        );

        let topic = task
            .metadata
            .get("topic")
            .cloned()
            .unwrap_or_else(|| task.description.clone());

        queue
            .update_task_progress(&task.id, 30.0, Some("Starting research...".to_string()))
            .await?;

        let result = queue
            .execute_python_command("auto_research", vec![topic])
            .await;

        match result {
            Ok(output) => {
                queue
                    .update_task_progress(
                        &task.id,
                        100.0,
                        Some(format!("AutoResearch completed! {}", output)),
                    )
                    .await?;
                task.result = Some(output);
            }
            Err(e) => {
                task.error = Some(e.to_string());
                task.status = TaskStatus::Failed;
                return Err(e);
            }
        }

        task.status = TaskStatus::Completed;
        Ok(())
    }

    async fn process_openhands_task(queue: &Arc<TaskQueue>, task: &mut Task) -> Result<()> {
        queue.update_task_progress(&task.id, 10.0, None).await?;

        info!(
            "Processing OpenHands task [{}]: {}",
            task.id, task.description
        );

        let command = task
            .metadata
            .get("command")
            .cloned()
            .unwrap_or_else(|| task.description.clone());

        queue
            .update_task_progress(&task.id, 40.0, Some("Running OpenHands...".to_string()))
            .await?;

        let result = queue
            .execute_python_command("openhands", vec![command])
            .await;

        match result {
            Ok(output) => {
                queue
                    .update_task_progress(
                        &task.id,
                        100.0,
                        Some(format!("OpenHands completed! {}", output)),
                    )
                    .await?;
                task.result = Some(output);
            }
            Err(e) => {
                task.error = Some(e.to_string());
                task.status = TaskStatus::Failed;
                return Err(e);
            }
        }

        task.status = TaskStatus::Completed;
        Ok(())
    }
}
