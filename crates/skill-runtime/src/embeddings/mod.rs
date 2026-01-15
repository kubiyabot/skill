//! Embedding provider abstraction for vector generation
//!
//! This module provides a trait-based abstraction for embedding generation,
//! supporting multiple providers (FastEmbed, OpenAI, Ollama) with a unified interface.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │                  EmbeddingProvider Trait                     │
//! │  embed_documents, embed_query, dimensions, model_name       │
//! └──────────────────────────────────────────────────────────────┘
//!                              │
//!          ┌───────────────────┼───────────────────┐
//!          ▼                   ▼                   ▼
//!   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
//!   │  FastEmbed  │    │   OpenAI    │    │   Ollama    │
//!   │  (local)    │    │   (API)     │    │  (local)    │
//!   └─────────────┘    └─────────────┘    └─────────────┘
//! ```
//!
//! # Example
//!
//! ```ignore
//! use skill_runtime::embeddings::{EmbeddingProvider, FastEmbedProvider, EmbeddingConfig};
//!
//! // Create a provider
//! let provider = FastEmbedProvider::new(FastEmbedModel::AllMiniLM)?;
//!
//! // Embed a query
//! let query_embedding = provider.embed_query("search for kubernetes tools").await?;
//!
//! // Embed multiple documents
//! let texts = vec!["doc1".to_string(), "doc2".to_string()];
//! let embeddings = provider.embed_documents(texts).await?;
//! ```

mod types;
mod fastembed;
mod openai;
mod ollama;
mod factory;

pub use types::*;
pub use fastembed::FastEmbedProvider;
pub use openai::OpenAIEmbedProvider;
pub use ollama::OllamaProvider;
pub use factory::{EmbeddingProviderFactory, create_provider};

use async_trait::async_trait;
use anyhow::Result;

/// Trait for embedding generation providers
///
/// Implementors generate vector embeddings from text, supporting both
/// single queries and batch document processing.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embeddings for multiple documents
    ///
    /// # Arguments
    /// * `texts` - List of text documents to embed
    ///
    /// # Returns
    /// Vector of embeddings, one per input document, in the same order
    async fn embed_documents(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>>;

    /// Generate embedding for a single query
    ///
    /// Some providers optimize query embeddings differently than document embeddings.
    /// By default, this calls embed_documents with a single item.
    ///
    /// # Arguments
    /// * `text` - The query text to embed
    async fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        let results = self.embed_documents(vec![text.to_string()]).await?;
        results.into_iter().next().ok_or_else(|| {
            anyhow::anyhow!("embed_documents returned empty result for single query")
        })
    }

    /// Get the embedding dimension size
    fn dimensions(&self) -> usize;

    /// Get the model name/identifier
    fn model_name(&self) -> &str;

    /// Get the provider name (e.g., "fastembed", "openai", "ollama")
    fn provider_name(&self) -> &str;

    /// Check if the provider is available (API key set, server running, etc.)
    async fn health_check(&self) -> Result<bool> {
        // Default: try to embed a simple query
        match self.embed_query("test").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get the maximum batch size for embed_documents
    fn max_batch_size(&self) -> usize {
        100 // Default, can be overridden
    }

    /// Embed documents in batches, respecting max_batch_size
    async fn embed_documents_batched(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let batch_size = self.max_batch_size();
        if texts.len() <= batch_size {
            return self.embed_documents(texts).await;
        }

        let mut all_embeddings = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(batch_size) {
            let embeddings = self.embed_documents(chunk.to_vec()).await?;
            all_embeddings.extend(embeddings);
        }
        Ok(all_embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock provider for testing
    struct MockProvider {
        dims: usize,
    }

    #[async_trait]
    impl EmbeddingProvider for MockProvider {
        async fn embed_documents(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
            Ok(texts
                .iter()
                .map(|_| vec![0.1; self.dims])
                .collect())
        }

        fn dimensions(&self) -> usize {
            self.dims
        }

        fn model_name(&self) -> &str {
            "mock-model"
        }

        fn provider_name(&self) -> &str {
            "mock"
        }

        fn max_batch_size(&self) -> usize {
            2
        }
    }

    #[tokio::test]
    async fn test_embed_query_default() {
        let provider = MockProvider { dims: 384 };
        let embedding = provider.embed_query("test query").await.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    async fn test_embed_documents_batched() {
        let provider = MockProvider { dims: 3 };
        let texts: Vec<String> = (0..5).map(|i| format!("doc{}", i)).collect();

        let embeddings = provider.embed_documents_batched(texts).await.unwrap();
        assert_eq!(embeddings.len(), 5);
        for emb in embeddings {
            assert_eq!(emb.len(), 3);
        }
    }

    #[tokio::test]
    async fn test_health_check_default() {
        let provider = MockProvider { dims: 3 };
        let healthy = provider.health_check().await.unwrap();
        assert!(healthy);
    }
}
