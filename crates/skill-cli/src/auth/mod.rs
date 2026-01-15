//! Authentication system for Skill Engine.
//!
//! This module provides a pluggable authentication framework supporting:
//! - OAuth2 Device Flow (GitHub, Google, Azure AD)
//! - API Key authentication (OpenAI, Anthropic, etc.)
//! - AWS IAM credentials
//! - Custom authentication providers
//!
//! # Architecture
//!
//! Authentication happens at the CLI level, not in WASM skills. This design:
//! - Keeps skills simple and stateless
//! - Allows token refresh between skill executions
//! - Uses the system keyring for secure credential storage
//! - Provides automatic token refresh when credentials expire
//!
//! # Usage
//!
//! ```bash
//! # Authenticate with a provider
//! skill auth login github
//!
//! # Check authentication status
//! skill auth status
//!
//! # Associate credentials with a skill
//! skill auth login github --skill my-skill
//!
//! # Logout (revoke credentials)
//! skill auth logout github
//! ```
//!
//! # For Skill Developers
//!
//! Skills receive credentials via the `get-config()` function:
//!
//! ```javascript
//! // In your skill
//! const token = getConfig('GITHUB_TOKEN');
//! const response = await fetch('https://api.github.com/user', {
//!   headers: { 'Authorization': `Bearer ${token}` }
//! });
//! ```

pub mod provider;
pub mod providers;
pub mod token_store;
pub mod commands;

pub use provider::{
    AuthProvider, AuthResult, AuthType, Credentials, CredentialType,
    ProviderConfig, OAuth2Config, ApiKeyConfig, AwsConfig,
};
pub use token_store::TokenStore;
pub use commands::{ProviderRegistry, login, status, logout, providers};
