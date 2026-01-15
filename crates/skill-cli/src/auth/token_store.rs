//! Token store for caching and managing authentication credentials.
//!
//! Uses the system keyring for secure storage and provides:
//! - In-memory caching with TTL
//! - Automatic refresh when credentials expire
//! - Thread-safe access

use crate::auth::provider::{AuthProvider, AuthResult, Credentials};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use keyring::Entry;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const SERVICE_NAME: &str = "skill-engine-auth";

/// Cached credentials with metadata.
#[derive(Debug, Clone)]
struct CachedCredentials {
    credentials: Credentials,
    refresh_token: Option<SecretString>,
    cached_at: DateTime<Utc>,
}

/// Stored credential data in keyring.
#[derive(Debug, Serialize, Deserialize)]
struct StoredCredentials {
    credentials: Credentials,
    refresh_token_key: Option<String>,
}

/// Token store for managing authentication credentials.
pub struct TokenStore {
    /// In-memory cache of credentials
    cache: Arc<RwLock<HashMap<String, CachedCredentials>>>,
}

impl TokenStore {
    /// Create a new token store.
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a storage key for credentials.
    fn storage_key(provider_id: &str, skill: Option<&str>, instance: Option<&str>) -> String {
        match (skill, instance) {
            (Some(s), Some(i)) => format!("{}:{}:{}", provider_id, s, i),
            (Some(s), None) => format!("{}:{}:default", provider_id, s),
            (None, Some(i)) => format!("{}:global:{}", provider_id, i),
            (None, None) => format!("{}:global:default", provider_id),
        }
    }

    /// Store credentials in keyring.
    pub async fn store(
        &self,
        provider_id: &str,
        skill: Option<&str>,
        instance: Option<&str>,
        result: &AuthResult,
    ) -> Result<()> {
        let key = Self::storage_key(provider_id, skill, instance);

        // Store refresh token separately if present
        let refresh_token_key = if let Some(refresh_token) = &result.refresh_token {
            let rt_key = format!("{}_refresh", key);
            let entry = Entry::new(SERVICE_NAME, &rt_key)?;
            entry.set_password(refresh_token.expose_secret())?;
            Some(rt_key)
        } else {
            None
        };

        // Store main credentials
        let stored = StoredCredentials {
            credentials: result.credentials.clone(),
            refresh_token_key,
        };

        let json = serde_json::to_string(&stored)?;
        let entry = Entry::new(SERVICE_NAME, &key)?;
        entry.set_password(&json)?;

        // Update cache
        let cached = CachedCredentials {
            credentials: result.credentials.clone(),
            refresh_token: result.refresh_token.clone(),
            cached_at: Utc::now(),
        };

        self.cache.write().await.insert(key, cached);

        Ok(())
    }

    /// Load credentials from keyring.
    pub async fn load(
        &self,
        provider_id: &str,
        skill: Option<&str>,
        instance: Option<&str>,
    ) -> Result<Option<(Credentials, Option<SecretString>)>> {
        let key = Self::storage_key(provider_id, skill, instance);

        // Check cache first
        if let Some(cached) = self.cache.read().await.get(&key) {
            return Ok(Some((cached.credentials.clone(), cached.refresh_token.clone())));
        }

        // Load from keyring
        let entry = Entry::new(SERVICE_NAME, &key)?;
        let json = match entry.get_password() {
            Ok(json) => json,
            Err(keyring::Error::NoEntry) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let stored: StoredCredentials = serde_json::from_str(&json)
            .context("Failed to parse stored credentials")?;

        // Load refresh token if present
        let refresh_token = if let Some(rt_key) = &stored.refresh_token_key {
            let rt_entry = Entry::new(SERVICE_NAME, rt_key)?;
            match rt_entry.get_password() {
                Ok(token) => Some(SecretString::from(token)),
                Err(_) => None,
            }
        } else {
            None
        };

        // Update cache
        let cached = CachedCredentials {
            credentials: stored.credentials.clone(),
            refresh_token: refresh_token.clone(),
            cached_at: Utc::now(),
        };
        self.cache.write().await.insert(key, cached);

        Ok(Some((stored.credentials, refresh_token)))
    }

    /// Get credentials, refreshing if necessary.
    pub async fn get_credentials(
        &self,
        provider: &dyn AuthProvider,
        skill: Option<&str>,
        instance: Option<&str>,
    ) -> Result<Option<Credentials>> {
        let provider_id = provider.id();

        // Load existing credentials
        let (credentials, refresh_token) = match self.load(provider_id, skill, instance).await? {
            Some(creds) => creds,
            None => return Ok(None),
        };

        // Check if refresh is needed
        if credentials.needs_refresh() {
            if let Some(refresh_token) = refresh_token {
                // Try to refresh
                match provider.refresh(&credentials, &refresh_token).await {
                    Ok(result) => {
                        // Store refreshed credentials
                        self.store(provider_id, skill, instance, &result).await?;
                        return Ok(Some(result.credentials));
                    }
                    Err(e) => {
                        // Refresh failed - credentials may still be valid for a bit
                        if !credentials.is_expired() {
                            eprintln!("Warning: Token refresh failed: {}. Using existing token.", e);
                            return Ok(Some(credentials));
                        }
                        // Credentials are expired and refresh failed
                        return Err(anyhow!(
                            "Credentials expired and refresh failed: {}. Please run 'skill auth login {}'.",
                            e, provider_id
                        ));
                    }
                }
            } else if credentials.is_expired() {
                // No refresh token and credentials are expired
                return Err(anyhow!(
                    "Credentials expired. Please run 'skill auth login {}'.",
                    provider_id
                ));
            }
        }

        Ok(Some(credentials))
    }

    /// Delete credentials from keyring.
    pub async fn delete(
        &self,
        provider_id: &str,
        skill: Option<&str>,
        instance: Option<&str>,
    ) -> Result<()> {
        let key = Self::storage_key(provider_id, skill, instance);

        // Load to get refresh token key
        let entry = Entry::new(SERVICE_NAME, &key)?;
        if let Ok(json) = entry.get_password() {
            if let Ok(stored) = serde_json::from_str::<StoredCredentials>(&json) {
                // Delete refresh token if present
                if let Some(rt_key) = stored.refresh_token_key {
                    let rt_entry = Entry::new(SERVICE_NAME, &rt_key)?;
                    let _ = rt_entry.delete_credential();
                }
            }
        }

        // Delete main credentials
        let _ = entry.delete_credential();

        // Remove from cache
        self.cache.write().await.remove(&key);

        Ok(())
    }

    /// List all stored credentials.
    pub async fn list(&self) -> Result<Vec<CredentialInfo>> {
        // This is a simplified implementation - keyring doesn't support listing
        // In practice, we'd need to maintain a separate index of stored credentials
        // For now, return cached entries
        let cache = self.cache.read().await;
        let mut infos = Vec::new();

        for (key, cached) in cache.iter() {
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() >= 3 {
                infos.push(CredentialInfo {
                    provider_id: parts[0].to_string(),
                    skill: if parts[1] == "global" { None } else { Some(parts[1].to_string()) },
                    instance: if parts[2] == "default" { None } else { Some(parts[2].to_string()) },
                    expires_at: cached.credentials.expires_at,
                    has_refresh_token: cached.refresh_token.is_some(),
                });
            }
        }

        Ok(infos)
    }

    /// Clear the in-memory cache.
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }
}

impl Default for TokenStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about stored credentials.
#[derive(Debug, Clone)]
pub struct CredentialInfo {
    pub provider_id: String,
    pub skill: Option<String>,
    pub instance: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub has_refresh_token: bool,
}

impl CredentialInfo {
    /// Check if these credentials are expired.
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() >= expires,
            None => false,
        }
    }

    /// Get a human-readable expiry description.
    pub fn expiry_description(&self) -> String {
        match self.expires_at {
            Some(expires) => {
                let now = Utc::now();
                if expires <= now {
                    "Expired".to_string()
                } else {
                    let duration = expires - now;
                    if duration.num_days() > 0 {
                        format!("Expires in {} days", duration.num_days())
                    } else if duration.num_hours() > 0 {
                        format!("Expires in {} hours", duration.num_hours())
                    } else {
                        format!("Expires in {} minutes", duration.num_minutes())
                    }
                }
            }
            None => "Never expires".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_key_generation() {
        assert_eq!(
            TokenStore::storage_key("github", Some("my-skill"), Some("prod")),
            "github:my-skill:prod"
        );
        assert_eq!(
            TokenStore::storage_key("github", Some("my-skill"), None),
            "github:my-skill:default"
        );
        assert_eq!(
            TokenStore::storage_key("github", None, None),
            "github:global:default"
        );
    }
}
