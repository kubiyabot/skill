//! Resource configuration types.
//!
//! This module defines resource limits and capabilities for execution contexts.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Resource limits and capabilities configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResourceConfig {
    /// CPU limits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<CpuConfig>,

    /// Memory limits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemoryConfig>,

    /// Network configuration.
    #[serde(default)]
    pub network: NetworkConfig,

    /// Filesystem capabilities.
    #[serde(default)]
    pub filesystem: FilesystemConfig,

    /// Execution limits.
    #[serde(default)]
    pub execution: ExecutionLimits,
}

impl ResourceConfig {
    /// Create a new default resource configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set CPU limits.
    pub fn with_cpu(mut self, cpu: CpuConfig) -> Self {
        self.cpu = Some(cpu);
        self
    }

    /// Set memory limits.
    pub fn with_memory(mut self, memory: MemoryConfig) -> Self {
        self.memory = Some(memory);
        self
    }

    /// Set network configuration.
    pub fn with_network(mut self, network: NetworkConfig) -> Self {
        self.network = network;
        self
    }

    /// Set filesystem configuration.
    pub fn with_filesystem(mut self, filesystem: FilesystemConfig) -> Self {
        self.filesystem = filesystem;
        self
    }

    /// Set execution limits.
    pub fn with_execution(mut self, execution: ExecutionLimits) -> Self {
        self.execution = execution;
        self
    }

    /// Enable network access with default settings.
    pub fn with_network_enabled(mut self) -> Self {
        self.network.enabled = true;
        self
    }

    /// Disable network access.
    pub fn with_network_disabled(mut self) -> Self {
        self.network.enabled = false;
        self
    }

    /// Set execution timeout.
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.execution.timeout_seconds = Some(seconds);
        self
    }

    /// Set memory limit using a size string (e.g., "512m", "2g").
    pub fn with_memory_limit(mut self, limit: impl Into<String>) -> Self {
        self.memory = Some(MemoryConfig {
            limit: limit.into(),
            swap: None,
            reservation: None,
        });
        self
    }

    /// Set CPU limit.
    pub fn with_cpu_limit(mut self, limit: impl Into<String>) -> Self {
        self.cpu = Some(CpuConfig {
            limit: limit.into(),
            shares: None,
        });
        self
    }
}

/// CPU configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CpuConfig {
    /// CPU cores limit (e.g., "0.5", "2", "4").
    pub limit: String,

    /// CPU shares for relative priority (Docker cgroups).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shares: Option<u32>,
}

impl CpuConfig {
    /// Create a new CPU configuration.
    pub fn new(limit: impl Into<String>) -> Self {
        Self {
            limit: limit.into(),
            shares: None,
        }
    }

    /// Set CPU shares.
    pub fn with_shares(mut self, shares: u32) -> Self {
        self.shares = Some(shares);
        self
    }

    /// Parse the limit as a float (number of cores).
    pub fn limit_as_cores(&self) -> Option<f64> {
        self.limit.parse().ok()
    }

    /// Convert to Docker CPU quota format (microseconds per 100ms period).
    pub fn as_docker_quota(&self) -> Option<i64> {
        self.limit_as_cores().map(|cores| (cores * 100_000.0) as i64)
    }
}

/// Memory configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MemoryConfig {
    /// Memory limit (e.g., "512m", "2g").
    pub limit: String,

    /// Swap limit (e.g., "1g", "0" to disable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swap: Option<String>,

    /// Memory reservation (soft limit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reservation: Option<String>,
}

impl MemoryConfig {
    /// Create a new memory configuration.
    pub fn new(limit: impl Into<String>) -> Self {
        Self {
            limit: limit.into(),
            swap: None,
            reservation: None,
        }
    }

    /// Set swap limit.
    pub fn with_swap(mut self, swap: impl Into<String>) -> Self {
        self.swap = Some(swap.into());
        self
    }

    /// Disable swap.
    pub fn without_swap(mut self) -> Self {
        self.swap = Some("0".to_string());
        self
    }

    /// Set memory reservation.
    pub fn with_reservation(mut self, reservation: impl Into<String>) -> Self {
        self.reservation = Some(reservation.into());
        self
    }

    /// Parse the limit as bytes.
    pub fn limit_as_bytes(&self) -> Option<u64> {
        parse_size(&self.limit)
    }

    /// Parse the swap limit as bytes.
    pub fn swap_as_bytes(&self) -> Option<u64> {
        self.swap.as_ref().and_then(|s| parse_size(s))
    }

    /// Parse the reservation as bytes.
    pub fn reservation_as_bytes(&self) -> Option<u64> {
        self.reservation.as_ref().and_then(|s| parse_size(s))
    }
}

/// Network configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NetworkConfig {
    /// Whether network access is allowed.
    #[serde(default)]
    pub enabled: bool,

    /// Network mode for Docker (none, bridge, host).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,

    /// Allowed outbound hosts (whitelist).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_hosts: Option<Vec<String>>,

    /// Blocked hosts (blacklist).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_hosts: Option<Vec<String>>,

    /// DNS servers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dns: Option<Vec<String>>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: None,
            allowed_hosts: None,
            blocked_hosts: None,
            dns: None,
        }
    }
}

impl NetworkConfig {
    /// Create a new network configuration with network enabled.
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Create a new network configuration with network disabled.
    pub fn disabled() -> Self {
        Self::default()
    }

    /// Set the network mode.
    pub fn with_mode(mut self, mode: impl Into<String>) -> Self {
        self.mode = Some(mode.into());
        self
    }

    /// Set allowed hosts.
    pub fn with_allowed_hosts(mut self, hosts: Vec<String>) -> Self {
        self.allowed_hosts = Some(hosts);
        self
    }

    /// Add an allowed host.
    pub fn allow_host(mut self, host: impl Into<String>) -> Self {
        self.allowed_hosts
            .get_or_insert_with(Vec::new)
            .push(host.into());
        self
    }

    /// Set blocked hosts.
    pub fn with_blocked_hosts(mut self, hosts: Vec<String>) -> Self {
        self.blocked_hosts = Some(hosts);
        self
    }

    /// Block a host.
    pub fn block_host(mut self, host: impl Into<String>) -> Self {
        self.blocked_hosts
            .get_or_insert_with(Vec::new)
            .push(host.into());
        self
    }

    /// Set DNS servers.
    pub fn with_dns(mut self, servers: Vec<String>) -> Self {
        self.dns = Some(servers);
        self
    }

    /// Check if a host is allowed.
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if !self.enabled {
            return false;
        }

        // Check blocked list first
        if let Some(ref blocked) = self.blocked_hosts {
            if blocked.iter().any(|b| host_matches(host, b)) {
                return false;
            }
        }

        // If there's a whitelist, host must be in it
        if let Some(ref allowed) = self.allowed_hosts {
            return allowed.iter().any(|a| host_matches(host, a));
        }

        // No whitelist means all non-blocked hosts are allowed
        true
    }
}

/// Filesystem configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FilesystemConfig {
    /// Read-only root filesystem.
    #[serde(default)]
    pub read_only_root: bool,

    /// Paths that are writable (within read-only root).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub writable_paths: Vec<String>,

    /// Maximum file size that can be created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_file_size: Option<String>,

    /// Maximum total disk usage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_disk_usage: Option<String>,
}

impl FilesystemConfig {
    /// Create a new filesystem configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable read-only root filesystem.
    pub fn read_only(mut self) -> Self {
        self.read_only_root = true;
        self
    }

    /// Add a writable path.
    pub fn with_writable_path(mut self, path: impl Into<String>) -> Self {
        self.writable_paths.push(path.into());
        self
    }

    /// Set maximum file size.
    pub fn with_max_file_size(mut self, size: impl Into<String>) -> Self {
        self.max_file_size = Some(size.into());
        self
    }

    /// Set maximum disk usage.
    pub fn with_max_disk_usage(mut self, size: impl Into<String>) -> Self {
        self.max_disk_usage = Some(size.into());
        self
    }

    /// Parse max file size as bytes.
    pub fn max_file_size_bytes(&self) -> Option<u64> {
        self.max_file_size.as_ref().and_then(|s| parse_size(s))
    }

    /// Parse max disk usage as bytes.
    pub fn max_disk_usage_bytes(&self) -> Option<u64> {
        self.max_disk_usage.as_ref().and_then(|s| parse_size(s))
    }
}

/// Execution limits.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExecutionLimits {
    /// Maximum execution time in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,

    /// Maximum concurrent executions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_concurrent: Option<u32>,

    /// Rate limiting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimit>,
}

impl ExecutionLimits {
    /// Create new execution limits.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set timeout.
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }

    /// Set max concurrent executions.
    pub fn with_max_concurrent(mut self, max: u32) -> Self {
        self.max_concurrent = Some(max);
        self
    }

    /// Set rate limit.
    pub fn with_rate_limit(mut self, requests: u32, window_seconds: u32) -> Self {
        self.rate_limit = Some(RateLimit {
            requests,
            window_seconds,
        });
        self
    }

    /// Get timeout as Duration.
    pub fn timeout(&self) -> Option<Duration> {
        self.timeout_seconds.map(Duration::from_secs)
    }
}

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RateLimit {
    /// Maximum requests per window.
    pub requests: u32,

    /// Window duration in seconds.
    pub window_seconds: u32,
}

impl RateLimit {
    /// Create a new rate limit.
    pub fn new(requests: u32, window_seconds: u32) -> Self {
        Self {
            requests,
            window_seconds,
        }
    }

    /// Get window as Duration.
    pub fn window(&self) -> Duration {
        Duration::from_secs(self.window_seconds as u64)
    }

    /// Calculate requests per second.
    pub fn requests_per_second(&self) -> f64 {
        if self.window_seconds == 0 {
            0.0
        } else {
            self.requests as f64 / self.window_seconds as f64
        }
    }
}

/// Parse a size string (e.g., "512m", "2g", "1024k") into bytes.
pub fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }

    let (num_str, multiplier) = if s.ends_with("gb") || s.ends_with("g") {
        let num = s.trim_end_matches(|c| c == 'g' || c == 'b');
        (num, 1024 * 1024 * 1024)
    } else if s.ends_with("mb") || s.ends_with("m") {
        let num = s.trim_end_matches(|c| c == 'm' || c == 'b');
        (num, 1024 * 1024)
    } else if s.ends_with("kb") || s.ends_with("k") {
        let num = s.trim_end_matches(|c| c == 'k' || c == 'b');
        (num, 1024)
    } else if s.ends_with('b') {
        let num = s.trim_end_matches('b');
        (num, 1)
    } else {
        // Assume bytes if no suffix
        (s.as_str(), 1)
    };

    num_str.trim().parse::<u64>().ok().map(|n| n * multiplier)
}

/// Check if a hostname matches a pattern (supports wildcards like *.example.com).
fn host_matches(host: &str, pattern: &str) -> bool {
    if pattern.starts_with("*.") {
        let suffix = &pattern[1..]; // Keep the dot
        host.ends_with(suffix) || host == &pattern[2..]
    } else {
        host == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_config_builder() {
        let config = ResourceConfig::new()
            .with_cpu_limit("2")
            .with_memory_limit("1g")
            .with_network_enabled()
            .with_timeout(300);

        assert!(config.cpu.is_some());
        assert!(config.memory.is_some());
        assert!(config.network.enabled);
        assert_eq!(config.execution.timeout_seconds, Some(300));
    }

    #[test]
    fn test_cpu_config() {
        let cpu = CpuConfig::new("2.5").with_shares(1024);

        assert_eq!(cpu.limit_as_cores(), Some(2.5));
        assert_eq!(cpu.shares, Some(1024));
        assert_eq!(cpu.as_docker_quota(), Some(250_000));
    }

    #[test]
    fn test_memory_config() {
        let mem = MemoryConfig::new("512m")
            .with_swap("1g")
            .with_reservation("256m");

        assert_eq!(mem.limit_as_bytes(), Some(512 * 1024 * 1024));
        assert_eq!(mem.swap_as_bytes(), Some(1024 * 1024 * 1024));
        assert_eq!(mem.reservation_as_bytes(), Some(256 * 1024 * 1024));
    }

    #[test]
    fn test_network_config() {
        let net = NetworkConfig::enabled()
            .with_mode("bridge")
            .allow_host("api.example.com")
            .allow_host("*.amazonaws.com")
            .block_host("blocked.example.com");

        assert!(net.enabled);
        assert!(net.is_host_allowed("api.example.com"));
        assert!(net.is_host_allowed("s3.amazonaws.com"));
        assert!(!net.is_host_allowed("blocked.example.com"));
        assert!(!net.is_host_allowed("other.com"));
    }

    #[test]
    fn test_network_disabled() {
        let net = NetworkConfig::disabled();
        assert!(!net.is_host_allowed("any.com"));
    }

    #[test]
    fn test_filesystem_config() {
        let fs = FilesystemConfig::new()
            .read_only()
            .with_writable_path("/tmp")
            .with_max_file_size("100m");

        assert!(fs.read_only_root);
        assert!(fs.writable_paths.contains(&"/tmp".to_string()));
        assert_eq!(fs.max_file_size_bytes(), Some(100 * 1024 * 1024));
    }

    #[test]
    fn test_execution_limits() {
        let limits = ExecutionLimits::new()
            .with_timeout(60)
            .with_max_concurrent(10)
            .with_rate_limit(100, 60);

        assert_eq!(limits.timeout(), Some(Duration::from_secs(60)));
        assert_eq!(limits.max_concurrent, Some(10));

        let rate = limits.rate_limit.unwrap();
        assert_eq!(rate.requests_per_second(), 100.0 / 60.0);
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024"), Some(1024));
        assert_eq!(parse_size("1k"), Some(1024));
        assert_eq!(parse_size("1kb"), Some(1024));
        assert_eq!(parse_size("1m"), Some(1024 * 1024));
        assert_eq!(parse_size("1mb"), Some(1024 * 1024));
        assert_eq!(parse_size("1g"), Some(1024 * 1024 * 1024));
        assert_eq!(parse_size("1gb"), Some(1024 * 1024 * 1024));
        assert_eq!(parse_size("512M"), Some(512 * 1024 * 1024));
        assert_eq!(parse_size(""), None);
        assert_eq!(parse_size("invalid"), None);
    }

    #[test]
    fn test_host_matches() {
        assert!(host_matches("api.example.com", "api.example.com"));
        assert!(host_matches("api.example.com", "*.example.com"));
        assert!(host_matches("sub.api.example.com", "*.example.com"));
        assert!(host_matches("example.com", "*.example.com"));
        assert!(!host_matches("other.com", "*.example.com"));
    }

    #[test]
    fn test_resource_config_serialization() {
        let config = ResourceConfig::new()
            .with_cpu_limit("2")
            .with_memory_limit("1g")
            .with_network_enabled();

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ResourceConfig = serde_json::from_str(&json).unwrap();

        assert!(deserialized.cpu.is_some());
        assert!(deserialized.memory.is_some());
        assert!(deserialized.network.enabled);
    }
}
