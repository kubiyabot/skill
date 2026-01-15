//! Job storage abstraction
//!
//! Provides a unified interface for job queue storage backends,
//! abstracting over SQLite, PostgreSQL, and Redis implementations.

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::broadcast;

use super::config::{JobConfig, ConfigError, StorageBackend};
use super::types::{Job, JobId, JobStatus, JobProgress, JobStats};

/// Error type for storage operations
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Job not found: {0}")]
    NotFound(JobId),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Backend not available: {0}")]
    BackendNotAvailable(String),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Operation failed: {0}")]
    Operation(String),
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Job storage trait
///
/// Provides the core interface for storing and retrieving jobs.
/// Implementations must be thread-safe and support async operations.
#[async_trait]
pub trait JobStorage: Send + Sync + 'static {
    /// Get the storage backend name
    fn backend_name(&self) -> &'static str;

    /// Initialize the storage (run migrations, create tables, etc.)
    async fn setup(&self) -> StorageResult<()>;

    /// Enqueue a new job
    async fn enqueue(&self, job: Job) -> StorageResult<JobId>;

    /// Dequeue the next available job for processing
    async fn dequeue(&self, worker_id: &str) -> StorageResult<Option<Job>>;

    /// Get a job by ID
    async fn get(&self, job_id: JobId) -> StorageResult<Option<Job>>;

    /// Update a job
    async fn update(&self, job: &Job) -> StorageResult<()>;

    /// Mark a job as completed with result
    async fn complete(&self, job_id: JobId, result: Option<serde_json::Value>) -> StorageResult<()>;

    /// Mark a job as failed with error
    async fn fail(&self, job_id: JobId, error: &str) -> StorageResult<()>;

    /// Cancel a job
    async fn cancel(&self, job_id: JobId) -> StorageResult<()>;

    /// Retry a failed job
    async fn retry(&self, job_id: JobId) -> StorageResult<()>;

    /// List jobs with optional filters
    async fn list(&self, filter: JobFilter) -> StorageResult<Vec<Job>>;

    /// Get job statistics
    async fn stats(&self) -> StorageResult<JobStats>;

    /// Send heartbeat for a worker (keep job lease alive)
    async fn heartbeat(&self, worker_id: &str, job_id: JobId) -> StorageResult<()>;

    /// Re-queue orphaned jobs (jobs from dead workers)
    async fn requeue_orphaned(&self, timeout_secs: u64) -> StorageResult<usize>;

    /// Cleanup old completed/failed jobs
    async fn cleanup(&self, older_than_secs: u64) -> StorageResult<usize>;

    /// Close the storage connection
    async fn close(&self) -> StorageResult<()>;
}

/// Job filter for listing jobs
#[derive(Debug, Clone, Default)]
pub struct JobFilter {
    /// Filter by status
    pub status: Option<JobStatus>,

    /// Filter by job type (e.g., "skill_execution")
    pub job_type: Option<String>,

    /// Filter by skill ID
    pub skill_id: Option<String>,

    /// Filter by worker ID
    pub worker_id: Option<String>,

    /// Maximum number of results
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,

    /// Order by (field name)
    pub order_by: Option<String>,

    /// Descending order
    pub descending: bool,
}

impl JobFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_status(mut self, status: JobStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_skill_id(mut self, skill_id: impl Into<String>) -> Self {
        self.skill_id = Some(skill_id.into());
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn order_by(mut self, field: impl Into<String>, descending: bool) -> Self {
        self.order_by = Some(field.into());
        self.descending = descending;
        self
    }
}

/// Job queue manager
///
/// High-level interface for managing background jobs, wrapping the storage
/// and providing additional features like progress tracking and events.
pub struct JobQueue {
    storage: Arc<dyn JobStorage>,
    config: JobConfig,
    progress_tx: broadcast::Sender<JobProgress>,
}

impl JobQueue {
    /// Create a new job queue with the given storage
    pub fn new(storage: Arc<dyn JobStorage>, config: JobConfig) -> Self {
        let (progress_tx, _) = broadcast::channel(100);
        Self {
            storage,
            config,
            progress_tx,
        }
    }

    /// Get the underlying storage
    pub fn storage(&self) -> &Arc<dyn JobStorage> {
        &self.storage
    }

    /// Get configuration
    pub fn config(&self) -> &JobConfig {
        &self.config
    }

    /// Subscribe to job progress updates
    pub fn subscribe_progress(&self) -> broadcast::Receiver<JobProgress> {
        self.progress_tx.subscribe()
    }

    /// Enqueue a job
    pub async fn enqueue(&self, job: Job) -> StorageResult<JobId> {
        self.storage.enqueue(job).await
    }

    /// Get a job by ID
    pub async fn get(&self, job_id: JobId) -> StorageResult<Option<Job>> {
        self.storage.get(job_id).await
    }

    /// Report progress for a job
    pub fn report_progress(&self, progress: JobProgress) {
        let _ = self.progress_tx.send(progress);
    }

    /// List pending jobs
    pub async fn pending_jobs(&self) -> StorageResult<Vec<Job>> {
        self.storage.list(JobFilter::new().with_status(JobStatus::Pending)).await
    }

    /// List running jobs
    pub async fn running_jobs(&self) -> StorageResult<Vec<Job>> {
        self.storage.list(JobFilter::new().with_status(JobStatus::Running)).await
    }

    /// Get queue statistics
    pub async fn stats(&self) -> StorageResult<JobStats> {
        self.storage.stats().await
    }

    /// Run cleanup of old jobs
    pub async fn cleanup(&self) -> StorageResult<usize> {
        let older_than = self.config.cleanup_after.as_secs();
        self.storage.cleanup(older_than).await
    }

    /// Re-queue orphaned jobs
    pub async fn recover_orphans(&self) -> StorageResult<usize> {
        let timeout = self.config.job_timeout.as_secs();
        self.storage.requeue_orphaned(timeout).await
    }
}

/// Create storage from configuration
///
/// This is the main factory function for creating storage backends.
/// It will return an error if the required feature is not enabled.
pub async fn create_storage(config: &JobConfig) -> StorageResult<Arc<dyn JobStorage>> {
    config.validate()?;

    match config.backend {
        StorageBackend::Memory => {
            #[cfg(feature = "sqlite-storage")]
            {
                // Use in-memory SQLite for "memory" backend
                let memory_config = JobConfig {
                    connection: ":memory:".to_string(),
                    ..config.clone()
                };
                let storage = super::sqlite::SqliteJobStorage::new(&memory_config).await?;
                storage.setup().await?;
                Ok(Arc::new(storage))
            }
            #[cfg(not(feature = "sqlite-storage"))]
            {
                Err(StorageError::BackendNotAvailable(
                    "Memory backend requires 'sqlite-storage' feature".to_string()
                ))
            }
        }

        StorageBackend::Sqlite => {
            #[cfg(feature = "sqlite-storage")]
            {
                let storage = super::sqlite::SqliteJobStorage::new(config).await?;
                storage.setup().await?;
                Ok(Arc::new(storage))
            }
            #[cfg(not(feature = "sqlite-storage"))]
            {
                Err(StorageError::BackendNotAvailable(
                    "Enable 'sqlite-storage' feature for SQLite support".to_string()
                ))
            }
        }

        StorageBackend::Postgres => {
            #[cfg(feature = "postgres-storage")]
            {
                let storage = super::postgres::PostgresJobStorage::new(config).await?;
                storage.setup().await?;
                Ok(Arc::new(storage))
            }
            #[cfg(not(feature = "postgres-storage"))]
            {
                Err(StorageError::BackendNotAvailable(
                    "Enable 'postgres-storage' feature for PostgreSQL support".to_string()
                ))
            }
        }

        StorageBackend::Redis => {
            #[cfg(feature = "redis-storage")]
            {
                let storage = super::redis_backend::RedisJobStorage::new(config).await?;
                Ok(Arc::new(storage))
            }
            #[cfg(not(feature = "redis-storage"))]
            {
                Err(StorageError::BackendNotAvailable(
                    "Enable 'redis-storage' feature for Redis support".to_string()
                ))
            }
        }
    }
}

/// Create a job queue from configuration
pub async fn create_job_queue(config: JobConfig) -> StorageResult<JobQueue> {
    let storage = create_storage(&config).await?;
    Ok(JobQueue::new(storage, config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_filter() {
        let filter = JobFilter::new()
            .with_status(JobStatus::Pending)
            .with_skill_id("kubernetes")
            .with_limit(10)
            .order_by("created_at", true);

        assert_eq!(filter.status, Some(JobStatus::Pending));
        assert_eq!(filter.skill_id, Some("kubernetes".to_string()));
        assert_eq!(filter.limit, Some(10));
        assert!(filter.descending);
    }
}
