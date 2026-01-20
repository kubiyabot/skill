# Claude Bridge

Generate Claude Agent Skills from Skill Engine skills for seamless integration with Claude Code.

## Overview

Claude Bridge automatically converts Skill Engine skills into Claude Agent Skills format, enabling your skills to work natively with Claude Code's filesystem discovery system. This provides dual-mode execution: MCP tools for fast integration, with standalone scripts as a fallback.

## What is Claude Bridge?

Claude Bridge is a code generator that transforms Skill Engine skills (defined in TOML manifests and SKILL.md files) into the Claude Agent Skills format. It creates:

- **SKILL.md**: Instructions for Claude with YAML frontmatter
- **TOOLS.md**: Detailed parameter reference and examples
- **scripts/**: Standalone shell scripts for each tool

This allows Claude Code to automatically discover and use your skills without manual configuration.

## Quick Start

### Generate All Skills

```bash
skill claude-bridge generate
```

This generates Claude Agent Skills for all skills in your manifest to `~/.claude/skills/`.

### Generate Specific Skill

```bash
skill claude-bridge generate --skill kubernetes
```

### Output to Custom Directory

```bash
skill claude-bridge generate --output ~/my-project/.claude/skills
```

### Project-Level Skills

For project-specific skills in `.claude/skills/`:

```bash
skill claude-bridge generate --project
```

## Generated Structure

For each skill, Claude Bridge generates:

```
~/.claude/skills/kubernetes/
├── SKILL.md              # Claude instructions with YAML frontmatter
├── TOOLS.md              # Detailed tool reference
└── scripts/
    ├── get.sh            # Wraps: skill run kubernetes get "$@"
    ├── describe.sh
    ├── apply.sh
    └── delete.sh
```

### SKILL.md Format

```markdown
---
name: kubernetes
description: Kubernetes cluster management with kubectl integration
allowed-tools: Bash, skill-run
---

# Kubernetes Skill

Comprehensive Kubernetes cluster management through kubectl CLI.

## When to Use

- Managing Kubernetes resources (pods, services, deployments)
- Viewing cluster status and logs
- Applying manifests and configurations

## Tools Provided

### get
Retrieve Kubernetes resources.

**Parameters**:
- `resource` (required): Resource type (pods, services, deployments, etc.)
- `name` (optional): Resource name
- `namespace` (optional): Namespace

**Example**:
```bash
skill run kubernetes get resource=pods namespace=default
```
```

### YAML Frontmatter

The YAML frontmatter is required for Claude Code discovery:

```yaml
---
name: skill-name          # Lowercase, max 64 chars
description: Brief description (max 1024 chars)
allowed-tools: Bash, skill-run
---
```

**Required Fields**:
- `name`: Skill identifier (must match directory name)
- `description`: Short description for Claude
- `allowed-tools`: Always include `Bash` and `skill-run`

## Dual Execution Modes

Claude Bridge generates skills that support two execution modes:

### Mode 1: MCP Tools (Preferred)

When Skill Engine's MCP server is running, Claude uses MCP tools directly:

```typescript
// Claude Code internally uses:
mcp.callTool('skill-engine/execute', {
  skill_name: 'kubernetes',
  tool_name: 'get',
  parameters: {
    resource: 'pods',
    namespace: 'default'
  }
})
```

**Advantages**:
- Faster execution (no process spawning)
- Better error handling and validation
- Structured input/output
- Type checking

### Mode 2: Scripts (Fallback)

If MCP server is unavailable, Claude falls back to scripts:

```bash
# Generated script: ~/.claude/skills/kubernetes/scripts/get.sh
#!/bin/bash
set -euo pipefail

# All scripts wrap skill run
exec skill run kubernetes get "$@"
```

**Usage**:
```bash
./get.sh --resource pods --namespace default
```

**Advantages**:
- Works without MCP server running
- Portable (can be copied between machines)
- Easy to debug and inspect
- Shell script familiarity

## Command Reference

### generate

Generate Claude Agent Skills from Skill Engine skills.

```bash
skill claude-bridge generate [OPTIONS]
```

**Options**:

- `--skill <name>`: Generate specific skill only
- `--output <dir>`: Output directory (default: `~/.claude/skills`)
- `--project`: Generate to `.claude/skills` in current directory
- `--force`: Overwrite existing files
- `--dry-run`: Show what would be generated without writing
- `--no-scripts`: Skip script generation (MCP-only mode)
- `--manifest <path>`: Path to manifest file (auto-detected by default)

**Examples**:

```bash
# Generate all skills
skill claude-bridge generate

# Generate specific skill
skill claude-bridge generate --skill terraform

# Dry run to preview
skill claude-bridge generate --dry-run

# Force overwrite existing
skill claude-bridge generate --force

# Project-level skills
cd my-project
skill claude-bridge generate --project

# MCP-only (no scripts)
skill claude-bridge generate --no-scripts

# Custom output directory
skill claude-bridge generate --output ~/custom-skills
```

### validate

Validate skills meet Claude Agent Skills requirements.

```bash
skill claude-bridge validate [OPTIONS]
```

**Options**:

- `--skill <name>`: Validate specific skill
- `--manifest <path>`: Path to manifest file

**Validation Checks**:
- Skill name: lowercase, max 64 characters
- Description: max 1024 characters
- Tool names: valid identifiers
- Parameter types: supported types only
- Required vs optional parameters
- Example syntax correctness

**Example**:

```bash
# Validate all skills
skill claude-bridge validate

# Validate specific skill
skill claude-bridge validate --skill github

# Use custom manifest
skill claude-bridge validate --manifest ./custom.toml
```

## Configuration

### Manifest Requirements

Your `.skill-engine.toml` or `manifest.toml` must include:

```toml
[[skills]]
name = "kubernetes"
description = "Kubernetes cluster management"
source = "./kubernetes-skill"
runtime = "native"

[skills.tools.get]
description = "Get Kubernetes resources"
parameters = [
  { name = "resource", type = "string", required = true },
  { name = "namespace", type = "string", required = false }
]
```

### SKILL.md Enhancement

Add a `SKILL.md` file to your skill directory for richer documentation:

```markdown
# Kubernetes Skill

Comprehensive Kubernetes cluster management through kubectl CLI.

## Prerequisites

- kubectl installed and in PATH
- Valid kubeconfig with cluster access
- Appropriate RBAC permissions

## Common Workflows

### View Pod Status
1. List all pods: `skill run kubernetes get resource=pods`
2. Get specific pod: `skill run kubernetes get resource=pods name=my-pod`
3. View pod logs: `skill run kubernetes logs pod=my-pod`

### Deploy Application
1. Apply manifest: `skill run kubernetes apply file=deployment.yaml`
2. Check rollout: `skill run kubernetes get resource=deployments`
3. View pods: `skill run kubernetes get resource=pods`
```

Claude Bridge will include this content in the generated `SKILL.md`.

## Usage with Claude Code

### Setup

1. **Generate Skills**:
   ```bash
   skill claude-bridge generate
   ```

2. **Configure MCP** (optional, for Mode 1):
   ```json
   // ~/.config/claude/mcp.json
   {
     "mcpServers": {
       "skill-engine": {
         "command": "skill",
         "args": ["serve"]
       }
     }
   }
   ```

3. **Start Claude Code**:
   ```bash
   claude
   ```

### Claude Will Auto-Discover

Claude Code automatically discovers skills in `~/.claude/skills/`:

```
You: "List all Kubernetes pods"

Claude: I'll use the kubernetes skill to list pods.
        [Uses ~/.claude/skills/kubernetes/SKILL.md instructions]
        [Executes: skill run kubernetes get resource=pods]

        Found 12 pods running...
```

### MCP vs Script Selection

Claude automatically chooses the best execution mode:

**Uses MCP** when:
- MCP server is configured and running
- Tool needs structured input/output
- Error handling is critical

**Uses Scripts** when:
- MCP server not available
- Simple command-line style invocation
- Debugging or inspection needed

You don't need to specify which mode - Claude handles it automatically.

## Best Practices

### 1. Keep Descriptions Concise

```toml
# Good
description = "Kubernetes cluster management with kubectl"

# Too verbose (will be truncated)
description = "A comprehensive Kubernetes cluster management tool that provides full access to all kubectl commands including get, describe, apply, delete, and many more for managing pods, services, deployments, configmaps, secrets, and other Kubernetes resources"
```

### 2. Use Clear Tool Names

```toml
# Good
[skills.tools.get]
description = "Get Kubernetes resources"

[skills.tools.describe]
description = "Show detailed resource information"

# Avoid
[skills.tools.k8s_resource_getter]  # Too technical
```

### 3. Provide Examples

Add examples to your manifest:

```toml
[skills.tools.get]
description = "Get Kubernetes resources"
examples = [
  "skill run kubernetes get resource=pods",
  "skill run kubernetes get resource=services namespace=production"
]
```

### 4. Document Prerequisites

Include setup requirements in SKILL.md:

```markdown
## Prerequisites

- kubectl 1.25+ installed
- Valid kubeconfig (~/.kube/config)
- Cluster access with appropriate RBAC
```

### 5. Test Generated Skills

After generation, verify:

```bash
# Check generated structure
ls -la ~/.claude/skills/kubernetes/

# Test script execution
~/.claude/skills/kubernetes/scripts/get.sh --resource pods

# Test with Claude
claude -p "Show me all pods using kubernetes skill"
```

### 6. Version Control

Add generated skills to `.gitignore`:

```bash
# .gitignore
.claude/skills/
~/.claude/skills/
```

Generate fresh on each machine to ensure latest version.

## Troubleshooting

### Skills Not Appearing in Claude

**Problem**: Claude doesn't see generated skills

**Solutions**:
1. Check generation output:
   ```bash
   skill claude-bridge generate --dry-run
   ```

2. Verify location:
   ```bash
   ls -la ~/.claude/skills/
   ```

3. Check YAML frontmatter:
   ```bash
   head -5 ~/.claude/skills/kubernetes/SKILL.md
   ```

4. Restart Claude Code

### Scripts Don't Execute

**Problem**: Permission denied when running scripts

**Solution**:
```bash
chmod +x ~/.claude/skills/*/scripts/*.sh
```

Claude Bridge sets executable permissions automatically, but they may be lost if files are copied.

### MCP Mode Not Working

**Problem**: Claude uses scripts instead of MCP

**Diagnosis**:
```bash
# Check MCP server is running
ps aux | grep "skill serve"

# Test MCP connection
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | skill serve

# Check Claude Code MCP config
cat ~/.config/claude/mcp.json
```

**Solution**:
1. Start MCP server: `skill serve`
2. Verify configuration in `~/.config/claude/mcp.json`
3. Restart Claude Code

### Generation Fails

**Problem**: `skill claude-bridge generate` errors

**Common Causes**:

1. **Invalid manifest**:
   ```bash
   # Validate manifest first
   skill claude-bridge validate
   ```

2. **Missing SKILL.md**:
   - SKILL.md is optional but recommended
   - Claude Bridge generates one from manifest if missing

3. **Permission issues**:
   ```bash
   # Check write permissions
   ls -la ~/.claude/
   mkdir -p ~/.claude/skills
   ```

4. **Invalid skill names**:
   - Must be lowercase
   - Max 64 characters
   - No spaces or special characters

## Advanced Usage

### Custom Templates

Override default templates:

```bash
export CLAUDE_BRIDGE_TEMPLATE_DIR=~/my-templates
skill claude-bridge generate
```

Template structure:

```
my-templates/
├── SKILL.md.hbs         # Handlebars template for SKILL.md
├── TOOLS.md.hbs         # Handlebars template for TOOLS.md
└── script.sh.hbs        # Handlebars template for scripts
```

### Programmatic Usage

Use Claude Bridge in Rust code:

```rust
use skill_cli::commands::claude_bridge::{generate, GenerateOptions};

let options = GenerateOptions {
    output_dir: "~/.claude/skills".into(),
    skill_name: Some("kubernetes".to_string()),
    force: true,
    dry_run: false,
    no_scripts: false,
    project: false,
    manifest_path: None,
};

let result = generate(options).await?;
println!("Generated skills: {:?}", result.generated_skills);
```

### CI/CD Integration

Generate skills in CI/CD:

```yaml
# .github/workflows/claude-skills.yml
name: Generate Claude Skills

on: [push]

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install Skill Engine
        run: cargo install skill-cli

      - name: Generate Claude Skills
        run: skill claude-bridge generate --output ./dist/skills

      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: claude-skills
          path: ./dist/skills
```

### Testing Generated Skills

Create integration tests:

```bash
#!/bin/bash
# test-claude-skills.sh

set -e

# Generate skills
skill claude-bridge generate --output /tmp/test-skills --force

# Test structure
test -f /tmp/test-skills/kubernetes/SKILL.md
test -f /tmp/test-skills/kubernetes/TOOLS.md
test -x /tmp/test-skills/kubernetes/scripts/get.sh

# Test script execution
/tmp/test-skills/kubernetes/scripts/get.sh --resource pods

echo "✓ All tests passed"
```

## Migration Guide

### From Manual Skill Files

If you have existing Claude skills:

1. **Create Skill Engine manifest**:
   ```bash
   skill init
   ```

2. **Convert to Skill Engine format**:
   ```toml
   [[skills]]
   name = "my-skill"
   description = "My custom skill"
   source = "./my-skill"
   runtime = "native"
   ```

3. **Generate new format**:
   ```bash
   skill claude-bridge generate --force
   ```

4. **Compare and test**:
   ```bash
   diff -r ~/.claude/skills/my-skill.old ~/.claude/skills/my-skill
   ```

### From MCP-Only Setup

If you only use MCP:

```bash
# Generate with scripts disabled
skill claude-bridge generate --no-scripts
```

This creates SKILL.md for Claude discovery but skips script generation.

## Performance

### Generation Speed

- **1 skill**: < 5 seconds
- **10 skills**: < 30 seconds
- **50 skills**: < 120 seconds

### Memory Usage

- Peak: < 500 MB
- Typical: 50-100 MB

### Optimization Tips

1. **Generate specific skills only**:
   ```bash
   skill claude-bridge generate --skill kubernetes
   ```

2. **Skip validation** (at your own risk):
   ```bash
   SKIP_VALIDATION=1 skill claude-bridge generate
   ```

3. **Parallel generation** (future):
   ```bash
   skill claude-bridge generate --parallel
   ```

## Security Considerations

### Script Safety

Generated scripts are safe by design:

```bash
#!/bin/bash
set -euo pipefail  # Fail fast, no unset variables

# All scripts just wrap skill run
exec skill run kubernetes get "$@"
```

They:
- Use `exec` to replace process (no shell injection)
- Enable strict error handling
- Don't eval or source user input
- Pass arguments safely via `"$@"`

### Permissions

Scripts inherit Skill Engine's security model:
- Native skills: allowlisted commands only
- WASM skills: WASI sandbox
- Docker skills: containerized with resource limits

### Audit Trail

All executions are logged:

```bash
# View execution history
skill history

# Filter by skill
skill history --skill kubernetes
```

## Related Documentation

- [Claude Code Integration](../claude-code.md) - Complete Claude Code setup
- [MCP Server Mode](../mcp.md) - MCP server configuration
- [Skill Development](../developing-skills.md) - Creating skills
- [CLI Reference](../../api/cli.md) - All CLI commands

## Support

- **Issues**: [GitHub Issues](https://github.com/kubiyabot/skill/issues)
- **Discussions**: [GitHub Discussions](https://github.com/kubiyabot/skill/discussions)
- **Examples**: See `examples/` directory in repository
