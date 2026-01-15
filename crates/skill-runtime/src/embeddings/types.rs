//! Types for embedding configuration and providers

use serde::{Deserialize, Serialize};

/// Configuration for embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Provider type: "fastembed", "openai", "ollama"
    pub provider: EmbeddingProviderType,

    /// Model name/identifier (provider-specific)
    #[serde(default)]
    pub model: Option<String>,

    /// API key (for cloud providers)
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,

    /// Base URL (for self-hosted or custom endpoints)
    #[serde(default)]
    pub base_url: Option<String>,

    /// Maximum batch size for document embedding
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_batch_size() -> usize {
    100
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: EmbeddingProviderType::FastEmbed,
            model: None,
            api_key: None,
            base_url: None,
            batch_size: default_batch_size(),
        }
    }
}

impl EmbeddingConfig {
    /// Create a FastEmbed configuration
    pub fn fastembed() -> Self {
        Self {
            provider: EmbeddingProviderType::FastEmbed,
            model: Some(FastEmbedModel::AllMiniLM.to_string()),
            ..Default::default()
        }
    }

    /// Create a FastEmbed configuration with a specific model
    pub fn fastembed_with_model(model: FastEmbedModel) -> Self {
        Self {
            provider: EmbeddingProviderType::FastEmbed,
            model: Some(model.to_string()),
            ..Default::default()
        }
    }

    /// Create an OpenAI configuration
    pub fn openai() -> Self {
        Self {
            provider: EmbeddingProviderType::OpenAI,
            model: Some(OpenAIEmbeddingModel::Ada002.to_string()),
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            ..Default::default()
        }
    }

    /// Create an OpenAI configuration with a specific model
    pub fn openai_with_model(model: OpenAIEmbeddingModel) -> Self {
        Self {
            provider: EmbeddingProviderType::OpenAI,
            model: Some(model.to_string()),
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            ..Default::default()
        }
    }

    /// Create an Ollama configuration
    pub fn ollama() -> Self {
        Self {
            provider: EmbeddingProviderType::Ollama,
            model: Some("nomic-embed-text".to_string()),
            base_url: Some("http://localhost:11434".to_string()),
            ..Default::default()
        }
    }

    /// Create an Ollama configuration with a specific model
    pub fn ollama_with_model(model: &str) -> Self {
        Self {
            provider: EmbeddingProviderType::Ollama,
            model: Some(model.to_string()),
            base_url: Some("http://localhost:11434".to_string()),
            ..Default::default()
        }
    }

    /// Set the API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set the batch size
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// Set the model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

/// Supported embedding provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingProviderType {
    /// Local FastEmbed (ONNX-based)
    #[default]
    FastEmbed,

    /// OpenAI API
    OpenAI,

    /// Ollama local server
    Ollama,
}

impl std::fmt::Display for EmbeddingProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FastEmbed => write!(f, "fastembed"),
            Self::OpenAI => write!(f, "openai"),
            Self::Ollama => write!(f, "ollama"),
        }
    }
}

impl std::str::FromStr for EmbeddingProviderType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fastembed" | "fast_embed" | "fast-embed" => Ok(Self::FastEmbed),
            "openai" | "open_ai" | "open-ai" => Ok(Self::OpenAI),
            "ollama" => Ok(Self::Ollama),
            _ => Err(anyhow::anyhow!(
                "Unknown embedding provider: {}. Supported: fastembed, openai, ollama",
                s
            )),
        }
    }
}

/// FastEmbed model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FastEmbedModel {
    /// all-MiniLM-L6-v2 (Quantized) - 384 dimensions, fastest
    #[default]
    AllMiniLM,

    /// BGE-small-en-v1.5 (Quantized) - 384 dimensions, good quality
    BGESmallEN,

    /// BGE-base-en-v1.5 - 768 dimensions, better quality
    BGEBaseEN,

    /// BGE-large-en-v1.5 - 1024 dimensions, best quality
    BGELargeEN,
}

impl FastEmbedModel {
    /// Get the embedding dimensions for this model
    pub fn dimensions(&self) -> usize {
        match self {
            Self::AllMiniLM => 384,
            Self::BGESmallEN => 384,
            Self::BGEBaseEN => 768,
            Self::BGELargeEN => 1024,
        }
    }

    /// Get the model name as used by rig-fastembed
    pub fn rig_model_name(&self) -> &'static str {
        match self {
            Self::AllMiniLM => "all-minilm",
            Self::BGESmallEN => "bge-small",
            Self::BGEBaseEN => "bge-base",
            Self::BGELargeEN => "bge-large",
        }
    }
}

impl std::fmt::Display for FastEmbedModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllMiniLM => write!(f, "all-minilm"),
            Self::BGESmallEN => write!(f, "bge-small"),
            Self::BGEBaseEN => write!(f, "bge-base"),
            Self::BGELargeEN => write!(f, "bge-large"),
        }
    }
}

impl std::str::FromStr for FastEmbedModel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all-minilm" | "allminilm" | "minilm" => Ok(Self::AllMiniLM),
            "bge-small" | "bgesmall" | "bge-small-en" => Ok(Self::BGESmallEN),
            "bge-base" | "bgebase" | "bge-base-en" => Ok(Self::BGEBaseEN),
            "bge-large" | "bgelarge" | "bge-large-en" => Ok(Self::BGELargeEN),
            _ => Err(anyhow::anyhow!(
                "Unknown FastEmbed model: {}. Supported: all-minilm, bge-small, bge-base, bge-large",
                s
            )),
        }
    }
}

/// OpenAI embedding model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OpenAIEmbeddingModel {
    /// text-embedding-ada-002 - 1536 dimensions (legacy, widely supported)
    #[default]
    Ada002,

    /// text-embedding-3-small - 1536 dimensions (newer, better)
    TextEmbedding3Small,

    /// text-embedding-3-large - 3072 dimensions (best quality)
    TextEmbedding3Large,
}

impl OpenAIEmbeddingModel {
    /// Get the embedding dimensions for this model
    pub fn dimensions(&self) -> usize {
        match self {
            Self::Ada002 => 1536,
            Self::TextEmbedding3Small => 1536,
            Self::TextEmbedding3Large => 3072,
        }
    }

    /// Get the model name as used by OpenAI API
    pub fn api_name(&self) -> &'static str {
        match self {
            Self::Ada002 => "text-embedding-ada-002",
            Self::TextEmbedding3Small => "text-embedding-3-small",
            Self::TextEmbedding3Large => "text-embedding-3-large",
        }
    }
}

impl std::fmt::Display for OpenAIEmbeddingModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.api_name())
    }
}

impl std::str::FromStr for OpenAIEmbeddingModel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ada-002" | "text-embedding-ada-002" | "ada" => Ok(Self::Ada002),
            "3-small" | "text-embedding-3-small" | "embedding-3-small" => {
                Ok(Self::TextEmbedding3Small)
            }
            "3-large" | "text-embedding-3-large" | "embedding-3-large" => {
                Ok(Self::TextEmbedding3Large)
            }
            _ => Err(anyhow::anyhow!(
                "Unknown OpenAI embedding model: {}. Supported: ada-002, 3-small, 3-large",
                s
            )),
        }
    }
}

/// Embedding result with metadata
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    /// The embedding vector
    pub embedding: Vec<f32>,

    /// Token count used (if available)
    pub tokens_used: Option<usize>,

    /// Model used for embedding
    pub model: String,
}

impl EmbeddingResult {
    pub fn new(embedding: Vec<f32>, model: impl Into<String>) -> Self {
        Self {
            embedding,
            tokens_used: None,
            model: model.into(),
        }
    }

    pub fn with_tokens(mut self, tokens: usize) -> Self {
        self.tokens_used = Some(tokens);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fastembed_model_dimensions() {
        assert_eq!(FastEmbedModel::AllMiniLM.dimensions(), 384);
        assert_eq!(FastEmbedModel::BGESmallEN.dimensions(), 384);
        assert_eq!(FastEmbedModel::BGEBaseEN.dimensions(), 768);
        assert_eq!(FastEmbedModel::BGELargeEN.dimensions(), 1024);
    }

    #[test]
    fn test_openai_model_dimensions() {
        assert_eq!(OpenAIEmbeddingModel::Ada002.dimensions(), 1536);
        assert_eq!(OpenAIEmbeddingModel::TextEmbedding3Small.dimensions(), 1536);
        assert_eq!(OpenAIEmbeddingModel::TextEmbedding3Large.dimensions(), 3072);
    }

    #[test]
    fn test_provider_type_parsing() {
        assert_eq!(
            "fastembed".parse::<EmbeddingProviderType>().unwrap(),
            EmbeddingProviderType::FastEmbed
        );
        assert_eq!(
            "openai".parse::<EmbeddingProviderType>().unwrap(),
            EmbeddingProviderType::OpenAI
        );
        assert_eq!(
            "ollama".parse::<EmbeddingProviderType>().unwrap(),
            EmbeddingProviderType::Ollama
        );
    }

    #[test]
    fn test_fastembed_model_parsing() {
        assert_eq!(
            "all-minilm".parse::<FastEmbedModel>().unwrap(),
            FastEmbedModel::AllMiniLM
        );
        assert_eq!(
            "bge-small".parse::<FastEmbedModel>().unwrap(),
            FastEmbedModel::BGESmallEN
        );
    }

    #[test]
    fn test_embedding_config_builders() {
        let config = EmbeddingConfig::fastembed();
        assert_eq!(config.provider, EmbeddingProviderType::FastEmbed);

        let config = EmbeddingConfig::openai_with_model(OpenAIEmbeddingModel::TextEmbedding3Large);
        assert_eq!(config.provider, EmbeddingProviderType::OpenAI);
        assert_eq!(config.model, Some("text-embedding-3-large".to_string()));

        let config = EmbeddingConfig::ollama().with_base_url("http://custom:11434");
        assert_eq!(config.base_url, Some("http://custom:11434".to_string()));
    }
}
