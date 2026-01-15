# Execution Contexts PRD

## Product Requirements Document: Skill Execution Contexts

**Version:** 1.0
**Date:** December 2025
**Author:** Skill Engine Team
**Status:** Draft

---

## 1. Executive Summary

### 1.1 Problem Statement

The current skill-engine architecture supports **skill instances** which provide configuration and environment isolation per-skill. However, there's no unified way to define and manage **execution contexts** - the complete environment in which tools execute, including:

- Files and directories that should be accessible
- Volume mounts for data persistence
- Environment variables and their sources
- Secrets and credential injection
- Resource limits and capabilities
- Runtime-specific configurations (WASM, Docker, native)

Users currently must:
1. Manually configure each instance separately
2. Manage secrets through CLI commands without UI support
3. Lack visibility into what files/volumes a skill can access
4. Have no way to share execution contexts across multiple skills
5. Cannot template or inherit common configurations

### 1.2 Proposed Solution

Introduce **Execution Contexts** as a first-class concept that:

1. **Separates concerns**: Skill definitions (what tools exist) from execution contexts (how/where they run)
2. **Enables reuse**: Share contexts across multiple skills and instances
3. **Provides full UI support**: Create, edit, and manage contexts through the web interface
4. **Supports templating**: Inherit from base contexts and override specific values
5. **Ensures security**: Clear visibility into capabilities granted to each context

### 1.3 Success Metrics

- 80% of skill configurations can reuse shared execution contexts
- Zero secret values exposed in config files or logs
- Full UI coverage for context management (no CLI-only operations)
- Sub-second context switching between executions
- 100% audit trail for context modifications

---

## 2. Background & Current State

### 2.1 Current Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     CURRENT MODEL                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Skill                                                      │
│   └── Instance (e.g., "default", "prod")                   │
│        ├── config: HashMap<String, ConfigValue>            │
│        ├── environment: HashMap<String, String>            │
│        ├── capabilities: Capabilities                       │
│        └── metadata: InstanceMetadata                       │
│                                                             │
│  Storage:                                                   │
│   - ~/.skill-engine/instances/{skill}/{instance}/config.toml│
│   - Platform Keychain (secrets)                            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Limitations:**
1. Configuration is tightly coupled to specific skill instances
2. No way to share configuration across skills
3. Secrets managed separately without unified abstraction
4. Volume mounts only available for Docker runtime
5. No UI for managing any of these settings
6. Cannot template or inherit configurations

### 2.2 Relevant Codebase Components

| Component | Location | Current Responsibility |
|-----------|----------|----------------------|
| InstanceConfig | `skill-runtime/src/instance.rs` | Per-instance configuration |
| CredentialStore | `skill-runtime/src/credentials.rs` | Platform keychain access |
| SandboxBuilder | `skill-runtime/src/sandbox.rs` | WASI context creation |
| Capabilities | `skill-runtime/src/instance.rs` | Security capabilities |
| Manifest | `skill-runtime/src/manifest.rs` | Skill source definitions |
| HttpHandlers | `skill-http/src/handlers.rs` | API endpoints |

---

## 3. Detailed Requirements

### 3.1 Execution Context Model

#### 3.1.1 Core Data Structure

```rust
/// A complete execution environment definition
pub struct ExecutionContext {
    /// Unique identifier for this context
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// Optional parent context to inherit from
    pub inherits_from: Option<String>,

    /// File and directory mounts
    pub mounts: Vec<Mount>,

    /// Environment variable definitions
    pub environment: EnvironmentConfig,

    /// Secret references
    pub secrets: SecretsConfig,

    /// Resource limits and capabilities
    pub resources: ResourceConfig,

    /// Runtime-specific overrides
    pub runtime_overrides: RuntimeOverrides,

    /// Metadata
    pub metadata: ContextMetadata,
}

pub struct ContextMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub tags: Vec<String>,
    pub version: u32,
}
```

#### 3.1.2 Mount Configuration

```rust
/// File/directory mount specification
pub struct Mount {
    /// Unique identifier within context
    pub id: String,

    /// Mount type
    pub mount_type: MountType,

    /// Host path (supports env var expansion)
    pub source: String,

    /// Path inside execution environment
    pub target: String,

    /// Read-only flag
    pub read_only: bool,

    /// Required or optional
    pub required: bool,

    /// Human-readable description
    pub description: Option<String>,
}

pub enum MountType {
    /// Regular file
    File,
    /// Directory
    Directory,
    /// Docker volume (named volume)
    Volume,
    /// Temporary filesystem (tmpfs)
    Tmpfs { size_mb: u32 },
    /// Config file generated from template
    ConfigFile { template: String },
}
```

#### 3.1.3 Environment Configuration

```rust
pub struct EnvironmentConfig {
    /// Static environment variables
    pub variables: HashMap<String, EnvValue>,

    /// Environment files to load (.env format)
    pub env_files: Vec<EnvFileRef>,

    /// Environment variable prefixes to pass through from host
    pub passthrough_prefixes: Vec<String>,

    /// Specific host env vars to pass through
    pub passthrough_vars: Vec<String>,
}

pub enum EnvValue {
    /// Plain text value
    Plain(String),
    /// Reference to another env var: ${VAR_NAME}
    Reference(String),
    /// Reference to a secret: secret://context/key
    Secret(SecretRef),
    /// Generated value (e.g., UUID, timestamp)
    Generated(GeneratedValue),
    /// Value from file
    FromFile(PathBuf),
}

pub struct EnvFileRef {
    /// Path to .env file (supports glob)
    pub path: String,
    /// Whether file must exist
    pub required: bool,
    /// Optional prefix to add to all vars
    pub prefix: Option<String>,
}

pub enum GeneratedValue {
    Uuid,
    Timestamp,
    RandomString { length: usize },
    Hash { algorithm: String, of: String },
}
```

#### 3.1.4 Secrets Configuration

```rust
pub struct SecretsConfig {
    /// Individual secret definitions
    pub secrets: HashMap<String, SecretDefinition>,

    /// Secret provider configuration
    pub providers: Vec<SecretProvider>,
}

pub struct SecretDefinition {
    /// Secret key name
    pub key: String,

    /// Human-readable description
    pub description: Option<String>,

    /// Whether this secret is required
    pub required: bool,

    /// Provider to use (defaults to platform keychain)
    pub provider: Option<String>,

    /// Environment variable name to inject as
    pub env_var: Option<String>,

    /// File path to write secret to (for file-based secrets)
    pub file_path: Option<String>,

    /// File permissions (octal, e.g., "0600")
    pub file_mode: Option<String>,
}

pub enum SecretProvider {
    /// Platform keychain (default)
    Keychain,
    /// Environment variable (for CI/CD)
    EnvironmentVariable { prefix: String },
    /// File-based secrets
    File { path: String, format: SecretFileFormat },
    /// External secret manager
    External {
        provider_type: ExternalSecretProvider,
        config: HashMap<String, String>,
    },
}

pub enum ExternalSecretProvider {
    Vault,           // HashiCorp Vault
    AwsSecretsManager,
    GcpSecretManager,
    AzureKeyVault,
    OnePassword,     // 1Password CLI
    Doppler,         // Doppler
}

pub enum SecretFileFormat {
    Env,        // KEY=value format
    Json,       // JSON object
    Yaml,       // YAML file
    Raw,        // Single secret per file
}
```

#### 3.1.5 Resource Configuration

```rust
pub struct ResourceConfig {
    /// CPU limits
    pub cpu: Option<CpuConfig>,

    /// Memory limits
    pub memory: Option<MemoryConfig>,

    /// Network configuration
    pub network: NetworkConfig,

    /// Filesystem capabilities
    pub filesystem: FilesystemConfig,

    /// Execution limits
    pub execution: ExecutionLimits,
}

pub struct CpuConfig {
    /// CPU cores (e.g., "0.5", "2")
    pub limit: String,
    /// CPU shares for relative priority
    pub shares: Option<u32>,
}

pub struct MemoryConfig {
    /// Memory limit (e.g., "512m", "2g")
    pub limit: String,
    /// Swap limit
    pub swap: Option<String>,
    /// Memory reservation (soft limit)
    pub reservation: Option<String>,
}

pub struct NetworkConfig {
    /// Whether network access is allowed
    pub enabled: bool,
    /// Network mode for Docker (none, bridge, host)
    pub mode: Option<String>,
    /// Allowed outbound hosts (whitelist)
    pub allowed_hosts: Option<Vec<String>>,
    /// Blocked hosts (blacklist)
    pub blocked_hosts: Option<Vec<String>>,
    /// DNS servers
    pub dns: Option<Vec<String>>,
}

pub struct FilesystemConfig {
    /// Read-only root filesystem
    pub read_only_root: bool,
    /// Paths that are writable (within read-only root)
    pub writable_paths: Vec<String>,
    /// Maximum file size that can be created
    pub max_file_size: Option<String>,
    /// Maximum total disk usage
    pub max_disk_usage: Option<String>,
}

pub struct ExecutionLimits {
    /// Maximum execution time
    pub timeout_seconds: Option<u64>,
    /// Maximum concurrent executions
    pub max_concurrent: Option<u32>,
    /// Rate limiting
    pub rate_limit: Option<RateLimit>,
}

pub struct RateLimit {
    /// Requests per window
    pub requests: u32,
    /// Window duration in seconds
    pub window_seconds: u32,
}
```

#### 3.1.6 Runtime Overrides

```rust
pub struct RuntimeOverrides {
    /// WASM-specific configuration
    pub wasm: Option<WasmOverrides>,

    /// Docker-specific configuration
    pub docker: Option<DockerOverrides>,

    /// Native execution configuration
    pub native: Option<NativeOverrides>,
}

pub struct WasmOverrides {
    /// Stack size in bytes
    pub stack_size: Option<usize>,
    /// Enable/disable specific WASI capabilities
    pub wasi_capabilities: HashMap<String, bool>,
    /// Fuel limit for execution metering
    pub fuel_limit: Option<u64>,
}

pub struct DockerOverrides {
    /// Override container image
    pub image: Option<String>,
    /// Additional docker run arguments
    pub extra_args: Vec<String>,
    /// Override entrypoint
    pub entrypoint: Option<String>,
    /// Override command
    pub command: Option<Vec<String>>,
    /// User to run as
    pub user: Option<String>,
    /// GPU configuration
    pub gpus: Option<String>,
    /// Platform for multi-arch
    pub platform: Option<String>,
    /// Privileged mode (dangerous!)
    pub privileged: bool,
    /// Security options
    pub security_opt: Vec<String>,
    /// Sysctls
    pub sysctls: HashMap<String, String>,
}

pub struct NativeOverrides {
    /// Working directory
    pub working_dir: Option<String>,
    /// Shell to use
    pub shell: Option<String>,
    /// PATH additions
    pub path_additions: Vec<String>,
}
```

### 3.2 Context Inheritance

Execution contexts support single inheritance with override semantics:

```yaml
# base-python.context.yaml
id: base-python
name: Base Python Environment
environment:
  variables:
    PYTHONUNBUFFERED: "1"
    PYTHONDONTWRITEBYTECODE: "1"
resources:
  memory:
    limit: "512m"
  network:
    enabled: true

---
# ml-python.context.yaml
id: ml-python
name: ML Python Environment
inherits_from: base-python  # Inherits all settings from base-python
environment:
  variables:
    # Adds to parent's variables
    CUDA_VISIBLE_DEVICES: "0"
resources:
  memory:
    limit: "4g"  # Overrides parent's 512m
  cpu:
    limit: "4"   # Adds CPU limit
mounts:
  - id: model-cache
    mount_type: Directory
    source: "~/.cache/huggingface"
    target: "/root/.cache/huggingface"
    read_only: false
```

**Inheritance Rules:**
1. Scalar values (strings, numbers, booleans): Child overrides parent
2. Arrays: Child replaces parent's array entirely (use `+` prefix to append)
3. Maps: Deep merge with child taking precedence
4. Secrets: Child can add new secrets or override provider for existing ones

### 3.3 Context Storage

#### 3.3.1 File System Layout

```
~/.skill-engine/
├── contexts/
│   ├── index.json              # Context registry
│   ├── base-python/
│   │   ├── context.toml        # Context definition
│   │   └── .secrets/           # Local secret cache (encrypted)
│   └── ml-python/
│       └── context.toml
├── instances/
│   └── {skill}/
│       └── {instance}/
│           ├── config.toml     # Instance config (now references context)
│           └── context.toml    # Optional instance-specific overrides
└── templates/
    └── contexts/
        ├── default.toml        # Default context template
        └── high-security.toml  # Security-focused template
```

#### 3.3.2 Context Definition Format (TOML)

```toml
# context.toml
[context]
id = "api-production"
name = "API Production Context"
description = "Production environment for API skills"
inherits_from = "base-api"
tags = ["production", "api", "high-security"]

[mounts]
[[mounts.entries]]
id = "config"
mount_type = "ConfigFile"
source = """
[api]
endpoint = "${API_ENDPOINT}"
timeout = 30
"""
target = "/etc/app/config.toml"
read_only = true

[[mounts.entries]]
id = "data"
mount_type = "Directory"
source = "/data/api-cache"
target = "/var/cache/api"
read_only = false
required = true
description = "API response cache directory"

[environment]
[environment.variables]
LOG_LEVEL = "info"
API_ENDPOINT = { type = "reference", value = "PRODUCTION_API_URL" }
API_KEY = { type = "secret", ref = "api-production/api-key" }

[[environment.env_files]]
path = ".env.production"
required = false

[environment]
passthrough_prefixes = ["AWS_", "OTEL_"]

[secrets]
[secrets.secrets.api-key]
key = "api-key"
description = "Production API authentication key"
required = true
env_var = "API_KEY"

[secrets.secrets.db-password]
key = "db-password"
description = "Database password"
required = true
file_path = "/run/secrets/db-password"
file_mode = "0400"

[resources]
[resources.memory]
limit = "1g"
reservation = "256m"

[resources.network]
enabled = true
allowed_hosts = ["api.example.com", "*.amazonaws.com"]

[resources.execution]
timeout_seconds = 300
max_concurrent = 10

[runtime_overrides]
[runtime_overrides.docker]
user = "1000:1000"
security_opt = ["no-new-privileges"]
```

### 3.4 API Specification

#### 3.4.1 REST Endpoints

```
# Context Management
GET    /api/contexts                    # List all contexts
POST   /api/contexts                    # Create new context
GET    /api/contexts/{id}               # Get context details
PUT    /api/contexts/{id}               # Update context
DELETE /api/contexts/{id}               # Delete context
POST   /api/contexts/{id}/duplicate     # Duplicate a context
GET    /api/contexts/{id}/effective     # Get resolved/merged context

# Context Validation
POST   /api/contexts/validate           # Validate context definition
POST   /api/contexts/{id}/test          # Test context (dry run)

# Secret Management within Context
GET    /api/contexts/{id}/secrets       # List secret keys (not values!)
PUT    /api/contexts/{id}/secrets/{key} # Set secret value
DELETE /api/contexts/{id}/secrets/{key} # Delete secret
POST   /api/contexts/{id}/secrets/verify# Verify all secrets are set

# Context Templates
GET    /api/contexts/templates          # List available templates
POST   /api/contexts/from-template      # Create context from template

# Context-Instance Binding
PUT    /api/skills/{skill}/instances/{instance}/context
                                        # Bind context to instance
GET    /api/skills/{skill}/instances/{instance}/context
                                        # Get bound context
DELETE /api/skills/{skill}/instances/{instance}/context
                                        # Unbind context

# Context Usage
GET    /api/contexts/{id}/usage         # List skills/instances using context
```

#### 3.4.2 Request/Response Examples

**Create Context:**
```http
POST /api/contexts
Content-Type: application/json

{
  "name": "My API Context",
  "description": "Context for API integrations",
  "inherits_from": "base-api",
  "environment": {
    "variables": {
      "LOG_LEVEL": "debug"
    }
  },
  "secrets": {
    "secrets": {
      "api-key": {
        "key": "api-key",
        "description": "API authentication key",
        "required": true,
        "env_var": "API_KEY"
      }
    }
  }
}
```

**Response:**
```json
{
  "id": "my-api-context-a1b2c3",
  "name": "My API Context",
  "description": "Context for API integrations",
  "inherits_from": "base-api",
  "environment": {
    "variables": {
      "LOG_LEVEL": "debug"
    }
  },
  "secrets": {
    "secrets": {
      "api-key": {
        "key": "api-key",
        "description": "API authentication key",
        "required": true,
        "env_var": "API_KEY",
        "is_set": false
      }
    }
  },
  "metadata": {
    "created_at": "2025-12-22T10:00:00Z",
    "updated_at": "2025-12-22T10:00:00Z",
    "version": 1
  }
}
```

**Set Secret:**
```http
PUT /api/contexts/my-api-context-a1b2c3/secrets/api-key
Content-Type: application/json

{
  "value": "sk-1234567890abcdef"
}
```

**Response:**
```json
{
  "success": true,
  "key": "api-key",
  "updated_at": "2025-12-22T10:01:00Z"
}
```

### 3.5 Web UI Requirements

#### 3.5.1 Context List View

```
┌─────────────────────────────────────────────────────────────────┐
│ Execution Contexts                              [+ New Context] │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ 🔷 base-python                                    [Template]│ │
│ │ Base Python Environment                                     │ │
│ │ 3 skills using • Last modified 2 days ago                  │ │
│ │ Tags: python, base                                         │ │
│ └─────────────────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ 🔷 ml-python                              [Inherits: base]  │ │
│ │ ML Python Environment                                       │ │
│ │ 1 skill using • ⚠️ 1 secret missing                        │ │
│ │ Tags: ml, gpu                                              │ │
│ └─────────────────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ 🔷 api-production                                           │ │
│ │ API Production Context                           [In Use]   │ │
│ │ 5 skills using • All secrets configured ✓                  │ │
│ │ Tags: production, api                                       │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

#### 3.5.2 Context Detail/Edit View

**Tab: Overview**
```
┌─────────────────────────────────────────────────────────────────┐
│ ← Back    api-production                    [Duplicate] [Delete]│
├─────────────────────────────────────────────────────────────────┤
│ [Overview] [Mounts] [Environment] [Secrets] [Resources] [Usage] │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ Name: [api-production________________]                          │
│                                                                 │
│ Description:                                                    │
│ [Production environment for API skills_______________________]  │
│                                                                 │
│ Inherits From: [base-api ▼]                                    │
│                                                                 │
│ Tags: [production] [api] [high-security] [+]                   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ Status Summary                                              │ │
│ │                                                             │ │
│ │ ✓ All required secrets configured                          │ │
│ │ ✓ All required mounts available                            │ │
│ │ ⚠️ 1 optional mount not found                               │ │
│ │ ✓ Context validation passed                                 │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│                                         [Test Context] [Save]   │
└─────────────────────────────────────────────────────────────────┘
```

**Tab: Mounts**
```
┌─────────────────────────────────────────────────────────────────┐
│ [Overview] [Mounts] [Environment] [Secrets] [Resources] [Usage] │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ File & Directory Mounts                            [+ Add Mount]│
│                                                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ 📁 config                                        [Required] │ │
│ │ Type: Config File                                           │ │
│ │ Target: /etc/app/config.toml                               │ │
│ │ Mode: Read-only                                            │ │
│ │                                    [Edit Template] [Delete] │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ 📂 data                                         [Required]  │ │
│ │ Type: Directory                                             │ │
│ │ Source: /data/api-cache                                     │ │
│ │ Target: /var/cache/api                                      │ │
│ │ Mode: Read-write                                            │ │
│ │ Status: ✓ Directory exists                                  │ │
│ │                                              [Edit] [Delete]│ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ 📂 logs (inherited from base-api)               [Optional]  │ │
│ │ Type: Directory                                             │ │
│ │ Source: ~/.skill-engine/logs                               │ │
│ │ Target: /var/log/app                                        │ │
│ │ Status: ⚠️ Directory not found                              │ │
│ │                                       [Override] [Disable]  │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

**Tab: Secrets**
```
┌─────────────────────────────────────────────────────────────────┐
│ [Overview] [Mounts] [Environment] [Secrets] [Resources] [Usage] │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ Secrets                                          [+ Add Secret] │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ 🔐 api-key                                      [Required]  │ │
│ │ Production API authentication key                           │ │
│ │ Injected as: $API_KEY                                       │ │
│ │ Status: ✓ Configured                                        │ │
│ │                                      [Update] [View] [Delete]│ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ 🔐 db-password                                  [Required]  │ │
│ │ Database password                                           │ │
│ │ Injected as: File at /run/secrets/db-password              │ │
│ │ Status: ✓ Configured                                        │ │
│ │                                      [Update] [View] [Delete]│ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ 🔐 analytics-token                              [Optional]  │ │
│ │ Analytics service token                                     │ │
│ │ Injected as: $ANALYTICS_TOKEN                              │ │
│ │ Status: ⚠️ Not configured                                   │ │
│ │                                             [Set] [Delete]  │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ ┌────────────────────────────────────────────────────────────┐  │
│ │ Set Secret Value                                           │  │
│ │                                                            │  │
│ │ Secret: [api-key ▼]                                        │  │
│ │                                                            │  │
│ │ Value: [••••••••••••••••••••__________________]           │  │
│ │        [Show] [Generate Random]                            │  │
│ │                                                            │  │
│ │                                    [Cancel] [Save Secret]  │  │
│ └────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

**Tab: Resources**
```
┌─────────────────────────────────────────────────────────────────┐
│ [Overview] [Mounts] [Environment] [Secrets] [Resources] [Usage] │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ Resource Limits                                                 │
│                                                                 │
│ ┌─ CPU ─────────────────────────────────────────────────────┐   │
│ │ Limit: [2_____] cores                                     │   │
│ │ [x] Inherit from parent (1 core)                          │   │
│ └───────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─ Memory ──────────────────────────────────────────────────┐   │
│ │ Limit: [1g____]          Reservation: [256m___]           │   │
│ │ Swap:  [512m__]          [ ] Disable swap                 │   │
│ └───────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─ Network ─────────────────────────────────────────────────┐   │
│ │ [x] Enable network access                                 │   │
│ │                                                           │   │
│ │ Mode: [Bridge ▼]                                          │   │
│ │                                                           │   │
│ │ Allowed hosts (whitelist):                                │   │
│ │ [api.example.com_______________________________] [+]      │   │
│ │ [*.amazonaws.com_______________________________]          │   │
│ │                                                           │   │
│ │ [ ] Block specific hosts                                  │   │
│ └───────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─ Execution ───────────────────────────────────────────────┐   │
│ │ Timeout: [300___] seconds    Max concurrent: [10___]      │   │
│ │                                                           │   │
│ │ Rate limiting:                                            │   │
│ │ [ ] Enable rate limiting                                  │   │
│ │ [100] requests per [60] seconds                           │   │
│ └───────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─ Filesystem ──────────────────────────────────────────────┐   │
│ │ [ ] Read-only root filesystem                             │   │
│ │ Max file size: [100m___]   Max disk usage: [1g____]       │   │
│ └───────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

#### 3.5.3 Context Binding in Skill View

```
┌─────────────────────────────────────────────────────────────────┐
│ ← Back    github-skill                                          │
├─────────────────────────────────────────────────────────────────┤
│ [Overview] [Tools] [Instances]                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ Instances                                        [+ New Instance]│
│                                                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ default                                          [Primary]  │ │
│ │                                                             │ │
│ │ Execution Context: [api-production ▼]      [Manage Context] │ │
│ │                                                             │ │
│ │ Context Status:                                             │ │
│ │ ✓ All secrets configured                                    │ │
│ │ ✓ All mounts available                                      │ │
│ │                                                             │ │
│ │ Instance Overrides:                         [Edit Overrides] │ │
│ │ • LOG_LEVEL: debug (overrides context's "info")            │ │
│ │                                                             │ │
│ │                                    [Execute Tool] [Delete]  │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ ci-testing                                                  │ │
│ │                                                             │ │
│ │ Execution Context: [ci-environment ▼]      [Manage Context] │ │
│ │                                                             │ │
│ │ Context Status:                                             │ │
│ │ ⚠️ Secrets configured via environment variables             │ │
│ │ ✓ All mounts available                                      │ │
│ │                                                             │ │
│ │                                    [Execute Tool] [Delete]  │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

#### 3.5.4 New Context Wizard

```
┌─────────────────────────────────────────────────────────────────┐
│                     Create New Context                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ Step 1 of 4: Basic Information                                  │
│ ───────────────────────────────                                 │
│                                                                 │
│ Start from:                                                     │
│ (•) Blank context                                               │
│ ( ) Template: [Select template... ▼]                           │
│ ( ) Existing context: [Select context... ▼]                    │
│                                                                 │
│ Name: [my-new-context_________________________]                 │
│                                                                 │
│ Description:                                                    │
│ [________________________________________________]             │
│ [________________________________________________]             │
│                                                                 │
│ Inherit from: [None ▼]                                         │
│               (Settings will be inherited and can be overridden)│
│                                                                 │
│                                         [Cancel] [Next →]       │
└─────────────────────────────────────────────────────────────────┘
```

### 3.6 CLI Commands

```bash
# Context management
skill-engine context list                    # List all contexts
skill-engine context show <id>               # Show context details
skill-engine context create <name>           # Create new context (interactive)
skill-engine context create <name> --from-template=<template>
skill-engine context create <name> --from=<context-id>
skill-engine context delete <id>             # Delete context
skill-engine context export <id> [--output=file.toml]
skill-engine context import <file.toml>

# Context editing
skill-engine context edit <id>               # Open in $EDITOR
skill-engine context set <id> <key> <value>  # Set config value
skill-engine context env <id> <key>=<value>  # Set environment variable

# Secret management
skill-engine context secret list <id>        # List secrets (keys only)
skill-engine context secret set <id> <key>   # Set secret (prompts for value)
skill-engine context secret set <id> <key> --from-env=<VAR>
skill-engine context secret set <id> <key> --from-file=<file>
skill-engine context secret delete <id> <key>
skill-engine context secret verify <id>      # Verify all secrets are set

# Mount management
skill-engine context mount add <id> --source=<path> --target=<path>
skill-engine context mount add <id> --type=tmpfs --target=<path> --size=100m
skill-engine context mount list <id>
skill-engine context mount remove <id> <mount-id>

# Context binding
skill-engine instance set-context <skill> <instance> <context-id>
skill-engine instance get-context <skill> <instance>

# Validation and testing
skill-engine context validate <id>           # Validate context definition
skill-engine context test <id>               # Test context (dry run)
skill-engine context effective <id>          # Show resolved/merged context
```

---

## 4. Technical Implementation

### 4.1 Component Changes

#### 4.1.1 New Crate: `skill-context`

```
crates/skill-context/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── context.rs          # ExecutionContext struct
│   ├── mounts.rs           # Mount handling
│   ├── environment.rs      # Environment variable processing
│   ├── secrets.rs          # Secret management
│   ├── resources.rs        # Resource configuration
│   ├── inheritance.rs      # Context inheritance logic
│   ├── validation.rs       # Context validation
│   ├── storage.rs          # Filesystem persistence
│   └── providers/          # Secret providers
│       ├── mod.rs
│       ├── keychain.rs
│       ├── env.rs
│       ├── file.rs
│       └── external.rs     # External providers (Vault, AWS, etc.)
```

#### 4.1.2 Changes to `skill-runtime`

```rust
// executor.rs changes
pub struct SkillExecutor {
    // ... existing fields ...
    context_manager: ContextManager,  // NEW
}

impl SkillExecutor {
    pub async fn execute_tool_with_context(
        &self,
        tool_name: &str,
        args: &str,
        context: &ExecutionContext,  // NEW: explicit context
    ) -> Result<ExecutionResult> {
        // 1. Resolve context (apply inheritance)
        let resolved = self.context_manager.resolve(context)?;

        // 2. Validate context (secrets, mounts)
        resolved.validate()?;

        // 3. Prepare mounts
        let mounts = self.prepare_mounts(&resolved)?;

        // 4. Resolve secrets
        let secrets = self.context_manager.resolve_secrets(&resolved).await?;

        // 5. Build environment
        let env = self.build_environment(&resolved, &secrets)?;

        // 6. Create sandbox with context
        let sandbox = SandboxBuilder::new(&self.instance_name, instance_dir)
            .with_mounts(mounts)
            .with_environment(env)
            .with_resources(&resolved.resources)
            .build()?;

        // 7. Execute
        self.execute_in_sandbox(sandbox, tool_name, args).await
    }
}
```

#### 4.1.3 Changes to `skill-http`

New API routes:

```rust
// routes/contexts.rs (new file)
pub fn context_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_contexts).post(create_context))
        .route("/:id", get(get_context).put(update_context).delete(delete_context))
        .route("/:id/duplicate", post(duplicate_context))
        .route("/:id/effective", get(get_effective_context))
        .route("/:id/test", post(test_context))
        .route("/:id/secrets", get(list_secrets))
        .route("/:id/secrets/:key", put(set_secret).delete(delete_secret))
        .route("/:id/secrets/verify", post(verify_secrets))
        .route("/validate", post(validate_context))
        .route("/templates", get(list_templates))
        .route("/from-template", post(create_from_template))
}
```

#### 4.1.4 Changes to `skill-web`

New React components:

```
crates/skill-web/src/
├── components/
│   ├── contexts/
│   │   ├── ContextList.tsx
│   │   ├── ContextDetail.tsx
│   │   ├── ContextForm.tsx
│   │   ├── ContextWizard.tsx
│   │   ├── MountEditor.tsx
│   │   ├── EnvironmentEditor.tsx
│   │   ├── SecretsManager.tsx
│   │   ├── ResourcesConfig.tsx
│   │   └── ContextSelector.tsx    # Dropdown for binding
│   └── ...
├── hooks/
│   ├── useContexts.ts
│   ├── useContextMutations.ts
│   └── useSecrets.ts
└── pages/
    ├── ContextsPage.tsx
    └── ContextDetailPage.tsx
```

### 4.2 Database Schema (if using SQLite)

```sql
-- Context definitions
CREATE TABLE contexts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    inherits_from TEXT REFERENCES contexts(id),
    definition_json TEXT NOT NULL,  -- Full context as JSON
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT,
    version INTEGER DEFAULT 1
);

CREATE TABLE context_tags (
    context_id TEXT REFERENCES contexts(id),
    tag TEXT NOT NULL,
    PRIMARY KEY (context_id, tag)
);

-- Secret metadata (not values!)
CREATE TABLE context_secrets (
    context_id TEXT REFERENCES contexts(id),
    key TEXT NOT NULL,
    description TEXT,
    required BOOLEAN DEFAULT FALSE,
    env_var TEXT,
    file_path TEXT,
    provider TEXT DEFAULT 'keychain',
    is_set BOOLEAN DEFAULT FALSE,
    updated_at TIMESTAMP,
    PRIMARY KEY (context_id, key)
);

-- Context-instance bindings
CREATE TABLE instance_contexts (
    skill_name TEXT NOT NULL,
    instance_name TEXT NOT NULL,
    context_id TEXT REFERENCES contexts(id),
    override_json TEXT,  -- Instance-specific overrides
    PRIMARY KEY (skill_name, instance_name)
);

-- Audit log
CREATE TABLE context_audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    context_id TEXT,
    action TEXT NOT NULL,  -- create, update, delete, secret_set, secret_delete
    actor TEXT,
    details_json TEXT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### 4.3 Migration Strategy

#### Phase 1: Backward Compatible
- Add new context system alongside existing InstanceConfig
- InstanceConfig continues to work as before
- New contexts are optional

#### Phase 2: Migration Tools
- CLI command to migrate InstanceConfig to ExecutionContext
- Auto-detect and suggest migration for existing instances

#### Phase 3: Default to Contexts
- New instances created with contexts by default
- Legacy mode flag for InstanceConfig

#### Phase 4: Deprecation
- Deprecation warnings for InstanceConfig
- Documentation for migration
- (Eventually) Remove InstanceConfig

---

## 5. Security Considerations

### 5.1 Secret Handling

1. **Never log secret values** - Only log secret keys
2. **Memory safety** - Use `Zeroizing<String>` for all secret values
3. **No secrets in config files** - Only references (`keyring://`, `secret://`)
4. **Audit logging** - Log all secret access, modification, deletion
5. **Provider isolation** - Each provider handles its own security model

### 5.2 Context Validation

1. **Mount validation** - Verify paths exist and are accessible
2. **Permission checks** - Ensure user has access to mounted directories
3. **Secret verification** - Validate all required secrets are set before execution
4. **Resource limit enforcement** - Prevent resource exhaustion attacks

### 5.3 API Security

1. **Authentication** - Require authentication for secret operations
2. **Rate limiting** - Prevent brute-force attacks on secrets
3. **Input validation** - Sanitize all context definitions
4. **Path traversal prevention** - Validate mount paths

---

## 6. Testing Strategy

### 6.1 Unit Tests

- Context inheritance resolution
- Environment variable expansion
- Secret reference parsing
- Resource limit validation
- Mount path validation

### 6.2 Integration Tests

- Full context creation → execution flow
- Secret provider integration (keychain, env, file)
- Context migration from InstanceConfig
- API endpoint testing

### 6.3 E2E Tests

- UI context wizard flow
- Secret management through UI
- Context binding to instances
- Execution with different context configurations

---

## 7. Documentation Requirements

### 7.1 User Documentation

- Execution Contexts Overview
- Creating Your First Context
- Managing Secrets
- Context Inheritance Guide
- Migrating from InstanceConfig
- Secret Provider Configuration

### 7.2 API Documentation

- OpenAPI/Swagger specification for all new endpoints
- Request/response examples
- Error code documentation

### 7.3 Developer Documentation

- Context system architecture
- Adding new secret providers
- Context validation hooks
- Custom mount types

---

## 8. Rollout Plan

### Phase 1: Foundation (Tasks 1-10)
- Core data structures
- Storage layer
- Basic CRUD operations
- CLI commands

### Phase 2: Secret Management (Tasks 11-20)
- Secret provider abstraction
- Keychain integration
- Environment variable provider
- File-based secrets

### Phase 3: Runtime Integration (Tasks 21-30)
- Context resolution in executor
- Mount preparation
- Environment building
- Resource enforcement

### Phase 4: API Layer (Tasks 31-40)
- REST endpoints
- Validation endpoints
- Context testing (dry run)

### Phase 5: Web UI (Tasks 41-60)
- Context list/detail views
- Context wizard
- Secret management UI
- Resource configuration UI
- Context binding UI

### Phase 6: Advanced Features (Tasks 61-70)
- External secret providers (Vault, AWS, etc.)
- Context templates
- Context sharing/export
- Migration tools

### Phase 7: Polish & Documentation (Tasks 71-80)
- Comprehensive testing
- Documentation
- Performance optimization
- Security audit

---

## 9. Open Questions

1. **Sharing contexts across projects?** Should contexts be shareable between different skill-engine installations?

2. **Context versioning?** Should we support versioned contexts for rollback?

3. **Remote context storage?** Should contexts be syncable to a remote service?

4. **Context dependencies?** Should contexts be able to depend on other contexts (not just inherit)?

5. **Dynamic secrets?** Should we support secrets that are refreshed periodically (e.g., rotating credentials)?

6. **Context locking?** Should we prevent editing a context while skills are using it?

---

## 10. Appendix

### A. Environment Variable Expansion Syntax

```
${VAR}              - Required variable
${VAR:-default}     - Variable with default
${VAR:?error}       - Required with custom error message
${VAR:+alt}         - Use 'alt' if VAR is set
${!PREFIX*}         - All variables starting with PREFIX
```

### B. Secret Reference Syntax

```
secret://context-id/key         - Secret from specific context
secret://./key                  - Secret from current context
keyring://service/user/key      - Direct keyring reference
env://VAR_NAME                  - Environment variable as secret
file:///path/to/secret          - File containing secret
vault://path/to/secret#key      - HashiCorp Vault
aws-sm://secret-name            - AWS Secrets Manager
```

### C. Mount Type Examples

```toml
# Regular directory mount
[[mounts.entries]]
id = "data"
mount_type = "Directory"
source = "/host/data"
target = "/container/data"
read_only = false

# Read-only file mount
[[mounts.entries]]
id = "config"
mount_type = "File"
source = "/host/config.json"
target = "/etc/app/config.json"
read_only = true

# Named volume (Docker)
[[mounts.entries]]
id = "cache"
mount_type = "Volume"
source = "my-cache-volume"
target = "/var/cache"

# Tmpfs mount
[[mounts.entries]]
id = "temp"
mount_type = "Tmpfs"
target = "/tmp"
[mounts.entries.options]
size_mb = 100

# Generated config file
[[mounts.entries]]
id = "generated-config"
mount_type = "ConfigFile"
target = "/etc/app/config.toml"
template = """
[api]
endpoint = "${API_ENDPOINT}"
key = "${API_KEY}"
"""
```

---

*End of PRD*
