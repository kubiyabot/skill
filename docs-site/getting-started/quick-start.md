# Quick Start

Get running with Skill Engine in under 5 minutes.

## Install

One command to install the pre-compiled binary:

```bash
curl -fsSL https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/install.sh | sh
```

This automatically:
- Detects your OS and architecture
- Downloads the correct binary
- Installs to `~/.skill-engine/bin/skill`
- Adds to your PATH

**No Rust compiler needed** - it's a pre-compiled binary.

### Verify Installation

```bash
skill --version
# Output: skill 1.0.0
```

## For Claude Code Users

If you're using Claude Code, set up MCP integration with one command:

```bash
skill claude setup
```

That's it! This command:
- Finds your skill binary automatically
- Creates/updates `.mcp.json` in your project
- Configures the MCP server correctly

**Restart Claude Code** and you'll have access to all your skills through Claude.

### Use Skills in Claude

Now ask Claude to use skills:

```
You: "List the kubernetes pods in the default namespace"

Claude: I'll check the pods using the Kubernetes skill.
[Uses skill-engine MCP tools]

Here are the pods in the default namespace:
- nginx-deployment-xxx (Running)
- redis-cache-yyy (Running)
...
```

### Global Setup (Optional)

To use Skill Engine with Claude across all projects:

```bash
skill claude setup --global
```

This adds Skill Engine to your global Claude Code configuration at `~/.claude/mcp.json`.

## For Other MCP Clients

If you're using a different MCP client (not Claude Code), manually add to your MCP config:

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

## List Available Skills

See what skills are available:

```bash
skill list
```

Output:

```
Installed Skills:
  kubernetes  - Kubernetes cluster management with kubectl
  terraform   - Infrastructure as Code management
  github      - GitHub API integration
  ...
```

## Run Your First Tool

Execute a tool from any skill:

```bash
# List Kubernetes pods
skill run kubernetes get --resource pods

# Check Terraform plan
skill run terraform plan

# List GitHub repositories
skill run github list-repos --owner=yourusername
```

## Check Claude Integration Status

Verify your Claude Code integration is working:

```bash
skill claude status
```

Output:

```
✓ Claude Code integration configured
  Location: /path/to/project/.mcp.json
  Server name: skill-engine
  Binary: /Users/you/.skill-engine/bin/skill
```

## Start Web UI (Optional)

For a visual interface with API explorer:

```bash
skill web
```

Opens at `http://localhost:3000` with:
- Skill browser and search
- Interactive tool tester
- API documentation
- Usage history

## Next Steps

- **[Installation Guide](./installation.md)** - Platform-specific details and troubleshooting
- **[Claude Code Integration Guide](../guides/claude-code.md)** - Advanced Claude setup and tips
- **[MCP Protocol Guide](../guides/mcp.md)** - Configure other MCP clients
- **[Skill Development](../guides/developing-skills.md)** - Build your own skills
- **[Browse Catalog](/catalog)** - Explore available skills

## Common Issues

### "skill: command not found"

Reload your shell:

```bash
source ~/.bashrc  # or ~/.zshrc
```

Or manually add to PATH:

```bash
export PATH="$HOME/.skill-engine/bin:$PATH"
```

### Claude Code not seeing skills

1. Verify integration: `skill claude status`
2. Check `.mcp.json` exists in your project
3. Restart Claude Code completely
4. Check Claude Code MCP logs for errors

### Docker skills failing

Make sure Docker is running:

```bash
docker ps
```

If Docker isn't installed, Docker-based skills won't work. Use native or WASM skills instead.

## Tips

**Skill Discovery**: Use `skill search <keyword>` to find skills semantically

**Tool Help**: Run `skill run <skill-name> <tool-name> --help` for tool-specific options

**Environment Variables**: Set credentials once:
```bash
export GITHUB_TOKEN=your_token_here
```

**Shortcuts**: Create aliases for common commands:
```bash
alias k8s='skill run kubernetes'
alias tf='skill run terraform'
```
