//! Analytics module for tracking search history and feedback
//!
//! Provides SQLite-based storage for:
//! - Search history with client metadata
//! - User feedback on search results
//! - Query performance metrics

pub mod db;
pub mod types;

pub use db::SearchAnalyticsDb;
pub use types::*;
