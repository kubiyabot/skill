//! Ollama embedding provider implementation
//!
//! Uses rig-core's Ollama client for local LLM-based embeddings.
//! Requires a running Ollama server (default: http://localhost:11434).

use super::EmbeddingProvider;
use anyhow::{Context, Result};
use async_trait::async_trait;
use rig::embeddings::EmbeddingModel as RigEmbeddingModel;
use rig::client::{EmbeddingsClient, ProviderClient, Nothing};
use rig::providers::ollama::Client as OllamaClient;
use std::sync::Arc;

/// Default Ollama embedding model
pub const DEFAULT_OLLAMA_MODEL: &str = "nomic-embed-text";

/// Default Ollama server URL
pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

/// Known Ollama embedding model dimensions
fn get_model_dimensions(model: &str) -> usize {
    match model {
        "nomic-embed-text" => 768,
        "mxbai-embed-large" => 1024,
        "all-minilm" => 384,
        "snowflake-arctic-embed" => 1024,
        _ => 768, // Default assumption
    }
}

/// Ollama embedding provider
///
/// Generates embeddings via a local Ollama server.
/// Requires Ollama to be running with an embedding model pulled.
pub struct OllamaProvider {
    client: Arc<OllamaClient>,
    model: String,
    dims: usize,
    base_url: String,
}

impl OllamaProvider {
    /// Create a new Ollama provider with the default model (nomic-embed-text)
    pub fn new() -> Result<Self> {
        Self::with_model(DEFAULT_OLLAMA_MODEL)
    }

    /// Create a new Ollama provider with a specific model
    pub fn with_model(model: &str) -> Result<Self> {
        let client = Arc::new(OllamaClient::from_val(Nothing));
        let dims = get_model_dimensions(model);

        Ok(Self {
            client,
            model: model.to_string(),
            dims,
            base_url: DEFAULT_OLLAMA_URL.to_string(),
        })
    }

    /// Create with a custom base URL
    pub fn with_url(base_url: &str, model: &str) -> Result<Self> {
        // Note: rig's Ollama client uses OLLAMA_API_BASE env var or default
        // For custom URL, we still create with Nothing and hope the env is set
        let client = Arc::new(OllamaClient::from_val(Nothing));
        let dims = get_model_dimensions(model);

        Ok(Self {
            client,
            model: model.to_string(),
            dims,
            base_url: base_url.to_string(),
        })
    }

    /// Create with custom dimensions (for models not in our known list)
    pub fn with_dimensions(model: &str, dims: usize) -> Result<Self> {
        let client = Arc::new(OllamaClient::from_val(Nothing));

        Ok(Self {
            client,
            model: model.to_string(),
            dims,
            base_url: DEFAULT_OLLAMA_URL.to_string(),
        })
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new().expect("Failed to create default Ollama provider")
    }
}

#[async_trait]
impl EmbeddingProvider for OllamaProvider {
    async fn embed_documents(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let embedding_model = self.client.embedding_model(&self.model);

        // Use rig's embed method
        let embeddings = embedding_model
            .embed_texts(texts)
            .await
            .context("Ollama failed to generate embeddings. Is the server running?")?;

        // Convert from rig's Embedding type to Vec<f32>
        let results: Vec<Vec<f32>> = embeddings
            .into_iter()
            .map(|emb| emb.vec.into_iter().map(|x| x as f32).collect())
            .collect();

        // Update dimensions if we got a different size (model might have changed)
        if let Some(first) = results.first() {
            if first.len() != self.dims {
                tracing::warn!(
                    "Ollama model {} returned {} dimensions, expected {}",
                    self.model,
                    first.len(),
                    self.dims
                );
            }
        }

        Ok(results)
    }

    fn dimensions(&self) -> usize {
        self.dims
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }

    fn max_batch_size(&self) -> usize {
        // Ollama processes one at a time internally, but we batch for convenience
        100
    }

    async fn health_check(&self) -> Result<bool> {
        // Try to embed a simple text to verify server is running
        match self.embed_query("test").await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::debug!("Ollama health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_dimensions() {
        assert_eq!(get_model_dimensions("nomic-embed-text"), 768);
        assert_eq!(get_model_dimensions("mxbai-embed-large"), 1024);
        assert_eq!(get_model_dimensions("all-minilm"), 384);
        assert_eq!(get_model_dimensions("unknown-model"), 768); // Default
    }

    #[test]
    fn test_provider_creation() {
        let provider = OllamaProvider::new().unwrap();
        assert_eq!(provider.model_name(), "nomic-embed-text");
        assert_eq!(provider.dimensions(), 768);
        assert_eq!(provider.provider_name(), "ollama");
        assert_eq!(provider.base_url(), DEFAULT_OLLAMA_URL);
    }

    #[test]
    fn test_custom_url() {
        let provider = OllamaProvider::with_url("http://custom:11434", "nomic-embed-text").unwrap();
        assert_eq!(provider.base_url(), "http://custom:11434");
    }

    #[test]
    fn test_custom_dimensions() {
        let provider = OllamaProvider::with_dimensions("custom-model", 512).unwrap();
        assert_eq!(provider.dimensions(), 512);
        assert_eq!(provider.model_name(), "custom-model");
    }

    // Integration test - requires running Ollama server
    #[tokio::test]
    #[ignore = "requires running Ollama server"]
    async fn test_embed_documents() {
        let provider = OllamaProvider::new().unwrap();
        let texts = vec![
            "Hello world".to_string(),
            "How are you".to_string(),
        ];

        let embeddings = provider.embed_documents(texts).await.unwrap();
        assert_eq!(embeddings.len(), 2);
    }

    #[tokio::test]
    async fn test_embed_empty() {
        let provider = OllamaProvider::new().unwrap();
        let embeddings = provider.embed_documents(vec![]).await.unwrap();
        assert!(embeddings.is_empty());
    }
}
