//! Skill Web UI Library
//!
//! This library exports the core modules of the Skill Engine web interface
//! for use in tests and potentially other consumers.

// Re-export all public modules
pub mod api;
pub mod store;
pub mod utils;
pub mod router;
pub mod pages;
pub mod components;
pub mod hooks;
pub mod app;

// Re-export common types
pub use api::client::ApiClient;
pub use api::types::*;
pub use api::error::{ApiError, ApiResult};
pub use store::skills::*;
pub use store::executions::*;
