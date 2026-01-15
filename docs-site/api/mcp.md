# MCP Protocol Reference

Technical reference for the Model Context Protocol implementation in Skill Engine.

For user-focused guide, see [MCP Server Mode](../guides/mcp.md).

## Protocol Version

Skill Engine implements **MCP v1.0** specification.

## Transport

### stdio (Default)

JSON-RPC 2.0 over standard input/output.

**Start server:**
```bash
skill serve
```

### HTTP/SSE (Future)

HTTP with Server-Sent Events for streaming.

**Start server:**
```bash
skill serve --http --port 3000
```

## Message Format

All messages follow JSON-RPC 2.0 format:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "method_name",
  "params": {}
}
```

## Methods

### tools/list

List all available tools.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "name": "skill-engine/execute",
        "description": "Execute a skill tool",
        "inputSchema": {
          "type": "object",
          "properties": {
            "skill_name": {"type": "string"},
            "tool_name": {"type": "string"},
            "parameters": {"type": "object"}
          },
          "required": ["skill_name", "tool_name", "parameters"]
        }
      }
    ]
  }
}
```

### tools/call

Execute a tool.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "skill-engine/execute",
    "arguments": {
      "skill_name": "kubernetes",
      "tool_name": "get",
      "parameters": {
        "resource": "pods"
      }
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
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

## Error Handling

Errors follow JSON-RPC 2.0 error format:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32603,
    "message": "Internal error",
    "data": {
      "details": "Tool execution failed"
    }
  }
}
```

**Error Codes:**
- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

## Tool Schema

### skill-engine/execute

Execute any skill tool.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "skill_name": {
      "type": "string",
      "description": "Name of the skill to execute"
    },
    "tool_name": {
      "type": "string",
      "description": "Name of the tool within the skill"
    },
    "parameters": {
      "type": "object",
      "description": "Tool-specific parameters"
    }
  },
  "required": ["skill_name", "tool_name", "parameters"]
}
```

### skill-engine/list_skills

List installed skills with pagination.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "limit": {
      "type": "number",
      "description": "Maximum skills to return",
      "default": 50
    },
    "offset": {
      "type": "number",
      "description": "Pagination offset",
      "default": 0
    }
  }
}
```

### skill-engine/search_skills

Semantic search for tools.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "Natural language search query"
    },
    "top_k": {
      "type": "number",
      "description": "Number of results",
      "default": 10
    }
  },
  "required": ["query"]
}
```

## Implementation Details

### Connection Lifecycle

1. Client spawns `skill serve` process
2. Server initializes and loads skills
3. Client sends `initialize` message
4. Server responds with capabilities
5. Client sends tool requests
6. Server executes and responds
7. Client sends `shutdown` when done

### Concurrent Requests

Server handles requests concurrently. Each tool execution runs in isolation.

### Timeouts

- Default: 30 seconds per tool execution
- Configurable per skill
- Streaming operations may run longer

### Resource Limits

- Memory: Configurable per skill
- CPU: Shares host resources
- Network: Configurable via skill manifest

## Security

### Sandboxing

Skills run in sandboxed environments:
- **WASM**: WASI sandbox with capability-based security
- **Docker**: Containerized with resource limits
- **Native**: Allowlisted commands only

### Permissions

Skills declare required permissions in manifest:
```toml
[capabilities]
network = ["*.example.com"]
filesystem = ["read:/data", "write:/tmp"]
```

### Audit Logging

All tool executions are logged with:
- Timestamp
- Skill and tool names
- Parameters (sanitized)
- Execution result
- Duration

## Testing

### Manual Testing

```bash
# Start server
skill serve

# In another terminal, send JSON-RPC request
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | skill serve
```

### Client Libraries

**Node.js:**
```typescript
import { MCPClient } from '@modelcontextprotocol/sdk';

const client = new MCPClient({
  transport: {
    type: 'stdio',
    command: 'skill',
    args: ['serve']
  }
});

await client.connect();
const result = await client.callTool({
  name: 'skill-engine/execute',
  arguments: {
    skill_name: 'kubernetes',
    tool_name: 'get',
    parameters: { resource: 'pods' }
  }
});
```

## See Also

- [MCP Specification](https://modelcontextprotocol.io/docs)
- [MCP Server Guide](../guides/mcp.md)
- [Claude Code Integration](../guides/claude-code.md)
