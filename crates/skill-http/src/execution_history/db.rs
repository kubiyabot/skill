//! Database implementation for execution history persistence

use std::path::Path;
use anyhow::{Context, Result};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions, Row};
use chrono::{DateTime, Utc};

use crate::types::{ExecutionHistoryEntry, ExecutionStatus};

/// SQLite-based execution history database
pub struct ExecutionHistoryDb {
    pool: SqlitePool,
}

impl ExecutionHistoryDb {
    /// Create a new execution history database
    ///
    /// # Arguments
    /// * `db_path` - Path to the SQLite database file
    ///
    /// # Example
    /// ```no_run
    /// # use skill_http::execution_history::ExecutionHistoryDb;
    /// # async fn example() -> anyhow::Result<()> {
    /// let db = ExecutionHistoryDb::new("~/.skill-engine/execution-history.db").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(db_path: &str) -> Result<Self> {
        // Expand home directory
        let db_path = shellexpand::tilde(db_path).to_string();

        // Ensure parent directory exists
        if let Some(parent) = Path::new(&db_path).parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create execution history database directory")?;
        }

        // Build connection URL
        let url = if db_path == ":memory:" {
            "sqlite::memory:".to_string()
        } else {
            format!("sqlite:{}?mode=rwc", db_path)
        };

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(&url)
            .await
            .context("Failed to connect to execution history database")?;

        let db = Self { pool };

        // Initialize database schema
        db.setup().await?;

        Ok(db)
    }

    /// Get the connection pool (for advanced usage)
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Initialize database schema with tables and indexes
    async fn setup(&self) -> Result<()> {
        // Create execution_history table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS execution_history (
                id TEXT PRIMARY KEY,
                skill TEXT NOT NULL,
                tool TEXT NOT NULL,
                instance TEXT NOT NULL,
                status TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                started_at TEXT NOT NULL,
                error TEXT,
                output TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create execution_history table")?;

        // Create indexes for common queries
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_execution_history_started_at
            ON execution_history(started_at DESC);
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create started_at index")?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_execution_history_skill
            ON execution_history(skill);
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create skill index")?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_execution_history_status
            ON execution_history(status);
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create status index")?;

        Ok(())
    }

    /// Add an execution to history
    pub async fn add_execution(&self, entry: &ExecutionHistoryEntry) -> Result<()> {
        let status_str = match entry.status {
            ExecutionStatus::Pending => "pending",
            ExecutionStatus::Running => "running",
            ExecutionStatus::Success => "success",
            ExecutionStatus::Failed => "failed",
            ExecutionStatus::Timeout => "timeout",
            ExecutionStatus::Cancelled => "cancelled",
        };

        sqlx::query(
            r#"
            INSERT INTO execution_history (
                id, skill, tool, instance, status, duration_ms, started_at, error, output
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&entry.id)
        .bind(&entry.skill)
        .bind(&entry.tool)
        .bind(&entry.instance)
        .bind(status_str)
        .bind(entry.duration_ms as i64)
        .bind(entry.started_at.to_rfc3339())
        .bind(&entry.error)
        .bind(&entry.output)
        .execute(&self.pool)
        .await
        .context("Failed to insert execution history entry")?;

        Ok(())
    }

    /// Get execution by ID
    pub async fn get_execution(&self, id: &str) -> Result<Option<ExecutionHistoryEntry>> {
        let row = sqlx::query(
            r#"
            SELECT id, skill, tool, instance, status, duration_ms, started_at, error, output
            FROM execution_history
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query execution history")?;

        match row {
            Some(row) => Ok(Some(row_to_entry(row)?)),
            None => Ok(None),
        }
    }

    /// List executions with pagination
    pub async fn list_executions(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ExecutionHistoryEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT id, skill, tool, instance, status, duration_ms, started_at, error, output
            FROM execution_history
            ORDER BY started_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list execution history")?;

        rows.into_iter()
            .map(row_to_entry)
            .collect::<Result<Vec<_>>>()
    }

    /// List executions for a specific skill
    pub async fn list_by_skill(
        &self,
        skill: &str,
        limit: usize,
    ) -> Result<Vec<ExecutionHistoryEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT id, skill, tool, instance, status, duration_ms, started_at, error, output
            FROM execution_history
            WHERE skill = ?
            ORDER BY started_at DESC
            LIMIT ?
            "#,
        )
        .bind(skill)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list execution history by skill")?;

        rows.into_iter()
            .map(row_to_entry)
            .collect::<Result<Vec<_>>>()
    }

    /// List executions by status
    pub async fn list_by_status(
        &self,
        status: &ExecutionStatus,
        limit: usize,
    ) -> Result<Vec<ExecutionHistoryEntry>> {
        let status_str = match status {
            ExecutionStatus::Pending => "pending",
            ExecutionStatus::Running => "running",
            ExecutionStatus::Success => "success",
            ExecutionStatus::Failed => "failed",
            ExecutionStatus::Timeout => "timeout",
            ExecutionStatus::Cancelled => "cancelled",
        };

        let rows = sqlx::query(
            r#"
            SELECT id, skill, tool, instance, status, duration_ms, started_at, error, output
            FROM execution_history
            WHERE status = ?
            ORDER BY started_at DESC
            LIMIT ?
            "#,
        )
        .bind(status_str)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list execution history by status")?;

        rows.into_iter()
            .map(row_to_entry)
            .collect::<Result<Vec<_>>>()
    }

    /// Get total count of executions
    pub async fn count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM execution_history")
            .fetch_one(&self.pool)
            .await
            .context("Failed to count execution history")?;

        Ok(row.get("count"))
    }

    /// Delete execution by ID
    pub async fn delete_execution(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM execution_history WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete execution history entry")?;

        Ok(())
    }

    /// Clear all execution history
    pub async fn clear_all(&self) -> Result<()> {
        sqlx::query("DELETE FROM execution_history")
            .execute(&self.pool)
            .await
            .context("Failed to clear execution history")?;

        Ok(())
    }

    /// Delete old executions, keeping only the most recent N entries
    pub async fn prune(&self, keep_count: usize) -> Result<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM execution_history
            WHERE id NOT IN (
                SELECT id FROM execution_history
                ORDER BY started_at DESC
                LIMIT ?
            )
            "#,
        )
        .bind(keep_count as i64)
        .execute(&self.pool)
        .await
        .context("Failed to prune execution history")?;

        Ok(result.rows_affected() as usize)
    }

    /// Get statistics
    pub async fn get_stats(&self) -> Result<ExecutionStats> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) as success_count,
                SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed_count,
                AVG(duration_ms) as avg_duration_ms
            FROM execution_history
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get execution stats")?;

        Ok(ExecutionStats {
            total: row.get("total"),
            success_count: row.get("success_count"),
            failed_count: row.get("failed_count"),
            avg_duration_ms: row.get::<Option<f64>, _>("avg_duration_ms").unwrap_or(0.0),
        })
    }
}

/// Convert database row to ExecutionHistoryEntry
fn row_to_entry(row: sqlx::sqlite::SqliteRow) -> Result<ExecutionHistoryEntry> {
    let status_str: String = row.get("status");
    let status = match status_str.as_str() {
        "pending" => ExecutionStatus::Pending,
        "running" => ExecutionStatus::Running,
        "success" => ExecutionStatus::Success,
        "failed" => ExecutionStatus::Failed,
        "timeout" => ExecutionStatus::Timeout,
        "cancelled" => ExecutionStatus::Cancelled,
        _ => ExecutionStatus::Failed,
    };

    let started_at_str: String = row.get("started_at");
    let started_at = DateTime::parse_from_rfc3339(&started_at_str)
        .context("Failed to parse started_at timestamp")?
        .with_timezone(&Utc);

    Ok(ExecutionHistoryEntry {
        id: row.get("id"),
        skill: row.get("skill"),
        tool: row.get("tool"),
        instance: row.get("instance"),
        status,
        duration_ms: row.get::<i64, _>("duration_ms") as u64,
        started_at,
        error: row.get("error"),
        output: row.get("output"),
    })
}

/// Execution statistics
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total: i64,
    pub success_count: i64,
    pub failed_count: i64,
    pub avg_duration_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_execution_history_crud() -> Result<()> {
        let db = ExecutionHistoryDb::new(":memory:").await?;

        // Create test entry
        let entry = ExecutionHistoryEntry {
            id: "test-123".to_string(),
            skill: "test-skill".to_string(),
            tool: "test-tool".to_string(),
            instance: "default".to_string(),
            status: ExecutionStatus::Success,
            duration_ms: 100,
            started_at: Utc::now(),
            error: None,
            output: Some("test output".to_string()),
        };

        // Add
        db.add_execution(&entry).await?;

        // Get by ID
        let retrieved = db.get_execution("test-123").await?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test-123");

        // List
        let list = db.list_executions(10, 0).await?;
        assert_eq!(list.len(), 1);

        // Stats
        let stats = db.get_stats().await?;
        assert_eq!(stats.total, 1);
        assert_eq!(stats.success_count, 1);

        Ok(())
    }
}
