//! Agent configuration API client

use super::client::ApiClient;
use super::error::ApiResult;
use super::types::*;

/// Agent configuration API operations
#[derive(Clone)]
pub struct AgentApi {
    client: ApiClient,
}

impl AgentApi {
    /// Create a new agent API client
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// Get agent configuration
    pub async fn get_config(&self) -> ApiResult<GetAgentConfigResponse> {
        self.client.get("/agent/config").await
    }

    /// Update agent configuration
    pub async fn update_config(
        &self,
        request: &UpdateAgentConfigRequest,
    ) -> ApiResult<AgentConfig> {
        self.client.put("/agent/config", request).await
    }

    /// Set agent runtime
    pub async fn set_runtime(&self, runtime: AgentRuntime) -> ApiResult<AgentConfig> {
        self.update_config(&UpdateAgentConfigRequest {
            runtime: Some(runtime),
            ..Default::default()
        })
        .await
    }

    /// Set model configuration
    pub async fn set_model_config(
        &self,
        model_config: AgentModelConfig,
    ) -> ApiResult<AgentConfig> {
        self.update_config(&UpdateAgentConfigRequest {
            model_config: Some(model_config),
            ..Default::default()
        })
        .await
    }

    /// Set execution timeout
    pub async fn set_timeout(&self, timeout_secs: u64) -> ApiResult<AgentConfig> {
        self.update_config(&UpdateAgentConfigRequest {
            timeout_secs: Some(timeout_secs),
            ..Default::default()
        })
        .await
    }

    /// Set Claude Code path override
    pub async fn set_claude_code_path(&self, path: String) -> ApiResult<AgentConfig> {
        self.update_config(&UpdateAgentConfigRequest {
            claude_code_path: Some(path),
            ..Default::default()
        })
        .await
    }
}
