# Skill Engine Manifest Guide

Complete reference for `.skill-engine.toml` manifest files. This guide covers all syntax, patterns, and configuration options.

## Table of Contents

- [Overview](#overview)
- [Basic Structure](#basic-structure)
- [Version](#version)
- [Skills Section](#skills-section)
- [Source Types](#source-types)
- [Runtime Types](#runtime-types)
- [Instance Configurations](#instance-configurations)
- [Environment Variables](#environment-variables)
- [Capabilities](#capabilities)
- [Service Dependencies](#service-dependencies)
- [Docker Runtime Configuration](#docker-runtime-configuration)
- [Defaults](#defaults)
- [Complete Example](#complete-example)

## Overview

The `.skill-engine.toml` manifest file is the **declarative configuration** for Skill Engine. It defines:

- Which skills are available
- Where skills come from (sources)
- How skills run (runtime types)
- Per-environment configurations (instances)
- Permissions and capabilities
- External service dependencies

**Key Benefits:**
- ✅ Version controlled with your project
- ✅ Stateless - no persistent installation needed
- ✅ Reproducible across team members
- ✅ Environment-specific configurations (dev/staging/prod)

## Basic Structure

```toml
# .skill-engine.toml
version = "1"

# Global defaults (optional)
[defaults]
# ... default settings for all skills

# Skill definitions
[skills.<skill-name>]
source = "..."
runtime = "wasm|docker|native"
description = "..."

# Instance configurations
[skills.<skill-name>.instances.<instance-name>]
config.<key> = "value"
env.<KEY> = "value"
capabilities.network_access = true
```

## Version

The manifest version must be specified:

```toml
version = "1"
```

**Current version:** `1`

This field is required and must be the first line in your manifest.

## Skills Section

Each skill is defined under `[skills.<skill-name>]`:

```toml
[skills.git]
source = "./examples/native-skills/git-skill"
description = "Git version control operations"
runtime = "native"
default_instance = "default"  # Optional, defaults to "default"
```

### Required Fields

- **`source`**: Where the skill comes from (path, URL, docker image)

### Optional Fields

- **`runtime`**: Execution environment (`wasm`, `docker`, `native`)
  - Default: `wasm`
- **`description`**: Human-readable description
- **`default_instance`**: Which instance to use by default
  - Default: `"default"`
- **`ref`**: Git reference (branch/tag/commit) for git sources
- **`docker`**: Docker configuration (required if `runtime = "docker"`)
- **`services`**: Array of service dependencies

## Source Types

### Local Path (Relative)

```toml
[skills.myskill]
source = "./examples/wasm-skills/myskill"  # Relative to manifest location
```

**Best for:**
- Skills in your project repository
- Custom skills you're developing

**Notes:**
- Must start with `./` or `../`
- Resolved relative to manifest file location

### Local Path (Absolute)

```toml
[skills.myskill]
source = "/absolute/path/to/skill"
```

**Best for:**
- System-wide skill installations
- Skills in fixed locations

**Notes:**
- Must start with `/`
- Not portable across machines (use with caution)

### Docker Image

```toml
[skills.python-runner]
source = "docker:python:3.12-slim"
runtime = "docker"  # Required

[skills.python-runner.docker]
image = "python:3.12-slim"
entrypoint = "python3"
```

**Best for:**
- Language runtimes (Python, Node.js, Ruby)
- CLI tools packaged as Docker images
- Sandboxed execution environments

**Notes:**
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

**Status:** Coming soon

## Runtime Types

### WASM (Default)

```toml
[skills.github]
source = "./examples/wasm-skills/github-skill"
runtime = "wasm"  # Optional - this is the default
```

**Characteristics:**
- Sandboxed WebAssembly component
- Cross-platform (runs anywhere)
- Fast startup
- Secure by default
- No system dependencies

**Use for:**
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

**Characteristics:**
- Wraps existing CLI tools
- Uses `SKILL.md` documentation format
- Command allowlist for security
- No WebAssembly compilation needed
- Requires system CLI to be installed

**Use for:**
- Wrapping existing tools (kubectl, terraform, git)
- System administration tasks
- Tools with complex native dependencies

**Requirements:**
- `SKILL.md` file in skill directory
- System CLI tool must be installed (e.g., `kubectl` for kubernetes skill)

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

**Characteristics:**
- Runs in Docker container
- Isolated from host system
- Reproducible environments
- Can use any Docker image

**Use for:**
- Media processing (ffmpeg, imagemagick)
- Language runtimes (Python, Node.js, Ruby)
- Tools with complex dependencies
- Security-sensitive operations

**Requirements:**
- Docker daemon running
- `[skills.name.docker]` configuration block

## Instance Configurations

Multiple instances allow different configurations per environment:

```toml
[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"
runtime = "native"

# Development environment
[skills.kubernetes.instances.dev]
config.cluster = "minikube"
config.kubeconfig = "~/.kube/dev-config"
capabilities.network_access = true

# Staging environment
[skills.kubernetes.instances.staging]
config.cluster = "staging-cluster"
config.kubeconfig = "/etc/kube/staging-config"
capabilities.network_access = true

# Production environment
[skills.kubernetes.instances.prod]
config.cluster = "production-cluster"
config.kubeconfig = "/etc/kube/prod-config"
capabilities.network_access = true
capabilities.max_concurrent_requests = 5  # Rate limit for prod
```

### Using Instances

```bash
# Use specific instance
skill run kubernetes:dev get pods

# In Claude Code
> Use the kubernetes:prod skill to list pods
```

### Instance Fields

#### `config.*` - Configuration Values

Pass configuration to the skill:

```toml
[skills.myskill.instances.default]
config.api_url = "https://api.example.com"
config.timeout = "30"
config.retry_count = "3"
```

**Notes:**
- All values are strings
- Support environment variable expansion
- Passed to skill as configuration

#### `env.*` - Environment Variables

Set environment variables for skill execution:

```toml
[skills.myskill.instances.default]
env.API_KEY = "${MY_API_KEY}"
env.LOG_LEVEL = "debug"
env.REGION = "us-east-1"
```

**Notes:**
- Environment variables for the skill process
- Support environment variable expansion
- Uppercase by convention

#### `capabilities` - Permissions

See [Capabilities](#capabilities) section below.

#### `description` - Instance Description

```toml
[skills.myskill.instances.prod]
description = "Production environment with strict rate limits"
config.environment = "production"
```

## Environment Variables

Skill Engine supports environment variable expansion in config and env values:

### Syntax

```toml
# Required - error if not set
config.api_key = "${API_KEY}"

# Optional with default value
config.region = "${AWS_REGION:-us-east-1}"

# Optional with error message
config.token = "${GITHUB_TOKEN:?GitHub token is required}"

# Complex expressions
config.database_url = "postgresql://${DB_USER}:${DB_PASS}@${DB_HOST:-localhost}:${DB_PORT:-5432}/${DB_NAME}"
```

### Patterns

| Pattern | Meaning | Behavior if unset |
|---------|---------|-------------------|
| `${VAR}` | Required variable | **Error** - stops execution |
| `${VAR:-default}` | Optional with default | Uses `default` value |
| `${VAR:?message}` | Required with custom error | **Error** with `message` |

### Examples

```toml
[skills.github.instances.default]
# GitHub token from environment (required)
env.GITHUB_TOKEN = "${SKILL_GITHUB_TOKEN}"
config.api_url = "${GITHUB_API_URL:-https://api.github.com}"

[skills.postgres.instances.prod]
config.host = "${POSTGRES_HOST:-localhost}"
config.port = "${POSTGRES_PORT:-5432}"
config.database = "${POSTGRES_DB:?Database name is required}"
config.username = "${POSTGRES_USER}"
config.password = "${POSTGRES_PASSWORD}"

[skills.aws.instances.default]
env.AWS_ACCESS_KEY_ID = "${AWS_ACCESS_KEY_ID}"
env.AWS_SECRET_ACCESS_KEY = "${AWS_SECRET_ACCESS_KEY}"
env.AWS_REGION = "${AWS_REGION:-us-east-1}"
```

### Environment File

Create a `.env` file for local development:

```bash
# .env (don't commit to git)
SKILL_GITHUB_TOKEN=ghp_xxxxx
POSTGRES_HOST=localhost
POSTGRES_DB=myapp
POSTGRES_USER=admin
POSTGRES_PASSWORD=secret
AWS_REGION=us-west-2
```

Skill Engine will automatically load `.env` from the current directory.

## Capabilities

Capabilities control what skills can access:

```toml
[skills.myskill.instances.default]
capabilities.network_access = true
capabilities.allowed_paths = ["/tmp", "/var/data"]
capabilities.max_concurrent_requests = 10
```

### Fields

#### `network_access` (boolean)

Allow network access (HTTP, HTTPS, TCP, UDP):

```toml
capabilities.network_access = true  # Allow
capabilities.network_access = false  # Deny (default)
```

**Default:** `false`

**Use cases:**
- API integrations (GitHub, Slack, Jira)
- HTTP clients
- Database connections
- External service calls

**Security note:** Only enable for skills that need it.

#### `allowed_paths` (array of strings)

Grant filesystem access to specific paths:

```toml
capabilities.allowed_paths = [
  "/tmp",
  "/var/data",
  "${HOME}/.config",
  "${PWD}"  # Current working directory
]
```

**Default:** `[]` (no additional paths)

**Use cases:**
- Reading configuration files
- Writing output files
- Accessing data directories

**Security note:**
- WASM skills are sandboxed - only specified paths are accessible
- Environment variable expansion supported
- Relative paths resolved from manifest location

#### `max_concurrent_requests` (integer)

Limit concurrent executions of this skill:

```toml
capabilities.max_concurrent_requests = 5
```

**Default:** `10`

**Use cases:**
- Rate limiting for production
- Resource management
- Preventing API quota exhaustion

## Service Dependencies

Skills can declare external services they depend on:

```toml
[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"
runtime = "native"

# Declare service dependency
[[skills.kubernetes.services]]
name = "kubectl-proxy"
description = "Kubernetes API proxy for executing commands"
optional = false
default_port = 8001

[[skills.kubernetes.services]]
name = "metrics-server"
description = "Cluster metrics for monitoring"
optional = true  # Skill works without this
default_port = 10250
```

### Service Fields

- **`name`** (required): Service identifier
- **`description`** (optional): What the service provides
- **`optional`** (boolean): Whether skill can work without it
  - `false`: Skill won't work without this service (default)
  - `true`: Service enhances functionality but isn't required
- **`default_port`** (integer): Default port the service runs on

### Usage

Services are informational - they help users understand dependencies. Skill Engine doesn't automatically start services (use Docker Compose, Kubernetes, or systemd for that).

## Docker Runtime Configuration

When `runtime = "docker"`, configure the container:

```toml
[skills.python-runner]
source = "docker:python:3.12-slim"
runtime = "docker"

[skills.python-runner.docker]
image = "python:3.12-slim"
entrypoint = "python3"
command = ["-u"]  # Optional command args
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
environment = ["PYTHONUNBUFFERED=1"]
memory = "512m"
cpus = "1.0"
network = "none"
rm = true
user = "1000:1000"
read_only = false
platform = "linux/amd64"
```

### Docker Fields

#### `image` (required)

Docker image to use:

```toml
image = "python:3.12-slim"
image = "node:20-alpine"
image = "postgres:16"
```

#### `entrypoint` (optional)

Override container entrypoint:

```toml
entrypoint = "python3"
entrypoint = "/bin/bash"
```

#### `command` (optional)

Arguments passed to entrypoint:

```toml
command = ["-c", "python script.py"]
```

#### `volumes` (array)

Mount host paths into container:

```toml
volumes = [
  "${PWD}:/workdir",
  "/tmp:/tmp",
  "${HOME}/.aws:/root/.aws:ro"  # Read-only
]
```

**Format:** `host_path:container_path` or `host_path:container_path:ro`

**Environment variable expansion:** Supported

#### `working_dir` (optional)

Working directory inside container:

```toml
working_dir = "/workdir"
```

#### `environment` (array)

Environment variables for container:

```toml
environment = [
  "PYTHONUNBUFFERED=1",
  "LOG_LEVEL=debug",
  "API_KEY=${API_KEY}"  # From host environment
]
```

**Format:** `KEY=value`

#### `memory` (optional)

Memory limit:

```toml
memory = "512m"   # 512 megabytes
memory = "1g"     # 1 gigabyte
memory = "2048m"  # 2 gigabytes
```

#### `cpus` (optional)

CPU limit:

```toml
cpus = "0.5"  # Half a CPU
cpus = "2"    # 2 CPUs
cpus = "1.5"  # 1.5 CPUs
```

#### `network` (optional)

Network mode:

```toml
network = "none"    # No network access (default, most secure)
network = "bridge"  # Standard bridge network
network = "host"    # Host networking (least secure)
```

**Default:** `"none"` for security

**Use cases:**
- `none`: Isolated processing (ffmpeg, imagemagick)
- `bridge`: API clients, database connections
- `host`: Tools that need to access host network services

#### `rm` (optional)

Remove container after execution:

```toml
rm = true   # Remove after execution (default)
rm = false  # Keep container for debugging
```

**Default:** `true`

#### `user` (optional)

Run as specific user:

```toml
user = "1000:1000"  # UID:GID
user = "node"       # Named user from image
user = "${UID}:${GID}"  # From environment
```

#### `read_only` (optional)

Make root filesystem read-only:

```toml
read_only = true   # Read-only filesystem
read_only = false  # Read-write (default)
```

**Security:** Enable for untrusted code

#### `platform` (optional)

Platform for multi-arch images:

```toml
platform = "linux/amd64"
platform = "linux/arm64"
platform = "linux/arm/v7"
```

#### `gpus` (optional)

GPU access (requires nvidia-container-runtime):

```toml
gpus = "all"      # All GPUs
gpus = "0"        # GPU 0
gpus = "0,1"      # GPUs 0 and 1
```

#### `extra_args` (array, optional)

Additional docker run arguments:

```toml
extra_args = ["--privileged", "--cap-add=SYS_ADMIN"]
```

**Use with caution:** Can bypass security restrictions

### Complete Docker Example

```toml
[skills.ffmpeg]
source = "docker:linuxserver/ffmpeg:latest"
runtime = "docker"
description = "Video and audio processing"

[skills.ffmpeg.docker]
image = "linuxserver/ffmpeg:latest"
entrypoint = "ffmpeg"
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
memory = "2g"
cpus = "4"
network = "none"  # No network needed for processing
rm = true
user = "${UID}:${GID}"
read_only = false  # FFmpeg needs to write temp files

[skills.ffmpeg.instances.default]
# No additional config needed
```

## Defaults

Global defaults for all skills:

```toml
version = "1"

# Apply to all instances unless overridden
[defaults]
capabilities.network_access = false
capabilities.max_concurrent_requests = 10
capabilities.allowed_paths = ["/tmp"]

# Skills inherit these defaults
[skills.github]
source = "./examples/wasm-skills/github-skill"

[skills.github.instances.default]
# Inherits defaults, but can override
capabilities.network_access = true  # Override for this instance
```

### Default Fields

- `capabilities.network_access`
- `capabilities.allowed_paths`
- `capabilities.max_concurrent_requests`

Skill instances can override any default.

## Complete Example

Comprehensive manifest showing all features:

```toml
# .skill-engine.toml
version = "1"

# Global defaults
[defaults]
capabilities.network_access = false
capabilities.max_concurrent_requests = 10

# ============================================================================
# WASM Skill - GitHub API Integration
# ============================================================================

[skills.github]
source = "./examples/wasm-skills/github-skill"
runtime = "wasm"  # Default, can omit
description = "GitHub API integration for repos, issues, and PRs"

[skills.github.instances.default]
config.api_url = "${GITHUB_API_URL:-https://api.github.com}"
env.GITHUB_TOKEN = "${SKILL_GITHUB_TOKEN}"
capabilities.network_access = true

# ============================================================================
# Native Skill - Kubernetes CLI Wrapper
# ============================================================================

[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"
runtime = "native"
description = "Kubernetes cluster management via kubectl"

# Service dependencies
[[skills.kubernetes.services]]
name = "kubectl-proxy"
description = "Kubernetes API proxy"
optional = false
default_port = 8001

# Development instance
[skills.kubernetes.instances.dev]
description = "Development cluster (minikube)"
config.cluster = "minikube"
config.kubeconfig = "${KUBECONFIG:-~/.kube/config}"
capabilities.network_access = true

# Production instance
[skills.kubernetes.instances.prod]
description = "Production cluster with strict limits"
config.cluster = "production-cluster"
config.kubeconfig = "/etc/kube/prod-config"
capabilities.network_access = true
capabilities.max_concurrent_requests = 5  # Rate limit

# ============================================================================
# Docker Runtime - Python Script Execution
# ============================================================================

[skills.python-runner]
source = "docker:python:3.12-slim"
runtime = "docker"
description = "Sandboxed Python script execution"

[skills.python-runner.docker]
image = "python:3.12-slim"
entrypoint = "python3"
command = ["-u"]  # Unbuffered output
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
environment = ["PYTHONUNBUFFERED=1"]
memory = "512m"
cpus = "1.0"
network = "none"  # No network access
rm = true
user = "1000:1000"
read_only = false

[skills.python-runner.instances.default]
# Docker config covers most settings
capabilities.allowed_paths = ["/workdir"]

# ============================================================================
# Database Skill with Multiple Environments
# ============================================================================

[skills.postgres]
source = "./examples/native-skills/postgres-skill"
runtime = "native"
description = "PostgreSQL database operations"

[[skills.postgres.services]]
name = "postgres"
description = "PostgreSQL database server"
optional = false
default_port = 5432

[skills.postgres.instances.dev]
config.host = "${POSTGRES_HOST:-localhost}"
config.port = "${POSTGRES_PORT:-5432}"
config.database = "${POSTGRES_DB:-myapp_dev}"
config.username = "${POSTGRES_USER:-postgres}"
capabilities.network_access = true

[skills.postgres.instances.prod]
config.host = "prod-db.company.com"
config.port = "5432"
config.database = "myapp_production"
config.username = "${POSTGRES_USER}"
capabilities.network_access = true
capabilities.max_concurrent_requests = 20
```

## Best Practices

### 1. Use Environment Variables for Secrets

❌ **Don't:**
```toml
env.API_KEY = "ghp_hardcoded_secret_bad"
```

✅ **Do:**
```toml
env.API_KEY = "${SKILL_GITHUB_TOKEN}"
```

### 2. Provide Default Values

```toml
config.api_url = "${API_URL:-https://api.example.com}"
config.timeout = "${TIMEOUT:-30}"
config.region = "${AWS_REGION:-us-east-1}"
```

### 3. Use Relative Paths

❌ **Don't:**
```toml
source = "/Users/yourname/projects/skill/examples/git-skill"
```

✅ **Do:**
```toml
source = "./examples/native-skills/git-skill"
```

### 4. Enable Network Access Only When Needed

```toml
# Default: no network
[skills.processing]
capabilities.network_access = false

# Only for API clients
[skills.github]
capabilities.network_access = true
```

### 5. Use Descriptive Instance Names

```toml
[skills.db.instances.dev]       # ✅ Clear
[skills.db.instances.staging]   # ✅ Clear
[skills.db.instances.prod]      # ✅ Clear

[skills.db.instances.config1]   # ❌ Unclear
[skills.db.instances.test123]   # ❌ Unclear
```

### 6. Document Instances

```toml
[skills.kubernetes.instances.prod]
description = "Production cluster - requires VPN access and strict rate limiting"
config.cluster = "prod-cluster"
capabilities.max_concurrent_requests = 5
```

### 7. Use Services to Document Dependencies

```toml
[[skills.myskill.services]]
name = "redis"
description = "Redis cache required for session storage"
optional = false
default_port = 6379
```

## Validation

Validate your manifest:

```bash
# Check syntax
skill list --manifest .skill-engine.toml

# Validate and show details
skill manifest validate .skill-engine.toml
```

## Troubleshooting

### Syntax Errors

```bash
# Error: Invalid TOML syntax at line 42
skill list
```

**Solution:** Check TOML syntax, common issues:
- Missing quotes around strings
- Incorrect table names (`[skills.name]` not `[skill.name]`)
- Duplicate keys

### Environment Variable Not Set

```bash
# Error: Environment variable GITHUB_TOKEN not set
skill run github:default list-repos
```

**Solution:** Set the environment variable:
```bash
export GITHUB_TOKEN=ghp_xxxxx
# or create .env file
```

### Source Path Not Found

```bash
# Error: Skill source not found: ./examples/missing-skill
```

**Solution:** Check path is correct and relative to manifest location

### Docker Image Not Found

```bash
# Error: Docker image not found: python:invalid-tag
```

**Solution:** Pull the image or fix the tag:
```bash
docker pull python:3.12-slim
```

## Next Steps

- 📖 [Quick Start Guide](./QUICK_START_CLAUDE_CODE.md) - Get started quickly
- 🔐 [Environment Variables Guide](./ENVIRONMENT_VARIABLES.md) - Manage secrets
- 📦 [Example Manifests](../examples/configs/) - Templates for common setups
- 🛠 [Creating Custom Skills](./CREATING_SKILLS.md) - Build your own

## Reference

- **TOML Spec:** https://toml.io/
- **Environment Variable Expansion:** Similar to Bash/Shell syntax
- **Docker Run Reference:** https://docs.docker.com/engine/reference/run/
