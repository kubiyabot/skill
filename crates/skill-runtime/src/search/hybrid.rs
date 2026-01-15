//! Hybrid retrieval combining dense and sparse search
//!
//! Provides a unified interface for combining vector similarity search
//! with BM25 keyword search using configurable fusion methods.

use super::bm25::{BM25Index, BM25Config};
use super::fusion::{FusionMethod, reciprocal_rank_fusion, weighted_sum_fusion, max_score_fusion};
use crate::embeddings::EmbeddingProvider;
use crate::vector_store::{Filter, SearchResult, VectorStore};
use anyhow::{Context, Result};
use std::sync::Arc;

/// Configuration for hybrid search
#[derive(Debug, Clone)]
pub struct HybridConfig {
    /// Weight for dense (vector) search results (0.0-1.0)
    pub dense_weight: f32,
    /// Weight for sparse (BM25) search results (0.0-1.0)
    pub sparse_weight: f32,
    /// RRF k parameter (typically 60)
    pub rrf_k: f32,
    /// Fusion method
    pub fusion_method: FusionMethod,
    /// Multiplier for initial retrieval (retrieve top_k * multiplier from each source)
    pub retrieval_multiplier: usize,
    /// BM25 configuration
    pub bm25_config: BM25Config,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            dense_weight: 0.7,
            sparse_weight: 0.3,
            rrf_k: 60.0,
            fusion_method: FusionMethod::ReciprocalRank,
            retrieval_multiplier: 3,
            bm25_config: BM25Config::in_memory(),
        }
    }
}

impl HybridConfig {
    /// Create config with custom weights
    pub fn with_weights(dense_weight: f32, sparse_weight: f32) -> Self {
        Self {
            dense_weight,
            sparse_weight,
            ..Default::default()
        }
    }

    /// Set the fusion method
    pub fn with_fusion(mut self, method: FusionMethod) -> Self {
        self.fusion_method = method;
        self
    }

    /// Set the RRF k parameter
    pub fn with_rrf_k(mut self, k: f32) -> Self {
        self.rrf_k = k;
        self
    }
}

/// Result from hybrid search
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    /// Document ID
    pub id: String,
    /// Combined score after fusion
    pub score: f32,
    /// Original dense search score (if found)
    pub dense_score: Option<f32>,
    /// Original sparse search score (if found)
    pub sparse_score: Option<f32>,
    /// Metadata from the result (if available from dense search)
    pub metadata: Option<crate::vector_store::DocumentMetadata>,
}

/// Hybrid retriever combining dense and sparse search
pub struct HybridRetriever<V: VectorStore, E: EmbeddingProvider> {
    /// Dense vector store
    dense_store: Arc<V>,
    /// Sparse BM25 index
    sparse_index: BM25Index,
    /// Embedding provider for query embedding
    embedding_provider: Arc<E>,
    /// Configuration
    config: HybridConfig,
}

impl<V: VectorStore, E: EmbeddingProvider> HybridRetriever<V, E> {
    /// Create a new hybrid retriever
    pub fn new(
        dense_store: Arc<V>,
        embedding_provider: Arc<E>,
        config: HybridConfig,
    ) -> Result<Self> {
        let sparse_index = BM25Index::new(config.bm25_config.clone())
            .context("Failed to create BM25 index")?;

        Ok(Self {
            dense_store,
            sparse_index,
            embedding_provider,
            config,
        })
    }

    /// Create with existing BM25 index
    pub fn with_index(
        dense_store: Arc<V>,
        sparse_index: BM25Index,
        embedding_provider: Arc<E>,
        config: HybridConfig,
    ) -> Self {
        Self {
            dense_store,
            sparse_index,
            embedding_provider,
            config,
        }
    }

    /// Add a document to the sparse index
    pub fn add_to_sparse_index(
        &mut self,
        id: &str,
        tool_name: &str,
        skill_name: &str,
        description: &str,
        full_text: &str,
    ) -> Result<()> {
        self.sparse_index.add_document(id, tool_name, skill_name, description, full_text)
    }

    /// Commit changes to the sparse index
    pub fn commit_sparse_index(&mut self) -> Result<()> {
        self.sparse_index.commit()
    }

    /// Clear the sparse index
    pub fn clear_sparse_index(&mut self) -> Result<()> {
        self.sparse_index.clear()
    }

    /// Search using hybrid retrieval
    pub async fn search(
        &self,
        query: &str,
        filter: Option<Filter>,
        top_k: usize,
    ) -> Result<Vec<HybridSearchResult>> {
        let expanded_k = top_k * self.config.retrieval_multiplier;

        // Stage 1: Dense retrieval
        let query_embedding = self.embedding_provider
            .embed_query(query)
            .await
            .context("Failed to embed query")?;

        let dense_results = self.dense_store
            .search(query_embedding, filter, expanded_k)
            .await
            .context("Dense search failed")?;

        // Stage 2: Sparse retrieval
        let sparse_results = self.sparse_index
            .search(query, expanded_k)
            .context("Sparse search failed")?;

        // Convert to ranked lists for fusion
        let dense_ranked: Vec<(String, f32)> = dense_results
            .iter()
            .map(|r| (r.id.clone(), r.score))
            .collect();

        let sparse_ranked: Vec<(String, f32)> = sparse_results
            .iter()
            .map(|r| (r.id.clone(), r.score))
            .collect();

        // Stage 3: Fuse results
        let fused_results = match self.config.fusion_method {
            FusionMethod::ReciprocalRank => reciprocal_rank_fusion(
                vec![("dense", dense_ranked), ("sparse", sparse_ranked)],
                self.config.rrf_k,
                top_k,
            ),
            FusionMethod::WeightedSum => weighted_sum_fusion(
                vec![
                    ("dense", self.config.dense_weight, dense_ranked),
                    ("sparse", self.config.sparse_weight, sparse_ranked),
                ],
                top_k,
            ),
            FusionMethod::MaxScore => max_score_fusion(
                vec![("dense", dense_ranked), ("sparse", sparse_ranked)],
                top_k,
            ),
        };

        // Build final results with metadata from dense results
        let dense_metadata: std::collections::HashMap<_, _> = dense_results
            .into_iter()
            .map(|r| (r.id, r.metadata))
            .collect();

        let results = fused_results
            .into_iter()
            .map(|fused| {
                HybridSearchResult {
                    id: fused.id.clone(),
                    score: fused.score,
                    dense_score: fused.source_scores.get("dense").copied(),
                    sparse_score: fused.source_scores.get("sparse").copied(),
                    metadata: dense_metadata.get(&fused.id).cloned(),
                }
            })
            .collect();

        Ok(results)
    }

    /// Search with only dense retrieval (for comparison/fallback)
    pub async fn dense_only(
        &self,
        query: &str,
        filter: Option<Filter>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embedding_provider
            .embed_query(query)
            .await
            .context("Failed to embed query")?;

        self.dense_store.search(query_embedding, filter, top_k).await
    }

    /// Search with only sparse retrieval (for comparison/fallback)
    pub fn sparse_only(&self, query: &str, top_k: usize) -> Result<Vec<super::bm25::BM25SearchResult>> {
        self.sparse_index.search(query, top_k)
    }

    /// Get the configuration
    pub fn config(&self) -> &HybridConfig {
        &self.config
    }

    /// Get sparse index document count
    pub fn sparse_document_count(&self) -> u64 {
        self.sparse_index.document_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests require mock implementations
    // Here we test configuration and fusion logic

    #[test]
    fn test_config_default() {
        let config = HybridConfig::default();
        assert!((config.dense_weight - 0.7).abs() < 0.001);
        assert!((config.sparse_weight - 0.3).abs() < 0.001);
        assert_eq!(config.fusion_method, FusionMethod::ReciprocalRank);
    }

    #[test]
    fn test_config_builder() {
        let config = HybridConfig::with_weights(0.5, 0.5)
            .with_fusion(FusionMethod::WeightedSum)
            .with_rrf_k(30.0);

        assert!((config.dense_weight - 0.5).abs() < 0.001);
        assert!((config.sparse_weight - 0.5).abs() < 0.001);
        assert_eq!(config.fusion_method, FusionMethod::WeightedSum);
        assert!((config.rrf_k - 30.0).abs() < 0.001);
    }
}
