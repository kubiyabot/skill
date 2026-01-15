//! Search module for hybrid retrieval
//!
//! Provides combined dense (vector) and sparse (BM25) search capabilities
//! with Reciprocal Rank Fusion for optimal result quality.
//!
//! Also includes cross-encoder reranking for improved precision,
//! context compression for token-efficient output, query understanding
//! for intelligent search preprocessing, and persistent index management.

#[cfg(feature = "hybrid-search")]
mod bm25;
#[cfg(feature = "hybrid-search")]
mod hybrid;
mod fusion;
#[cfg(feature = "reranker")]
mod reranker;
#[cfg(feature = "context-compression")]
mod context;
mod query_processor;
mod index_manager;
mod pipeline;

pub use fusion::{FusionMethod, reciprocal_rank_fusion, weighted_sum_fusion};

#[cfg(feature = "hybrid-search")]
pub use bm25::{BM25Index, BM25Config, BM25SearchResult};
#[cfg(feature = "hybrid-search")]
pub use hybrid::{HybridRetriever, HybridConfig, HybridSearchResult};

#[cfg(feature = "reranker")]
pub use reranker::{
    Reranker, RerankResult, RerankDocument,
    FastEmbedReranker, RerankerModel, RerankerConfig,
};

#[cfg(feature = "context-compression")]
pub use context::{
    ContextCompressor, CompressionStrategy, CompressionConfig,
    CompressedToolContext, ToolParameter, CompressionResult,
};

pub use query_processor::{
    QueryProcessor, QueryIntent, ExtractedEntity, EntityType,
    ProcessedQuery, QueryExpansion,
};

pub use index_manager::{
    IndexManager, IndexConfig, IndexMetadata, SkillChecksum,
    IndexStats, SyncResult,
};

pub use pipeline::{
    SearchPipeline, PipelineSearchResult, PipelineIndexStats,
    PipelineHealth, ProviderStatus, IndexDocument,
};
