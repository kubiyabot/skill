//! Context inheritance and resolution logic.
//!
//! This module provides functionality for resolving context inheritance chains,
//! merging configurations from parent contexts into child contexts according to
//! defined merge rules.
//!
//! # Inheritance Rules
//!
//! - **Scalar values** (strings, numbers, booleans): Child completely overrides parent
//! - **Arrays**: Child replaces parent array entirely (use `+` prefix in config to append)
//! - **Maps** (HashMaps): Deep merge with child taking precedence
//! - **Secrets**: Child can add new secrets or override provider for existing ones
//! - **Mounts**: Merged by mount ID, child overrides parent mounts with same ID

use std::collections::{HashMap, HashSet};

use crate::context::ExecutionContext;
use crate::environment::EnvironmentConfig;
use crate::mounts::Mount;
use crate::resources::ResourceConfig;
use crate::runtime::RuntimeOverrides;
use crate::secrets::SecretsConfig;
use crate::ContextError;

/// Resolves context inheritance chains.
///
/// The resolver takes contexts with `inherits_from` references and produces
/// fully resolved contexts with all inherited values merged.
pub struct ContextResolver<F> {
    /// Function to load a context by ID.
    loader: F,
    /// Cache of resolved contexts.
    cache: HashMap<String, ExecutionContext>,
    /// Set of context IDs currently being resolved (for cycle detection).
    resolving: HashSet<String>,
}

impl<F> ContextResolver<F>
where
    F: Fn(&str) -> Result<ExecutionContext, ContextError>,
{
    /// Create a new resolver with the given context loader function.
    pub fn new(loader: F) -> Self {
        Self {
            loader,
            cache: HashMap::new(),
            resolving: HashSet::new(),
        }
    }

    /// Resolve a context, applying all inherited values from parent contexts.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A parent context cannot be loaded
    /// - Circular inheritance is detected
    pub fn resolve(&mut self, context: &ExecutionContext) -> Result<ExecutionContext, ContextError> {
        // Check cache first
        if let Some(cached) = self.cache.get(&context.id) {
            return Ok(cached.clone());
        }

        // Check for circular inheritance
        if self.resolving.contains(&context.id) {
            return Err(ContextError::CircularInheritance(format!(
                "Circular inheritance detected involving context '{}'",
                context.id
            )));
        }

        // Mark as currently resolving
        self.resolving.insert(context.id.clone());

        let resolved = if let Some(ref parent_id) = context.inherits_from {
            // Check if parent is already resolved in cache
            let resolved_parent = if let Some(cached_parent) = self.cache.get(parent_id) {
                cached_parent.clone()
            } else {
                // Load and resolve parent
                let parent = (self.loader)(parent_id).map_err(|_| {
                    ContextError::ParentNotFound(format!(
                        "Parent context '{}' not found for context '{}'",
                        parent_id, context.id
                    ))
                })?;

                self.resolve(&parent)?
            };

            // Merge child onto resolved parent
            self.merge_contexts(&resolved_parent, context)
        } else {
            // No parent, return context as-is
            context.clone()
        };

        // Remove from resolving set
        self.resolving.remove(&context.id);

        // Cache the result
        self.cache.insert(context.id.clone(), resolved.clone());

        Ok(resolved)
    }

    /// Clear the resolution cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Invalidate a specific context from the cache.
    pub fn invalidate(&mut self, context_id: &str) {
        self.cache.remove(context_id);
        // Also invalidate any contexts that might inherit from this one
        // (This is a simple approach; a more sophisticated one would track dependencies)
        self.cache.clear();
    }

    /// Merge a child context onto a parent context.
    fn merge_contexts(&self, parent: &ExecutionContext, child: &ExecutionContext) -> ExecutionContext {
        ExecutionContext {
            // Identity comes from child
            id: child.id.clone(),
            name: child.name.clone(),
            description: child.description.clone().or_else(|| parent.description.clone()),
            inherits_from: child.inherits_from.clone(),

            // Merge complex fields
            mounts: merge_mounts(&parent.mounts, &child.mounts),
            environment: merge_environments(&parent.environment, &child.environment),
            secrets: merge_secrets(&parent.secrets, &child.secrets),
            resources: merge_resources(&parent.resources, &child.resources),
            runtime_overrides: merge_runtime_overrides(
                parent.runtime_overrides.as_ref(),
                child.runtime_overrides.as_ref(),
            ),

            // Metadata comes from child
            metadata: child.metadata.clone(),
        }
    }
}

/// Resolve a single context with its inheritance chain.
///
/// This is a convenience function for one-off resolution without maintaining a cache.
pub fn resolve_context<F>(
    context: &ExecutionContext,
    loader: F,
) -> Result<ExecutionContext, ContextError>
where
    F: Fn(&str) -> Result<ExecutionContext, ContextError>,
{
    let mut resolver = ContextResolver::new(loader);
    resolver.resolve(context)
}

/// Merge mounts from parent and child.
///
/// Mounts are merged by ID - child mounts with the same ID override parent mounts.
pub fn merge_mounts(parent: &[Mount], child: &[Mount]) -> Vec<Mount> {
    let mut result: HashMap<String, Mount> = parent
        .iter()
        .map(|m| (m.id.clone(), m.clone()))
        .collect();

    // Child mounts override parent mounts with same ID
    for mount in child {
        result.insert(mount.id.clone(), mount.clone());
    }

    result.into_values().collect()
}

/// Merge environment configurations.
///
/// Variables are merged with child taking precedence.
/// Env files, passthrough prefixes, and passthrough vars are concatenated.
pub fn merge_environments(parent: &EnvironmentConfig, child: &EnvironmentConfig) -> EnvironmentConfig {
    let mut variables = parent.variables.clone();
    for (key, value) in &child.variables {
        variables.insert(key.clone(), value.clone());
    }

    // Concatenate arrays, deduplicating
    let mut env_files = parent.env_files.clone();
    for file in &child.env_files {
        if !env_files.iter().any(|f| f.path == file.path) {
            env_files.push(file.clone());
        }
    }

    let mut passthrough_prefixes: Vec<String> = parent.passthrough_prefixes.clone();
    for prefix in &child.passthrough_prefixes {
        if !passthrough_prefixes.contains(prefix) {
            passthrough_prefixes.push(prefix.clone());
        }
    }

    let mut passthrough_vars: Vec<String> = parent.passthrough_vars.clone();
    for var in &child.passthrough_vars {
        if !passthrough_vars.contains(var) {
            passthrough_vars.push(var.clone());
        }
    }

    EnvironmentConfig {
        variables,
        env_files,
        passthrough_prefixes,
        passthrough_vars,
    }
}

/// Merge secrets configurations.
///
/// Secret definitions are merged by key with child taking precedence.
/// Providers are concatenated.
pub fn merge_secrets(parent: &SecretsConfig, child: &SecretsConfig) -> SecretsConfig {
    let mut secrets = parent.secrets.clone();
    for (key, def) in &child.secrets {
        secrets.insert(key.clone(), def.clone());
    }

    // Concatenate providers, child providers come first (higher priority)
    let mut providers = child.providers.clone();
    for provider in &parent.providers {
        // Only add parent provider if child doesn't have one with same name
        let parent_name = provider.name();
        if !providers.iter().any(|p| p.name() == parent_name) {
            providers.push(provider.clone());
        }
    }

    SecretsConfig { secrets, providers }
}

/// Merge resource configurations.
///
/// Child values completely override parent values for each field.
pub fn merge_resources(parent: &ResourceConfig, child: &ResourceConfig) -> ResourceConfig {
    ResourceConfig {
        cpu: child.cpu.clone().or_else(|| parent.cpu.clone()),
        memory: child.memory.clone().or_else(|| parent.memory.clone()),
        network: merge_network_config(&parent.network, &child.network),
        filesystem: merge_filesystem_config(&parent.filesystem, &child.filesystem),
        execution: merge_execution_limits(&parent.execution, &child.execution),
    }
}

/// Merge network configurations.
fn merge_network_config(
    parent: &crate::resources::NetworkConfig,
    child: &crate::resources::NetworkConfig,
) -> crate::resources::NetworkConfig {
    crate::resources::NetworkConfig {
        // If child explicitly sets enabled, use that; otherwise inherit
        enabled: child.enabled || parent.enabled,
        mode: child.mode.clone().or_else(|| parent.mode.clone()),
        // Merge allowed/blocked hosts
        allowed_hosts: match (&parent.allowed_hosts, &child.allowed_hosts) {
            (Some(p), Some(c)) => {
                let mut hosts = p.clone();
                for h in c {
                    if !hosts.contains(h) {
                        hosts.push(h.clone());
                    }
                }
                Some(hosts)
            }
            (None, Some(c)) => Some(c.clone()),
            (Some(p), None) => Some(p.clone()),
            (None, None) => None,
        },
        blocked_hosts: match (&parent.blocked_hosts, &child.blocked_hosts) {
            (Some(p), Some(c)) => {
                let mut hosts = p.clone();
                for h in c {
                    if !hosts.contains(h) {
                        hosts.push(h.clone());
                    }
                }
                Some(hosts)
            }
            (None, Some(c)) => Some(c.clone()),
            (Some(p), None) => Some(p.clone()),
            (None, None) => None,
        },
        dns: child.dns.clone().or_else(|| parent.dns.clone()),
    }
}

/// Merge filesystem configurations.
fn merge_filesystem_config(
    parent: &crate::resources::FilesystemConfig,
    child: &crate::resources::FilesystemConfig,
) -> crate::resources::FilesystemConfig {
    // Merge writable paths
    let mut writable_paths = parent.writable_paths.clone();
    for path in &child.writable_paths {
        if !writable_paths.contains(path) {
            writable_paths.push(path.clone());
        }
    }

    crate::resources::FilesystemConfig {
        read_only_root: child.read_only_root || parent.read_only_root,
        writable_paths,
        max_file_size: child
            .max_file_size
            .clone()
            .or_else(|| parent.max_file_size.clone()),
        max_disk_usage: child
            .max_disk_usage
            .clone()
            .or_else(|| parent.max_disk_usage.clone()),
    }
}

/// Merge execution limits.
fn merge_execution_limits(
    parent: &crate::resources::ExecutionLimits,
    child: &crate::resources::ExecutionLimits,
) -> crate::resources::ExecutionLimits {
    crate::resources::ExecutionLimits {
        timeout_seconds: child.timeout_seconds.or(parent.timeout_seconds),
        max_concurrent: child.max_concurrent.or(parent.max_concurrent),
        rate_limit: child.rate_limit.clone().or_else(|| parent.rate_limit.clone()),
    }
}

/// Merge runtime overrides.
fn merge_runtime_overrides(
    parent: Option<&RuntimeOverrides>,
    child: Option<&RuntimeOverrides>,
) -> Option<RuntimeOverrides> {
    match (parent, child) {
        (None, None) => None,
        (Some(p), None) => Some(p.clone()),
        (None, Some(c)) => Some(c.clone()),
        (Some(p), Some(c)) => Some(RuntimeOverrides {
            wasm: merge_wasm_overrides(p.wasm.as_ref(), c.wasm.as_ref()),
            docker: merge_docker_overrides(p.docker.as_ref(), c.docker.as_ref()),
            native: merge_native_overrides(p.native.as_ref(), c.native.as_ref()),
        }),
    }
}

/// Merge WASM overrides.
fn merge_wasm_overrides(
    parent: Option<&crate::runtime::WasmOverrides>,
    child: Option<&crate::runtime::WasmOverrides>,
) -> Option<crate::runtime::WasmOverrides> {
    match (parent, child) {
        (None, None) => None,
        (Some(p), None) => Some(p.clone()),
        (None, Some(c)) => Some(c.clone()),
        (Some(p), Some(c)) => {
            let mut wasi_capabilities = p.wasi_capabilities.clone();
            for (key, value) in &c.wasi_capabilities {
                wasi_capabilities.insert(key.clone(), *value);
            }

            Some(crate::runtime::WasmOverrides {
                stack_size: c.stack_size.or(p.stack_size),
                wasi_capabilities,
                fuel_limit: c.fuel_limit.or(p.fuel_limit),
                epoch_interruption: c.epoch_interruption.or(p.epoch_interruption),
                max_memory_pages: c.max_memory_pages.or(p.max_memory_pages),
                debug_info: c.debug_info || p.debug_info,
            })
        }
    }
}

/// Merge Docker overrides.
fn merge_docker_overrides(
    parent: Option<&crate::runtime::DockerOverrides>,
    child: Option<&crate::runtime::DockerOverrides>,
) -> Option<crate::runtime::DockerOverrides> {
    match (parent, child) {
        (None, None) => None,
        (Some(p), None) => Some(p.clone()),
        (None, Some(c)) => Some(c.clone()),
        (Some(p), Some(c)) => {
            // Merge extra_args
            let mut extra_args = p.extra_args.clone();
            extra_args.extend(c.extra_args.clone());

            // Merge security_opt
            let mut security_opt = p.security_opt.clone();
            for opt in &c.security_opt {
                if !security_opt.contains(opt) {
                    security_opt.push(opt.clone());
                }
            }

            // Merge sysctls
            let mut sysctls = p.sysctls.clone();
            for (key, value) in &c.sysctls {
                sysctls.insert(key.clone(), value.clone());
            }

            // Merge labels
            let mut labels = p.labels.clone();
            for (key, value) in &c.labels {
                labels.insert(key.clone(), value.clone());
            }

            // Merge cap_add/cap_drop
            let mut cap_add = p.cap_add.clone();
            for cap in &c.cap_add {
                if !cap_add.contains(cap) {
                    cap_add.push(cap.clone());
                }
            }

            let mut cap_drop = p.cap_drop.clone();
            for cap in &c.cap_drop {
                if !cap_drop.contains(cap) {
                    cap_drop.push(cap.clone());
                }
            }

            Some(crate::runtime::DockerOverrides {
                image: c.image.clone().or_else(|| p.image.clone()),
                extra_args,
                entrypoint: c.entrypoint.clone().or_else(|| p.entrypoint.clone()),
                command: c.command.clone().or_else(|| p.command.clone()),
                user: c.user.clone().or_else(|| p.user.clone()),
                gpus: c.gpus.clone().or_else(|| p.gpus.clone()),
                platform: c.platform.clone().or_else(|| p.platform.clone()),
                privileged: c.privileged || p.privileged,
                security_opt,
                sysctls,
                labels,
                restart: c.restart.clone().or_else(|| p.restart.clone()),
                rm: c.rm && p.rm, // Both must be true to remove
                init: c.init || p.init,
                hostname: c.hostname.clone().or_else(|| p.hostname.clone()),
                ipc: c.ipc.clone().or_else(|| p.ipc.clone()),
                pid: c.pid.clone().or_else(|| p.pid.clone()),
                cap_add,
                cap_drop,
            })
        }
    }
}

/// Merge native overrides.
fn merge_native_overrides(
    parent: Option<&crate::runtime::NativeOverrides>,
    child: Option<&crate::runtime::NativeOverrides>,
) -> Option<crate::runtime::NativeOverrides> {
    match (parent, child) {
        (None, None) => None,
        (Some(p), None) => Some(p.clone()),
        (None, Some(c)) => Some(c.clone()),
        (Some(p), Some(c)) => {
            // Merge path_additions
            let mut path_additions = p.path_additions.clone();
            for path in &c.path_additions {
                if !path_additions.contains(path) {
                    path_additions.push(path.clone());
                }
            }

            Some(crate::runtime::NativeOverrides {
                working_dir: c.working_dir.clone().or_else(|| p.working_dir.clone()),
                shell: c.shell.clone().or_else(|| p.shell.clone()),
                path_additions,
                run_as: c.run_as.clone().or_else(|| p.run_as.clone()),
                clear_env: c.clear_env || p.clear_env,
                inherit_env: c.inherit_env && p.inherit_env,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::EnvValue;
    use crate::resources::{CpuConfig, MemoryConfig, NetworkConfig};
    use crate::secrets::SecretDefinition;

    #[test]
    fn test_simple_inheritance() {
        let parent = ExecutionContext::new("parent", "Parent")
            .with_environment(EnvironmentConfig::new().with_var("PARENT_VAR", "parent_value"));

        let child = ExecutionContext::inheriting("child", "Child", "parent")
            .with_environment(EnvironmentConfig::new().with_var("CHILD_VAR", "child_value"));

        let contexts: HashMap<String, ExecutionContext> =
            [("parent".to_string(), parent)].into_iter().collect();

        let resolved = resolve_context(&child, |id| {
            contexts
                .get(id)
                .cloned()
                .ok_or_else(|| ContextError::NotFound(id.to_string()))
        })
        .unwrap();

        // Both variables should be present
        assert!(resolved.environment.variables.contains_key("PARENT_VAR"));
        assert!(resolved.environment.variables.contains_key("CHILD_VAR"));
    }

    #[test]
    fn test_child_overrides_parent() {
        let parent = ExecutionContext::new("parent", "Parent")
            .with_environment(EnvironmentConfig::new().with_var("SHARED_VAR", "parent_value"));

        let child = ExecutionContext::inheriting("child", "Child", "parent")
            .with_environment(EnvironmentConfig::new().with_var("SHARED_VAR", "child_value"));

        let contexts: HashMap<String, ExecutionContext> =
            [("parent".to_string(), parent)].into_iter().collect();

        let resolved = resolve_context(&child, |id| {
            contexts
                .get(id)
                .cloned()
                .ok_or_else(|| ContextError::NotFound(id.to_string()))
        })
        .unwrap();

        // Child value should win
        match resolved.environment.variables.get("SHARED_VAR") {
            Some(EnvValue::Plain(v)) => assert_eq!(v, "child_value"),
            _ => panic!("Expected plain value"),
        }
    }

    #[test]
    fn test_multi_level_inheritance() {
        let base = ExecutionContext::new("base", "Base")
            .with_environment(EnvironmentConfig::new().with_var("BASE_VAR", "base"));

        let middle = ExecutionContext::inheriting("middle", "Middle", "base")
            .with_environment(EnvironmentConfig::new().with_var("MIDDLE_VAR", "middle"));

        let child = ExecutionContext::inheriting("child", "Child", "middle")
            .with_environment(EnvironmentConfig::new().with_var("CHILD_VAR", "child"));

        let contexts: HashMap<String, ExecutionContext> = [
            ("base".to_string(), base),
            ("middle".to_string(), middle),
        ]
        .into_iter()
        .collect();

        let resolved = resolve_context(&child, |id| {
            contexts
                .get(id)
                .cloned()
                .ok_or_else(|| ContextError::NotFound(id.to_string()))
        })
        .unwrap();

        // All three variables should be present
        assert!(resolved.environment.variables.contains_key("BASE_VAR"));
        assert!(resolved.environment.variables.contains_key("MIDDLE_VAR"));
        assert!(resolved.environment.variables.contains_key("CHILD_VAR"));
    }

    #[test]
    fn test_circular_inheritance_detection() {
        let ctx_a = ExecutionContext::inheriting("a", "Context A", "b");
        let ctx_b = ExecutionContext::inheriting("b", "Context B", "a");

        let contexts: HashMap<String, ExecutionContext> = [
            ("a".to_string(), ctx_a.clone()),
            ("b".to_string(), ctx_b),
        ]
        .into_iter()
        .collect();

        let result = resolve_context(&ctx_a, |id| {
            contexts
                .get(id)
                .cloned()
                .ok_or_else(|| ContextError::NotFound(id.to_string()))
        });

        assert!(matches!(result, Err(ContextError::CircularInheritance(_))));
    }

    #[test]
    fn test_missing_parent() {
        let child = ExecutionContext::inheriting("child", "Child", "nonexistent");

        let result = resolve_context(&child, |_| Err(ContextError::NotFound("not found".into())));

        assert!(matches!(result, Err(ContextError::ParentNotFound(_))));
    }

    #[test]
    fn test_mount_merge() {
        let parent_mounts = vec![
            Mount::directory("data", "/parent/data", "/data"),
            Mount::directory("config", "/parent/config", "/config"),
        ];

        let child_mounts = vec![
            Mount::directory("config", "/child/config", "/config"), // Override
            Mount::directory("logs", "/child/logs", "/logs"),       // New
        ];

        let merged = merge_mounts(&parent_mounts, &child_mounts);

        assert_eq!(merged.len(), 3);

        // Config should be from child
        let config_mount = merged.iter().find(|m| m.id == "config").unwrap();
        assert_eq!(config_mount.source, "/child/config");
    }

    #[test]
    fn test_secrets_merge() {
        let parent_secrets = SecretsConfig::new()
            .with_secret("parent-key", SecretDefinition::required("parent-key"))
            .with_secret("shared-key", SecretDefinition::required("shared-key"));

        let child_secrets = SecretsConfig::new()
            .with_secret(
                "shared-key",
                SecretDefinition::optional("shared-key"), // Override to optional
            )
            .with_secret("child-key", SecretDefinition::required("child-key"));

        let merged = merge_secrets(&parent_secrets, &child_secrets);

        assert_eq!(merged.secrets.len(), 3);
        assert!(merged.secrets.get("parent-key").unwrap().required);
        assert!(!merged.secrets.get("shared-key").unwrap().required); // Child override
        assert!(merged.secrets.get("child-key").unwrap().required);
    }

    #[test]
    fn test_resources_merge() {
        let parent_resources = ResourceConfig::new()
            .with_cpu(CpuConfig::new("2"))
            .with_memory(MemoryConfig::new("1g"))
            .with_network_enabled();

        let child_resources = ResourceConfig::new().with_memory(MemoryConfig::new("2g")); // Override memory

        let merged = merge_resources(&parent_resources, &child_resources);

        // CPU from parent
        assert_eq!(merged.cpu.as_ref().unwrap().limit, "2");
        // Memory from child
        assert_eq!(merged.memory.as_ref().unwrap().limit, "2g");
        // Network inherited
        assert!(merged.network.enabled);
    }

    #[test]
    fn test_network_hosts_merge() {
        let parent_network = NetworkConfig::enabled()
            .allow_host("parent.example.com")
            .block_host("blocked.example.com");

        let child_network = NetworkConfig::enabled()
            .allow_host("child.example.com")
            .allow_host("parent.example.com"); // Duplicate

        let merged = merge_network_config(&parent_network, &child_network);

        let allowed = merged.allowed_hosts.unwrap();
        assert_eq!(allowed.len(), 2); // Deduplicated
        assert!(allowed.contains(&"parent.example.com".to_string()));
        assert!(allowed.contains(&"child.example.com".to_string()));
    }

    #[test]
    fn test_no_inheritance() {
        let context = ExecutionContext::new("standalone", "Standalone")
            .with_environment(EnvironmentConfig::new().with_var("VAR", "value"));

        let resolved = resolve_context(&context, |_| {
            Err(ContextError::NotFound("should not be called".into()))
        })
        .unwrap();

        assert_eq!(resolved.id, "standalone");
        assert!(resolved.environment.variables.contains_key("VAR"));
    }

    #[test]
    fn test_resolver_cache() {
        let call_count = std::cell::RefCell::new(0);

        let parent = ExecutionContext::new("parent", "Parent");
        let child1 = ExecutionContext::inheriting("child1", "Child 1", "parent");
        let child2 = ExecutionContext::inheriting("child2", "Child 2", "parent");

        let contexts: HashMap<String, ExecutionContext> =
            [("parent".to_string(), parent)].into_iter().collect();

        let mut resolver = ContextResolver::new(|id| {
            *call_count.borrow_mut() += 1;
            contexts
                .get(id)
                .cloned()
                .ok_or_else(|| ContextError::NotFound(id.to_string()))
        });

        // Resolve child1
        resolver.resolve(&child1).unwrap();
        assert_eq!(*call_count.borrow(), 1);

        // Resolve child2 - parent should be loaded from cache
        resolver.resolve(&child2).unwrap();
        // Parent should only be loaded once due to caching
        assert_eq!(*call_count.borrow(), 1);
    }
}
