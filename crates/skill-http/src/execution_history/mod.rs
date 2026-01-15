//! Execution history module for persistent storage
//!
//! Provides SQLite-based storage for:
//! - Execution history with full details
//! - Filtering and pagination
//! - Statistics and analytics

pub mod db;

pub use db::ExecutionHistoryDb;
