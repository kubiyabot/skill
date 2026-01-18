//! Embedding provider factory
//!
//! Creates embedding providers from configuration.

use super::{
    EmbeddingConfig, EmbeddingProvider, EmbeddingProviderType,
    FastEmbedModel, FastEmbedProvider,
    OpenAIEmbedProvider, OpenAIEmbeddingModel,
    OllamaProvider,
};
use anyhow::{Context, Result};
use std::sync::Arc;

/// Factory for creating embedding providers from configuration
pub struct EmbeddingProviderFactory;

impl EmbeddingProviderFactory {
    /// Create an embedding provider from configuration
    pub fn create(config: &EmbeddingConfig) -> Result<Arc<dyn EmbeddingProvider>> {
        match config.provider {
            EmbeddingProviderType::FastEmbed => {
                let model = config
                    .model
                    .as_ref()
                    .filter(|m| !m.trim().is_empty()) // Filter out empty/whitespace strings
                    .map(|m| m.parse::<FastEmbedModel>())
                    .transpose()
                    .context("Invalid FastEmbed model")?
                    .unwrap_or_default();

                let provider = FastEmbedProvider::with_model(model)?;
                Ok(Arc::new(provider))
            }

            EmbeddingProviderType::OpenAI => {
                let model = config
                    .model
                    .as_ref()
                    .filter(|m| !m.trim().is_empty()) // Filter out empty/whitespace strings
                    .map(|m| m.parse::<OpenAIEmbeddingModel>())
                    .transpose()
                    .context("Invalid OpenAI model")?
                    .unwrap_or_default();

                let provider = if let Some(ref api_key) = config.api_key {
                    OpenAIEmbedProvider::with_api_key(api_key, model)?
                } else {
                    OpenAIEmbedProvider::with_model(model)?
                };

                Ok(Arc::new(provider))
            }

            EmbeddingProviderType::Ollama => {
                let model = config
                    .model
                    .as_deref()
                    .filter(|m| !m.trim().is_empty()) // Filter out empty/whitespace strings
                    .unwrap_or(super::ollama::DEFAULT_OLLAMA_MODEL);

                let provider = if let Some(ref base_url) = config.base_url {
                    OllamaProvider::with_url(base_url, model)?
                } else {
                    OllamaProvider::with_model(model)?
                };

                Ok(Arc::new(provider))
            }
        }
    }

    /// Create a default provider (FastEmbed with AllMiniLM)
    pub fn default_provider() -> Result<Arc<dyn EmbeddingProvider>> {
        Self::create(&EmbeddingConfig::default())
    }

    /// Create a FastEmbed provider with default model
    pub fn fastembed() -> Result<Arc<dyn EmbeddingProvider>> {
        Ok(Arc::new(FastEmbedProvider::new()?))
    }

    /// Create an OpenAI provider with default model
    pub fn openai() -> Result<Arc<dyn EmbeddingProvider>> {
        Ok(Arc::new(OpenAIEmbedProvider::new()?))
    }

    /// Create an Ollama provider with default model
    pub fn ollama() -> Result<Arc<dyn EmbeddingProvider>> {
        Ok(Arc::new(OllamaProvider::new()?))
    }
}

/// Convenience function to create a provider from configuration
pub fn create_provider(config: &EmbeddingConfig) -> Result<Arc<dyn EmbeddingProvider>> {
    EmbeddingProviderFactory::create(config)
}

/// Convenience function to create a provider from provider type string
#[allow(dead_code)]
pub fn create_provider_from_type(
    provider_type: &str,
    model: Option<&str>,
) -> Result<Arc<dyn EmbeddingProvider>> {
    let provider_type: EmbeddingProviderType = provider_type.parse()?;

    let config = EmbeddingConfig {
        provider: provider_type,
        model: model.map(String::from),
        ..Default::default()
    };

    create_provider(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_fastembed() {
        let config = EmbeddingConfig::fastembed();
        let provider = EmbeddingProviderFactory::create(&config).unwrap();
        assert_eq!(provider.provider_name(), "fastembed");
        assert_eq!(provider.dimensions(), 384);
    }

    #[test]
    fn test_create_fastembed_with_model() {
        let config = EmbeddingConfig::fastembed_with_model(FastEmbedModel::BGEBaseEN);
        let provider = EmbeddingProviderFactory::create(&config).unwrap();
        assert_eq!(provider.dimensions(), 768);
    }

    #[test]
    fn test_create_ollama() {
        let config = EmbeddingConfig::ollama();
        let provider = EmbeddingProviderFactory::create(&config).unwrap();
        assert_eq!(provider.provider_name(), "ollama");
        assert_eq!(provider.model_name(), "nomic-embed-text");
    }

    #[test]
    fn test_create_from_type_string() {
        let provider = create_provider_from_type("fastembed", Some("bge-small")).unwrap();
        assert_eq!(provider.provider_name(), "fastembed");
        assert_eq!(provider.dimensions(), 384);
    }

    #[test]
    fn test_default_provider() {
        let provider = EmbeddingProviderFactory::default_provider().unwrap();
        assert_eq!(provider.provider_name(), "fastembed");
    }

    // OpenAI tests require API key, so we just test error handling
    #[test]
    fn test_openai_requires_api_key() {
        // Save and clear API key
        let original = std::env::var("OPENAI_API_KEY").ok();
        std::env::remove_var("OPENAI_API_KEY");

        let config = EmbeddingConfig::openai();
        let result = EmbeddingProviderFactory::create(&config);
        assert!(result.is_err());

        // Restore
        if let Some(key) = original {
            std::env::set_var("OPENAI_API_KEY", key);
        }
    }
}
