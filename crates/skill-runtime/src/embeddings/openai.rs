//! OpenAI embedding provider implementation
//!
//! Uses rig-core's OpenAI client for API-based embeddings.
//! Requires OPENAI_API_KEY environment variable.

use super::{EmbeddingProvider, OpenAIEmbeddingModel};
use anyhow::{Context, Result};
use async_trait::async_trait;
use rig::embeddings::EmbeddingModel as RigEmbeddingModel;
use rig::client::{EmbeddingsClient, ProviderClient};
use rig::providers::openai::{self, Client as OpenAIClient};
use std::sync::Arc;

/// OpenAI embedding provider
///
/// Generates embeddings via OpenAI's API.
/// Requires OPENAI_API_KEY environment variable to be set.
pub struct OpenAIEmbedProvider {
    client: Arc<OpenAIClient>,
    model: OpenAIEmbeddingModel,
    dims: usize,
}

impl OpenAIEmbedProvider {
    /// Create a new OpenAI provider with the default model (Ada002)
    ///
    /// # Errors
    /// Returns error if OPENAI_API_KEY is not set
    pub fn new() -> Result<Self> {
        Self::with_model(OpenAIEmbeddingModel::default())
    }

    /// Create a new OpenAI provider with a specific model
    pub fn with_model(model: OpenAIEmbeddingModel) -> Result<Self> {
        // Check for API key
        std::env::var("OPENAI_API_KEY").context(
            "OPENAI_API_KEY environment variable not set. Set it with: export OPENAI_API_KEY=your-key-here"
        )?;

        let client = Arc::new(OpenAIClient::from_env());
        let dims = model.dimensions();

        Ok(Self {
            client,
            model,
            dims,
        })
    }

    /// Create with a custom API key
    pub fn with_api_key(api_key: &str, model: OpenAIEmbeddingModel) -> Result<Self> {
        let client = Arc::new(OpenAIClient::new(api_key).context("Failed to create OpenAI client")?);
        let dims = model.dimensions();

        Ok(Self {
            client,
            model,
            dims,
        })
    }

    /// Create from a model name string
    pub fn from_model_name(name: &str) -> Result<Self> {
        let model: OpenAIEmbeddingModel = name.parse()?;
        Self::with_model(model)
    }

    /// Get the API model name
    fn api_model_name(&self) -> &'static str {
        match self.model {
            OpenAIEmbeddingModel::Ada002 => openai::TEXT_EMBEDDING_ADA_002,
            OpenAIEmbeddingModel::TextEmbedding3Small => "text-embedding-3-small",
            OpenAIEmbeddingModel::TextEmbedding3Large => "text-embedding-3-large",
        }
    }

}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbedProvider {
    async fn embed_documents(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let embedding_model = self.client.embedding_model(self.api_model_name());

        // Use rig's embed method
        let embeddings = embedding_model
            .embed_texts(texts)
            .await
            .context("OpenAI failed to generate embeddings")?;

        // Convert from rig's Embedding type to Vec<f32>
        let results: Vec<Vec<f32>> = embeddings
            .into_iter()
            .map(|emb| emb.vec.into_iter().map(|x| x as f32).collect())
            .collect();

        Ok(results)
    }

    fn dimensions(&self) -> usize {
        self.dims
    }

    fn model_name(&self) -> &str {
        self.api_model_name()
    }

    fn provider_name(&self) -> &str {
        "openai"
    }

    fn max_batch_size(&self) -> usize {
        // OpenAI API limit is 2048 texts per request
        2048
    }

    async fn health_check(&self) -> Result<bool> {
        // Try a minimal embedding to verify API key works
        match self.embed_query("test").await {
            Ok(emb) => Ok(emb.len() == self.dims),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_dimensions() {
        // Just test the dimensions without actually creating a provider (needs API key)
        assert_eq!(OpenAIEmbeddingModel::Ada002.dimensions(), 1536);
        assert_eq!(OpenAIEmbeddingModel::TextEmbedding3Small.dimensions(), 1536);
        assert_eq!(OpenAIEmbeddingModel::TextEmbedding3Large.dimensions(), 3072);
    }

    #[test]
    fn test_api_model_names() {
        assert_eq!(OpenAIEmbeddingModel::Ada002.api_name(), "text-embedding-ada-002");
        assert_eq!(OpenAIEmbeddingModel::TextEmbedding3Small.api_name(), "text-embedding-3-small");
        assert_eq!(OpenAIEmbeddingModel::TextEmbedding3Large.api_name(), "text-embedding-3-large");
    }

    // Integration test - requires API key
    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY"]
    async fn test_embed_documents() {
        let provider = OpenAIEmbedProvider::new().unwrap();
        let texts = vec![
            "Hello world".to_string(),
            "How are you".to_string(),
        ];

        let embeddings = provider.embed_documents(texts).await.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), provider.dimensions());
    }

    #[test]
    fn test_missing_api_key() {
        // Temporarily unset the API key
        let original = std::env::var("OPENAI_API_KEY").ok();
        std::env::remove_var("OPENAI_API_KEY");

        let result = OpenAIEmbedProvider::new();
        assert!(result.is_err());

        // Restore if it was set
        if let Some(key) = original {
            std::env::set_var("OPENAI_API_KEY", key);
        }
    }
}
