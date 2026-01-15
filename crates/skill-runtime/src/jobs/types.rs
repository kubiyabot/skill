//! Job types and data structures
//!
//! Defines the core job types used for background processing of skill operations.

use std::collections::HashMap;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique job identifier
pub type JobId = Uuid;

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Job is queued and waiting to be processed
    Pending,
    /// Job is currently being processed
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed (may be retried)
    Failed,
    /// Job was cancelled
    Cancelled,
    /// Job exceeded max retries and is dead
    Dead,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Dead => write!(f, "dead"),
        }
    }
}

/// Job priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for JobPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Types of background jobs supported
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JobType {
    /// Execute a skill with given parameters
    SkillExecution {
        skill_id: String,
        tool_name: String,
        parameters: serde_json::Value,
    },

    /// Generate examples for a skill using AI
    ExampleGeneration {
        skill_id: String,
        tool_names: Vec<String>,
        provider: String,
    },

    /// Index a skill into the search pipeline
    SkillIndexing {
        skill_id: String,
        skill_path: String,
    },

    /// Reindex all skills
    FullReindex,

    /// Train/update embeddings based on usage patterns
    EmbeddingUpdate {
        skill_ids: Vec<String>,
    },

    /// Cleanup old job records
    Maintenance {
        task: MaintenanceTask,
    },

    /// Custom job type for extensibility
    Custom {
        name: String,
        payload: serde_json::Value,
    },
}

/// Maintenance tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceTask {
    /// Remove completed jobs older than threshold
    CleanupCompletedJobs { older_than_days: u32 },
    /// Remove dead jobs
    CleanupDeadJobs,
    /// Vacuum database (SQLite only)
    VacuumDatabase,
    /// Re-queue orphaned jobs
    RequeueOrphaned,
}

/// A background job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique job identifier
    pub id: JobId,

    /// Job type and payload
    pub job_type: JobType,

    /// Current status
    pub status: JobStatus,

    /// Job priority
    pub priority: JobPriority,

    /// Number of attempts made
    pub attempts: u32,

    /// Maximum retry attempts
    pub max_attempts: u32,

    /// When the job was created
    pub created_at: DateTime<Utc>,

    /// When the job was last updated
    pub updated_at: DateTime<Utc>,

    /// When the job should run (for delayed jobs)
    pub scheduled_at: Option<DateTime<Utc>>,

    /// When the job started running
    pub started_at: Option<DateTime<Utc>>,

    /// When the job completed
    pub completed_at: Option<DateTime<Utc>>,

    /// Worker ID processing this job
    pub worker_id: Option<String>,

    /// Error message if failed
    pub error: Option<String>,

    /// Job result (if completed)
    pub result: Option<serde_json::Value>,

    /// Arbitrary metadata
    pub metadata: HashMap<String, String>,
}

impl Job {
    /// Create a new job
    pub fn new(job_type: JobType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            job_type,
            status: JobStatus::Pending,
            priority: JobPriority::Normal,
            attempts: 0,
            max_attempts: 3,
            created_at: now,
            updated_at: now,
            scheduled_at: None,
            started_at: None,
            completed_at: None,
            worker_id: None,
            error: None,
            result: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a skill execution job
    pub fn skill_execution(
        skill_id: impl Into<String>,
        tool_name: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Self::new(JobType::SkillExecution {
            skill_id: skill_id.into(),
            tool_name: tool_name.into(),
            parameters,
        })
    }

    /// Create an example generation job
    pub fn example_generation(
        skill_id: impl Into<String>,
        tool_names: Vec<String>,
        provider: impl Into<String>,
    ) -> Self {
        Self::new(JobType::ExampleGeneration {
            skill_id: skill_id.into(),
            tool_names,
            provider: provider.into(),
        })
    }

    /// Create a skill indexing job
    pub fn skill_indexing(
        skill_id: impl Into<String>,
        skill_path: impl Into<String>,
    ) -> Self {
        Self::new(JobType::SkillIndexing {
            skill_id: skill_id.into(),
            skill_path: skill_path.into(),
        })
    }

    /// Set job priority
    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set max attempts
    pub fn with_max_attempts(mut self, max: u32) -> Self {
        self.max_attempts = max;
        self
    }

    /// Schedule job for later
    pub fn scheduled_at(mut self, when: DateTime<Utc>) -> Self {
        self.scheduled_at = Some(when);
        self
    }

    /// Schedule job after a delay
    pub fn delayed(mut self, delay: Duration) -> Self {
        self.scheduled_at = Some(Utc::now() + chrono::Duration::from_std(delay).unwrap_or_default());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if job can be retried
    pub fn can_retry(&self) -> bool {
        self.attempts < self.max_attempts && self.status == JobStatus::Failed
    }

    /// Check if job is terminal (won't change state)
    pub fn is_terminal(&self) -> bool {
        matches!(self.status, JobStatus::Completed | JobStatus::Cancelled | JobStatus::Dead)
    }

    /// Get time since creation
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.created_at
    }

    /// Get execution duration (if completed)
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

/// Job progress update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    /// Job ID
    pub job_id: JobId,

    /// Progress percentage (0-100)
    pub percentage: u8,

    /// Current step description
    pub step: String,

    /// Additional details
    pub details: Option<String>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl JobProgress {
    pub fn new(job_id: JobId, percentage: u8, step: impl Into<String>) -> Self {
        Self {
            job_id,
            percentage: percentage.min(100),
            step: step.into(),
            details: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Job statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobStats {
    /// Total jobs in queue
    pub total: usize,

    /// Jobs by status
    pub by_status: HashMap<String, usize>,

    /// Average execution time (ms)
    pub avg_execution_ms: u64,

    /// Success rate (0.0-1.0)
    pub success_rate: f32,

    /// Jobs processed in last hour
    pub throughput_per_hour: usize,

    /// Current active workers
    pub active_workers: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let job = Job::skill_execution("kubernetes", "apply", serde_json::json!({"file": "deploy.yaml"}));

        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.priority, JobPriority::Normal);
        assert_eq!(job.attempts, 0);
        assert_eq!(job.max_attempts, 3);
    }

    #[test]
    fn test_job_builder() {
        let job = Job::skill_indexing("kubernetes", "/skills/kubernetes")
            .with_priority(JobPriority::High)
            .with_max_attempts(5)
            .with_metadata("source", "cli");

        assert_eq!(job.priority, JobPriority::High);
        assert_eq!(job.max_attempts, 5);
        assert_eq!(job.metadata.get("source"), Some(&"cli".to_string()));
    }

    #[test]
    fn test_job_can_retry() {
        let mut job = Job::skill_execution("test", "run", serde_json::json!({}));
        job.status = JobStatus::Failed;
        job.attempts = 1;
        job.max_attempts = 3;

        assert!(job.can_retry());

        job.attempts = 3;
        assert!(!job.can_retry());
    }

    #[test]
    fn test_job_is_terminal() {
        let mut job = Job::skill_execution("test", "run", serde_json::json!({}));

        assert!(!job.is_terminal());

        job.status = JobStatus::Completed;
        assert!(job.is_terminal());

        job.status = JobStatus::Cancelled;
        assert!(job.is_terminal());

        job.status = JobStatus::Dead;
        assert!(job.is_terminal());
    }

    #[test]
    fn test_job_progress() {
        let job_id = Uuid::new_v4();
        let progress = JobProgress::new(job_id, 50, "Processing tools")
            .with_details("Tool 5 of 10");

        assert_eq!(progress.percentage, 50);
        assert_eq!(progress.step, "Processing tools");
        assert_eq!(progress.details, Some("Tool 5 of 10".to_string()));
    }

    #[test]
    fn test_job_type_serialization() {
        let job_type = JobType::SkillExecution {
            skill_id: "kubernetes".to_string(),
            tool_name: "apply".to_string(),
            parameters: serde_json::json!({"file": "test.yaml"}),
        };

        let json = serde_json::to_string(&job_type).unwrap();
        assert!(json.contains("skill_execution"));

        let parsed: JobType = serde_json::from_str(&json).unwrap();
        match parsed {
            JobType::SkillExecution { skill_id, .. } => assert_eq!(skill_id, "kubernetes"),
            _ => panic!("Wrong job type"),
        }
    }
}
