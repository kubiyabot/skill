use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

use crate::credentials::{parse_keyring_reference, CredentialStore};

/// Configuration for a skill instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceConfig {
    /// Instance metadata
    pub metadata: InstanceMetadata,

    /// Configuration key-value pairs
    pub config: HashMap<String, ConfigValue>,

    /// Environment variables to pass to the skill
    pub environment: HashMap<String, String>,

    /// Capabilities granted to this instance
    pub capabilities: Capabilities,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            metadata: InstanceMetadata::default(),
            config: HashMap::new(),
            environment: HashMap::new(),
            capabilities: Capabilities::default(),
        }
    }
}

/// Metadata about a skill instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceMetadata {
    /// Name of the skill this instance belongs to
    pub skill_name: String,
    /// Version of the skill
    pub skill_version: String,
    /// Unique name for this instance
    pub instance_name: String,
    /// Timestamp when the instance was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp when the instance was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for InstanceMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            skill_name: String::new(),
            skill_version: String::new(),
            instance_name: String::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// A configuration value with optional secret marking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValue {
    /// The configuration value (may be a keyring reference if secret=true)
    pub value: String,
    /// Whether this value is a secret stored in the keyring
    #[serde(default)]
    pub secret: bool,
}

/// Capabilities and permissions granted to a skill instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    /// Allowed filesystem paths (outside of preopened dirs)
    #[serde(default)]
    pub allowed_paths: Vec<PathBuf>,

    /// Network access permission
    #[serde(default)]
    pub network_access: bool,

    /// Maximum concurrent requests
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_requests: usize,
}

fn default_max_concurrent() -> usize {
    10
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            allowed_paths: Vec::new(),
            network_access: false,
            max_concurrent_requests: default_max_concurrent(),
        }
    }
}

impl InstanceConfig {
    /// Load instance configuration from TOML file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let contents = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: Self = toml::from_str(&contents)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save instance configuration to TOML file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        std::fs::write(path.as_ref(), contents)
            .with_context(|| format!("Failed to write config file: {}", path.as_ref().display()))?;

        Ok(())
    }

    /// Get a configuration value (non-secret values only)
    /// For secret values, use get_secret_config which returns a Zeroizing string
    pub fn get_config(&self, key: &str) -> Option<String> {
        self.config.get(key).and_then(|v| {
            if v.secret {
                None // Secret values must be retrieved via get_secret_config
            } else {
                Some(v.value.clone())
            }
        })
    }

    /// Get a secret configuration value from keyring
    /// Returns a Zeroizing string that clears memory on drop
    pub fn get_secret_config(&self, key: &str) -> Result<Option<Zeroizing<String>>> {
        if let Some(config_value) = self.config.get(key) {
            if config_value.secret {
                // Value is a keyring reference: "keyring://skill-engine/{skill}/{instance}/{key}"
                let (skill, instance, secret_key) = parse_keyring_reference(&config_value.value)?;

                let credential_store = CredentialStore::new();
                let secret = credential_store.get_credential(&skill, &instance, &secret_key)?;

                return Ok(Some(secret));
            }
        }
        Ok(None)
    }

    /// Get all configuration including resolved secrets
    /// Returns a HashMap with secret values resolved from keyring
    /// IMPORTANT: Caller must ensure returned map is zeroed after use
    pub fn get_all_config(&self) -> Result<HashMap<String, Zeroizing<String>>> {
        let mut result = HashMap::new();

        for (key, value) in &self.config {
            if value.secret {
                // Resolve from keyring
                if let Some(secret) = self.get_secret_config(key)? {
                    result.insert(key.clone(), secret);
                }
            } else {
                // Plain value
                result.insert(key.clone(), Zeroizing::new(value.value.clone()));
            }
        }

        Ok(result)
    }

    /// Set a configuration value
    pub fn set_config(&mut self, key: String, value: String, secret: bool) {
        self.config.insert(key, ConfigValue { value, secret });
        self.metadata.updated_at = chrono::Utc::now();
    }

    /// Get instance directory path
    pub fn instance_dir(skill_name: &str, instance_name: &str) -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Failed to get home directory")?;

        Ok(home
            .join(".skill-engine")
            .join("instances")
            .join(skill_name)
            .join(instance_name))
    }

    /// Get config file path for an instance
    pub fn config_path(skill_name: &str, instance_name: &str) -> Result<PathBuf> {
        Ok(Self::instance_dir(skill_name, instance_name)?.join("config.toml"))
    }

    /// Create instance directory structure
    pub fn create_instance_dir(skill_name: &str, instance_name: &str) -> Result<PathBuf> {
        let instance_dir = Self::instance_dir(skill_name, instance_name)?;
        std::fs::create_dir_all(&instance_dir)
            .with_context(|| format!("Failed to create instance directory: {}", instance_dir.display()))?;
        Ok(instance_dir)
    }
}

/// Manager for skill instances
pub struct InstanceManager {
    instances_root: PathBuf,
    credential_store: CredentialStore,
}

impl InstanceManager {
    /// Create a new instance manager
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .context("Failed to get home directory")?;

        let instances_root = home.join(".skill-engine").join("instances");
        std::fs::create_dir_all(&instances_root)?;

        Ok(Self {
            instances_root,
            credential_store: CredentialStore::new(),
        })
    }

    /// Create a new instance with configuration and secrets
    pub fn create_instance(
        &self,
        skill_name: &str,
        instance_name: &str,
        config: InstanceConfig,
        secrets: HashMap<String, String>,
    ) -> Result<()> {
        // Create instance directory
        InstanceConfig::create_instance_dir(skill_name, instance_name)?;

        // Store secrets in keyring and update config with references
        let mut updated_config = config;
        for (key, value) in secrets {
            // Store in keyring
            self.credential_store
                .store_credential(skill_name, instance_name, &key, &value)?;

            // Add keyring reference to config
            let keyring_ref =
                format!("keyring://skill-engine/{}/{}/{}", skill_name, instance_name, key);
            updated_config.config.insert(
                key,
                ConfigValue {
                    value: keyring_ref,
                    secret: true,
                },
            );
        }

        // Save config to file
        self.save_instance(skill_name, instance_name, &updated_config)?;

        tracing::info!(
            skill = %skill_name,
            instance = %instance_name,
            "Created instance"
        );

        Ok(())
    }

    /// List all instances for a skill
    pub fn list_instances(&self, skill_name: &str) -> Result<Vec<String>> {
        let skill_dir = self.instances_root.join(skill_name);

        if !skill_dir.exists() {
            return Ok(Vec::new());
        }

        let mut instances = Vec::new();

        for entry in std::fs::read_dir(&skill_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    instances.push(name.to_string());
                }
            }
        }

        Ok(instances)
    }

    /// Load instance configuration
    pub fn load_instance(&self, skill_name: &str, instance_name: &str) -> Result<InstanceConfig> {
        let config_path = InstanceConfig::config_path(skill_name, instance_name)?;
        InstanceConfig::load(config_path)
    }

    /// Save instance configuration
    pub fn save_instance(&self, skill_name: &str, instance_name: &str, config: &InstanceConfig) -> Result<()> {
        let config_path = InstanceConfig::config_path(skill_name, instance_name)?;
        config.save(config_path)
    }

    /// Delete an instance and all associated credentials
    pub fn delete_instance(&self, skill_name: &str, instance_name: &str) -> Result<()> {
        // Load config to find all secret keys
        if let Ok(config) = self.load_instance(skill_name, instance_name) {
            // Delete all credentials from keyring
            for (_key, value) in &config.config {
                if value.secret {
                    // Parse keyring reference and delete
                    if let Ok((_, _, secret_key)) = parse_keyring_reference(&value.value) {
                        let _ = self
                            .credential_store
                            .delete_credential(skill_name, instance_name, &secret_key);
                    }
                }
            }
        }

        // Delete instance directory
        let instance_dir = InstanceConfig::instance_dir(skill_name, instance_name)?;
        if instance_dir.exists() {
            std::fs::remove_dir_all(&instance_dir)
                .with_context(|| format!("Failed to delete instance directory: {}", instance_dir.display()))?;
        }

        tracing::info!(
            skill = %skill_name,
            instance = %instance_name,
            "Deleted instance and credentials"
        );

        Ok(())
    }

    /// Update a secret value for an instance
    pub fn update_secret(
        &self,
        skill_name: &str,
        instance_name: &str,
        key: &str,
        value: &str,
    ) -> Result<()> {
        self.credential_store
            .store_credential(skill_name, instance_name, key, value)?;

        tracing::debug!(
            skill = %skill_name,
            instance = %instance_name,
            key = %key,
            "Updated secret"
        );

        Ok(())
    }
}

impl Default for InstanceManager {
    fn default() -> Self {
        Self::new().expect("Failed to create InstanceManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_instance_config_serialization() {
        let mut config = InstanceConfig::default();
        config.metadata.skill_name = "test-skill".to_string();
        config.metadata.instance_name = "test-instance".to_string();
        config.set_config("key1".to_string(), "value1".to_string(), false);

        let toml = toml::to_string(&config).unwrap();
        let deserialized: InstanceConfig = toml::from_str(&toml).unwrap();

        assert_eq!(deserialized.metadata.skill_name, "test-skill");
        assert_eq!(deserialized.get_config("key1"), Some("value1".to_string()));
    }

    #[test]
    fn test_config_value() {
        let mut config = InstanceConfig::default();
        config.set_config("test".to_string(), "value".to_string(), false);

        assert_eq!(config.get_config("test"), Some("value".to_string()));
        assert_eq!(config.get_config("nonexistent"), None);
    }
}
