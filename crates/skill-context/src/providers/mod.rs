//! Secret provider implementations.
//!
//! This module provides a pluggable secret provider system with multiple
//! backends for storing and retrieving secrets.
//!
//! # Available Providers
//!
//! - [`KeychainProvider`]: Platform-native keychain (default)
//! - [`EnvironmentProvider`]: Environment variables (useful for CI/CD)
//! - [`FileProvider`]: File-based secrets
//!
//! # Example
//!
//! ```rust,no_run
//! use skill_context::providers::{SecretProvider, KeychainProvider};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = KeychainProvider::new();
//!
//! // Store a secret
//! provider.set_secret("my-context", "api-key", "secret-value").await?;
//!
//! // Retrieve a secret
//! if let Some(secret) = provider.get_secret("my-context", "api-key").await? {
//!     println!("Got secret (length: {})", secret.len());
//! }
//! # Ok(())
//! # }
//! ```

pub mod env;
pub mod file;
pub mod keychain;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::RwLock;
use zeroize::Zeroizing;

use crate::secrets::{SecretDefinition, SecretProviderConfig};
use crate::ContextError;

pub use env::EnvironmentProvider;
pub use file::FileProvider;
pub use keychain::KeychainProvider;

/// A secret value that is automatically zeroed when dropped.
pub type SecretValue = Zeroizing<String>;

/// Trait for secret providers.
///
/// All secret values are wrapped in `Zeroizing<String>` to ensure
/// they are cleared from memory when no longer needed.
#[async_trait]
pub trait SecretProvider: Send + Sync {
    /// Get a secret value.
    ///
    /// Returns `None` if the secret doesn't exist.
    async fn get_secret(
        &self,
        context_id: &str,
        key: &str,
    ) -> Result<Option<SecretValue>, ContextError>;

    /// Set a secret value.
    async fn set_secret(
        &self,
        context_id: &str,
        key: &str,
        value: &str,
    ) -> Result<(), ContextError>;

    /// Delete a secret.
    async fn delete_secret(&self, context_id: &str, key: &str) -> Result<(), ContextError>;

    /// Check if a secret exists.
    async fn has_secret(&self, context_id: &str, key: &str) -> Result<bool, ContextError> {
        Ok(self.get_secret(context_id, key).await?.is_some())
    }

    /// List all secret keys for a context.
    async fn list_keys(&self, context_id: &str) -> Result<Vec<String>, ContextError>;

    /// Get the provider name.
    fn name(&self) -> &'static str;

    /// Check if this provider is read-only.
    fn is_read_only(&self) -> bool {
        false
    }
}

/// Manager for routing secret operations to the appropriate provider.
pub struct SecretManager {
    /// Available providers by name.
    providers: HashMap<String, Arc<dyn SecretProvider>>,
    /// Default provider name.
    default_provider: String,
    /// Secret cache.
    cache: Arc<RwLock<SecretCache>>,
    /// Cache TTL.
    cache_ttl: Duration,
}

impl SecretManager {
    /// Create a new secret manager with the keychain as the default provider.
    pub fn new() -> Self {
        let mut providers: HashMap<String, Arc<dyn SecretProvider>> = HashMap::new();
        providers.insert("keychain".to_string(), Arc::new(KeychainProvider::new()));

        Self {
            providers,
            default_provider: "keychain".to_string(),
            cache: Arc::new(RwLock::new(SecretCache::new())),
            cache_ttl: Duration::from_secs(300), // 5 minutes default
        }
    }

    /// Add a provider.
    pub fn with_provider(mut self, name: impl Into<String>, provider: Arc<dyn SecretProvider>) -> Self {
        self.providers.insert(name.into(), provider);
        self
    }

    /// Set the default provider.
    pub fn with_default_provider(mut self, name: impl Into<String>) -> Self {
        self.default_provider = name.into();
        self
    }

    /// Set the cache TTL.
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Disable caching.
    pub fn without_cache(mut self) -> Self {
        self.cache_ttl = Duration::ZERO;
        self
    }

    /// Add providers from configuration.
    pub fn with_provider_configs(mut self, configs: &[SecretProviderConfig]) -> Self {
        for config in configs {
            match config {
                SecretProviderConfig::Keychain => {
                    self.providers
                        .insert("keychain".to_string(), Arc::new(KeychainProvider::new()));
                }
                SecretProviderConfig::EnvironmentVariable { prefix } => {
                    self.providers.insert(
                        "environment".to_string(),
                        Arc::new(EnvironmentProvider::new(prefix)),
                    );
                }
                SecretProviderConfig::File { path, format } => {
                    if let Ok(provider) = FileProvider::new(path, format.clone()) {
                        self.providers
                            .insert("file".to_string(), Arc::new(provider));
                    }
                }
                SecretProviderConfig::External { .. } => {
                    // External providers require feature flags
                    tracing::warn!("External secret providers not yet implemented");
                }
            }
        }
        self
    }

    /// Get a secret using the appropriate provider.
    pub async fn get_secret(
        &self,
        context_id: &str,
        definition: &SecretDefinition,
    ) -> Result<Option<SecretValue>, ContextError> {
        let cache_key = format!("{}:{}", context_id, definition.key);

        // Check cache first
        if !self.cache_ttl.is_zero() {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key, self.cache_ttl) {
                tracing::debug!(
                    context_id = context_id,
                    key = definition.key,
                    "Secret cache hit"
                );
                return Ok(Some(cached));
            }
        }

        // Get from provider
        let provider_name = definition
            .provider
            .as_deref()
            .unwrap_or(&self.default_provider);

        let provider = self
            .providers
            .get(provider_name)
            .ok_or_else(|| ContextError::SecretProvider(format!(
                "Provider '{}' not configured",
                provider_name
            )))?;

        tracing::debug!(
            context_id = context_id,
            key = definition.key,
            provider = provider_name,
            "Fetching secret from provider"
        );

        let secret = provider.get_secret(context_id, &definition.key).await?;

        // Update cache
        if !self.cache_ttl.is_zero() {
            if let Some(ref value) = secret {
                let mut cache = self.cache.write().await;
                cache.set(cache_key, value.clone());
            }
        }

        Ok(secret)
    }

    /// Set a secret using the appropriate provider.
    pub async fn set_secret(
        &self,
        context_id: &str,
        definition: &SecretDefinition,
        value: &str,
    ) -> Result<(), ContextError> {
        let provider_name = definition
            .provider
            .as_deref()
            .unwrap_or(&self.default_provider);

        let provider = self
            .providers
            .get(provider_name)
            .ok_or_else(|| ContextError::SecretProvider(format!(
                "Provider '{}' not configured",
                provider_name
            )))?;

        if provider.is_read_only() {
            return Err(ContextError::SecretProvider(format!(
                "Provider '{}' is read-only",
                provider_name
            )));
        }

        tracing::info!(
            context_id = context_id,
            key = definition.key,
            provider = provider_name,
            "Setting secret"
        );

        provider.set_secret(context_id, &definition.key, value).await?;

        // Invalidate cache
        if !self.cache_ttl.is_zero() {
            let cache_key = format!("{}:{}", context_id, definition.key);
            let mut cache = self.cache.write().await;
            cache.invalidate(&cache_key);
        }

        Ok(())
    }

    /// Delete a secret using the appropriate provider.
    pub async fn delete_secret(
        &self,
        context_id: &str,
        definition: &SecretDefinition,
    ) -> Result<(), ContextError> {
        let provider_name = definition
            .provider
            .as_deref()
            .unwrap_or(&self.default_provider);

        let provider = self
            .providers
            .get(provider_name)
            .ok_or_else(|| ContextError::SecretProvider(format!(
                "Provider '{}' not configured",
                provider_name
            )))?;

        if provider.is_read_only() {
            return Err(ContextError::SecretProvider(format!(
                "Provider '{}' is read-only",
                provider_name
            )));
        }

        tracing::info!(
            context_id = context_id,
            key = definition.key,
            provider = provider_name,
            "Deleting secret"
        );

        provider.delete_secret(context_id, &definition.key).await?;

        // Invalidate cache
        if !self.cache_ttl.is_zero() {
            let cache_key = format!("{}:{}", context_id, definition.key);
            let mut cache = self.cache.write().await;
            cache.invalidate(&cache_key);
        }

        Ok(())
    }

    /// Check if all required secrets for a context are set.
    pub async fn verify_secrets(
        &self,
        context_id: &str,
        definitions: &[(&str, &SecretDefinition)],
    ) -> Result<Vec<String>, ContextError> {
        let mut missing = Vec::new();

        for (key, def) in definitions {
            if def.required {
                let has = self.get_secret(context_id, def).await?.is_some();
                if !has {
                    missing.push(key.to_string());
                }
            }
        }

        Ok(missing)
    }

    /// Clear the secret cache.
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

impl Default for SecretManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal cache for secret values.
struct SecretCache {
    entries: HashMap<String, CacheEntry>,
}

struct CacheEntry {
    value: SecretValue,
    cached_at: Instant,
}

impl SecretCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn get(&self, key: &str, ttl: Duration) -> Option<SecretValue> {
        self.entries.get(key).and_then(|entry| {
            if entry.cached_at.elapsed() < ttl {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    fn set(&mut self, key: String, value: SecretValue) {
        self.entries.insert(
            key,
            CacheEntry {
                value,
                cached_at: Instant::now(),
            },
        );
    }

    fn invalidate(&mut self, key: &str) {
        self.entries.remove(key);
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_secret_manager_default() {
        let manager = SecretManager::new();
        assert!(manager.providers.contains_key("keychain"));
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let mut cache = SecretCache::new();

        cache.set("key1".to_string(), Zeroizing::new("value1".to_string()));

        // Should hit cache
        let result = cache.get("key1", Duration::from_secs(60));
        assert!(result.is_some());
        assert_eq!(&*result.unwrap(), "value1");

        // Should miss for nonexistent key
        let result = cache.get("key2", Duration::from_secs(60));
        assert!(result.is_none());

        // Invalidate
        cache.invalidate("key1");
        let result = cache.get("key1", Duration::from_secs(60));
        assert!(result.is_none());
    }
}
