# API Reference

Complete API documentation for Skill Engine.

## Interfaces

Skill Engine provides multiple interfaces for different use cases:

### [CLI Reference](./cli.md)

Command-line interface for direct skill execution and management.

```bash
skill find "kubernetes pods"
skill run kubernetes get --resource pods
skill list
```

Perfect for:
- Terminal-based AI agents (Claude Code, Aider)
- Shell scripts and automation
- Interactive development

### [REST API](./rest.md)

HTTP API for web applications and services.

```http
POST /api/execute
GET /api/skills
POST /api/search
```

Features:
- OpenAPI 3.1 specification
- Interactive Swagger UI at `/docs/api`
- JSON request/response
- Execution history tracking

**[View Interactive API Docs →](/docs/api)** (when server is running)

### [MCP Protocol](./mcp.md)

Model Context Protocol for AI agent integration.

```json
{
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

Supported transports:
- stdio (for Claude Desktop, Claude Code)
- HTTP with SSE streaming

## Language SDKs

### [Rust API](./rust.md)

Native Rust interface for embedding Skill Engine.

```rust
use skill_runtime::{Runtime, Manifest};

let runtime = Runtime::new()?;
let manifest = Manifest::load_from_path("skill.wasm")?;
let result = runtime.execute(&manifest, "tool_name", args).await?;
```

Crates:
- `skill-runtime` - Core runtime and WASM execution
- `skill-mcp` - MCP server implementation  
- `skill-http` - HTTP server and REST API
- `skill-cli` - Command-line interface

**[View on docs.rs →](https://docs.rs/skill-runtime)**

## References

- **[Manifest Format](../reference/manifest.md)** - TOML and Markdown skill definitions
- **[Tool Parameters](../reference/parameters.md)** - Parameter types and validation
- **[Error Codes](../reference/errors.md)** - Error handling and status codes
- **[Environment Variables](../reference/environment.md)** - Configuration options

## OpenAPI Specification

The complete REST API specification is available in OpenAPI 3.1 format:

**When skill serve is running:**
- JSON spec: `http://localhost:3000/api/openapi.json`
- Interactive UI: `http://localhost:3000/docs/api`

**Key Endpoints:**

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/skills` | GET | List all installed skills |
| `/api/skills/{name}` | GET | Get skill details |
| `/api/execute` | POST | Execute a tool |
| `/api/search` | POST | Semantic search for tools |
| `/api/executions` | GET | List execution history |
| `/api/health` | GET | Health check |

## MCP Tools Reference

When running as an MCP server, these tools are exposed:

| Tool | Description | Parameters |
|------|-------------|------------|
| `execute` | Run a skill tool | `skill_name`, `tool_name`, `parameters` |
| `list_skills` | List installed skills | `limit`, `offset` |
| `search_skills` | Semantic tool search | `query`, `top_k` |

## SDK Examples

### Rust

```rust
use skill_runtime::{Runtime, LocalSkillLoader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = LocalSkillLoader::new()?;
    let runtime = Runtime::new()?;
    
    // Load skill
    let manifest = loader.load_skill("kubernetes").await?;
    
    // Execute tool
    let result = runtime.execute(
        &manifest,
        "get",
        serde_json::json!({
            "resource": "pods",
            "namespace": "default"
        })
    ).await?;
    
    println!("{}", result.output);
    Ok(())
}
```

### TypeScript (MCP Client)

```typescript
import { MCPClient } from '@modelcontextprotocol/sdk';

const client = new MCPClient({
  transport: {
    type: 'stdio',
    command: 'skill',
    args: ['mcp']
  }
});

await client.connect();

// Call tool
const result = await client.callTool({
  name: 'skill-engine/execute',
  arguments: {
    skill_name: 'kubernetes',
    tool_name: 'get',
    parameters: { resource: 'pods' }
  }
});

console.log(result);
```

### Shell (CLI)

```bash
#!/bin/bash

# Find relevant tool
TOOL=$(skill find "list kubernetes pods" | head -1)

# Execute
skill run kubernetes get --resource pods --all-namespaces

# With jq for parsing
skill run kubernetes get --resource pods | jq '.items[].metadata.name'
```

## Error Handling

All interfaces return errors in a consistent format:

```json
{
  "success": false,
  "error": {
    "code": "TOOL_EXECUTION_FAILED",
    "message": "Tool execution failed: kubectl command not found",
    "details": {
      "skill": "kubernetes",
      "tool": "get",
      "exit_code": 127
    }
  }
}
```

See [Error Codes Reference](../reference/errors.md) for complete list.

## Rate Limits

### CLI Mode
No rate limits (local execution)

### HTTP Server
Configurable via config file:
```toml
[server]
rate_limit_per_minute = 1000
rate_limit_per_hour = 10000
```

### MCP Protocol
Respects agent-side rate limiting

## Authentication

### CLI Mode
Uses local credentials (no auth required)

### HTTP Server
Optional API key authentication:
```bash
export SKILL_ENGINE_API_KEY=sk_xxxxx
skill serve --auth-required
```

### MCP Server
Authenticated via agent's stdio connection

## Versioning

API version is included in all responses:

```json
{
  "api_version": "1.0.0",
  "skill_engine_version": "1.0.0"
}
```

Breaking changes follow semantic versioning.
