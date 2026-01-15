# Skill Engine Installation for Claude Code

This guide walks you through installing and configuring the Skill Engine MCP server for Claude Code.

## Prerequisites

- **Claude Code CLI** installed (`npm install -g @anthropic-ai/claude-code`)
- **Skill Engine** binary built or installed

## Quick Start (3 Steps)

### 1. Install Skill Engine

Choose one of these methods:

#### Option A: Install from your local build
```bash
cd /path/to/skill
cargo build --release

# Add to PATH for easy access
export PATH="$PATH:$(pwd)/target/release"
```

#### Option B: Install from npm (coming soon)
```bash
npm install -g skill-engine
```

### 2. Configure MCP Server

The MCP server can be configured at different scopes:

#### Project Scope (Recommended for Teams)
Add to `.mcp.json` in your project root:

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

Or use the CLI:
```bash
cd /path/to/your/project
claude mcp add --transport stdio skill-engine --scope project -- skill serve
```

#### User Scope (Personal, Available Everywhere)
```bash
claude mcp add --transport stdio skill-engine --scope user -- skill serve
```

### 3. Verify Installation

```bash
# List configured MCP servers
claude mcp list

# Check skill-engine details
claude mcp get skill-engine

# Test in Claude Code
claude
> /mcp
```

You should see `skill-engine` listed as "Connected".

## Configuration Options

### Using Absolute Paths

If `skill` is not in your PATH, use the full path:

```json
{
  "mcpServers": {
    "skill-engine": {
      "type": "stdio",
      "command": "/Users/yourname/projects/skill/target/release/skill",
      "args": ["serve"]
    }
  }
}
```

### Environment Variables

Configure skill behavior with environment variables:

```json
{
  "mcpServers": {
    "skill-engine": {
      "type": "stdio",
      "command": "skill",
      "args": ["serve"],
      "env": {
        "SKILL_LOG_LEVEL": "debug",
        "SKILL_MANIFEST": "/custom/path/.skill-engine.toml"
      }
    }
  }
}
```

### Custom Manifest Location

Point to a specific manifest file:

```json
{
  "mcpServers": {
    "skill-engine": {
      "type": "stdio",
      "command": "skill",
      "args": ["serve", "--manifest", "/path/to/.skill-engine.toml"]
    }
  }
}
```

## Testing the Integration

### 1. Verify MCP Server Status

Start Claude Code and check MCP connection:

```bash
claude
```

Within Claude Code:
```
> /mcp

Expected output:
┌──────────────┬───────────┬─────────────────────┐
│ Server       │ Status    │ Tools               │
├──────────────┼───────────┼─────────────────────┤
│ skill-engine │ Connected │ list_skills,        │
│              │           │ search_skills,      │
│              │           │ execute,            │
│              │           │ generate_examples   │
└──────────────┴───────────┴─────────────────────┘
```

### 2. Test Skill Discovery

```
> List all available skills from skill-engine
```

Claude should respond with a list of available skills (kubernetes, terraform, git, etc.).

### 3. Test Skill Execution

```
> Use skill-engine to list kubernetes pods
```

Claude should execute the kubernetes skill's `get` tool to list pods.

## Troubleshooting

### Server Not Found

**Symptom**: `skill-engine` not listed in `claude mcp list`

**Solution**:
1. Check the binary exists:
   ```bash
   which skill
   # or
   ls -la /path/to/skill/target/release/skill
   ```

2. Test the serve command manually:
   ```bash
   skill serve
   # Should show: "✓ MCP server ready - waiting for connections..."
   ```

3. Verify the configuration:
   ```bash
   claude mcp get skill-engine
   ```

### Connection Failed

**Symptom**: `skill-engine` shows as "Disconnected" in `/mcp`

**Solution**:
1. Check permissions:
   ```bash
   chmod +x /path/to/skill/target/release/skill
   ```

2. Test the command in isolation:
   ```bash
   /path/to/skill/target/release/skill serve
   ```

3. Check for errors in Claude Code logs:
   ```bash
   claude --verbose
   ```

### Tools Not Appearing

**Symptom**: MCP server connected but tools not available

**Solution**:
1. Verify manifest is loaded:
   ```bash
   skill serve
   # Look for: "✓ Loaded manifest with X skills"
   ```

2. Check manifest file exists:
   ```bash
   cat .skill-engine.toml
   ```

3. Ensure Claude Code has proper permissions to execute tools

### Windows-Specific Issues

On Windows, you may need to adjust the command:

**For native Windows:**
```json
{
  "mcpServers": {
    "skill-engine": {
      "type": "stdio",
      "command": "C:\\path\\to\\skill.exe",
      "args": ["serve"]
    }
  }
}
```

**For WSL:**
```json
{
  "mcpServers": {
    "skill-engine": {
      "type": "stdio",
      "command": "wsl",
      "args": ["skill", "serve"]
    }
  }
}
```

### Timeout Issues

If the server takes too long to start:

```bash
# Increase MCP timeout (milliseconds)
export MCP_TIMEOUT=10000
claude
```

### Large Output Warnings

If skill tools produce large outputs:

```bash
# Increase output token limit
export MAX_MCP_OUTPUT_TOKENS=50000
claude
```

## Advanced Configuration

### Using Variable Expansion

Support team members with different paths:

```json
{
  "mcpServers": {
    "skill-engine": {
      "type": "stdio",
      "command": "${SKILL_ENGINE_PATH:-skill}",
      "args": ["serve"],
      "env": {
        "SKILL_LOG_LEVEL": "${SKILL_LOG_LEVEL:-info}"
      }
    }
  }
}
```

Team members can override:
```bash
export SKILL_ENGINE_PATH=/custom/path/to/skill
```

### Remote MCP Server (HTTP)

For remote deployments:

```json
{
  "mcpServers": {
    "skill-engine": {
      "type": "http",
      "url": "http://localhost:3000/mcp",
      "headers": {
        "Authorization": "Bearer ${SKILL_API_KEY}"
      }
    }
  }
}
```

Start the HTTP server:
```bash
skill serve --http --port 3000
```

## Uninstalling

Remove the MCP server configuration:

```bash
# Remove project-scoped configuration
claude mcp remove skill-engine

# Or manually delete from .mcp.json
```

## Getting Help

- **Documentation**: See `/docs` folder for more guides
- **MCP Protocol**: https://modelcontextprotocol.io/
- **Claude Code Docs**: https://code.claude.com/docs
- **Skill Engine Issues**: https://github.com/yourusername/skill/issues

## Next Steps

After installation:

1. **Explore Skills**: Run `skill list` to see available skills
2. **Configure Skills**: Edit `.skill-engine.toml` to add/customize skills
3. **Test Execution**: Try executing skills from Claude Code
4. **Create Custom Skills**: See `docs/CREATING_SKILLS.md`

## Example Usage in Claude Code

Once configured, you can use natural language:

```
> List all my kubernetes pods

> Use terraform to show the current state

> Search for skills related to "database"

> Execute the github skill to list my repositories
```

Claude will automatically use the skill-engine MCP tools to fulfill these requests.

## Configuration Scopes Summary

| Scope | Config File | Visibility | Use Case |
|-------|------------|------------|----------|
| **project** | `.mcp.json` (project root) | Team (version controlled) | Shared team setup |
| **user** | `~/.claude.json` | Personal (all projects) | Personal tools |
| **local** | `~/.claude.json` | Personal (project-specific) | Override defaults |

Choose `project` scope for team collaboration, `user` scope for personal use across all projects.
