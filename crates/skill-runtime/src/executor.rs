use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use wasmtime::{
    component::{Component, Linker},
    Store,
};

use crate::engine::SkillEngine;
use crate::instance::InstanceConfig;
use crate::sandbox::SandboxBuilder;
use crate::types::{ExecutionResult, SkillMetadata, ToolDefinition, Parameter, ParameterType};

// Generate WIT bindings for the skill interface
// TODO: Add host function imports for configuration access
#[allow(missing_docs)]
mod bindings {
    wasmtime::component::bindgen!({
        inline: "
            package skill-engine:skill@1.0.0;

            world skill {
                export get-metadata: func() -> string;
                export get-tools: func() -> string;
                export execute-tool: func(tool-name: string, args: string) -> string;
                export validate-config: func(config: string) -> string;
            }
        ",
        async: true,
    });
}
use bindings::*;

/// High-level executor for running skills
pub struct SkillExecutor {
    engine: Arc<SkillEngine>,
    skill_name: String,
    instance_name: String,
    config: InstanceConfig,
    component: Component,
}

impl SkillExecutor {
    /// Load a skill and prepare for execution
    pub async fn load(
        engine: Arc<SkillEngine>,
        skill_path: impl AsRef<Path>,
        skill_name: String,
        instance_name: String,
        config: InstanceConfig,
    ) -> Result<Self> {
        tracing::info!(
            skill = %skill_name,
            instance = %instance_name,
            path = %skill_path.as_ref().display(),
            "Loading skill"
        );

        let start = Instant::now();

        // Load the component
        let component = engine.load_component(skill_path.as_ref()).await?;

        // Validate the component
        engine.validate_component(&component).await?;

        let duration = start.elapsed();
        tracing::info!(
            skill = %skill_name,
            instance = %instance_name,
            duration_ms = duration.as_millis(),
            "Skill loaded successfully"
        );

        Ok(Self {
            engine,
            skill_name,
            instance_name,
            config,
            component,
        })
    }

    /// Create an executor from an already-loaded component
    pub fn from_component(
        engine: Arc<SkillEngine>,
        component: Component,
        skill_name: String,
        instance_name: String,
        config: InstanceConfig,
    ) -> Result<Self> {
        Ok(Self {
            engine,
            skill_name,
            instance_name,
            config,
            component,
        })
    }

    /// Get skill metadata
    pub async fn get_metadata(&self) -> Result<SkillMetadata> {
        // Create a store for this execution
        let instance_dir = InstanceConfig::instance_dir(&self.skill_name, &self.instance_name)?;

        let sandbox = SandboxBuilder::new(&self.instance_name, instance_dir)
            .env_from_config(&self.config)
            .build()?;

        let mut store = Store::new(self.engine.wasmtime_engine(), sandbox);

        // Create linker and instantiate component
        let mut linker = Linker::new(self.engine.wasmtime_engine());
        wasmtime_wasi::add_to_linker_async(&mut linker)?;

        let skill = Skill::instantiate_async(&mut store, &self.component, &linker).await?;

        // Call get-metadata export
        let metadata_json = skill.call_get_metadata(&mut store).await?;

        // Parse JSON metadata
        let metadata: serde_json::Value = serde_json::from_str(&metadata_json)
            .context("Failed to parse skill metadata JSON")?;

        Ok(SkillMetadata {
            name: metadata["name"].as_str().unwrap_or(&self.skill_name).to_string(),
            version: metadata["version"].as_str().unwrap_or("0.0.0").to_string(),
            description: metadata["description"].as_str().unwrap_or("").to_string(),
            author: metadata["author"].as_str().unwrap_or("").to_string(),
            repository: metadata["repository"].as_str().map(|s| s.to_string()),
            license: metadata["license"].as_str().map(|s| s.to_string()),
        })
    }

    /// Get list of tools provided by this skill
    pub async fn get_tools(&self) -> Result<Vec<ToolDefinition>> {
        // Create a store for this execution
        let instance_dir = InstanceConfig::instance_dir(&self.skill_name, &self.instance_name)?;

        let sandbox = SandboxBuilder::new(&self.instance_name, instance_dir)
            .env_from_config(&self.config)
            .build()?;

        let mut store = Store::new(self.engine.wasmtime_engine(), sandbox);

        // Create linker and instantiate component
        let mut linker = Linker::new(self.engine.wasmtime_engine());
        wasmtime_wasi::add_to_linker_async(&mut linker)?;

        let skill = Skill::instantiate_async(&mut store, &self.component, &linker).await?;

        // Call get-tools export
        let tools_json = skill.call_get_tools(&mut store).await?;

        // Parse JSON tools list
        let tools: Vec<serde_json::Value> = serde_json::from_str(&tools_json)
            .context("Failed to parse tools JSON")?;

        // Convert to ToolDefinition structs
        let mut tool_defs = Vec::new();
        let empty_params = Vec::new();
        for tool in tools {
            let params_json = tool["parameters"].as_array().unwrap_or(&empty_params);
            let mut parameters = Vec::new();

            for param in params_json {
                let param_type_str = param["paramType"].as_str().unwrap_or("string");
                let param_type = match param_type_str {
                    "number" => ParameterType::Number,
                    "boolean" => ParameterType::Boolean,
                    "file" => ParameterType::File,
                    "json" => ParameterType::Json,
                    "array" => ParameterType::Array,
                    _ => ParameterType::String,
                };

                parameters.push(Parameter {
                    name: param["name"].as_str().unwrap_or("").to_string(),
                    param_type,
                    description: param["description"].as_str().unwrap_or("").to_string(),
                    required: param["required"].as_bool().unwrap_or(false),
                    default_value: param["defaultValue"].as_str().map(|s| s.to_string()),
                });
            }

            tool_defs.push(ToolDefinition {
                name: tool["name"].as_str().unwrap_or("").to_string(),
                description: tool["description"].as_str().unwrap_or("").to_string(),
                parameters,
                streaming: false, // TODO: Support streaming tools
            });
        }

        Ok(tool_defs)
    }

    /// Execute a tool
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        args: Vec<(String, String)>,
    ) -> Result<ExecutionResult> {
        let start = Instant::now();

        tracing::info!(
            skill = %self.skill_name,
            instance = %self.instance_name,
            tool = %tool_name,
            args_count = args.len(),
            "Executing tool"
        );

        // Create sandbox environment
        let instance_dir = InstanceConfig::instance_dir(&self.skill_name, &self.instance_name)?;

        let sandbox = SandboxBuilder::new(&self.instance_name, instance_dir)
            .env_from_config(&self.config)
            .args(vec![tool_name.to_string()])
            .build()?;

        let mut store = Store::new(self.engine.wasmtime_engine(), sandbox);

        // Create linker and instantiate component
        let mut linker = Linker::new(self.engine.wasmtime_engine());
        wasmtime_wasi::add_to_linker_async(&mut linker)?;

        let skill = Skill::instantiate_async(&mut store, &self.component, &linker).await?;

        // Convert args to JSON string
        let args_json = serde_json::to_string(&serde_json::Map::from_iter(
            args.into_iter().map(|(k, v)| (k, serde_json::Value::String(v)))
        ))?;

        // Call execute-tool export
        let result_json = skill.call_execute_tool(&mut store, tool_name, args_json.as_str()).await?;

        // Parse JSON result
        let result_value: serde_json::Value = serde_json::from_str(&result_json)
            .context("Failed to parse execution result JSON")?;

        let result = if let Some(ok) = result_value.get("ok") {
            // Success case
            ExecutionResult {
                success: ok["success"].as_bool().unwrap_or(true),
                output: ok["output"].as_str().unwrap_or("").to_string(),
                error_message: ok["errorMessage"].as_str().map(|s| s.to_string()),
                metadata: None,
            }
        } else if let Some(err) = result_value.get("err") {
            // Error case
            ExecutionResult {
                success: false,
                output: String::new(),
                error_message: Some(err.as_str().unwrap_or("Unknown error").to_string()),
                metadata: None,
            }
        } else {
            // Fallback
            ExecutionResult {
                success: false,
                output: String::new(),
                error_message: Some("Invalid result format".to_string()),
                metadata: None,
            }
        };

        let duration = start.elapsed();
        tracing::info!(
            skill = %self.skill_name,
            instance = %self.instance_name,
            tool = %tool_name,
            success = result.success,
            duration_ms = duration.as_millis(),
            "Tool execution completed"
        );

        Ok(result)
    }

    /// Validate configuration
    pub async fn validate_config(&self) -> Result<()> {
        // Create a store for this execution
        let instance_dir = InstanceConfig::instance_dir(&self.skill_name, &self.instance_name)?;

        let sandbox = SandboxBuilder::new(&self.instance_name, instance_dir)
            .env_from_config(&self.config)
            .build()?;

        let mut store = Store::new(self.engine.wasmtime_engine(), sandbox);

        // Create linker and instantiate component
        let mut linker = Linker::new(self.engine.wasmtime_engine());
        wasmtime_wasi::add_to_linker_async(&mut linker)?;

        let skill = Skill::instantiate_async(&mut store, &self.component, &linker).await?;

        // Convert config to JSON string
        let config_json = serde_json::to_string(&self.config.config)?;

        // Call validate-config export
        let result_json = skill.call_validate_config(&mut store, config_json.as_str()).await?;

        // Parse result
        let result: serde_json::Value = serde_json::from_str(&result_json)
            .context("Failed to parse validate-config result")?;

        if let Some(err) = result.get("err") {
            anyhow::bail!("Configuration validation failed: {}", err.as_str().unwrap_or("Unknown error"));
        }

        Ok(())
    }

    /// Get the underlying component
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Get skill name
    pub fn skill_name(&self) -> &str {
        &self.skill_name
    }

    /// Get instance name
    pub fn instance_name(&self) -> &str {
        &self.instance_name
    }

    /// Get configuration
    pub fn config(&self) -> &InstanceConfig {
        &self.config
    }
}

/// Cache for compiled components
pub struct ComponentCache {
    cache_dir: std::path::PathBuf,
}

impl ComponentCache {
    /// Create a new component cache
    pub fn new(cache_dir: impl AsRef<Path>) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    /// Get cache key for a component
    fn cache_key(&self, skill_name: &str, version: &str) -> String {
        // Include wasmtime version in cache key (hardcoded for now)
        format!("{}_{}_wasmtime_26", skill_name, version)
    }

    /// Get cached component path
    pub fn cache_path(&self, skill_name: &str, version: &str) -> std::path::PathBuf {
        self.cache_dir.join(format!("{}.cwasm", self.cache_key(skill_name, version)))
    }

    /// Check if component is cached
    pub fn is_cached(&self, skill_name: &str, version: &str) -> bool {
        self.cache_path(skill_name, version).exists()
    }

    /// Load component from cache
    pub fn load(&self, skill_name: &str, version: &str) -> Result<Vec<u8>> {
        let path = self.cache_path(skill_name, version);
        std::fs::read(&path)
            .with_context(|| format!("Failed to read cached component: {}", path.display()))
    }

    /// Save component to cache
    pub fn save(&self, skill_name: &str, version: &str, data: &[u8]) -> Result<()> {
        let path = self.cache_path(skill_name, version);
        std::fs::write(&path, data)
            .with_context(|| format!("Failed to write cached component: {}", path.display()))?;

        tracing::debug!(
            skill = %skill_name,
            version = %version,
            size = data.len(),
            "Saved component to cache"
        );

        Ok(())
    }

    /// Clear cache for a specific skill
    pub fn clear_skill(&self, skill_name: &str) -> Result<()> {
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let filename = entry.file_name();
            if let Some(name) = filename.to_str() {
                if name.starts_with(&format!("{}_", skill_name)) {
                    std::fs::remove_file(entry.path())?;
                    tracing::debug!(file = %name, "Removed cached component");
                }
            }
        }
        Ok(())
    }

    /// Clear entire cache
    pub fn clear_all(&self) -> Result<()> {
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                std::fs::remove_file(entry.path())?;
            }
        }
        tracing::info!("Cleared component cache");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_component_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ComponentCache::new(temp_dir.path()).unwrap();

        let skill_name = "test-skill";
        let version = "1.0.0";
        let data = vec![1, 2, 3, 4, 5];

        // Should not be cached initially
        assert!(!cache.is_cached(skill_name, version));

        // Save to cache
        cache.save(skill_name, version, &data).unwrap();

        // Should be cached now
        assert!(cache.is_cached(skill_name, version));

        // Load from cache
        let loaded = cache.load(skill_name, version).unwrap();
        assert_eq!(loaded, data);

        // Clear cache
        cache.clear_all().unwrap();
        assert!(!cache.is_cached(skill_name, version));
    }

    #[test]
    fn test_cache_key_includes_wasmtime_version() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ComponentCache::new(temp_dir.path()).unwrap();

        let key = cache.cache_key("test", "1.0.0");
        assert!(key.contains("wasmtime"));
        assert!(key.contains("wasmtime_26"));
    }
}
