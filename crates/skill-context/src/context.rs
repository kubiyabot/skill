//! Core execution context types.
//!
//! This module defines the main `ExecutionContext` struct which represents
//! a complete execution environment for skill tools.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::environment::EnvironmentConfig;
use crate::mounts::Mount;
use crate::resources::ResourceConfig;
use crate::runtime::RuntimeOverrides;
use crate::secrets::SecretsConfig;

/// A complete execution environment definition.
///
/// Execution contexts define everything needed to run a skill tool:
/// - File and directory mounts
/// - Environment variables
/// - Secrets and credentials
/// - Resource limits
/// - Runtime-specific overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExecutionContext {
    /// Unique identifier for this context.
    pub id: String,

    /// Human-readable name.
    pub name: String,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional parent context to inherit from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherits_from: Option<String>,

    /// File and directory mounts.
    #[serde(default)]
    pub mounts: Vec<Mount>,

    /// Environment variable definitions.
    #[serde(default)]
    pub environment: EnvironmentConfig,

    /// Secret references.
    #[serde(default)]
    pub secrets: SecretsConfig,

    /// Resource limits and capabilities.
    #[serde(default)]
    pub resources: ResourceConfig,

    /// Runtime-specific overrides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_overrides: Option<RuntimeOverrides>,

    /// Metadata.
    #[serde(default)]
    pub metadata: ContextMetadata,
}

impl ExecutionContext {
    /// Create a new execution context with the given ID and name.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            inherits_from: None,
            mounts: Vec::new(),
            environment: EnvironmentConfig::default(),
            secrets: SecretsConfig::default(),
            resources: ResourceConfig::default(),
            runtime_overrides: None,
            metadata: ContextMetadata::new(),
        }
    }

    /// Create a context that inherits from another context.
    pub fn inheriting(
        id: impl Into<String>,
        name: impl Into<String>,
        parent_id: impl Into<String>,
    ) -> Self {
        let mut ctx = Self::new(id, name);
        ctx.inherits_from = Some(parent_id.into());
        ctx
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a mount.
    pub fn with_mount(mut self, mount: Mount) -> Self {
        self.mounts.push(mount);
        self
    }

    /// Set environment configuration.
    pub fn with_environment(mut self, environment: EnvironmentConfig) -> Self {
        self.environment = environment;
        self
    }

    /// Set secrets configuration.
    pub fn with_secrets(mut self, secrets: SecretsConfig) -> Self {
        self.secrets = secrets;
        self
    }

    /// Set resource configuration.
    pub fn with_resources(mut self, resources: ResourceConfig) -> Self {
        self.resources = resources;
        self
    }

    /// Set runtime overrides.
    pub fn with_runtime_overrides(mut self, overrides: RuntimeOverrides) -> Self {
        self.runtime_overrides = Some(overrides);
        self
    }

    /// Add a tag to the context.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.metadata.tags.push(tag.into());
        self
    }

    /// Check if this context inherits from another.
    pub fn has_parent(&self) -> bool {
        self.inherits_from.is_some()
    }

    /// Get all required secret keys.
    pub fn required_secrets(&self) -> Vec<&str> {
        self.secrets
            .secrets
            .iter()
            .filter(|(_, def)| def.required)
            .map(|(key, _)| key.as_str())
            .collect()
    }

    /// Get all required mounts.
    pub fn required_mounts(&self) -> Vec<&Mount> {
        self.mounts.iter().filter(|m| m.required).collect()
    }

    /// Update the metadata timestamps.
    pub fn touch(&mut self) {
        self.metadata.updated_at = Utc::now();
        self.metadata.version += 1;
    }
}

/// Metadata about an execution context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ContextMetadata {
    /// When the context was created.
    pub created_at: DateTime<Utc>,

    /// When the context was last modified.
    pub updated_at: DateTime<Utc>,

    /// Who created this context (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,

    /// Tags for categorization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Version number (incremented on each update).
    #[serde(default = "default_version")]
    pub version: u32,
}

fn default_version() -> u32 {
    1
}

impl ContextMetadata {
    /// Create new metadata with current timestamps.
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            created_by: None,
            tags: Vec::new(),
            version: 1,
        }
    }

    /// Create metadata with a creator.
    pub fn with_creator(mut self, creator: impl Into<String>) -> Self {
        self.created_by = Some(creator.into());
        self
    }
}

impl Default for ContextMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = ExecutionContext::new("test-ctx", "Test Context")
            .with_description("A test context")
            .with_tag("test")
            .with_tag("development");

        assert_eq!(ctx.id, "test-ctx");
        assert_eq!(ctx.name, "Test Context");
        assert_eq!(ctx.description, Some("A test context".to_string()));
        assert_eq!(ctx.metadata.tags, vec!["test", "development"]);
        assert!(!ctx.has_parent());
    }

    #[test]
    fn test_context_inheritance() {
        let ctx = ExecutionContext::inheriting("child-ctx", "Child Context", "parent-ctx");

        assert!(ctx.has_parent());
        assert_eq!(ctx.inherits_from, Some("parent-ctx".to_string()));
    }

    #[test]
    fn test_context_serialization() {
        let ctx = ExecutionContext::new("test", "Test");
        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: ExecutionContext = serde_json::from_str(&json).unwrap();

        assert_eq!(ctx.id, deserialized.id);
        assert_eq!(ctx.name, deserialized.name);
    }

    #[test]
    fn test_context_toml_serialization() {
        let ctx = ExecutionContext::new("test", "Test")
            .with_description("Test context")
            .with_tag("test");

        let toml_str = toml::to_string_pretty(&ctx).unwrap();
        let deserialized: ExecutionContext = toml::from_str(&toml_str).unwrap();

        assert_eq!(ctx.id, deserialized.id);
        assert_eq!(ctx.description, deserialized.description);
    }

    #[test]
    fn test_metadata_versioning() {
        let mut ctx = ExecutionContext::new("test", "Test");
        assert_eq!(ctx.metadata.version, 1);

        ctx.touch();
        assert_eq!(ctx.metadata.version, 2);

        ctx.touch();
        assert_eq!(ctx.metadata.version, 3);
    }
}
