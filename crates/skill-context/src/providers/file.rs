//! File-based secret provider.
//!
//! This provider reads secrets from files in various formats:
//! - `.env` format (KEY=value)
//! - JSON format ({"key": "value"})
//! - YAML format (key: value)
//! - Raw format (single secret per file)

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use async_trait::async_trait;
use zeroize::Zeroizing;

use super::{SecretProvider, SecretValue};
use crate::secrets::SecretFileFormat;
use crate::ContextError;

/// Secret provider that reads from files.
pub struct FileProvider {
    /// Base path for secret files.
    base_path: PathBuf,
    /// File format.
    format: SecretFileFormat,
    /// Cached secrets (key -> value).
    cache: RwLock<HashMap<String, HashMap<String, String>>>,
    /// Whether to allow writes.
    allow_writes: bool,
}

impl FileProvider {
    /// Create a new file provider.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the secrets file or directory
    /// * `format` - Format of the secrets file(s)
    pub fn new(path: impl AsRef<Path>, format: SecretFileFormat) -> Result<Self, ContextError> {
        let base_path = path.as_ref().to_path_buf();

        Ok(Self {
            base_path,
            format,
            cache: RwLock::new(HashMap::new()),
            allow_writes: false,
        })
    }

    /// Allow writes to the secrets file.
    ///
    /// By default, the file provider is read-only for safety.
    pub fn with_writes(mut self) -> Self {
        self.allow_writes = true;
        self
    }

    /// Get the file path for a context's secrets.
    fn context_file(&self, context_id: &str) -> PathBuf {
        if self.base_path.is_file() {
            self.base_path.clone()
        } else {
            let ext = self.format.extension();
            self.base_path.join(format!("{}.{}", context_id, ext))
        }
    }

    /// Load secrets from a file.
    fn load_file(&self, path: &Path) -> Result<HashMap<String, String>, ContextError> {
        if !path.exists() {
            return Ok(HashMap::new());
        }

        // Check permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(path)?;
            let mode = metadata.permissions().mode();

            // Warn if file is world-readable
            if mode & 0o004 != 0 {
                tracing::warn!(
                    path = %path.display(),
                    mode = format!("{:o}", mode),
                    "Secrets file is world-readable, consider restricting permissions"
                );
            }
        }

        let content = fs::read_to_string(path)?;

        match self.format {
            SecretFileFormat::Env => self.parse_env(&content),
            SecretFileFormat::Json => self.parse_json(&content),
            SecretFileFormat::Yaml => self.parse_yaml(&content),
            SecretFileFormat::Raw => {
                // For raw format, the filename (without extension) is the key
                let key = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("secret")
                    .to_string();
                let mut map = HashMap::new();
                map.insert(key, content.trim().to_string());
                Ok(map)
            }
        }
    }

    /// Parse .env format.
    fn parse_env(&self, content: &str) -> Result<HashMap<String, String>, ContextError> {
        let mut secrets = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse KEY=value
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let mut value = value.trim().to_string();

                // Remove surrounding quotes if present
                if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    value = value[1..value.len() - 1].to_string();
                }

                secrets.insert(key, value);
            }
        }

        Ok(secrets)
    }

    /// Parse JSON format.
    fn parse_json(&self, content: &str) -> Result<HashMap<String, String>, ContextError> {
        let value: serde_json::Value = serde_json::from_str(content)?;

        let mut secrets = HashMap::new();

        if let serde_json::Value::Object(map) = value {
            for (k, v) in map {
                let string_value = match v {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                };
                secrets.insert(k, string_value);
            }
        }

        Ok(secrets)
    }

    /// Parse YAML format.
    fn parse_yaml(&self, content: &str) -> Result<HashMap<String, String>, ContextError> {
        // Use serde_json as an intermediate format since we have it as a dependency
        // A proper implementation would use serde_yaml
        let value: serde_json::Value = serde_yaml::from_str(content)
            .map_err(|e| ContextError::Serialization(e.to_string()))?;

        let mut secrets = HashMap::new();

        if let serde_json::Value::Object(map) = value {
            for (k, v) in map {
                let string_value = match v {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                };
                secrets.insert(k, string_value);
            }
        }

        Ok(secrets)
    }

    /// Save secrets to a file.
    fn save_file(&self, path: &Path, secrets: &HashMap<String, String>) -> Result<(), ContextError> {
        let content = match self.format {
            SecretFileFormat::Env => {
                secrets
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            SecretFileFormat::Json => serde_json::to_string_pretty(secrets)?,
            SecretFileFormat::Yaml => {
                serde_yaml::to_string(secrets)
                    .map_err(|e| ContextError::Serialization(e.to_string()))?
            }
            SecretFileFormat::Raw => {
                // Raw format can only store one secret
                secrets
                    .values()
                    .next()
                    .cloned()
                    .unwrap_or_default()
            }
        };

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)?;

        // Set restrictive permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    /// Get cached secrets for a context, loading from file if needed.
    fn get_cached(&self, context_id: &str) -> Result<HashMap<String, String>, ContextError> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(secrets) = cache.get(context_id) {
                return Ok(secrets.clone());
            }
        }

        // Load from file
        let file = self.context_file(context_id);
        let secrets = self.load_file(&file)?;

        // Update cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(context_id.to_string(), secrets.clone());
        }

        Ok(secrets)
    }

    /// Invalidate the cache for a context.
    fn invalidate_cache(&self, context_id: &str) {
        let mut cache = self.cache.write().unwrap();
        cache.remove(context_id);
    }
}

#[async_trait]
impl SecretProvider for FileProvider {
    async fn get_secret(
        &self,
        context_id: &str,
        key: &str,
    ) -> Result<Option<SecretValue>, ContextError> {
        let secrets = self.get_cached(context_id)?;

        Ok(secrets.get(key).map(|v| Zeroizing::new(v.clone())))
    }

    async fn set_secret(
        &self,
        context_id: &str,
        key: &str,
        value: &str,
    ) -> Result<(), ContextError> {
        if !self.allow_writes {
            return Err(ContextError::SecretProvider(
                "File provider is configured as read-only".to_string(),
            ));
        }

        let file = self.context_file(context_id);
        let mut secrets = self.get_cached(context_id)?;

        secrets.insert(key.to_string(), value.to_string());

        self.save_file(&file, &secrets)?;
        self.invalidate_cache(context_id);

        tracing::info!(
            context_id = context_id,
            key = key,
            file = %file.display(),
            "Stored secret in file"
        );

        Ok(())
    }

    async fn delete_secret(&self, context_id: &str, key: &str) -> Result<(), ContextError> {
        if !self.allow_writes {
            return Err(ContextError::SecretProvider(
                "File provider is configured as read-only".to_string(),
            ));
        }

        let file = self.context_file(context_id);
        let mut secrets = self.get_cached(context_id)?;

        if secrets.remove(key).is_some() {
            self.save_file(&file, &secrets)?;
            self.invalidate_cache(context_id);

            tracing::info!(
                context_id = context_id,
                key = key,
                file = %file.display(),
                "Deleted secret from file"
            );
        }

        Ok(())
    }

    async fn list_keys(&self, context_id: &str) -> Result<Vec<String>, ContextError> {
        let secrets = self.get_cached(context_id)?;
        Ok(secrets.keys().cloned().collect())
    }

    fn name(&self) -> &'static str {
        "file"
    }

    fn is_read_only(&self) -> bool {
        !self.allow_writes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_env_format() {
        let temp_dir = TempDir::new().unwrap();
        let secrets_file = temp_dir.path().join("test.env");

        fs::write(
            &secrets_file,
            r#"
# Comment line
API_KEY=secret123
DB_PASSWORD="quoted value"
EMPTY=
"#,
        )
        .unwrap();

        let provider = FileProvider::new(&secrets_file, SecretFileFormat::Env).unwrap();

        let api_key = provider.get_secret("test", "API_KEY").await.unwrap();
        assert_eq!(&*api_key.unwrap(), "secret123");

        let db_pass = provider.get_secret("test", "DB_PASSWORD").await.unwrap();
        assert_eq!(&*db_pass.unwrap(), "quoted value");

        let empty = provider.get_secret("test", "EMPTY").await.unwrap();
        assert_eq!(&*empty.unwrap(), "");

        let missing = provider.get_secret("test", "MISSING").await.unwrap();
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn test_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let secrets_file = temp_dir.path().join("secrets.json");

        fs::write(
            &secrets_file,
            r#"{"api_key": "secret123", "number": 42, "nested": {"key": "value"}}"#,
        )
        .unwrap();

        let provider = FileProvider::new(&secrets_file, SecretFileFormat::Json).unwrap();

        let api_key = provider.get_secret("secrets", "api_key").await.unwrap();
        assert_eq!(&*api_key.unwrap(), "secret123");

        let number = provider.get_secret("secrets", "number").await.unwrap();
        assert_eq!(&*number.unwrap(), "42");
    }

    #[tokio::test]
    async fn test_yaml_format() {
        let temp_dir = TempDir::new().unwrap();
        let secrets_file = temp_dir.path().join("secrets.yaml");

        fs::write(
            &secrets_file,
            r#"
api_key: secret123
db_password: "quoted value"
"#,
        )
        .unwrap();

        let provider = FileProvider::new(&secrets_file, SecretFileFormat::Yaml).unwrap();

        let api_key = provider.get_secret("secrets", "api_key").await.unwrap();
        assert_eq!(&*api_key.unwrap(), "secret123");
    }

    #[tokio::test]
    async fn test_directory_mode() {
        let temp_dir = TempDir::new().unwrap();

        let provider = FileProvider::new(temp_dir.path(), SecretFileFormat::Env).unwrap();

        // Create a context file
        let ctx_file = temp_dir.path().join("my-context.env");
        fs::write(&ctx_file, "SECRET_KEY=value123").unwrap();

        let secret = provider.get_secret("my-context", "SECRET_KEY").await.unwrap();
        assert_eq!(&*secret.unwrap(), "value123");
    }

    #[tokio::test]
    async fn test_write_secrets() {
        let temp_dir = TempDir::new().unwrap();
        let secrets_file = temp_dir.path().join("writable.env");

        let provider = FileProvider::new(&secrets_file, SecretFileFormat::Env)
            .unwrap()
            .with_writes();

        // Set a secret
        provider.set_secret("writable", "NEW_KEY", "new_value").await.unwrap();

        // Read it back
        let secret = provider.get_secret("writable", "NEW_KEY").await.unwrap();
        assert_eq!(&*secret.unwrap(), "new_value");

        // Delete it
        provider.delete_secret("writable", "NEW_KEY").await.unwrap();

        // Verify deleted
        let secret = provider.get_secret("writable", "NEW_KEY").await.unwrap();
        assert!(secret.is_none());
    }

    #[tokio::test]
    async fn test_read_only_mode() {
        let temp_dir = TempDir::new().unwrap();
        let secrets_file = temp_dir.path().join("readonly.env");
        fs::write(&secrets_file, "").unwrap();

        let provider = FileProvider::new(&secrets_file, SecretFileFormat::Env).unwrap();

        // Should fail to write
        let result = provider.set_secret("readonly", "KEY", "value").await;
        assert!(result.is_err());

        assert!(provider.is_read_only());
    }

    #[tokio::test]
    async fn test_list_keys() {
        let temp_dir = TempDir::new().unwrap();
        let secrets_file = temp_dir.path().join("list.env");

        fs::write(&secrets_file, "KEY1=v1\nKEY2=v2\nKEY3=v3").unwrap();

        let provider = FileProvider::new(&secrets_file, SecretFileFormat::Env).unwrap();

        let keys = provider.list_keys("list").await.unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"KEY1".to_string()));
        assert!(keys.contains(&"KEY2".to_string()));
        assert!(keys.contains(&"KEY3".to_string()));
    }

    #[tokio::test]
    async fn test_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent.env");

        let provider = FileProvider::new(&nonexistent, SecretFileFormat::Env).unwrap();

        // Should return None, not error
        let result = provider.get_secret("nonexistent", "KEY").await.unwrap();
        assert!(result.is_none());

        // List should return empty
        let keys = provider.list_keys("nonexistent").await.unwrap();
        assert!(keys.is_empty());
    }
}
