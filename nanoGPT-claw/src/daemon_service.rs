//! Daemon service module placeholder

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub task_type: TaskType,
    pub description: String,
    pub status: TaskStatus,
    pub progress: f64,
    pub result: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskType {
    TodoComplete,
    CodeFix,
    Research,
    Benchmark,
    GitHubSearch,
    AutoResearch,
    OpenHands,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl Task {
    pub fn new(task_type: TaskType, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            task_type,
            description,
            status: TaskStatus::Pending,
            progress: 0.0,
            result: None,
            error: None,
            created_at: now,
            started_at: None,
            completed_at: None,
        }
    }
}

pub struct TaskQueue {
    tasks: Arc<RwLock<VecDeque<Task>>>,
}

impl TaskQueue {
    pub fn new(_capacity: usize) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub async fn enqueue(&self, task: Task) {
        self.tasks.write().await.push_back(task);
    }

    pub async fn dequeue(&self) -> Option<Task> {
        self.tasks.write().await.pop_front()
    }

    pub async fn len(&self) -> usize {
        self.tasks.read().await.len()
    }

    pub async fn add_task(&self, task: Task) -> Result<String, String> {
        let task_id = task.id.clone();
        self.enqueue(task).await;
        Ok(task_id)
    }

    pub async fn list_tasks(&self) -> Vec<Task> {
        self.tasks.read().await.iter().cloned().collect()
    }

    pub async fn get_task(&self, id: &str) -> Option<Task> {
        self.tasks.read().await.iter().find(|t| t.id == id).cloned()
    }

    pub async fn cancel_task(&self, id: &str) -> Result<(), String> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
            task.status = TaskStatus::Cancelled;
            Ok(())
        } else {
            Err(format!("Task {} not found", id))
        }
    }
}

pub static TASK_QUEUE: Lazy<Arc<TaskQueue>> = Lazy::new(|| Arc::new(TaskQueue::new(4)));

pub struct TaskWorker {
    queue: Arc<TaskQueue>,
}

impl TaskWorker {
    pub fn new(queue: Arc<TaskQueue>) -> Self {
        Self { queue }
    }

    pub async fn start(&self) {
        loop {
            if let Some(task) = self.queue.dequeue().await {
                tracing::info!("Processing task: {}", task.id);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

impl Default for TaskWorker {
    fn default() -> Self {
        Self::new(TASK_QUEUE.clone())
    }
}
