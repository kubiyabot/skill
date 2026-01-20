//! SearchPipeline orchestrator for end-to-end RAG search
//!
//! Provides a unified interface for semantic search that orchestrates:
//! - Embedding generation (FastEmbed, OpenAI, Ollama)
//! - Vector storage (InMemory, Qdrant)
//! - Hybrid retrieval (dense + BM25)
//! - Cross-encoder reranking
//! - Context compression
//! - Query understanding
//!
//! # Architecture
//!
//! The pipeline is designed to be **stateless per CLI invocation** but benefits from
//! filesystem caches for fast subsequent runs:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                          SearchPipeline                                  │
//! │                                                                          │
//! │  ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐    │
//! │  │   Embedding     │     │   Vector Store  │     │   BM25 Index    │    │
//! │  │   Provider      │     │   (InMemory/    │     │   (Tantivy)     │    │
//! │  │   (FastEmbed)   │     │    Qdrant)      │     │   [optional]    │    │
//! │  └────────┬────────┘     └────────┬────────┘     └────────┬────────┘    │
//! │           │                       │                       │             │
//! │           ▼                       ▼                       ▼             │
//! │  ┌─────────────────────────────────────────────────────────────────┐    │
//! │  │                     Filesystem Caches                            │    │
//! │  │  ~/.fastembed_cache/   ~/.skill-engine/index/   ~/.skill-engine/ │    │
//! │  │  (model weights)       (index metadata)         (search.toml)    │    │
//! │  └─────────────────────────────────────────────────────────────────┘    │
//! │                                                                          │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! **CLI Mode**: Pipeline is created per-invocation, indexes on-demand, benefits from caches.
//! **MCP Mode**: Pipeline is created once at server startup, kept in memory.
//!
//! # Example
//!
//! ```ignore
//! use skill_runtime::search::{SearchPipeline, SearchConfig};
//!
//! // Create pipeline from config
//! let config = SearchConfig::default();
//! let mut pipeline = SearchPipeline::from_config(config).await?;
//!
//! // Index skills
//! let skills = load_skills().await?;
//! pipeline.index_documents(skills).await?;
//!
//! // Search
//! let results = pipeline.search("deploy kubernetes pods", 5).await?;
//! ```

use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::embeddings::{EmbeddingProvider, EmbeddingProviderFactory};
use crate::search_config::{BackendType, SearchConfig};
use crate::vector_store::{
    EmbeddedDocument, DocumentMetadata, FileVectorStore, Filter, InMemoryVectorStore, VectorStore,
};

#[cfg(feature = "ai-ingestion")]
use crate::generation::{ExampleGenerator, GeneratorConfig, GenerationEvent, GeneratedExample, create_llm_provider};
#[cfg(feature = "ai-ingestion")]
use crate::skill_md::ToolDocumentation;
#[cfg(feature = "ai-ingestion")]
use futures_util::Stream;
#[cfg(feature = "ai-ingestion")]
use tokio_stream::StreamExt;

#[cfg(feature = "qdrant")]
use crate::vector_store::QdrantVectorStore;

#[cfg(feature = "hybrid-search")]
use super::{BM25Index, BM25Config};

#[cfg(feature = "hybrid-search")]
use tokio::sync::RwLock;

#[cfg(feature = "reranker")]
use super::{FastEmbedReranker, RerankerConfig as SearchRerankerConfig, Reranker, RerankDocument};

#[cfg(feature = "context-compression")]
use super::{ContextCompressor, CompressionConfig, CompressedToolContext};

use super::{QueryProcessor, ProcessedQuery};

/// Result from a search operation
#[derive(Debug, Clone)]
pub struct PipelineSearchResult {
    /// Document ID
    pub id: String,
    /// Original content
    pub content: String,
    /// Relevance score (0.0 - 1.0)
    pub score: f32,
    /// Document metadata
    pub metadata: DocumentMetadata,
    /// Reranker score (if reranking was performed)
    pub rerank_score: Option<f32>,
}

/// Statistics about indexed documents
#[derive(Debug, Clone, Default)]
pub struct PipelineIndexStats {
    /// Number of documents added
    pub documents_added: usize,
    /// Number of documents updated
    pub documents_updated: usize,
    /// Total documents in index
    pub total_documents: usize,
    /// Index size in bytes (approximate)
    pub index_size_bytes: Option<usize>,
}

/// Health status of the pipeline
#[derive(Debug, Clone)]
pub struct PipelineHealth {
    /// Overall health status
    pub healthy: bool,
    /// Embedding provider status
    pub embedding_provider: ProviderStatus,
    /// Vector store status
    pub vector_store: ProviderStatus,
    /// BM25 index status (if enabled)
    pub bm25_index: Option<ProviderStatus>,
    /// Reranker status (if enabled)
    pub reranker: Option<ProviderStatus>,
    /// Example generator status (if AI ingestion enabled)
    pub example_generator: Option<ProviderStatus>,
    /// Number of indexed documents
    pub indexed_documents: usize,
}

/// Status of an individual component
#[derive(Debug, Clone)]
pub struct ProviderStatus {
    /// Component name
    pub name: String,
    /// Whether the component is healthy
    pub healthy: bool,
    /// Optional error message if unhealthy
    pub error: Option<String>,
}

/// Document to be indexed
#[derive(Debug, Clone)]
pub struct IndexDocument {
    /// Unique document ID
    pub id: String,
    /// Text content to embed and index
    pub content: String,
    /// Optional metadata
    pub metadata: DocumentMetadata,
}

/// Unified search pipeline that orchestrates all RAG components
pub struct SearchPipeline {
    /// Configuration
    config: SearchConfig,
    /// Embedding provider
    embedding_provider: Arc<dyn EmbeddingProvider>,
    /// Vector store
    vector_store: Arc<dyn VectorStore>,
    /// BM25 index for hybrid search
    #[cfg(feature = "hybrid-search")]
    bm25_index: Option<Arc<RwLock<BM25Index>>>,
    /// Cross-encoder reranker
    #[cfg(feature = "reranker")]
    reranker: Option<Arc<dyn Reranker>>,
    /// Context compressor
    #[cfg(feature = "context-compression")]
    compressor: Option<ContextCompressor>,
    /// AI example generator
    #[cfg(feature = "ai-ingestion")]
    example_generator: Option<Arc<ExampleGenerator>>,
    /// Query processor
    query_processor: QueryProcessor,
    /// Known skills for query processing
    known_skills: Vec<String>,
    /// Known tools for query processing
    known_tools: Vec<String>,
}

impl SearchPipeline {
    /// Create a new search pipeline from configuration
    pub async fn from_config(config: SearchConfig) -> Result<Self> {
        info!("Initializing SearchPipeline with config");

        // Validate configuration
        config.validate().context("Invalid search configuration")?;

        // Create embedding provider
        let embedding_config = crate::embeddings::EmbeddingConfig {
            provider: config.embedding.provider.parse()
                .unwrap_or(crate::embeddings::EmbeddingProviderType::FastEmbed),
            model: Some(config.embedding.model.clone()),
            api_key: config.embedding.openai_api_key.clone(),
            base_url: config.embedding.ollama_host.clone(),
            batch_size: 100,
        };

        let embedding_provider = EmbeddingProviderFactory::create(&embedding_config)
            .context("Failed to create embedding provider")?;

        debug!(
            "Created embedding provider: {} ({})",
            embedding_provider.provider_name(),
            embedding_provider.model_name()
        );

        // Create vector store
        let vector_store: Arc<dyn VectorStore> = match config.backend.backend_type {
            BackendType::File => {
                let file_config_from_search = config.file.as_ref();

                let file_config = crate::vector_store::FileConfig {
                    storage_dir: file_config_from_search.and_then(|c| c.storage_path.clone()),
                    distance_metric: file_config_from_search
                        .map(|c| c.distance_metric)
                        .unwrap_or(crate::vector_store::DistanceMetric::Cosine),
                };

                Arc::new(
                    FileVectorStore::new(file_config)
                        .context("Failed to create File vector store")?
                )
            }
            BackendType::InMemory => {
                Arc::new(InMemoryVectorStore::with_dimensions(config.embedding.dimensions))
            }
            #[cfg(feature = "qdrant")]
            BackendType::Qdrant => {
                let qdrant_config = config.qdrant.as_ref()
                    .context("Qdrant config required for qdrant backend")?;

                let qdrant_store = QdrantVectorStore::new(crate::vector_store::QdrantConfig {
                    url: qdrant_config.url.clone(),
                    api_key: qdrant_config.api_key.clone(),
                    collection_name: qdrant_config.collection.clone(),
                    vector_size: config.embedding.dimensions,
                    ..Default::default()
                }).await.context("Failed to create Qdrant store")?;

                Arc::new(qdrant_store)
            }
            #[cfg(not(feature = "qdrant"))]
            BackendType::Qdrant => {
                anyhow::bail!("Qdrant backend requires 'qdrant' feature to be enabled");
            }
        };

        debug!("Created vector store: {}", vector_store.backend_name());

        // Create BM25 index if hybrid search is enabled
        #[cfg(feature = "hybrid-search")]
        let bm25_index = if config.retrieval.enable_hybrid {
            let bm25_config = BM25Config::default();
            let index = BM25Index::new(bm25_config)?;
            Some(Arc::new(RwLock::new(index)))
        } else {
            None
        };

        // Create reranker if enabled
        #[cfg(feature = "reranker")]
        let reranker: Option<Arc<dyn Reranker>> = if config.reranker.enabled {
            let reranker_config = SearchRerankerConfig {
                model: config.reranker.model.parse().unwrap_or_default(),
                top_k: config.retrieval.final_k,
                ..Default::default()
            };
            let fastembed_reranker = FastEmbedReranker::new(reranker_config)
                .context("Failed to create reranker")?;
            Some(Arc::new(fastembed_reranker))
        } else {
            None
        };

        // Create context compressor if enabled
        #[cfg(feature = "context-compression")]
        let compressor = {
            let compression_config = CompressionConfig {
                max_tokens_per_result: config.context.max_tokens_per_result,
                max_total_tokens: config.context.max_total_tokens,
                strategy: match config.context.compression {
                    crate::search_config::CompressionStrategy::Extractive => {
                        super::CompressionStrategy::Extractive
                    }
                    crate::search_config::CompressionStrategy::Template => {
                        super::CompressionStrategy::TemplateBased
                    }
                    crate::search_config::CompressionStrategy::Progressive => {
                        super::CompressionStrategy::Progressive
                    }
                    crate::search_config::CompressionStrategy::None => {
                        super::CompressionStrategy::None
                    }
                },
                ..Default::default()
            };
            Some(ContextCompressor::new(compression_config)?)
        };

        // Create query processor
        let query_processor = QueryProcessor::new();

        // Create example generator if AI ingestion is enabled
        #[cfg(feature = "ai-ingestion")]
        let example_generator = if config.ai_ingestion.enabled {
            match create_llm_provider(&config.ai_ingestion) {
                Ok(llm) => {
                    let gen_config = GeneratorConfig::from(&config.ai_ingestion);
                    info!(
                        "AI example generation enabled: {} / {}",
                        llm.name(),
                        llm.model()
                    );
                    Some(Arc::new(ExampleGenerator::new(llm, gen_config)))
                }
                Err(e) => {
                    warn!("Failed to create LLM provider for AI ingestion: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            config,
            embedding_provider,
            vector_store,
            #[cfg(feature = "hybrid-search")]
            bm25_index,
            #[cfg(feature = "reranker")]
            reranker,
            #[cfg(feature = "context-compression")]
            compressor,
            #[cfg(feature = "ai-ingestion")]
            example_generator,
            query_processor,
            known_skills: Vec::new(),
            known_tools: Vec::new(),
        })
    }

    /// Create a pipeline with default configuration (FastEmbed, InMemory)
    pub async fn default_pipeline() -> Result<Self> {
        Self::from_config(SearchConfig::default()).await
    }

    /// Index documents into the pipeline
    ///
    /// This embeds the documents and stores them in both the vector store
    /// and BM25 index (if hybrid search is enabled).
    pub async fn index_documents(&self, documents: Vec<IndexDocument>) -> Result<PipelineIndexStats> {
        if documents.is_empty() {
            return Ok(PipelineIndexStats::default());
        }

        info!("Indexing {} documents", documents.len());

        // Extract texts for embedding
        let texts: Vec<String> = documents.iter().map(|d| d.content.clone()).collect();

        // Generate embeddings
        let embeddings = self.embedding_provider
            .embed_documents_batched(texts)
            .await
            .context("Failed to generate embeddings")?;

        // Create embedded documents
        let embedded_docs: Vec<EmbeddedDocument> = documents
            .into_iter()
            .zip(embeddings)
            .map(|(doc, embedding)| EmbeddedDocument {
                id: doc.id,
                content: Some(doc.content),
                embedding,
                metadata: doc.metadata,
            })
            .collect();

        let _doc_count = embedded_docs.len();

        // Index in BM25 if enabled
        #[cfg(feature = "hybrid-search")]
        if let Some(ref bm25) = self.bm25_index {
            let mut bm25_guard = bm25.write().await;
            for doc in &embedded_docs {
                if let Some(ref content) = doc.content {
                    bm25_guard.add_document(&doc.id, content)?;
                }
            }
            debug!("Added {} documents to BM25 index", doc_count);
        }

        // Upsert to vector store
        let stats = self.vector_store.upsert(embedded_docs).await
            .context("Failed to upsert to vector store")?;

        let total = self.vector_store.count(None).await.unwrap_or(0);

        Ok(PipelineIndexStats {
            documents_added: stats.inserted,
            documents_updated: stats.updated,
            total_documents: total,
            index_size_bytes: None,
        })
    }

    /// Index documents with AI-generated examples
    ///
    /// When AI ingestion is enabled, this method generates synthetic examples
    /// for each tool and appends them to the document content before indexing.
    #[cfg(feature = "ai-ingestion")]
    pub async fn index_documents_with_generation(
        &self,
        documents: Vec<IndexDocument>,
        tools: Vec<ToolDocumentation>,
    ) -> Result<(PipelineIndexStats, Vec<GeneratedExample>)> {
        if let Some(ref generator) = self.example_generator {
            let enhanced = self.enhance_documents_with_examples(documents, &tools, generator).await?;
            let all_examples = enhanced.1;
            let stats = self.index_documents(enhanced.0).await?;
            Ok((stats, all_examples))
        } else {
            let stats = self.index_documents(documents).await?;
            Ok((stats, Vec::new()))
        }
    }

    /// Index documents with streaming generation events
    ///
    /// Returns a stream of generation events while indexing documents.
    /// Useful for progress feedback in CLI/MCP contexts.
    #[cfg(feature = "ai-ingestion")]
    pub fn index_documents_stream<'a>(
        &'a self,
        documents: Vec<IndexDocument>,
        tools: Vec<ToolDocumentation>,
    ) -> impl Stream<Item = GenerationEvent> + 'a {
        async_stream::stream! {
            if let Some(ref generator) = self.example_generator {
                let total_tools = tools.len();
                let mut all_examples = Vec::new();

                // Generate examples for each tool
                for (idx, tool) in tools.iter().enumerate() {
                    let mut stream = Box::pin(generator.generate_stream(tool, idx + 1, total_tools));

                    while let Some(event) = stream.next().await {
                        // Collect examples from events
                        if let GenerationEvent::Example { ref example } = event {
                            all_examples.push(example.clone());
                        }
                        yield event;
                    }
                }

                // Enhance documents with generated examples
                let enhanced_docs = self.enhance_documents_inline(&documents, &all_examples);

                // Index the enhanced documents
                match self.index_documents(enhanced_docs).await {
                    Ok(stats) => {
                        yield GenerationEvent::Completed {
                            total_examples: all_examples.len(),
                            total_valid: all_examples.iter().filter(|e| e.validated).count(),
                            total_tools,
                            duration_ms: 0, // Would need to track actual duration
                        };
                        info!(
                            "Indexed {} documents with {} generated examples",
                            stats.total_documents, all_examples.len()
                        );
                    }
                    Err(e) => {
                        yield GenerationEvent::Error {
                            message: format!("Failed to index documents: {}", e),
                            recoverable: false,
                            tool_name: None,
                        };
                    }
                }
            } else {
                // No generator, just index directly
                match self.index_documents(documents).await {
                    Ok(stats) => {
                        yield GenerationEvent::Completed {
                            total_examples: 0,
                            total_valid: 0,
                            total_tools: tools.len(),
                            duration_ms: 0,
                        };
                        info!("Indexed {} documents (no AI generation)", stats.total_documents);
                    }
                    Err(e) => {
                        yield GenerationEvent::Error {
                            message: format!("Failed to index documents: {}", e),
                            recoverable: false,
                            tool_name: None,
                        };
                    }
                }
            }
        }
    }

    /// Enhance documents with AI-generated examples
    #[cfg(feature = "ai-ingestion")]
    async fn enhance_documents_with_examples(
        &self,
        documents: Vec<IndexDocument>,
        tools: &[ToolDocumentation],
        generator: &ExampleGenerator,
    ) -> Result<(Vec<IndexDocument>, Vec<GeneratedExample>)> {
        let mut all_examples = Vec::new();

        // Generate examples for each tool
        for tool in tools {
            match generator.generate(tool).await {
                Ok(examples) => {
                    info!(
                        "Generated {} examples for tool '{}'",
                        examples.len(), tool.name
                    );
                    all_examples.extend(examples);
                }
                Err(e) => {
                    warn!("Failed to generate examples for '{}': {}", tool.name, e);
                }
            }
        }

        // Enhance document content with examples
        let enhanced = self.enhance_documents_inline(&documents, &all_examples);

        Ok((enhanced, all_examples))
    }

    /// Enhance document content with generated examples (inline)
    #[cfg(feature = "ai-ingestion")]
    fn enhance_documents_inline(
        &self,
        documents: &[IndexDocument],
        examples: &[GeneratedExample],
    ) -> Vec<IndexDocument> {
        if examples.is_empty() {
            return documents.to_vec();
        }

        // Build example text to append
        let example_text = Self::format_examples_for_embedding(examples);

        // Enhance each document by appending examples
        documents
            .iter()
            .map(|doc| {
                // Append examples that might be relevant to this document
                // For now, append all examples - could be more selective based on tool_name
                let enhanced_content = format!(
                    "{}\n\n## Generated Examples\n\n{}",
                    doc.content, example_text
                );

                IndexDocument {
                    id: doc.id.clone(),
                    content: enhanced_content,
                    metadata: doc.metadata.clone(),
                }
            })
            .collect()
    }

    /// Format examples for embedding text
    #[cfg(feature = "ai-ingestion")]
    fn format_examples_for_embedding(examples: &[GeneratedExample]) -> String {
        examples
            .iter()
            .map(|e| {
                format!(
                    "Example: {}\n{}",
                    e.command,
                    e.explanation
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Check if AI example generation is enabled
    #[cfg(feature = "ai-ingestion")]
    pub fn has_example_generator(&self) -> bool {
        self.example_generator.is_some()
    }

    /// Get the example generator info (provider name, model)
    #[cfg(feature = "ai-ingestion")]
    pub fn example_generator_info(&self) -> Option<(&str, &str)> {
        self.example_generator.as_ref().map(|g| {
            (g.provider_name(), g.model_name())
        })
    }

    /// Search for documents matching the query
    ///
    /// # Arguments
    /// * `query` - Natural language search query
    /// * `top_k` - Maximum number of results to return
    ///
    /// # Returns
    /// Ranked list of search results
    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<PipelineSearchResult>> {
        debug!("Searching for: {} (top_k={})", query, top_k);

        // Process query for understanding
        let processed = self.query_processor.process(query);
        let search_query = if !processed.normalized.is_empty() {
            &processed.normalized
        } else {
            query
        };

        debug!(
            "Query processed: intent={:?}, confidence={:.2}",
            processed.intent, processed.intent_confidence
        );

        // Generate query embedding
        let query_embedding = self.embedding_provider
            .embed_query(search_query)
            .await
            .context("Failed to embed query")?;

        // Determine how many candidates to fetch
        let first_stage_k = self.config.retrieval.first_stage_k.max(top_k * 2);

        // Perform search (hybrid or dense-only)
        let candidates = self.retrieve_candidates(&query_embedding, search_query, first_stage_k).await?;

        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // Rerank if enabled
        #[cfg(feature = "reranker")]
        let reranked = if let Some(ref reranker) = self.reranker {
            self.rerank_results(reranker.as_ref(), query, candidates, top_k).await?
        } else {
            candidates.into_iter().take(top_k).collect()
        };

        #[cfg(not(feature = "reranker"))]
        let reranked: Vec<PipelineSearchResult> = candidates.into_iter().take(top_k).collect();

        Ok(reranked)
    }

    /// Search with metadata filtering
    pub async fn search_with_filter(
        &self,
        query: &str,
        filter: Filter,
        top_k: usize,
    ) -> Result<Vec<PipelineSearchResult>> {
        let query_embedding = self.embedding_provider
            .embed_query(query)
            .await
            .context("Failed to embed query")?;

        let results = self.vector_store
            .search(query_embedding, Some(filter), top_k)
            .await
            .context("Vector search failed")?;

        Ok(results
            .into_iter()
            .map(|r| PipelineSearchResult {
                id: r.id,
                content: r.content.unwrap_or_default(),
                score: r.score,
                metadata: r.metadata,
                rerank_score: None,
            })
            .collect())
    }

    /// Retrieve candidates using hybrid or dense search
    async fn retrieve_candidates(
        &self,
        query_embedding: &[f32],
        _query_text: &str,
        k: usize,
    ) -> Result<Vec<PipelineSearchResult>> {
        #[cfg(feature = "hybrid-search")]
        if self.config.retrieval.enable_hybrid {
            if let Some(ref bm25) = self.bm25_index {
                return self.hybrid_retrieve(query_embedding, query_text, bm25, k).await;
            }
        }

        // Dense-only search
        let results = self.vector_store
            .search(query_embedding.to_vec(), None, k)
            .await
            .context("Vector search failed")?;

        Ok(results
            .into_iter()
            .map(|r| PipelineSearchResult {
                id: r.id,
                content: r.content.unwrap_or_default(),
                score: r.score,
                metadata: r.metadata,
                rerank_score: None,
            })
            .collect())
    }

    /// Perform hybrid retrieval (dense + BM25)
    #[cfg(feature = "hybrid-search")]
    async fn hybrid_retrieve(
        &self,
        query_embedding: &[f32],
        query_text: &str,
        bm25: &Arc<RwLock<BM25Index>>,
        k: usize,
    ) -> Result<Vec<PipelineSearchResult>> {
        use super::reciprocal_rank_fusion;

        // Dense search
        let dense_results = self.vector_store
            .search(query_embedding.to_vec(), None, k)
            .await
            .context("Dense search failed")?;

        // BM25 search
        let bm25_guard = bm25.read().await;
        let sparse_results = bm25_guard.search(query_text, k)?;

        // Convert to common format for fusion
        let dense_scores: Vec<(String, f32)> = dense_results
            .iter()
            .map(|r| (r.id.clone(), r.score))
            .collect();

        let sparse_scores: Vec<(String, f32)> = sparse_results
            .iter()
            .map(|r| (r.doc_id.clone(), r.score))
            .collect();

        // Reciprocal Rank Fusion
        let rrf_k = self.config.retrieval.rrf_k;
        let fused = reciprocal_rank_fusion(
            vec![dense_scores, sparse_scores],
            rrf_k,
        );

        // Rebuild results with fused scores
        let mut results: Vec<PipelineSearchResult> = Vec::with_capacity(k);

        for (id, score) in fused.into_iter().take(k) {
            // Find the document content from dense results or BM25
            if let Some(dense_match) = dense_results.iter().find(|r| r.id == id) {
                results.push(PipelineSearchResult {
                    id: dense_match.id.clone(),
                    content: dense_match.content.clone().unwrap_or_default(),
                    score,
                    metadata: dense_match.metadata.clone(),
                    rerank_score: None,
                });
            } else if let Some(_sparse_match) = sparse_results.iter().find(|r| r.doc_id == id) {
                // Get full document from vector store
                if let Ok(docs) = self.vector_store.get(vec![id.clone()]).await {
                    if let Some(doc) = docs.into_iter().next() {
                        results.push(PipelineSearchResult {
                            id: doc.id,
                            content: doc.content.unwrap_or_default(),
                            score,
                            metadata: doc.metadata,
                            rerank_score: None,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// Rerank results using cross-encoder
    #[cfg(feature = "reranker")]
    async fn rerank_results(
        &self,
        reranker: &dyn Reranker,
        query: &str,
        candidates: Vec<PipelineSearchResult>,
        top_k: usize,
    ) -> Result<Vec<PipelineSearchResult>> {
        if candidates.is_empty() {
            return Ok(candidates);
        }

        let rerank_docs: Vec<RerankDocument> = candidates
            .iter()
            .map(|r| RerankDocument {
                id: r.id.clone(),
                text: r.content.clone(),
            })
            .collect();

        let reranked = reranker.rerank(query, rerank_docs, top_k)?;

        // Rebuild results with rerank scores
        let results: Vec<PipelineSearchResult> = reranked
            .into_iter()
            .filter_map(|rr| {
                candidates.iter().find(|c| c.id == rr.document.id).map(|c| {
                    PipelineSearchResult {
                        id: c.id.clone(),
                        content: c.content.clone(),
                        score: c.score,
                        metadata: c.metadata.clone(),
                        rerank_score: Some(rr.score),
                    }
                })
            })
            .collect();

        Ok(results)
    }

    /// Get compressed context for LLM consumption
    #[cfg(feature = "context-compression")]
    pub fn compress_results(
        &self,
        results: &[PipelineSearchResult],
    ) -> Result<Vec<CompressedToolContext>> {
        let compressor = self.compressor.as_ref()
            .context("Context compression not enabled")?;

        let tools: Vec<_> = results
            .iter()
            .map(|r| {
                // Parse as tool if possible
                super::CompressedToolContext {
                    name: r.metadata.tool_name.clone().unwrap_or_else(|| r.id.clone()),
                    description: r.content.clone(),
                    parameters: Vec::new(),
                    example: None,
                    score: r.rerank_score.unwrap_or(r.score),
                }
            })
            .collect();

        Ok(tools)
    }

    /// Check health of all pipeline components
    pub async fn health_check(&self) -> PipelineHealth {
        let mut healthy = true;

        // Check embedding provider
        let embedding_status = match self.embedding_provider.health_check().await {
            Ok(true) => ProviderStatus {
                name: self.embedding_provider.provider_name().to_string(),
                healthy: true,
                error: None,
            },
            Ok(false) => {
                healthy = false;
                ProviderStatus {
                    name: self.embedding_provider.provider_name().to_string(),
                    healthy: false,
                    error: Some("Provider reported unhealthy".to_string()),
                }
            }
            Err(e) => {
                healthy = false;
                ProviderStatus {
                    name: self.embedding_provider.provider_name().to_string(),
                    healthy: false,
                    error: Some(e.to_string()),
                }
            }
        };

        // Check vector store
        let vector_status = match self.vector_store.health_check().await {
            Ok(status) => ProviderStatus {
                name: self.vector_store.backend_name().to_string(),
                healthy: status.healthy,
                error: if status.healthy { None } else { Some("Unhealthy".to_string()) },
            },
            Err(e) => {
                healthy = false;
                ProviderStatus {
                    name: self.vector_store.backend_name().to_string(),
                    healthy: false,
                    error: Some(e.to_string()),
                }
            }
        };

        // Check BM25 if enabled
        #[cfg(feature = "hybrid-search")]
        let bm25_status = if self.bm25_index.is_some() {
            Some(ProviderStatus {
                name: "BM25 (Tantivy)".to_string(),
                healthy: true,
                error: None,
            })
        } else {
            None
        };
        #[cfg(not(feature = "hybrid-search"))]
        let bm25_status: Option<ProviderStatus> = None;

        // Check reranker if enabled
        #[cfg(feature = "reranker")]
        let reranker_status = if let Some(ref reranker) = self.reranker {
            Some(ProviderStatus {
                name: reranker.model_name().to_string(),
                healthy: true,
                error: None,
            })
        } else {
            None
        };
        #[cfg(not(feature = "reranker"))]
        let reranker_status: Option<ProviderStatus> = None;

        // Check example generator if enabled
        #[cfg(feature = "ai-ingestion")]
        let generator_status = if let Some(ref generator) = self.example_generator {
            Some(ProviderStatus {
                name: format!("{}/{}", generator.provider_name(), generator.model_name()),
                healthy: true,
                error: None,
            })
        } else {
            None
        };
        #[cfg(not(feature = "ai-ingestion"))]
        let generator_status: Option<ProviderStatus> = None;

        let indexed = self.vector_store.count(None).await.unwrap_or(0);

        PipelineHealth {
            healthy,
            embedding_provider: embedding_status,
            vector_store: vector_status,
            bm25_index: bm25_status,
            reranker: reranker_status,
            example_generator: generator_status,
            indexed_documents: indexed,
        }
    }

    /// Get the number of indexed documents
    pub async fn document_count(&self) -> Result<usize> {
        self.vector_store.count(None).await
    }

    /// Clear all indexed documents
    pub async fn clear(&self) -> Result<()> {
        // For InMemory, we'd need to recreate it
        // For now, just warn
        warn!("Clear not fully implemented - documents may persist");
        Ok(())
    }

    /// Get the configuration
    pub fn config(&self) -> &SearchConfig {
        &self.config
    }

    /// Get the embedding provider info
    pub fn embedding_info(&self) -> (&str, &str, usize) {
        (
            self.embedding_provider.provider_name(),
            self.embedding_provider.model_name(),
            self.embedding_provider.dimensions(),
        )
    }

    /// Add known skills to the query processor for better understanding
    pub fn add_known_skill(&mut self, skill_name: &str) {
        self.known_skills.push(skill_name.to_string());
        self.rebuild_query_processor();
    }

    /// Add known tools to the query processor
    pub fn add_known_tool(&mut self, tool_name: &str) {
        self.known_tools.push(tool_name.to_string());
        self.rebuild_query_processor();
    }

    /// Add multiple known skills at once
    pub fn add_known_skills(&mut self, skills: impl IntoIterator<Item = impl Into<String>>) {
        for skill in skills {
            self.known_skills.push(skill.into());
        }
        self.rebuild_query_processor();
    }

    /// Add multiple known tools at once
    pub fn add_known_tools(&mut self, tools: impl IntoIterator<Item = impl Into<String>>) {
        for tool in tools {
            self.known_tools.push(tool.into());
        }
        self.rebuild_query_processor();
    }

    /// Rebuild the query processor with current known skills and tools
    fn rebuild_query_processor(&mut self) {
        self.query_processor = QueryProcessor::new()
            .with_skills(self.known_skills.iter().cloned())
            .with_tools(self.known_tools.iter().cloned());
    }

    /// Process a query without searching (for debugging)
    pub fn process_query(&self, query: &str) -> ProcessedQuery {
        self.query_processor.process(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_pipeline_creation() {
        let config = SearchConfig::default();
        let pipeline = SearchPipeline::from_config(config).await;
        assert!(pipeline.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_pipeline_index_and_search() {
        let config = SearchConfig::default();
        let pipeline = SearchPipeline::from_config(config).await.unwrap();

        // Index some documents
        let docs = vec![
            IndexDocument {
                id: "1".to_string(),
                content: "List all Kubernetes pods in the cluster".to_string(),
                metadata: DocumentMetadata {
                    skill_name: Some("kubernetes".to_string()),
                    tool_name: Some("list-pods".to_string()),
                    ..Default::default()
                },
            },
            IndexDocument {
                id: "2".to_string(),
                content: "Deploy a new application to Kubernetes".to_string(),
                metadata: DocumentMetadata {
                    skill_name: Some("kubernetes".to_string()),
                    tool_name: Some("deploy".to_string()),
                    ..Default::default()
                },
            },
            IndexDocument {
                id: "3".to_string(),
                content: "Create an S3 bucket in AWS".to_string(),
                metadata: DocumentMetadata {
                    skill_name: Some("aws".to_string()),
                    tool_name: Some("create-bucket".to_string()),
                    ..Default::default()
                },
            },
        ];

        let stats = pipeline.index_documents(docs).await.unwrap();
        assert_eq!(stats.documents_added, 3);
        assert_eq!(stats.total_documents, 3);

        // Search
        let results = pipeline.search("kubernetes pods", 2).await.unwrap();
        assert!(!results.is_empty());
        assert!(results.len() <= 2);

        // First result should be related to kubernetes
        assert!(results[0].content.to_lowercase().contains("kubernetes"));
    }

    #[tokio::test]
    #[serial]
    async fn test_pipeline_health_check() {
        let config = SearchConfig::default();
        let pipeline = SearchPipeline::from_config(config).await.unwrap();

        let health = pipeline.health_check().await;
        assert!(health.healthy);
        assert!(health.embedding_provider.healthy);
        assert!(health.vector_store.healthy);
    }

    #[tokio::test]
    #[serial]
    async fn test_query_processing() {
        let config = SearchConfig::default();
        let mut pipeline = SearchPipeline::from_config(config).await.unwrap();

        pipeline.add_known_skill("kubernetes");
        pipeline.add_known_tool("list-pods");

        let processed = pipeline.process_query("how do I list k8s pods?");
        assert!(!processed.normalized.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_empty_search() {
        let config = SearchConfig::default();
        let pipeline = SearchPipeline::from_config(config).await.unwrap();

        // Search without indexing
        let results = pipeline.search("kubernetes", 5).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_search_with_filter() {
        let config = SearchConfig::default();
        let pipeline = SearchPipeline::from_config(config).await.unwrap();

        // Index documents
        let docs = vec![
            IndexDocument {
                id: "1".to_string(),
                content: "Kubernetes pods".to_string(),
                metadata: DocumentMetadata {
                    skill_name: Some("kubernetes".to_string()),
                    ..Default::default()
                },
            },
            IndexDocument {
                id: "2".to_string(),
                content: "AWS S3 bucket".to_string(),
                metadata: DocumentMetadata {
                    skill_name: Some("aws".to_string()),
                    ..Default::default()
                },
            },
        ];
        pipeline.index_documents(docs).await.unwrap();

        // Search with filter
        let filter = Filter::new().skill("kubernetes");
        let results = pipeline.search_with_filter("bucket", filter, 5).await.unwrap();

        // Should only return kubernetes results even though we searched for "bucket"
        for result in &results {
            if let Some(ref skill) = result.metadata.skill_name {
                assert_eq!(skill, "kubernetes");
            }
        }
    }
}
