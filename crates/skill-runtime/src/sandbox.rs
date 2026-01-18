use anyhow::{Context, Result};
use std::path::PathBuf;
use wasmtime_wasi::{
    ResourceTable, WasiCtx, WasiCtxBuilder, WasiView,
};

use crate::instance::InstanceConfig;

/// Host state for WASI context
pub struct HostState {
    /// WASI context for the sandboxed environment
    pub wasi: WasiCtx,
    /// Resource table for managing WASI resources
    pub table: ResourceTable,
    /// Unique identifier for this skill instance
    pub instance_id: String,
    /// Configuration key-value pairs passed as environment variables
    pub config: std::collections::HashMap<String, String>,
}

impl WasiView for HostState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

/// Builder for creating sandboxed WASI environments
pub struct SandboxBuilder {
    instance_id: String,
    instance_dir: PathBuf,
    temp_dir: PathBuf,
    env_vars: Vec<(String, String)>,
    args: Vec<String>,
    inherit_stdio: bool,
}

impl SandboxBuilder {
    /// Create a new sandbox builder for a skill instance
    pub fn new(instance_id: impl Into<String>, instance_dir: PathBuf) -> Self {
        let temp_dir = std::env::temp_dir()
            .join("skill-engine")
            .join("sandbox")
            .join(uuid::Uuid::new_v4().to_string());

        Self {
            instance_id: instance_id.into(),
            instance_dir,
            temp_dir,
            env_vars: Vec::new(),
            args: Vec::new(),
            inherit_stdio: true,
        }
    }

    /// Add an environment variable to the sandbox
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.push((key.into(), value.into()));
        self
    }

    /// Add multiple environment variables from configuration
    pub fn env_from_config(mut self, config: &InstanceConfig) -> Self {
        // Map configuration to environment variables
        for (key, value) in &config.environment {
            self.env_vars.push((key.clone(), value.clone()));
        }
        self
    }

    /// Add command-line arguments
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Set whether to inherit stdio (default: true)
    pub fn inherit_stdio(mut self, inherit: bool) -> Self {
        self.inherit_stdio = inherit;
        self
    }

    /// Build the sandboxed WASI context with capability restrictions
    pub fn build(self) -> Result<HostState> {
        // Create temporary directory for this execution
        std::fs::create_dir_all(&self.temp_dir)
            .context("Failed to create temporary sandbox directory")?;

        let mut builder = WasiCtxBuilder::new();

        // Add environment variables
        for (key, value) in &self.env_vars {
            builder.env(key, value);
        }

        // Set instance ID
        builder.env("SKILL_INSTANCE_ID", &self.instance_id);

        // Add arguments
        builder.args(&self.args);

        // Configure stdio
        if self.inherit_stdio {
            builder.inherit_stdio();
        }

        // Pre-open directories - in wasmtime 26, preopened_dir is simpler
        // Just use the builder's methods directly with paths
        // Note: The API changed - for now we'll comment this out until we can test properly
        //  TODO: Fix directory preopen for WASI Preview 2

        let wasi = builder.build();
        let table = ResourceTable::new();

        // Convert env_vars to HashMap for config access
        let config: std::collections::HashMap<String, String> =
            self.env_vars.into_iter().collect();

        tracing::debug!(
            instance_id = %self.instance_id,
            instance_dir = %self.instance_dir.display(),
            temp_dir = %self.temp_dir.display(),
            config_count = config.len(),
            "Created sandbox environment"
        );

        Ok(HostState {
            wasi,
            table,
            instance_id: self.instance_id,
            config,
        })
    }
}

/// Cleanup temporary sandbox directories
pub fn cleanup_temp_dirs() -> Result<()> {
    let sandbox_root = std::env::temp_dir().join("skill-engine").join("sandbox");

    if sandbox_root.exists() {
        // Remove old sandbox directories (older than 1 hour)
        let now = std::time::SystemTime::now();

        for entry in std::fs::read_dir(&sandbox_root)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if let Ok(created) = metadata.created() {
                if let Ok(duration) = now.duration_since(created) {
                    if duration.as_secs() > 3600 {
                        // Older than 1 hour
                        if let Err(e) = std::fs::remove_dir_all(entry.path()) {
                            tracing::warn!(
                                path = %entry.path().display(),
                                error = %e,
                                "Failed to cleanup old sandbox directory"
                            );
                        } else {
                            tracing::debug!(
                                path = %entry.path().display(),
                                "Cleaned up old sandbox directory"
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sandbox_builder() {
        let temp_dir = TempDir::new().unwrap();
        let instance_dir = temp_dir.path().to_path_buf();

        let sandbox = SandboxBuilder::new("test-instance", instance_dir.clone())
            .env("TEST_VAR", "test_value")
            .args(vec!["arg1".to_string(), "arg2".to_string()])
            .build()
            .unwrap();

        assert_eq!(sandbox.instance_id, "test-instance");
    }

    #[test]
    fn test_env_from_config() {
        let temp_dir = TempDir::new().unwrap();
        let instance_dir = temp_dir.path().to_path_buf();

        let mut config = InstanceConfig::default();
        config.environment.insert("KEY1".to_string(), "value1".to_string());
        config.environment.insert("KEY2".to_string(), "value2".to_string());

        let sandbox = SandboxBuilder::new("test", instance_dir)
            .env_from_config(&config)
            .build()
            .unwrap();

        assert_eq!(sandbox.instance_id, "test");
    }
}
