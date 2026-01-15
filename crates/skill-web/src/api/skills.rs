//! Skills API client

use super::client::ApiClient;
use super::error::ApiResult;
use super::types::*;

/// Skills API operations
#[derive(Clone)]
pub struct SkillsApi {
    client: ApiClient,
}

impl SkillsApi {
    /// Create a new skills API client
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// List all installed skills with pagination
    pub async fn list(&self, pagination: Option<PaginationParams>) -> ApiResult<PaginatedResponse<SkillSummary>> {
        let params = pagination.unwrap_or_default();
        self.client.get_with_query("/skills", &params).await
    }

    /// List all skills (no pagination)
    pub async fn list_all(&self) -> ApiResult<Vec<SkillSummary>> {
        let response: PaginatedResponse<SkillSummary> = self.client
            .get_with_query("/skills", &PaginationParams { page: 1, per_page: 1000 })
            .await?;
        Ok(response.items)
    }

    /// Get details for a specific skill
    pub async fn get(&self, name: &str) -> ApiResult<SkillDetail> {
        self.client.get(&format!("/skills/{}", name)).await
    }

    /// Install a skill from a source
    pub async fn install(&self, request: &InstallSkillRequest) -> ApiResult<InstallSkillResponse> {
        self.client.post("/skills", request).await
    }

    /// Install a skill from a git URL
    pub async fn install_from_git(
        &self,
        url: &str,
        git_ref: Option<&str>,
        force: bool,
    ) -> ApiResult<InstallSkillResponse> {
        self.install(&InstallSkillRequest {
            source: url.to_string(),
            name: None,
            git_ref: git_ref.map(String::from),
            force,
        })
        .await
    }

    /// Install a skill with a custom name
    pub async fn install_with_name(
        &self,
        source: &str,
        name: &str,
    ) -> ApiResult<InstallSkillResponse> {
        self.install(&InstallSkillRequest {
            source: source.to_string(),
            name: Some(name.to_string()),
            git_ref: None,
            force: false,
        })
        .await
    }

    /// Uninstall a skill
    pub async fn uninstall(&self, name: &str) -> ApiResult<()> {
        self.client.delete(&format!("/skills/{}", name)).await
    }

    /// Get the tools for a skill
    pub async fn get_tools(&self, skill_name: &str) -> ApiResult<Vec<ToolInfo>> {
        let detail = self.get(skill_name).await?;
        Ok(detail.tools)
    }

    /// Get instances for a skill
    pub async fn get_instances(&self, skill_name: &str) -> ApiResult<Vec<InstanceInfo>> {
        let detail = self.get(skill_name).await?;
        Ok(detail.instances)
    }
}
