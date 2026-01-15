# Manifest Configuration

Complete reference for `.skill-engine.toml` manifest files.

## Overview

The `.skill-engine.toml` manifest file is the declarative configuration for Skill Engine. It defines which skills are available, where they come from, how they run, and their configurations.

**Key Benefits**:
- ✅ Version controlled with your project
- ✅ Stateless - no persistent installation needed
- ✅ Reproducible across team members
- ✅ Environment-specific configurations (dev/staging/prod)
- ✅ Works with Claude Code MCP integration

## Basic Structure

```toml
# .skill-engine.toml
version = "1"

# Global defaults (optional)
[defaults]
timeout_ms = 30000

# Skill definitions
[skills.<skill-name>]
source = "..."
runtime = "wasm|docker|native"
description = "..."

# Instance configurations
[skills.<skill-name>.instances.<instance-name>]
config.<key> = "value"
env.<KEY> = "value"
```

## Quick Start

### Create New Manifest

```bash
# Initialize in current directory
skill init

# This creates .skill-engine.toml with examples
```

### Minimal Example

```toml
version = "1"

[skills.kubernetes]
source = "./kubernetes-skill"
runtime = "native"
description = "Kubernetes cluster management"
```

### Complete Example

```toml
version = "1"

[defaults]
timeout_ms = 30000
max_concurrent = 10

# WASM skill from local path
[skills.github]
source = "./wasm-skills/github-skill"
runtime = "wasm"
description = "GitHub API integration"

[skills.github.instances.default]
config.base_url = "https://api.github.com"

# Native skill with multiple instances
[skills.kubernetes]
source = "./native-skills/kubernetes-skill"
runtime = "native"
description = "Kubernetes management"

[skills.kubernetes.instances.dev]
config.kubeconfig = "~/.kube/dev-config"
env.KUBECONFIG = "~/.kube/dev-config"

[skills.kubernetes.instances.prod]
config.kubeconfig = "~/.kube/prod-config"
env.KUBECONFIG = "~/.kube/prod-config"
capabilities.max_concurrent_requests = 5

# Docker skill
[skills.ffmpeg]
source = "docker:linuxserver/ffmpeg:latest"
runtime = "docker"
description = "Video processing with ffmpeg"

[skills.ffmpeg.docker]
image = "linuxserver/ffmpeg:latest"
entrypoint = "ffmpeg"
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
network = "none"
```

## Version

The manifest version must be specified:

```toml
version = "1"
```

**Current version**: `1`

This field is required and must be the first line in your manifest.

## Skills Section

Each skill is defined under `[skills.<skill-name>]`:

```toml
[skills.git]
source = "./examples/native-skills/git-skill"
description = "Git version control operations"
runtime = "native"
default_instance = "default"  # Optional
```

### Required Fields

- **`source`**: Where the skill comes from (path, URL, docker image)

### Optional Fields

- **`runtime`**: Execution environment (`wasm`, `docker`, `native`) - Default: `wasm`
- **`description`**: Human-readable description
- **`default_instance`**: Which instance to use by default - Default: `"default"`
- **`ref`**: Git reference (branch/tag/commit) for git sources
- **`docker`**: Docker configuration (required if `runtime = "docker"`)
- **`services`**: Array of service dependencies

## Source Types

### Local Path (Relative)

```toml
[skills.myskill]
source = "./examples/wasm-skills/myskill"
```

**Best for**:
- Skills in your project repository
- Custom skills you're developing

**Notes**:
- Must start with `./` or `../`
- Resolved relative to manifest file location

### Local Path (Absolute)

```toml
[skills.myskill]
source = "/absolute/path/to/skill"
```

**Best for**:
- System-wide skill installations
- Skills in fixed locations

**Notes**:
- Must start with `/`
- Not portable across machines

### Docker Image

```toml
[skills.python-runner]
source = "docker:python:3.12-slim"
runtime = "docker"

[skills.python-runner.docker]
image = "python:3.12-slim"
entrypoint = "python3"
```

**Best for**:
- Language runtimes (Python, Node.js, Ruby)
- CLI tools packaged as Docker images
- Sandboxed execution

**Notes**:
- Source format: `docker:<image>:<tag>`
- Requires `runtime = "docker"`
- Requires `[skills.name.docker]` configuration

### Git Repository (Future)

```toml
[skills.myskill]
source = "github:org/repo@v1.0.0"
# or
source = "https://github.com/org/repo.git"
ref = "main"  # Optional: branch, tag, or commit
```

**Status**: Coming soon

## Runtime Types

### WASM (Default)

```toml
[skills.github]
source = "./examples/wasm-skills/github-skill"
runtime = "wasm"  # Optional - this is the default
```

**Characteristics**:
- Sandboxed WebAssembly component
- Cross-platform (runs anywhere)
- Fast startup (~10ms)
- Secure by default
- No system dependencies

**Use for**:
- API integrations (GitHub, Slack, Jira)
- Data processing
- Custom business logic
- Portable skills

### Native

```toml
[skills.kubectl]
source = "./examples/native-skills/kubernetes-skill"
runtime = "native"
```

**Characteristics**:
- Wraps existing CLI tools
- Uses `SKILL.md` documentation format
- Command allowlist for security
- No compilation needed
- Requires system CLI installed

**Use for**:
- Wrapping existing tools (kubectl, terraform, git)
- System administration tasks
- Tools with complex native dependencies

**Requirements**:
- `SKILL.md` file in skill directory
- System CLI tool must be installed

### Docker

```toml
[skills.ffmpeg]
source = "docker:linuxserver/ffmpeg:latest"
runtime = "docker"

[skills.ffmpeg.docker]
image = "linuxserver/ffmpeg:latest"
entrypoint = "ffmpeg"
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
network = "none"
```

**Characteristics**:
- Runs in Docker container
- Isolated from host system
- Reproducible environments
- Can use any Docker image

**Use for**:
- Media processing (ffmpeg, imagemagick)
- Language runtimes
- Tools with complex dependencies
- Security-sensitive operations

**Requirements**:
- Docker daemon running
- `[skills.name.docker]` configuration block

## Instance Configurations

Multiple instances allow different configurations per environment:

```toml
[skills.kubernetes]
source = "./kubernetes-skill"
runtime = "native"

# Development
[skills.kubernetes.instances.dev]
config.cluster = "minikube"
config.kubeconfig = "~/.kube/dev-config"

# Staging
[skills.kubernetes.instances.staging]
config.cluster = "staging-cluster"
config.kubeconfig = "/etc/kube/staging-config"

# Production
[skills.kubernetes.instances.prod]
config.cluster = "production-cluster"
config.kubeconfig = "/etc/kube/prod-config"
capabilities.max_concurrent_requests = 5
```

### Using Instances

```bash
# Use specific instance
skill run kubernetes:dev get pods

# Default instance
skill run kubernetes get pods
```

### Instance Fields

#### config.* - Configuration Values

```toml
[skills.myskill.instances.default]
config.api_url = "https://api.example.com"
config.timeout = "30"
config.retry_count = "3"
```

All values are strings. Supports environment variable expansion.

#### env.* - Environment Variables

```toml
[skills.myskill.instances.default]
env.API_KEY = "${GITHUB_TOKEN}"
env.DEBUG = "true"
```

Passed as environment variables to skill execution.

#### capabilities.* - Permissions

```toml
[skills.myskill.instances.prod]
capabilities.network_access = true
capabilities.max_concurrent_requests = 10
capabilities.timeout_ms = 60000
```

See [Capabilities](#capabilities) section for full reference.

## Environment Variables

### In Configuration Values

```toml
[skills.myskill.instances.default]
config.api_key = "${API_KEY}"
config.base_url = "${BASE_URL:-https://api.default.com}"  # With default
```

**Syntax**:
- `${VAR}`: Required variable (fails if not set)
- `${VAR:-default}`: Optional with default value

### In Environment Section

```toml
[skills.myskill.instances.default]
env.API_KEY = "${GITHUB_TOKEN}"
env.LOG_LEVEL = "debug"
```

Variables are expanded when skill runs.

## Capabilities

Control permissions and resource limits:

### Network Access

```toml
[skills.myskill.instances.default]
capabilities.network_access = true
capabilities.allowed_domains = ["api.github.com", "*.example.com"]
capabilities.blocked_domains = ["tracker.com"]
```

### Filesystem Access

```toml
[skills.myskill.instances.default]
capabilities.filesystem_access = true
capabilities.allowed_paths = ["/data", "/tmp"]
capabilities.read_only_paths = ["/config"]
```

### Resource Limits

```toml
[skills.myskill.instances.default]
capabilities.timeout_ms = 30000           # 30 seconds
capabilities.max_memory_mb = 512          # 512 MB
capabilities.max_concurrent_requests = 5  # Rate limiting
```

### Command Allowlist (Native Runtime)

```toml
[skills.kubectl.instances.default]
capabilities.allowed_commands = ["kubectl", "helm"]
capabilities.allowed_args = ["get", "describe", "logs", "apply"]
capabilities.forbidden_args = ["--token", "--password"]
```

## Docker Configuration

Required for Docker runtime skills:

```toml
[skills.ffmpeg.docker]
# Required
image = "linuxserver/ffmpeg:latest"
entrypoint = "ffmpeg"

# Optional
volumes = [
  "${PWD}:/workdir",
  "/host/input:/input:ro",
  "/host/output:/output:rw"
]
working_dir = "/workdir"
user = "1000:1000"
network = "none"  # or "bridge", "host", "container:name"

# Resource limits
memory = "512m"
cpus = "0.5"

# Security
read_only = true
cap_drop = ["ALL"]
privileged = false
```

### Volume Syntax

```toml
volumes = [
  "/host/path:/container/path",      # Read-write
  "/host/path:/container/path:ro",   # Read-only
  "/host/path:/container/path:rw",   # Explicit read-write
  "${PWD}:/workdir"                   # With env var
]
```

## Service Dependencies

Declare external service dependencies:

```toml
[skills.myapp]
source = "./myapp-skill"
services = [
  { name = "postgres", type = "database" },
  { name = "redis", type = "cache" }
]

[skills.myapp.instances.default]
config.postgres_url = "${DATABASE_URL}"
config.redis_url = "${REDIS_URL}"
```

This metadata helps with:
- Documentation generation
- Dependency validation
- Service mesh integration

## Global Defaults

Set defaults for all skills:

```toml
[defaults]
timeout_ms = 30000
max_concurrent = 10
log_level = "info"

[defaults.capabilities]
network_access = false
filesystem_access = false
```

Skills can override defaults in their instance configuration.

## Environment-Specific Manifests

### Using Environment Variables

```toml
version = "1"

[skills.api]
source = "./api-skill"

[skills.api.instances.default]
config.base_url = "${API_BASE_URL}"
config.timeout = "${API_TIMEOUT:-30}"
```

Set via environment:
```bash
export API_BASE_URL=https://api.prod.com
export API_TIMEOUT=60
skill run api:default call
```

### Multiple Manifest Files

```bash
# Development
skill run --manifest .skill-engine.dev.toml

# Production
skill run --manifest .skill-engine.prod.toml
```

## Validation

### Validate Manifest

```bash
# Check syntax and references
skill validate

# Check specific manifest
skill validate --manifest custom.toml
```

### Common Errors

**Missing source**:
```toml
[skills.myskill]
# Error: source is required
runtime = "native"
```

**Invalid runtime**:
```toml
[skills.myskill]
source = "./myskill"
runtime = "invalid"  # Error: must be wasm, docker, or native
```

**Docker runtime without config**:
```toml
[skills.myskill]
source = "docker:image"
runtime = "docker"
# Error: [skills.myskill.docker] section required
```

## Best Practices

### 1. Use Relative Paths

```toml
# Good - portable
[skills.myskill]
source = "./skills/myskill"

# Bad - not portable
[skills.myskill]
source = "/Users/alice/projects/skills/myskill"
```

### 2. Environment-Specific Instances

```toml
[skills.api]
source = "./api-skill"

[skills.api.instances.dev]
config.base_url = "http://localhost:3000"

[skills.api.instances.prod]
config.base_url = "https://api.prod.com"
capabilities.max_concurrent_requests = 100
```

### 3. Use Environment Variables for Secrets

```toml
# Good - secrets from environment
[skills.api.instances.default]
env.API_KEY = "${API_KEY}"

# Bad - hardcoded secrets
[skills.api.instances.default]
env.API_KEY = "sk-abc123"  # Never do this!
```

### 4. Principle of Least Privilege

```toml
# Good - minimal permissions
[skills.reader.instances.default]
capabilities.network_access = false
capabilities.filesystem_access = true
capabilities.read_only_paths = ["/data"]

# Bad - overly permissive
[skills.reader.instances.default]
capabilities.network_access = true
capabilities.filesystem_access = true
capabilities.allowed_paths = ["/"]
```

### 5. Document Your Skills

```toml
[skills.myskill]
source = "./myskill"
description = "Clear description of what this skill does"  # Always include

# Document instances too
[skills.myskill.instances.prod]
# Production instance with rate limiting and monitoring
capabilities.max_concurrent_requests = 50
```

## Examples

### Multi-Environment API Skill

```toml
version = "1"

[skills.api]
source = "./api-skill"
runtime = "wasm"
description = "REST API integration"

[skills.api.instances.dev]
config.base_url = "http://localhost:8000"
config.timeout = "10"
capabilities.network_access = true
env.LOG_LEVEL = "debug"

[skills.api.instances.staging]
config.base_url = "https://staging-api.example.com"
config.timeout = "30"
capabilities.network_access = true
capabilities.max_concurrent_requests = 20
env.LOG_LEVEL = "info"

[skills.api.instances.prod]
config.base_url = "https://api.example.com"
config.timeout = "30"
capabilities.network_access = true
capabilities.max_concurrent_requests = 100
env.LOG_LEVEL = "warn"
```

### Docker-Based Media Processing

```toml
version = "1"

[skills.ffmpeg]
source = "docker:linuxserver/ffmpeg:latest"
runtime = "docker"
description = "Video/audio processing with ffmpeg"

[skills.ffmpeg.docker]
image = "linuxserver/ffmpeg:latest"
entrypoint = "ffmpeg"
volumes = [
  "${PWD}/input:/input:ro",
  "${PWD}/output:/output:rw"
]
working_dir = "/input"
network = "none"
memory = "2g"
cpus = "2.0"
```

### Native CLI Wrapper

```toml
version = "1"

[skills.kubectl]
source = "./kubectl-skill"
runtime = "native"
description = "Kubernetes cluster management"

[skills.kubectl.instances.dev]
env.KUBECONFIG = "~/.kube/dev-config"
capabilities.allowed_commands = ["kubectl"]
capabilities.allowed_args = ["get", "describe", "logs", "apply", "delete"]
capabilities.timeout_ms = 60000

[skills.kubectl.instances.prod]
env.KUBECONFIG = "/etc/kube/prod-config"
capabilities.allowed_commands = ["kubectl"]
capabilities.allowed_args = ["get", "describe", "logs"]  # No apply/delete
capabilities.forbidden_args = ["--insecure-skip-tls-verify"]
capabilities.max_concurrent_requests = 10
capabilities.timeout_ms = 30000
```

## Related Documentation

- [Environment Variables Guide](./environment.md) - Detailed environment variable handling
- [Security Model](./advanced/security.md) - Security and capability system
- [Skill Development](./developing-skills.md) - Creating skills
- [CLI Reference](../api/cli.md) - Command-line interface

## Further Reading

- [TOML Specification](https://toml.io/) - TOML format details
- [Docker Configuration](https://docs.docker.com/engine/reference/commandline/run/) - Docker options
- [WASI Capabilities](https://github.com/WebAssembly/WASI/blob/main/docs/Capabilities.md) - WASM security
