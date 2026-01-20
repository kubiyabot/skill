//! File-based vector store with persistence to local disk
//!
//! Stores vectors in bincode format at ~/.skill-engine/vectors/store.bin
//! Provides atomic writes and automatic persistence.
//!
//! # Features
//!
//! - **Persistent storage**: Vectors survive server restarts
//! - **Atomic writes**: Uses temp file + rename for safe persistence
//! - **Lazy loading**: Loads from disk on first access
//! - **Auto-save**: Persists after each modification
//! - **Simple and fast**: Binary serialization with bincode
//!
//! # Performance
//!
//! - Write latency: ~5-20ms for 1000 documents
//! - Search: O(n) linear scan (acceptable for <10k documents)
//! - File size: ~4-8 bytes per dimension per document
//!
//! # Example
//!
//! ```ignore
//! use skill_runtime::vector_store::{FileVectorStore, FileConfig};
//!
//! let config = FileConfig::default(); // Uses ~/.skill-engine/vectors/store.bin
//! let store = FileVectorStore::new(config)?;
//!
//! // Data persists to disk automatically
//! store.upsert(documents).await?;
//! ```

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::Instant;

use super::{
    cosine_similarity, euclidean_distance, DeleteStats, DistanceMetric, EmbeddedDocument, Filter,
    HealthStatus, SearchResult, UpsertStats, VectorStore,
};

/// Metadata about the vector store file
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoreMetadata {
    /// Format version for forward compatibility
    version: u32,
    /// When the store was first created
    created_at: DateTime<Utc>,
    /// Last modification time
    updated_at: DateTime<Utc>,
    /// Number of documents currently stored
    document_count: usize,
    /// Embedding dimensions (if known)
    dimensions: Option<usize>,
}

/// Serializable container for the vector store
#[derive(Serialize, Deserialize)]
struct FileStoreData {
    /// Store metadata
    metadata: StoreMetadata,
    /// Documents indexed by ID
    documents: HashMap<String, EmbeddedDocument>,
    /// Distance metric for similarity calculation
    distance_metric: DistanceMetric,
}

/// File-based vector store with automatic persistence
///
/// This implementation provides persistent vector storage using local files.
/// Data is serialized with bincode for performance and compactness.
pub struct FileVectorStore {
    /// The store data (protected by RwLock for thread safety)
    data: RwLock<FileStoreData>,
    /// Path to the storage file
    file_path: PathBuf,
}

impl FileVectorStore {
    /// Create new file-based vector store
    ///
    /// If the file exists, loads data from disk. Otherwise, creates a new empty store.
    /// The parent directory will be created if it doesn't exist.
    pub fn new(config: FileConfig) -> Result<Self> {
        let file_path = config.storage_path();

        // Create directory if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Load existing data or create new
        let data = if file_path.exists() {
            tracing::info!("Loading vector store from {}", file_path.display());
            Self::load_from_disk(&file_path)?
        } else {
            tracing::info!("Creating new vector store at {}", file_path.display());
            FileStoreData {
                metadata: StoreMetadata {
                    version: 1,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    document_count: 0,
                    dimensions: None,
                },
                documents: HashMap::new(),
                distance_metric: config.distance_metric,
            }
        };

        Ok(Self {
            data: RwLock::new(data),
            file_path,
        })
    }

    /// Load store data from disk
    fn load_from_disk(path: &Path) -> Result<FileStoreData> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open vector store file: {}", path.display()))?;
        let reader = BufReader::new(file);
        let data: FileStoreData = bincode::deserialize_from(reader)
            .context("Failed to deserialize vector store")?;

        tracing::info!(
            "Loaded {} documents from vector store (version {})",
            data.documents.len(),
            data.metadata.version
        );

        Ok(data)
    }

    /// Save store data to disk atomically
    ///
    /// Writes to a temporary file first, then renames it to the target path.
    /// This ensures the store is never left in a corrupted state.
    fn save_to_disk(&self) -> Result<()> {
        let data = self.data.read().unwrap();

        // Write to temp file first (atomic operation)
        let temp_path = self.file_path.with_extension("tmp");
        let file = File::create(&temp_path)
            .with_context(|| format!("Failed to create temp file: {}", temp_path.display()))?;
        let writer = BufWriter::new(file);

        bincode::serialize_into(writer, &*data).context("Failed to serialize vector store")?;

        // Rename temp file to actual file (atomic on Unix)
        fs::rename(&temp_path, &self.file_path).with_context(|| {
            format!(
                "Failed to rename {} to {}",
                temp_path.display(),
                self.file_path.display()
            )
        })?;

        tracing::debug!(
            "Persisted vector store with {} documents to {}",
            data.documents.len(),
            self.file_path.display()
        );

        Ok(())
    }

    /// Auto-persist after modification
    ///
    /// Updates metadata and saves to disk.
    fn persist(&self) -> Result<()> {
        // Update metadata
        {
            let mut data = self.data.write().unwrap();
            data.metadata.updated_at = Utc::now();
            data.metadata.document_count = data.documents.len();
            // Update dimensions if they changed
            if let Some(first_doc) = data.documents.values().next() {
                let dims = first_doc.embedding.len();
                if data.metadata.dimensions != Some(dims) {
                    data.metadata.dimensions = Some(dims);
                }
            }
        }

        self.save_to_disk()
    }

    /// Calculate similarity score between two embeddings
    fn calculate_score(&self, embedding_a: &[f32], embedding_b: &[f32]) -> f32 {
        let data = self.data.read().unwrap();
        match data.distance_metric {
            DistanceMetric::Cosine => {
                // Convert cosine similarity to score (0-1 range, higher is better)
                let similarity = cosine_similarity(embedding_a, embedding_b);
                (similarity + 1.0) / 2.0 // Map [-1, 1] to [0, 1]
            }
            DistanceMetric::Euclidean => {
                // Convert euclidean distance to score (higher is better)
                let distance = euclidean_distance(embedding_a, embedding_b);
                1.0 / (1.0 + distance) // Closer distances = higher scores
            }
            DistanceMetric::DotProduct => {
                // Dot product (assumes normalized vectors)
                embedding_a
                    .iter()
                    .zip(embedding_b.iter())
                    .map(|(a, b)| a * b)
                    .sum::<f32>()
                    .clamp(0.0, 1.0) // Clamp to 0-1 for score
            }
        }
    }
}

#[async_trait]
impl VectorStore for FileVectorStore {
    async fn upsert(&self, documents: Vec<EmbeddedDocument>) -> Result<UpsertStats> {
        let start = Instant::now();
        let mut inserted = 0;
        let mut updated = 0;

        {
            let mut data = self.data.write().unwrap();

            // Set dimensions from first document if not set
            if data.metadata.dimensions.is_none() && !documents.is_empty() {
                data.metadata.dimensions = Some(documents[0].embedding.len());
            }

            for doc in documents {
                // Validate dimensions match
                if let Some(expected_dims) = data.metadata.dimensions {
                    if doc.embedding.len() != expected_dims {
                        anyhow::bail!(
                            "Document {} has {} dimensions, expected {}",
                            doc.id,
                            doc.embedding.len(),
                            expected_dims
                        );
                    }
                }

                // Track insert vs update
                if data.documents.contains_key(&doc.id) {
                    updated += 1;
                } else {
                    inserted += 1;
                }

                data.documents.insert(doc.id.clone(), doc);
            }
        }

        // Persist to disk
        self.persist()?;

        let duration_ms = start.elapsed().as_millis() as u64;

        tracing::debug!(
            "Upserted {} documents ({} inserted, {} updated) in {}ms",
            inserted + updated,
            inserted,
            updated,
            duration_ms
        );

        Ok(UpsertStats::new(inserted, updated, duration_ms))
    }

    async fn search(
        &self,
        query_embedding: Vec<f32>,
        filter: Option<Filter>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        let data = self.data.read().unwrap();

        // Calculate scores for all documents
        let mut scored_results: Vec<(String, f32, &EmbeddedDocument)> = data
            .documents
            .iter()
            .filter_map(|(id, doc)| {
                // Apply metadata filter if provided
                if let Some(ref f) = filter {
                    if !f.matches(&doc.metadata) {
                        return None;
                    }
                }

                // Calculate similarity score
                let score = self.calculate_score(&query_embedding, &doc.embedding);

                // Apply minimum score filter if provided
                if let Some(ref f) = filter {
                    if let Some(min_score) = f.min_score {
                        if score < min_score {
                            return None;
                        }
                    }
                }

                Some((id.clone(), score, doc))
            })
            .collect();

        // Sort by score (descending - higher scores first)
        scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top_k and convert to SearchResult
        let results: Vec<SearchResult> = scored_results
            .into_iter()
            .take(top_k)
            .map(|(id, score, doc)| SearchResult {
                id,
                score,
                metadata: doc.metadata.clone(),
                content: doc.content.clone(),
                embedding: None, // Don't include embedding in results for efficiency
            })
            .collect();

        tracing::debug!(
            "Search completed: {} results out of {} documents",
            results.len(),
            data.documents.len()
        );

        Ok(results)
    }

    async fn delete(&self, ids: Vec<String>) -> Result<DeleteStats> {
        let start = Instant::now();
        let mut deleted = 0;
        let mut not_found = 0;

        {
            let mut data = self.data.write().unwrap();
            for id in &ids {
                if data.documents.remove(id).is_some() {
                    deleted += 1;
                } else {
                    not_found += 1;
                }
            }
        }

        // Persist to disk
        if deleted > 0 {
            self.persist()?;
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        tracing::debug!(
            "Deleted {} documents ({} not found) in {}ms",
            deleted,
            not_found,
            duration_ms
        );

        Ok(DeleteStats::new(deleted, not_found, duration_ms))
    }

    async fn get(&self, ids: Vec<String>) -> Result<Vec<EmbeddedDocument>> {
        let data = self.data.read().unwrap();
        let docs: Vec<EmbeddedDocument> = ids
            .iter()
            .filter_map(|id| data.documents.get(id).cloned())
            .collect();

        Ok(docs)
    }

    async fn count(&self, filter: Option<Filter>) -> Result<usize> {
        let data = self.data.read().unwrap();

        if let Some(f) = filter {
            // Count with filter
            let count = data
                .documents
                .values()
                .filter(|doc| f.matches(&doc.metadata))
                .count();
            Ok(count)
        } else {
            // Total count
            Ok(data.documents.len())
        }
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let start = Instant::now();

        // Check if we can read the data
        let count = {
            let data = self.data.read().unwrap();
            data.documents.len()
        };

        // Check if file exists and is readable
        let file_exists = self.file_path.exists();
        let latency_ms = start.elapsed().as_millis() as u64;

        if file_exists {
            Ok(HealthStatus::healthy("file", latency_ms).with_document_count(count))
        } else {
            Ok(HealthStatus::unhealthy(
                "file",
                format!("Store file not found: {}", self.file_path.display()),
                latency_ms,
            ))
        }
    }

    fn backend_name(&self) -> &'static str {
        "file"
    }

    fn dimensions(&self) -> Option<usize> {
        let data = self.data.read().unwrap();
        data.metadata.dimensions
    }
}

/// Configuration for file-based vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    /// Custom storage directory (if None, uses default ~/.skill-engine/vectors/store.bin)
    pub storage_dir: Option<PathBuf>,
    /// Distance metric for similarity calculation
    pub distance_metric: DistanceMetric,
}

impl FileConfig {
    /// Get the storage path, defaulting to ~/.skill-engine/vectors/store.bin
    pub fn storage_path(&self) -> PathBuf {
        self.storage_dir.clone().unwrap_or_else(|| {
            let home = dirs::home_dir().expect("Could not determine home directory");
            home.join(".skill-engine/vectors/store.bin")
        })
    }

    /// Create config with custom storage path
    pub fn with_storage_path(mut self, path: PathBuf) -> Self {
        self.storage_dir = Some(path);
        self
    }

    /// Create config with custom distance metric
    pub fn with_distance_metric(mut self, metric: DistanceMetric) -> Self {
        self.distance_metric = metric;
        self
    }
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            storage_dir: None,
            distance_metric: DistanceMetric::Cosine,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_file_vector_store_persistence() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().join("test_store.bin");

        let config = FileConfig::default().with_storage_path(storage_path.clone());

        // Create store and add documents
        let store = FileVectorStore::new(config.clone()).unwrap();

        let docs = vec![
            EmbeddedDocument::new("doc1", vec![0.1, 0.2, 0.3])
                .with_skill_name("test")
                .with_content("Test document 1"),
            EmbeddedDocument::new("doc2", vec![0.4, 0.5, 0.6])
                .with_skill_name("test")
                .with_content("Test document 2"),
        ];

        store.upsert(docs).await.unwrap();

        // Verify count
        assert_eq!(store.count(None).await.unwrap(), 2);

        // Drop store (simulating server restart)
        drop(store);

        // Create new store - should load persisted data
        let store2 = FileVectorStore::new(config).unwrap();
        assert_eq!(store2.count(None).await.unwrap(), 2);

        // Verify documents are intact
        let loaded_docs = store2.get(vec!["doc1".to_string(), "doc2".to_string()]).await.unwrap();
        assert_eq!(loaded_docs.len(), 2);
        assert_eq!(loaded_docs[0].id, "doc1");
        assert_eq!(loaded_docs[0].embedding, vec![0.1, 0.2, 0.3]);
    }

    #[tokio::test]
    async fn test_file_vector_store_search() {
        let temp_dir = tempdir().unwrap();
        let config = FileConfig::default().with_storage_path(temp_dir.path().join("search_test.bin"));

        let store = FileVectorStore::new(config).unwrap();

        let docs = vec![
            EmbeddedDocument::new("doc1", vec![1.0, 0.0, 0.0])
                .with_skill_name("skill1")
                .with_content("Document 1"),
            EmbeddedDocument::new("doc2", vec![0.0, 1.0, 0.0])
                .with_skill_name("skill2")
                .with_content("Document 2"),
            EmbeddedDocument::new("doc3", vec![0.9, 0.1, 0.0])
                .with_skill_name("skill1")
                .with_content("Document 3"),
        ];

        store.upsert(docs).await.unwrap();

        // Search for vectors similar to [1, 0, 0]
        let results = store
            .search(vec![1.0, 0.0, 0.0], None, 2)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "doc1"); // Exact match should be first
        assert_eq!(results[1].id, "doc3"); // Similar vector should be second
        assert!(results[0].score > results[1].score);
    }

    #[tokio::test]
    async fn test_file_vector_store_filter() {
        let temp_dir = tempdir().unwrap();
        let config = FileConfig::default().with_storage_path(temp_dir.path().join("filter_test.bin"));

        let store = FileVectorStore::new(config).unwrap();

        let docs = vec![
            EmbeddedDocument::new("doc1", vec![1.0, 0.0])
                .with_skill_name("skill1")
                .with_content("Document 1"),
            EmbeddedDocument::new("doc2", vec![0.9, 0.1])
                .with_skill_name("skill2")
                .with_content("Document 2"),
            EmbeddedDocument::new("doc3", vec![0.8, 0.2])
                .with_skill_name("skill1")
                .with_content("Document 3"),
        ];

        store.upsert(docs).await.unwrap();

        // Search with filter
        let filter = Filter::new().skill("skill1");
        let results = store
            .search(vec![1.0, 0.0], Some(filter), 10)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.metadata.skill_name.as_deref() == Some("skill1")));
    }

    #[tokio::test]
    async fn test_file_vector_store_delete() {
        let temp_dir = tempdir().unwrap();
        let config = FileConfig::default().with_storage_path(temp_dir.path().join("delete_test.bin"));

        let store = FileVectorStore::new(config).unwrap();

        let docs = vec![
            EmbeddedDocument::new("doc1", vec![1.0, 0.0]),
            EmbeddedDocument::new("doc2", vec![0.0, 1.0]),
        ];

        store.upsert(docs).await.unwrap();
        assert_eq!(store.count(None).await.unwrap(), 2);

        // Delete one document
        let stats = store.delete(vec!["doc1".to_string()]).await.unwrap();
        assert_eq!(stats.deleted, 1);
        assert_eq!(stats.not_found, 0);
        assert_eq!(store.count(None).await.unwrap(), 1);

        // Try to delete non-existent document
        let stats = store.delete(vec!["doc3".to_string()]).await.unwrap();
        assert_eq!(stats.deleted, 0);
        assert_eq!(stats.not_found, 1);
    }

    #[tokio::test]
    async fn test_file_vector_store_health_check() {
        let temp_dir = tempdir().unwrap();
        let config = FileConfig::default().with_storage_path(temp_dir.path().join("health_test.bin"));

        let store = FileVectorStore::new(config).unwrap();

        // Add some documents
        store
            .upsert(vec![EmbeddedDocument::new("doc1", vec![1.0, 0.0])])
            .await
            .unwrap();

        let health = store.health_check().await.unwrap();
        assert!(health.healthy);
        assert_eq!(health.backend, "file");
        assert_eq!(health.document_count, Some(1));
    }
}
