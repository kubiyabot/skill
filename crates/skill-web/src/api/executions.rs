//! Executions API client

use std::collections::HashMap;

use super::client::ApiClient;
use super::error::ApiResult;
use super::types::*;

/// Executions API operations
#[derive(Clone)]
pub struct ExecutionsApi {
    client: ApiClient,
}

impl ExecutionsApi {
    /// Create a new executions API client
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// Execute a skill tool
    pub async fn execute(&self, request: &ExecutionRequest) -> ApiResult<ExecutionResponse> {
        self.client.post("/execute", request).await
    }

    /// Execute a skill tool with simple parameters
    pub async fn execute_simple(
        &self,
        skill: &str,
        tool: &str,
        args: HashMap<String, serde_json::Value>,
    ) -> ApiResult<ExecutionResponse> {
        self.execute(&ExecutionRequest {
            skill: skill.to_string(),
            tool: tool.to_string(),
            instance: None,
            args,
            stream: false,
            timeout_secs: None,
        })
        .await
    }

    /// Execute a skill tool on a specific instance
    pub async fn execute_on_instance(
        &self,
        skill: &str,
        tool: &str,
        instance: &str,
        args: HashMap<String, serde_json::Value>,
    ) -> ApiResult<ExecutionResponse> {
        self.execute(&ExecutionRequest {
            skill: skill.to_string(),
            tool: tool.to_string(),
            instance: Some(instance.to_string()),
            args,
            stream: false,
            timeout_secs: None,
        })
        .await
    }

    /// Execute with custom timeout
    pub async fn execute_with_timeout(
        &self,
        skill: &str,
        tool: &str,
        args: HashMap<String, serde_json::Value>,
        timeout_secs: u64,
    ) -> ApiResult<ExecutionResponse> {
        self.execute(&ExecutionRequest {
            skill: skill.to_string(),
            tool: tool.to_string(),
            instance: None,
            args,
            stream: false,
            timeout_secs: Some(timeout_secs),
        })
        .await
    }

    /// List execution history with pagination
    pub async fn list_history(
        &self,
        pagination: Option<PaginationParams>,
    ) -> ApiResult<PaginatedResponse<ExecutionHistoryEntry>> {
        let params = pagination.unwrap_or_default();
        self.client.get_with_query("/executions", &params).await
    }

    /// List all execution history (no pagination)
    pub async fn list_all_history(&self) -> ApiResult<Vec<ExecutionHistoryEntry>> {
        let response: PaginatedResponse<ExecutionHistoryEntry> = self
            .client
            .get_with_query(
                "/executions",
                &PaginationParams {
                    page: 1,
                    per_page: 1000,
                },
            )
            .await?;
        Ok(response.items)
    }

    /// Get a specific execution by ID
    pub async fn get(&self, id: &str) -> ApiResult<ExecutionHistoryEntry> {
        self.client.get(&format!("/executions/{}", id)).await
    }

    /// Get recent executions for a skill
    pub async fn recent_for_skill(
        &self,
        skill: &str,
        limit: usize,
    ) -> ApiResult<Vec<ExecutionHistoryEntry>> {
        // For now, fetch all and filter client-side
        // TODO: Add server-side filtering
        let all = self.list_all_history().await?;
        Ok(all
            .into_iter()
            .filter(|e| e.skill == skill)
            .take(limit)
            .collect())
    }

    /// Clear all execution history
    pub async fn clear_history(&self) -> ApiResult<()> {
        self.client.delete("/executions").await?;
        Ok(())
    }
}
