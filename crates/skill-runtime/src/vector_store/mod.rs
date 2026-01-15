//! Vector Store abstraction for pluggable vector database backends
//!
//! This module provides a trait-based abstraction for vector storage,
//! enabling different backends (in-memory, Qdrant, Pinecone, etc.)
//! to be used interchangeably for semantic skill search.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │                    VectorStore Trait                         │
//! │  upsert, search, delete, get, count, health_check           │
//! └──────────────────────────────────────────────────────────────┘
//!                              │
//!          ┌───────────────────┼───────────────────┐
//!          ▼                   ▼                   ▼
//!   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
//!   │  InMemory   │    │   Qdrant    │    │  Pinecone   │
//!   │  (default)  │    │   (local/   │    │   (cloud)   │
//!   │             │    │    cloud)   │    │             │
//!   └─────────────┘    └─────────────┘    └─────────────┘
//! ```
//!
//! # Example
//!
//! ```ignore
//! use skill_runtime::vector_store::{VectorStore, InMemoryVectorStore, EmbeddedDocument};
//!
//! let store = InMemoryVectorStore::new();
//!
//! // Upsert documents
//! let docs = vec![EmbeddedDocument::new("id1", vec![0.1, 0.2, 0.3])];
//! store.upsert(docs).await?;
//!
//! // Search
//! let results = store.search(vec![0.1, 0.2, 0.3], None, 5).await?;
//! ```

mod types;
mod in_memory;
mod file;

#[cfg(feature = "qdrant")]
mod qdrant;

pub use types::*;
pub use in_memory::InMemoryVectorStore;
pub use file::{FileVectorStore, FileConfig};

#[cfg(feature = "qdrant")]
pub use qdrant::{QdrantVectorStore, QdrantConfig};

use async_trait::async_trait;
use anyhow::Result;

/// Trait for vector storage backends
///
/// Implementors provide vector similarity search with metadata filtering.
/// All operations are async to support both local and remote backends.
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Insert or update documents in the store
    ///
    /// Documents with existing IDs will be updated, new IDs will be inserted.
    async fn upsert(&self, documents: Vec<EmbeddedDocument>) -> Result<UpsertStats>;

    /// Search for similar vectors
    ///
    /// # Arguments
    /// * `query_embedding` - The query vector to find similar documents for
    /// * `filter` - Optional metadata filter to narrow results
    /// * `top_k` - Maximum number of results to return
    ///
    /// # Returns
    /// Results sorted by descending similarity score
    async fn search(
        &self,
        query_embedding: Vec<f32>,
        filter: Option<Filter>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Delete documents by ID
    async fn delete(&self, ids: Vec<String>) -> Result<DeleteStats>;

    /// Get documents by ID
    async fn get(&self, ids: Vec<String>) -> Result<Vec<EmbeddedDocument>>;

    /// Count documents, optionally filtered
    async fn count(&self, filter: Option<Filter>) -> Result<usize>;

    /// Check backend health/connectivity
    async fn health_check(&self) -> Result<HealthStatus>;

    /// Get the name of this backend (for logging/debugging)
    fn backend_name(&self) -> &'static str;

    /// Get configured vector dimensions (if known)
    fn dimensions(&self) -> Option<usize> {
        None
    }
}

/// Compute cosine similarity between two vectors
///
/// Returns a value between -1 and 1, where 1 means identical direction,
/// 0 means orthogonal, and -1 means opposite direction.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

/// Compute euclidean distance between two vectors
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::MAX;
    }

    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let a: Vec<f32> = vec![];
        let b: Vec<f32> = vec![];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_cosine_similarity_different_lengths() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_euclidean_distance_same_point() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        assert!(euclidean_distance(&a, &b) < 1e-6);
    }

    #[test]
    fn test_euclidean_distance_unit() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((euclidean_distance(&a, &b) - 1.0).abs() < 1e-6);
    }
}
