//! Background job processing for persistent skill execution
//!
//! This module provides a configurable job queue abstraction using apalis,
//! supporting multiple storage backends:
//! - SQLite (default, local-first)
//! - PostgreSQL (optional, for distributed deployments)
//! - Redis (optional, for high-throughput scenarios)
//!
//! # Feature Flags
//!
//! - `job-queue` - Enables the base job queue functionality
//! - `sqlite-storage` - SQLite backend (default for local mode)
//! - `postgres-storage` - PostgreSQL backend
//! - `redis-storage` - Redis backend
//!
//! # Example
//!
//! ```rust,ignore
//! use skill_runtime::jobs::{JobConfig, JobStorage, create_storage};
//!
//! // Create SQLite storage (local-first default)
//! let config = JobConfig::sqlite("~/.skill-engine/jobs.db");
//! let storage = create_storage(&config).await?;
//!
//! // Or use environment-based configuration
//! let config = JobConfig::from_env()?;
//! let storage = create_storage(&config).await?;
//! ```

mod config;
mod types;

#[cfg(feature = "job-queue")]
mod storage;

#[cfg(feature = "job-queue")]
mod worker;

#[cfg(feature = "sqlite-storage")]
mod sqlite;

#[cfg(feature = "postgres-storage")]
mod postgres;

#[cfg(feature = "redis-storage")]
mod redis_backend;

pub use config::*;
pub use types::*;

#[cfg(feature = "job-queue")]
pub use storage::*;

#[cfg(feature = "job-queue")]
pub use worker::*;

#[cfg(feature = "sqlite-storage")]
pub use sqlite::*;

#[cfg(feature = "postgres-storage")]
pub use postgres::*;

#[cfg(feature = "redis-storage")]
pub use redis_backend::*;
