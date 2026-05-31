//! Task Module - Asynchronous Task Distribution
//!
//! Manages task lifecycle, priority queuing, and parallel execution
//! across the LLM cluster.

use std::collections::VecDeque;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Task structure
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub assigned_model: Option<String>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
}

impl Task {
    /// Create new task
    pub fn new(id: String, description: String, priority: TaskPriority) -> Self {
        Self {
            id,
            description,
            priority,
            status: TaskStatus::Pending,
            assigned_model: None,
            result: None,
            error: None,
            created_at: current_timestamp(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Mark task as running
    pub fn start(&mut self, model: &str) {
        self.status = TaskStatus::Running;
        self.assigned_model = Some(model.to_string());
        self.started_at = Some(current_timestamp());
    }

    /// Mark task as completed
    pub fn complete(&mut self, result: String) {
        self.status = TaskStatus::Completed;
        self.result = Some(result);
        self.completed_at = Some(current_timestamp());
    }

    /// Mark task as failed
    pub fn fail(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(current_timestamp());
    }

    /// Get task age in seconds
    pub fn age_seconds(&self) -> i64 {
        current_timestamp() - self.created_at
    }

    /// Get execution duration in seconds
    pub fn duration_seconds(&self) -> Option<i64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

/// Task queue with priority ordering
pub struct TaskQueue {
    queue: Arc<RwLock<VecDeque<Task>>>,
    max_size: usize,
}

impl TaskQueue {
    /// Create new task queue
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::with_capacity(max_size))),
            max_size,
        }
    }

    /// Add task to queue
    pub fn push(&self, task: Task) -> Result<(), String> {
        let mut queue = self.queue.write();
        if queue.len() >= self.max_size {
            return Err("Task queue full".to_string());
        }

        // Insert in priority order
        let pos = queue.iter()
            .position(|t| t.priority < task.priority)
            .unwrap_or(queue.len());
        queue.insert(pos, task);
        Ok(())
    }

    /// Pop next task
    pub fn pop(&self) -> Option<Task> {
        self.queue.write().pop_front()
    }

    /// Peek at next task
    pub fn peek(&self) -> Option<Task> {
        self.queue.read().front().cloned()
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.queue.read().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.queue.read().is_empty()
    }

    /// Get all tasks
    pub fn get_all(&self) -> Vec<Task> {
        self.queue.read().iter().cloned().collect()
    }

    /// Find task by ID
    pub fn find(&self, id: &str) -> Option<Task> {
        self.queue.read().iter().find(|t| t.id == id).cloned()
    }

    /// Remove task by ID
    pub fn remove(&self, id: &str) -> Option<Task> {
        let mut queue = self.queue.write();
        let pos = queue.iter().position(|t| t.id == id)?;
        Some(queue.remove(pos).unwrap())
    }

    /// Clear all tasks
    pub fn clear(&self) {
        self.queue.write().clear();
    }
}

/// Task result wrapper
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: String,
    pub status: TaskStatus,
    pub output: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

impl TaskResult {
    pub fn success(task_id: String, output: String, time_ms: u64) -> Self {
        Self {
            task_id,
            status: TaskStatus::Completed,
            output: Some(output),
            error: None,
            execution_time_ms: time_ms,
        }
    }

    pub fn failure(task_id: String, error: String, time_ms: u64) -> Self {
        Self {
            task_id,
            status: TaskStatus::Failed,
            output: None,
            error: Some(error),
            execution_time_ms: time_ms,
        }
    }

    pub fn is_success(&self) -> bool {
        self.status == TaskStatus::Completed
    }
}

// Utility
fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Task builder for fluent creation
pub struct TaskBuilder {
    id: String,
    description: String,
    priority: TaskPriority,
}

impl TaskBuilder {
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            priority: TaskPriority::Normal,
        }
    }

    pub fn priority(mut self, p: TaskPriority) -> Self {
        self.priority = p;
        self
    }

    pub fn build(self) -> Task {
        Task::new(self.id, self.description, self.priority)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("t1".to_string(), "Test task".to_string(), TaskPriority::Normal);
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_task_queue_priority() {
        let queue = TaskQueue::new(10);
        queue.push(Task::new("1".to_string(), "Low".to_string(), TaskPriority::Low)).unwrap();
        queue.push(Task::new("2".to_string(), "High".to_string(), TaskPriority::High)).unwrap();
        queue.push(Task::new("3".to_string(), "Normal".to_string(), TaskPriority::Normal)).unwrap();

        let first = queue.pop().unwrap();
        assert_eq!(first.id, "2"); // High priority first
    }
}
