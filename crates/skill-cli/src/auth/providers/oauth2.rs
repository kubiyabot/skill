//! OAuth2 authentication provider supporting Device Flow (RFC 8628).
//!
//! The Device Flow is ideal for CLI applications because:
//! - No need to run a local HTTP server
//! - User authenticates in their browser
//! - CLI polls for completion
//!
//! Supported providers: GitHub, Google, Azure AD, and any OAuth2 provider
//! that implements the Device Authorization Grant.

use crate::auth::provider::{
    AuthProvider, AuthResult, AuthType, Credentials, CredentialType,
    DeviceAuthorizationResponse, OAuth2Config, OAuth2Error, ProviderConfig,
    TokenResponse,
};
use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use std::collections::HashMap;
use std::time::Duration as StdDuration;

/// OAuth2 provider implementing Device Authorization Grant (RFC 8628).
pub struct OAuth2Provider {
    config: ProviderConfig,
    client: Client,
}

impl OAuth2Provider {
    /// Create a new OAuth2 provider with the given configuration.
    pub fn new(config: ProviderConfig) -> Result<Self> {
        if config.oauth2.is_none() {
            bail!("OAuth2 configuration required for OAuth2 provider");
        }

        let client = Client::builder()
            .timeout(StdDuration::from_secs(30))
            .build()?;

        Ok(Self { config, client })
    }

    /// Get the OAuth2 configuration.
    fn oauth2_config(&self) -> &OAuth2Config {
        self.config.oauth2.as_ref().unwrap()
    }

    /// Request device authorization code.
    async fn request_device_code(&self, scopes: &[String]) -> Result<DeviceAuthorizationResponse> {
        let oauth2 = self.oauth2_config();
        let endpoint = oauth2.device_authorization_endpoint.as_ref()
            .ok_or_else(|| anyhow!("Device authorization endpoint not configured"))?;

        let scope = if scopes.is_empty() {
            oauth2.scopes.join(" ")
        } else {
            scopes.join(" ")
        };

        let mut params = vec![
            ("client_id", oauth2.client_id.clone()),
            ("scope", scope),
        ];

        if let Some(audience) = &oauth2.audience {
            params.push(("audience", audience.clone()));
        }

        let response = self.client
            .post(endpoint)
            .form(&params)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to request device code")?;

        if !response.status().is_success() {
            let error: OAuth2Error = response.json().await
                .unwrap_or_else(|_| OAuth2Error {
                    error: "unknown_error".to_string(),
                    error_description: Some("Failed to parse error response".to_string()),
                    error_uri: None,
                });
            bail!("Device authorization failed: {}", error);
        }

        response.json().await.context("Failed to parse device authorization response")
    }

    /// Poll for token after user authorization.
    async fn poll_for_token(&self, device_response: &DeviceAuthorizationResponse) -> Result<TokenResponse> {
        let oauth2 = self.oauth2_config();
        let mut interval = device_response.interval;
        let deadline = Utc::now() + Duration::seconds(device_response.expires_in as i64);

        // Create progress spinner
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
        );
        pb.set_message("Waiting for authorization...");
        pb.enable_steady_tick(StdDuration::from_millis(100));

        loop {
            if Utc::now() >= deadline {
                pb.finish_and_clear();
                bail!("Authorization timed out. Please try again.");
            }

            tokio::time::sleep(StdDuration::from_secs(interval)).await;

            let mut params = vec![
                ("client_id", oauth2.client_id.clone()),
                ("device_code", device_response.device_code.clone()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code".to_string()),
            ];

            if let Some(secret) = &oauth2.client_secret {
                params.push(("client_secret", secret.clone()));
            }

            let response = self.client
                .post(&oauth2.token_endpoint)
                .form(&params)
                .header("Accept", "application/json")
                .send()
                .await?;

            if response.status().is_success() {
                pb.finish_and_clear();
                return response.json().await.context("Failed to parse token response");
            }

            let error: OAuth2Error = response.json().await
                .unwrap_or_else(|_| OAuth2Error {
                    error: "unknown_error".to_string(),
                    error_description: None,
                    error_uri: None,
                });

            match error.error.as_str() {
                "authorization_pending" => {
                    // User hasn't authorized yet, keep polling
                    continue;
                }
                "slow_down" => {
                    // Increase polling interval
                    interval += 5;
                    continue;
                }
                "access_denied" => {
                    pb.finish_and_clear();
                    bail!("Authorization denied by user");
                }
                "expired_token" => {
                    pb.finish_and_clear();
                    bail!("Device code expired. Please try again.");
                }
                _ => {
                    pb.finish_and_clear();
                    bail!("Authorization failed: {}", error);
                }
            }
        }
    }

    /// Refresh an access token using a refresh token.
    #[allow(dead_code)]
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let oauth2 = self.oauth2_config();

        let mut params = vec![
            ("client_id", oauth2.client_id.clone()),
            ("grant_type", "refresh_token".to_string()),
            ("refresh_token", refresh_token.to_string()),
        ];

        if let Some(secret) = &oauth2.client_secret {
            params.push(("client_secret", secret.clone()));
        }

        let response = self.client
            .post(&oauth2.token_endpoint)
            .form(&params)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to refresh token")?;

        if !response.status().is_success() {
            let error: OAuth2Error = response.json().await
                .unwrap_or_else(|_| OAuth2Error {
                    error: "unknown_error".to_string(),
                    error_description: Some("Failed to refresh token".to_string()),
                    error_uri: None,
                });
            bail!("Token refresh failed: {}", error);
        }

        response.json().await.context("Failed to parse token response")
    }

    /// Revoke a token.
    async fn revoke_token(&self, token: &str, token_type: &str) -> Result<()> {
        let oauth2 = self.oauth2_config();

        let endpoint = match &oauth2.revocation_endpoint {
            Some(endpoint) => endpoint,
            None => {
                // No revocation endpoint configured, just return success
                return Ok(());
            }
        };

        let mut params = vec![
            ("token", token.to_string()),
            ("token_type_hint", token_type.to_string()),
        ];

        if let Some(secret) = &oauth2.client_secret {
            params.push(("client_secret", secret.clone()));
        }

        let response = self.client
            .post(endpoint)
            .form(&params)
            .send()
            .await
            .context("Failed to revoke token")?;

        // RFC 7009: Revocation endpoint should return 200 even if token was already invalid
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("Token revocation failed: {} - {}", status, body);
        }

        Ok(())
    }
}

#[async_trait]
impl AuthProvider for OAuth2Provider {
    fn id(&self) -> &str {
        &self.config.id
    }

    fn display_name(&self) -> &str {
        &self.config.display_name
    }

    fn auth_type(&self) -> AuthType {
        self.config.auth_type
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }

    async fn authenticate(&self, scopes: Option<Vec<String>>) -> Result<AuthResult> {
        let oauth2 = self.oauth2_config();
        let scopes = scopes.unwrap_or_else(|| oauth2.scopes.clone());

        // Step 1: Request device code
        let device_response = self.request_device_code(&scopes).await?;

        // Step 2: Display instructions to user
        println!();
        println!("{} Open this URL in your browser:", "->".cyan().bold());
        println!("   {}", device_response.verification_uri.yellow());
        println!();
        println!("{} Enter code: {}", "->".cyan().bold(),
                 device_response.user_code.green().bold());

        if let Some(uri) = &device_response.verification_uri_complete {
            println!();
            println!("   Or open: {}", uri.dimmed());
        }
        println!();

        // Step 3: Poll for token
        let token_response = self.poll_for_token(&device_response).await?;

        // Step 4: Build credentials
        let expires_at = token_response.expires_in
            .map(|secs| Utc::now() + Duration::seconds(secs as i64));

        let granted_scopes: Vec<String> = token_response.scope
            .as_ref()
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_else(|| scopes.clone());

        let credentials = Credentials {
            provider_id: self.config.id.clone(),
            credential_type: CredentialType::OAuth2AccessToken,
            expires_at,
            scopes: granted_scopes.clone(),
            data: HashMap::from([
                ("access_token".to_string(), token_response.access_token.clone()),
            ]),
            metadata: HashMap::from([
                ("token_type".to_string(), token_response.token_type),
            ]),
        };

        Ok(AuthResult {
            credentials,
            expires_at,
            refresh_token: token_response.refresh_token.map(SecretString::from),
            scopes: granted_scopes,
            metadata: HashMap::new(),
        })
    }

    async fn refresh(&self, _credentials: &Credentials, refresh_token: &SecretString) -> Result<AuthResult> {
        let token_response = self.refresh_token(refresh_token.expose_secret()).await?;

        let expires_at = token_response.expires_in
            .map(|secs| Utc::now() + Duration::seconds(secs as i64));

        let granted_scopes: Vec<String> = token_response.scope
            .as_ref()
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default();

        let credentials = Credentials {
            provider_id: self.config.id.clone(),
            credential_type: CredentialType::OAuth2AccessToken,
            expires_at,
            scopes: granted_scopes.clone(),
            data: HashMap::from([
                ("access_token".to_string(), token_response.access_token.clone()),
            ]),
            metadata: HashMap::from([
                ("token_type".to_string(), token_response.token_type),
            ]),
        };

        // Use new refresh token if provided, otherwise keep the old one
        let new_refresh_token = token_response.refresh_token
            .map(SecretString::from)
            .or_else(|| Some(refresh_token.clone()));

        Ok(AuthResult {
            credentials,
            expires_at,
            refresh_token: new_refresh_token,
            scopes: granted_scopes,
            metadata: HashMap::new(),
        })
    }

    async fn validate(&self, credentials: &Credentials) -> Result<bool> {
        // Check expiration first
        if credentials.is_expired() {
            return Ok(false);
        }

        // For OAuth2, we could make a test API call, but for now just check expiration
        // This is provider-specific and would require knowing the API to call
        Ok(true)
    }

    async fn revoke(&self, credentials: &Credentials) -> Result<()> {
        if let Some(token) = credentials.data.get("access_token") {
            self.revoke_token(token, "access_token").await?;
        }
        Ok(())
    }

    fn to_skill_config(&self, credentials: &Credentials) -> HashMap<String, String> {
        let mut config = HashMap::new();

        // Map access token to provider-specific env var
        if let Some(token) = credentials.data.get("access_token") {
            let env_var = format!("{}_TOKEN", self.config.id.to_uppercase());
            config.insert(env_var, token.clone());

            // Also provide as ACCESS_TOKEN for generic usage
            config.insert("ACCESS_TOKEN".to_string(), token.clone());
        }

        config
    }

    fn secret_keys(&self) -> Vec<&str> {
        // Note: We only return static strings here since we can't return references to owned data
        vec!["ACCESS_TOKEN"]
    }
}

/// Create a GitHub OAuth2 provider with default configuration.
pub fn github_provider(client_id: String) -> Result<OAuth2Provider> {
    let config = ProviderConfig {
        id: "github".to_string(),
        display_name: "GitHub".to_string(),
        auth_type: AuthType::OAuth2DeviceFlow,
        oauth2: Some(OAuth2Config {
            device_authorization_endpoint: Some("https://github.com/login/device/code".to_string()),
            authorization_endpoint: Some("https://github.com/login/oauth/authorize".to_string()),
            token_endpoint: "https://github.com/login/oauth/access_token".to_string(),
            revocation_endpoint: None, // GitHub doesn't support OAuth revocation
            client_id,
            client_secret: None,
            scopes: vec!["repo".to_string(), "read:user".to_string()],
            audience: None,
        }),
        api_key: None,
        aws: None,
        custom: HashMap::new(),
    };

    OAuth2Provider::new(config)
}

/// Create a Google OAuth2 provider with default configuration.
pub fn google_provider(client_id: String, client_secret: Option<String>) -> Result<OAuth2Provider> {
    let config = ProviderConfig {
        id: "google".to_string(),
        display_name: "Google".to_string(),
        auth_type: AuthType::OAuth2DeviceFlow,
        oauth2: Some(OAuth2Config {
            device_authorization_endpoint: Some("https://oauth2.googleapis.com/device/code".to_string()),
            authorization_endpoint: Some("https://accounts.google.com/o/oauth2/v2/auth".to_string()),
            token_endpoint: "https://oauth2.googleapis.com/token".to_string(),
            revocation_endpoint: Some("https://oauth2.googleapis.com/revoke".to_string()),
            client_id,
            client_secret,
            scopes: vec!["https://www.googleapis.com/auth/cloud-platform".to_string()],
            audience: None,
        }),
        api_key: None,
        aws: None,
        custom: HashMap::new(),
    };

    OAuth2Provider::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_expiry() {
        let mut creds = Credentials {
            provider_id: "test".to_string(),
            credential_type: CredentialType::OAuth2AccessToken,
            expires_at: Some(Utc::now() + Duration::hours(1)),
            scopes: vec![],
            data: HashMap::new(),
            metadata: HashMap::new(),
        };

        assert!(!creds.is_expired());
        assert!(!creds.needs_refresh());

        // Set to expire in 2 minutes
        creds.expires_at = Some(Utc::now() + Duration::minutes(2));
        assert!(!creds.is_expired());
        assert!(creds.needs_refresh()); // Within 5-minute refresh window

        // Set to expired
        creds.expires_at = Some(Utc::now() - Duration::minutes(1));
        assert!(creds.is_expired());
        assert!(creds.needs_refresh());
    }

    #[test]
    fn test_github_provider_creation() {
        let provider = github_provider("test_client_id".to_string());
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.id(), "github");
        assert_eq!(provider.display_name(), "GitHub");
        assert_eq!(provider.auth_type(), AuthType::OAuth2DeviceFlow);
    }
}
