//! In-memory vector store implementation
//!
//! This is the default backend that stores all vectors in memory.
//! Suitable for development, testing, and small-to-medium workloads.
//!
//! # Features
//!
//! - Zero external dependencies
//! - Fast for small datasets (<10k documents)
//! - Thread-safe with RwLock
//! - Supports all filter operations
//! - Cosine, Euclidean, and Dot Product distance metrics
//!
//! # Limitations
//!
//! - All data is lost on process restart (no persistence)
//! - Memory usage grows linearly with documents
//! - O(n) search complexity (no indexing)
//! - Not suitable for >100k documents

use super::{
    cosine_similarity, euclidean_distance, DeleteStats, DistanceMetric, EmbeddedDocument, Filter,
    HealthStatus, SearchResult, UpsertStats, VectorStore,
};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;

/// In-memory vector store implementation
///
/// Stores documents in a HashMap protected by RwLock for thread-safety.
/// Uses brute-force similarity search (suitable for small datasets).
pub struct InMemoryVectorStore {
    /// Document storage: id -> document
    documents: RwLock<HashMap<String, EmbeddedDocument>>,

    /// Distance metric to use for similarity
    distance_metric: DistanceMetric,

    /// Expected vector dimensions (for validation)
    dimensions: Option<usize>,
}

impl InMemoryVectorStore {
    /// Create a new in-memory vector store with default settings
    pub fn new() -> Self {
        Self {
            documents: RwLock::new(HashMap::new()),
            distance_metric: DistanceMetric::Cosine,
            dimensions: None,
        }
    }

    /// Create with a specific distance metric
    pub fn with_metric(metric: DistanceMetric) -> Self {
        Self {
            documents: RwLock::new(HashMap::new()),
            distance_metric: metric,
            dimensions: None,
        }
    }

    /// Create with expected dimensions for validation
    pub fn with_dimensions(dimensions: usize) -> Self {
        Self {
            documents: RwLock::new(HashMap::new()),
            distance_metric: DistanceMetric::Cosine,
            dimensions: Some(dimensions),
        }
    }

    /// Create with both metric and dimensions
    pub fn with_config(metric: DistanceMetric, dimensions: usize) -> Self {
        Self {
            documents: RwLock::new(HashMap::new()),
            distance_metric: metric,
            dimensions: Some(dimensions),
        }
    }

    /// Calculate similarity between two vectors based on configured metric
    fn calculate_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        match self.distance_metric {
            DistanceMetric::Cosine => cosine_similarity(a, b),
            DistanceMetric::Euclidean => {
                // Convert distance to similarity (0 distance = 1 similarity)
                let dist = euclidean_distance(a, b);
                1.0 / (1.0 + dist)
            }
            DistanceMetric::DotProduct => {
                // Dot product directly (assumes normalized vectors)
                a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
            }
        }
    }

    /// Validate document dimensions
    fn validate_dimensions(&self, embedding: &[f32]) -> Result<()> {
        if let Some(expected) = self.dimensions {
            if embedding.len() != expected {
                anyhow::bail!(
                    "Embedding dimension mismatch: expected {}, got {}",
                    expected,
                    embedding.len()
                );
            }
        }
        Ok(())
    }

    /// Get current document count (sync version for internal use)
    fn document_count(&self) -> usize {
        self.documents.read().unwrap().len()
    }

    /// Clear all documents
    pub fn clear(&self) {
        let mut docs = self.documents.write().unwrap();
        docs.clear();
    }
}

impl Default for InMemoryVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn upsert(&self, documents: Vec<EmbeddedDocument>) -> Result<UpsertStats> {
        let start = Instant::now();
        let mut inserted = 0;
        let mut updated = 0;

        // Validate all documents first
        for doc in &documents {
            self.validate_dimensions(&doc.embedding)?;
        }

        // Insert/update documents
        let mut store = self.documents.write().unwrap();
        for doc in documents {
            if store.contains_key(&doc.id) {
                updated += 1;
            } else {
                inserted += 1;
            }
            store.insert(doc.id.clone(), doc);
        }

        Ok(UpsertStats::new(inserted, updated, start.elapsed().as_millis() as u64))
    }

    async fn search(
        &self,
        query_embedding: Vec<f32>,
        filter: Option<Filter>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        self.validate_dimensions(&query_embedding)?;

        let store = self.documents.read().unwrap();

        // Calculate similarity for all documents
        let mut scored: Vec<(f32, &EmbeddedDocument)> = store
            .values()
            .filter(|doc| {
                // Apply metadata filter
                filter
                    .as_ref()
                    .map_or(true, |f| f.matches(&doc.metadata))
            })
            .map(|doc| {
                let score = self.calculate_similarity(&query_embedding, &doc.embedding);
                (score, doc)
            })
            .filter(|(score, _)| {
                // Apply min_score filter
                filter
                    .as_ref()
                    .and_then(|f| f.min_score)
                    .map_or(true, |min| *score >= min)
            })
            .collect();

        // Sort by descending score
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Take top_k and convert to SearchResult
        let results: Vec<SearchResult> = scored
            .into_iter()
            .take(top_k)
            .map(|(score, doc)| SearchResult::from_document(doc, score))
            .collect();

        Ok(results)
    }

    async fn delete(&self, ids: Vec<String>) -> Result<DeleteStats> {
        let start = Instant::now();
        let mut deleted = 0;
        let mut not_found = 0;

        let mut store = self.documents.write().unwrap();
        for id in &ids {
            if store.remove(id).is_some() {
                deleted += 1;
            } else {
                not_found += 1;
            }
        }

        Ok(DeleteStats::new(deleted, not_found, start.elapsed().as_millis() as u64))
    }

    async fn get(&self, ids: Vec<String>) -> Result<Vec<EmbeddedDocument>> {
        let store = self.documents.read().unwrap();
        let results: Vec<EmbeddedDocument> = ids
            .iter()
            .filter_map(|id| store.get(id).cloned())
            .collect();
        Ok(results)
    }

    async fn count(&self, filter: Option<Filter>) -> Result<usize> {
        let store = self.documents.read().unwrap();
        let count = match filter {
            Some(f) if !f.is_empty() => store.values().filter(|doc| f.matches(&doc.metadata)).count(),
            _ => store.len(),
        };
        Ok(count)
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let start = Instant::now();
        let count = self.document_count();
        let latency = start.elapsed().as_millis() as u64;

        Ok(HealthStatus::healthy("in_memory", latency).with_document_count(count))
    }

    fn backend_name(&self) -> &'static str {
        "in_memory"
    }

    fn dimensions(&self) -> Option<usize> {
        self.dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_documents() -> Vec<EmbeddedDocument> {
        vec![
            EmbeddedDocument::new("doc1", vec![1.0, 0.0, 0.0])
                .with_skill_name("kubernetes")
                .with_tool_name("get_pods")
                .with_tags(vec!["k8s".to_string()]),
            EmbeddedDocument::new("doc2", vec![0.9, 0.1, 0.0])
                .with_skill_name("kubernetes")
                .with_tool_name("create_deployment")
                .with_tags(vec!["k8s".to_string()]),
            EmbeddedDocument::new("doc3", vec![0.0, 1.0, 0.0])
                .with_skill_name("aws")
                .with_tool_name("list_buckets")
                .with_tags(vec!["cloud".to_string()]),
            EmbeddedDocument::new("doc4", vec![0.0, 0.0, 1.0])
                .with_skill_name("git")
                .with_tool_name("commit")
                .with_tags(vec!["vcs".to_string()]),
        ]
    }

    #[tokio::test]
    async fn test_upsert_and_count() {
        let store = InMemoryVectorStore::new();
        let docs = create_test_documents();

        let stats = store.upsert(docs).await.unwrap();
        assert_eq!(stats.inserted, 4);
        assert_eq!(stats.updated, 0);
        assert_eq!(stats.total, 4);

        let count = store.count(None).await.unwrap();
        assert_eq!(count, 4);
    }

    #[tokio::test]
    async fn test_upsert_update() {
        let store = InMemoryVectorStore::new();

        // Initial insert
        let docs = vec![EmbeddedDocument::new("doc1", vec![1.0, 0.0, 0.0])];
        let stats = store.upsert(docs).await.unwrap();
        assert_eq!(stats.inserted, 1);
        assert_eq!(stats.updated, 0);

        // Update same document
        let docs = vec![EmbeddedDocument::new("doc1", vec![0.0, 1.0, 0.0])];
        let stats = store.upsert(docs).await.unwrap();
        assert_eq!(stats.inserted, 0);
        assert_eq!(stats.updated, 1);

        // Verify count unchanged
        let count = store.count(None).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_search_basic() {
        let store = InMemoryVectorStore::new();
        store.upsert(create_test_documents()).await.unwrap();

        // Search for vector similar to doc1
        let results = store
            .search(vec![1.0, 0.0, 0.0], None, 2)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "doc1"); // Exact match should be first
        assert!((results[0].score - 1.0).abs() < 1e-5); // Perfect score
        assert_eq!(results[1].id, "doc2"); // Second most similar
    }

    #[tokio::test]
    async fn test_search_with_filter() {
        let store = InMemoryVectorStore::new();
        store.upsert(create_test_documents()).await.unwrap();

        // Search only kubernetes skills
        let filter = Filter::new().skill("kubernetes");
        let results = store
            .search(vec![0.5, 0.5, 0.0], Some(filter), 10)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        for result in results {
            assert_eq!(result.metadata.skill_name, Some("kubernetes".to_string()));
        }
    }

    #[tokio::test]
    async fn test_search_with_tag_filter() {
        let store = InMemoryVectorStore::new();
        store.upsert(create_test_documents()).await.unwrap();

        // Search only k8s tagged documents
        let filter = Filter::new().tags(vec!["k8s".to_string()]);
        let results = store
            .search(vec![0.5, 0.5, 0.0], Some(filter), 10)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        for result in results {
            assert!(result.metadata.tags.contains(&"k8s".to_string()));
        }
    }

    #[tokio::test]
    async fn test_search_with_min_score() {
        let store = InMemoryVectorStore::new();
        store.upsert(create_test_documents()).await.unwrap();

        // Search with very high min_score (only exact match passes)
        let filter = Filter::new().min_score(0.9999);
        let results = store
            .search(vec![1.0, 0.0, 0.0], Some(filter), 10)
            .await
            .unwrap();

        // Only exact match should pass
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc1");

        // Search with moderate min_score
        let filter = Filter::new().min_score(0.8);
        let results = store
            .search(vec![1.0, 0.0, 0.0], Some(filter), 10)
            .await
            .unwrap();

        // doc1 (1.0) and doc2 (0.9949...) should pass
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_delete() {
        let store = InMemoryVectorStore::new();
        store.upsert(create_test_documents()).await.unwrap();

        let stats = store
            .delete(vec!["doc1".to_string(), "doc2".to_string(), "nonexistent".to_string()])
            .await
            .unwrap();

        assert_eq!(stats.deleted, 2);
        assert_eq!(stats.not_found, 1);

        let count = store.count(None).await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_get() {
        let store = InMemoryVectorStore::new();
        store.upsert(create_test_documents()).await.unwrap();

        let docs = store
            .get(vec!["doc1".to_string(), "doc3".to_string(), "nonexistent".to_string()])
            .await
            .unwrap();

        assert_eq!(docs.len(), 2);
        assert!(docs.iter().any(|d| d.id == "doc1"));
        assert!(docs.iter().any(|d| d.id == "doc3"));
    }

    #[tokio::test]
    async fn test_count_with_filter() {
        let store = InMemoryVectorStore::new();
        store.upsert(create_test_documents()).await.unwrap();

        let filter = Filter::new().skill("kubernetes");
        let count = store.count(Some(filter)).await.unwrap();
        assert_eq!(count, 2);

        let filter = Filter::new().skill("git");
        let count = store.count(Some(filter)).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_health_check() {
        let store = InMemoryVectorStore::new();
        store.upsert(create_test_documents()).await.unwrap();

        let status = store.health_check().await.unwrap();
        assert!(status.healthy);
        assert_eq!(status.backend, "in_memory");
        assert_eq!(status.document_count, Some(4));
    }

    #[tokio::test]
    async fn test_dimension_validation() {
        let store = InMemoryVectorStore::with_dimensions(3);

        // Valid dimensions
        let docs = vec![EmbeddedDocument::new("doc1", vec![1.0, 0.0, 0.0])];
        assert!(store.upsert(docs).await.is_ok());

        // Invalid dimensions
        let docs = vec![EmbeddedDocument::new("doc2", vec![1.0, 0.0])];
        assert!(store.upsert(docs).await.is_err());
    }

    #[tokio::test]
    async fn test_euclidean_metric() {
        let store = InMemoryVectorStore::with_metric(DistanceMetric::Euclidean);
        store.upsert(create_test_documents()).await.unwrap();

        let results = store
            .search(vec![1.0, 0.0, 0.0], None, 2)
            .await
            .unwrap();

        // doc1 should still be first (closest)
        assert_eq!(results[0].id, "doc1");
    }

    #[tokio::test]
    async fn test_clear() {
        let store = InMemoryVectorStore::new();
        store.upsert(create_test_documents()).await.unwrap();

        assert_eq!(store.count(None).await.unwrap(), 4);

        store.clear();

        assert_eq!(store.count(None).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_backend_name() {
        let store = InMemoryVectorStore::new();
        assert_eq!(store.backend_name(), "in_memory");
    }
}
