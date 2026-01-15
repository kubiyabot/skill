//! Database implementation for search analytics

use std::path::Path;
use anyhow::{Context, Result};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use uuid::Uuid;

use super::types::*;

/// SQLite-based search analytics database
pub struct SearchAnalyticsDb {
    pool: SqlitePool,
}

impl SearchAnalyticsDb {
    /// Create a new analytics database
    ///
    /// # Arguments
    /// * `db_path` - Path to the SQLite database file
    ///
    /// # Example
    /// ```no_run
    /// # use skill_http::analytics::SearchAnalyticsDb;
    /// # async fn example() -> anyhow::Result<()> {
    /// let db = SearchAnalyticsDb::new("~/.skill-engine/analytics.db").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(db_path: &str) -> Result<Self> {
        // Expand home directory
        let db_path = shellexpand::tilde(db_path).to_string();

        // Ensure parent directory exists
        if let Some(parent) = Path::new(&db_path).parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create analytics database directory")?;
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
            .context("Failed to connect to analytics database")?;

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
        // Create search_history table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS search_history (
                id TEXT PRIMARY KEY,
                query TEXT NOT NULL,
                top_k INTEGER NOT NULL,
                results_count INTEGER NOT NULL,
                avg_score REAL,
                duration_ms INTEGER NOT NULL,
                client_type TEXT NOT NULL,
                client_id TEXT,
                session_id TEXT,
                timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create search_history table")?;

        // Create indexes for search_history
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_search_history_timestamp
            ON search_history(timestamp);
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create search_history timestamp index")?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_search_history_client
            ON search_history(client_type, client_id);
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create search_history client index")?;

        // Create search_feedback table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS search_feedback (
                id TEXT PRIMARY KEY,
                query TEXT NOT NULL,
                result_id TEXT NOT NULL,
                score REAL NOT NULL,
                rank INTEGER NOT NULL,
                feedback_type TEXT NOT NULL,
                reason TEXT,
                comment TEXT,
                client_type TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create search_feedback table")?;

        // Create indexes for search_feedback
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_search_feedback_result
            ON search_feedback(result_id, feedback_type);
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create search_feedback result index")?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_search_feedback_timestamp
            ON search_feedback(timestamp);
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create search_feedback timestamp index")?;

        Ok(())
    }

    /// Log a search to the history
    pub async fn log_search(&self, entry: &SearchHistoryEntry) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO search_history
            (id, query, top_k, results_count, avg_score, duration_ms,
             client_type, client_id, session_id, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(entry.id.to_string())
        .bind(&entry.query)
        .bind(entry.top_k as i64)
        .bind(entry.results_count as i64)
        .bind(entry.avg_score)
        .bind(entry.duration_ms as i64)
        .bind(&entry.client_type)
        .bind(&entry.client_id)
        .bind(&entry.session_id)
        .bind(entry.timestamp.to_rfc3339())
        .execute(&self.pool)
        .await
        .context("Failed to insert search history entry")?;

        Ok(())
    }

    /// Log feedback for a search result
    pub async fn log_feedback(&self, entry: &SearchFeedbackEntry) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO search_feedback
            (id, query, result_id, score, rank, feedback_type,
             reason, comment, client_type, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(entry.id.to_string())
        .bind(&entry.query)
        .bind(&entry.result_id)
        .bind(entry.score)
        .bind(entry.rank as i64)
        .bind(entry.feedback_type.as_str())
        .bind(&entry.reason)
        .bind(&entry.comment)
        .bind(&entry.client_type)
        .bind(entry.timestamp.to_rfc3339())
        .execute(&self.pool)
        .await
        .context("Failed to insert feedback entry")?;

        Ok(())
    }

    /// Get search history with optional filtering
    pub async fn get_history(&self, filter: &SearchHistoryFilter) -> Result<Vec<SearchHistoryEntry>> {
        // Build query dynamically based on filters
        let mut query = "SELECT * FROM search_history WHERE 1=1".to_string();
        let mut bindings: Vec<String> = Vec::new();

        if let Some(client_type) = &filter.client_type {
            query.push_str(&format!(" AND client_type = ?{}", bindings.len() + 1));
            bindings.push(client_type.clone());
        }

        if let Some(client_id) = &filter.client_id {
            query.push_str(&format!(" AND client_id = ?{}", bindings.len() + 1));
            bindings.push(client_id.clone());
        }

        if let Some(session_id) = &filter.session_id {
            query.push_str(&format!(" AND session_id = ?{}", bindings.len() + 1));
            bindings.push(session_id.clone());
        }

        if let Some(from_date) = &filter.from_date {
            query.push_str(&format!(" AND timestamp >= ?{}", bindings.len() + 1));
            bindings.push(from_date.to_rfc3339());
        }

        if let Some(to_date) = &filter.to_date {
            query.push_str(&format!(" AND timestamp <= ?{}", bindings.len() + 1));
            bindings.push(to_date.to_rfc3339());
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        // Execute query
        let mut query_builder = sqlx::query_as::<_, (
            String, String, i64, i64, Option<f32>, i64,
            String, Option<String>, Option<String>, String,
        )>(&query);

        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch search history")?;

        // Convert rows to entries
        let entries = rows
            .into_iter()
            .filter_map(|row| {
                Some(SearchHistoryEntry {
                    id: Uuid::parse_str(&row.0).ok()?,
                    query: row.1,
                    top_k: row.2 as usize,
                    results_count: row.3 as usize,
                    avg_score: row.4,
                    duration_ms: row.5 as u64,
                    client_type: row.6,
                    client_id: row.7,
                    session_id: row.8,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&row.9)
                        .ok()?
                        .with_timezone(&chrono::Utc),
                })
            })
            .collect();

        Ok(entries)
    }

    /// Get feedback with optional filtering
    pub async fn get_feedback(&self, filter: &FeedbackFilter) -> Result<Vec<SearchFeedbackEntry>> {
        let mut query = "SELECT * FROM search_feedback WHERE 1=1".to_string();
        let mut bindings: Vec<String> = Vec::new();

        if let Some(query_text) = &filter.query {
            query.push_str(&format!(" AND query = ?{}", bindings.len() + 1));
            bindings.push(query_text.clone());
        }

        if let Some(result_id) = &filter.result_id {
            query.push_str(&format!(" AND result_id = ?{}", bindings.len() + 1));
            bindings.push(result_id.clone());
        }

        if let Some(feedback_type) = &filter.feedback_type {
            query.push_str(&format!(" AND feedback_type = ?{}", bindings.len() + 1));
            bindings.push(feedback_type.as_str().to_string());
        }

        if let Some(client_type) = &filter.client_type {
            query.push_str(&format!(" AND client_type = ?{}", bindings.len() + 1));
            bindings.push(client_type.clone());
        }

        if let Some(from_date) = &filter.from_date {
            query.push_str(&format!(" AND timestamp >= ?{}", bindings.len() + 1));
            bindings.push(from_date.to_rfc3339());
        }

        if let Some(to_date) = &filter.to_date {
            query.push_str(&format!(" AND timestamp <= ?{}", bindings.len() + 1));
            bindings.push(to_date.to_rfc3339());
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        // Execute query
        let mut query_builder = sqlx::query_as::<_, (
            String, String, String, f32, i64, String,
            Option<String>, Option<String>, String, String,
        )>(&query);

        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch feedback")?;

        // Convert rows to entries
        let entries = rows
            .into_iter()
            .filter_map(|row| {
                Some(SearchFeedbackEntry {
                    id: Uuid::parse_str(&row.0).ok()?,
                    query: row.1,
                    result_id: row.2,
                    score: row.3,
                    rank: row.4 as usize,
                    feedback_type: FeedbackType::from_str(&row.5)?,
                    reason: row.6,
                    comment: row.7,
                    client_type: row.8,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&row.9)
                        .ok()?
                        .with_timezone(&chrono::Utc),
                })
            })
            .collect();

        Ok(entries)
    }

    /// Get analytics overview statistics
    pub async fn get_overview(&self, days: u32) -> Result<AnalyticsOverview> {
        let from_date = chrono::Utc::now() - chrono::Duration::days(days as i64);

        // Get total searches
        let total_searches: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM search_history WHERE timestamp >= ?1"
        )
        .bind(from_date.to_rfc3339())
        .fetch_one(&self.pool)
        .await
        .context("Failed to get total searches")?;

        // Get average latency and results
        let search_stats: (Option<f64>, Option<f64>) = sqlx::query_as(
            "SELECT AVG(duration_ms), AVG(results_count) FROM search_history WHERE timestamp >= ?1"
        )
        .bind(from_date.to_rfc3339())
        .fetch_one(&self.pool)
        .await
        .context("Failed to get search stats")?;

        // Get feedback counts
        let total_feedback: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM search_feedback WHERE timestamp >= ?1"
        )
        .bind(from_date.to_rfc3339())
        .fetch_one(&self.pool)
        .await
        .context("Failed to get total feedback")?;

        let positive_feedback: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM search_feedback WHERE feedback_type = 'positive' AND timestamp >= ?1"
        )
        .bind(from_date.to_rfc3339())
        .fetch_one(&self.pool)
        .await
        .context("Failed to get positive feedback")?;

        let negative_feedback: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM search_feedback WHERE feedback_type = 'negative' AND timestamp >= ?1"
        )
        .bind(from_date.to_rfc3339())
        .fetch_one(&self.pool)
        .await
        .context("Failed to get negative feedback")?;

        Ok(AnalyticsOverview {
            total_searches: total_searches.0 as usize,
            total_feedback: total_feedback.0 as usize,
            positive_feedback: positive_feedback.0 as usize,
            negative_feedback: negative_feedback.0 as usize,
            avg_latency_ms: search_stats.0.unwrap_or(0.0),
            avg_results: search_stats.1.unwrap_or(0.0),
        })
    }

    /// Get top queries by frequency
    pub async fn get_top_queries(&self, limit: usize, days: u32) -> Result<Vec<TopQuery>> {
        let from_date = chrono::Utc::now() - chrono::Duration::days(days as i64);

        let rows: Vec<(String, i64, f64, f64)> = sqlx::query_as(
            r#"
            SELECT
                query,
                COUNT(*) as count,
                AVG(results_count) as avg_results,
                AVG(duration_ms) as avg_latency
            FROM search_history
            WHERE timestamp >= ?1
            GROUP BY query
            ORDER BY count DESC
            LIMIT ?2
            "#
        )
        .bind(from_date.to_rfc3339())
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get top queries")?;

        let mut result = Vec::new();
        for row in rows {
            // Get feedback counts for this query
            let feedback_counts: (i64, i64) = sqlx::query_as(
                r#"
                SELECT
                    SUM(CASE WHEN feedback_type = 'positive' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN feedback_type = 'negative' THEN 1 ELSE 0 END)
                FROM search_feedback
                WHERE query = ?1 AND timestamp >= ?2
                "#
            )
            .bind(&row.0)
            .bind(from_date.to_rfc3339())
            .fetch_one(&self.pool)
            .await
            .unwrap_or((0, 0));

            result.push(TopQuery {
                query: row.0,
                count: row.1 as usize,
                avg_results: row.2,
                avg_latency_ms: row.3,
                positive_feedback: feedback_counts.0 as usize,
                negative_feedback: feedback_counts.1 as usize,
            });
        }

        Ok(result)
    }

    /// Get feedback statistics by result
    pub async fn get_feedback_stats(&self, days: u32) -> Result<FeedbackStats> {
        let from_date = chrono::Utc::now() - chrono::Duration::days(days as i64);

        // Get feedback by type
        let by_type: Vec<(String, i64)> = sqlx::query_as(
            "SELECT feedback_type, COUNT(*) FROM search_feedback WHERE timestamp >= ?1 GROUP BY feedback_type"
        )
        .bind(from_date.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .context("Failed to get feedback by type")?;

        // Get top positive results
        let top_positive: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT result_id, COUNT(*) as count
            FROM search_feedback
            WHERE feedback_type = 'positive' AND timestamp >= ?1
            GROUP BY result_id
            ORDER BY count DESC
            LIMIT 10
            "#
        )
        .bind(from_date.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .context("Failed to get top positive results")?;

        // Get top negative results
        let top_negative: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT result_id, COUNT(*) as count
            FROM search_feedback
            WHERE feedback_type = 'negative' AND timestamp >= ?1
            GROUP BY result_id
            ORDER BY count DESC
            LIMIT 10
            "#
        )
        .bind(from_date.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .context("Failed to get top negative results")?;

        Ok(FeedbackStats {
            by_type: by_type.into_iter().map(|(t, c)| (t, c as usize)).collect(),
            top_positive: top_positive.into_iter().map(|(id, c)| (id, c as usize)).collect(),
            top_negative: top_negative.into_iter().map(|(id, c)| (id, c as usize)).collect(),
        })
    }

    /// Get search timeline data
    pub async fn get_timeline(&self, days: u32, interval_hours: u32) -> Result<Vec<TimelinePoint>> {
        let from_date = chrono::Utc::now() - chrono::Duration::days(days as i64);

        let rows: Vec<(String, i64, f64)> = sqlx::query_as(
            r#"
            SELECT
                datetime((strftime('%s', timestamp) / (?2 * 3600)) * (?2 * 3600), 'unixepoch') as time_bucket,
                COUNT(*) as count,
                AVG(duration_ms) as avg_latency
            FROM search_history
            WHERE timestamp >= ?1
            GROUP BY time_bucket
            ORDER BY time_bucket ASC
            "#
        )
        .bind(from_date.to_rfc3339())
        .bind(interval_hours as i64)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get timeline data")?;

        let points = rows
            .into_iter()
            .filter_map(|(ts, count, latency)| {
                Some(TimelinePoint {
                    timestamp: chrono::DateTime::parse_from_rfc3339(&format!("{}Z", ts))
                        .ok()?
                        .with_timezone(&chrono::Utc),
                    search_count: count as usize,
                    avg_latency_ms: latency,
                })
            })
            .collect();

        Ok(points)
    }
}

// Helper types for analytics queries
#[derive(Debug, Clone)]
pub struct AnalyticsOverview {
    pub total_searches: usize,
    pub total_feedback: usize,
    pub positive_feedback: usize,
    pub negative_feedback: usize,
    pub avg_latency_ms: f64,
    pub avg_results: f64,
}

#[derive(Debug, Clone)]
pub struct TopQuery {
    pub query: String,
    pub count: usize,
    pub avg_results: f64,
    pub avg_latency_ms: f64,
    pub positive_feedback: usize,
    pub negative_feedback: usize,
}

#[derive(Debug, Clone)]
pub struct FeedbackStats {
    pub by_type: Vec<(String, usize)>,
    pub top_positive: Vec<(String, usize)>,
    pub top_negative: Vec<(String, usize)>,
}

#[derive(Debug, Clone)]
pub struct TimelinePoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub search_count: usize,
    pub avg_latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creation() {
        let db = SearchAnalyticsDb::new(":memory:").await.unwrap();
        assert!(db.pool().is_closed() == false);
    }

    #[tokio::test]
    async fn test_log_search() {
        let db = SearchAnalyticsDb::new(":memory:").await.unwrap();

        let entry = SearchHistoryEntry {
            id: Uuid::new_v4(),
            query: "test query".to_string(),
            top_k: 10,
            results_count: 5,
            avg_score: Some(0.85),
            duration_ms: 150,
            client_type: "mcp".to_string(),
            client_id: Some("client-1".to_string()),
            session_id: Some("session-1".to_string()),
            timestamp: chrono::Utc::now(),
        };

        db.log_search(&entry).await.unwrap();

        let history = db.get_history(&SearchHistoryFilter::default()).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].query, "test query");
    }
}
