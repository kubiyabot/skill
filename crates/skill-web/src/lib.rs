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

// Re-export common types - be explicit to avoid ambiguous glob re-exports
pub use api::client::ApiClient;
pub use api::error::{ApiError, ApiResult};
// Re-export API types (the canonical source)
pub use api::types::{
    ExecutionHistoryEntry, ExecutionRequest, ExecutionResponse, ExecutionStatus,
    InstallSkillRequest, InstallSkillResponse, InstanceInfo, PaginatedResponse,
    PaginationParams, ParameterInfo, QueryInfo, SearchConfigResponse, SearchRequest,
    SearchResponse, SearchResult, SkillDetail, SkillServiceRequirement, SkillSummary,
    ToolInfo, UpdateSearchConfigRequest,
};
// Re-export store types (avoiding duplicates with api::types)
pub use store::executions::{ActiveExecution, ExecutionEntry, ExecutionsAction, ExecutionsStore};
pub use store::skills::{SkillRuntime, SkillSortBy, SkillStatus, SkillsAction, SkillsStore};
