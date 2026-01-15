use anyhow::{Context, Result};
use std::collections::HashMap;
use wasmtime_wasi::WasiCtxBuilder;
use zeroize::Zeroizing;

use crate::instance::{InstanceConfig, InstanceManager};

/// Maps instance configuration to environment variables for WASM execution
pub struct ConfigMapper {
    instance_manager: InstanceManager,
}

impl ConfigMapper {
    /// Create a new config mapper
    pub fn new(instance_manager: InstanceManager) -> Self {
        Self { instance_manager }
    }

    /// Resolve configuration including secrets from keyring
    /// Returns a HashMap of environment variables ready for injection
    pub async fn resolve_config(
        &self,
        skill_name: &str,
        instance_name: &str,
    ) -> Result<HashMap<String, Zeroizing<String>>> {
        tracing::debug!(
            skill = %skill_name,
            instance = %instance_name,
            "Resolving instance configuration"
        );

        // Load instance config
        let config = self
            .instance_manager
            .load_instance(skill_name, instance_name)
            .with_context(|| {
                format!(
                    "Failed to load instance: {}/{}",
                    skill_name, instance_name
                )
            })?;

        // Get all config including resolved secrets
        let resolved = config.get_all_config()?;

        // Merge with explicit environment variables from config
        let mut env_vars = resolved;
        for (key, value) in &config.environment {
            env_vars.insert(key.clone(), Zeroizing::new(value.clone()));
        }

        tracing::debug!(
            skill = %skill_name,
            instance = %instance_name,
            var_count = env_vars.len(),
            "Resolved configuration"
        );

        Ok(env_vars)
    }

    /// Apply environment variables to WASI context builder
    /// Converts all keys to SKILL_{KEY_NAME_UPPER} format
    pub fn apply_to_wasi_context(
        &self,
        ctx_builder: &mut WasiCtxBuilder,
        env_vars: HashMap<String, Zeroizing<String>>,
    ) -> Result<()> {
        for (key, value) in env_vars {
            // Convert to SKILL_ prefix and uppercase
            let env_key = Self::to_env_var_name(&key);

            // Add to WASI context
            ctx_builder.env(&env_key, value.as_str());

            tracing::trace!(key = %env_key, "Added environment variable");
        }

        Ok(())
    }

    /// Convert config key to environment variable name
    /// Example: "aws_access_key_id" -> "SKILL_AWS_ACCESS_KEY_ID"
    fn to_env_var_name(key: &str) -> String {
        format!("SKILL_{}", key.to_uppercase())
    }

    /// Get redacted environment map for logging (secrets replaced with [REDACTED])
    pub fn get_redacted_env_map(
        config: &InstanceConfig,
    ) -> HashMap<String, String> {
        let mut result = HashMap::new();

        for (key, value) in &config.config {
            if value.secret {
                result.insert(key.clone(), "[REDACTED]".to_string());
            } else {
                result.insert(key.clone(), value.value.clone());
            }
        }

        for (key, value) in &config.environment {
            result.insert(key.clone(), value.clone());
        }

        result
    }

    /// Support config value templating with environment variable substitution
    /// Example: "region = ${AWS_REGION:-us-east-1}" -> "us-east-1" (if AWS_REGION not set)
    pub fn expand_template(template: &str) -> String {
        let mut result = template.to_string();

        // Simple regex-free implementation for ${VAR:-default} syntax
        while let Some(start) = result.find("${") {
            if let Some(end) = result[start..].find('}') {
                let end = start + end;
                let expr = &result[start + 2..end];

                let value = if let Some(sep_pos) = expr.find(":-") {
                    let var_name = &expr[..sep_pos];
                    let default_value = &expr[sep_pos + 2..];

                    std::env::var(var_name).unwrap_or_else(|_| default_value.to_string())
                } else {
                    std::env::var(expr).unwrap_or_default()
                };

                result.replace_range(start..=end, &value);
            } else {
                break;
            }
        }

        result
    }
}

impl Default for ConfigMapper {
    fn default() -> Self {
        Self::new(InstanceManager::new().expect("Failed to create InstanceManager"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_env_var_name() {
        assert_eq!(
            ConfigMapper::to_env_var_name("aws_access_key_id"),
            "SKILL_AWS_ACCESS_KEY_ID"
        );
        assert_eq!(ConfigMapper::to_env_var_name("region"), "SKILL_REGION");
        assert_eq!(
            ConfigMapper::to_env_var_name("max_retries"),
            "SKILL_MAX_RETRIES"
        );
    }

    #[test]
    fn test_expand_template() {
        // Test with environment variable set
        std::env::set_var("TEST_VAR", "test_value");
        assert_eq!(ConfigMapper::expand_template("${TEST_VAR}"), "test_value");
        std::env::remove_var("TEST_VAR");

        // Test with default value
        assert_eq!(
            ConfigMapper::expand_template("${MISSING_VAR:-default}"),
            "default"
        );

        // Test with no template
        assert_eq!(ConfigMapper::expand_template("plain_text"), "plain_text");

        // Test with multiple variables
        std::env::set_var("VAR1", "value1");
        std::env::set_var("VAR2", "value2");
        assert_eq!(
            ConfigMapper::expand_template("${VAR1}-${VAR2}"),
            "value1-value2"
        );
        std::env::remove_var("VAR1");
        std::env::remove_var("VAR2");
    }

    #[test]
    fn test_redacted_env_map() {
        let mut config = InstanceConfig::default();
        config.set_config("public_key".to_string(), "public_value".to_string(), false);
        config.set_config(
            "secret_key".to_string(),
            "keyring://ref".to_string(),
            true,
        );

        let redacted = ConfigMapper::get_redacted_env_map(&config);

        assert_eq!(redacted.get("public_key"), Some(&"public_value".to_string()));
        assert_eq!(redacted.get("secret_key"), Some(&"[REDACTED]".to_string()));
    }
}
