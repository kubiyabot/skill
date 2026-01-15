//! Mount configuration types.
//!
//! This module defines file and directory mount specifications
//! for execution contexts.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// File/directory mount specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Mount {
    /// Unique identifier within context.
    pub id: String,

    /// Mount type.
    pub mount_type: MountType,

    /// Host path or source (supports env var expansion like `${HOME}`).
    pub source: String,

    /// Path inside execution environment.
    pub target: String,

    /// Read-only flag.
    #[serde(default)]
    pub read_only: bool,

    /// Required or optional.
    #[serde(default = "default_required")]
    pub required: bool,

    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_required() -> bool {
    true
}

impl Mount {
    /// Create a new directory mount.
    pub fn directory(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            mount_type: MountType::Directory,
            source: source.into(),
            target: target.into(),
            read_only: false,
            required: true,
            description: None,
        }
    }

    /// Create a new file mount.
    pub fn file(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            mount_type: MountType::File,
            source: source.into(),
            target: target.into(),
            read_only: true,
            required: true,
            description: None,
        }
    }

    /// Create a named volume mount (Docker).
    pub fn volume(id: impl Into<String>, name: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            mount_type: MountType::Volume,
            source: name.into(),
            target: target.into(),
            read_only: false,
            required: true,
            description: None,
        }
    }

    /// Create a tmpfs mount.
    pub fn tmpfs(id: impl Into<String>, target: impl Into<String>, size_mb: u32) -> Self {
        Self {
            id: id.into(),
            mount_type: MountType::Tmpfs { size_mb },
            source: String::new(),
            target: target.into(),
            read_only: false,
            required: true,
            description: None,
        }
    }

    /// Create a config file mount from a template.
    pub fn config_file(
        id: impl Into<String>,
        template: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            mount_type: MountType::ConfigFile {
                template: template.into(),
            },
            source: String::new(),
            target: target.into(),
            read_only: true,
            required: true,
            description: None,
        }
    }

    /// Set the mount as read-only.
    pub fn as_read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    /// Set the mount as read-write.
    pub fn as_read_write(mut self) -> Self {
        self.read_only = false;
        self
    }

    /// Set the mount as optional.
    pub fn as_optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Set the mount as required.
    pub fn as_required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Add a description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Expand environment variables in the source path.
    ///
    /// Supports `${VAR}` and `$VAR` syntax.
    pub fn expand_source(&self) -> String {
        expand_env_vars(&self.source)
    }

    /// Get the source as a PathBuf with environment variables expanded.
    pub fn source_path(&self) -> PathBuf {
        PathBuf::from(self.expand_source())
    }

    /// Get the target as a PathBuf.
    pub fn target_path(&self) -> PathBuf {
        PathBuf::from(&self.target)
    }

    /// Check if this mount requires a source path to exist.
    pub fn requires_source(&self) -> bool {
        matches!(
            self.mount_type,
            MountType::File | MountType::Directory | MountType::Volume
        )
    }
}

/// Type of mount.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum MountType {
    /// Regular file.
    File,

    /// Directory.
    Directory,

    /// Docker volume (named volume).
    Volume,

    /// Temporary filesystem (tmpfs).
    Tmpfs {
        /// Size in megabytes.
        size_mb: u32,
    },

    /// Config file generated from template.
    ConfigFile {
        /// Template content with variable substitution.
        template: String,
    },
}

impl MountType {
    /// Check if this is a file mount.
    pub fn is_file(&self) -> bool {
        matches!(self, MountType::File)
    }

    /// Check if this is a directory mount.
    pub fn is_directory(&self) -> bool {
        matches!(self, MountType::Directory)
    }

    /// Check if this is a volume mount.
    pub fn is_volume(&self) -> bool {
        matches!(self, MountType::Volume)
    }

    /// Check if this is a tmpfs mount.
    pub fn is_tmpfs(&self) -> bool {
        matches!(self, MountType::Tmpfs { .. })
    }

    /// Check if this is a config file mount.
    pub fn is_config_file(&self) -> bool {
        matches!(self, MountType::ConfigFile { .. })
    }

    /// Get the display name for this mount type.
    pub fn display_name(&self) -> &'static str {
        match self {
            MountType::File => "File",
            MountType::Directory => "Directory",
            MountType::Volume => "Volume",
            MountType::Tmpfs { .. } => "Tmpfs",
            MountType::ConfigFile { .. } => "Config File",
        }
    }
}

/// Expand environment variables in a string.
///
/// Supports:
/// - `${VAR}` - Required variable
/// - `${VAR:-default}` - Variable with default value
/// - `$VAR` - Simple variable reference
fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();

    // Match ${VAR:-default} pattern
    let re_default = regex::Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*):-([^}]*)\}").unwrap();
    result = re_default
        .replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            let default = &caps[2];
            std::env::var(var_name).unwrap_or_else(|_| default.to_string())
        })
        .to_string();

    // Match ${VAR} pattern
    let re_braced = regex::Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
    result = re_braced
        .replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            std::env::var(var_name).unwrap_or_default()
        })
        .to_string();

    // Match $VAR pattern (simple, no braces)
    let re_simple = regex::Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
    result = re_simple
        .replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            std::env::var(var_name).unwrap_or_default()
        })
        .to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_mount() {
        let mount = Mount::directory("data", "/host/data", "/container/data")
            .as_read_write()
            .with_description("Data directory");

        assert_eq!(mount.id, "data");
        assert!(mount.mount_type.is_directory());
        assert!(!mount.read_only);
        assert!(mount.required);
    }

    #[test]
    fn test_file_mount() {
        let mount = Mount::file("config", "/etc/config.json", "/app/config.json").as_read_only();

        assert!(mount.mount_type.is_file());
        assert!(mount.read_only);
    }

    #[test]
    fn test_tmpfs_mount() {
        let mount = Mount::tmpfs("temp", "/tmp", 100);

        assert!(mount.mount_type.is_tmpfs());
        if let MountType::Tmpfs { size_mb } = mount.mount_type {
            assert_eq!(size_mb, 100);
        }
    }

    #[test]
    fn test_config_file_mount() {
        let template = r#"
[api]
endpoint = "${API_ENDPOINT}"
key = "${API_KEY}"
"#;
        let mount = Mount::config_file("api-config", template, "/etc/app/config.toml");

        assert!(mount.mount_type.is_config_file());
        if let MountType::ConfigFile { template: t } = &mount.mount_type {
            assert!(t.contains("${API_ENDPOINT}"));
        }
    }

    #[test]
    fn test_env_var_expansion() {
        std::env::set_var("TEST_VAR", "test_value");

        assert_eq!(expand_env_vars("${TEST_VAR}"), "test_value");
        assert_eq!(expand_env_vars("$TEST_VAR"), "test_value");
        assert_eq!(
            expand_env_vars("/path/${TEST_VAR}/file"),
            "/path/test_value/file"
        );

        std::env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_env_var_default() {
        std::env::remove_var("NONEXISTENT_VAR");

        assert_eq!(
            expand_env_vars("${NONEXISTENT_VAR:-default_value}"),
            "default_value"
        );
    }

    #[test]
    fn test_mount_serialization() {
        let mount = Mount::directory("data", "/host/data", "/container/data")
            .as_read_only()
            .as_optional();

        let json = serde_json::to_string(&mount).unwrap();
        let deserialized: Mount = serde_json::from_str(&json).unwrap();

        assert_eq!(mount.id, deserialized.id);
        assert_eq!(mount.read_only, deserialized.read_only);
        assert_eq!(mount.required, deserialized.required);
    }

    #[test]
    fn test_mount_type_display() {
        assert_eq!(MountType::File.display_name(), "File");
        assert_eq!(MountType::Directory.display_name(), "Directory");
        assert_eq!(MountType::Volume.display_name(), "Volume");
        assert_eq!(MountType::Tmpfs { size_mb: 100 }.display_name(), "Tmpfs");
        assert_eq!(
            MountType::ConfigFile {
                template: String::new()
            }
            .display_name(),
            "Config File"
        );
    }
}
