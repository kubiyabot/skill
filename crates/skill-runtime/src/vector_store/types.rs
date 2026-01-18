//! Types for vector store operations
//!
//! This module defines the core data structures used across all vector store
//! backends, including documents, filters, search results, and statistics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A document with its embedding vector and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedDocument {
    /// Unique identifier for this document
    pub id: String,

    /// The embedding vector
    pub embedding: Vec<f32>,

    /// Arbitrary metadata for filtering and display
    pub metadata: DocumentMetadata,

    /// Original text content (optional, for debugging/display)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

impl EmbeddedDocument {
    /// Create a new document with just ID and embedding
    pub fn new(id: impl Into<String>, embedding: Vec<f32>) -> Self {
        Self {
            id: id.into(),
            embedding,
            metadata: DocumentMetadata::default(),
            content: None,
        }
    }

    /// Create a document with metadata
    pub fn with_metadata(
        id: impl Into<String>,
        embedding: Vec<f32>,
        metadata: DocumentMetadata,
    ) -> Self {
        Self {
            id: id.into(),
            embedding,
            metadata,
            content: None,
        }
    }

    /// Add original content
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Builder pattern for metadata fields
    pub fn with_skill_name(mut self, skill_name: impl Into<String>) -> Self {
        self.metadata.skill_name = Some(skill_name.into());
        self
    }

    /// Set the instance name metadata field
    pub fn with_instance_name(mut self, instance_name: impl Into<String>) -> Self {
        self.metadata.instance_name = Some(instance_name.into());
        self
    }

    /// Set the tool name metadata field
    pub fn with_tool_name(mut self, tool_name: impl Into<String>) -> Self {
        self.metadata.tool_name = Some(tool_name.into());
        self
    }

    /// Set the category metadata field
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.metadata.category = Some(category.into());
        self
    }

    /// Set the tags metadata field
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.metadata.tags = tags;
        self
    }

    /// Add a custom key-value pair to the metadata
    pub fn with_custom(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.custom.insert(key.into(), value.into());
        self
    }
}

/// Metadata associated with a document for filtering and display
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DocumentMetadata {
    /// Skill name (e.g., "kubernetes", "git")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,

    /// Instance name (e.g., "default", "production")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_name: Option<String>,

    /// Tool name (e.g., "get_pods", "create_deployment")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    /// Category (e.g., "infrastructure", "development")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Tags for additional classification
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Arbitrary key-value pairs for custom filtering
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, String>,
}

/// Filter for narrowing search results based on metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Filter {
    /// Filter by skill name (exact match)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,

    /// Filter by instance name (exact match)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_name: Option<String>,

    /// Filter by tool name (exact match)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    /// Filter by category (exact match)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Filter by tags (document must have ALL specified tags)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Filter by custom metadata (all must match)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, String>,

    /// Minimum similarity score (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_score: Option<f32>,
}

impl Filter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by skill name
    pub fn skill(mut self, skill_name: impl Into<String>) -> Self {
        self.skill_name = Some(skill_name.into());
        self
    }

    /// Filter by instance name
    pub fn instance(mut self, instance_name: impl Into<String>) -> Self {
        self.instance_name = Some(instance_name.into());
        self
    }

    /// Filter by tool name
    pub fn tool(mut self, tool_name: impl Into<String>) -> Self {
        self.tool_name = Some(tool_name.into());
        self
    }

    /// Filter by category
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Filter by tags (must have all)
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Filter by minimum score
    pub fn min_score(mut self, score: f32) -> Self {
        self.min_score = Some(score.clamp(0.0, 1.0));
        self
    }

    /// Add custom filter
    pub fn custom(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom.insert(key.into(), value.into());
        self
    }

    /// Check if this filter matches a document's metadata
    pub fn matches(&self, metadata: &DocumentMetadata) -> bool {
        // Check skill name
        if let Some(ref skill) = self.skill_name {
            if metadata.skill_name.as_ref() != Some(skill) {
                return false;
            }
        }

        // Check instance name
        if let Some(ref instance) = self.instance_name {
            if metadata.instance_name.as_ref() != Some(instance) {
                return false;
            }
        }

        // Check tool name
        if let Some(ref tool) = self.tool_name {
            if metadata.tool_name.as_ref() != Some(tool) {
                return false;
            }
        }

        // Check category
        if let Some(ref category) = self.category {
            if metadata.category.as_ref() != Some(category) {
                return false;
            }
        }

        // Check tags (all must be present)
        for tag in &self.tags {
            if !metadata.tags.contains(tag) {
                return false;
            }
        }

        // Check custom metadata (all must match)
        for (key, value) in &self.custom {
            if metadata.custom.get(key) != Some(value) {
                return false;
            }
        }

        true
    }

    /// Check if this filter is empty (matches everything)
    pub fn is_empty(&self) -> bool {
        self.skill_name.is_none()
            && self.instance_name.is_none()
            && self.tool_name.is_none()
            && self.category.is_none()
            && self.tags.is_empty()
            && self.custom.is_empty()
            && self.min_score.is_none()
    }
}

/// A search result with score and document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The document ID
    pub id: String,

    /// Similarity score (0.0 to 1.0, higher is more similar)
    pub score: f32,

    /// Document metadata
    pub metadata: DocumentMetadata,

    /// Original content (if stored)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// The embedding vector (optional, usually not returned for efficiency)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(id: impl Into<String>, score: f32, metadata: DocumentMetadata) -> Self {
        Self {
            id: id.into(),
            score,
            metadata,
            content: None,
            embedding: None,
        }
    }

    /// Create from an embedded document with score
    pub fn from_document(doc: &EmbeddedDocument, score: f32) -> Self {
        Self {
            id: doc.id.clone(),
            score,
            metadata: doc.metadata.clone(),
            content: doc.content.clone(),
            embedding: None, // Don't include embedding by default
        }
    }

    /// Include embedding in result
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
}

/// Statistics from an upsert operation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpsertStats {
    /// Number of documents inserted (new)
    pub inserted: usize,

    /// Number of documents updated (existing)
    pub updated: usize,

    /// Total documents processed
    pub total: usize,

    /// Time taken in milliseconds
    pub duration_ms: u64,
}

impl UpsertStats {
    /// Create new upsert statistics
    pub fn new(inserted: usize, updated: usize, duration_ms: u64) -> Self {
        Self {
            inserted,
            updated,
            total: inserted + updated,
            duration_ms,
        }
    }
}

/// Statistics from a delete operation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeleteStats {
    /// Number of documents deleted
    pub deleted: usize,

    /// Number of IDs not found
    pub not_found: usize,

    /// Total IDs requested
    pub total: usize,

    /// Time taken in milliseconds
    pub duration_ms: u64,
}

impl DeleteStats {
    /// Create new delete statistics
    pub fn new(deleted: usize, not_found: usize, duration_ms: u64) -> Self {
        Self {
            deleted,
            not_found,
            total: deleted + not_found,
            duration_ms,
        }
    }
}

/// Health status of a vector store backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Whether the backend is healthy
    pub healthy: bool,

    /// Backend name
    pub backend: String,

    /// Optional status message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Number of documents in the store
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_count: Option<usize>,

    /// Latency of health check in milliseconds
    pub latency_ms: u64,
}

impl HealthStatus {
    /// Create a healthy status
    pub fn healthy(backend: impl Into<String>, latency_ms: u64) -> Self {
        Self {
            healthy: true,
            backend: backend.into(),
            message: None,
            document_count: None,
            latency_ms,
        }
    }

    /// Create an unhealthy status with error message
    pub fn unhealthy(backend: impl Into<String>, message: impl Into<String>, latency_ms: u64) -> Self {
        Self {
            healthy: false,
            backend: backend.into(),
            message: Some(message.into()),
            document_count: None,
            latency_ms,
        }
    }

    /// Add document count to the health status
    pub fn with_document_count(mut self, count: usize) -> Self {
        self.document_count = Some(count);
        self
    }
}

/// Distance metric for similarity calculation
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum DistanceMetric {
    /// Cosine similarity (default, good for text embeddings)
    #[default]
    Cosine,

    /// Euclidean distance (L2)
    Euclidean,

    /// Dot product (for normalized vectors)
    DotProduct,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_document_builder() {
        let doc = EmbeddedDocument::new("doc1", vec![0.1, 0.2, 0.3])
            .with_skill_name("kubernetes")
            .with_instance_name("production")
            .with_tool_name("get_pods")
            .with_category("infrastructure")
            .with_tags(vec!["k8s".to_string(), "devops".to_string()])
            .with_content("Get pods from cluster");

        assert_eq!(doc.id, "doc1");
        assert_eq!(doc.metadata.skill_name, Some("kubernetes".to_string()));
        assert_eq!(doc.metadata.instance_name, Some("production".to_string()));
        assert_eq!(doc.metadata.tool_name, Some("get_pods".to_string()));
        assert_eq!(doc.metadata.category, Some("infrastructure".to_string()));
        assert_eq!(doc.metadata.tags, vec!["k8s", "devops"]);
        assert_eq!(doc.content, Some("Get pods from cluster".to_string()));
    }

    #[test]
    fn test_filter_matches() {
        let metadata = DocumentMetadata {
            skill_name: Some("kubernetes".to_string()),
            instance_name: Some("production".to_string()),
            tool_name: Some("get_pods".to_string()),
            category: Some("infrastructure".to_string()),
            tags: vec!["k8s".to_string(), "devops".to_string()],
            custom: HashMap::new(),
        };

        // Empty filter matches everything
        assert!(Filter::new().matches(&metadata));

        // Skill filter
        assert!(Filter::new().skill("kubernetes").matches(&metadata));
        assert!(!Filter::new().skill("aws").matches(&metadata));

        // Combined filter
        assert!(Filter::new()
            .skill("kubernetes")
            .instance("production")
            .matches(&metadata));

        // Tag filter
        assert!(Filter::new().tags(vec!["k8s".to_string()]).matches(&metadata));
        assert!(Filter::new()
            .tags(vec!["k8s".to_string(), "devops".to_string()])
            .matches(&metadata));
        assert!(!Filter::new()
            .tags(vec!["missing".to_string()])
            .matches(&metadata));
    }

    #[test]
    fn test_filter_is_empty() {
        assert!(Filter::new().is_empty());
        assert!(!Filter::new().skill("test").is_empty());
        assert!(!Filter::new().min_score(0.5).is_empty());
    }

    #[test]
    fn test_search_result_from_document() {
        let doc = EmbeddedDocument::new("doc1", vec![0.1, 0.2])
            .with_skill_name("test")
            .with_content("Test content");

        let result = SearchResult::from_document(&doc, 0.95);

        assert_eq!(result.id, "doc1");
        assert_eq!(result.score, 0.95);
        assert_eq!(result.metadata.skill_name, Some("test".to_string()));
        assert_eq!(result.content, Some("Test content".to_string()));
        assert!(result.embedding.is_none());
    }

    #[test]
    fn test_upsert_stats() {
        let stats = UpsertStats::new(5, 3, 100);
        assert_eq!(stats.inserted, 5);
        assert_eq!(stats.updated, 3);
        assert_eq!(stats.total, 8);
        assert_eq!(stats.duration_ms, 100);
    }

    #[test]
    fn test_health_status() {
        let healthy = HealthStatus::healthy("in_memory", 5).with_document_count(100);
        assert!(healthy.healthy);
        assert_eq!(healthy.document_count, Some(100));

        let unhealthy = HealthStatus::unhealthy("qdrant", "Connection refused", 1000);
        assert!(!unhealthy.healthy);
        assert_eq!(unhealthy.message, Some("Connection refused".to_string()));
    }
}
