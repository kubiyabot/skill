//! SQLite storage backend for job queue
//!
//! Default local-first storage using SQLite. Provides persistent job storage
//! with minimal dependencies and easy setup.

use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};

use super::config::JobConfig;
use super::storage::{JobStorage, JobFilter, StorageError, StorageResult};
use super::types::{Job, JobId, JobStatus, JobPriority, JobType, JobStats};

/// SQLite-based job storage
pub struct SqliteJobStorage {
    pool: SqlitePool,
    config: JobConfig,
}

impl SqliteJobStorage {
    /// Create a new SQLite storage
    pub async fn new(config: &JobConfig) -> StorageResult<Self> {
        let connection = &config.connection;

        // Ensure parent directory exists for file-based databases
        if connection != ":memory:" {
            if let Some(parent) = Path::new(connection).parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    StorageError::Connection(format!("Failed to create directory: {}", e))
                })?;
            }
        }

        // Build connection URL
        let url = if connection.starts_with("sqlite:") {
            connection.clone()
        } else if connection == ":memory:" {
            "sqlite::memory:".to_string()
        } else {
            format!("sqlite:{}?mode=rwc", connection)
        };

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .map_err(|e| StorageError::Connection(e.to_string()))?;

        Ok(Self {
            pool,
            config: config.clone(),
        })
    }

    /// Get the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl JobStorage for SqliteJobStorage {
    fn backend_name(&self) -> &'static str {
        "sqlite"
    }

    async fn setup(&self) -> StorageResult<()> {
        // Create jobs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS skill_jobs (
                id TEXT PRIMARY KEY,
                job_type TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                priority INTEGER NOT NULL DEFAULT 1,
                attempts INTEGER NOT NULL DEFAULT 0,
                max_attempts INTEGER NOT NULL DEFAULT 3,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                scheduled_at TEXT,
                started_at TEXT,
                completed_at TEXT,
                worker_id TEXT,
                error TEXT,
                result TEXT,
                metadata TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        // Create indexes for common queries
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_jobs_status ON skill_jobs(status);
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_jobs_scheduled ON skill_jobs(scheduled_at);
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_jobs_worker ON skill_jobs(worker_id);
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    async fn enqueue(&self, job: Job) -> StorageResult<JobId> {
        let job_type_json = serde_json::to_string(&job.job_type)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        let metadata_json = serde_json::to_string(&job.metadata)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO skill_jobs (
                id, job_type, status, priority, attempts, max_attempts,
                created_at, updated_at, scheduled_at, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(job.id.to_string())
        .bind(job_type_json)
        .bind(job.status.to_string())
        .bind(job.priority as i32)
        .bind(job.attempts as i32)
        .bind(job.max_attempts as i32)
        .bind(job.created_at.to_rfc3339())
        .bind(job.updated_at.to_rfc3339())
        .bind(job.scheduled_at.map(|dt| dt.to_rfc3339()))
        .bind(metadata_json)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(job.id)
    }

    async fn dequeue(&self, worker_id: &str) -> StorageResult<Option<Job>> {
        let now = Utc::now().to_rfc3339();

        // Find and lock the next available job atomically
        let result = sqlx::query(
            r#"
            UPDATE skill_jobs
            SET status = 'running',
                worker_id = ?,
                started_at = ?,
                updated_at = ?,
                attempts = attempts + 1
            WHERE id = (
                SELECT id FROM skill_jobs
                WHERE status = 'pending'
                AND (scheduled_at IS NULL OR scheduled_at <= ?)
                ORDER BY priority DESC, created_at ASC
                LIMIT 1
            )
            RETURNING *
            "#,
        )
        .bind(worker_id)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        match result {
            Some(row) => Ok(Some(row_to_job(&row)?)),
            None => Ok(None),
        }
    }

    async fn get(&self, job_id: JobId) -> StorageResult<Option<Job>> {
        let result = sqlx::query(
            r#"SELECT * FROM skill_jobs WHERE id = ?"#,
        )
        .bind(job_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        match result {
            Some(row) => Ok(Some(row_to_job(&row)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, job: &Job) -> StorageResult<()> {
        let job_type_json = serde_json::to_string(&job.job_type)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        let metadata_json = serde_json::to_string(&job.metadata)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        let result_json = job.result.as_ref()
            .map(|r| serde_json::to_string(r))
            .transpose()
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        sqlx::query(
            r#"
            UPDATE skill_jobs SET
                job_type = ?,
                status = ?,
                priority = ?,
                attempts = ?,
                max_attempts = ?,
                updated_at = ?,
                scheduled_at = ?,
                started_at = ?,
                completed_at = ?,
                worker_id = ?,
                error = ?,
                result = ?,
                metadata = ?
            WHERE id = ?
            "#,
        )
        .bind(job_type_json)
        .bind(job.status.to_string())
        .bind(job.priority as i32)
        .bind(job.attempts as i32)
        .bind(job.max_attempts as i32)
        .bind(Utc::now().to_rfc3339())
        .bind(job.scheduled_at.map(|dt| dt.to_rfc3339()))
        .bind(job.started_at.map(|dt| dt.to_rfc3339()))
        .bind(job.completed_at.map(|dt| dt.to_rfc3339()))
        .bind(&job.worker_id)
        .bind(&job.error)
        .bind(result_json)
        .bind(metadata_json)
        .bind(job.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    async fn complete(&self, job_id: JobId, result: Option<serde_json::Value>) -> StorageResult<()> {
        let now = Utc::now().to_rfc3339();
        let result_json = result
            .map(|r| serde_json::to_string(&r))
            .transpose()
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        let rows = sqlx::query(
            r#"
            UPDATE skill_jobs
            SET status = 'completed',
                completed_at = ?,
                updated_at = ?,
                result = ?
            WHERE id = ?
            "#,
        )
        .bind(&now)
        .bind(&now)
        .bind(result_json)
        .bind(job_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        if rows.rows_affected() == 0 {
            return Err(StorageError::NotFound(job_id));
        }

        Ok(())
    }

    async fn fail(&self, job_id: JobId, error: &str) -> StorageResult<()> {
        let now = Utc::now().to_rfc3339();

        // Check if job can be retried
        let job = self.get(job_id).await?.ok_or(StorageError::NotFound(job_id))?;

        let new_status = if job.attempts >= job.max_attempts {
            "dead"
        } else {
            "failed"
        };

        sqlx::query(
            r#"
            UPDATE skill_jobs
            SET status = ?,
                error = ?,
                updated_at = ?,
                worker_id = NULL
            WHERE id = ?
            "#,
        )
        .bind(new_status)
        .bind(error)
        .bind(&now)
        .bind(job_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    async fn cancel(&self, job_id: JobId) -> StorageResult<()> {
        let now = Utc::now().to_rfc3339();

        let rows = sqlx::query(
            r#"
            UPDATE skill_jobs
            SET status = 'cancelled',
                updated_at = ?
            WHERE id = ? AND status IN ('pending', 'failed')
            "#,
        )
        .bind(&now)
        .bind(job_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        if rows.rows_affected() == 0 {
            return Err(StorageError::Operation(
                "Cannot cancel job that is running or already terminal".to_string()
            ));
        }

        Ok(())
    }

    async fn retry(&self, job_id: JobId) -> StorageResult<()> {
        let now = Utc::now().to_rfc3339();

        let rows = sqlx::query(
            r#"
            UPDATE skill_jobs
            SET status = 'pending',
                error = NULL,
                worker_id = NULL,
                updated_at = ?
            WHERE id = ? AND status = 'failed'
            "#,
        )
        .bind(&now)
        .bind(job_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        if rows.rows_affected() == 0 {
            return Err(StorageError::Operation(
                "Cannot retry job that is not in failed status".to_string()
            ));
        }

        Ok(())
    }

    async fn list(&self, filter: JobFilter) -> StorageResult<Vec<Job>> {
        let mut query = String::from("SELECT * FROM skill_jobs WHERE 1=1");
        let mut params: Vec<String> = Vec::new();

        if let Some(status) = &filter.status {
            query.push_str(" AND status = ?");
            params.push(status.to_string());
        }

        if let Some(skill_id) = &filter.skill_id {
            query.push_str(" AND job_type LIKE ?");
            params.push(format!("%\"skill_id\":\"{}%", skill_id));
        }

        if let Some(worker_id) = &filter.worker_id {
            query.push_str(" AND worker_id = ?");
            params.push(worker_id.clone());
        }

        // Order by
        let order_field = filter.order_by.as_deref().unwrap_or("created_at");
        let order_dir = if filter.descending { "DESC" } else { "ASC" };
        query.push_str(&format!(" ORDER BY {} {}", order_field, order_dir));

        // Pagination
        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        // Build the query dynamically
        let mut sql_query = sqlx::query(&query);
        for param in &params {
            sql_query = sql_query.bind(param);
        }

        let rows = sql_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(row_to_job(&row)?);
        }

        Ok(jobs)
    }

    async fn stats(&self) -> StorageResult<JobStats> {
        let mut stats = JobStats::default();

        // Count by status
        let rows = sqlx::query(
            r#"SELECT status, COUNT(*) as count FROM skill_jobs GROUP BY status"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        for row in rows {
            let status: String = row.get("status");
            let count: i64 = row.get("count");
            stats.by_status.insert(status, count as usize);
            stats.total += count as usize;
        }

        // Calculate success rate
        let completed = *stats.by_status.get("completed").unwrap_or(&0);
        let failed = *stats.by_status.get("failed").unwrap_or(&0);
        let dead = *stats.by_status.get("dead").unwrap_or(&0);
        let total_finished = completed + failed + dead;
        if total_finished > 0 {
            stats.success_rate = completed as f32 / total_finished as f32;
        }

        // Average execution time
        let avg_row = sqlx::query(
            r#"
            SELECT AVG(
                (julianday(completed_at) - julianday(started_at)) * 86400000
            ) as avg_ms
            FROM skill_jobs
            WHERE status = 'completed' AND started_at IS NOT NULL AND completed_at IS NOT NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        stats.avg_execution_ms = avg_row.get::<Option<f64>, _>("avg_ms")
            .map(|v| v as u64)
            .unwrap_or(0);

        // Active workers
        let workers_row = sqlx::query(
            r#"SELECT COUNT(DISTINCT worker_id) as count FROM skill_jobs WHERE status = 'running'"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        stats.active_workers = workers_row.get::<i64, _>("count") as usize;

        Ok(stats)
    }

    async fn heartbeat(&self, worker_id: &str, job_id: JobId) -> StorageResult<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE skill_jobs
            SET updated_at = ?
            WHERE id = ? AND worker_id = ? AND status = 'running'
            "#,
        )
        .bind(&now)
        .bind(job_id.to_string())
        .bind(worker_id)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    async fn requeue_orphaned(&self, timeout_secs: u64) -> StorageResult<usize> {
        let cutoff = Utc::now() - chrono::Duration::seconds(timeout_secs as i64);
        let cutoff_str = cutoff.to_rfc3339();
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            UPDATE skill_jobs
            SET status = 'pending',
                worker_id = NULL,
                updated_at = ?
            WHERE status = 'running'
            AND updated_at < ?
            "#,
        )
        .bind(&now)
        .bind(&cutoff_str)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(result.rows_affected() as usize)
    }

    async fn cleanup(&self, older_than_secs: u64) -> StorageResult<usize> {
        let cutoff = Utc::now() - chrono::Duration::seconds(older_than_secs as i64);
        let cutoff_str = cutoff.to_rfc3339();

        let result = sqlx::query(
            r#"
            DELETE FROM skill_jobs
            WHERE status IN ('completed', 'cancelled', 'dead')
            AND updated_at < ?
            "#,
        )
        .bind(&cutoff_str)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(result.rows_affected() as usize)
    }

    async fn close(&self) -> StorageResult<()> {
        self.pool.close().await;
        Ok(())
    }
}

/// Convert a database row to a Job struct
fn row_to_job(row: &sqlx::sqlite::SqliteRow) -> StorageResult<Job> {
    let id_str: String = row.get("id");
    let id = id_str.parse::<JobId>()
        .map_err(|e| StorageError::Serialization(format!("Invalid job ID: {}", e)))?;

    let job_type_json: String = row.get("job_type");
    let job_type: JobType = serde_json::from_str(&job_type_json)
        .map_err(|e| StorageError::Serialization(format!("Invalid job type: {}", e)))?;

    let status_str: String = row.get("status");
    let status = match status_str.as_str() {
        "pending" => JobStatus::Pending,
        "running" => JobStatus::Running,
        "completed" => JobStatus::Completed,
        "failed" => JobStatus::Failed,
        "cancelled" => JobStatus::Cancelled,
        "dead" => JobStatus::Dead,
        _ => return Err(StorageError::Serialization(format!("Unknown status: {}", status_str))),
    };

    let priority_int: i32 = row.get("priority");
    let priority = match priority_int {
        0 => JobPriority::Low,
        1 => JobPriority::Normal,
        2 => JobPriority::High,
        3 => JobPriority::Critical,
        _ => JobPriority::Normal,
    };

    let created_at_str: String = row.get("created_at");
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| StorageError::Serialization(format!("Invalid created_at: {}", e)))?
        .with_timezone(&Utc);

    let updated_at_str: String = row.get("updated_at");
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|e| StorageError::Serialization(format!("Invalid updated_at: {}", e)))?
        .with_timezone(&Utc);

    let scheduled_at: Option<DateTime<Utc>> = row.get::<Option<String>, _>("scheduled_at")
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let started_at: Option<DateTime<Utc>> = row.get::<Option<String>, _>("started_at")
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let completed_at: Option<DateTime<Utc>> = row.get::<Option<String>, _>("completed_at")
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let metadata_json: Option<String> = row.get("metadata");
    let metadata: HashMap<String, String> = metadata_json
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let result_json: Option<String> = row.get("result");
    let result = result_json
        .and_then(|s| serde_json::from_str(&s).ok());

    Ok(Job {
        id,
        job_type,
        status,
        priority,
        attempts: row.get::<i32, _>("attempts") as u32,
        max_attempts: row.get::<i32, _>("max_attempts") as u32,
        created_at,
        updated_at,
        scheduled_at,
        started_at,
        completed_at,
        worker_id: row.get("worker_id"),
        error: row.get("error"),
        result,
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_storage() -> SqliteJobStorage {
        let config = JobConfig::memory();
        let storage = SqliteJobStorage::new(&config).await.unwrap();
        storage.setup().await.unwrap();
        storage
    }

    #[tokio::test]
    async fn test_enqueue_and_get() {
        let storage = create_test_storage().await;

        let job = Job::skill_execution("kubernetes", "apply", serde_json::json!({"file": "test.yaml"}));
        let job_id = job.id;

        storage.enqueue(job).await.unwrap();

        let retrieved = storage.get(job_id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, job_id);
        assert_eq!(retrieved.status, JobStatus::Pending);
    }

    #[tokio::test]
    async fn test_dequeue() {
        let storage = create_test_storage().await;

        let job = Job::skill_execution("kubernetes", "apply", serde_json::json!({}));
        let job_id = job.id;
        storage.enqueue(job).await.unwrap();

        let dequeued = storage.dequeue("worker-1").await.unwrap().unwrap();
        assert_eq!(dequeued.id, job_id);
        assert_eq!(dequeued.status, JobStatus::Running);
        assert_eq!(dequeued.worker_id, Some("worker-1".to_string()));
        assert_eq!(dequeued.attempts, 1);
    }

    #[tokio::test]
    async fn test_complete() {
        let storage = create_test_storage().await;

        let job = Job::skill_execution("test", "run", serde_json::json!({}));
        let job_id = job.id;
        storage.enqueue(job).await.unwrap();
        storage.dequeue("worker-1").await.unwrap();

        let result = serde_json::json!({"success": true});
        storage.complete(job_id, Some(result.clone())).await.unwrap();

        let job = storage.get(job_id).await.unwrap().unwrap();
        assert_eq!(job.status, JobStatus::Completed);
        assert_eq!(job.result, Some(result));
    }

    #[tokio::test]
    async fn test_fail_and_retry() {
        let storage = create_test_storage().await;

        let job = Job::skill_execution("test", "run", serde_json::json!({}))
            .with_max_attempts(3);
        let job_id = job.id;
        storage.enqueue(job).await.unwrap();
        storage.dequeue("worker-1").await.unwrap();

        storage.fail(job_id, "Test error").await.unwrap();

        let job = storage.get(job_id).await.unwrap().unwrap();
        assert_eq!(job.status, JobStatus::Failed);
        assert_eq!(job.error, Some("Test error".to_string()));

        // Retry
        storage.retry(job_id).await.unwrap();
        let job = storage.get(job_id).await.unwrap().unwrap();
        assert_eq!(job.status, JobStatus::Pending);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let storage = create_test_storage().await;

        // Enqueue jobs in reverse priority order
        let low = Job::skill_execution("test", "low", serde_json::json!({}))
            .with_priority(JobPriority::Low);
        let high = Job::skill_execution("test", "high", serde_json::json!({}))
            .with_priority(JobPriority::High);
        let normal = Job::skill_execution("test", "normal", serde_json::json!({}))
            .with_priority(JobPriority::Normal);

        storage.enqueue(low).await.unwrap();
        storage.enqueue(high.clone()).await.unwrap();
        storage.enqueue(normal).await.unwrap();

        // High priority should be dequeued first
        let first = storage.dequeue("worker-1").await.unwrap().unwrap();
        assert_eq!(first.priority, JobPriority::High);
    }

    #[tokio::test]
    async fn test_list_with_filter() {
        let storage = create_test_storage().await;

        // Create some jobs
        for i in 0..5 {
            let job = Job::skill_execution("kubernetes", &format!("tool-{}", i), serde_json::json!({}));
            storage.enqueue(job).await.unwrap();
        }

        // Complete some
        for _ in 0..2 {
            if let Some(job) = storage.dequeue("worker-1").await.unwrap() {
                storage.complete(job.id, None).await.unwrap();
            }
        }

        // List pending
        let pending = storage.list(JobFilter::new().with_status(JobStatus::Pending)).await.unwrap();
        assert_eq!(pending.len(), 3);

        // List completed
        let completed = storage.list(JobFilter::new().with_status(JobStatus::Completed)).await.unwrap();
        assert_eq!(completed.len(), 2);
    }

    #[tokio::test]
    async fn test_stats() {
        let storage = create_test_storage().await;

        // Create and process some jobs
        for i in 0..5 {
            let job = Job::skill_execution("test", &format!("tool-{}", i), serde_json::json!({}));
            storage.enqueue(job).await.unwrap();
        }

        let stats = storage.stats().await.unwrap();
        assert_eq!(stats.total, 5);
        assert_eq!(*stats.by_status.get("pending").unwrap_or(&0), 5);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let storage = create_test_storage().await;

        let job = Job::skill_execution("test", "run", serde_json::json!({}));
        let job_id = job.id;
        storage.enqueue(job).await.unwrap();
        storage.dequeue("worker-1").await.unwrap();
        storage.complete(job_id, None).await.unwrap();

        // Cleanup with 0 seconds should remove the job
        let cleaned = storage.cleanup(0).await.unwrap();
        assert_eq!(cleaned, 1);

        // Job should be gone
        let job = storage.get(job_id).await.unwrap();
        assert!(job.is_none());
    }
}
