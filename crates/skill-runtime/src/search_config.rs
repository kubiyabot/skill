//! Configuration schema for RAG search pipeline
//!
//! Provides comprehensive configuration for embedding providers, vector backends,
//! hybrid retrieval, reranking, and context compression.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Root search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Vector store backend
    #[serde(default)]
    pub backend: BackendConfig,

    /// Embedding configuration
    #[serde(default)]
    pub embedding: EmbeddingConfig,

    /// Retrieval configuration
    #[serde(default)]
    pub retrieval: RetrievalConfig,

    /// Reranker configuration
    #[serde(default)]
    pub reranker: RerankerConfig,

    /// Context compression configuration
    #[serde(default)]
    pub context: ContextConfig,

    /// File-based vector store configuration (if backend = "file")
    #[serde(default)]
    pub file: Option<FileConfig>,

    /// Qdrant-specific configuration (if backend = "qdrant")
    #[serde(default)]
    pub qdrant: Option<QdrantConfig>,

    /// Index configuration
    #[serde(default)]
    pub index: IndexConfig,

    /// AI-powered example generation during ingestion
    #[serde(default)]
    pub ai_ingestion: AiIngestionConfig,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            backend: BackendConfig::default(),
            embedding: EmbeddingConfig::default(),
            retrieval: RetrievalConfig::default(),
            reranker: RerankerConfig::default(),
            context: ContextConfig::default(),
            file: None,
            qdrant: None,
            index: IndexConfig::default(),
            ai_ingestion: AiIngestionConfig::default(),
        }
    }
}

impl SearchConfig {
    /// Load config from TOML file
    pub fn from_toml_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        Self::from_toml(&content)
    }

    /// Parse from TOML string
    ///
    /// Supports both wrapped format (with `[search]` section) and unwrapped format.
    pub fn from_toml(content: &str) -> Result<Self> {
        // Check if the TOML uses wrapped format (has [search] section)
        let is_wrapped = content.contains("[search]") || content.contains("[search.");

        if is_wrapped {
            // Wrapped format (with [search] section)
            #[derive(Deserialize)]
            struct Wrapper {
                #[serde(default)]
                search: Option<SearchConfig>,
            }

            let wrapper: Wrapper = toml::from_str(content)
                .context("Failed to parse TOML config (wrapped format)")?;

            Ok(wrapper.search.unwrap_or_default())
        } else {
            // Unwrapped format (direct sections like [embedding], [backend], etc.)
            toml::from_str::<SearchConfig>(content)
                .context("Failed to parse TOML config (unwrapped format)")
        }
    }

    /// Apply environment variable overrides
    pub fn with_env_overrides(mut self) -> Self {
        // Backend
        if let Ok(val) = std::env::var("SKILL_SEARCH_BACKEND") {
            self.backend.backend_type = val.parse().unwrap_or_default();
        }

        // Embedding
        if let Ok(val) = std::env::var("SKILL_EMBEDDING_PROVIDER") {
            self.embedding.provider = val;
        }
        if let Ok(val) = std::env::var("SKILL_EMBEDDING_MODEL") {
            self.embedding.model = val;
        }
        if let Ok(val) = std::env::var("SKILL_EMBEDDING_DIMENSIONS") {
            if let Ok(dims) = val.parse() {
                self.embedding.dimensions = dims;
            }
        }

        // Retrieval
        if let Ok(val) = std::env::var("SKILL_SEARCH_ENABLE_HYBRID") {
            self.retrieval.enable_hybrid = val.parse().unwrap_or(true);
        }
        if let Ok(val) = std::env::var("SKILL_SEARCH_DENSE_WEIGHT") {
            if let Ok(weight) = val.parse() {
                self.retrieval.dense_weight = weight;
            }
        }
        if let Ok(val) = std::env::var("SKILL_SEARCH_TOP_K") {
            if let Ok(k) = val.parse() {
                self.retrieval.final_k = k;
            }
        }

        // Reranker
        if let Ok(val) = std::env::var("SKILL_RERANKER_ENABLED") {
            self.reranker.enabled = val.parse().unwrap_or(false);
        }
        if let Ok(val) = std::env::var("SKILL_RERANKER_MODEL") {
            self.reranker.model = val;
        }

        // Context
        if let Ok(val) = std::env::var("SKILL_CONTEXT_MAX_TOKENS") {
            if let Ok(tokens) = val.parse() {
                self.context.max_total_tokens = tokens;
            }
        }

        // Qdrant
        if let Ok(url) = std::env::var("QDRANT_URL") {
            let qdrant = self.qdrant.get_or_insert_with(QdrantConfig::default);
            qdrant.url = url;
        }
        if let Ok(key) = std::env::var("QDRANT_API_KEY") {
            let qdrant = self.qdrant.get_or_insert_with(QdrantConfig::default);
            qdrant.api_key = Some(key);
        }

        // AI Ingestion
        if let Ok(val) = std::env::var("SKILL_AI_INGESTION_ENABLED") {
            self.ai_ingestion.enabled = val.parse().unwrap_or(false);
        }
        if let Ok(val) = std::env::var("SKILL_AI_INGESTION_PROVIDER") {
            self.ai_ingestion.provider = val.parse().unwrap_or_default();
        }
        if let Ok(val) = std::env::var("SKILL_AI_INGESTION_MODEL") {
            self.ai_ingestion.model = val;
        }
        if let Ok(val) = std::env::var("SKILL_AI_EXAMPLES_PER_TOOL") {
            if let Ok(n) = val.parse() {
                self.ai_ingestion.examples_per_tool = n;
            }
        }
        if let Ok(val) = std::env::var("OLLAMA_HOST") {
            self.ai_ingestion.ollama.host = val;
        }
        if let Ok(_) = std::env::var("OPENAI_API_KEY") {
            self.ai_ingestion.openai.api_key_env = Some("OPENAI_API_KEY".to_string());
        }
        if let Ok(_) = std::env::var("ANTHROPIC_API_KEY") {
            self.ai_ingestion.anthropic.api_key_env = Some("ANTHROPIC_API_KEY".to_string());
        }

        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate embedding dimensions
        if self.embedding.dimensions == 0 {
            anyhow::bail!("Embedding dimensions must be > 0");
        }

        // Validate retrieval weights
        if self.retrieval.enable_hybrid {
            let total_weight = self.retrieval.dense_weight + self.retrieval.sparse_weight;
            if (total_weight - 1.0).abs() > 0.01 {
                anyhow::bail!("Dense and sparse weights should sum to 1.0");
            }
        }

        // Validate retrieval k values
        if self.retrieval.final_k > self.retrieval.rerank_k {
            anyhow::bail!("final_k cannot be greater than rerank_k");
        }
        if self.retrieval.rerank_k > self.retrieval.first_stage_k {
            anyhow::bail!("rerank_k cannot be greater than first_stage_k");
        }

        // Validate context tokens
        if self.context.max_tokens_per_result > self.context.max_total_tokens {
            anyhow::bail!("max_tokens_per_result cannot exceed max_total_tokens");
        }

        // Validate File config if using File backend
        if matches!(self.backend.backend_type, BackendType::File) {
            // File config is optional (uses default ~/.skill-engine/vectors/store.bin)
            // No validation needed
        }

        // Validate Qdrant config if using Qdrant backend
        if matches!(self.backend.backend_type, BackendType::Qdrant) {
            if self.qdrant.is_none() {
                anyhow::bail!("Qdrant configuration required when backend = 'qdrant'");
            }
        }

        // Validate AI ingestion config
        if self.ai_ingestion.enabled {
            if self.ai_ingestion.examples_per_tool == 0 {
                anyhow::bail!("examples_per_tool must be > 0 when AI ingestion is enabled");
            }
            if self.ai_ingestion.timeout_secs == 0 {
                anyhow::bail!("timeout_secs must be > 0 when AI ingestion is enabled");
            }
        }

        Ok(())
    }
}

/// Vector store backend type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendType {
    /// File-based vector store (default) - persistent local storage
    #[default]
    File,
    /// In-memory vector store - fast but no persistence
    InMemory,
    /// Qdrant vector database - production-grade with Docker
    Qdrant,
}

impl std::str::FromStr for BackendType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "file" => Ok(Self::File),
            "in-memory" | "inmemory" | "memory" => Ok(Self::InMemory),
            "qdrant" => Ok(Self::Qdrant),
            _ => anyhow::bail!("Unknown backend type: {}. Options: file, in-memory, qdrant", s),
        }
    }
}

/// Backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Backend type
    #[serde(default, rename = "type")]
    pub backend_type: BackendType,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            backend_type: BackendType::default(),
        }
    }
}

/// Embedding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Embedding provider (fastembed, openai, ollama)
    #[serde(default = "default_embedding_provider")]
    pub provider: String,

    /// Model name
    #[serde(default = "default_embedding_model")]
    pub model: String,

    /// Embedding dimensions
    #[serde(default = "default_embedding_dimensions")]
    pub dimensions: usize,

    /// Batch size for embedding generation
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// OpenAI API key (if provider = "openai")
    pub openai_api_key: Option<String>,

    /// Ollama host (if provider = "ollama")
    pub ollama_host: Option<String>,
}

fn default_embedding_provider() -> String { "fastembed".to_string() }
fn default_embedding_model() -> String { "all-minilm".to_string() }
fn default_embedding_dimensions() -> usize { 384 }
fn default_batch_size() -> usize { 32 }

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: default_embedding_provider(),
            model: default_embedding_model(),
            dimensions: default_embedding_dimensions(),
            batch_size: default_batch_size(),
            openai_api_key: None,
            ollama_host: None,
        }
    }
}

/// Retrieval configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalConfig {
    /// Enable hybrid (dense + sparse) search
    #[serde(default = "default_enable_hybrid")]
    pub enable_hybrid: bool,

    /// Weight for dense (vector) search
    #[serde(default = "default_dense_weight")]
    pub dense_weight: f32,

    /// Weight for sparse (BM25) search
    #[serde(default = "default_sparse_weight")]
    pub sparse_weight: f32,

    /// Number of results for first stage retrieval
    #[serde(default = "default_first_stage_k")]
    pub first_stage_k: usize,

    /// Number of results to rerank
    #[serde(default = "default_rerank_k")]
    pub rerank_k: usize,

    /// Final number of results to return
    #[serde(default = "default_final_k")]
    pub final_k: usize,

    /// Fusion method for hybrid search
    #[serde(default)]
    pub fusion_method: FusionMethod,

    /// RRF k parameter (for reciprocal rank fusion)
    #[serde(default = "default_rrf_k")]
    pub rrf_k: f32,
}

fn default_enable_hybrid() -> bool { true }
fn default_dense_weight() -> f32 { 0.7 }
fn default_sparse_weight() -> f32 { 0.3 }
fn default_first_stage_k() -> usize { 100 }
fn default_rerank_k() -> usize { 20 }
fn default_final_k() -> usize { 5 }
fn default_rrf_k() -> f32 { 60.0 }

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            enable_hybrid: default_enable_hybrid(),
            dense_weight: default_dense_weight(),
            sparse_weight: default_sparse_weight(),
            first_stage_k: default_first_stage_k(),
            rerank_k: default_rerank_k(),
            final_k: default_final_k(),
            fusion_method: FusionMethod::default(),
            rrf_k: default_rrf_k(),
        }
    }
}

/// Fusion method for combining search results
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FusionMethod {
    /// Reciprocal Rank Fusion (default)
    #[default]
    ReciprocalRank,
    /// Weighted sum of normalized scores
    WeightedSum,
    /// Take maximum score
    MaxScore,
}

/// Reranker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankerConfig {
    /// Enable reranking
    #[serde(default)]
    pub enabled: bool,

    /// Reranker provider (fastembed, cohere)
    #[serde(default = "default_reranker_provider")]
    pub provider: String,

    /// Reranker model
    #[serde(default = "default_reranker_model")]
    pub model: String,

    /// Maximum documents to rerank
    #[serde(default = "default_max_rerank_docs")]
    pub max_documents: usize,

    /// Cohere API key (if provider = "cohere")
    pub cohere_api_key: Option<String>,
}

fn default_reranker_provider() -> String { "fastembed".to_string() }
fn default_reranker_model() -> String { "bge-reranker-base".to_string() }
fn default_max_rerank_docs() -> usize { 50 }

impl Default for RerankerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_reranker_provider(),
            model: default_reranker_model(),
            max_documents: default_max_rerank_docs(),
            cohere_api_key: None,
        }
    }
}

/// Context compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Maximum tokens per result
    #[serde(default = "default_max_tokens_per_result")]
    pub max_tokens_per_result: usize,

    /// Maximum total tokens
    #[serde(default = "default_max_total_tokens")]
    pub max_total_tokens: usize,

    /// Include code examples in output
    #[serde(default)]
    pub include_examples: bool,

    /// Compression strategy
    #[serde(default)]
    pub compression: CompressionStrategy,
}

fn default_max_tokens_per_result() -> usize { 200 }
fn default_max_total_tokens() -> usize { 800 }

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens_per_result: default_max_tokens_per_result(),
            max_total_tokens: default_max_total_tokens(),
            include_examples: false,
            compression: CompressionStrategy::default(),
        }
    }
}

/// Compression strategy
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionStrategy {
    /// Keep first sentence + parameters
    Extractive,
    /// Template-based structured format (default)
    #[default]
    Template,
    /// Progressive detail based on rank
    Progressive,
    /// No compression
    None,
}

/// File-based vector store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    /// Custom storage directory (defaults to ~/.skill-engine/vectors/store.bin)
    pub storage_path: Option<PathBuf>,

    /// Distance metric for similarity calculation
    #[serde(default)]
    pub distance_metric: crate::vector_store::DistanceMetric,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            storage_path: None,
            distance_metric: crate::vector_store::DistanceMetric::Cosine,
        }
    }
}

/// Qdrant-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    /// Qdrant URL
    #[serde(default = "default_qdrant_url")]
    pub url: String,

    /// API key (optional, for Qdrant Cloud)
    pub api_key: Option<String>,

    /// Collection name
    #[serde(default = "default_collection_name")]
    pub collection: String,

    /// Enable TLS
    #[serde(default)]
    pub tls: bool,
}

fn default_qdrant_url() -> String { "http://localhost:6334".to_string() }
fn default_collection_name() -> String { "skill-tools".to_string() }

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: default_qdrant_url(),
            api_key: None,
            collection: default_collection_name(),
            tls: false,
        }
    }
}

/// Index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Index directory path
    pub path: Option<PathBuf>,

    /// Index on startup
    #[serde(default = "default_index_on_startup")]
    pub index_on_startup: bool,

    /// Watch for skill changes
    #[serde(default)]
    pub watch_for_changes: bool,
}

fn default_index_on_startup() -> bool { true }

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            path: None,
            index_on_startup: default_index_on_startup(),
            watch_for_changes: false,
        }
    }
}

// =============================================================================
// AI Ingestion Configuration
// =============================================================================

/// LLM provider for AI-powered example generation
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    /// Ollama for local inference (default)
    #[default]
    Ollama,
    /// OpenAI API
    OpenAi,
    /// Anthropic Claude API
    Anthropic,
}

impl std::str::FromStr for AiProvider {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ollama" => Ok(Self::Ollama),
            "openai" => Ok(Self::OpenAi),
            "anthropic" | "claude" => Ok(Self::Anthropic),
            _ => anyhow::bail!("Unknown AI provider: {}. Options: ollama, openai, anthropic", s),
        }
    }
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProvider::Ollama => write!(f, "ollama"),
            AiProvider::OpenAi => write!(f, "openai"),
            AiProvider::Anthropic => write!(f, "anthropic"),
        }
    }
}

/// AI-powered ingestion configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiIngestionConfig {
    /// Enable AI example generation during skill indexing
    #[serde(default)]
    pub enabled: bool,

    /// Number of examples to generate per tool
    #[serde(default = "default_examples_per_tool")]
    pub examples_per_tool: usize,

    /// LLM provider for generation
    #[serde(default)]
    pub provider: AiProvider,

    /// Model name (provider-specific)
    #[serde(default = "default_ai_model")]
    pub model: String,

    /// Validate generated examples against tool schema
    #[serde(default = "default_validate_examples")]
    pub validate_examples: bool,

    /// Stream generation progress to terminal/MCP
    #[serde(default = "default_stream_progress")]
    pub stream_progress: bool,

    /// Cache generated examples (skip regeneration if tool unchanged)
    #[serde(default = "default_cache_examples")]
    pub cache_examples: bool,

    /// Timeout per tool generation in seconds
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    /// Ollama-specific configuration
    #[serde(default)]
    pub ollama: OllamaLlmConfig,

    /// OpenAI-specific configuration
    #[serde(default)]
    pub openai: OpenAiLlmConfig,

    /// Anthropic-specific configuration
    #[serde(default)]
    pub anthropic: AnthropicLlmConfig,
}

fn default_examples_per_tool() -> usize { 5 }
fn default_ai_model() -> String { "llama3.2".to_string() }
fn default_validate_examples() -> bool { true }
fn default_stream_progress() -> bool { true }
fn default_cache_examples() -> bool { true }
fn default_timeout_secs() -> u64 { 30 }

impl Default for AiIngestionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            examples_per_tool: default_examples_per_tool(),
            provider: AiProvider::default(),
            model: default_ai_model(),
            validate_examples: default_validate_examples(),
            stream_progress: default_stream_progress(),
            cache_examples: default_cache_examples(),
            timeout_secs: default_timeout_secs(),
            ollama: OllamaLlmConfig::default(),
            openai: OpenAiLlmConfig::default(),
            anthropic: AnthropicLlmConfig::default(),
        }
    }
}

impl AiIngestionConfig {
    /// Get the model name for the current provider
    pub fn get_model(&self) -> &str {
        if !self.model.is_empty() {
            return &self.model;
        }
        match self.provider {
            AiProvider::Ollama => &self.ollama.model,
            AiProvider::OpenAi => &self.openai.model,
            AiProvider::Anthropic => &self.anthropic.model,
        }
    }
}

/// Ollama LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaLlmConfig {
    /// Ollama API host
    #[serde(default = "default_ollama_host")]
    pub host: String,

    /// Model to use (if not set in parent config)
    #[serde(default = "default_ollama_model")]
    pub model: String,
}

fn default_ollama_host() -> String { "http://localhost:11434".to_string() }
fn default_ollama_model() -> String { "llama3.2".to_string() }

impl Default for OllamaLlmConfig {
    fn default() -> Self {
        Self {
            host: default_ollama_host(),
            model: default_ollama_model(),
        }
    }
}

/// OpenAI LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiLlmConfig {
    /// API key environment variable name (default: OPENAI_API_KEY)
    #[serde(default)]
    pub api_key_env: Option<String>,

    /// Model to use (if not set in parent config)
    #[serde(default = "default_openai_llm_model")]
    pub model: String,

    /// Max tokens for completion
    #[serde(default = "default_openai_max_tokens")]
    pub max_tokens: u32,

    /// Temperature for generation
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_openai_llm_model() -> String { "gpt-4o-mini".to_string() }
fn default_openai_max_tokens() -> u32 { 2048 }
fn default_temperature() -> f32 { 0.7 }

impl Default for OpenAiLlmConfig {
    fn default() -> Self {
        Self {
            api_key_env: None,
            model: default_openai_llm_model(),
            max_tokens: default_openai_max_tokens(),
            temperature: default_temperature(),
        }
    }
}

/// Anthropic Claude LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicLlmConfig {
    /// API key environment variable name (default: ANTHROPIC_API_KEY)
    #[serde(default)]
    pub api_key_env: Option<String>,

    /// Model to use (if not set in parent config)
    #[serde(default = "default_anthropic_model")]
    pub model: String,

    /// Max tokens for completion
    #[serde(default = "default_anthropic_max_tokens")]
    pub max_tokens: u32,

    /// Temperature for generation
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_anthropic_model() -> String { "claude-3-haiku-20240307".to_string() }
fn default_anthropic_max_tokens() -> u32 { 2048 }

impl Default for AnthropicLlmConfig {
    fn default() -> Self {
        Self {
            api_key_env: None,
            model: default_anthropic_model(),
            max_tokens: default_anthropic_max_tokens(),
            temperature: default_temperature(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SearchConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.embedding.provider, "fastembed");
        assert_eq!(config.embedding.dimensions, 384);
        assert!(config.retrieval.enable_hybrid);
        assert!(!config.reranker.enabled);
    }

    #[test]
    fn test_parse_toml() {
        let toml = r#"
[search]
backend = { type = "qdrant" }

[search.embedding]
provider = "openai"
model = "text-embedding-3-small"
dimensions = 1536

[search.retrieval]
enable_hybrid = true
dense_weight = 0.8
sparse_weight = 0.2
final_k = 10

[search.reranker]
enabled = true
model = "bge-reranker-large"

[search.context]
max_total_tokens = 1000
compression = "progressive"

[search.qdrant]
url = "http://qdrant:6334"
collection = "my-tools"
"#;

        let config = SearchConfig::from_toml(toml).unwrap();

        assert!(matches!(config.backend.backend_type, BackendType::Qdrant));
        assert_eq!(config.embedding.provider, "openai");
        assert_eq!(config.embedding.dimensions, 1536);
        assert!((config.retrieval.dense_weight - 0.8).abs() < 0.001);
        assert_eq!(config.retrieval.final_k, 10);
        assert!(config.reranker.enabled);
        assert_eq!(config.reranker.model, "bge-reranker-large");
        assert!(matches!(config.context.compression, CompressionStrategy::Progressive));
        assert_eq!(config.qdrant.as_ref().unwrap().url, "http://qdrant:6334");
    }

    #[test]
    fn test_validation_weights() {
        let mut config = SearchConfig::default();
        config.retrieval.dense_weight = 0.5;
        config.retrieval.sparse_weight = 0.3; // Sum is 0.8, not 1.0

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_k_values() {
        let mut config = SearchConfig::default();
        config.retrieval.final_k = 50;
        config.retrieval.rerank_k = 20; // final_k > rerank_k

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_qdrant_required() {
        let mut config = SearchConfig::default();
        config.backend.backend_type = BackendType::Qdrant;
        config.qdrant = None;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_backend_type_from_str() {
        assert!(matches!("in-memory".parse::<BackendType>().unwrap(), BackendType::InMemory));
        assert!(matches!("inmemory".parse::<BackendType>().unwrap(), BackendType::InMemory));
        assert!(matches!("qdrant".parse::<BackendType>().unwrap(), BackendType::Qdrant));
        assert!("invalid".parse::<BackendType>().is_err());
    }

    #[test]
    fn test_env_overrides() {
        std::env::set_var("SKILL_SEARCH_BACKEND", "qdrant");
        std::env::set_var("SKILL_EMBEDDING_DIMENSIONS", "768");
        std::env::set_var("SKILL_RERANKER_ENABLED", "true");
        std::env::set_var("QDRANT_URL", "http://custom:6334");

        let config = SearchConfig::default().with_env_overrides();

        assert!(matches!(config.backend.backend_type, BackendType::Qdrant));
        assert_eq!(config.embedding.dimensions, 768);
        assert!(config.reranker.enabled);
        assert_eq!(config.qdrant.as_ref().unwrap().url, "http://custom:6334");

        // Clean up
        std::env::remove_var("SKILL_SEARCH_BACKEND");
        std::env::remove_var("SKILL_EMBEDDING_DIMENSIONS");
        std::env::remove_var("SKILL_RERANKER_ENABLED");
        std::env::remove_var("QDRANT_URL");
    }

    #[test]
    fn test_minimal_toml() {
        let toml = r#"
[search]
"#;

        let config = SearchConfig::from_toml(toml).unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_empty_file() {
        let toml = "";
        let config = SearchConfig::from_toml(toml).unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_ai_ingestion_defaults() {
        let config = AiIngestionConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.examples_per_tool, 5);
        assert!(matches!(config.provider, AiProvider::Ollama));
        assert_eq!(config.model, "llama3.2");
        assert!(config.validate_examples);
        assert!(config.stream_progress);
        assert!(config.cache_examples);
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_ai_provider_from_str() {
        assert!(matches!("ollama".parse::<AiProvider>().unwrap(), AiProvider::Ollama));
        assert!(matches!("openai".parse::<AiProvider>().unwrap(), AiProvider::OpenAi));
        assert!(matches!("anthropic".parse::<AiProvider>().unwrap(), AiProvider::Anthropic));
        assert!(matches!("claude".parse::<AiProvider>().unwrap(), AiProvider::Anthropic));
        assert!("invalid".parse::<AiProvider>().is_err());
    }

    #[test]
    fn test_ai_ingestion_toml_parsing() {
        let toml = r#"
[ai_ingestion]
enabled = true
examples_per_tool = 3
provider = "openai"
model = "gpt-4o"
validate_examples = false
stream_progress = true
timeout_secs = 60

[ai_ingestion.openai]
model = "gpt-4o-mini"
max_tokens = 4096
temperature = 0.5
"#;

        let config: SearchConfig = toml::from_str(toml).unwrap();
        assert!(config.ai_ingestion.enabled);
        assert_eq!(config.ai_ingestion.examples_per_tool, 3);
        assert!(matches!(config.ai_ingestion.provider, AiProvider::OpenAi));
        assert_eq!(config.ai_ingestion.model, "gpt-4o");
        assert!(!config.ai_ingestion.validate_examples);
        assert_eq!(config.ai_ingestion.timeout_secs, 60);
        assert_eq!(config.ai_ingestion.openai.model, "gpt-4o-mini");
        assert_eq!(config.ai_ingestion.openai.max_tokens, 4096);
        assert!((config.ai_ingestion.openai.temperature - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_ai_ingestion_validation() {
        let mut config = SearchConfig::default();
        config.ai_ingestion.enabled = true;
        config.ai_ingestion.examples_per_tool = 0;

        assert!(config.validate().is_err());

        config.ai_ingestion.examples_per_tool = 5;
        config.ai_ingestion.timeout_secs = 0;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ai_ingestion_get_model() {
        let mut config = AiIngestionConfig::default();

        // Default model from provider config
        config.model = String::new();
        config.provider = AiProvider::Ollama;
        assert_eq!(config.get_model(), "llama3.2");

        config.provider = AiProvider::OpenAi;
        assert_eq!(config.get_model(), "gpt-4o-mini");

        config.provider = AiProvider::Anthropic;
        assert_eq!(config.get_model(), "claude-3-haiku-20240307");

        // Override with explicit model
        config.model = "custom-model".to_string();
        assert_eq!(config.get_model(), "custom-model");
    }
}
