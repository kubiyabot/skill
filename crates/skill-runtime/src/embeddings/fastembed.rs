//! FastEmbed embedding provider implementation
//!
//! Uses rig-fastembed for local ONNX-based embeddings.
//! No API key required - models are downloaded and cached locally.

use super::{EmbeddingProvider, FastEmbedModel};
use anyhow::{Context, Result};
use async_trait::async_trait;
use rig::embeddings::embedding::EmbeddingModel as RigEmbeddingModel;
use rig_fastembed::{Client as FastembedClient, FastembedModel as RigFastembedModel};
use std::sync::Arc;

/// FastEmbed embedding provider
///
/// Generates embeddings locally using ONNX runtime.
/// Models are downloaded on first use and cached in ~/.fastembed_cache/
pub struct FastEmbedProvider {
    client: Arc<FastembedClient>,
    model: FastEmbedModel,
    rig_model: RigFastembedModel,
    dims: usize,
}

impl FastEmbedProvider {
    /// Create a new FastEmbed provider with the default model (AllMiniLM)
    pub fn new() -> Result<Self> {
        Self::with_model(FastEmbedModel::default())
    }

    /// Create a new FastEmbed provider with a specific model
    pub fn with_model(model: FastEmbedModel) -> Result<Self> {
        let client = Arc::new(FastembedClient::new());
        let rig_model = Self::to_rig_model(&model);
        let dims = model.dimensions();

        Ok(Self {
            client,
            model,
            rig_model,
            dims,
        })
    }

    /// Create from a model name string
    pub fn from_model_name(name: &str) -> Result<Self> {
        let model: FastEmbedModel = name.parse()?;
        Self::with_model(model)
    }

    /// Convert our model enum to rig's model enum
    fn to_rig_model(model: &FastEmbedModel) -> RigFastembedModel {
        match model {
            FastEmbedModel::AllMiniLM => RigFastembedModel::AllMiniLML6V2Q,
            FastEmbedModel::BGESmallEN => RigFastembedModel::BGESmallENV15Q,
            FastEmbedModel::BGEBaseEN => RigFastembedModel::BGEBaseENV15,
            FastEmbedModel::BGELargeEN => RigFastembedModel::BGELargeENV15,
        }
    }

}

impl Default for FastEmbedProvider {
    fn default() -> Self {
        Self::new().expect("Failed to create default FastEmbed provider")
    }
}

#[async_trait]
impl EmbeddingProvider for FastEmbedProvider {
    async fn embed_documents(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let embedding_model = self.client.embedding_model(&self.rig_model);

        // Use rig's embed method
        let embeddings = embedding_model
            .embed_texts(texts)
            .await
            .context("FastEmbed failed to generate embeddings")?;

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
        self.model.rig_model_name()
    }

    fn provider_name(&self) -> &str {
        "fastembed"
    }

    fn max_batch_size(&self) -> usize {
        // FastEmbed handles batching internally, but we cap for memory reasons
        256
    }

    async fn health_check(&self) -> Result<bool> {
        // FastEmbed is always available (local), just check if we can create embeddings
        match self.embed_query("health check").await {
            Ok(emb) => Ok(emb.len() == self.dims),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_conversion() {
        // Test that all models convert correctly
        let _ = FastEmbedProvider::to_rig_model(&FastEmbedModel::AllMiniLM);
        let _ = FastEmbedProvider::to_rig_model(&FastEmbedModel::BGESmallEN);
        let _ = FastEmbedProvider::to_rig_model(&FastEmbedModel::BGEBaseEN);
        let _ = FastEmbedProvider::to_rig_model(&FastEmbedModel::BGELargeEN);
    }

    #[test]
    fn test_provider_creation() {
        let provider = FastEmbedProvider::new().unwrap();
        assert_eq!(provider.dimensions(), 384);
        assert_eq!(provider.model_name(), "all-minilm");
        assert_eq!(provider.provider_name(), "fastembed");
    }

    #[test]
    fn test_from_model_name() {
        let provider = FastEmbedProvider::from_model_name("bge-small").unwrap();
        assert_eq!(provider.dimensions(), 384);

        let provider = FastEmbedProvider::from_model_name("bge-base").unwrap();
        assert_eq!(provider.dimensions(), 768);
    }

    // Integration test - requires model download, so marked ignore
    #[tokio::test]
    #[ignore = "requires model download"]
    async fn test_embed_documents() {
        let provider = FastEmbedProvider::new().unwrap();
        let texts = vec![
            "Hello world".to_string(),
            "How are you".to_string(),
        ];

        let embeddings = provider.embed_documents(texts).await.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 384);
        assert_eq!(embeddings[1].len(), 384);
    }

    #[tokio::test]
    async fn test_embed_empty() {
        let provider = FastEmbedProvider::new().unwrap();
        let embeddings = provider.embed_documents(vec![]).await.unwrap();
        assert!(embeddings.is_empty());
    }
}
