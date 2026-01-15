# Skill Instances

Skill instances (also called variants) allow you to configure multiple versions of the same skill with different settings, credentials, or environments.

## Why Use Instances?

**Common use cases:**
- **Multiple environments**: dev, staging, production
- **Different credentials**: personal vs team accounts
- **Region-specific configs**: us-east-1 vs eu-west-1
- **Testing variations**: different API endpoints, timeouts

## Basic Example

```toml
[skills.kubernetes]
source = "./kubernetes-skill"
runtime = "native"

# Development cluster
[skills.kubernetes.instances.dev]
env.KUBECONFIG = "~/.kube/dev-config"
config.cluster = "minikube"

# Production cluster
[skills.kubernetes.instances.prod]
env.KUBECONFIG = "/etc/kube/prod-config"
config.cluster = "production-gke"
capabilities.timeout_ms = 60000
```

## Using Instances

### Run with Specific Instance

```bash
# Use dev instance
skill run kubernetes@dev get --resource pods

# Use prod instance
skill run kubernetes@prod get --resource pods

# Default instance (if defined)
skill run kubernetes get --resource pods
```

### List Available Instances

```bash
skill list kubernetes
```

Output shows all configured instances:

```
Skill: kubernetes
Instances:
  - dev (default)
  - prod
  - staging
```

## Instance Configuration

### Config Parameters

Instance-specific configuration passed to the skill:

```toml
[skills.api.instances.default]
config.base_url = "https://api.example.com"
config.timeout = "30"
config.retries = "3"
config.rate_limit = "100"
```

**Access in skill**: These become available as parameters to your skill's tools.

### Environment Variables

Instance-specific environment variables:

```toml
[skills.github.instances.work]
env.GITHUB_TOKEN = "${WORK_GITHUB_TOKEN}"
env.GITHUB_ORG = "mycompany"

[skills.github.instances.personal]
env.GITHUB_TOKEN = "${PERSONAL_GITHUB_TOKEN}"
env.GITHUB_ORG = "myusername"
```

Usage:

```bash
# Use work GitHub account
skill run github@work list-repos

# Use personal GitHub account
skill run github@personal list-repos
```

### Capabilities per Instance

Different security/resource limits per instance:

```toml
[skills.api.instances.dev]
capabilities.network_access = true
capabilities.allowed_domains = ["*"]  # Allow all
capabilities.timeout_ms = 10000

[skills.api.instances.prod]
capabilities.network_access = true
capabilities.allowed_domains = ["api.example.com"]  # Restricted
capabilities.timeout_ms = 30000
capabilities.max_concurrent_requests = 100
```

## Multi-Environment Pattern

### Example: AWS Skills Across Regions

```toml
[skills.aws]
source = "./aws-skill"
runtime = "wasm"

[skills.aws.instances.us-east-1]
config.region = "us-east-1"
env.AWS_PROFILE = "prod-us"

[skills.aws.instances.eu-west-1]
config.region = "eu-west-1"
env.AWS_PROFILE = "prod-eu"

[skills.aws.instances.ap-south-1]
config.region = "ap-south-1"
env.AWS_PROFILE = "prod-ap"
```

Usage:

```bash
# List S3 buckets in US
skill run aws@us-east-1 s3-list

# List S3 buckets in EU
skill run aws@eu-west-1 s3-list
```

### Example: Database Connections

```toml
[skills.postgres]
source = "./postgres-skill"
runtime = "docker"

[skills.postgres.instances.local]
config.host = "localhost"
config.port = "5432"
config.database = "dev_db"
env.PGPASSWORD = "${LOCAL_DB_PASSWORD}"

[skills.postgres.instances.staging]
config.host = "staging-db.example.com"
config.port = "5432"
config.database = "staging_db"
env.PGPASSWORD = "${STAGING_DB_PASSWORD}"
capabilities.network_access = true
capabilities.allowed_domains = ["staging-db.example.com"]

[skills.postgres.instances.prod]
config.host = "prod-db.example.com"
config.port = "5432"
config.database = "prod_db"
env.PGPASSWORD = "${PROD_DB_PASSWORD}"
capabilities.network_access = true
capabilities.allowed_domains = ["prod-db.example.com"]
capabilities.read_only = true  # Safety: read-only in prod
```

## Default Instance

If you don't specify an instance, Skill Engine looks for:

1. Instance named `default`
2. First defined instance
3. Skill-level config (no instance)

**Recommended pattern**: Always define a `default` instance:

```toml
[skills.myskill]
source = "./myskill"

[skills.myskill.instances.default]
config.api_url = "${API_URL:-https://api.example.com}"
env.LOG_LEVEL = "info"

[skills.myskill.instances.debug]
config.api_url = "http://localhost:8080"
env.LOG_LEVEL = "debug"
```

Usage:

```bash
# Uses default instance
skill run myskill mytool

# Uses debug instance
skill run myskill@debug mytool
```

## Dynamic Configuration with CLI

Override instance config at runtime:

```bash
# Override config
skill run kubernetes@dev get --resource pods \
  --config cluster=minikube-local

# Override environment variable
skill run github@work list-repos \
  --env GITHUB_ORG=different-org

# Override multiple
skill run api@staging call-endpoint \
  --config timeout=60 \
  --env DEBUG=true
```

## Instance Inheritance

Instances inherit from skill-level configuration:

```toml
[skills.api]
source = "./api-skill"
runtime = "wasm"
# These apply to ALL instances
config.user_agent = "SkillEngine/1.0.0"
env.LOG_FORMAT = "json"

[skills.api.instances.dev]
# Inherits user_agent and LOG_FORMAT
config.base_url = "http://localhost:3000"

[skills.api.instances.prod]
# Inherits user_agent and LOG_FORMAT
config.base_url = "https://api.example.com"
# Can override inherited values
env.LOG_FORMAT = "text"  # Override
```

## MCP Integration

Instances work seamlessly with Claude Code:

```json
{
  "mcpServers": {
    "skill-engine": {
      "command": "skill",
      "args": ["serve"]
    }
  }
}
```

Claude can access all instances:

```
You: "List pods in the dev kubernetes cluster"

Claude: [Uses skill kubernetes@dev get --resource pods]

You: "Now list pods in prod"

Claude: [Uses skill kubernetes@prod get --resource pods]
```

## Best Practices

### 1. Use Descriptive Instance Names

```toml
# Good
[skills.db.instances.local-dev]
[skills.db.instances.staging-us-east]
[skills.db.instances.prod-primary]

# Bad
[skills.db.instances.db1]
[skills.db.instances.db2]
[skills.db.instances.test]
```

### 2. Always Define Default

```toml
[skills.myskill.instances.default]
# Safe, reasonable defaults
config.timeout = "30"
env.LOG_LEVEL = "info"
```

### 3. Use Environment Variables for Secrets

```toml
# Good - secrets from env
[skills.api.instances.prod]
env.API_KEY = "${API_KEY}"

# Bad - hardcoded secrets
[skills.api.instances.prod]
env.API_KEY = "sk-abc123"  # Never!
```

### 4. Restrict Production Capabilities

```toml
[skills.kubectl.instances.prod]
capabilities.allowed_commands = ["kubectl"]
capabilities.allowed_args = ["get", "describe", "logs"]  # Read-only
capabilities.timeout_ms = 30000
capabilities.max_concurrent_requests = 10
```

### 5. Document Each Instance

```toml
[skills.api.instances.dev]
# Development instance with debug logging and localhost API
config.base_url = "http://localhost:3000"
env.LOG_LEVEL = "debug"

[skills.api.instances.prod]
# Production instance with rate limiting and monitoring
config.base_url = "https://api.example.com"
capabilities.max_concurrent_requests = 100
```

## Troubleshooting

### Instance Not Found

```bash
skill run myskill@nonexistent tool
# Error: Instance 'nonexistent' not found for skill 'myskill'
```

**Fix**: List available instances:

```bash
skill list myskill
```

### Wrong Instance Used

```bash
# Explicitly specify instance
skill run myskill@prod tool

# Check which instance is default
skill info myskill
```

### Config Not Applying

**Check precedence** (highest to lowest):
1. CLI flags (`--config`, `--env`)
2. Instance configuration
3. Skill-level configuration
4. Skill defaults

### Environment Variable Not Expanding

```toml
[skills.api.instances.default]
env.API_KEY = "${API_KEY}"  # Must be set in environment!
```

**Check**:

```bash
echo $API_KEY  # Should show value
skill run api@default tool  # Uses env variable
```

## Examples

### Multi-Cloud Kubernetes

```toml
[skills.kubectl]
source = "./kubectl-skill"

[skills.kubectl.instances.aws-us]
env.KUBECONFIG = "~/.kube/eks-us-config"
config.cloud = "aws"

[skills.kubectl.instances.gcp-eu]
env.KUBECONFIG = "~/.kube/gke-eu-config"
config.cloud = "gcp"

[skills.kubectl.instances.azure-ap]
env.KUBECONFIG = "~/.kube/aks-ap-config"
config.cloud = "azure"
```

### API Testing

```toml
[skills.api-test]
source = "./api-test-skill"

[skills.api-test.instances.local]
config.base_url = "http://localhost:8080"
env.SKIP_TLS_VERIFY = "true"

[skills.api-test.instances.integration]
config.base_url = "https://int.example.com"
env.TEST_USER = "integration-tester"

[skills.api-test.instances.load]
config.base_url = "https://staging.example.com"
config.concurrent_requests = "100"
config.duration_seconds = "300"
```

## Next Steps

- **[Manifest Reference](./manifest.md)** - Complete manifest syntax
- **[Environment Variables](./environment.md)** - Managing credentials
- **[Testing](./testing.md)** - Testing different instances
- **[Security](./advanced/security.md)** - Instance-level security
