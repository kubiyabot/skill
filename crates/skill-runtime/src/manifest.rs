//! Declarative skill manifest for stateless environments.
//!
//! This module provides support for `.skill-engine.toml` files that declare
//! skills and their configurations. This is useful for:
//!
//! - CI/CD pipelines where no persistent state exists
//! - Checking skill configurations into version control
//! - Sharing skill setups across teams
//! - Reproducible environments
//!
//! # Example `.skill-engine.toml`
//!
//! ```toml
//! # Skills configuration manifest
//! # This file can be checked into version control
//!
//! [skills.hello]
//! # Local skill from path
//! source = "./examples/hello-skill"
//!
//! [skills.github-ops]
//! # Skill from GitHub
//! source = "github:org/skill-github@v1.0.0"
//!
//! [skills.aws]
//! # Skill from git with explicit ref
//! source = "https://github.com/example/skill-aws.git"
//! ref = "main"
//!
//! # Instance configurations for aws skill
//! [skills.aws.instances.prod]
//! config.region = "us-east-1"
//! config.profile = "${AWS_PROFILE}"  # Environment variable reference
//! env.AWS_ACCESS_KEY_ID = "${AWS_ACCESS_KEY_ID}"
//! env.AWS_SECRET_ACCESS_KEY = "${AWS_SECRET_ACCESS_KEY}"
//! capabilities.network_access = true
//!
//! [skills.aws.instances.dev]
//! config.region = "us-west-2"
//! config.profile = "dev"
//! capabilities.network_access = true
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::instance::{Capabilities, ConfigValue, InstanceConfig, InstanceMetadata};

/// Runtime type for skill execution
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SkillRuntime {
    /// WebAssembly runtime (default)
    #[default]
    Wasm,
    /// Docker container runtime
    Docker,
    /// Native command execution (SKILL.md-based)
    Native,
}

/// Docker runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerRuntimeConfig {
    /// Docker image to use (e.g., "python:3.11-slim", "jrottenberg/ffmpeg:5-alpine")
    pub image: String,

    /// Container entrypoint (overrides image default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,

    /// Command to run (overrides image CMD)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,

    /// Volume mounts in "host:container" format
    /// Supports env var expansion: "${SKILL_WORKDIR}:/workdir"
    #[serde(default)]
    pub volumes: Vec<String>,

    /// Working directory inside container
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,

    /// Environment variables (KEY=value format)
    #[serde(default)]
    pub environment: Vec<String>,

    /// Memory limit (e.g., "512m", "1g", "2048m")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,

    /// CPU limit (e.g., "0.5", "2", "1.5")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<String>,

    /// Network mode: none, bridge, host (default: none for security)
    #[serde(default = "default_network")]
    pub network: String,

    /// Remove container after execution (default: true)
    #[serde(default = "default_true")]
    pub rm: bool,

    /// User to run as (uid:gid format, e.g., "1000:1000" or "node")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// GPU access ("all" or device IDs like "0,1")
    /// Requires nvidia-container-runtime
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpus: Option<String>,

    /// Read-only root filesystem (default: false)
    #[serde(default)]
    pub read_only: bool,

    /// Platform for multi-arch images (e.g., "linux/amd64", "linux/arm64")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,

    /// Additional docker run arguments (advanced use)
    #[serde(default)]
    pub extra_args: Vec<String>,
}

fn default_network() -> String {
    "none".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for DockerRuntimeConfig {
    fn default() -> Self {
        Self {
            image: String::new(),
            entrypoint: None,
            command: None,
            volumes: Vec::new(),
            working_dir: None,
            environment: Vec::new(),
            memory: None,
            cpus: None,
            network: default_network(),
            rm: true,
            user: None,
            gpus: None,
            read_only: false,
            platform: None,
            extra_args: Vec::new(),
        }
    }
}

/// Root manifest structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillManifest {
    /// Manifest version (for future compatibility)
    #[serde(default = "default_version")]
    pub version: String,

    /// Global defaults applied to all skills
    #[serde(default)]
    pub defaults: ManifestDefaults,

    /// Skill definitions
    #[serde(default)]
    pub skills: HashMap<String, SkillDefinition>,

    /// Base directory for resolving relative paths (set during load)
    #[serde(skip)]
    pub base_dir: PathBuf,
}

fn default_version() -> String {
    "1".to_string()
}

/// Global defaults for all skills
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManifestDefaults {
    /// Default capabilities for all instances
    #[serde(default)]
    pub capabilities: ManifestCapabilities,

    /// Default environment variables for all instances
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Host service requirement for a skill
///
/// Skills can declare dependencies on host services (like kubectl-proxy)
/// that must be running for the skill to function properly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequirement {
    /// Service name (e.g., "kubectl-proxy")
    pub name: String,

    /// Human-readable description of what this service provides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// If true, the service enhances functionality but isn't required
    /// If false (default), the skill won't work properly without this service
    #[serde(default)]
    pub optional: bool,

    /// Default port the service runs on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_port: Option<u16>,
}

/// Skill definition in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    /// Skill source: local path, git URL, registry reference, or docker image
    /// Examples:
    ///   - "./path/to/skill"
    ///   - "github:user/repo"
    ///   - "github:user/repo@v1.0.0"
    ///   - "https://github.com/user/repo.git"
    ///   - "docker:python:3.11-slim" (for docker runtime)
    pub source: String,

    /// Runtime type for this skill (wasm, docker, or native)
    #[serde(default)]
    pub runtime: SkillRuntime,

    /// Git ref (branch, tag, commit) - only for git sources
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,

    /// Description of this skill
    pub description: Option<String>,

    /// Docker runtime configuration (required when runtime = "docker")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docker: Option<DockerRuntimeConfig>,

    /// Instance configurations for this skill
    #[serde(default)]
    pub instances: HashMap<String, InstanceDefinition>,

    /// Default instance name (defaults to "default")
    #[serde(default = "default_instance_name")]
    pub default_instance: String,

    /// Host services this skill requires (e.g., kubectl-proxy)
    #[serde(default)]
    pub services: Vec<ServiceRequirement>,
}

fn default_instance_name() -> String {
    "default".to_string()
}

/// Instance definition within a skill
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstanceDefinition {
    /// Configuration values (supports ${ENV_VAR} syntax)
    #[serde(default)]
    pub config: HashMap<String, String>,

    /// Environment variables (supports ${ENV_VAR} syntax)
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Capabilities for this instance
    #[serde(default)]
    pub capabilities: ManifestCapabilities,

    /// Description of this instance
    pub description: Option<String>,
}

/// Capabilities in manifest format
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManifestCapabilities {
    /// Allow network access
    #[serde(default)]
    pub network_access: bool,

    /// Allowed filesystem paths
    #[serde(default)]
    pub allowed_paths: Vec<String>,

    /// Max concurrent requests
    pub max_concurrent_requests: Option<usize>,
}

impl SkillManifest {
    /// Load manifest from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest file: {}", path.display()))?;

        let mut manifest = Self::parse(&content)?;

        // Set base_dir to the manifest file's parent directory
        manifest.base_dir = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        // Canonicalize if possible
        if let Ok(canonical) = std::fs::canonicalize(&manifest.base_dir) {
            manifest.base_dir = canonical;
        }

        Ok(manifest)
    }

    /// Parse manifest from TOML string
    pub fn parse(content: &str) -> Result<Self> {
        toml::from_str(content).context("Failed to parse manifest TOML")
    }

    /// Find manifest file in current or parent directories
    pub fn find(start_dir: &Path) -> Option<PathBuf> {
        let mut current = start_dir.to_path_buf();

        loop {
            // Check for .skill-engine.toml
            let manifest_path = current.join(".skill-engine.toml");
            if manifest_path.exists() {
                return Some(manifest_path);
            }

            // Check for skill-engine.toml
            let alt_path = current.join("skill-engine.toml");
            if alt_path.exists() {
                return Some(alt_path);
            }

            // Move to parent directory
            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Get all skill names defined in the manifest
    pub fn skill_names(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }

    /// Get skill definition by name
    pub fn get_skill(&self, name: &str) -> Option<&SkillDefinition> {
        self.skills.get(name)
    }

    /// Resolve a skill's instance configuration
    ///
    /// This expands environment variable references and merges with defaults.
    pub fn resolve_instance(
        &self,
        skill_name: &str,
        instance_name: Option<&str>,
    ) -> Result<ResolvedInstance> {
        let skill = self
            .skills
            .get(skill_name)
            .with_context(|| format!("Skill '{}' not found in manifest", skill_name))?;

        let instance_name = instance_name.unwrap_or(&skill.default_instance);

        // Get instance definition, or create empty one if using default
        let instance_def = skill
            .instances
            .get(instance_name)
            .cloned()
            .unwrap_or_default();

        // Build resolved config
        let mut config = HashMap::new();
        for (key, value) in &instance_def.config {
            config.insert(
                key.clone(),
                ConfigValue {
                    value: expand_env_vars(value)?,
                    secret: is_likely_secret(key),
                },
            );
        }

        // Build resolved environment
        let mut environment = HashMap::new();

        // Add global defaults first
        for (key, value) in &self.defaults.env {
            environment.insert(key.clone(), expand_env_vars(value)?);
        }

        // Add instance-specific env vars (override defaults)
        for (key, value) in &instance_def.env {
            environment.insert(key.clone(), expand_env_vars(value)?);
        }

        // Build capabilities
        let capabilities = Capabilities {
            network_access: instance_def.capabilities.network_access
                || self.defaults.capabilities.network_access,
            allowed_paths: instance_def
                .capabilities
                .allowed_paths
                .iter()
                .chain(self.defaults.capabilities.allowed_paths.iter())
                .map(|p| PathBuf::from(expand_env_vars(p).unwrap_or_default()))
                .collect(),
            max_concurrent_requests: instance_def
                .capabilities
                .max_concurrent_requests
                .or(self.defaults.capabilities.max_concurrent_requests)
                .unwrap_or(10),
        };

        // Resolve relative paths against base_dir
        let resolved_source = if skill.source.starts_with("./") || skill.source.starts_with("../") {
            self.base_dir.join(&skill.source).to_string_lossy().to_string()
        } else {
            skill.source.clone()
        };

        // Resolve Docker config with env var expansion
        let docker_config = if let Some(ref docker) = skill.docker {
            Some(DockerRuntimeConfig {
                image: expand_env_vars(&docker.image)?,
                entrypoint: docker.entrypoint.clone(),
                command: docker.command.clone(),
                volumes: docker
                    .volumes
                    .iter()
                    .map(|v| expand_env_vars(v))
                    .collect::<Result<Vec<_>>>()?,
                working_dir: docker.working_dir.clone(),
                environment: docker
                    .environment
                    .iter()
                    .map(|e| expand_env_vars(e))
                    .collect::<Result<Vec<_>>>()?,
                memory: docker.memory.clone(),
                cpus: docker.cpus.clone(),
                network: docker.network.clone(),
                rm: docker.rm,
                user: docker.user.clone(),
                gpus: docker.gpus.clone(),
                read_only: docker.read_only,
                platform: docker.platform.clone(),
                extra_args: docker.extra_args.clone(),
            })
        } else {
            None
        };

        Ok(ResolvedInstance {
            skill_name: skill_name.to_string(),
            instance_name: instance_name.to_string(),
            source: resolved_source,
            git_ref: skill.git_ref.clone(),
            config: InstanceConfig {
                metadata: InstanceMetadata {
                    skill_name: skill_name.to_string(),
                    skill_version: String::new(),
                    instance_name: instance_name.to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                },
                config,
                environment,
                capabilities,
            },
            runtime: skill.runtime.clone(),
            docker: docker_config,
        })
    }

    /// List all skills with their resolved sources
    pub fn list_skills(&self) -> Vec<SkillInfo> {
        self.skills
            .iter()
            .map(|(name, def)| SkillInfo {
                name: name.clone(),
                source: def.source.clone(),
                description: def.description.clone(),
                instances: def.instances.keys().cloned().collect(),
                default_instance: def.default_instance.clone(),
                runtime: def.runtime.clone(),
            })
            .collect()
    }
}

/// Resolved instance ready for execution
#[derive(Debug, Clone)]
pub struct ResolvedInstance {
    /// Name of the skill
    pub skill_name: String,
    /// Name of the instance
    pub instance_name: String,
    /// Resolved source path or URL
    pub source: String,
    /// Git ref (branch, tag, or commit) if applicable
    pub git_ref: Option<String>,
    /// Instance configuration with expanded values
    pub config: InstanceConfig,
    /// Runtime type (wasm, docker, or native)
    pub runtime: SkillRuntime,
    /// Docker configuration (when runtime = docker)
    pub docker: Option<DockerRuntimeConfig>,
}

/// Summary info about a skill
#[derive(Debug, Clone)]
pub struct SkillInfo {
    /// Skill name
    pub name: String,
    /// Skill source path or URL
    pub source: String,
    /// Optional description
    pub description: Option<String>,
    /// List of instance names
    pub instances: Vec<String>,
    /// Default instance name
    pub default_instance: String,
    /// Runtime type (wasm, docker, or native)
    pub runtime: SkillRuntime,
}

/// Expand environment variable references in a string.
///
/// Supports formats:
/// - `${VAR}` - Required env var, errors if not set
/// - `${VAR:-default}` - With default value
/// - `${VAR:?error message}` - Required with custom error
pub fn expand_env_vars(input: &str) -> Result<String> {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'

            let mut var_expr = String::new();
            let mut depth = 1;

            for c in chars.by_ref() {
                if c == '{' {
                    depth += 1;
                    var_expr.push(c);
                } else if c == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    var_expr.push(c);
                } else {
                    var_expr.push(c);
                }
            }

            // Parse the variable expression
            let value = if let Some(pos) = var_expr.find(":-") {
                // ${VAR:-default}
                let var_name = &var_expr[..pos];
                let default_value = &var_expr[pos + 2..];
                std::env::var(var_name).unwrap_or_else(|_| default_value.to_string())
            } else if let Some(pos) = var_expr.find(":?") {
                // ${VAR:?error}
                let var_name = &var_expr[..pos];
                let error_msg = &var_expr[pos + 2..];
                std::env::var(var_name)
                    .with_context(|| format!("Environment variable {} not set: {}", var_name, error_msg))?
            } else {
                // ${VAR}
                std::env::var(&var_expr)
                    .with_context(|| format!("Environment variable {} not set", var_expr))?
            };

            result.push_str(&value);
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

/// Check if a config key is likely a secret
fn is_likely_secret(key: &str) -> bool {
    let key_lower = key.to_lowercase();
    key_lower.contains("secret")
        || key_lower.contains("password")
        || key_lower.contains("token")
        || key_lower.contains("key")
        || key_lower.contains("credential")
        || key_lower.contains("auth")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let toml = r#"
            version = "1"

            [skills.hello]
            source = "./examples/hello-skill"

            [skills.aws]
            source = "github:example/aws-skill@v1.0.0"
            description = "AWS operations skill"

            [skills.aws.instances.prod]
            config.region = "us-east-1"
            capabilities.network_access = true

            [skills.aws.instances.dev]
            config.region = "us-west-2"
        "#;

        let manifest = SkillManifest::parse(toml).unwrap();
        assert_eq!(manifest.skills.len(), 2);
        assert!(manifest.skills.contains_key("hello"));
        assert!(manifest.skills.contains_key("aws"));

        let aws = &manifest.skills["aws"];
        assert_eq!(aws.source, "github:example/aws-skill@v1.0.0");
        assert_eq!(aws.instances.len(), 2);
    }

    #[test]
    fn test_expand_env_vars() {
        std::env::set_var("TEST_VAR", "hello");

        assert_eq!(expand_env_vars("${TEST_VAR}").unwrap(), "hello");
        assert_eq!(expand_env_vars("prefix_${TEST_VAR}_suffix").unwrap(), "prefix_hello_suffix");
        assert_eq!(expand_env_vars("${MISSING:-default}").unwrap(), "default");
        assert!(expand_env_vars("${MISSING}").is_err());
        assert!(expand_env_vars("${MISSING:?custom error}").is_err());

        std::env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_is_likely_secret() {
        assert!(is_likely_secret("api_key"));
        assert!(is_likely_secret("AWS_SECRET_ACCESS_KEY"));
        assert!(is_likely_secret("password"));
        assert!(is_likely_secret("auth_token"));
        assert!(!is_likely_secret("region"));
        assert!(!is_likely_secret("bucket_name"));
    }

    #[test]
    fn test_parse_docker_runtime_skill() {
        let toml = r#"
            version = "1"

            [skills.ffmpeg]
            source = "docker:jrottenberg/ffmpeg:5-alpine"
            runtime = "docker"
            description = "FFmpeg video processing"

            [skills.ffmpeg.docker]
            image = "jrottenberg/ffmpeg:5-alpine"
            entrypoint = "/usr/local/bin/ffmpeg"
            volumes = ["/workdir:/workdir"]
            working_dir = "/workdir"
            memory = "512m"
            cpus = "2"
            network = "none"
            rm = true
            read_only = true
        "#;

        let manifest = SkillManifest::parse(toml).unwrap();
        assert!(manifest.skills.contains_key("ffmpeg"));

        let ffmpeg = &manifest.skills["ffmpeg"];
        assert_eq!(ffmpeg.runtime, SkillRuntime::Docker);
        assert!(ffmpeg.docker.is_some());

        let docker = ffmpeg.docker.as_ref().unwrap();
        assert_eq!(docker.image, "jrottenberg/ffmpeg:5-alpine");
        assert_eq!(docker.entrypoint, Some("/usr/local/bin/ffmpeg".to_string()));
        assert_eq!(docker.memory, Some("512m".to_string()));
        assert_eq!(docker.cpus, Some("2".to_string()));
        assert_eq!(docker.network, "none");
        assert!(docker.rm);
        assert!(docker.read_only);
    }

    #[test]
    fn test_skill_runtime_default() {
        let toml = r#"
            [skills.hello]
            source = "./examples/hello-skill"
        "#;

        let manifest = SkillManifest::parse(toml).unwrap();
        let hello = &manifest.skills["hello"];
        assert_eq!(hello.runtime, SkillRuntime::Wasm);
    }

    #[test]
    fn test_native_runtime_skill() {
        let toml = r#"
            [skills.kubernetes]
            source = "./examples/kubernetes-skill"
            runtime = "native"
            description = "Kubernetes management"
        "#;

        let manifest = SkillManifest::parse(toml).unwrap();
        let k8s = &manifest.skills["kubernetes"];
        assert_eq!(k8s.runtime, SkillRuntime::Native);
    }

    #[test]
    fn test_docker_config_defaults() {
        let config = DockerRuntimeConfig::default();
        assert_eq!(config.network, "none");
        assert!(config.rm);
        assert!(!config.read_only);
        assert!(config.volumes.is_empty());
        assert!(config.environment.is_empty());
    }

    #[test]
    fn test_docker_with_env_expansion() {
        std::env::set_var("TEST_WORKDIR", "/tmp/test");
        std::env::set_var("TEST_IMAGE", "alpine:latest");

        let toml = r#"
            [skills.test]
            source = "docker:${TEST_IMAGE}"
            runtime = "docker"

            [skills.test.docker]
            image = "${TEST_IMAGE}"
            volumes = ["${TEST_WORKDIR}:/workdir"]
        "#;

        let manifest = SkillManifest::parse(toml).unwrap();
        let resolved = manifest.resolve_instance("test", None).unwrap();

        assert_eq!(resolved.runtime, SkillRuntime::Docker);
        let docker = resolved.docker.as_ref().unwrap();
        assert_eq!(docker.image, "alpine:latest");
        assert_eq!(docker.volumes, vec!["/tmp/test:/workdir"]);

        std::env::remove_var("TEST_WORKDIR");
        std::env::remove_var("TEST_IMAGE");
    }

    #[test]
    fn test_docker_with_gpu() {
        let toml = r#"
            [skills.ml]
            source = "docker:nvidia/cuda:12.0-runtime"
            runtime = "docker"

            [skills.ml.docker]
            image = "nvidia/cuda:12.0-runtime"
            gpus = "all"
            memory = "8g"
        "#;

        let manifest = SkillManifest::parse(toml).unwrap();
        let ml = &manifest.skills["ml"];
        let docker = ml.docker.as_ref().unwrap();
        assert_eq!(docker.gpus, Some("all".to_string()));
        assert_eq!(docker.memory, Some("8g".to_string()));
    }

    #[test]
    fn test_docker_extra_args() {
        let toml = r#"
            [skills.custom]
            source = "docker:myimage"
            runtime = "docker"

            [skills.custom.docker]
            image = "myimage:latest"
            extra_args = ["--cap-add=SYS_PTRACE", "--security-opt=seccomp=unconfined"]
        "#;

        let manifest = SkillManifest::parse(toml).unwrap();
        let docker = manifest.skills["custom"].docker.as_ref().unwrap();
        assert_eq!(docker.extra_args.len(), 2);
        assert!(docker.extra_args.contains(&"--cap-add=SYS_PTRACE".to_string()));
    }
}
