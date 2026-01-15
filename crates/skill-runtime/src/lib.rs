//! Skill Runtime - Universal execution engine for AI agent skills
//!
//! This crate provides a secure, portable runtime for executing AI agent skills across multiple
//! runtime types: WASM Component Model, Docker containers, and native command execution.
//!
//! # Features
//!
//! - **WASM Sandbox**: Execute skills in isolated WASM environments with capability-based security
//! - **Docker Runtime**: Run containerized skills with full environment control
//! - **Native Execution**: Direct command execution for system tools (kubectl, git, etc.)
//! - **RAG-Powered Search**: Semantic search with hybrid retrieval, reranking, and context compression
//! - **Multi-Instance Support**: Configure multiple instances per skill (dev/staging/prod)
//! - **Audit Logging**: Comprehensive execution tracking and security auditing
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use skill_runtime::{SkillEngine, SkillManifest};
//!
//! # async fn run() -> anyhow::Result<()> {
//! // Initialize the runtime
//! let engine = SkillEngine::new()?;
//!
//! // Load a skill manifest
//! let manifest = SkillManifest::from_file(".skill-engine.toml")?;
//!
//! // Execute a tool
//! let result = engine.execute_tool("kubernetes", "get", serde_json::json!({
//!     "resource": "pods",
//!     "namespace": "default"
//! })).await?;
//!
//! println!("Result: {}", result);
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           SkillEngine                    │
//! │  (Orchestrates execution & search)       │
//! └─────────────────────────────────────────┘
//!                   │
//!       ┌───────────┼───────────┐
//!       ▼           ▼           ▼
//! ┌─────────┐ ┌──────────┐ ┌────────────┐
//! │  WASM   │ │  Docker  │ │   Native   │
//! │ Runtime │ │ Runtime  │ │  Executor  │
//! └─────────┘ └──────────┘ └────────────┘
//!       │           │           │
//!       └───────────┴───────────┘
//!                   │
//!       ┌───────────┴───────────┐
//!       ▼                       ▼
//! ┌──────────────┐    ┌────────────────┐
//! │ Vector Store │    │  Audit Logger  │
//! │ (Search)     │    │  (Security)    │
//! └──────────────┘    └────────────────┘
//! ```
//!
//! # Security Model
//!
//! Skills execute with capability-based security:
//!
//! - **WASI Sandbox**: Network and filesystem access must be explicitly granted
//! - **Command Allowlist**: Native skills declare allowed commands in `allowed-tools`
//! - **Docker Isolation**: Containerized skills run in separate namespaces
//! - **Audit Trail**: All executions are logged with timestamps and arguments
//!
//! # Performance
//!
//! - WASM cold start: ~100ms (includes AOT compilation)
//! - WASM warm start: <10ms (cached)
//! - Vector search: <50ms (384-dim embeddings)
//! - Native commands: Near-instant (direct execution)
//!
//! # Feature Flags
//!
//! - `hybrid-search`: BM25 + dense vector fusion with RRF
//! - `reranker`: Cross-encoder reranking for improved precision
//! - `context-compression`: Token-aware output compression
//! - `qdrant`: Production vector database backend
//! - `job-queue`: Async job scheduling and execution
//! - `sqlite-storage`: SQLite-backed job storage

#![warn(missing_docs)]

pub mod audit;
pub mod config_mapper;
pub mod credentials;
pub mod docker_runtime;
pub mod engine;
pub mod errors;
pub mod executor;
pub mod generation;
pub mod git_loader;
pub mod git_source;
pub mod instance;
pub mod local_loader;
pub mod manifest;
pub mod metrics;
pub mod sandbox;
pub mod skill_md;
pub mod types;
pub mod vector_store;
pub mod embeddings;
pub mod search;
pub mod search_config;

#[cfg(feature = "job-queue")]
pub mod jobs;

pub use audit::{AuditEntry, AuditEventType, AuditLogger};
pub use config_mapper::ConfigMapper;
pub use credentials::{parse_keyring_reference, CredentialStore, SecureString};
pub use engine::SkillEngine;
pub use errors::{RuntimeError, Result};
pub use executor::{ComponentCache, SkillExecutor};
pub use git_loader::{ClonedSkill, GitSkillLoader, SkillType};
pub use git_source::{is_git_url, parse_git_url, GitRef, GitSource};
pub use instance::{InstanceConfig, InstanceManager};
pub use local_loader::LocalSkillLoader;
pub use docker_runtime::{DockerOutput, DockerRuntime, DockerSecurityPolicy};
pub use manifest::{
    DockerRuntimeConfig, ServiceRequirement, SkillManifest, SkillRuntime, ResolvedInstance, SkillInfo, expand_env_vars
};
pub use metrics::ExecutionMetrics;
pub use sandbox::{HostState, SandboxBuilder};
pub use skill_md::{
    parse_skill_md, parse_skill_md_content, find_skill_md,
    SkillMdContent, SkillMdFrontmatter, ToolDocumentation, CodeExample, ParameterDoc
};
pub use types::*;
pub use vector_store::{
    VectorStore, InMemoryVectorStore,
    EmbeddedDocument, DocumentMetadata, Filter, SearchResult,
    UpsertStats, DeleteStats, HealthStatus, DistanceMetric,
    cosine_similarity, euclidean_distance,
};

#[cfg(feature = "qdrant")]
pub use vector_store::{QdrantVectorStore, QdrantConfig};
pub use embeddings::{
    EmbeddingProvider, EmbeddingConfig, EmbeddingProviderType,
    FastEmbedProvider, FastEmbedModel,
    OpenAIEmbedProvider, OpenAIEmbeddingModel,
    OllamaProvider,
    EmbeddingProviderFactory, create_provider,
};

pub use search::{FusionMethod, reciprocal_rank_fusion, weighted_sum_fusion};

#[cfg(feature = "hybrid-search")]
pub use search::{BM25Index, BM25Config, BM25SearchResult, HybridRetriever, HybridConfig, HybridSearchResult};

#[cfg(feature = "reranker")]
pub use search::{Reranker, RerankResult, RerankDocument, FastEmbedReranker, RerankerModel, RerankerConfig};

#[cfg(feature = "context-compression")]
pub use search::{
    ContextCompressor, CompressionStrategy, CompressionConfig,
    CompressedToolContext, ToolParameter, CompressionResult,
};

pub use search::{
    QueryProcessor, QueryIntent, ExtractedEntity, EntityType,
    ProcessedQuery, QueryExpansion,
};

pub use search::{
    IndexManager, IndexMetadata, SkillChecksum,
    IndexStats, SyncResult,
};

pub use search::{
    SearchPipeline, PipelineSearchResult, PipelineIndexStats,
    PipelineHealth, ProviderStatus, IndexDocument,
};

pub use search_config::{
    SearchConfig, BackendConfig, BackendType,
    EmbeddingConfig as SearchEmbeddingConfig,
    RetrievalConfig, RerankerConfig as SearchRerankerConfig,
    ContextConfig, QdrantConfig as SearchQdrantConfig,
    IndexConfig as SearchIndexConfig,
    FusionMethod as SearchFusionMethod,
    CompressionStrategy as SearchCompressionStrategy,
    AiIngestionConfig, AiProvider,
    OllamaLlmConfig, OpenAiLlmConfig, AnthropicLlmConfig,
};

pub use generation::{
    GenerationEvent, GeneratedExample, AgentStep,
    SearchResultRef, GenerationStreamBuilder,
    LlmProvider, LlmResponse, LlmChunk, TokenUsage,
    ChatMessage, CompletionRequest, create_llm_provider,
    ExampleValidator, ValidationResult, ParsedCommand,
    ExampleGenerator, GeneratorConfig,
};

#[cfg(feature = "ollama")]
pub use generation::OllamaProvider;

#[cfg(feature = "openai")]
pub use generation::OpenAIProvider;

#[cfg(feature = "job-queue")]
pub use jobs::{
    JobConfig, StorageBackend, ConfigError as JobConfigError,
    Job, JobId, JobStatus, JobPriority, JobType, JobProgress, JobStats,
    MaintenanceTask, JobStorage, JobFilter, JobQueue,
    StorageError, StorageResult, create_storage, create_job_queue,
    WorkerConfig, WorkerPool, WorkerPoolStats, WorkerPoolError,
    JobHandler, JobError, WorkerContext, PoolState, LoggingJobHandler,
};

#[cfg(feature = "sqlite-storage")]
pub use jobs::SqliteJobStorage;

/// Initialize the skill runtime
///
///  Creates a new [`SkillEngine`] instance with default configuration.
///
/// # Returns
///
/// Returns a configured `SkillEngine` ready to load and execute skills.
///
/// # Errors
///
/// Returns an error if the runtime fails to initialize, typically due to:
/// - Missing dependencies (e.g., Wasmtime components)
/// - Invalid system configuration
/// - Insufficient permissions
///
/// # Example
///
/// ```rust,no_run
/// use skill_runtime::init;
///
/// # fn main() -> anyhow::Result<()> {
/// let engine = init()?;
/// // Use the engine to load and execute skills
/// # Ok(())
/// # }
/// ```
pub fn init() -> anyhow::Result<SkillEngine> {
    SkillEngine::new()
}
