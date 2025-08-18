use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub task_type: TaskType,
    pub priority: u8,
    pub scheduled_time: DateTime<Utc>,
    pub dependencies: Vec<String>,
    pub max_retries: u32,
    pub retry_count: u32,
    pub timeout_seconds: u64,
    pub status: TaskStatus,
    pub result: Option<TaskResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    HealthCheck,
    Replication,
    Backup,
    Deployment,
    Monitoring,
    Analysis,
    Cleanup,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub execution_time_seconds: u64,
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Task {}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then earlier scheduled time
        other
            .priority
            .cmp(&self.priority)
            .then(self.scheduled_time.cmp(&other.scheduled_time))
    }
}

pub struct TaskScheduler {
    task_queue: Arc<RwLock<BinaryHeap<Task>>>,
    running_tasks: Arc<RwLock<HashMap<String, Task>>>,
    completed_tasks: Arc<RwLock<Vec<Task>>>,
    task_executors: Arc<RwLock<HashMap<TaskType, Box<dyn TaskExecutor>>>>,
    max_concurrent_tasks: usize,
    default_retry_delay_seconds: u64,
}

#[async_trait::async_trait]
pub trait TaskExecutor: Send + Sync {
    async fn execute(&self, task: &Task) -> Result<TaskResult>;
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            task_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(Vec::new())),
            task_executors: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent_tasks: 5,
            default_retry_delay_seconds: 30,
        }
    }

    pub async fn register_executor(&self, task_type: TaskType, executor: Box<dyn TaskExecutor>) {
        self.task_executors
            .write()
            .await
            .insert(task_type, executor);
    }

    pub async fn schedule_task(&self, mut task: Task) -> Result<()> {
        info!("Scheduling task: {} ({})", task.name, task.id);

        // Validate dependencies
        if !self.validate_dependencies(&task).await {
            return Err(anyhow::anyhow!("Task has unmet dependencies"));
        }

        task.status = TaskStatus::Pending;
        self.task_queue.write().await.push(task);

        Ok(())
    }

    pub async fn schedule_recurring_task(
        &self,
        base_task: Task,
        interval: Duration,
        count: usize,
    ) -> Result<()> {
        let mut scheduled_time = base_task.scheduled_time;

        for i in 0..count {
            let mut task = base_task.clone();
            task.id = format!("{}-{}", base_task.id, i);
            task.scheduled_time = scheduled_time;

            self.schedule_task(task).await?;
            scheduled_time += interval;
        }

        Ok(())
    }

    async fn validate_dependencies(&self, task: &Task) -> bool {
        if task.dependencies.is_empty() {
            return true;
        }

        let completed = self.completed_tasks.read().await;
        for dep_id in &task.dependencies {
            let dep_completed = completed
                .iter()
                .any(|t| t.id == *dep_id && t.status == TaskStatus::Completed);

            if !dep_completed {
                debug!("Task {} has unmet dependency: {}", task.id, dep_id);
                return false;
            }
        }

        true
    }

    pub async fn run(&self) {
        info!("Starting autonomous task scheduler");

        loop {
            // Process pending tasks
            self.process_pending_tasks().await;

            // Check running tasks for timeout
            self.check_running_tasks().await;

            // Clean up completed tasks
            self.cleanup_completed_tasks().await;

            // Wait before next cycle
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    async fn process_pending_tasks(&self) {
        let now = Utc::now();
        let running_count = self.running_tasks.read().await.len();

        if running_count >= self.max_concurrent_tasks {
            return;
        }

        let mut queue = self.task_queue.write().await;
        let mut tasks_to_run = Vec::new();

        // Get tasks that are ready to run
        while let Some(task) = queue.peek() {
            if task.scheduled_time > now {
                break; // No more tasks ready
            }

            if running_count + tasks_to_run.len() >= self.max_concurrent_tasks {
                break; // At capacity
            }

            if let Some(mut task) = queue.pop() {
                task.status = TaskStatus::Running;
                tasks_to_run.push(task);
            }
        }

        drop(queue); // Release lock

        // Execute tasks
        for task in tasks_to_run {
            self.execute_task(task).await;
        }
    }

    async fn execute_task(&self, mut task: Task) {
        let task_id = task.id.clone();
        info!("Executing task: {} ({})", task.name, task_id);

        // Add to running tasks
        self.running_tasks
            .write()
            .await
            .insert(task_id.clone(), task.clone());

        // Spawn task execution
        let executors = self.task_executors.clone();
        let running_tasks = self.running_tasks.clone();
        let completed_tasks = self.completed_tasks.clone();
        let task_queue = self.task_queue.clone();
        let retry_delay = self.default_retry_delay_seconds;

        tokio::spawn(async move {
            let start_time = Utc::now();

            // Get executor for task type
            // Get executor for task type (cannot clone Box<dyn TaskExecutor>)
            // So we'll need to check if executor exists
            let executor_exists = {
                let executors = executors.read().await;
                executors.contains_key(&task.task_type)
            };

            let result = if executor_exists {
                // Execute the task with the executor
                // We need to access the executor within the lock
                let executors = executors.read().await;
                if let Some(executor) = executors.get(&task.task_type) {
                    // Execute with timeout
                    let timeout = tokio::time::Duration::from_secs(task.timeout_seconds);
                    match tokio::time::timeout(timeout, executor.execute(&task)).await {
                        Ok(Ok(result)) => Some(result),
                        Ok(Err(e)) => {
                            error!("Task {} failed: {}", task_id, e);
                            None
                        }
                        Err(_) => {
                            error!("Task {} timed out", task_id);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                warn!("No executor found for task type: {:?}", task.task_type);
                None
            };

            let execution_time = (Utc::now() - start_time).num_seconds() as u64;

            // Update task result
            if let Some(result) = result {
                task.result = Some(result.clone());

                if result.success {
                    task.status = TaskStatus::Completed;
                    info!("Task {} completed successfully", task_id);
                } else {
                    task.status = TaskStatus::Failed;
                    warn!("Task {} failed: {}", task_id, result.message);
                }
            } else {
                task.status = TaskStatus::Failed;
                task.result = Some(TaskResult {
                    success: false,
                    message: "Task execution failed".to_string(),
                    data: None,
                    execution_time_seconds: execution_time,
                });
            }

            // Handle retry logic
            if task.status == TaskStatus::Failed && task.retry_count < task.max_retries {
                task.retry_count += 1;
                task.status = TaskStatus::Retrying;
                task.scheduled_time = Utc::now() + Duration::seconds(retry_delay as i64);

                info!(
                    "Retrying task {} (attempt {}/{})",
                    task_id, task.retry_count, task.max_retries
                );

                task_queue.write().await.push(task.clone());
            } else {
                // Move to completed
                completed_tasks.write().await.push(task);
            }

            // Remove from running
            running_tasks.write().await.remove(&task_id);
        });
    }

    async fn check_running_tasks(&self) {
        let now = Utc::now();
        let mut tasks_to_cancel = Vec::new();

        {
            let running = self.running_tasks.read().await;
            for (id, task) in running.iter() {
                let runtime = (now - task.scheduled_time).num_seconds() as u64;
                if runtime > task.timeout_seconds * 2 {
                    // Task has been running for too long
                    warn!("Task {} appears to be stuck", id);
                    tasks_to_cancel.push(id.clone());
                }
            }
        }

        // Cancel stuck tasks
        for id in tasks_to_cancel {
            if let Some(mut task) = self.running_tasks.write().await.remove(&id) {
                task.status = TaskStatus::Failed;
                task.result = Some(TaskResult {
                    success: false,
                    message: "Task cancelled due to timeout".to_string(),
                    data: None,
                    execution_time_seconds: task.timeout_seconds,
                });
                self.completed_tasks.write().await.push(task);
            }
        }
    }

    async fn cleanup_completed_tasks(&self) {
        let mut completed = self.completed_tasks.write().await;

        // Keep only last 1000 tasks or tasks from last 24 hours
        let cutoff = Utc::now() - Duration::hours(24);

        if completed.len() > 1000 {
            let drain_count = completed.len() - 1000;
            completed.drain(0..drain_count);
        }

        completed.retain(|t| t.scheduled_time > cutoff);
    }

    pub async fn get_status(&self) -> SchedulerStatus {
        SchedulerStatus {
            pending_tasks: self.task_queue.read().await.len(),
            running_tasks: self.running_tasks.read().await.len(),
            completed_tasks: self.completed_tasks.read().await.len(),
            next_task_time: self
                .task_queue
                .read()
                .await
                .peek()
                .map(|t| t.scheduled_time),
        }
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        // Check if task is pending
        {
            let mut queue = self.task_queue.write().await;
            let tasks: Vec<_> = queue.drain().collect();
            for task in tasks {
                if task.id != task_id {
                    queue.push(task);
                }
            }
        }

        // Check if task is running
        if let Some(mut task) = self.running_tasks.write().await.remove(task_id) {
            task.status = TaskStatus::Cancelled;
            self.completed_tasks.write().await.push(task);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatus {
    pub pending_tasks: usize,
    pub running_tasks: usize,
    pub completed_tasks: usize,
    pub next_task_time: Option<DateTime<Utc>>,
}

// Example executor implementations
pub struct HealthCheckExecutor;

#[async_trait::async_trait]
impl TaskExecutor for HealthCheckExecutor {
    async fn execute(&self, _task: &Task) -> Result<TaskResult> {
        // Perform health check
        Ok(TaskResult {
            success: true,
            message: "Health check completed".to_string(),
            data: Some(serde_json::json!({
                "status": "healthy",
                "timestamp": Utc::now()
            })),
            execution_time_seconds: 1,
        })
    }
}

pub struct ReplicationExecutor;

#[async_trait::async_trait]
impl TaskExecutor for ReplicationExecutor {
    async fn execute(&self, _task: &Task) -> Result<TaskResult> {
        // Trigger replication
        Ok(TaskResult {
            success: true,
            message: "Replication initiated".to_string(),
            data: None,
            execution_time_seconds: 5,
        })
    }
}
