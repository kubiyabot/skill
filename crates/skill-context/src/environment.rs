//! Environment variable configuration types.
//!
//! This module defines environment variable configuration and value types
//! for execution contexts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Environment variable configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EnvironmentConfig {
    /// Static environment variables.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub variables: HashMap<String, EnvValue>,

    /// Environment files to load (.env format).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env_files: Vec<EnvFileRef>,

    /// Environment variable prefixes to pass through from host.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub passthrough_prefixes: Vec<String>,

    /// Specific host env vars to pass through.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub passthrough_vars: Vec<String>,
}

impl EnvironmentConfig {
    /// Create a new empty environment configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a plain text variable.
    pub fn with_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables
            .insert(key.into(), EnvValue::Plain(value.into()));
        self
    }

    /// Add a variable reference.
    pub fn with_reference(mut self, key: impl Into<String>, ref_var: impl Into<String>) -> Self {
        self.variables
            .insert(key.into(), EnvValue::Reference(ref_var.into()));
        self
    }

    /// Add a secret reference.
    pub fn with_secret(mut self, key: impl Into<String>, secret_ref: SecretRef) -> Self {
        self.variables
            .insert(key.into(), EnvValue::Secret(secret_ref));
        self
    }

    /// Add an environment file to load.
    pub fn with_env_file(mut self, path: impl Into<String>) -> Self {
        self.env_files.push(EnvFileRef {
            path: path.into(),
            required: true,
            prefix: None,
        });
        self
    }

    /// Add an optional environment file.
    pub fn with_optional_env_file(mut self, path: impl Into<String>) -> Self {
        self.env_files.push(EnvFileRef {
            path: path.into(),
            required: false,
            prefix: None,
        });
        self
    }

    /// Add a passthrough prefix.
    pub fn with_passthrough_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.passthrough_prefixes.push(prefix.into());
        self
    }

    /// Add a specific passthrough variable.
    pub fn with_passthrough_var(mut self, var: impl Into<String>) -> Self {
        self.passthrough_vars.push(var.into());
        self
    }

    /// Get all variable keys.
    pub fn variable_keys(&self) -> Vec<&str> {
        self.variables.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a variable is a secret reference.
    pub fn is_secret(&self, key: &str) -> bool {
        self.variables
            .get(key)
            .map(|v| matches!(v, EnvValue::Secret(_)))
            .unwrap_or(false)
    }

    /// Get all secret references.
    pub fn secret_refs(&self) -> Vec<(&str, &SecretRef)> {
        self.variables
            .iter()
            .filter_map(|(k, v)| match v {
                EnvValue::Secret(r) => Some((k.as_str(), r)),
                _ => None,
            })
            .collect()
    }
}

/// Environment variable value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum EnvValue {
    /// Plain text value.
    Plain(String),

    /// Reference to another env var: `${VAR_NAME}`.
    Reference(String),

    /// Reference to a secret: `secret://context/key`.
    Secret(SecretRef),

    /// Generated value (e.g., UUID, timestamp).
    Generated(GeneratedValue),

    /// Value from file.
    FromFile(PathBuf),
}

impl EnvValue {
    /// Create a plain text value.
    pub fn plain(value: impl Into<String>) -> Self {
        Self::Plain(value.into())
    }

    /// Create a reference to another env var.
    pub fn reference(var_name: impl Into<String>) -> Self {
        Self::Reference(var_name.into())
    }

    /// Create a secret reference.
    pub fn secret(context_id: impl Into<String>, key: impl Into<String>) -> Self {
        Self::Secret(SecretRef::new(context_id, key))
    }

    /// Create a generated UUID value.
    pub fn uuid() -> Self {
        Self::Generated(GeneratedValue::Uuid)
    }

    /// Create a generated timestamp value.
    pub fn timestamp() -> Self {
        Self::Generated(GeneratedValue::Timestamp)
    }

    /// Create a value from file.
    pub fn from_file(path: impl Into<PathBuf>) -> Self {
        Self::FromFile(path.into())
    }

    /// Check if this is a plain value.
    pub fn is_plain(&self) -> bool {
        matches!(self, Self::Plain(_))
    }

    /// Check if this is a secret reference.
    pub fn is_secret(&self) -> bool {
        matches!(self, Self::Secret(_))
    }

    /// Check if this requires resolution.
    pub fn needs_resolution(&self) -> bool {
        !matches!(self, Self::Plain(_))
    }
}

/// Reference to a secret value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct SecretRef {
    /// Context ID containing the secret (use "." for current context).
    pub context_id: String,

    /// Secret key name.
    pub key: String,
}

impl SecretRef {
    /// Create a new secret reference.
    pub fn new(context_id: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            context_id: context_id.into(),
            key: key.into(),
        }
    }

    /// Create a reference to a secret in the current context.
    pub fn current(key: impl Into<String>) -> Self {
        Self::new(".", key)
    }

    /// Check if this references the current context.
    pub fn is_current_context(&self) -> bool {
        self.context_id == "."
    }

    /// Parse a secret reference string like `secret://context/key`.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.strip_prefix("secret://")?;
        let parts: Vec<&str> = s.splitn(2, '/').collect();
        if parts.len() == 2 {
            Some(Self::new(parts[0], parts[1]))
        } else {
            None
        }
    }

    /// Convert to a secret reference string.
    pub fn to_uri(&self) -> String {
        format!("secret://{}/{}", self.context_id, self.key)
    }
}

/// Generated value type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "generator")]
pub enum GeneratedValue {
    /// Generate a UUID v4.
    Uuid,

    /// Generate current timestamp (ISO 8601).
    Timestamp,

    /// Generate a random string.
    RandomString {
        /// Length of the random string.
        length: usize,
    },

    /// Generate a hash of another value.
    Hash {
        /// Hash algorithm (sha256, blake3, etc.).
        algorithm: String,
        /// Value to hash (can be a variable reference).
        of: String,
    },
}

impl GeneratedValue {
    /// Create a random string generator.
    pub fn random_string(length: usize) -> Self {
        Self::RandomString { length }
    }

    /// Create a hash generator.
    pub fn hash(algorithm: impl Into<String>, of: impl Into<String>) -> Self {
        Self::Hash {
            algorithm: algorithm.into(),
            of: of.into(),
        }
    }

    /// Generate the value.
    pub fn generate(&self) -> String {
        match self {
            Self::Uuid => uuid::Uuid::new_v4().to_string(),
            Self::Timestamp => chrono::Utc::now().to_rfc3339(),
            Self::RandomString { length } => {
                use std::iter;
                const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
                let mut rng = rand_simple();
                iter::repeat_with(|| CHARSET[rng.next() % CHARSET.len()])
                    .map(|c| c as char)
                    .take(*length)
                    .collect()
            }
            Self::Hash { algorithm, of } => {
                // For now, just use a simple hash representation
                // In production, this would use the actual hash algorithm
                format!("{}:{}", algorithm, of)
            }
        }
    }
}

// Simple random number generator for random strings
struct SimpleRng(u64);

impl SimpleRng {
    fn next(&mut self) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.0 >> 33) as usize
    }
}

fn rand_simple() -> SimpleRng {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    SimpleRng(seed)
}

/// Reference to an environment file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EnvFileRef {
    /// Path to .env file (supports glob patterns).
    pub path: String,

    /// Whether file must exist.
    #[serde(default = "default_true")]
    pub required: bool,

    /// Optional prefix to add to all vars from this file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
}

fn default_true() -> bool {
    true
}

impl EnvFileRef {
    /// Create a new required env file reference.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            required: true,
            prefix: None,
        }
    }

    /// Create an optional env file reference.
    pub fn optional(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            required: false,
            prefix: None,
        }
    }

    /// Set a prefix for all variables from this file.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_config_builder() {
        let config = EnvironmentConfig::new()
            .with_var("LOG_LEVEL", "debug")
            .with_reference("API_URL", "PRODUCTION_API_URL")
            .with_passthrough_prefix("AWS_")
            .with_passthrough_var("PATH");

        assert_eq!(config.variables.len(), 2);
        assert!(config.passthrough_prefixes.contains(&"AWS_".to_string()));
        assert!(config.passthrough_vars.contains(&"PATH".to_string()));
    }

    #[test]
    fn test_env_value_types() {
        let plain = EnvValue::plain("value");
        let reference = EnvValue::reference("OTHER_VAR");
        let secret = EnvValue::secret("my-context", "api-key");
        let uuid = EnvValue::uuid();

        assert!(plain.is_plain());
        assert!(!plain.needs_resolution());

        assert!(!reference.is_plain());
        assert!(reference.needs_resolution());

        assert!(secret.is_secret());
        assert!(secret.needs_resolution());

        assert!(uuid.needs_resolution());
    }

    #[test]
    fn test_secret_ref_parsing() {
        let ref1 = SecretRef::parse("secret://my-context/api-key").unwrap();
        assert_eq!(ref1.context_id, "my-context");
        assert_eq!(ref1.key, "api-key");
        assert!(!ref1.is_current_context());

        let ref2 = SecretRef::parse("secret://./local-key").unwrap();
        assert!(ref2.is_current_context());

        assert!(SecretRef::parse("invalid").is_none());
        assert!(SecretRef::parse("other://scheme").is_none());
    }

    #[test]
    fn test_secret_ref_uri() {
        let secret_ref = SecretRef::new("ctx", "key");
        assert_eq!(secret_ref.to_uri(), "secret://ctx/key");
    }

    #[test]
    fn test_generated_value() {
        let uuid = GeneratedValue::Uuid.generate();
        assert_eq!(uuid.len(), 36); // UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx

        let timestamp = GeneratedValue::Timestamp.generate();
        assert!(timestamp.contains("T")); // ISO 8601 format

        let random = GeneratedValue::random_string(10).generate();
        assert_eq!(random.len(), 10);
    }

    #[test]
    fn test_env_config_serialization() {
        let config = EnvironmentConfig::new()
            .with_var("KEY", "value")
            .with_secret("SECRET_KEY", SecretRef::current("my-secret"));

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: EnvironmentConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.variables.len(), deserialized.variables.len());
    }

    #[test]
    fn test_env_file_ref() {
        let required = EnvFileRef::new(".env.production");
        assert!(required.required);

        let optional = EnvFileRef::optional(".env.local").with_prefix("LOCAL_");
        assert!(!optional.required);
        assert_eq!(optional.prefix, Some("LOCAL_".to_string()));
    }
}
