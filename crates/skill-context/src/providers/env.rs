//! Environment variable secret provider.
//!
//! This provider reads secrets from environment variables, which is useful
//! for CI/CD environments where secrets are injected as environment variables.
//!
//! This provider is **read-only** - secrets can only be read, not written or deleted.

use async_trait::async_trait;
use zeroize::Zeroizing;

use super::{SecretProvider, SecretValue};
use crate::ContextError;

/// Secret provider that reads from environment variables.
///
/// Secrets are looked up using the pattern: `{PREFIX}{CONTEXT}_{KEY}`
/// where context and key are converted to uppercase with `-` replaced by `_`.
///
/// # Example
///
/// With prefix `SECRET_` and context `my-context`, key `api-key`:
/// - Environment variable: `SECRET_MY_CONTEXT_API_KEY`
pub struct EnvironmentProvider {
    /// Prefix for environment variable names.
    prefix: String,
}

impl EnvironmentProvider {
    /// Create a new environment provider with the given prefix.
    ///
    /// # Example
    ///
    /// ```rust
    /// use skill_context::providers::EnvironmentProvider;
    ///
    /// let provider = EnvironmentProvider::new("SECRET_");
    /// ```
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }

    /// Create a provider with no prefix.
    pub fn without_prefix() -> Self {
        Self {
            prefix: String::new(),
        }
    }

    /// Build the environment variable name for a secret.
    fn build_env_var(&self, context_id: &str, key: &str) -> String {
        let context_part = context_id.to_uppercase().replace(['-', '.'], "_");
        let key_part = key.to_uppercase().replace(['-', '.'], "_");
        format!("{}{}__{}", self.prefix, context_part, key_part)
    }
}

#[async_trait]
impl SecretProvider for EnvironmentProvider {
    async fn get_secret(
        &self,
        context_id: &str,
        key: &str,
    ) -> Result<Option<SecretValue>, ContextError> {
        let env_var = self.build_env_var(context_id, key);

        match std::env::var(&env_var) {
            Ok(value) => {
                tracing::debug!(
                    context_id = context_id,
                    key = key,
                    env_var = env_var,
                    "Retrieved secret from environment"
                );
                Ok(Some(Zeroizing::new(value)))
            }
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(std::env::VarError::NotUnicode(_)) => {
                tracing::warn!(
                    context_id = context_id,
                    key = key,
                    env_var = env_var,
                    "Environment variable contains invalid UTF-8"
                );
                Err(ContextError::SecretProvider(format!(
                    "Environment variable '{}' contains invalid UTF-8",
                    env_var
                )))
            }
        }
    }

    async fn set_secret(
        &self,
        context_id: &str,
        key: &str,
        _value: &str,
    ) -> Result<(), ContextError> {
        let env_var = self.build_env_var(context_id, key);
        tracing::warn!(
            context_id = context_id,
            key = key,
            env_var = env_var,
            "Attempted to set secret via environment provider (read-only)"
        );
        Err(ContextError::SecretProvider(
            "Environment provider is read-only. Cannot set secrets.".to_string(),
        ))
    }

    async fn delete_secret(&self, context_id: &str, key: &str) -> Result<(), ContextError> {
        let env_var = self.build_env_var(context_id, key);
        tracing::warn!(
            context_id = context_id,
            key = key,
            env_var = env_var,
            "Attempted to delete secret via environment provider (read-only)"
        );
        Err(ContextError::SecretProvider(
            "Environment provider is read-only. Cannot delete secrets.".to_string(),
        ))
    }

    async fn list_keys(&self, context_id: &str) -> Result<Vec<String>, ContextError> {
        let context_prefix = format!(
            "{}{}__",
            self.prefix,
            context_id.to_uppercase().replace(['-', '.'], "_")
        );

        let keys: Vec<String> = std::env::vars()
            .filter_map(|(k, _)| {
                if k.starts_with(&context_prefix) {
                    Some(
                        k[context_prefix.len()..]
                            .to_lowercase()
                            .replace('_', "-"),
                    )
                } else {
                    None
                }
            })
            .collect();

        Ok(keys)
    }

    fn name(&self) -> &'static str {
        "environment"
    }

    fn is_read_only(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_naming() {
        let provider = EnvironmentProvider::new("SECRET_");

        assert_eq!(
            provider.build_env_var("my-context", "api-key"),
            "SECRET_MY_CONTEXT__API_KEY"
        );

        assert_eq!(
            provider.build_env_var("production.api", "database.password"),
            "SECRET_PRODUCTION_API__DATABASE_PASSWORD"
        );
    }

    #[test]
    fn test_no_prefix() {
        let provider = EnvironmentProvider::without_prefix();

        assert_eq!(
            provider.build_env_var("context", "key"),
            "CONTEXT__KEY"
        );
    }

    #[tokio::test]
    async fn test_get_from_env() {
        let provider = EnvironmentProvider::new("TEST_SECRET_");

        // Set env var for test
        std::env::set_var("TEST_SECRET_MY_CTX__MY_KEY", "my-secret-value");

        let result = provider.get_secret("my-ctx", "my-key").await.unwrap();
        assert!(result.is_some());
        assert_eq!(&*result.unwrap(), "my-secret-value");

        // Clean up
        std::env::remove_var("TEST_SECRET_MY_CTX__MY_KEY");
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let provider = EnvironmentProvider::new("NONEXISTENT_PREFIX_");

        let result = provider
            .get_secret("context", "key")
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_set_is_read_only() {
        let provider = EnvironmentProvider::new("TEST_");

        let result = provider.set_secret("ctx", "key", "value").await;

        assert!(result.is_err());
        assert!(provider.is_read_only());
    }

    #[tokio::test]
    async fn test_delete_is_read_only() {
        let provider = EnvironmentProvider::new("TEST_");

        let result = provider.delete_secret("ctx", "key").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_keys() {
        let provider = EnvironmentProvider::new("TEST_LIST_");

        // Set some env vars
        std::env::set_var("TEST_LIST_MY_CTX__KEY_ONE", "value1");
        std::env::set_var("TEST_LIST_MY_CTX__KEY_TWO", "value2");
        std::env::set_var("TEST_LIST_OTHER_CTX__KEY", "value3");

        let keys = provider.list_keys("my-ctx").await.unwrap();

        assert!(keys.contains(&"key-one".to_string()));
        assert!(keys.contains(&"key-two".to_string()));
        assert!(!keys.contains(&"key".to_string())); // Wrong context

        // Clean up
        std::env::remove_var("TEST_LIST_MY_CTX__KEY_ONE");
        std::env::remove_var("TEST_LIST_MY_CTX__KEY_TWO");
        std::env::remove_var("TEST_LIST_OTHER_CTX__KEY");
    }
}
