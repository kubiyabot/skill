//! API client module for communicating with the skill-http backend
//!
//! This module provides a type-safe HTTP client for all API operations.
//!
//! ## Usage
//!
//! ```ignore
//! use crate::api::Api;
//!
//! // Create API client (uses /api prefix for Trunk proxy)
//! let api = Api::new();
//!
//! // List skills
//! let skills = api.skills.list_all().await?;
//!
//! // Execute a tool
//! let result = api.executions.execute_simple("kubernetes", "get_pods", args).await?;
//!
//! // Search
//! let results = api.search.query("deploy pods").await?;
//!
//! // Check health
//! let healthy = api.config.is_healthy().await;
//! ```

pub mod agent;
pub mod analytics;
pub mod client;
pub mod config;
pub mod error;
pub mod executions;
pub mod feedback;
pub mod search;
pub mod services;
pub mod skills;
pub mod types;

pub use agent::AgentApi;
pub use analytics::AnalyticsApi;
pub use client::ApiClient;
pub use config::ConfigApi;
pub use error::{ApiError, ApiResult};
pub use executions::ExecutionsApi;
pub use feedback::{
    FeedbackApi, FeedbackEntry, GetFeedbackRequest, GetFeedbackResponse, SubmitFeedbackRequest,
    SubmitFeedbackResponse,
};
pub use search::SearchApi;
pub use services::ServicesApi;
pub use skills::SkillsApi;
pub use types::*;

/// Unified API facade providing access to all API endpoints
#[derive(Clone)]
pub struct Api {
    /// Skills API operations
    pub skills: SkillsApi,
    /// Execution API operations
    pub executions: ExecutionsApi,
    /// Search API operations
    pub search: SearchApi,
    /// Configuration API operations
    pub config: ConfigApi,
    /// System services API operations
    pub services: ServicesApi,
    /// Agent configuration API operations
    pub agent: AgentApi,
    /// Feedback API operations
    pub feedback: FeedbackApi,
    /// Analytics API operations
    pub analytics: AnalyticsApi,
}

impl Default for Api {
    fn default() -> Self {
        Self::new()
    }
}

impl Api {
    /// Create a new API facade with the default local client
    pub fn new() -> Self {
        let client = ApiClient::local();
        Self::with_client(client)
    }

    /// Create an API facade with a custom base URL
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let client = ApiClient::new(base_url);
        Self::with_client(client)
    }

    /// Create an API facade with a specific host and port
    pub fn with_host(host: &str, port: u16) -> Self {
        let client = ApiClient::with_host(host, port);
        Self::with_client(client)
    }

    /// Create an API facade with an existing client
    pub fn with_client(client: ApiClient) -> Self {
        Self {
            skills: SkillsApi::new(client.clone()),
            executions: ExecutionsApi::new(client.clone()),
            search: SearchApi::new(client.clone()),
            config: ConfigApi::new(client.clone()),
            services: ServicesApi::new(client.clone()),
            agent: AgentApi::new(client.clone()),
            feedback: FeedbackApi::new(client.clone()),
            analytics: AnalyticsApi::new(client),
        }
    }

    /// Check if the API server is reachable and healthy
    pub async fn is_available(&self) -> bool {
        self.config.is_healthy().await
    }

    /// Get version information from the server
    pub async fn server_version(&self) -> ApiResult<String> {
        let version = self.config.version().await?;
        Ok(version.version)
    }
}

/// Create a default API client (convenience function)
pub fn api() -> Api {
    Api::new()
}
