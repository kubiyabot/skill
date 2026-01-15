//! Skill execution context management.
//!
//! This crate provides types and utilities for defining and managing
//! execution contexts for skill-engine skills. An execution context
//! defines the complete environment in which a skill's tools execute,
//! including:
//!
//! - File and directory mounts
//! - Environment variables
//! - Secrets and credentials
//! - Resource limits (CPU, memory, network)
//! - Runtime-specific overrides
//!
//! # Core Concepts
//!
//! ## Execution Context
//!
//! An [`ExecutionContext`] is the central type that combines all configuration
//! needed to run a skill. Contexts can inherit from other contexts, allowing
//! for a hierarchy of configurations (e.g., base → development → production).
//!
//! ```rust
//! use skill_context::{ExecutionContext, EnvironmentConfig, ResourceConfig};
//!
//! let context = ExecutionContext::new("my-context", "My Context")
//!     .with_description("A production context")
//!     .with_environment(
//!         EnvironmentConfig::new()
//!             .with_var("LOG_LEVEL", "info")
//!             .with_passthrough_prefix("AWS_")
//!     )
//!     .with_resources(
//!         ResourceConfig::new()
//!             .with_memory_limit("1g")
//!             .with_network_enabled()
//!             .with_timeout(300)
//!     )
//!     .with_tag("production");
//! ```
//!
//! ## Mounts
//!
//! [`Mount`]s define files and directories that should be accessible
//! within the execution environment:
//!
//! ```rust
//! use skill_context::Mount;
//!
//! let data_mount = Mount::directory("data", "/host/data", "/app/data")
//!     .as_read_write()
//!     .with_description("Application data directory");
//!
//! let config_mount = Mount::config_file(
//!     "app-config",
//!     r#"
//!     [api]
//!     endpoint = "${API_ENDPOINT}"
//!     "#,
//!     "/etc/app/config.toml"
//! );
//! ```
//!
//! ## Secrets
//!
//! The [`SecretsConfig`] type manages secret definitions and providers:
//!
//! ```rust
//! use skill_context::{SecretsConfig, SecretDefinition};
//!
//! let secrets = SecretsConfig::new()
//!     .with_required_env_secret("api-key", "API_KEY", "API authentication key")
//!     .with_required_file_secret("db-password", "/run/secrets/db", "Database password");
//! ```
//!
//! ## Resources
//!
//! [`ResourceConfig`] defines limits and capabilities:
//!
//! ```rust
//! use skill_context::{ResourceConfig, NetworkConfig};
//!
//! let resources = ResourceConfig::new()
//!     .with_cpu_limit("2")
//!     .with_memory_limit("1g")
//!     .with_network(
//!         NetworkConfig::enabled()
//!             .allow_host("api.example.com")
//!             .allow_host("*.amazonaws.com")
//!     )
//!     .with_timeout(300);
//! ```
//!
//! # Features
//!
//! - `vault` - Enable HashiCorp Vault secret provider
//! - `aws-secrets` - Enable AWS Secrets Manager provider
//! - `azure-keyvault` - Enable Azure Key Vault provider
//! - `gcp-secrets` - Enable GCP Secret Manager provider

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod context;
pub mod environment;
pub mod inheritance;
pub mod mounts;
pub mod providers;
pub mod resources;
pub mod runtime;
pub mod secrets;
pub mod storage;

// Re-export main types at crate root
pub use context::{ContextMetadata, ExecutionContext};
pub use environment::{
    EnvFileRef, EnvValue, EnvironmentConfig, GeneratedValue, SecretRef,
};
pub use mounts::{Mount, MountType};
pub use resources::{
    CpuConfig, ExecutionLimits, FilesystemConfig, MemoryConfig, NetworkConfig,
    RateLimit, ResourceConfig,
};
pub use runtime::{DockerOverrides, NativeOverrides, RuntimeOverrides, WasmOverrides};
pub use secrets::{
    ExternalSecretProvider, SecretDefinition, SecretFileFormat, SecretInjectionTarget,
    SecretProviderConfig, SecretsConfig,
};

// Re-export inheritance types
pub use inheritance::{
    merge_environments, merge_mounts, merge_resources, merge_secrets, resolve_context,
    ContextResolver,
};

// Re-export storage types
pub use storage::{BackupInfo, ContextIndex, ContextIndexEntry, ContextStorage};

// Re-export provider types
pub use providers::{
    EnvironmentProvider, FileProvider, KeychainProvider, SecretManager, SecretProvider,
    SecretValue,
};

/// Error types for the skill-context crate.
pub mod error {
    use thiserror::Error;

    /// Errors that can occur during context operations.
    #[derive(Debug, Error)]
    pub enum ContextError {
        /// Context not found.
        #[error("Context not found: {0}")]
        NotFound(String),

        /// Context already exists.
        #[error("Context already exists: {0}")]
        AlreadyExists(String),

        /// Invalid context configuration.
        #[error("Invalid context configuration: {0}")]
        InvalidConfig(String),

        /// Circular inheritance detected.
        #[error("Circular inheritance detected: {0}")]
        CircularInheritance(String),

        /// Parent context not found.
        #[error("Parent context not found: {0}")]
        ParentNotFound(String),

        /// Secret not found.
        #[error("Secret not found: {0}")]
        SecretNotFound(String),

        /// Required secret not set.
        #[error("Required secret not set: {0}")]
        RequiredSecretNotSet(String),

        /// Mount source not found.
        #[error("Mount source not found: {0}")]
        MountSourceNotFound(String),

        /// Invalid mount configuration.
        #[error("Invalid mount configuration: {0}")]
        InvalidMount(String),

        /// IO error.
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),

        /// Serialization error.
        #[error("Serialization error: {0}")]
        Serialization(String),

        /// Secret provider error.
        #[error("Secret provider error: {0}")]
        SecretProvider(String),
    }

    impl From<serde_json::Error> for ContextError {
        fn from(e: serde_json::Error) -> Self {
            Self::Serialization(e.to_string())
        }
    }

    impl From<toml::de::Error> for ContextError {
        fn from(e: toml::de::Error) -> Self {
            Self::Serialization(e.to_string())
        }
    }

    impl From<toml::ser::Error> for ContextError {
        fn from(e: toml::ser::Error) -> Self {
            Self::Serialization(e.to_string())
        }
    }
}

pub use error::ContextError;

/// Result type for context operations.
pub type Result<T> = std::result::Result<T, ContextError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_context_creation() {
        let context = ExecutionContext::new("test-context", "Test Context")
            .with_description("A comprehensive test context")
            .with_mount(
                Mount::directory("data", "/host/data", "/app/data")
                    .as_read_write()
                    .with_description("Data directory"),
            )
            .with_mount(
                Mount::tmpfs("temp", "/tmp", 100)
                    .as_optional(),
            )
            .with_environment(
                EnvironmentConfig::new()
                    .with_var("LOG_LEVEL", "debug")
                    .with_var("APP_ENV", "test")
                    .with_passthrough_prefix("AWS_")
                    .with_passthrough_var("PATH"),
            )
            .with_secrets(
                SecretsConfig::new()
                    .with_required_env_secret("api-key", "API_KEY", "API key for auth"),
            )
            .with_resources(
                ResourceConfig::new()
                    .with_cpu_limit("2")
                    .with_memory_limit("1g")
                    .with_network_enabled()
                    .with_timeout(300),
            )
            .with_runtime_overrides(
                RuntimeOverrides::new()
                    .with_docker(
                        DockerOverrides::new()
                            .with_user("1000:1000")
                            .with_no_new_privileges(),
                    ),
            )
            .with_tag("test")
            .with_tag("comprehensive");

        // Verify structure
        assert_eq!(context.id, "test-context");
        assert_eq!(context.name, "Test Context");
        assert_eq!(context.mounts.len(), 2);
        assert_eq!(context.environment.variables.len(), 2);
        assert!(!context.secrets.is_empty());
        assert!(context.resources.cpu.is_some());
        assert!(context.resources.memory.is_some());
        assert!(context.resources.network.enabled);
        assert!(context.runtime_overrides.is_some());
        assert_eq!(context.metadata.tags.len(), 2);
    }

    #[test]
    fn test_context_inheritance_setup() {
        let base = ExecutionContext::new("base", "Base Context")
            .with_environment(
                EnvironmentConfig::new()
                    .with_var("BASE_VAR", "base_value"),
            );

        let child = ExecutionContext::inheriting("child", "Child Context", "base")
            .with_environment(
                EnvironmentConfig::new()
                    .with_var("CHILD_VAR", "child_value"),
            );

        assert!(!base.has_parent());
        assert!(child.has_parent());
        assert_eq!(child.inherits_from, Some("base".to_string()));
    }

    #[test]
    fn test_full_serialization_roundtrip() {
        let context = ExecutionContext::new("roundtrip-test", "Roundtrip Test")
            .with_mount(Mount::directory("data", "/host", "/container"))
            .with_environment(
                EnvironmentConfig::new()
                    .with_var("KEY", "value"),
            )
            .with_secrets(
                SecretsConfig::new()
                    .with_required_env_secret("secret", "SECRET_VAR", "A secret"),
            )
            .with_resources(
                ResourceConfig::new()
                    .with_memory_limit("512m"),
            );

        // JSON roundtrip
        let json = serde_json::to_string_pretty(&context).unwrap();
        let from_json: ExecutionContext = serde_json::from_str(&json).unwrap();
        assert_eq!(context.id, from_json.id);
        assert_eq!(context.mounts.len(), from_json.mounts.len());

        // TOML roundtrip
        let toml_str = toml::to_string_pretty(&context).unwrap();
        let from_toml: ExecutionContext = toml::from_str(&toml_str).unwrap();
        assert_eq!(context.id, from_toml.id);
        assert_eq!(context.mounts.len(), from_toml.mounts.len());
    }
}
