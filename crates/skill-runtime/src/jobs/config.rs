//! Job queue configuration
//!
//! Provides flexible configuration for job storage backends,
//! supporting both programmatic and environment-based setup.

use std::path::PathBuf;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Storage backend type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    /// In-memory storage (for testing, non-persistent)
    Memory,
    /// SQLite storage (default, local-first)
    Sqlite,
    /// PostgreSQL storage (distributed deployments)
    Postgres,
    /// Redis storage (high-throughput)
    Redis,
}

impl Default for StorageBackend {
    fn default() -> Self {
        Self::Sqlite
    }
}

impl std::fmt::Display for StorageBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Memory => write!(f, "memory"),
            Self::Sqlite => write!(f, "sqlite"),
            Self::Postgres => write!(f, "postgres"),
            Self::Redis => write!(f, "redis"),
        }
    }
}

impl std::str::FromStr for StorageBackend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "memory" | "mem" => Ok(Self::Memory),
            "sqlite" | "sqlite3" => Ok(Self::Sqlite),
            "postgres" | "postgresql" | "pg" => Ok(Self::Postgres),
            "redis" => Ok(Self::Redis),
            _ => Err(format!("Unknown storage backend: {}", s)),
        }
    }
}

/// Job queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    /// Storage backend type
    pub backend: StorageBackend,

    /// Connection string or path
    /// - SQLite: file path (e.g., "/data/jobs.db" or ":memory:")
    /// - PostgreSQL: connection URL (e.g., "postgres://user:pass@host/db")
    /// - Redis: connection URL (e.g., "redis://localhost:6379")
    pub connection: String,

    /// Number of concurrent workers
    #[serde(default = "default_workers")]
    pub workers: usize,

    /// Maximum retry attempts for failed jobs
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Retry delay (exponential backoff base)
    #[serde(default = "default_retry_delay_secs", with = "duration_secs")]
    pub retry_delay: Duration,

    /// Job timeout before considered stale
    #[serde(default = "default_job_timeout_secs", with = "duration_secs")]
    pub job_timeout: Duration,

    /// Poll interval for checking new jobs
    #[serde(default = "default_poll_interval_ms", with = "duration_millis")]
    pub poll_interval: Duration,

    /// Enable job persistence (survive restarts)
    #[serde(default = "default_persistent")]
    pub persistent: bool,

    /// Cleanup completed jobs after this duration
    #[serde(default = "default_cleanup_after_secs", with = "duration_secs")]
    pub cleanup_after: Duration,
}

fn default_workers() -> usize { 4 }
fn default_max_retries() -> u32 { 3 }
fn default_retry_delay_secs() -> Duration { Duration::from_secs(60) }
fn default_job_timeout_secs() -> Duration { Duration::from_secs(3600) } // 1 hour
fn default_poll_interval_ms() -> Duration { Duration::from_millis(500) }
fn default_persistent() -> bool { true }
fn default_cleanup_after_secs() -> Duration { Duration::from_secs(86400) } // 24 hours

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::default(),
            connection: default_sqlite_path(),
            workers: default_workers(),
            max_retries: default_max_retries(),
            retry_delay: default_retry_delay_secs(),
            job_timeout: default_job_timeout_secs(),
            poll_interval: default_poll_interval_ms(),
            persistent: default_persistent(),
            cleanup_after: default_cleanup_after_secs(),
        }
    }
}

impl JobConfig {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create SQLite configuration with specified path
    pub fn sqlite(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let connection = path.to_string_lossy().to_string();
        Self {
            backend: StorageBackend::Sqlite,
            connection,
            ..Default::default()
        }
    }

    /// Create SQLite configuration with default path
    pub fn sqlite_default() -> Self {
        Self::sqlite(default_sqlite_path())
    }

    /// Create in-memory SQLite configuration (for testing)
    pub fn memory() -> Self {
        Self {
            backend: StorageBackend::Memory,
            connection: ":memory:".to_string(),
            persistent: false,
            ..Default::default()
        }
    }

    /// Create PostgreSQL configuration
    #[cfg(feature = "postgres-storage")]
    pub fn postgres(connection_url: impl Into<String>) -> Self {
        Self {
            backend: StorageBackend::Postgres,
            connection: connection_url.into(),
            ..Default::default()
        }
    }

    /// Create Redis configuration
    #[cfg(feature = "redis-storage")]
    pub fn redis(connection_url: impl Into<String>) -> Self {
        Self {
            backend: StorageBackend::Redis,
            connection: connection_url.into(),
            ..Default::default()
        }
    }

    /// Load configuration from environment variables
    ///
    /// Environment variables:
    /// - `SKILL_JOB_BACKEND`: Storage backend (sqlite, postgres, redis)
    /// - `SKILL_JOB_CONNECTION`: Connection string/path
    /// - `SKILL_JOB_WORKERS`: Number of workers
    /// - `SKILL_JOB_MAX_RETRIES`: Maximum retry attempts
    /// - `SKILL_JOB_TIMEOUT_SECS`: Job timeout in seconds
    /// - `DATABASE_URL`: Fallback for connection string
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Backend type
        if let Ok(backend) = std::env::var("SKILL_JOB_BACKEND") {
            config.backend = backend.parse().map_err(ConfigError::InvalidBackend)?;
        }

        // Connection string
        if let Ok(conn) = std::env::var("SKILL_JOB_CONNECTION") {
            config.connection = conn;
        } else if let Ok(conn) = std::env::var("DATABASE_URL") {
            config.connection = conn;
            // Auto-detect backend from URL
            if config.connection.starts_with("postgres") {
                config.backend = StorageBackend::Postgres;
            } else if config.connection.starts_with("redis") {
                config.backend = StorageBackend::Redis;
            }
        }

        // Workers
        if let Ok(workers) = std::env::var("SKILL_JOB_WORKERS") {
            config.workers = workers.parse().map_err(|_| ConfigError::InvalidWorkers)?;
        }

        // Max retries
        if let Ok(retries) = std::env::var("SKILL_JOB_MAX_RETRIES") {
            config.max_retries = retries.parse().map_err(|_| ConfigError::InvalidRetries)?;
        }

        // Job timeout
        if let Ok(timeout) = std::env::var("SKILL_JOB_TIMEOUT_SECS") {
            let secs: u64 = timeout.parse().map_err(|_| ConfigError::InvalidTimeout)?;
            config.job_timeout = Duration::from_secs(secs);
        }

        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.workers == 0 {
            return Err(ConfigError::InvalidWorkers);
        }

        if self.connection.is_empty() {
            return Err(ConfigError::EmptyConnection);
        }

        // Validate backend-specific requirements
        match self.backend {
            StorageBackend::Postgres => {
                if !self.connection.starts_with("postgres") {
                    return Err(ConfigError::InvalidConnectionFormat(
                        "PostgreSQL connection must start with 'postgres://'".to_string()
                    ));
                }
            }
            StorageBackend::Redis => {
                if !self.connection.starts_with("redis") {
                    return Err(ConfigError::InvalidConnectionFormat(
                        "Redis connection must start with 'redis://'".to_string()
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Set the number of workers
    pub fn with_workers(mut self, workers: usize) -> Self {
        self.workers = workers;
        self
    }

    /// Set max retry attempts
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set retry delay
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Set job timeout
    pub fn with_job_timeout(mut self, timeout: Duration) -> Self {
        self.job_timeout = timeout;
        self
    }

    /// Set poll interval
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }
}

/// Configuration error
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid storage backend: {0}")]
    InvalidBackend(String),

    #[error("Invalid worker count")]
    InvalidWorkers,

    #[error("Invalid retry count")]
    InvalidRetries,

    #[error("Invalid timeout value")]
    InvalidTimeout,

    #[error("Empty connection string")]
    EmptyConnection,

    #[error("Invalid connection format: {0}")]
    InvalidConnectionFormat(String),

    #[error("Backend not available: {0} (enable feature flag)")]
    BackendNotAvailable(String),
}

/// Get the default SQLite database path
fn default_sqlite_path() -> String {
    dirs::data_local_dir()
        .map(|p| p.join("skill-engine").join("jobs.db"))
        .unwrap_or_else(|| PathBuf::from("~/.skill-engine/jobs.db"))
        .to_string_lossy()
        .to_string()
}

// Serde helpers for Duration
mod duration_secs {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

mod duration_millis {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = JobConfig::default();
        assert_eq!(config.backend, StorageBackend::Sqlite);
        assert_eq!(config.workers, 4);
        assert_eq!(config.max_retries, 3);
        assert!(config.persistent);
    }

    #[test]
    fn test_sqlite_config() {
        let config = JobConfig::sqlite("/tmp/test.db");
        assert_eq!(config.backend, StorageBackend::Sqlite);
        assert_eq!(config.connection, "/tmp/test.db");
    }

    #[test]
    fn test_memory_config() {
        let config = JobConfig::memory();
        assert_eq!(config.backend, StorageBackend::Memory);
        assert_eq!(config.connection, ":memory:");
        assert!(!config.persistent);
    }

    #[test]
    fn test_config_builder() {
        let config = JobConfig::sqlite("/tmp/test.db")
            .with_workers(8)
            .with_max_retries(5)
            .with_job_timeout(Duration::from_secs(7200));

        assert_eq!(config.workers, 8);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.job_timeout, Duration::from_secs(7200));
    }

    #[test]
    fn test_backend_parsing() {
        assert_eq!("sqlite".parse::<StorageBackend>().unwrap(), StorageBackend::Sqlite);
        assert_eq!("postgres".parse::<StorageBackend>().unwrap(), StorageBackend::Postgres);
        assert_eq!("redis".parse::<StorageBackend>().unwrap(), StorageBackend::Redis);
        assert_eq!("memory".parse::<StorageBackend>().unwrap(), StorageBackend::Memory);
    }

    #[test]
    fn test_config_validation() {
        let mut config = JobConfig::sqlite("/tmp/test.db");
        assert!(config.validate().is_ok());

        config.workers = 0;
        assert!(config.validate().is_err());

        config.workers = 4;
        config.connection = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = JobConfig::sqlite("/tmp/test.db");
        let json = serde_json::to_string(&config).unwrap();
        let parsed: JobConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.backend, config.backend);
        assert_eq!(parsed.connection, config.connection);
    }
}
