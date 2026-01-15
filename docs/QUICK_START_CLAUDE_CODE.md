# Quick Start: Skill Engine + Claude Code

Get Skill Engine working with Claude Code in **under 3 minutes** with zero installation required. This guide uses a manifest-based approach that's perfect for teams and version control.

## Prerequisites

- [Claude Code CLI](https://github.com/anthropics/claude-code) installed
- Skill Engine binary (see [Installation](#installing-skill-engine) below)

## 3-Step Setup

### Step 1: Create Skill Manifest (30 seconds)

Create a `.skill-engine.toml` file in your project root:

```toml
# .skill-engine.toml
version = "1"

# Essential development skills - copy/paste and customize
[skills.git]
source = "./examples/native-skills/git-skill"
description = "Git version control operations"
runtime = "native"
[skills.git.instances.default]

[skills.docker]
source = "./examples/native-skills/docker-skill"
description = "Docker container management"
runtime = "native"
[skills.docker.instances.default]

[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"
description = "Kubernetes cluster management"
runtime = "native"
[skills.kubernetes.instances.default]

[skills.http]
source = "./examples/wasm-skills/http-skill"
description = "Universal HTTP client"
[skills.http.instances.default]

[skills.github]
source = "./examples/wasm-skills/github-skill"
description = "GitHub API integration"
[skills.github.instances.default]
# Set SKILL_GITHUB_TOKEN environment variable
```

**💡 Tip**: See [examples/configs/](../examples/configs/) for more templates (minimal, team, enterprise).

### Step 2: Configure Claude Code MCP (30 seconds)

Create a `.mcp.json` file in your project root:

```json
{
  "mcpServers": {
    "skill-engine": {
      "type": "stdio",
      "command": "skill",
      "args": ["serve"]
    }
  }
}
```

**Alternative**: Use the Claude CLI to configure:

```bash
cd /your/project
claude mcp add --transport stdio skill-engine --scope project -- skill serve
```

### Step 3: Set Permissions (30 seconds)

Create `.claude/settings.json` (or `.claude/settings.local.json` for personal settings):

```bash
mkdir -p .claude
cat > .claude/settings.json <<'EOF'
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
EOF
```

## Verify Setup

### 1. Check MCP Connection

```bash
claude mcp list
```

Expected output:
```
Checking MCP server health...

skill-engine: skill serve - ✓ Connected
```

### 2. Test Skill Discovery

Start Claude Code:

```bash
claude
```

Then ask:
```
> List all available skills from skill-engine
```

You should see your configured skills (git, docker, kubernetes, http, github).

### 3. Test Skill Execution

```
> Use the git skill to show the current status

> Use kubernetes to list all pods

> Search for skills related to "API"
```

## Complete Example

Here's what your project structure should look like:

```
your-project/
├── .skill-engine.toml       # Skill manifest (version controlled)
├── .mcp.json                # Claude Code MCP config (version controlled)
├── .claude/
│   └── settings.json        # Permissions (optional, for team)
├── .env                     # Secrets (NOT in git)
└── [your project files...]
```

## Installing Skill Engine

If you don't have the `skill` binary yet:

### Option A: Build from Source (Recommended)

```bash
git clone https://github.com/your-org/skill.git
cd skill
cargo build --release

# Add to PATH
export PATH="$PATH:$(pwd)/target/release"

# Or copy to system location
sudo cp target/release/skill /usr/local/bin/
```

### Option B: Download Binary (Coming Soon)

```bash
# macOS
curl -L https://github.com/your-org/skill/releases/latest/download/skill-macos -o skill
chmod +x skill
sudo mv skill /usr/local/bin/

# Linux
curl -L https://github.com/your-org/skill/releases/latest/download/skill-linux -o skill
chmod +x skill
sudo mv skill /usr/local/bin/
```

## Troubleshooting

### Server Not Found

**Symptom**: `skill-engine` not listed in `claude mcp list`

**Solution**:
```bash
# 1. Verify skill binary exists
which skill
# or
skill --version

# 2. Test the serve command manually
skill serve
# Should show: "✓ MCP server ready - waiting for connections..."

# 3. Check .mcp.json configuration
cat .mcp.json
```

### Connection Failed

**Symptom**: `skill-engine` shows as "Disconnected"

**Solution**:
```bash
# 1. Check manifest loads correctly
skill list
# Should show your skills from .skill-engine.toml

# 2. Verify manifest syntax
cat .skill-engine.toml

# 3. Start server in debug mode
skill serve --help
```

### Skills Not Appearing

**Symptom**: MCP connected but no skills listed

**Solution**:
```bash
# 1. Verify manifest is found
skill list
# Should list skills from .skill-engine.toml

# 2. Check manifest paths are correct
# Paths should be relative to manifest location or absolute
# Wrong: source = "examples/native-skills/git-skill"
# Right: source = "./examples/native-skills/git-skill"

# 3. Ensure skills exist at specified paths
ls -la examples/native-skills/git-skill/
```

### Permission Denied

**Symptom**: Claude asks for permission to use skill-engine tools

**Solution**: Add to `.claude/settings.json`:
```json
{
  "permissions": {
    "allow": [
      "mcp__skill-engine__*"
    ]
  }
}
```

## Example Prompts

Once setup is complete, try these with Claude:

### Git Operations
```
> Show the current git status and branch

> List all recent commits with messages

> Show what files have changed since the last commit
```

### Kubernetes Management
```
> List all pods in the default namespace

> Show me the deployment status

> Get logs from the frontend pod
```

### Docker Operations
```
> List all running containers

> Show docker images

> Get container stats
```

### HTTP Requests
```
> Make a GET request to https://api.github.com

> Fetch the weather from wttr.in/London
```

### GitHub Integration
```
> List my repositories

> Show open pull requests

> Get the latest issues
```

## Team Setup

For team collaboration, commit these files to version control:

```bash
# Add to git
git add .skill-engine.toml .mcp.json .claude/settings.json
git commit -m "Add skill-engine configuration for Claude Code"
git push
```

Team members can now:
```bash
git clone <your-repo>
cd <your-repo>
claude  # All skills automatically available!
```

## Environment Variables

For skills that need secrets (like GitHub):

### Local Development

Create a `.env` file (don't commit this):
```bash
# .env
SKILL_GITHUB_TOKEN=ghp_your_token_here
SKILL_JIRA_URL=https://yourorg.atlassian.net
SKILL_SLACK_TOKEN=xoxb-your-token
```

### Team Template

Commit a `.env.example` file:
```bash
# .env.example
SKILL_GITHUB_TOKEN=your_github_token
SKILL_JIRA_URL=https://yourorg.atlassian.net
SKILL_SLACK_TOKEN=your_slack_token
```

Team members copy and fill in their values:
```bash
cp .env.example .env
# Edit .env with your actual secrets
```

## Multiple Environments

For dev/staging/prod configurations:

```toml
# .skill-engine.toml

[skills.kubernetes.instances.dev]
config.cluster = "dev-cluster"
config.kubeconfig = "${KUBECONFIG:-~/.kube/dev-config}"

[skills.kubernetes.instances.staging]
config.cluster = "staging-cluster"
config.kubeconfig = "${KUBECONFIG:-~/.kube/staging-config}"

[skills.kubernetes.instances.prod]
config.cluster = "prod-cluster"
config.kubeconfig = "${KUBECONFIG:-~/.kube/prod-config}"
```

Then use:
```
> Use kubernetes:prod to list pods
```

## What's Happening Behind the Scenes?

1. **Claude Code starts** → Reads `.mcp.json`
2. **Connects to skill-engine** → Runs `skill serve`
3. **skill serve** → Finds `.skill-engine.toml` in current directory
4. **Loads manifest** → Parses all skill definitions
5. **Expands env vars** → Substitutes `${VAR}` from environment
6. **Exposes tools** → Makes all skill tools available via MCP
7. **Claude can now** → Discover and execute skills!

## Benefits of This Approach

✅ **Stateless**: No persistent installation in `~/.skill-engine/`
✅ **Version Controlled**: Entire team uses same configuration
✅ **Reproducible**: `git clone` → `claude` → works
✅ **Environment Specific**: Different configs per environment
✅ **Zero Installation**: No `skill install` commands needed
✅ **Secrets Managed**: Environment variables for sensitive data

## Next Steps

- 📖 [Manifest Guide](./MANIFEST_GUIDE.md) - Complete syntax reference
- 🔧 [Environment Variables Guide](./ENVIRONMENT_VARIABLES.md) - Best practices
- 📦 [Example Manifests](../examples/configs/) - Templates for different use cases
- 🚀 [Creating Custom Skills](./CREATING_SKILLS.md) - Build your own skills

## Need Help?

- **Documentation**: See `/docs` for detailed guides
- **Examples**: Check `examples/configs/` for manifest templates
- **Issues**: Report bugs at [GitHub Issues](https://github.com/your-org/skill/issues)

---

**Time to complete**: 2-3 minutes
**Files created**: 3 (`.skill-engine.toml`, `.mcp.json`, `.claude/settings.json`)
**Commands run**: 1 (`claude`)
**Result**: Full skill integration with Claude Code! 🎉
