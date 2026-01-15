//! Worker pool and job execution
//!
//! Provides configurable worker pools for processing background jobs
//! with concurrency controls, retries, and graceful shutdown.

use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::task::JoinHandle;
use tracing::{info, warn, error, debug, instrument};

use super::storage::{JobStorage, StorageResult, StorageError};
use super::types::{Job, JobId, JobStatus, JobType, JobProgress};

/// Worker pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    /// Number of concurrent workers
    #[serde(default = "default_num_workers")]
    pub num_workers: usize,

    /// Maximum concurrent jobs per worker
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,

    /// Job timeout in seconds
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Base retry delay in seconds (exponential backoff)
    #[serde(default = "default_retry_delay_secs")]
    pub retry_delay_secs: u64,

    /// Poll interval for checking new jobs
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,

    /// Enable worker heartbeats
    #[serde(default = "default_heartbeat_enabled")]
    pub heartbeat_enabled: bool,

    /// Heartbeat interval in seconds
    #[serde(default = "default_heartbeat_interval_secs")]
    pub heartbeat_interval_secs: u64,
}

fn default_num_workers() -> usize { 4 }
fn default_concurrency() -> usize { 2 }
fn default_timeout_secs() -> u64 { 300 }
fn default_max_retries() -> u32 { 3 }
fn default_retry_delay_secs() -> u64 { 5 }
fn default_poll_interval_ms() -> u64 { 500 }
fn default_heartbeat_enabled() -> bool { true }
fn default_heartbeat_interval_secs() -> u64 { 30 }

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            num_workers: default_num_workers(),
            concurrency: default_concurrency(),
            timeout_secs: default_timeout_secs(),
            max_retries: default_max_retries(),
            retry_delay_secs: default_retry_delay_secs(),
            poll_interval_ms: default_poll_interval_ms(),
            heartbeat_enabled: default_heartbeat_enabled(),
            heartbeat_interval_secs: default_heartbeat_interval_secs(),
        }
    }
}

impl WorkerConfig {
    /// Create a new worker configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set number of workers
    pub fn with_workers(mut self, n: usize) -> Self {
        self.num_workers = n;
        self
    }

    /// Set concurrency per worker
    pub fn with_concurrency(mut self, n: usize) -> Self {
        self.concurrency = n;
        self
    }

    /// Set job timeout
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Set max retries
    pub fn with_max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }
}

/// Job handler trait
///
/// Implement this trait to create custom job handlers.
#[async_trait::async_trait]
pub trait JobHandler: Send + Sync + 'static {
    /// Handle a job
    async fn handle(&self, job: &Job, ctx: &WorkerContext) -> Result<serde_json::Value, JobError>;

    /// Check if this handler can process the given job type
    fn can_handle(&self, job_type: &JobType) -> bool;

    /// Get handler name
    fn name(&self) -> &str;
}

/// Job execution error
#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error("Job execution failed: {0}")]
    Execution(String),

    #[error("Job timed out after {0} seconds")]
    Timeout(u64),

    #[error("Job was cancelled")]
    Cancelled,

    #[error("Invalid job type: {0}")]
    InvalidJobType(String),

    #[error("Handler error: {0}")]
    Handler(#[source] anyhow::Error),
}

/// Context provided to job handlers
pub struct WorkerContext {
    /// Worker ID
    pub worker_id: String,

    /// Progress channel for reporting updates
    progress_tx: mpsc::Sender<JobProgress>,
}

impl WorkerContext {
    /// Report progress on current job
    pub async fn report_progress(&self, job_id: JobId, percentage: u8, step: &str) {
        let progress = JobProgress::new(job_id, percentage, step);
        let _ = self.progress_tx.send(progress).await;
    }

    /// Report progress with details
    pub async fn report_progress_with_details(
        &self,
        job_id: JobId,
        percentage: u8,
        step: &str,
        details: &str,
    ) {
        let progress = JobProgress::new(job_id, percentage, step)
            .with_details(details);
        let _ = self.progress_tx.send(progress).await;
    }
}

/// Worker pool state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolState {
    /// Pool is created but not started
    Created,
    /// Pool is running
    Running,
    /// Pool is shutting down
    ShuttingDown,
    /// Pool has stopped
    Stopped,
}

/// Worker pool for processing background jobs
pub struct WorkerPool {
    /// Storage backend
    storage: Arc<dyn JobStorage>,

    /// Worker configuration
    config: WorkerConfig,

    /// Job handlers
    handlers: Arc<Vec<Box<dyn JobHandler>>>,

    /// Pool state
    state: Arc<RwLock<PoolState>>,

    /// Worker task handles
    workers: Arc<RwLock<Vec<JoinHandle<()>>>>,

    /// Shutdown signal sender
    shutdown_tx: broadcast::Sender<()>,

    /// Progress channel sender
    progress_tx: mpsc::Sender<JobProgress>,

    /// Progress channel receiver (for consumers)
    progress_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<JobProgress>>>,
}

impl WorkerPool {
    /// Create a new worker pool
    pub fn new(storage: Arc<dyn JobStorage>, config: WorkerConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        let (progress_tx, progress_rx) = mpsc::channel(100);

        Self {
            storage,
            config,
            handlers: Arc::new(Vec::new()),
            state: Arc::new(RwLock::new(PoolState::Created)),
            workers: Arc::new(RwLock::new(Vec::new())),
            shutdown_tx,
            progress_tx,
            progress_rx: Arc::new(tokio::sync::Mutex::new(progress_rx)),
        }
    }

    /// Add a job handler
    pub fn with_handler(mut self, handler: Box<dyn JobHandler>) -> Self {
        Arc::get_mut(&mut self.handlers)
            .expect("handlers not shared yet")
            .push(handler);
        self
    }

    /// Start the worker pool
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<(), WorkerPoolError> {
        let mut state = self.state.write().await;
        if *state != PoolState::Created {
            return Err(WorkerPoolError::InvalidState(
                "Pool can only be started from Created state".to_string()
            ));
        }

        info!(
            "Starting worker pool with {} workers",
            self.config.num_workers
        );

        let mut workers = self.workers.write().await;

        for i in 0..self.config.num_workers {
            let worker_id = format!("worker-{}", i);
            let handle = self.spawn_worker(worker_id).await;
            workers.push(handle);
        }

        *state = PoolState::Running;
        info!("Worker pool started");

        Ok(())
    }

    /// Spawn a single worker
    async fn spawn_worker(&self, worker_id: String) -> JoinHandle<()> {
        let storage = self.storage.clone();
        let config = self.config.clone();
        let handlers = self.handlers.clone();
        let state = self.state.clone();
        let shutdown_rx = self.shutdown_tx.subscribe();
        let progress_tx = self.progress_tx.clone();

        tokio::spawn(async move {
            let ctx = WorkerContext {
                worker_id: worker_id.clone(),
                progress_tx,
            };

            worker_loop(
                worker_id,
                storage,
                config,
                handlers,
                state,
                ctx,
                shutdown_rx,
            ).await;
        })
    }

    /// Gracefully shutdown the worker pool
    #[instrument(skip(self))]
    pub async fn shutdown(&self, timeout: Duration) -> Result<(), WorkerPoolError> {
        // Check and update state
        {
            let mut state = self.state.write().await;
            if *state != PoolState::Running {
                return Err(WorkerPoolError::InvalidState(
                    "Pool can only be shutdown from Running state".to_string()
                ));
            }
            info!("Initiating graceful shutdown");
            *state = PoolState::ShuttingDown;
        }

        // Signal all workers to stop
        let _ = self.shutdown_tx.send(());

        // Small delay to allow workers to see the shutdown signal
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Wait for timeout (workers should exit on their own by checking state)
        tokio::time::sleep(timeout.min(Duration::from_millis(100))).await;

        // Mark as stopped
        {
            let mut state = self.state.write().await;
            *state = PoolState::Stopped;
        }

        info!("Worker pool shutdown complete");
        Ok(())
    }

    /// Get current pool state
    pub async fn state(&self) -> PoolState {
        *self.state.read().await
    }

    /// Get progress receiver for monitoring
    pub fn progress_receiver(&self) -> Arc<tokio::sync::Mutex<mpsc::Receiver<JobProgress>>> {
        self.progress_rx.clone()
    }

    /// Get statistics
    pub async fn stats(&self) -> StorageResult<WorkerPoolStats> {
        let job_stats = self.storage.stats().await?;
        let state = *self.state.read().await;
        let workers = self.workers.read().await;

        Ok(WorkerPoolStats {
            state,
            num_workers: workers.len(),
            pending_jobs: *job_stats.by_status.get("pending").unwrap_or(&0),
            running_jobs: *job_stats.by_status.get("running").unwrap_or(&0),
            completed_jobs: *job_stats.by_status.get("completed").unwrap_or(&0),
            failed_jobs: *job_stats.by_status.get("failed").unwrap_or(&0),
            success_rate: job_stats.success_rate,
            avg_execution_ms: job_stats.avg_execution_ms,
        })
    }
}

/// Worker pool statistics
#[derive(Debug, Clone)]
pub struct WorkerPoolStats {
    pub state: PoolState,
    pub num_workers: usize,
    pub pending_jobs: usize,
    pub running_jobs: usize,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
    pub success_rate: f32,
    pub avg_execution_ms: u64,
}

/// Worker pool error
#[derive(Debug, thiserror::Error)]
pub enum WorkerPoolError {
    #[error("Invalid pool state: {0}")]
    InvalidState(String),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Worker error: {0}")]
    Worker(String),
}

/// Main worker loop
async fn worker_loop(
    worker_id: String,
    storage: Arc<dyn JobStorage>,
    config: WorkerConfig,
    handlers: Arc<Vec<Box<dyn JobHandler>>>,
    state: Arc<RwLock<PoolState>>,
    ctx: WorkerContext,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let poll_interval = Duration::from_millis(config.poll_interval_ms);
    let timeout = Duration::from_secs(config.timeout_secs);

    debug!(worker_id = %worker_id, "Worker started");

    loop {
        // Check shutdown signal first (non-blocking)
        match shutdown_rx.try_recv() {
            Ok(_) | Err(broadcast::error::TryRecvError::Closed) => {
                debug!(worker_id = %worker_id, "Worker received shutdown signal");
                break;
            }
            Err(broadcast::error::TryRecvError::Empty) => {}
            Err(broadcast::error::TryRecvError::Lagged(_)) => {}
        }

        // Check state
        {
            let current_state = *state.read().await;
            if current_state == PoolState::ShuttingDown || current_state == PoolState::Stopped {
                debug!(worker_id = %worker_id, "Worker shutting down via state");
                break;
            }
        }

        // Try to dequeue and process a job
        match storage.dequeue(&worker_id).await {
            Ok(Some(job)) => {
                process_job(&worker_id, &job, &storage, &handlers, &ctx, timeout).await;
            }
            Ok(None) => {
                // No jobs available, wait before polling again
                tokio::time::sleep(poll_interval).await;
            }
            Err(e) => {
                error!(worker_id = %worker_id, error = %e, "Failed to dequeue job");
                tokio::time::sleep(poll_interval).await;
            }
        }
    }

    debug!(worker_id = %worker_id, "Worker stopped");
}

/// Process a single job
async fn process_job(
    worker_id: &str,
    job: &Job,
    storage: &Arc<dyn JobStorage>,
    handlers: &Arc<Vec<Box<dyn JobHandler>>>,
    ctx: &WorkerContext,
    timeout: Duration,
) {
    debug!(worker_id = %worker_id, job_id = %job.id, "Processing job");

    // Find a handler for this job type
    let handler = handlers.iter().find(|h| h.can_handle(&job.job_type));

    match handler {
        Some(handler) => {
            // Execute with timeout
            let result = tokio::time::timeout(timeout, handler.handle(job, ctx)).await;

            match result {
                Ok(Ok(result)) => {
                    if let Err(e) = storage.complete(job.id, Some(result)).await {
                        error!(worker_id = %worker_id, job_id = %job.id, error = %e, "Failed to mark job as completed");
                    }
                }
                Ok(Err(e)) => {
                    error!(worker_id = %worker_id, job_id = %job.id, error = %e, "Job execution failed");
                    if let Err(e) = storage.fail(job.id, &e.to_string()).await {
                        error!(worker_id = %worker_id, job_id = %job.id, error = %e, "Failed to mark job as failed");
                    }
                }
                Err(_) => {
                    warn!(worker_id = %worker_id, job_id = %job.id, "Job timed out");
                    if let Err(e) = storage.fail(job.id, &format!("Job timed out after {} seconds", timeout.as_secs())).await {
                        error!(worker_id = %worker_id, job_id = %job.id, error = %e, "Failed to mark job as timed out");
                    }
                }
            }
        }
        None => {
            warn!(worker_id = %worker_id, job_id = %job.id, job_type = ?job.job_type, "No handler found for job type");
            if let Err(e) = storage.fail(job.id, "No handler found for job type").await {
                error!(worker_id = %worker_id, job_id = %job.id, error = %e, "Failed to mark job as failed");
            }
        }
    }
}

/// Default job handler that logs execution
pub struct LoggingJobHandler;

#[async_trait::async_trait]
impl JobHandler for LoggingJobHandler {
    async fn handle(&self, job: &Job, ctx: &WorkerContext) -> Result<serde_json::Value, JobError> {
        info!(
            job_id = %job.id,
            worker_id = %ctx.worker_id,
            job_type = ?job.job_type,
            "Executing job"
        );

        // Report progress
        ctx.report_progress(job.id, 0, "Starting").await;

        // Simulate some work based on job type
        match &job.job_type {
            JobType::SkillExecution { skill_id, tool_name, .. } => {
                ctx.report_progress_with_details(
                    job.id, 50,
                    "Executing skill",
                    &format!("Running {}:{}", skill_id, tool_name)
                ).await;

                // In a real implementation, this would call the skill executor
                tokio::time::sleep(Duration::from_millis(100)).await;

                ctx.report_progress(job.id, 100, "Completed").await;
                Ok(serde_json::json!({
                    "status": "success",
                    "skill": skill_id,
                    "tool": tool_name
                }))
            }
            JobType::ExampleGeneration { skill_id, tool_names, .. } => {
                ctx.report_progress_with_details(
                    job.id, 50,
                    "Generating examples",
                    &format!("Processing {} tools for {}", tool_names.len(), skill_id)
                ).await;

                // In a real implementation, this would call the example generator
                tokio::time::sleep(Duration::from_millis(100)).await;

                ctx.report_progress(job.id, 100, "Completed").await;
                Ok(serde_json::json!({
                    "status": "success",
                    "skill": skill_id,
                    "tools_processed": tool_names.len()
                }))
            }
            JobType::SkillIndexing { skill_id, .. } => {
                ctx.report_progress_with_details(
                    job.id, 50,
                    "Indexing skill",
                    skill_id
                ).await;

                // In a real implementation, this would call the search pipeline
                tokio::time::sleep(Duration::from_millis(100)).await;

                ctx.report_progress(job.id, 100, "Completed").await;
                Ok(serde_json::json!({
                    "status": "success",
                    "skill": skill_id
                }))
            }
            _ => {
                ctx.report_progress(job.id, 100, "Completed").await;
                Ok(serde_json::json!({"status": "success"}))
            }
        }
    }

    fn can_handle(&self, _job_type: &JobType) -> bool {
        true // Handles all job types
    }

    fn name(&self) -> &str {
        "logging"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::sqlite::SqliteJobStorage;
    use crate::jobs::config::JobConfig;

    #[tokio::test]
    async fn test_worker_config() {
        let config = WorkerConfig::new()
            .with_workers(8)
            .with_concurrency(4)
            .with_timeout(600)
            .with_max_retries(5);

        assert_eq!(config.num_workers, 8);
        assert_eq!(config.concurrency, 4);
        assert_eq!(config.timeout_secs, 600);
        assert_eq!(config.max_retries, 5);
    }

    #[tokio::test]
    async fn test_worker_config_default() {
        let config = WorkerConfig::default();
        assert_eq!(config.num_workers, 4);
        assert_eq!(config.concurrency, 2);
        assert_eq!(config.timeout_secs, 300);
        assert_eq!(config.max_retries, 3);
    }

    #[tokio::test]
    async fn test_logging_handler() {
        let handler = LoggingJobHandler;
        assert_eq!(handler.name(), "logging");
        assert!(handler.can_handle(&JobType::FullReindex));
    }

    #[tokio::test]
    async fn test_pool_state_transitions() {
        let config = JobConfig::memory();
        let storage = SqliteJobStorage::new(&config).await.unwrap();
        storage.setup().await.unwrap();

        let pool = WorkerPool::new(
            Arc::new(storage),
            WorkerConfig {
                num_workers: 0, // No workers so no background tasks
                ..WorkerConfig::default()
            }
        );

        assert_eq!(pool.state().await, PoolState::Created);

        // Can't shutdown from Created
        assert!(pool.shutdown(Duration::from_millis(100)).await.is_err());

        pool.start().await.unwrap();
        assert_eq!(pool.state().await, PoolState::Running);

        // Can't start twice
        assert!(pool.start().await.is_err());

        pool.shutdown(Duration::from_millis(100)).await.unwrap();
        assert_eq!(pool.state().await, PoolState::Stopped);
    }

    #[tokio::test]
    async fn test_job_handler_execution() {
        // Test handler directly without pool
        let handler = LoggingJobHandler;
        let (tx, _rx) = mpsc::channel(10);

        let ctx = WorkerContext {
            worker_id: "test-worker".to_string(),
            progress_tx: tx,
        };

        let job = Job::skill_execution("test-skill", "run", serde_json::json!({"input": "test"}));

        let result = handler.handle(&job, &ctx).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value["status"], "success");
        assert_eq!(value["skill"], "test-skill");
    }

    #[tokio::test]
    async fn test_process_job_function() {
        let config = JobConfig::memory();
        let storage = SqliteJobStorage::new(&config).await.unwrap();
        storage.setup().await.unwrap();
        let storage: Arc<dyn JobStorage> = Arc::new(storage);

        let handlers: Arc<Vec<Box<dyn JobHandler>>> = Arc::new(vec![
            Box::new(LoggingJobHandler) as Box<dyn JobHandler>
        ]);

        let (tx, _rx) = mpsc::channel(10);
        let ctx = WorkerContext {
            worker_id: "test".to_string(),
            progress_tx: tx,
        };

        // Enqueue and dequeue a job
        let job = Job::skill_execution("test", "run", serde_json::json!({}));
        let job_id = job.id;
        storage.enqueue(job).await.unwrap();

        let job = storage.dequeue("test").await.unwrap().unwrap();

        // Process it
        process_job("test", &job, &storage, &handlers, &ctx, Duration::from_secs(5)).await;

        // Check it completed
        let job = storage.get(job_id).await.unwrap().unwrap();
        assert_eq!(job.status, JobStatus::Completed);
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let config = JobConfig::memory();
        let storage = SqliteJobStorage::new(&config).await.unwrap();
        storage.setup().await.unwrap();

        let pool = WorkerPool::new(
            Arc::new(storage),
            WorkerConfig {
                num_workers: 0,
                ..WorkerConfig::default()
            }
        );

        pool.start().await.unwrap();

        let stats = pool.stats().await.unwrap();
        assert_eq!(stats.state, PoolState::Running);
        assert_eq!(stats.num_workers, 0);
        assert_eq!(stats.pending_jobs, 0);

        pool.shutdown(Duration::from_millis(100)).await.unwrap();
    }
}
