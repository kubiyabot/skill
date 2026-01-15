//! CLI commands for authentication management.
//!
//! Provides `skill auth login`, `skill auth status`, `skill auth logout`, and `skill auth providers`.

use crate::auth::provider::{AuthProvider, AuthStatus, AuthType};
use crate::auth::providers::{ApiKeyProvider, AwsProvider, OAuth2Provider};
use crate::auth::token_store::TokenStore;
use anyhow::{anyhow, bail, Result};
use chrono::Utc;
use colored::Colorize;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry of available authentication providers.
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn AuthProvider>>,
}

impl ProviderRegistry {
    /// Create a new registry with default providers.
    pub fn new() -> Self {
        let mut registry = Self {
            providers: HashMap::new(),
        };

        // Register default providers
        registry.register_defaults();

        registry
    }

    /// Register default providers.
    fn register_defaults(&mut self) {
        // GitHub OAuth2 (requires client_id from env or config)
        if let Ok(client_id) = std::env::var("GITHUB_OAUTH_CLIENT_ID") {
            if let Ok(provider) = crate::auth::providers::oauth2::github_provider(client_id) {
                self.providers.insert("github".to_string(), Arc::new(provider));
            }
        }

        // Google OAuth2 (requires client_id from env or config)
        if let Ok(client_id) = std::env::var("GOOGLE_OAUTH_CLIENT_ID") {
            let client_secret = std::env::var("GOOGLE_OAUTH_CLIENT_SECRET").ok();
            if let Ok(provider) = crate::auth::providers::oauth2::google_provider(client_id, client_secret) {
                self.providers.insert("google".to_string(), Arc::new(provider));
            }
        }

        // AWS (always available)
        if let Ok(provider) = crate::auth::providers::aws::aws_provider() {
            self.providers.insert("aws".to_string(), Arc::new(provider));
        }

        // OpenAI (always available)
        if let Ok(provider) = crate::auth::providers::api_key::openai_provider() {
            self.providers.insert("openai".to_string(), Arc::new(provider));
        }

        // Anthropic (always available)
        if let Ok(provider) = crate::auth::providers::api_key::anthropic_provider() {
            self.providers.insert("anthropic".to_string(), Arc::new(provider));
        }
    }

    /// Get a provider by ID.
    pub fn get(&self, id: &str) -> Option<Arc<dyn AuthProvider>> {
        self.providers.get(id).cloned()
    }

    /// List all available providers.
    pub fn list(&self) -> Vec<(&str, &dyn AuthProvider)> {
        self.providers
            .iter()
            .map(|(id, p)| (id.as_str(), p.as_ref()))
            .collect()
    }

    /// Register a custom provider.
    pub fn register(&mut self, provider: Arc<dyn AuthProvider>) {
        self.providers.insert(provider.id().to_string(), provider);
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute the `skill auth login` command.
pub async fn login(
    provider_id: &str,
    skill: Option<&str>,
    instance: Option<&str>,
    scopes: Option<Vec<String>>,
) -> Result<()> {
    let registry = ProviderRegistry::new();
    let token_store = TokenStore::new();

    let provider = registry.get(provider_id).ok_or_else(|| {
        anyhow!(
            "Unknown provider '{}'. Run 'skill auth providers' to see available providers.",
            provider_id
        )
    })?;

    println!();
    println!(
        "{} Authenticating with {}...",
        "->".cyan().bold(),
        provider.display_name().green()
    );

    // Run authentication flow
    let result = provider.authenticate(scopes).await?;

    // Store credentials
    token_store.store(provider_id, skill, instance, &result).await?;

    println!();
    println!(
        "{} Successfully authenticated with {}!",
        "✓".green().bold(),
        provider.display_name()
    );

    if !result.scopes.is_empty() {
        println!("  Scopes: {}", result.scopes.join(", ").dimmed());
    }

    if let Some(expires) = result.expires_at {
        let duration = expires - Utc::now();
        if duration.num_hours() > 24 {
            println!("  Expires in {} days", duration.num_days());
        } else {
            println!("  Expires in {} hours", duration.num_hours());
        }
    }

    if let Some(skill_name) = skill {
        println!("  Associated with skill: {}", skill_name.cyan());
    }

    println!();
    println!(
        "{} Credentials stored securely in system keyring.",
        "ℹ".blue()
    );

    Ok(())
}

/// Execute the `skill auth status` command.
pub async fn status(provider_filter: Option<&str>) -> Result<()> {
    let registry = ProviderRegistry::new();
    let token_store = TokenStore::new();

    println!();
    println!("{}", "Authentication Status".bold());
    println!("{}", "─".repeat(50));

    let mut found_any = false;

    for (provider_id, provider) in registry.list() {
        if let Some(filter) = provider_filter {
            if provider_id != filter {
                continue;
            }
        }

        // Try to load credentials
        if let Ok(Some((creds, _))) = token_store.load(provider_id, None, None).await {
            found_any = true;

            let status_icon = if creds.is_expired() {
                "✗".red()
            } else if creds.needs_refresh() {
                "⚠".yellow()
            } else {
                "✓".green()
            };

            println!();
            println!(
                "{} {} ({})",
                status_icon,
                provider.display_name().bold(),
                provider.auth_type()
            );

            if creds.is_expired() {
                println!("  Status: {}", "Expired".red());
            } else if creds.needs_refresh() {
                println!("  Status: {}", "Expiring soon".yellow());
            } else {
                println!("  Status: {}", "Active".green());
            }

            if let Some(expires) = creds.expires_at {
                let now = Utc::now();
                if expires > now {
                    let duration = expires - now;
                    if duration.num_days() > 0 {
                        println!("  Expires: in {} days", duration.num_days());
                    } else if duration.num_hours() > 0 {
                        println!("  Expires: in {} hours", duration.num_hours());
                    } else {
                        println!("  Expires: in {} minutes", duration.num_minutes());
                    }
                } else {
                    println!("  Expired: {}", expires.format("%Y-%m-%d %H:%M UTC"));
                }
            } else {
                println!("  Expires: Never");
            }

            if !creds.scopes.is_empty() {
                println!("  Scopes: {}", creds.scopes.join(", "));
            }
        }
    }

    if !found_any {
        println!();
        println!("  No active authentication sessions found.");
        println!();
        println!(
            "  Run {} to authenticate.",
            "skill auth login <provider>".cyan()
        );
    }

    println!();

    Ok(())
}

/// Execute the `skill auth logout` command.
pub async fn logout(
    provider_id: &str,
    skill: Option<&str>,
    instance: Option<&str>,
) -> Result<()> {
    let registry = ProviderRegistry::new();
    let token_store = TokenStore::new();

    let provider = registry.get(provider_id).ok_or_else(|| {
        anyhow!("Unknown provider '{}'", provider_id)
    })?;

    // Load credentials to revoke them
    if let Some((creds, _)) = token_store.load(provider_id, skill, instance).await? {
        // Try to revoke with provider
        if let Err(e) = provider.revoke(&creds).await {
            eprintln!("Warning: Failed to revoke token with provider: {}", e);
            // Continue anyway - we'll still delete local credentials
        }
    }

    // Delete from keyring
    token_store.delete(provider_id, skill, instance).await?;

    println!();
    println!(
        "{} Logged out from {}",
        "✓".green().bold(),
        provider.display_name()
    );
    println!();

    Ok(())
}

/// Execute the `skill auth providers` command.
pub async fn providers() -> Result<()> {
    let registry = ProviderRegistry::new();

    println!();
    println!("{}", "Available Authentication Providers".bold());
    println!("{}", "─".repeat(50));

    let mut oauth2_providers: Vec<_> = Vec::new();
    let mut api_key_providers: Vec<_> = Vec::new();
    let mut iam_providers: Vec<_> = Vec::new();

    for (id, provider) in registry.list() {
        match provider.auth_type() {
            AuthType::OAuth2DeviceFlow | AuthType::OAuth2AuthorizationCode => {
                oauth2_providers.push((id, provider));
            }
            AuthType::ApiKey => {
                api_key_providers.push((id, provider));
            }
            AuthType::AwsIam => {
                iam_providers.push((id, provider));
            }
            _ => {}
        }
    }

    if !oauth2_providers.is_empty() {
        println!();
        println!("{}", "OAuth2 Providers".cyan().bold());
        for (id, provider) in oauth2_providers {
            println!(
                "  {} - {} (Device Flow)",
                id.green(),
                provider.display_name()
            );
        }
    }

    if !api_key_providers.is_empty() {
        println!();
        println!("{}", "API Key Providers".cyan().bold());
        for (id, provider) in api_key_providers {
            println!("  {} - {}", id.green(), provider.display_name());
        }
    }

    if !iam_providers.is_empty() {
        println!();
        println!("{}", "Cloud IAM Providers".cyan().bold());
        for (id, provider) in iam_providers {
            println!("  {} - {}", id.green(), provider.display_name());
        }
    }

    println!();
    println!("{}", "Usage".bold());
    println!("  skill auth login <provider>        # Authenticate");
    println!("  skill auth login <provider> -s <skill>  # Associate with skill");
    println!("  skill auth status                  # Check authentication status");
    println!("  skill auth logout <provider>       # Remove credentials");
    println!();

    // Show configuration hints for OAuth2 providers
    if std::env::var("GITHUB_OAUTH_CLIENT_ID").is_err() {
        println!("{}", "Note".yellow().bold());
        println!("  To enable GitHub OAuth, set: GITHUB_OAUTH_CLIENT_ID");
        println!("  To enable Google OAuth, set: GOOGLE_OAUTH_CLIENT_ID");
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_registry() {
        let registry = ProviderRegistry::new();

        // AWS should always be available
        assert!(registry.get("aws").is_some());

        // OpenAI and Anthropic API key providers should be available
        assert!(registry.get("openai").is_some());
        assert!(registry.get("anthropic").is_some());
    }
}
