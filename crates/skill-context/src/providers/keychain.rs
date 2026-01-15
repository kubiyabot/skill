//! Platform keychain secret provider.
//!
//! Uses the system keychain for secure secret storage:
//! - macOS: Keychain
//! - Windows: Credential Manager
//! - Linux: Secret Service (via DBus)

use async_trait::async_trait;
use keyring::Entry;
use zeroize::Zeroizing;

use super::{SecretProvider, SecretValue};
use crate::ContextError;

/// Service name used for keyring entries.
const SERVICE_NAME: &str = "skill-engine-context";

/// Secret provider that uses the platform keychain.
pub struct KeychainProvider {
    /// Optional prefix for all keys.
    prefix: Option<String>,
}

impl KeychainProvider {
    /// Create a new keychain provider.
    pub fn new() -> Self {
        Self { prefix: None }
    }

    /// Create a new keychain provider with a key prefix.
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: Some(prefix.into()),
        }
    }

    /// Build the keyring user/key identifier.
    fn build_key(&self, context_id: &str, key: &str) -> String {
        match &self.prefix {
            Some(p) => format!("{}/{}/{}/{}", p, SERVICE_NAME, context_id, key),
            None => format!("{}/{}/{}", SERVICE_NAME, context_id, key),
        }
    }

    /// Get a keyring entry.
    fn get_entry(&self, context_id: &str, key: &str) -> Result<Entry, ContextError> {
        let user = self.build_key(context_id, key);
        Entry::new(SERVICE_NAME, &user).map_err(|e| {
            ContextError::SecretProvider(format!("Failed to create keyring entry: {}", e))
        })
    }
}

impl Default for KeychainProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretProvider for KeychainProvider {
    async fn get_secret(
        &self,
        context_id: &str,
        key: &str,
    ) -> Result<Option<SecretValue>, ContextError> {
        let entry = self.get_entry(context_id, key)?;

        match entry.get_password() {
            Ok(password) => {
                tracing::debug!(
                    context_id = context_id,
                    key = key,
                    "Retrieved secret from keychain"
                );
                Ok(Some(Zeroizing::new(password)))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => {
                tracing::warn!(
                    context_id = context_id,
                    key = key,
                    error = %e,
                    "Failed to get secret from keychain"
                );
                Err(ContextError::SecretProvider(format!(
                    "Failed to get secret '{}' from keychain: {}",
                    key, e
                )))
            }
        }
    }

    async fn set_secret(
        &self,
        context_id: &str,
        key: &str,
        value: &str,
    ) -> Result<(), ContextError> {
        let entry = self.get_entry(context_id, key)?;

        entry.set_password(value).map_err(|e| {
            tracing::error!(
                context_id = context_id,
                key = key,
                error = %e,
                "Failed to set secret in keychain"
            );
            ContextError::SecretProvider(format!(
                "Failed to set secret '{}' in keychain: {}",
                key, e
            ))
        })?;

        tracing::info!(
            context_id = context_id,
            key = key,
            "Stored secret in keychain"
        );

        Ok(())
    }

    async fn delete_secret(&self, context_id: &str, key: &str) -> Result<(), ContextError> {
        let entry = self.get_entry(context_id, key)?;

        match entry.delete_credential() {
            Ok(()) => {
                tracing::info!(
                    context_id = context_id,
                    key = key,
                    "Deleted secret from keychain"
                );
                Ok(())
            }
            Err(keyring::Error::NoEntry) => {
                // Already doesn't exist, that's fine
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    context_id = context_id,
                    key = key,
                    error = %e,
                    "Failed to delete secret from keychain"
                );
                Err(ContextError::SecretProvider(format!(
                    "Failed to delete secret '{}' from keychain: {}",
                    key, e
                )))
            }
        }
    }

    async fn list_keys(&self, _context_id: &str) -> Result<Vec<String>, ContextError> {
        // The keyring crate doesn't support listing keys
        // This would require platform-specific implementations
        tracing::warn!("Listing keys is not supported by the keychain provider");
        Ok(Vec::new())
    }

    fn name(&self) -> &'static str {
        "keychain"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests interact with the real system keychain
    // They're marked as ignored by default to avoid polluting the keychain

    #[tokio::test]
    #[ignore = "interacts with system keychain"]
    async fn test_keychain_set_get_delete() {
        let provider = KeychainProvider::new();
        let context_id = "test-context";
        let key = "test-secret-key";
        let value = "super-secret-value";

        // Set
        provider.set_secret(context_id, key, value).await.unwrap();

        // Get
        let retrieved = provider.get_secret(context_id, key).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(&*retrieved.unwrap(), value);

        // Delete
        provider.delete_secret(context_id, key).await.unwrap();

        // Verify deleted
        let retrieved = provider.get_secret(context_id, key).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_keychain_get_nonexistent() {
        let provider = KeychainProvider::new();
        let result = provider
            .get_secret("nonexistent-context", "nonexistent-key")
            .await;

        // Should return Ok(None), not an error
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_key_building() {
        let provider = KeychainProvider::new();
        let key = provider.build_key("my-context", "api-key");
        assert_eq!(key, "skill-engine-context/my-context/api-key");

        let provider_with_prefix = KeychainProvider::with_prefix("custom");
        let key = provider_with_prefix.build_key("my-context", "api-key");
        assert_eq!(key, "custom/skill-engine-context/my-context/api-key");
    }
}
