//! Cross-encoder reranking for improved search precision
//!
//! Provides a reranking stage using cross-encoder models to improve the
//! precision of search results. Cross-encoders score query-document pairs
//! together, capturing deeper semantic relationships than bi-encoders.

use anyhow::{Context, Result};
use fastembed::{TextRerank, RerankInitOptions, RerankerModel as FastEmbedRerankerModel};
use std::sync::Arc;

/// A document to be reranked
#[derive(Debug, Clone)]
pub struct RerankDocument {
    /// Document ID
    pub id: String,
    /// Text content to score against the query
    pub text: String,
    /// Original score from initial retrieval (optional)
    pub original_score: Option<f32>,
}

impl RerankDocument {
    /// Create a new rerank document
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            original_score: None,
        }
    }

    /// Create with an original score
    pub fn with_score(id: impl Into<String>, text: impl Into<String>, score: f32) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            original_score: Some(score),
        }
    }
}

/// Result from reranking
#[derive(Debug, Clone)]
pub struct RerankResult {
    /// Document ID
    pub id: String,
    /// Relevance score from cross-encoder (higher = more relevant)
    pub relevance_score: f32,
    /// Original index in the input list
    pub original_index: usize,
    /// Original score from initial retrieval (if available)
    pub original_score: Option<f32>,
}

/// Supported reranker models
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RerankerModel {
    /// BAAI BGE Reranker Base - balanced quality/speed (default)
    #[default]
    BGERerankerBase,
    /// BAAI BGE Reranker v2 M3 - improved multilingual
    BGERerankerV2M3,
    /// Jina Reranker v1 Turbo English - fast English reranking
    JinaRerankerV1TurboEn,
    /// Jina Reranker v2 Base Multilingual - multilingual support
    JinaRerankerV2BaseMultilingual,
}

impl RerankerModel {
    /// Convert to fastembed model
    fn to_fastembed_model(&self) -> FastEmbedRerankerModel {
        match self {
            RerankerModel::BGERerankerBase => FastEmbedRerankerModel::BGERerankerBase,
            RerankerModel::BGERerankerV2M3 => FastEmbedRerankerModel::BGERerankerV2M3,
            RerankerModel::JinaRerankerV1TurboEn => FastEmbedRerankerModel::JINARerankerV1TurboEn,
            RerankerModel::JinaRerankerV2BaseMultilingual => FastEmbedRerankerModel::JINARerankerV2BaseMultiligual,
        }
    }

    /// Get model name for display
    pub fn name(&self) -> &'static str {
        match self {
            RerankerModel::BGERerankerBase => "BAAI/bge-reranker-base",
            RerankerModel::BGERerankerV2M3 => "BAAI/bge-reranker-v2-m3",
            RerankerModel::JinaRerankerV1TurboEn => "jinaai/jina-reranker-v1-turbo-en",
            RerankerModel::JinaRerankerV2BaseMultilingual => "jinaai/jina-reranker-v2-base-multilingual",
        }
    }
}

impl std::str::FromStr for RerankerModel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bge-base" | "bge-reranker-base" | "baai/bge-reranker-base" => Ok(Self::BGERerankerBase),
            "bge-v2-m3" | "bge-reranker-v2-m3" | "baai/bge-reranker-v2-m3" => Ok(Self::BGERerankerV2M3),
            "jina-turbo" | "jina-v1-turbo" | "jinaai/jina-reranker-v1-turbo-en" => Ok(Self::JinaRerankerV1TurboEn),
            "jina-base" | "jina-v2-base" | "jinaai/jina-reranker-v2-base-multilingual" => Ok(Self::JinaRerankerV2BaseMultilingual),
            _ => anyhow::bail!("Unknown reranker model: {}. Options: bge-base, bge-v2-m3, jina-turbo, jina-base", s),
        }
    }
}

/// Configuration for reranking
#[derive(Debug, Clone)]
pub struct RerankerConfig {
    /// Model to use
    pub model: RerankerModel,
    /// Maximum documents to rerank (for latency control)
    pub max_documents: usize,
    /// Minimum relevance score threshold (0.0-1.0)
    pub min_score_threshold: Option<f32>,
    /// Show download progress for model files
    pub show_download_progress: bool,
}

impl Default for RerankerConfig {
    fn default() -> Self {
        Self {
            model: RerankerModel::default(),
            max_documents: 50,
            min_score_threshold: None,
            show_download_progress: false,
        }
    }
}

impl RerankerConfig {
    /// Create config with a specific model
    pub fn with_model(model: RerankerModel) -> Self {
        Self {
            model,
            ..Default::default()
        }
    }

    /// Set max documents
    pub fn max_documents(mut self, n: usize) -> Self {
        self.max_documents = n;
        self
    }

    /// Set minimum score threshold
    pub fn min_score(mut self, threshold: f32) -> Self {
        self.min_score_threshold = Some(threshold);
        self
    }
}

/// Trait for reranking documents
pub trait Reranker: Send + Sync {
    /// Rerank documents for a query
    ///
    /// Takes a query and a list of documents, returning them sorted by relevance.
    fn rerank(&self, query: &str, documents: Vec<RerankDocument>, top_k: usize) -> Result<Vec<RerankResult>>;

    /// Get the model name
    fn model_name(&self) -> &str;
}

/// FastEmbed-based cross-encoder reranker
///
/// Uses ONNX models locally for reranking. Models are downloaded on first use.
pub struct FastEmbedReranker {
    model: Arc<TextRerank>,
    config: RerankerConfig,
}

impl FastEmbedReranker {
    /// Create a new reranker with default settings
    pub fn new() -> Result<Self> {
        Self::with_config(RerankerConfig::default())
    }

    /// Create with specific model
    pub fn with_model(model: RerankerModel) -> Result<Self> {
        Self::with_config(RerankerConfig::with_model(model))
    }

    /// Create with full config
    pub fn with_config(config: RerankerConfig) -> Result<Self> {
        let fastembed_model = config.model.to_fastembed_model();

        let options = RerankInitOptions::new(fastembed_model)
            .with_show_download_progress(config.show_download_progress);

        let model = TextRerank::try_new(options)
            .context("Failed to initialize reranker model")?;

        Ok(Self {
            model: Arc::new(model),
            config,
        })
    }

    /// Get the config
    pub fn config(&self) -> &RerankerConfig {
        &self.config
    }
}

impl Reranker for FastEmbedReranker {
    fn rerank(&self, query: &str, documents: Vec<RerankDocument>, top_k: usize) -> Result<Vec<RerankResult>> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }

        // Limit documents for latency control
        let docs_to_rerank: Vec<_> = documents
            .iter()
            .take(self.config.max_documents)
            .collect();

        // Extract texts for reranking
        let texts: Vec<&str> = docs_to_rerank.iter().map(|d| d.text.as_str()).collect();

        // Perform reranking
        let rerank_results = self.model
            .rerank(query, texts, false, None)
            .context("Reranking failed")?;

        // Convert to our result type
        let mut results: Vec<RerankResult> = rerank_results
            .into_iter()
            .map(|r| {
                let original_doc = &docs_to_rerank[r.index];
                RerankResult {
                    id: original_doc.id.clone(),
                    relevance_score: r.score,
                    original_index: r.index,
                    original_score: original_doc.original_score,
                }
            })
            .collect();

        // Sort by relevance score descending
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        // Apply score threshold if configured
        if let Some(threshold) = self.config.min_score_threshold {
            results.retain(|r| r.relevance_score >= threshold);
        }

        // Truncate to top_k
        results.truncate(top_k);

        Ok(results)
    }

    fn model_name(&self) -> &str {
        self.config.model.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_from_str() {
        assert_eq!("bge-base".parse::<RerankerModel>().unwrap(), RerankerModel::BGERerankerBase);
        assert_eq!("bge-v2-m3".parse::<RerankerModel>().unwrap(), RerankerModel::BGERerankerV2M3);
        assert_eq!("jina-base".parse::<RerankerModel>().unwrap(), RerankerModel::JinaRerankerV2BaseMultilingual);
        assert_eq!("jina-turbo".parse::<RerankerModel>().unwrap(), RerankerModel::JinaRerankerV1TurboEn);
        assert!("unknown".parse::<RerankerModel>().is_err());
    }

    #[test]
    fn test_config_default() {
        let config = RerankerConfig::default();
        assert_eq!(config.model, RerankerModel::BGERerankerBase);
        assert_eq!(config.max_documents, 50);
        assert!(config.min_score_threshold.is_none());
    }

    #[test]
    fn test_config_builder() {
        let config = RerankerConfig::with_model(RerankerModel::BGERerankerV2M3)
            .max_documents(100)
            .min_score(0.5);

        assert_eq!(config.model, RerankerModel::BGERerankerV2M3);
        assert_eq!(config.max_documents, 100);
        assert_eq!(config.min_score_threshold, Some(0.5));
    }

    #[test]
    fn test_rerank_document_creation() {
        let doc = RerankDocument::new("doc1", "Hello world");
        assert_eq!(doc.id, "doc1");
        assert_eq!(doc.text, "Hello world");
        assert!(doc.original_score.is_none());

        let doc_with_score = RerankDocument::with_score("doc2", "Test", 0.95);
        assert_eq!(doc_with_score.original_score, Some(0.95));
    }

    // Integration test - requires model download
    #[test]
    #[ignore = "requires model download (~500MB)"]
    fn test_reranker_creation() {
        let reranker = FastEmbedReranker::new().unwrap();
        assert_eq!(reranker.model_name(), "BAAI/bge-reranker-base");
    }

    #[test]
    #[ignore = "requires model download (~500MB)"]
    fn test_reranking() {
        let reranker = FastEmbedReranker::new().unwrap();

        let documents = vec![
            RerankDocument::new("doc1", "The capital of France is Paris"),
            RerankDocument::new("doc2", "Python is a programming language"),
            RerankDocument::new("doc3", "Paris has the Eiffel Tower"),
        ];

        let results = reranker.rerank("What is the capital of France?", documents, 3).unwrap();

        // doc1 should be most relevant
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "doc1");
    }

    #[test]
    #[ignore = "requires model download (~500MB)"]
    fn test_reranking_with_threshold() {
        let config = RerankerConfig::default().min_score(0.9);
        let reranker = FastEmbedReranker::with_config(config).unwrap();

        let documents = vec![
            RerankDocument::new("doc1", "Completely irrelevant text about cooking"),
            RerankDocument::new("doc2", "The capital of France is Paris"),
        ];

        let results = reranker.rerank("What is the capital of France?", documents, 5).unwrap();

        // Only doc2 should pass the threshold
        for result in &results {
            assert!(result.relevance_score >= 0.9);
        }
    }

    #[test]
    #[ignore = "requires model download (~500MB)"]
    fn test_empty_documents() {
        let reranker = FastEmbedReranker::new().unwrap();
        let results = reranker.rerank("test query", vec![], 5).unwrap();
        assert!(results.is_empty());
    }
}
