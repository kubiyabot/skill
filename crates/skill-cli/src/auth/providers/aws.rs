//! AWS IAM authentication provider.
//!
//! Supports AWS credential management including:
//! - Access key + secret key pairs
//! - Session tokens (for temporary credentials)
//! - Region configuration

use crate::auth::provider::{
    AuthProvider, AuthResult, AuthType, AwsConfig, Credentials, CredentialType,
    ProviderConfig,
};
use anyhow::{bail, Result};
use async_trait::async_trait;
use dialoguer::{Input, Password, theme::ColorfulTheme};
use secrecy::SecretString;
use std::collections::HashMap;

/// AWS IAM authentication provider.
pub struct AwsProvider {
    config: ProviderConfig,
}

impl AwsProvider {
    /// Create a new AWS provider with the given configuration.
    pub fn new(config: ProviderConfig) -> Result<Self> {
        if config.aws.is_none() {
            bail!("AWS configuration required for AWS provider");
        }

        Ok(Self { config })
    }

    /// Get the AWS configuration.
    fn aws_config(&self) -> &AwsConfig {
        self.config.aws.as_ref().unwrap()
    }
}

#[async_trait]
impl AuthProvider for AwsProvider {
    fn id(&self) -> &str {
        &self.config.id
    }

    fn display_name(&self) -> &str {
        &self.config.display_name
    }

    fn auth_type(&self) -> AuthType {
        AuthType::AwsIam
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }

    async fn authenticate(&self, _scopes: Option<Vec<String>>) -> Result<AuthResult> {
        let aws_config = self.aws_config();
        let theme = ColorfulTheme::default();

        // Prompt for access key ID
        let access_key_id: String = Input::with_theme(&theme)
            .with_prompt("AWS Access Key ID")
            .interact_text()?;

        // Prompt for secret access key
        let secret_access_key = Password::with_theme(&theme)
            .with_prompt("AWS Secret Access Key")
            .interact()?;

        // Prompt for region (with default)
        let default_region = aws_config.region.clone().unwrap_or_else(|| "us-east-1".to_string());
        let region: String = Input::with_theme(&theme)
            .with_prompt("AWS Region")
            .default(default_region)
            .interact_text()?;

        // Optionally prompt for session token
        let session_token: String = Input::with_theme(&theme)
            .with_prompt("AWS Session Token (leave empty if not using temporary credentials)")
            .allow_empty(true)
            .interact_text()?;

        let mut data = HashMap::from([
            ("access_key_id".to_string(), access_key_id.clone()),
            ("secret_access_key".to_string(), secret_access_key.clone()),
            ("region".to_string(), region.clone()),
        ]);

        if !session_token.is_empty() {
            data.insert("session_token".to_string(), session_token);
        }

        let credentials = Credentials {
            provider_id: self.config.id.clone(),
            credential_type: CredentialType::AwsAccessKeyId,
            expires_at: None, // IAM credentials don't expire (session tokens do, but we don't track that)
            scopes: vec![],
            data,
            metadata: HashMap::new(),
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
        // AWS IAM credentials don't refresh - they're either valid or need to be replaced
        Ok(AuthResult {
            credentials: credentials.clone(),
            expires_at: None,
            refresh_token: None,
            scopes: vec![],
            metadata: HashMap::new(),
        })
    }

    async fn validate(&self, credentials: &Credentials) -> Result<bool> {
        // We could call STS GetCallerIdentity to validate, but for now assume valid
        // This would require making an actual AWS API call
        Ok(credentials.data.contains_key("access_key_id")
            && credentials.data.contains_key("secret_access_key"))
    }

    async fn revoke(&self, _credentials: &Credentials) -> Result<()> {
        // AWS credentials can't be revoked via API - user must do it in AWS Console
        Ok(())
    }

    fn to_skill_config(&self, credentials: &Credentials) -> HashMap<String, String> {
        let mut config = HashMap::new();

        if let Some(access_key) = credentials.data.get("access_key_id") {
            config.insert("AWS_ACCESS_KEY_ID".to_string(), access_key.clone());
        }

        if let Some(secret_key) = credentials.data.get("secret_access_key") {
            config.insert("AWS_SECRET_ACCESS_KEY".to_string(), secret_key.clone());
        }

        if let Some(region) = credentials.data.get("region") {
            config.insert("AWS_REGION".to_string(), region.clone());
            config.insert("AWS_DEFAULT_REGION".to_string(), region.clone());
        }

        if let Some(session_token) = credentials.data.get("session_token") {
            config.insert("AWS_SESSION_TOKEN".to_string(), session_token.clone());
        }

        config
    }

    fn secret_keys(&self) -> Vec<&str> {
        vec!["AWS_SECRET_ACCESS_KEY", "AWS_SESSION_TOKEN"]
    }
}

/// Create a default AWS provider.
pub fn aws_provider() -> Result<AwsProvider> {
    let config = ProviderConfig {
        id: "aws".to_string(),
        display_name: "Amazon Web Services".to_string(),
        auth_type: AuthType::AwsIam,
        oauth2: None,
        api_key: None,
        aws: Some(AwsConfig {
            region: Some("us-east-1".to_string()),
            profile: None,
            credential_source: Some("manual".to_string()),
        }),
        custom: HashMap::new(),
    };

    AwsProvider::new(config)
}

/// Create an AWS provider with a specific region.
pub fn aws_provider_with_region(region: &str) -> Result<AwsProvider> {
    let config = ProviderConfig {
        id: "aws".to_string(),
        display_name: "Amazon Web Services".to_string(),
        auth_type: AuthType::AwsIam,
        oauth2: None,
        api_key: None,
        aws: Some(AwsConfig {
            region: Some(region.to_string()),
            profile: None,
            credential_source: Some("manual".to_string()),
        }),
        custom: HashMap::new(),
    };

    AwsProvider::new(config)
}
