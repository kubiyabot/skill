# Claude Code Integration

Integrate Skill Engine with Claude Code for powerful AI-assisted development.

## Overview

Claude Code is Anthropic's official CLI tool that provides AI assistance directly in your terminal. Skill Engine integrates seamlessly with Claude Code through the Model Context Protocol (MCP).

## Quick Setup

### 1. Install Skill Engine

```bash
cargo install skill-cli
```

::: tip Prerequisites
You need Rust installed. Get it from [rustup.rs](https://rustup.rs/).
:::

### 2. Start MCP Server

```bash
skill serve
```

This starts the MCP server on stdio, which Claude Code can connect to.

### 3. Configure Claude Code

Add to your `~/.config/claude/mcp.json`:

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

### 4. Verify Connection

Start Claude Code:

```bash
claude
```

Claude Code will automatically connect to the Skill Engine MCP server and have access to all your installed skills.

## Available Tools

When connected, Claude Code can use these MCP tools:

### execute

Execute any skill tool with parameters.

**Example:**
```
You: "List all Kubernetes pods"
Claude: *uses skill-engine/execute*
       skill_name: kubernetes
       tool_name: get
       parameters: {resource: "pods"}
```

### list_skills

Discover available skills with pagination.

**Example:**
```
You: "What skills are available?"
Claude: *uses skill-engine/list_skills*
```

### search_skills

Semantic search across all tools.

**Example:**
```
You: "Find tools for managing infrastructure"
Claude: *uses skill-engine/search_skills*
       query: "managing infrastructure"
```

## Usage Examples

### Infrastructure Management

```
You: "Show me all pods in the production namespace"

Claude: I'll check the Kubernetes pods in the production namespace.
        [Uses skill-engine/execute: kubernetes/get]
        
        Found 12 pods running in production:
        - api-server-abc123 (Running)
        - worker-def456 (Running)
        ...
```

### GitHub Operations

```
You: "Create an issue in the my-repo repository"

Claude: I'll create an issue for you.
        [Uses skill-engine/execute: github/create-issue]
        
        Created issue #42: "Feature request"
        https://github.com/user/my-repo/issues/42
```

### Database Queries

```
You: "Query the users table for active accounts"

Claude: I'll query the database.
        [Uses skill-engine/execute: postgres/query]
        
        Found 1,234 active users
```

## Configuration

### Environment Variables

Pass environment variables to skills:

```json
{
  "mcpServers": {
    "skill-engine": {
      "command": "skill",
      "args": ["serve"],
      "env": {
        "KUBECONFIG": "/path/to/kubeconfig",
        "DATABASE_URL": "postgresql://localhost/mydb"
      }
    }
  }
}
```

### Custom Skill Path

Specify a custom skill directory:

```json
{
  "mcpServers": {
    "skill-engine": {
      "command": "skill",
      "args": ["serve", "--skills-dir", "/custom/path"]
    }
  }
}
```

## Troubleshooting

### MCP Server Not Connecting

**Problem:** Claude Code can't connect to Skill Engine

**Solution:**
```bash
# Test MCP server manually
skill serve

# Check if skill command is in PATH
which skill

# Verify config file exists
cat ~/.config/claude/mcp.json
```

### Skills Not Found

**Problem:** Claude Code says skills are not available

**Solution:**
```bash
# List installed skills
skill list

# Install example skills
skill install ./examples/native-skills/kubernetes-skill
```

### Permission Errors

**Problem:** Skills fail with permission denied

**Solution:**
```bash
# Check skill configuration
skill config kubernetes

# Verify environment variables
env | grep KUBECONFIG
```

## Best Practices

### 1. Explicit Skill Names

Be specific when asking Claude to use skills:

```
✅ "Use the kubernetes skill to list pods"
❌ "Show me pods" (Claude might not know which skill to use)
```

### 2. Provide Context

Give Claude the information it needs:

```
✅ "List pods in the production namespace using kubectl"
❌ "List pods" (missing namespace context)
```

### 3. Chain Operations

Claude can execute multiple skills in sequence:

```
You: "Deploy the latest version and check if pods are running"

Claude: I'll deploy and verify:
        1. [skill-engine/execute: kubernetes/apply]
        2. [skill-engine/execute: kubernetes/get]
        
        Deployment successful, 3 pods running.
```

## Advanced Usage

### Custom Instructions

Add Skill Engine context to your Claude Code instructions:

```markdown
# .claude/instructions.md

When managing infrastructure:
- Use the kubernetes skill for K8s operations
- Use the terraform skill for IaC changes
- Always verify changes before applying

Available skills: kubernetes, terraform, aws, github
```

### Workflow Automation

Create workflows that Claude can execute:

```
You: "Run the deployment workflow"

Claude: I'll execute the deployment workflow:
        1. Run tests
        2. Build Docker image
        3. Apply Kubernetes manifests
        4. Verify deployment
        5. Update monitoring
```

## Related Documentation

- [MCP Protocol Reference](./mcp.md)
- [Skill Development Guide](./developing-skills.md)
- [API Reference](../api/)
- [Example Skills](../examples/)

## Support

- [GitHub Issues](https://github.com/kubiyabot/skill/issues)
- [GitHub Discussions](https://github.com/kubiyabot/skill/discussions)
