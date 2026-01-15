# Skill Engine + Claude Code - Quick Start Guide

Get up and running with Skill Engine in Claude Code in under 2 minutes using a simple configuration file approach.

## One-Time Setup (< 2 minutes)

### Step 1: Install Skill Engine

```bash
# Option A: From npm (recommended)
npm install -g skill-engine

# Option B: From source
git clone https://github.com/yourusername/skill.git
cd skill
cargo build --release
export PATH="$PATH:$(pwd)/target/release"
```

### Step 2: Copy Manifest to Your Project

Choose a manifest template based on your needs:

**For minimal setup (git, docker, kubernetes, http, github):**
```bash
cd /your/project
curl -o .skill-engine.toml https://raw.githubusercontent.com/yourusername/skill/main/examples/configs/minimal-manifest.toml
```

**For full team setup (all skills):**
```bash
cd /your/project
curl -o .skill-engine.toml https://raw.githubusercontent.com/yourusername/skill/main/examples/configs/team-manifest.toml
```

**Or create your own** `.skill-engine.toml` in your project root:

```toml
# .skill-engine.toml
version = "1"

[skills.git]
source = "./examples/native-skills/git-skill"
description = "Git version control operations"
runtime = "native"

[skills.git.instances.default]
# No config needed

[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"
description = "Kubernetes cluster management"
runtime = "native"

[skills.kubernetes.instances.default]
# No config needed
```

### Step 3: Configure Claude Code MCP

Add to `.mcp.json` in your project root:

```json
{
  "mcpServers": {
    "skill-engine": {
      "type": "stdio",
      "command": "skill",
      "args": ["serve", "--manifest", ".skill-engine.toml"]
    }
  }
}
```

Or use the Claude CLI:

```bash
cd /your/project
claude mcp add --transport stdio skill-engine --scope project -- \
  skill serve --manifest .skill-engine.toml
```

### Step 4: Allow MCP Tools (One Time)

Add to `.claude/settings.local.json` (or commit to `.claude/settings.json` for team):

```json
{
  "permissions": {
    "allow": [
      "mcp__skill-engine__*"
    ]
  },
  "enabledMcpjsonServers": [
    "skill-engine"
  ]
}
```

## That's It! 🎉

Start using Claude Code with all your skills:

```bash
claude
```

```
> List all available skills

> Use kubernetes to show all pods in default namespace

> Execute git status

> Search for skills related to "database"
```

## Why This Approach is Better

### ✅ Stateless & Version Controlled

- **Single source of truth**: `.skill-engine.toml` defines all skills and their configs
- **Check into git**: Whole team uses the same skill configuration
- **No manual installation**: Just clone repo and run `claude`
- **Environment-specific configs**: Use different manifests for dev/staging/prod

### ✅ Easy Customization

**Add a skill:**
```toml
[skills.myskill]
source = "./path/to/myskill"
description = "My custom skill"
runtime = "native"

[skills.myskill.instances.default]
config.api_key = "${MY_API_KEY}"
```

**Multiple instances with different configs:**
```toml
[skills.kubernetes.instances.dev]
config.cluster = "dev-cluster"
config.kubeconfig = "~/.kube/dev-config"

[skills.kubernetes.instances.prod]
config.cluster = "prod-cluster"
config.kubeconfig = "~/.kube/prod-config"
```

**Environment variables with defaults:**
```toml
[skills.postgres.instances.default]
config.host = "${POSTGRES_HOST:-localhost}"
config.port = "${POSTGRES_PORT:-5432}"
config.database = "${POSTGRES_DB:-myapp}"
```

### ✅ Team Collaboration

**Scenario**: Your team needs the same skills across projects

**Solution**: Create a company-wide manifest template

```bash
# Company repo: company-configs/skill-engine.toml
# Team members clone and use
cd new-project
curl -o .skill-engine.toml https://github.com/company/configs/raw/main/skill-engine.toml
claude
```

## Example Manifests

### Minimal (6 skills)

```toml
version = "1"

[skills.git]
source = "./examples/native-skills/git-skill"
runtime = "native"
[skills.git.instances.default]

[skills.docker]
source = "./examples/native-skills/docker-skill"
runtime = "native"
[skills.docker.instances.default]

[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"
runtime = "native"
[skills.kubernetes.instances.default]

[skills.http]
source = "./examples/wasm-skills/http-skill"
[skills.http.instances.default]

[skills.github]
source = "./examples/wasm-skills/github-skill"
[skills.github.instances.default]
# Set SKILL_GITHUB_TOKEN env var

[skills.terraform]
source = "./examples/native-skills/terraform-skill"
runtime = "native"
[skills.terraform.instances.default]
```

### Full Team Setup (12 skills)

See `examples/configs/team-manifest.toml` for complete configuration with:
- Version control (git, github)
- CI/CD (circleci)
- Containers (docker, kubernetes)
- Infrastructure (terraform)
- Databases (postgres)
- Project management (jira, slack)
- Monitoring (prometheus)
- HTTP client

## Configuration Options

### Skill Source Types

```toml
# Local path (relative to manifest file)
[skills.myskill]
source = "./path/to/skill"

# Local path (absolute)
[skills.myskill]
source = "/absolute/path/to/skill"

# Docker image
[skills.python-runner]
source = "docker:python:3.12-slim"
runtime = "docker"
```

### Runtime Types

```toml
# WASM runtime (default, sandboxed)
[skills.http]
source = "./examples/wasm-skills/http-skill"

# Native CLI runtime (wraps existing CLI tools)
[skills.kubectl]
source = "./examples/native-skills/kubernetes-skill"
runtime = "native"

# Docker runtime (runs in container)
[skills.python-runner]
source = "docker:python:3.12-slim"
runtime = "docker"
```

### Environment Variables

```toml
[skills.myskill.instances.default]
# With default fallback
config.api_url = "${API_URL:-https://api.example.com}"

# Required (will error if not set)
config.api_key = "${API_KEY}"

# Multiple env vars
config.database = "${DB_HOST:-localhost}:${DB_PORT:-5432}/${DB_NAME:-app}"
```

### Services (Optional Dependencies)

```toml
[[skills.postgres.services]]
name = "postgres"
description = "PostgreSQL database server"
optional = false  # Skill requires this service
default_port = 5432

[[skills.kubernetes.services]]
name = "kubectl-proxy"
description = "Kubernetes API proxy"
optional = true  # Skill can work without this
default_port = 8001
```

## Advanced Usage

### Per-Environment Manifests

```bash
# .skill-engine.dev.toml
# .skill-engine.staging.toml
# .skill-engine.prod.toml

# In .mcp.json
{
  "mcpServers": {
    "skill-engine": {
      "command": "skill",
      "args": ["serve", "--manifest", ".skill-engine.${ENV:-dev}.toml"]
    }
  }
}
```

Then:
```bash
ENV=prod claude
```

### Multiple Manifest Files

For different use cases:

```bash
# .skill-engine.infra.toml - terraform, kubernetes, docker
# .skill-engine.dev.toml - git, github, jira, slack
# .skill-engine.data.toml - postgres, prometheus

# In .mcp.json, configure multiple servers
{
  "mcpServers": {
    "skill-infra": {
      "command": "skill",
      "args": ["serve", "--manifest", ".skill-engine.infra.toml"]
    },
    "skill-dev": {
      "command": "skill",
      "args": ["serve", "--manifest", ".skill-engine.dev.toml"]
    }
  }
}
```

### Remote Skill Sources (Coming Soon)

```toml
# URL source
[skills.myskill]
source = "https://skills.company.com/myskill.wasm"

# Git source
[skills.myskill]
source = "git://github.com/company/skills#myskill"

# Registry source
[skills.myskill]
source = "registry://skills.company.com/myskill@v1.2.3"
```

## Troubleshooting

### Skills Not Loading

```bash
# Verify manifest is valid
skill list --manifest .skill-engine.toml

# Check for syntax errors
cat .skill-engine.toml
```

### MCP Connection Issues

```bash
# Test the server manually
skill serve --manifest .skill-engine.toml
# Should show: "✓ Loaded manifest with X skills"

# Check Claude Code MCP status
claude mcp list
# Should show: skill-engine: ... - ✓ Connected
```

### Skill Path Issues

Make sure skill paths in the manifest are correct:

```toml
# ❌ Wrong - relative to current directory
source = "examples/native-skills/git-skill"

# ✅ Correct - relative to manifest file location
source = "./examples/native-skills/git-skill"

# ✅ Also correct - absolute path
source = "/absolute/path/to/skill"
```

### Environment Variables Not Working

```bash
# Check variables are set
echo $SKILL_GITHUB_TOKEN

# Or use .env file (if skill supports it)
cat > .env <<EOF
SKILL_GITHUB_TOKEN=ghp_xxxxx
SKILL_JIRA_URL=https://company.atlassian.net
EOF
```

## File Structure

Recommended project structure:

```
project/
├── .skill-engine.toml          # Main manifest (version controlled)
├── .mcp.json                   # Claude Code MCP config (version controlled)
├── .claude/
│   └── settings.json           # Shared team permissions (version controlled)
├── .env                        # Secrets (NOT in git)
└── examples/
    ├── native-skills/          # Your custom CLI-based skills
    └── wasm-skills/            # Your custom WASM skills
```

## Next Steps

1. ✅ Copy a manifest template to `.skill-engine.toml`
2. ✅ Configure `.mcp.json` to point to the manifest
3. ✅ Add tool permissions to `.claude/settings.json`
4. ✅ Start Claude Code and verify connection
5. 📚 [Create custom skills](./CREATING_SKILLS.md)
6. 🔧 [Customize manifest for your team](./MANIFEST_GUIDE.md)
7. 🚀 [Deploy skill-engine as HTTP server](./HTTP_DEPLOYMENT.md)

## Summary

**Traditional approach**: Install each skill individually, manage configs separately
**This approach**: One manifest file → All skills configured → Check into git → Done

```bash
# Traditional (complex)
skill install kubernetes
skill install terraform
skill install github
skill config kubernetes --cluster prod
skill config github --token $TOKEN
# ... repeat for each team member

# New way (simple)
git clone project
cd project
claude  # Everything works! 🎉
```
