//! Secrets configuration types.
//!
//! This module defines secret management configuration and provider types
//! for execution contexts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Secrets configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SecretsConfig {
    /// Individual secret definitions.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub secrets: HashMap<String, SecretDefinition>,

    /// Secret provider configuration.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<SecretProviderConfig>,
}

impl SecretsConfig {
    /// Create a new empty secrets configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a secret definition.
    pub fn with_secret(mut self, key: impl Into<String>, definition: SecretDefinition) -> Self {
        self.secrets.insert(key.into(), definition);
        self
    }

    /// Add a required secret with environment variable injection.
    pub fn with_required_env_secret(
        mut self,
        key: impl Into<String>,
        env_var: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let key = key.into();
        self.secrets.insert(
            key.clone(),
            SecretDefinition {
                key: key.clone(),
                description: Some(description.into()),
                required: true,
                provider: None,
                env_var: Some(env_var.into()),
                file_path: None,
                file_mode: None,
            },
        );
        self
    }

    /// Add a required secret that is written to a file.
    pub fn with_required_file_secret(
        mut self,
        key: impl Into<String>,
        file_path: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let key = key.into();
        self.secrets.insert(
            key.clone(),
            SecretDefinition {
                key: key.clone(),
                description: Some(description.into()),
                required: true,
                provider: None,
                env_var: None,
                file_path: Some(file_path.into()),
                file_mode: Some("0600".to_string()),
            },
        );
        self
    }

    /// Add a secret provider configuration.
    pub fn with_provider(mut self, provider: SecretProviderConfig) -> Self {
        self.providers.push(provider);
        self
    }

    /// Get all secret keys.
    pub fn keys(&self) -> Vec<&str> {
        self.secrets.keys().map(|s| s.as_str()).collect()
    }

    /// Get all required secret keys.
    pub fn required_keys(&self) -> Vec<&str> {
        self.secrets
            .iter()
            .filter(|(_, def)| def.required)
            .map(|(k, _)| k.as_str())
            .collect()
    }

    /// Get all optional secret keys.
    pub fn optional_keys(&self) -> Vec<&str> {
        self.secrets
            .iter()
            .filter(|(_, def)| !def.required)
            .map(|(k, _)| k.as_str())
            .collect()
    }

    /// Get a secret definition by key.
    pub fn get(&self, key: &str) -> Option<&SecretDefinition> {
        self.secrets.get(key)
    }

    /// Check if a secret is defined.
    pub fn contains(&self, key: &str) -> bool {
        self.secrets.contains_key(key)
    }

    /// Get the number of secrets defined.
    pub fn len(&self) -> usize {
        self.secrets.len()
    }

    /// Check if there are no secrets defined.
    pub fn is_empty(&self) -> bool {
        self.secrets.is_empty()
    }
}

/// Definition of a single secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SecretDefinition {
    /// Secret key name.
    pub key: String,

    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether this secret is required.
    #[serde(default)]
    pub required: bool,

    /// Provider to use (defaults to platform keychain).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Environment variable name to inject as.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env_var: Option<String>,

    /// File path to write secret to (for file-based secrets).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,

    /// File permissions (octal, e.g., "0600").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_mode: Option<String>,
}

impl SecretDefinition {
    /// Create a new required secret definition.
    pub fn required(key: impl Into<String>) -> Self {
        let key = key.into();
        Self {
            key: key.clone(),
            description: None,
            required: true,
            provider: None,
            env_var: None,
            file_path: None,
            file_mode: None,
        }
    }

    /// Create a new optional secret definition.
    pub fn optional(key: impl Into<String>) -> Self {
        let key = key.into();
        Self {
            key: key.clone(),
            description: None,
            required: false,
            provider: None,
            env_var: None,
            file_path: None,
            file_mode: None,
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the provider.
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Set the environment variable to inject as.
    pub fn inject_as_env(mut self, env_var: impl Into<String>) -> Self {
        self.env_var = Some(env_var.into());
        self
    }

    /// Set the file path to write to.
    pub fn write_to_file(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// Set the file mode.
    pub fn with_file_mode(mut self, mode: impl Into<String>) -> Self {
        self.file_mode = Some(mode.into());
        self
    }

    /// Check if this secret should be injected as an environment variable.
    pub fn has_env_var(&self) -> bool {
        self.env_var.is_some()
    }

    /// Check if this secret should be written to a file.
    pub fn has_file_path(&self) -> bool {
        self.file_path.is_some()
    }

    /// Get the injection targets for this secret.
    pub fn injection_targets(&self) -> Vec<SecretInjectionTarget> {
        let mut targets = Vec::new();
        if let Some(ref env_var) = self.env_var {
            targets.push(SecretInjectionTarget::EnvVar(env_var.clone()));
        }
        if let Some(ref file_path) = self.file_path {
            targets.push(SecretInjectionTarget::File {
                path: file_path.clone(),
                mode: self.file_mode.clone(),
            });
        }
        targets
    }
}

/// Where a secret should be injected.
#[derive(Debug, Clone, PartialEq)]
pub enum SecretInjectionTarget {
    /// Inject as environment variable.
    EnvVar(String),
    /// Write to file.
    File {
        /// File path to write the secret to.
        path: String,
        /// File permissions (octal, e.g., "0600").
        mode: Option<String>,
    },
}

/// Configuration for a secret provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SecretProviderConfig {
    /// Platform keychain (default).
    Keychain,

    /// Environment variable (for CI/CD).
    EnvironmentVariable {
        /// Prefix for environment variable names.
        prefix: String,
    },

    /// File-based secrets.
    File {
        /// Path to secrets file.
        path: String,
        /// File format.
        format: SecretFileFormat,
    },

    /// External secret manager.
    External {
        /// Provider type.
        provider_type: ExternalSecretProvider,
        /// Provider-specific configuration.
        #[serde(default)]
        config: HashMap<String, String>,
    },
}

impl SecretProviderConfig {
    /// Create a keychain provider config.
    pub fn keychain() -> Self {
        Self::Keychain
    }

    /// Create an environment variable provider config.
    pub fn environment_variable(prefix: impl Into<String>) -> Self {
        Self::EnvironmentVariable {
            prefix: prefix.into(),
        }
    }

    /// Create a file provider config.
    pub fn file(path: impl Into<String>, format: SecretFileFormat) -> Self {
        Self::File {
            path: path.into(),
            format,
        }
    }

    /// Create an external provider config.
    pub fn external(provider_type: ExternalSecretProvider) -> Self {
        Self::External {
            provider_type,
            config: HashMap::new(),
        }
    }

    /// Get the provider name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Keychain => "keychain",
            Self::EnvironmentVariable { .. } => "environment",
            Self::File { .. } => "file",
            Self::External { provider_type, .. } => provider_type.name(),
        }
    }
}

/// External secret provider type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExternalSecretProvider {
    /// HashiCorp Vault.
    Vault,
    /// AWS Secrets Manager.
    AwsSecretsManager,
    /// Google Cloud Secret Manager.
    GcpSecretManager,
    /// Azure Key Vault.
    AzureKeyVault,
    /// 1Password CLI.
    OnePassword,
    /// Doppler.
    Doppler,
}

impl ExternalSecretProvider {
    /// Get the provider name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Vault => "vault",
            Self::AwsSecretsManager => "aws-secrets-manager",
            Self::GcpSecretManager => "gcp-secret-manager",
            Self::AzureKeyVault => "azure-key-vault",
            Self::OnePassword => "1password",
            Self::Doppler => "doppler",
        }
    }

    /// Get a human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Vault => "HashiCorp Vault",
            Self::AwsSecretsManager => "AWS Secrets Manager",
            Self::GcpSecretManager => "GCP Secret Manager",
            Self::AzureKeyVault => "Azure Key Vault",
            Self::OnePassword => "1Password",
            Self::Doppler => "Doppler",
        }
    }
}

/// Secret file format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecretFileFormat {
    /// KEY=value format.
    Env,
    /// JSON object.
    Json,
    /// YAML file.
    Yaml,
    /// Single secret per file (raw content).
    Raw,
}

impl SecretFileFormat {
    /// Get the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Raw => "txt",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secrets_config_builder() {
        let config = SecretsConfig::new()
            .with_required_env_secret("api-key", "API_KEY", "API authentication key")
            .with_required_file_secret("db-password", "/run/secrets/db", "Database password");

        assert_eq!(config.secrets.len(), 2);
        assert!(config.secrets.get("api-key").unwrap().required);
        assert!(config.secrets.get("api-key").unwrap().env_var.is_some());
        assert!(config.secrets.get("db-password").unwrap().file_path.is_some());
    }

    #[test]
    fn test_secret_definition_builder() {
        let secret = SecretDefinition::required("api-key")
            .with_description("API key for authentication")
            .inject_as_env("API_KEY")
            .with_provider("keychain");

        assert!(secret.required);
        assert_eq!(secret.env_var, Some("API_KEY".to_string()));
        assert_eq!(secret.provider, Some("keychain".to_string()));
    }

    #[test]
    fn test_secret_injection_targets() {
        let secret = SecretDefinition::required("multi-target")
            .inject_as_env("SECRET_VAR")
            .write_to_file("/run/secrets/key")
            .with_file_mode("0400");

        let targets = secret.injection_targets();
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&SecretInjectionTarget::EnvVar("SECRET_VAR".to_string())));
        assert!(targets.contains(&SecretInjectionTarget::File {
            path: "/run/secrets/key".to_string(),
            mode: Some("0400".to_string()),
        }));
    }

    #[test]
    fn test_secret_provider_config() {
        let keychain = SecretProviderConfig::keychain();
        assert_eq!(keychain.name(), "keychain");

        let env = SecretProviderConfig::environment_variable("SECRET_");
        assert_eq!(env.name(), "environment");

        let file = SecretProviderConfig::file("/secrets.json", SecretFileFormat::Json);
        assert_eq!(file.name(), "file");

        let vault = SecretProviderConfig::external(ExternalSecretProvider::Vault);
        assert_eq!(vault.name(), "vault");
    }

    #[test]
    fn test_secrets_config_queries() {
        let config = SecretsConfig::new()
            .with_secret("required-key", SecretDefinition::required("required-key"))
            .with_secret("optional-key", SecretDefinition::optional("optional-key"));

        assert_eq!(config.required_keys(), vec!["required-key"]);
        assert_eq!(config.optional_keys(), vec!["optional-key"]);
        assert!(config.contains("required-key"));
        assert!(!config.contains("nonexistent"));
    }

    #[test]
    fn test_secrets_config_serialization() {
        let config = SecretsConfig::new()
            .with_required_env_secret("api-key", "API_KEY", "API key")
            .with_provider(SecretProviderConfig::keychain());

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SecretsConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.secrets.len(), deserialized.secrets.len());
        assert_eq!(config.providers.len(), deserialized.providers.len());
    }

    #[test]
    fn test_external_provider_names() {
        assert_eq!(ExternalSecretProvider::Vault.name(), "vault");
        assert_eq!(
            ExternalSecretProvider::AwsSecretsManager.display_name(),
            "AWS Secrets Manager"
        );
    }
}
