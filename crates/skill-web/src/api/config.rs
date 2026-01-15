//! Configuration API client

use super::client::ApiClient;
use super::error::ApiResult;
use super::types::*;

/// Configuration API operations
#[derive(Clone)]
pub struct ConfigApi {
    client: ApiClient,
}

impl ConfigApi {
    /// Create a new config API client
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// Get application configuration
    pub async fn get(&self) -> ApiResult<AppConfig> {
        self.client.get("/config").await
    }

    /// Update application configuration
    pub async fn update(&self, request: &UpdateAppConfigRequest) -> ApiResult<AppConfig> {
        self.client.put("/config", request).await
    }

    /// Set default execution timeout
    pub async fn set_timeout(&self, timeout_secs: u64) -> ApiResult<AppConfig> {
        self.update(&UpdateAppConfigRequest {
            default_timeout_secs: Some(timeout_secs),
            ..Default::default()
        })
        .await
    }

    /// Set max concurrent executions
    pub async fn set_max_concurrent(&self, max: usize) -> ApiResult<AppConfig> {
        self.update(&UpdateAppConfigRequest {
            max_concurrent_executions: Some(max),
            ..Default::default()
        })
        .await
    }

    /// Enable or disable execution history
    pub async fn set_history_enabled(&self, enabled: bool) -> ApiResult<AppConfig> {
        self.update(&UpdateAppConfigRequest {
            enable_history: Some(enabled),
            ..Default::default()
        })
        .await
    }

    /// Set max history entries
    pub async fn set_max_history(&self, max: usize) -> ApiResult<AppConfig> {
        self.update(&UpdateAppConfigRequest {
            max_history_entries: Some(max),
            ..Default::default()
        })
        .await
    }

    /// Health check
    pub async fn health(&self) -> ApiResult<HealthResponse> {
        self.client.get("/health").await
    }

    /// Get version information
    pub async fn version(&self) -> ApiResult<VersionResponse> {
        self.client.get("/version").await
    }

    /// Check if the server is healthy
    pub async fn is_healthy(&self) -> bool {
        match self.health().await {
            Ok(response) => response.healthy,
            Err(_) => false,
        }
    }

    // =========================================================================
    // Manifest Import/Export
    // =========================================================================

    /// Validate a manifest without importing
    pub async fn validate_manifest(&self, content: &str) -> ApiResult<ValidateManifestResponse> {
        self.client
            .post("/manifest/validate", &ValidateManifestRequest {
                content: content.to_string(),
            })
            .await
    }

    /// Import a manifest configuration
    pub async fn import_manifest(
        &self,
        content: &str,
        merge: bool,
        install: bool,
    ) -> ApiResult<ImportManifestResponse> {
        self.client
            .post("/manifest/import", &ImportManifestRequest {
                content: content.to_string(),
                merge,
                install,
            })
            .await
    }

    // =========================================================================
    // Search Configuration
    // =========================================================================

    /// Get search configuration
    pub async fn get_search_config(&self) -> ApiResult<SearchConfigResponse> {
        self.client.get("/search/config").await
    }

    /// Update search configuration
    pub async fn update_search_config(
        &self,
        request: &UpdateSearchConfigRequest,
    ) -> ApiResult<SearchConfigResponse> {
        self.client.put("/search/config", request).await
    }

    /// Set embedding provider
    pub async fn set_embedding_provider(
        &self,
        provider: &str,
        model: Option<&str>,
    ) -> ApiResult<SearchConfigResponse> {
        self.update_search_config(&UpdateSearchConfigRequest {
            embedding_provider: Some(provider.to_string()),
            embedding_model: model.map(String::from),
            ..Default::default()
        })
        .await
    }

    /// Enable or disable hybrid search
    pub async fn set_hybrid_search(&self, enabled: bool) -> ApiResult<SearchConfigResponse> {
        self.update_search_config(&UpdateSearchConfigRequest {
            enable_hybrid: Some(enabled),
            ..Default::default()
        })
        .await
    }

    /// Enable or disable reranking
    pub async fn set_reranking(&self, enabled: bool) -> ApiResult<SearchConfigResponse> {
        self.update_search_config(&UpdateSearchConfigRequest {
            enable_reranking: Some(enabled),
            ..Default::default()
        })
        .await
    }
}
