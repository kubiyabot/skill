//! Runtime-specific override types.
//!
//! This module defines runtime-specific configuration overrides
//! for WASM, Docker, and native execution environments.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime-specific overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeOverrides {
    /// WASM-specific configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wasm: Option<WasmOverrides>,

    /// Docker-specific configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docker: Option<DockerOverrides>,

    /// Native execution configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native: Option<NativeOverrides>,
}

impl RuntimeOverrides {
    /// Create new empty runtime overrides.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set WASM overrides.
    pub fn with_wasm(mut self, wasm: WasmOverrides) -> Self {
        self.wasm = Some(wasm);
        self
    }

    /// Set Docker overrides.
    pub fn with_docker(mut self, docker: DockerOverrides) -> Self {
        self.docker = Some(docker);
        self
    }

    /// Set native overrides.
    pub fn with_native(mut self, native: NativeOverrides) -> Self {
        self.native = Some(native);
        self
    }

    /// Check if any runtime overrides are set.
    pub fn is_empty(&self) -> bool {
        self.wasm.is_none() && self.docker.is_none() && self.native.is_none()
    }
}

/// WASM runtime overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WasmOverrides {
    /// Stack size in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack_size: Option<usize>,

    /// Enable/disable specific WASI capabilities.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub wasi_capabilities: HashMap<String, bool>,

    /// Fuel limit for execution metering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fuel_limit: Option<u64>,

    /// Enable epoch-based interruption.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epoch_interruption: Option<bool>,

    /// Memory pages limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_memory_pages: Option<u32>,

    /// Enable debug info.
    #[serde(default)]
    pub debug_info: bool,
}

impl WasmOverrides {
    /// Create new WASM overrides.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set stack size.
    pub fn with_stack_size(mut self, size: usize) -> Self {
        self.stack_size = Some(size);
        self
    }

    /// Set a WASI capability.
    pub fn with_wasi_capability(mut self, capability: impl Into<String>, enabled: bool) -> Self {
        self.wasi_capabilities.insert(capability.into(), enabled);
        self
    }

    /// Enable a WASI capability.
    pub fn enable_capability(self, capability: impl Into<String>) -> Self {
        self.with_wasi_capability(capability, true)
    }

    /// Disable a WASI capability.
    pub fn disable_capability(self, capability: impl Into<String>) -> Self {
        self.with_wasi_capability(capability, false)
    }

    /// Set fuel limit.
    pub fn with_fuel_limit(mut self, limit: u64) -> Self {
        self.fuel_limit = Some(limit);
        self
    }

    /// Enable epoch interruption.
    pub fn with_epoch_interruption(mut self) -> Self {
        self.epoch_interruption = Some(true);
        self
    }

    /// Set max memory pages.
    pub fn with_max_memory_pages(mut self, pages: u32) -> Self {
        self.max_memory_pages = Some(pages);
        self
    }

    /// Enable debug info.
    pub fn with_debug_info(mut self) -> Self {
        self.debug_info = true;
        self
    }

    /// Check if a WASI capability is enabled.
    pub fn is_capability_enabled(&self, capability: &str) -> Option<bool> {
        self.wasi_capabilities.get(capability).copied()
    }

    /// Get stack size in bytes, with a default.
    pub fn stack_size_or_default(&self) -> usize {
        self.stack_size.unwrap_or(1024 * 1024) // 1MB default
    }
}

/// Docker runtime overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DockerOverrides {
    /// Override container image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Additional docker run arguments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<String>,

    /// Override entrypoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,

    /// Override command.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,

    /// User to run as (uid:gid).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// GPU configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpus: Option<String>,

    /// Platform for multi-arch support.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,

    /// Privileged mode (dangerous!).
    #[serde(default)]
    pub privileged: bool,

    /// Security options.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security_opt: Vec<String>,

    /// Sysctls.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub sysctls: HashMap<String, String>,

    /// Container labels.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,

    /// Restart policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restart: Option<String>,

    /// Remove container after execution.
    #[serde(default = "default_true")]
    pub rm: bool,

    /// Init process.
    #[serde(default)]
    pub init: bool,

    /// Hostname.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,

    /// IPC mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipc: Option<String>,

    /// PID mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<String>,

    /// Capabilities to add.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cap_add: Vec<String>,

    /// Capabilities to drop.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cap_drop: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl Default for DockerOverrides {
    fn default() -> Self {
        Self {
            image: None,
            extra_args: Vec::new(),
            entrypoint: None,
            command: None,
            user: None,
            gpus: None,
            platform: None,
            privileged: false,
            security_opt: Vec::new(),
            sysctls: HashMap::new(),
            labels: HashMap::new(),
            restart: None,
            rm: true, // Default to removing container after execution
            init: false,
            hostname: None,
            ipc: None,
            pid: None,
            cap_add: Vec::new(),
            cap_drop: Vec::new(),
        }
    }
}

impl DockerOverrides {
    /// Create new Docker overrides.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set container image.
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// Add extra docker run argument.
    pub fn with_extra_arg(mut self, arg: impl Into<String>) -> Self {
        self.extra_args.push(arg.into());
        self
    }

    /// Set entrypoint.
    pub fn with_entrypoint(mut self, entrypoint: impl Into<String>) -> Self {
        self.entrypoint = Some(entrypoint.into());
        self
    }

    /// Set command.
    pub fn with_command(mut self, command: Vec<String>) -> Self {
        self.command = Some(command);
        self
    }

    /// Set user.
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Enable GPU access.
    pub fn with_gpus(mut self, gpus: impl Into<String>) -> Self {
        self.gpus = Some(gpus.into());
        self
    }

    /// Enable all GPUs.
    pub fn with_all_gpus(self) -> Self {
        self.with_gpus("all")
    }

    /// Set platform.
    pub fn with_platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Enable privileged mode (dangerous!).
    pub fn privileged(mut self) -> Self {
        self.privileged = true;
        self
    }

    /// Add security option.
    pub fn with_security_opt(mut self, opt: impl Into<String>) -> Self {
        self.security_opt.push(opt.into());
        self
    }

    /// Disable new privileges.
    pub fn with_no_new_privileges(self) -> Self {
        self.with_security_opt("no-new-privileges")
    }

    /// Add sysctl.
    pub fn with_sysctl(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.sysctls.insert(key.into(), value.into());
        self
    }

    /// Add label.
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Set restart policy.
    pub fn with_restart(mut self, policy: impl Into<String>) -> Self {
        self.restart = Some(policy.into());
        self
    }

    /// Keep container after execution.
    pub fn keep_container(mut self) -> Self {
        self.rm = false;
        self
    }

    /// Enable init process.
    pub fn with_init(mut self) -> Self {
        self.init = true;
        self
    }

    /// Set hostname.
    pub fn with_hostname(mut self, hostname: impl Into<String>) -> Self {
        self.hostname = Some(hostname.into());
        self
    }

    /// Add a capability.
    pub fn add_capability(mut self, cap: impl Into<String>) -> Self {
        self.cap_add.push(cap.into());
        self
    }

    /// Drop a capability.
    pub fn drop_capability(mut self, cap: impl Into<String>) -> Self {
        self.cap_drop.push(cap.into());
        self
    }

    /// Drop all capabilities.
    pub fn drop_all_capabilities(self) -> Self {
        self.drop_capability("ALL")
    }

    /// Build docker run arguments from these overrides.
    pub fn to_docker_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.rm {
            args.push("--rm".to_string());
        }

        if self.init {
            args.push("--init".to_string());
        }

        if self.privileged {
            args.push("--privileged".to_string());
        }

        if let Some(ref user) = self.user {
            args.push("--user".to_string());
            args.push(user.clone());
        }

        if let Some(ref gpus) = self.gpus {
            args.push("--gpus".to_string());
            args.push(gpus.clone());
        }

        if let Some(ref platform) = self.platform {
            args.push("--platform".to_string());
            args.push(platform.clone());
        }

        if let Some(ref entrypoint) = self.entrypoint {
            args.push("--entrypoint".to_string());
            args.push(entrypoint.clone());
        }

        if let Some(ref hostname) = self.hostname {
            args.push("--hostname".to_string());
            args.push(hostname.clone());
        }

        if let Some(ref ipc) = self.ipc {
            args.push("--ipc".to_string());
            args.push(ipc.clone());
        }

        if let Some(ref pid) = self.pid {
            args.push("--pid".to_string());
            args.push(pid.clone());
        }

        if let Some(ref restart) = self.restart {
            args.push("--restart".to_string());
            args.push(restart.clone());
        }

        for opt in &self.security_opt {
            args.push("--security-opt".to_string());
            args.push(opt.clone());
        }

        for (key, value) in &self.sysctls {
            args.push("--sysctl".to_string());
            args.push(format!("{}={}", key, value));
        }

        for (key, value) in &self.labels {
            args.push("--label".to_string());
            args.push(format!("{}={}", key, value));
        }

        for cap in &self.cap_add {
            args.push("--cap-add".to_string());
            args.push(cap.clone());
        }

        for cap in &self.cap_drop {
            args.push("--cap-drop".to_string());
            args.push(cap.clone());
        }

        args.extend(self.extra_args.clone());

        args
    }
}

/// Native execution overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NativeOverrides {
    /// Working directory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,

    /// Shell to use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,

    /// PATH additions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path_additions: Vec<String>,

    /// Run as different user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_as: Option<String>,

    /// Clear environment before execution.
    #[serde(default)]
    pub clear_env: bool,

    /// Inherit environment from parent.
    #[serde(default = "default_true")]
    pub inherit_env: bool,
}

impl Default for NativeOverrides {
    fn default() -> Self {
        Self {
            working_dir: None,
            shell: None,
            path_additions: Vec::new(),
            run_as: None,
            clear_env: false,
            inherit_env: true, // Default to inheriting environment
        }
    }
}

impl NativeOverrides {
    /// Create new native overrides.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set working directory.
    pub fn with_working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Set shell.
    pub fn with_shell(mut self, shell: impl Into<String>) -> Self {
        self.shell = Some(shell.into());
        self
    }

    /// Add to PATH.
    pub fn with_path_addition(mut self, path: impl Into<String>) -> Self {
        self.path_additions.push(path.into());
        self
    }

    /// Run as user.
    pub fn with_run_as(mut self, user: impl Into<String>) -> Self {
        self.run_as = Some(user.into());
        self
    }

    /// Clear environment before execution.
    pub fn with_clear_env(mut self) -> Self {
        self.clear_env = true;
        self.inherit_env = false;
        self
    }

    /// Don't inherit environment.
    pub fn without_inherit_env(mut self) -> Self {
        self.inherit_env = false;
        self
    }

    /// Get the shell to use, with a default.
    pub fn shell_or_default(&self) -> &str {
        self.shell.as_deref().unwrap_or(if cfg!(windows) {
            "cmd.exe"
        } else {
            "/bin/sh"
        })
    }

    /// Build the PATH environment variable.
    pub fn build_path(&self, existing_path: Option<&str>) -> String {
        let separator = if cfg!(windows) { ";" } else { ":" };
        let additions = self.path_additions.join(separator);

        match (additions.is_empty(), existing_path) {
            (true, Some(p)) => p.to_string(),
            (true, None) => String::new(),
            (false, Some(p)) if self.inherit_env => format!("{}{}{}",additions, separator, p),
            (false, _) => additions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_overrides_builder() {
        let overrides = RuntimeOverrides::new()
            .with_wasm(WasmOverrides::new().with_fuel_limit(1000))
            .with_docker(DockerOverrides::new().with_image("python:3.11"));

        assert!(overrides.wasm.is_some());
        assert!(overrides.docker.is_some());
        assert!(!overrides.is_empty());
    }

    #[test]
    fn test_wasm_overrides() {
        let wasm = WasmOverrides::new()
            .with_stack_size(2 * 1024 * 1024)
            .with_fuel_limit(100_000)
            .enable_capability("filesystem")
            .disable_capability("network")
            .with_debug_info();

        assert_eq!(wasm.stack_size, Some(2 * 1024 * 1024));
        assert_eq!(wasm.fuel_limit, Some(100_000));
        assert_eq!(wasm.is_capability_enabled("filesystem"), Some(true));
        assert_eq!(wasm.is_capability_enabled("network"), Some(false));
        assert!(wasm.debug_info);
    }

    #[test]
    fn test_docker_overrides() {
        let docker = DockerOverrides::new()
            .with_image("python:3.11-slim")
            .with_user("1000:1000")
            .with_no_new_privileges()
            .drop_all_capabilities()
            .add_capability("NET_BIND_SERVICE")
            .with_label("app", "skill-engine");

        assert_eq!(docker.image, Some("python:3.11-slim".to_string()));
        assert_eq!(docker.user, Some("1000:1000".to_string()));
        assert!(docker.security_opt.contains(&"no-new-privileges".to_string()));
        assert!(docker.cap_drop.contains(&"ALL".to_string()));
        assert!(docker.cap_add.contains(&"NET_BIND_SERVICE".to_string()));
    }

    #[test]
    fn test_docker_args() {
        let docker = DockerOverrides::new()
            .with_user("1000:1000")
            .with_all_gpus()
            .with_init()
            .with_no_new_privileges();

        let args = docker.to_docker_args();

        assert!(args.contains(&"--rm".to_string()));
        assert!(args.contains(&"--init".to_string()));
        assert!(args.contains(&"--user".to_string()));
        assert!(args.contains(&"--gpus".to_string()));
        assert!(args.contains(&"--security-opt".to_string()));
    }

    #[test]
    fn test_native_overrides() {
        let native = NativeOverrides::new()
            .with_working_dir("/app")
            .with_shell("/bin/bash")
            .with_path_addition("/custom/bin");

        assert_eq!(native.working_dir, Some("/app".to_string()));
        assert_eq!(native.shell_or_default(), "/bin/bash");
    }

    #[test]
    fn test_native_path_building() {
        let native = NativeOverrides::new()
            .with_path_addition("/usr/local/bin")
            .with_path_addition("/opt/bin");

        let path = native.build_path(Some("/usr/bin"));
        assert!(path.contains("/usr/local/bin"));
        assert!(path.contains("/opt/bin"));
        assert!(path.contains("/usr/bin"));
    }

    #[test]
    fn test_runtime_overrides_serialization() {
        let overrides = RuntimeOverrides::new()
            .with_wasm(WasmOverrides::new().with_fuel_limit(1000))
            .with_docker(DockerOverrides::new().with_image("python:3.11"));

        let json = serde_json::to_string(&overrides).unwrap();
        let deserialized: RuntimeOverrides = serde_json::from_str(&json).unwrap();

        assert!(deserialized.wasm.is_some());
        assert!(deserialized.docker.is_some());
    }
}
