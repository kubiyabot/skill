//! Search API client

use super::client::ApiClient;
use super::error::ApiResult;
use super::types::*;

/// Search API operations
#[derive(Clone)]
pub struct SearchApi {
    client: ApiClient,
}

impl SearchApi {
    /// Create a new search API client
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// Perform a semantic search
    pub async fn search(&self, request: &SearchRequest) -> ApiResult<SearchResponse> {
        self.client.post("/search", request).await
    }

    /// Simple search with query string
    pub async fn query(&self, query: &str) -> ApiResult<SearchResponse> {
        self.search(&SearchRequest {
            query: query.to_string(),
            top_k: 10,
            skill_filter: None,
            include_examples: false,
        })
        .await
    }

    /// Search with limited results
    pub async fn query_top_k(&self, query: &str, top_k: usize) -> ApiResult<SearchResponse> {
        self.search(&SearchRequest {
            query: query.to_string(),
            top_k,
            skill_filter: None,
            include_examples: false,
        })
        .await
    }

    /// Search within a specific skill
    pub async fn search_in_skill(&self, query: &str, skill: &str) -> ApiResult<SearchResponse> {
        self.search(&SearchRequest {
            query: query.to_string(),
            top_k: 10,
            skill_filter: Some(skill.to_string()),
            include_examples: false,
        })
        .await
    }

    /// Search with AI-generated examples included
    pub async fn search_with_examples(&self, query: &str) -> ApiResult<SearchResponse> {
        self.search(&SearchRequest {
            query: query.to_string(),
            top_k: 10,
            skill_filter: None,
            include_examples: true,
        })
        .await
    }

    /// Get search configuration
    pub async fn get_config(&self) -> ApiResult<SearchConfigResponse> {
        self.client.get("/search/config").await
    }

    /// Update search configuration
    pub async fn update_config(
        &self,
        request: &UpdateSearchConfigRequest,
    ) -> ApiResult<SearchConfigResponse> {
        self.client.put("/search/config", request).await
    }

    /// Enable or disable hybrid search
    pub async fn set_hybrid_search(&self, enabled: bool) -> ApiResult<SearchConfigResponse> {
        self.update_config(&UpdateSearchConfigRequest {
            enable_hybrid: Some(enabled),
            ..Default::default()
        })
        .await
    }

    /// Enable or disable reranking
    pub async fn set_reranking(&self, enabled: bool) -> ApiResult<SearchConfigResponse> {
        self.update_config(&UpdateSearchConfigRequest {
            enable_reranking: Some(enabled),
            ..Default::default()
        })
        .await
    }

    /// Change embedding provider
    pub async fn set_embedding_provider(
        &self,
        provider: &str,
        model: Option<&str>,
    ) -> ApiResult<SearchConfigResponse> {
        self.update_config(&UpdateSearchConfigRequest {
            embedding_provider: Some(provider.to_string()),
            embedding_model: model.map(String::from),
            ..Default::default()
        })
        .await
    }

    /// Change vector backend
    pub async fn set_vector_backend(&self, backend: &str) -> ApiResult<SearchConfigResponse> {
        self.update_config(&UpdateSearchConfigRequest {
            vector_backend: Some(backend.to_string()),
            ..Default::default()
        })
        .await
    }

    /// Test search connection (quick validation)
    pub async fn test_connection(
        &self,
        request: &TestConnectionRequest,
    ) -> ApiResult<TestConnectionResponse> {
        self.client.post("/search/test-connection", request).await
    }

    /// Test full search pipeline (indexing + search)
    pub async fn test_pipeline(
        &self,
        request: &TestPipelineRequest,
    ) -> ApiResult<TestPipelineResponse> {
        self.client.post("/search/test-pipeline", request).await
    }

    /// Index all skills into the search pipeline
    pub async fn index(&self) -> ApiResult<IndexResponse> {
        self.client.post("/search/index", &()).await
    }
}
