//! Docker Runtime - Execute skills in Docker containers
//!
//! This module provides the ability to run skills inside Docker containers,
//! enabling use of existing container images as skill backends.
//!
//! # Example Manifest Configuration
//!
//! ```toml
//! [skills.ffmpeg]
//! source = "docker:jrottenberg/ffmpeg:5-alpine"
//! runtime = "docker"
//!
//! [skills.ffmpeg.docker]
//! image = "jrottenberg/ffmpeg:5-alpine"
//! entrypoint = "/usr/local/bin/ffmpeg"
//! volumes = ["${SKILL_WORKDIR}:/workdir"]
//! working_dir = "/workdir"
//! memory = "512m"
//! cpus = "2"
//! network = "none"
//! rm = true
//! ```

use anyhow::{anyhow, Context, Result};
use std::process::Command;
use tracing::{debug, info, warn};

use crate::manifest::DockerRuntimeConfig;

/// Security constraints for Docker execution
pub struct DockerSecurityPolicy {
    /// Block privileged mode
    pub block_privileged: bool,
    /// Block docker.sock mounts
    pub block_docker_sock: bool,
    /// Block host network
    pub block_host_network: bool,
    /// Block mounting sensitive paths
    pub blocked_mount_paths: Vec<String>,
    /// Require resource limits
    pub require_resource_limits: bool,
}

impl Default for DockerSecurityPolicy {
    fn default() -> Self {
        Self {
            block_privileged: true,
            block_docker_sock: true,
            block_host_network: true,
            blocked_mount_paths: vec![
                "/etc/passwd".to_string(),
                "/etc/shadow".to_string(),
                "/var/run/docker.sock".to_string(),
                "/root".to_string(),
            ],
            require_resource_limits: false,
        }
    }
}

/// Docker runtime executor
pub struct DockerRuntime {
    policy: DockerSecurityPolicy,
}

impl DockerRuntime {
    /// Create a new Docker runtime with default security policy
    pub fn new() -> Self {
        Self {
            policy: DockerSecurityPolicy::default(),
        }
    }

    /// Create with custom security policy
    pub fn with_policy(policy: DockerSecurityPolicy) -> Self {
        Self { policy }
    }

    /// Check if Docker is available
    pub fn is_available() -> bool {
        Command::new("docker")
            .arg("version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Validate Docker configuration against security policy
    pub fn validate_config(&self, config: &DockerRuntimeConfig) -> Result<()> {
        // Check for privileged flag in extra_args
        if self.policy.block_privileged {
            if config.extra_args.iter().any(|a| a.contains("--privileged")) {
                return Err(anyhow!("Security policy blocks --privileged mode"));
            }
        }

        // Check for docker.sock mounts
        if self.policy.block_docker_sock {
            for volume in &config.volumes {
                if volume.contains("docker.sock") {
                    return Err(anyhow!("Security policy blocks mounting docker.sock"));
                }
            }
        }

        // Check for host network
        if self.policy.block_host_network && config.network == "host" {
            return Err(anyhow!("Security policy blocks host network mode"));
        }

        // Check for blocked mount paths
        for volume in &config.volumes {
            let host_path = volume.split(':').next().unwrap_or("");
            for blocked in &self.policy.blocked_mount_paths {
                if host_path.starts_with(blocked) {
                    return Err(anyhow!(
                        "Security policy blocks mounting path: {}",
                        blocked
                    ));
                }
            }
        }

        // Check resource limits if required
        if self.policy.require_resource_limits {
            if config.memory.is_none() {
                warn!("No memory limit set for Docker skill");
            }
            if config.cpus.is_none() {
                warn!("No CPU limit set for Docker skill");
            }
        }

        Ok(())
    }

    /// Build docker run command arguments
    pub fn build_command(
        &self,
        config: &DockerRuntimeConfig,
        tool_args: &[String],
    ) -> Result<Vec<String>> {
        self.validate_config(config)?;

        let mut args = vec!["run".to_string()];

        // Remove container after execution
        if config.rm {
            args.push("--rm".to_string());
        }

        // Network mode (default: none for isolation)
        args.push("--network".to_string());
        args.push(config.network.clone());

        // Memory limit
        if let Some(ref memory) = config.memory {
            args.push("--memory".to_string());
            args.push(memory.clone());
        }

        // CPU limit
        if let Some(ref cpus) = config.cpus {
            args.push("--cpus".to_string());
            args.push(cpus.clone());
        }

        // Working directory
        if let Some(ref workdir) = config.working_dir {
            args.push("--workdir".to_string());
            args.push(workdir.clone());
        }

        // User
        if let Some(ref user) = config.user {
            args.push("--user".to_string());
            args.push(user.clone());
        }

        // GPU support
        if let Some(ref gpus) = config.gpus {
            args.push("--gpus".to_string());
            args.push(gpus.clone());
        }

        // Read-only filesystem
        if config.read_only {
            args.push("--read-only".to_string());
        }

        // Platform (multi-arch)
        if let Some(ref platform) = config.platform {
            args.push("--platform".to_string());
            args.push(platform.clone());
        }

        // Volume mounts
        for volume in &config.volumes {
            args.push("-v".to_string());
            args.push(volume.clone());
        }

        // Environment variables
        for env_var in &config.environment {
            args.push("-e".to_string());
            args.push(env_var.clone());
        }

        // Extra args (validated against policy)
        for extra in &config.extra_args {
            args.push(extra.clone());
        }

        // Entrypoint override
        if let Some(ref entrypoint) = config.entrypoint {
            args.push("--entrypoint".to_string());
            args.push(entrypoint.clone());
        }

        // Image
        args.push(config.image.clone());

        // Command/args
        if let Some(ref cmd) = config.command {
            args.extend(cmd.iter().cloned());
        }

        // Additional tool arguments
        args.extend(tool_args.iter().cloned());

        Ok(args)
    }

    /// Execute a Docker container and capture output
    pub fn execute(
        &self,
        config: &DockerRuntimeConfig,
        tool_args: &[String],
    ) -> Result<DockerOutput> {
        let args = self.build_command(config, tool_args)?;

        debug!("Docker command: docker {}", args.join(" "));

        let output = Command::new("docker")
            .args(&args)
            .output()
            .context("Failed to execute docker command")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            info!("Docker container executed successfully");
            Ok(DockerOutput {
                success: true,
                stdout,
                stderr,
                exit_code: output.status.code().unwrap_or(0),
            })
        } else {
            let exit_code = output.status.code().unwrap_or(-1);
            warn!("Docker container failed with exit code {}", exit_code);
            Ok(DockerOutput {
                success: false,
                stdout,
                stderr,
                exit_code,
            })
        }
    }

    /// Pull an image if not already present
    pub fn ensure_image(&self, image: &str) -> Result<()> {
        info!("Ensuring Docker image: {}", image);

        // Check if image exists locally
        let check = Command::new("docker")
            .args(["image", "inspect", image])
            .output()
            .context("Failed to check for docker image")?;

        if check.status.success() {
            debug!("Image {} already exists locally", image);
            return Ok(());
        }

        // Pull the image
        info!("Pulling Docker image: {}", image);
        let pull = Command::new("docker")
            .args(["pull", image])
            .output()
            .context("Failed to pull docker image")?;

        if !pull.status.success() {
            let stderr = String::from_utf8_lossy(&pull.stderr);
            return Err(anyhow!("Failed to pull image {}: {}", image, stderr));
        }

        Ok(())
    }
}

impl Default for DockerRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Output from Docker container execution
#[derive(Debug, Clone)]
pub struct DockerOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_runtime_creation() {
        let runtime = DockerRuntime::new();
        assert!(runtime.policy.block_privileged);
        assert!(runtime.policy.block_docker_sock);
    }

    #[test]
    fn test_build_basic_command() {
        let runtime = DockerRuntime::new();
        let config = DockerRuntimeConfig {
            image: "alpine:latest".to_string(),
            ..Default::default()
        };

        let args = runtime.build_command(&config, &[]).unwrap();
        assert!(args.contains(&"run".to_string()));
        assert!(args.contains(&"--rm".to_string()));
        assert!(args.contains(&"--network".to_string()));
        assert!(args.contains(&"none".to_string()));
        assert!(args.contains(&"alpine:latest".to_string()));
    }

    #[test]
    fn test_build_command_with_volumes() {
        let runtime = DockerRuntime::new();
        let config = DockerRuntimeConfig {
            image: "python:3.11".to_string(),
            volumes: vec!["/tmp/data:/data".to_string()],
            working_dir: Some("/data".to_string()),
            ..Default::default()
        };

        let args = runtime.build_command(&config, &[]).unwrap();
        assert!(args.contains(&"-v".to_string()));
        assert!(args.contains(&"/tmp/data:/data".to_string()));
        assert!(args.contains(&"--workdir".to_string()));
        assert!(args.contains(&"/data".to_string()));
    }

    #[test]
    fn test_build_command_with_resources() {
        let runtime = DockerRuntime::new();
        let config = DockerRuntimeConfig {
            image: "nvidia/cuda:12.0".to_string(),
            memory: Some("4g".to_string()),
            cpus: Some("2".to_string()),
            gpus: Some("all".to_string()),
            ..Default::default()
        };

        let args = runtime.build_command(&config, &[]).unwrap();
        assert!(args.contains(&"--memory".to_string()));
        assert!(args.contains(&"4g".to_string()));
        assert!(args.contains(&"--cpus".to_string()));
        assert!(args.contains(&"2".to_string()));
        assert!(args.contains(&"--gpus".to_string()));
        assert!(args.contains(&"all".to_string()));
    }

    #[test]
    fn test_security_blocks_privileged() {
        let runtime = DockerRuntime::new();
        let config = DockerRuntimeConfig {
            image: "alpine".to_string(),
            extra_args: vec!["--privileged".to_string()],
            ..Default::default()
        };

        let result = runtime.validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("privileged"));
    }

    #[test]
    fn test_security_blocks_docker_sock() {
        let runtime = DockerRuntime::new();
        let config = DockerRuntimeConfig {
            image: "alpine".to_string(),
            volumes: vec!["/var/run/docker.sock:/var/run/docker.sock".to_string()],
            ..Default::default()
        };

        let result = runtime.validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("docker.sock"));
    }

    #[test]
    fn test_security_blocks_host_network() {
        let runtime = DockerRuntime::new();
        let config = DockerRuntimeConfig {
            image: "alpine".to_string(),
            network: "host".to_string(),
            ..Default::default()
        };

        let result = runtime.validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("host network"));
    }

    #[test]
    fn test_build_command_with_entrypoint() {
        let runtime = DockerRuntime::new();
        let config = DockerRuntimeConfig {
            image: "jrottenberg/ffmpeg:5".to_string(),
            entrypoint: Some("/usr/local/bin/ffmpeg".to_string()),
            ..Default::default()
        };

        let args = runtime.build_command(&config, &["-version".to_string()]).unwrap();
        assert!(args.contains(&"--entrypoint".to_string()));
        assert!(args.contains(&"/usr/local/bin/ffmpeg".to_string()));
        assert!(args.contains(&"-version".to_string()));
    }

    #[test]
    fn test_build_command_with_environment() {
        let runtime = DockerRuntime::new();
        let config = DockerRuntimeConfig {
            image: "node:20".to_string(),
            environment: vec!["NODE_ENV=production".to_string(), "PORT=3000".to_string()],
            ..Default::default()
        };

        let args = runtime.build_command(&config, &[]).unwrap();
        let e_count = args.iter().filter(|a| *a == "-e").count();
        assert_eq!(e_count, 2);
        assert!(args.contains(&"NODE_ENV=production".to_string()));
        assert!(args.contains(&"PORT=3000".to_string()));
    }

    #[test]
    fn test_custom_security_policy() {
        let policy = DockerSecurityPolicy {
            block_privileged: false, // Allow privileged for testing
            ..Default::default()
        };
        let runtime = DockerRuntime::with_policy(policy);
        let config = DockerRuntimeConfig {
            image: "alpine".to_string(),
            extra_args: vec!["--privileged".to_string()],
            ..Default::default()
        };

        // Should pass with relaxed policy
        assert!(runtime.validate_config(&config).is_ok());
    }
}
