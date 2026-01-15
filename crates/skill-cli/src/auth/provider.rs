//! Auth provider trait and types for the Skill Engine authentication system.
//!
//! This module defines the core abstractions for authentication providers,
//! supporting OAuth2, API keys, AWS IAM, mTLS, and custom authentication flows.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Type of authentication supported by a provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    /// OAuth2 Device Authorization Grant (RFC 8628)
    /// Best for CLI applications - user authenticates in browser
    OAuth2DeviceFlow,

    /// OAuth2 Authorization Code Grant
    /// Requires local HTTP server to receive callback
    OAuth2AuthorizationCode,

    /// OAuth2 Client Credentials Grant
    /// For service-to-service authentication
    OAuth2ClientCredentials,

    /// Simple API key authentication
    ApiKey,

    /// AWS IAM credentials (access key + secret key)
    AwsIam,

    /// Azure Active Directory authentication
    AzureAd,

    /// Mutual TLS certificate authentication
    MutualTls,

    /// Custom authentication provider
    Custom,
}

impl fmt::Display for AuthType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthType::OAuth2DeviceFlow => write!(f, "OAuth2 Device Flow"),
            AuthType::OAuth2AuthorizationCode => write!(f, "OAuth2 Authorization Code"),
            AuthType::OAuth2ClientCredentials => write!(f, "OAuth2 Client Credentials"),
            AuthType::ApiKey => write!(f, "API Key"),
            AuthType::AwsIam => write!(f, "AWS IAM"),
            AuthType::AzureAd => write!(f, "Azure AD"),
            AuthType::MutualTls => write!(f, "Mutual TLS"),
            AuthType::Custom => write!(f, "Custom"),
        }
    }
}

/// Configuration for an authentication provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider identifier (e.g., "github", "aws", "google")
    pub id: String,

    /// Human-readable display name
    pub display_name: String,

    /// Authentication type
    pub auth_type: AuthType,

    /// OAuth2-specific configuration
    #[serde(default)]
    pub oauth2: Option<OAuth2Config>,

    /// API key configuration
    #[serde(default)]
    pub api_key: Option<ApiKeyConfig>,

    /// AWS-specific configuration
    #[serde(default)]
    pub aws: Option<AwsConfig>,

    /// Custom configuration as key-value pairs
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

/// OAuth2-specific provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    /// Device authorization endpoint (for Device Flow)
    pub device_authorization_endpoint: Option<String>,

    /// Authorization endpoint (for Authorization Code Flow)
    pub authorization_endpoint: Option<String>,

    /// Token endpoint
    pub token_endpoint: String,

    /// Token revocation endpoint
    pub revocation_endpoint: Option<String>,

    /// Client ID
    pub client_id: String,

    /// Client secret (optional for public clients)
    #[serde(default)]
    pub client_secret: Option<String>,

    /// Default scopes to request
    #[serde(default)]
    pub scopes: Vec<String>,

    /// Audience parameter (required by some providers)
    pub audience: Option<String>,
}

/// API key provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// Header name for the API key (e.g., "X-API-Key", "Authorization")
    pub header_name: String,

    /// Prefix for the header value (e.g., "Bearer ", "Api-Key ")
    #[serde(default)]
    pub header_prefix: Option<String>,

    /// Environment variable name to map the key to
    pub env_var_name: String,
}

/// AWS-specific provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    /// AWS region
    pub region: Option<String>,

    /// AWS profile name
    pub profile: Option<String>,

    /// Credential source (environment, profile, imds, ecs)
    pub credential_source: Option<String>,
}

/// Result of a successful authentication.
#[derive(Debug, Clone)]
pub struct AuthResult {
    /// The obtained credentials
    pub credentials: Credentials,

    /// When the credentials expire (if known)
    pub expires_at: Option<DateTime<Utc>>,

    /// Refresh token for obtaining new access tokens
    pub refresh_token: Option<SecretString>,

    /// Granted scopes (may differ from requested)
    pub scopes: Vec<String>,

    /// Additional metadata from the auth response
    pub metadata: HashMap<String, String>,
}

/// Stored credentials for a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Provider that issued these credentials
    pub provider_id: String,

    /// Type of credential
    pub credential_type: CredentialType,

    /// When these credentials expire
    pub expires_at: Option<DateTime<Utc>>,

    /// Scopes granted with these credentials
    #[serde(default)]
    pub scopes: Vec<String>,

    /// Credential data (access token, API key, etc.)
    /// Note: Sensitive values stored separately in keyring
    #[serde(default)]
    pub data: HashMap<String, String>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Credentials {
    /// Check if these credentials have expired.
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() >= expires,
            None => false, // No expiry means never expires
        }
    }

    /// Check if these credentials will expire within the given duration.
    pub fn expires_within(&self, duration: chrono::Duration) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() + duration >= expires,
            None => false,
        }
    }

    /// Check if these credentials need refresh (expired or expiring soon).
    pub fn needs_refresh(&self) -> bool {
        // Refresh if expiring within 5 minutes
        self.expires_within(chrono::Duration::minutes(5))
    }
}

/// Type of credential stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialType {
    /// OAuth2 access token
    OAuth2AccessToken,

    /// OAuth2 refresh token
    OAuth2RefreshToken,

    /// Simple API key
    ApiKey,

    /// AWS access key ID
    AwsAccessKeyId,

    /// AWS secret access key
    AwsSecretAccessKey,

    /// AWS session token
    AwsSessionToken,

    /// Client certificate
    Certificate,

    /// Client private key
    PrivateKey,

    /// Generic secret
    Secret,
}

/// Status of authentication for a provider.
#[derive(Debug, Clone)]
pub struct AuthStatus {
    /// Provider ID
    pub provider_id: String,

    /// Provider display name
    pub display_name: String,

    /// Whether authenticated
    pub authenticated: bool,

    /// Associated skill (if any)
    pub skill: Option<String>,

    /// Associated instance (if any)
    pub instance: Option<String>,

    /// When credentials expire
    pub expires_at: Option<DateTime<Utc>>,

    /// Granted scopes
    pub scopes: Vec<String>,

    /// Human-readable status message
    pub message: String,
}

/// Trait for authentication providers.
///
/// Providers handle the authentication flow for their specific auth type,
/// including token refresh and revocation.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Get the provider's unique identifier.
    fn id(&self) -> &str;

    /// Get the provider's human-readable display name.
    fn display_name(&self) -> &str;

    /// Get the authentication type this provider uses.
    fn auth_type(&self) -> AuthType;

    /// Get the provider's configuration.
    fn config(&self) -> &ProviderConfig;

    /// Initiate the authentication flow.
    ///
    /// For OAuth2 Device Flow, this returns instructions for the user
    /// (URL to visit, code to enter) and polls for completion.
    ///
    /// For API key auth, this prompts for the key interactively.
    async fn authenticate(&self, scopes: Option<Vec<String>>) -> Result<AuthResult>;

    /// Refresh credentials using a refresh token.
    ///
    /// Returns new credentials if refresh succeeds, or an error if
    /// re-authentication is required.
    async fn refresh(&self, credentials: &Credentials, refresh_token: &SecretString) -> Result<AuthResult>;

    /// Validate that credentials are still valid.
    ///
    /// This may make a test API call or introspect the token.
    async fn validate(&self, credentials: &Credentials) -> Result<bool>;

    /// Revoke credentials (logout).
    ///
    /// This invalidates the credentials with the provider.
    async fn revoke(&self, credentials: &Credentials) -> Result<()>;

    /// Convert credentials to configuration map for skills.
    ///
    /// Returns a map of environment variable names to values that
    /// will be passed to skills via get-config().
    fn to_skill_config(&self, credentials: &Credentials) -> HashMap<String, String>;

    /// Get the list of secret keys that should be stored in keyring.
    ///
    /// These are the keys from to_skill_config() that contain sensitive
    /// values and should be stored securely rather than in plain config.
    fn secret_keys(&self) -> Vec<&str>;
}

/// Device authorization response from OAuth2 Device Flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuthorizationResponse {
    /// Device code for polling
    pub device_code: String,

    /// User code to enter at verification URI
    pub user_code: String,

    /// URI for user to visit
    pub verification_uri: String,

    /// Optional URI with code pre-filled
    pub verification_uri_complete: Option<String>,

    /// Seconds until device code expires
    pub expires_in: u64,

    /// Minimum seconds between polling requests
    pub interval: u64,
}

/// Token response from OAuth2 token endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    /// Access token
    pub access_token: String,

    /// Token type (usually "Bearer")
    pub token_type: String,

    /// Seconds until access token expires
    pub expires_in: Option<u64>,

    /// Refresh token for obtaining new access tokens
    pub refresh_token: Option<String>,

    /// Granted scopes (space-separated)
    pub scope: Option<String>,
}

/// Error response from OAuth2 endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Error {
    /// Error code
    pub error: String,

    /// Human-readable error description
    pub error_description: Option<String>,

    /// URI with more information
    pub error_uri: Option<String>,
}

impl fmt::Display for OAuth2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.error_description {
            Some(desc) => write!(f, "{}: {}", self.error, desc),
            None => write!(f, "{}", self.error),
        }
    }
}

impl std::error::Error for OAuth2Error {}
