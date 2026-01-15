//! API Key authentication provider.
//!
//! Simple provider for services that use API keys for authentication.
//! Prompts user for API key and stores it securely in keyring.

use crate::auth::provider::{
    ApiKeyConfig, AuthProvider, AuthResult, AuthType, Credentials, CredentialType,
    ProviderConfig,
};
use anyhow::{bail, Result};
use async_trait::async_trait;
use dialoguer::{Password, theme::ColorfulTheme};
use secrecy::SecretString;
use std::collections::HashMap;

/// API Key authentication provider.
pub struct ApiKeyProvider {
    config: ProviderConfig,
}

impl ApiKeyProvider {
    /// Create a new API Key provider with the given configuration.
    pub fn new(config: ProviderConfig) -> Result<Self> {
        if config.api_key.is_none() {
            bail!("API key configuration required for API key provider");
        }

        Ok(Self { config })
    }

    /// Get the API key configuration.
    fn api_key_config(&self) -> &ApiKeyConfig {
        self.config.api_key.as_ref().unwrap()
    }
}

#[async_trait]
impl AuthProvider for ApiKeyProvider {
    fn id(&self) -> &str {
        &self.config.id
    }

    fn display_name(&self) -> &str {
        &self.config.display_name
    }

    fn auth_type(&self) -> AuthType {
        AuthType::ApiKey
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }

    async fn authenticate(&self, _scopes: Option<Vec<String>>) -> Result<AuthResult> {
        let api_key_config = self.api_key_config();

        // Prompt for API key
        let api_key = Password::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Enter {} API key", self.config.display_name))
            .interact()?;

        let credentials = Credentials {
            provider_id: self.config.id.clone(),
            credential_type: CredentialType::ApiKey,
            expires_at: None, // API keys typically don't expire
            scopes: vec![],
            data: HashMap::from([
                ("api_key".to_string(), api_key.clone()),
            ]),
            metadata: HashMap::from([
                ("env_var_name".to_string(), api_key_config.env_var_name.clone()),
            ]),
        };

        Ok(AuthResult {
            credentials,
            expires_at: None,
            refresh_token: None,
            scopes: vec![],
            metadata: HashMap::new(),
        })
    }

    async fn refresh(&self, credentials: &Credentials, _refresh_token: &SecretString) -> Result<AuthResult> {
        // API keys don't refresh - just return existing credentials
        Ok(AuthResult {
            credentials: credentials.clone(),
            expires_at: None,
            refresh_token: None,
            scopes: vec![],
            metadata: HashMap::new(),
        })
    }

    async fn validate(&self, _credentials: &Credentials) -> Result<bool> {
        // We can't validate API keys without making a provider-specific API call
        // For now, assume they're valid
        Ok(true)
    }

    async fn revoke(&self, _credentials: &Credentials) -> Result<()> {
        // API keys can't be revoked via API - user must do it manually
        Ok(())
    }

    fn to_skill_config(&self, credentials: &Credentials) -> HashMap<String, String> {
        let mut config = HashMap::new();
        let api_key_config = self.api_key_config();

        if let Some(api_key) = credentials.data.get("api_key") {
            config.insert(api_key_config.env_var_name.clone(), api_key.clone());
            config.insert("API_KEY".to_string(), api_key.clone());
        }

        config
    }

    fn secret_keys(&self) -> Vec<&str> {
        vec!["API_KEY"]
    }
}

/// Create an OpenAI API key provider.
pub fn openai_provider() -> Result<ApiKeyProvider> {
    let config = ProviderConfig {
        id: "openai".to_string(),
        display_name: "OpenAI".to_string(),
        auth_type: AuthType::ApiKey,
        oauth2: None,
        api_key: Some(ApiKeyConfig {
            header_name: "Authorization".to_string(),
            header_prefix: Some("Bearer ".to_string()),
            env_var_name: "OPENAI_API_KEY".to_string(),
        }),
        aws: None,
        custom: HashMap::new(),
    };

    ApiKeyProvider::new(config)
}

/// Create an Anthropic API key provider.
pub fn anthropic_provider() -> Result<ApiKeyProvider> {
    let config = ProviderConfig {
        id: "anthropic".to_string(),
        display_name: "Anthropic".to_string(),
        auth_type: AuthType::ApiKey,
        oauth2: None,
        api_key: Some(ApiKeyConfig {
            header_name: "x-api-key".to_string(),
            header_prefix: None,
            env_var_name: "ANTHROPIC_API_KEY".to_string(),
        }),
        aws: None,
        custom: HashMap::new(),
    };

    ApiKeyProvider::new(config)
}

/// Create a generic API key provider.
pub fn generic_api_key_provider(id: &str, display_name: &str, env_var_name: &str) -> Result<ApiKeyProvider> {
    let config = ProviderConfig {
        id: id.to_string(),
        display_name: display_name.to_string(),
        auth_type: AuthType::ApiKey,
        oauth2: None,
        api_key: Some(ApiKeyConfig {
            header_name: "Authorization".to_string(),
            header_prefix: Some("Bearer ".to_string()),
            env_var_name: env_var_name.to_string(),
        }),
        aws: None,
        custom: HashMap::new(),
    };

    ApiKeyProvider::new(config)
}
