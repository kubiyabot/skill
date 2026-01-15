# MCP Server Mode

Run Skill Engine as a Model Context Protocol (MCP) server for AI agent integration.

## What is MCP?

The Model Context Protocol (MCP) is an open protocol that standardizes how AI applications communicate with external tools and data sources. Skill Engine implements MCP to expose skills as tools that any MCP-compatible agent can use.

## Quick Start

Start the MCP server:

```bash
skill serve
```

The server runs on stdio by default, perfect for Claude Code, Claude Desktop, and other MCP clients.

## Configuration

### Claude Code

Add to `~/.config/claude/mcp.json`:

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

### Claude Desktop

Add to Claude Desktop settings:

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

### VS Code (Continue.dev)

Add to Continue configuration:

```json
{
  "mcpServers": [
    {
      "name": "skill-engine",
      "command": "skill",
      "args": ["serve"]
    }
  ]
}
```

## Exposed Tools

### execute

Execute a skill tool with parameters.

**Parameters:**
- `skill_name` (string, required): Name of the skill
- `tool_name` (string, required): Name of the tool within the skill
- `parameters` (object, required): Tool-specific parameters

**Example:**
```json
{
  "name": "skill-engine/execute",
  "arguments": {
    "skill_name": "kubernetes",
    "tool_name": "get",
    "parameters": {
      "resource": "pods",
      "namespace": "default"
    }
  }
}
```

### list_skills

List all installed skills with pagination.

**Parameters:**
- `limit` (number, optional, default: 50): Maximum skills to return
- `offset` (number, optional, default: 0): Pagination offset

**Example:**
```json
{
  "name": "skill-engine/list_skills",
  "arguments": {
    "limit": 10,
    "offset": 0
  }
}
```

### search_skills

Semantic search for tools using natural language.

**Parameters:**
- `query` (string, required): Natural language search query
- `top_k` (number, optional, default: 10): Number of results to return

**Example:**
```json
{
  "name": "skill-engine/search_skills",
  "arguments": {
    "query": "deploy to kubernetes",
    "top_k": 5
  }
}
```

## Server Options

### Stdio Mode (Default)

```bash
skill serve
```

Communicates via standard input/output. Perfect for MCP clients.

### HTTP Mode (Future)

```bash
skill serve --http --port 3000
```

HTTP server with Server-Sent Events (SSE) for streaming responses.

### Custom Skills Directory

```bash
skill serve --skills-dir /custom/path
```

Load skills from a custom directory.

### Debug Mode

```bash
skill serve --debug
```

Enable verbose logging for troubleshooting.

## Environment Variables

Pass environment variables to skills:

```bash
export KUBECONFIG=/path/to/kubeconfig
export DATABASE_URL=postgresql://localhost/mydb
skill serve
```

Or in MCP config:

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

## Protocol Details

Skill Engine implements MCP v1.0 with these capabilities:

### Transport

- **stdio**: JSON-RPC 2.0 over standard input/output
- **HTTP/SSE** (coming soon): HTTP with Server-Sent Events for streaming

### Message Format

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "skill-engine/execute",
    "arguments": {
      "skill_name": "kubernetes",
      "tool_name": "get",
      "parameters": {"resource": "pods"}
    }
  }
}
```

### Response Format

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"items\": [...]\n}"
      }
    ]
  }
}
```

## Security

### Sandboxing

All skills run in sandboxed environments:
- WASM skills: WASI sandbox with capability-based permissions
- Docker skills: Containerized with resource limits
- Native skills: Allowlisted commands only

### Authentication

MCP server inherits the security context of the calling process:
- Uses local user permissions
- No network authentication required for stdio mode
- HTTP mode (future) will support API key authentication

### Audit Logging

All skill executions are logged:

```bash
# View execution history
skill history

# Export audit log
skill history export --format json
```

## Troubleshooting

### Server Won't Start

```bash
# Check if skill command exists
which skill

# Verify skill is executable
skill --version

# Try running server manually
skill serve
```

### Tools Not Appearing

```bash
# List installed skills
skill list

# Install example skills
skill install ./examples/native-skills/kubernetes-skill

# Restart MCP server
```

### Permission Errors

```bash
# Check skill permissions
skill config kubernetes

# Verify environment variables
env | grep KUBE
```

### Communication Errors

```bash
# Enable debug logging
skill serve --debug

# Check MCP client logs
cat ~/.config/claude/logs/mcp.log
```

## Advanced Usage

### Custom Tool Namespacing

Skills are namespaced as `skill-engine/{tool}`:

```
skill-engine/execute
skill-engine/list_skills
skill-engine/search_skills
```

### Batching Requests

MCP supports batching multiple tool calls:

```json
{
  "jsonrpc": "2.0",
  "batch": [
    {"id": 1, "method": "tools/call", "params": {...}},
    {"id": 2, "method": "tools/call", "params": {...}}
  ]
}
```

### Streaming Responses

For long-running operations, responses can be streamed via SSE (HTTP mode):

```http
GET /mcp/stream?tool=kubernetes/logs&follow=true
```

## Performance

- **Cold start**: ~100ms (includes loading skills)
- **Warm execution**: <10ms overhead
- **Memory**: ~50MB base + skill memory
- **Concurrent requests**: Unlimited (each spawns new execution)

## Related Documentation

- [Claude Code Integration](./claude-code.md)
- [API Reference](../api/)
- [Skill Development](./developing-skills.md)
- [Security Model](./advanced/security.md)
