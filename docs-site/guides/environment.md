# Environment Variables

Complete guide to environment variable management in Skill Engine.

## Overview

Skill Engine uses environment variables for:
- **Configuration**: Skill-specific settings
- **Credentials**: API keys, tokens, passwords
- **Runtime Behavior**: Logging, debugging, feature flags
- **Service Integration**: Database URLs, API endpoints

## Quick Start

### Set Environment Variables

```bash
# Shell environment
export GITHUB_TOKEN=ghp_abc123
export DATABASE_URL=postgresql://localhost/mydb

# Run skill with environment
skill run github create-issue --title "Bug report"
```

### In Manifest

```toml
[skills.github.instances.default]
env.GITHUB_TOKEN = "${GITHUB_TOKEN}"
env.API_BASE_URL = "${API_BASE_URL:-https://api.github.com}"
```

### Secure Storage

```bash
# Store encrypted credential
skill config github --set-credential token

# Credential stored encrypted in:
# ~/.config/skill-engine/credentials.enc
```

## Environment Variable Types

### 1. Skill Configuration

Pass configuration to skills:

```toml
[skills.api.instances.default]
config.base_url = "${API_BASE_URL}"
config.timeout = "${API_TIMEOUT:-30}"
config.retry_count = "${API_RETRY:-3}"
```

**Usage in skill**:
```javascript
// Skill can access via config
const baseUrl = config.base_url;
const timeout = parseInt(config.timeout);
```

### 2. Credentials and Secrets

Sensitive values like API keys:

```toml
[skills.github.instances.default]
env.GITHUB_TOKEN = "${GITHUB_TOKEN}"
env.AWS_ACCESS_KEY_ID = "${AWS_ACCESS_KEY_ID}"
env.AWS_SECRET_ACCESS_KEY = "${AWS_SECRET_ACCESS_KEY}"
```

**Best Practice**: Use encrypted storage instead:
```bash
skill config github --set-credential token
```

### 3. Runtime Variables

Control Skill Engine behavior:

```bash
# Logging level
export SKILL_LOG_LEVEL=debug

# Skills directory
export SKILL_ENGINE_DIR=./custom-skills

# Configuration file
export SKILL_ENGINE_CONFIG=./custom-manifest.toml

# Rust logging
export RUST_LOG=skill_runtime=debug,skill_mcp=trace
```

### 4. Service URLs

External service endpoints:

```toml
[skills.app.instances.default]
env.DATABASE_URL = "${DATABASE_URL}"
env.REDIS_URL = "${REDIS_URL}"
env.S3_BUCKET = "${S3_BUCKET}"
```

## Variable Expansion

### Basic Expansion

```toml
[skills.myskill.instances.default]
# Simple expansion
env.API_KEY = "${GITHUB_TOKEN}"

# Multiple variables
config.url = "${PROTOCOL}://${HOST}:${PORT}"
```

**Example**:
```bash
export PROTOCOL=https
export HOST=api.example.com
export PORT=8443

# Results in: https://api.example.com:8443
```

### Default Values

Provide fallback if variable not set:

```toml
[skills.myskill.instances.default]
env.LOG_LEVEL = "${LOG_LEVEL:-info}"
env.TIMEOUT = "${TIMEOUT:-30}"
env.BASE_URL = "${BASE_URL:-https://api.default.com}"
```

**Syntax**: `${VAR:-default}`

### Required Variables

Fail if variable not set:

```toml
[skills.myskill.instances.default]
env.API_KEY = "${API_KEY}"  # Error if not set
```

**Syntax**: `${VAR}` (no default)

### Escape Sequences

Literal dollar signs:

```toml
[skills.myskill.instances.default]
# Literal $VAR (not expanded)
config.example = "$$VAR"

# Results in: "$VAR"
```

## Secure Credential Management

### Encrypted Storage

```bash
# Store credential (encrypted)
skill config kubernetes --set-credential kubeconfig

# Prompts for value, stores encrypted

# Use in manifest
[skills.kubernetes.instances.default]
# Credentials automatically loaded from secure storage
```

**Storage Location**:
- Config: `~/.config/skill-engine/config.toml`
- Credentials: `~/.config/skill-engine/credentials.enc` (AES-256-GCM)

### Platform-Specific Keychains

Skill Engine integrates with OS keychains:

- **macOS**: Keychain
- **Linux**: Secret Service API (GNOME Keyring, KWallet)
- **Windows**: Windows Credential Manager

**Master key stored securely** in OS keychain, credentials encrypted with it.

### List Credentials

```bash
# List configured credentials
skill config list

# View skill configuration
skill config kubernetes --show
```

### Remove Credentials

```bash
# Remove specific credential
skill config kubernetes --remove-credential kubeconfig

# Remove all skill configuration
skill config kubernetes --reset
```

## Environment-Specific Configurations

### Development

```bash
# .env.development
export API_BASE_URL=http://localhost:3000
export DATABASE_URL=postgresql://localhost/dev
export LOG_LEVEL=debug
export FEATURE_FLAGS=experimental
```

Load with:
```bash
source .env.development
skill run myskill
```

### Staging

```bash
# .env.staging
export API_BASE_URL=https://staging-api.example.com
export DATABASE_URL=postgresql://staging-db/myapp
export LOG_LEVEL=info
export FEATURE_FLAGS=beta
```

### Production

```bash
# .env.production (never commit to git!)
export API_BASE_URL=https://api.example.com
export DATABASE_URL=postgresql://prod-db/myapp
export LOG_LEVEL=warn
export FEATURE_FLAGS=stable
```

**Security**: Use secure secret management (Vault, AWS Secrets Manager) in production.

## Instance-Specific Variables

### Per-Instance Configuration

```toml
[skills.api]
source = "./api-skill"

[skills.api.instances.dev]
env.API_BASE_URL = "http://localhost:3000"
env.LOG_LEVEL = "debug"

[skills.api.instances.staging]
env.API_BASE_URL = "https://staging-api.com"
env.LOG_LEVEL = "info"

[skills.api.instances.prod]
env.API_BASE_URL = "https://api.com"
env.LOG_LEVEL = "warn"
```

**Usage**:
```bash
# Development
skill run api:dev call

# Production
skill run api:prod call
```

## Common Patterns

### Database Connection

```toml
[skills.app.instances.default]
env.DATABASE_URL = "${DATABASE_URL}"
env.DB_POOL_SIZE = "${DB_POOL_SIZE:-10}"
env.DB_TIMEOUT = "${DB_TIMEOUT:-30}"
```

**Environment**:
```bash
export DATABASE_URL=postgresql://user:pass@host:5432/db?sslmode=require
export DB_POOL_SIZE=20
export DB_TIMEOUT=60
```

### API Integration

```toml
[skills.github.instances.default]
env.GITHUB_TOKEN = "${GITHUB_TOKEN}"
env.GITHUB_API_BASE = "${GITHUB_API_BASE:-https://api.github.com}"
env.GITHUB_TIMEOUT = "${GITHUB_TIMEOUT:-30}"
```

**Environment**:
```bash
export GITHUB_TOKEN=ghp_abc123
# GITHUB_API_BASE not set - uses default
# GITHUB_TIMEOUT not set - uses default
```

### Cloud Provider Credentials

```toml
[skills.aws.instances.default]
env.AWS_ACCESS_KEY_ID = "${AWS_ACCESS_KEY_ID}"
env.AWS_SECRET_ACCESS_KEY = "${AWS_SECRET_ACCESS_KEY}"
env.AWS_REGION = "${AWS_REGION:-us-west-2}"
env.AWS_PROFILE = "${AWS_PROFILE:-default}"
```

**Environment**:
```bash
export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
export AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
export AWS_REGION=us-east-1
```

### Feature Flags

```toml
[skills.app.instances.default]
env.FEATURE_NEW_UI = "${FEATURE_NEW_UI:-false}"
env.FEATURE_BETA_API = "${FEATURE_BETA_API:-false}"
env.FEATURE_EXPERIMENTAL = "${FEATURE_EXPERIMENTAL:-false}"
```

**Environment**:
```bash
# Development - enable all features
export FEATURE_NEW_UI=true
export FEATURE_BETA_API=true
export FEATURE_EXPERIMENTAL=true

# Production - stable only
# (all default to false)
```

## Skill Engine Variables

### Runtime Configuration

```bash
# Skills directory
export SKILL_ENGINE_DIR=/custom/skills

# Manifest location
export SKILL_ENGINE_CONFIG=/custom/manifest.toml

# Logging level
export SKILL_LOG_LEVEL=debug  # debug, info, warn, error

# Rust logging (more granular)
export RUST_LOG=skill_runtime=trace,skill_mcp=debug
```

### MCP Server

```bash
# MCP server port (HTTP mode)
export MCP_PORT=3000

# MCP server host
export MCP_HOST=0.0.0.0

# Debug MCP protocol
export MCP_DEBUG=true
```

### HTTP Server

```bash
# HTTP server port
export HTTP_PORT=8080

# CORS origins
export CORS_ORIGINS="http://localhost:3000,https://app.example.com"

# API rate limiting
export RATE_LIMIT=1000  # requests per minute
```

## Environment Files

### .env File

```bash
# .env (root of project)
API_BASE_URL=https://api.example.com
DATABASE_URL=postgresql://localhost/mydb
GITHUB_TOKEN=ghp_abc123
LOG_LEVEL=info
```

**Load with shell**:
```bash
# Manual loading
export $(cat .env | xargs)

# Or use dotenv tool
dotenv skill run myskill
```

### .env.example

```bash
# .env.example (commit to git)
API_BASE_URL=https://api.example.com
DATABASE_URL=postgresql://localhost/mydb
GITHUB_TOKEN=your_token_here
LOG_LEVEL=info
```

**Usage**:
```bash
# Copy and customize
cp .env.example .env
# Edit .env with your values
# Add .env to .gitignore
```

### Per-Environment Files

```
.env.development
.env.staging
.env.production
```

**Load dynamically**:
```bash
# Load based on environment
ENV=${NODE_ENV:-development}
export $(cat .env.$ENV | xargs)
skill run myskill
```

## Security Best Practices

### 1. Never Commit Secrets

```bash
# .gitignore
.env
.env.local
.env.*.local
credentials.json
*.key
*.pem
```

### 2. Use Encrypted Storage

```bash
# Good - encrypted storage
skill config github --set-credential token

# Bad - environment variable in manifest
[skills.github.instances.default]
env.GITHUB_TOKEN = "ghp_abc123"  # Never do this!
```

### 3. Principle of Least Privilege

```bash
# Good - read-only token
export GITHUB_TOKEN=ghp_readonly_abc123

# Bad - admin token for read operations
export GITHUB_TOKEN=ghp_admin_xyz789
```

### 4. Rotate Credentials Regularly

```bash
# Update encrypted credential
skill config github --set-credential token
# Enter new token value
```

### 5. Use Environment-Specific Credentials

```toml
[skills.api.instances.dev]
env.API_KEY = "${DEV_API_KEY}"

[skills.api.instances.prod]
env.API_KEY = "${PROD_API_KEY}"
```

Different keys for different environments.

## Debugging

### View Effective Configuration

```bash
# Show resolved configuration
skill config show kubernetes

# Show environment variables
skill run --dry-run kubernetes get pods
```

### Debug Logging

```bash
# Enable debug logging
export SKILL_LOG_LEVEL=debug
skill run myskill

# Rust-level logging
export RUST_LOG=skill_runtime=trace
skill serve
```

### Check Variable Expansion

```bash
# Test variable expansion
export TEST_VAR=hello
echo ${TEST_VAR:-default}  # Outputs: hello

unset TEST_VAR
echo ${TEST_VAR:-default}  # Outputs: default
```

## Troubleshooting

### "Environment variable not set"

**Error**: `Environment variable API_KEY not set`

**Solution**:
```bash
# Set the variable
export API_KEY=your_key_here

# Or provide a default in manifest
env.API_KEY = "${API_KEY:-default_value}"
```

### "Invalid variable expansion"

**Error**: `Invalid variable expansion: ${VAR`

**Solution**:
```toml
# Fix syntax - add closing brace
env.MY_VAR = "${VAR}"
```

### "Credential not found"

**Error**: `Credential 'token' not found for skill 'github'`

**Solution**:
```bash
# Set the credential
skill config github --set-credential token
```

### Variables Not Expanding

**Issue**: Seeing literal `${VAR}` in skill

**Cause**: Variables expanded at skill invocation, not in manifest

**Solution**: Ensure environment variables are set before running:
```bash
export VAR=value
skill run myskill
```

## Integration Examples

### Docker Compose

```yaml
# docker-compose.yml
version: '3'
services:
  skill-engine:
    image: skill-engine:latest
    environment:
      - GITHUB_TOKEN=${GITHUB_TOKEN}
      - DATABASE_URL=postgresql://postgres:5432/app
      - LOG_LEVEL=info
    env_file:
      - .env
```

### Kubernetes

```yaml
# deployment.yaml
apiVersion: v1
kind: Secret
metadata:
  name: skill-engine-secrets
type: Opaque
stringData:
  github-token: ${GITHUB_TOKEN}
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: skill-engine
spec:
  template:
    spec:
      containers:
      - name: skill-engine
        env:
        - name: GITHUB_TOKEN
          valueFrom:
            secretKeyRef:
              name: skill-engine-secrets
              key: github-token
```

### CI/CD (GitHub Actions)

```yaml
# .github/workflows/test.yml
name: Test
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run tests
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          DATABASE_URL: ${{ secrets.DATABASE_URL }}
        run: |
          skill run test-skill
```

## Related Documentation

- [Manifest Configuration](./manifest.md) - Complete manifest reference
- [Security Model](./advanced/security.md) - Credential security
- [Skill Development](./developing-skills.md) - Accessing config in skills
- [CLI Reference](../api/cli.md) - Environment variable flags

## External Resources

- [Twelve-Factor App](https://12factor.net/config) - Environment configuration best practices
- [dotenv](https://github.com/motdotla/dotenv) - Environment file loading
- [Vault](https://www.vaultproject.io/) - Enterprise secret management
