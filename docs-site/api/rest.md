# REST API Reference

Complete REST API documentation for Skill Engine HTTP server.

## Base URL

When running `skill serve --http`:
```
http://localhost:3000/api
```

## Interactive Documentation

Skill Engine provides interactive API documentation via Swagger UI:

**When server is running:**
- Swagger UI: `http://localhost:3000/docs/api`
- OpenAPI Spec: `http://localhost:3000/api/openapi.json`

## Authentication

Currently no authentication required for local development. API key authentication coming in future versions.

## Endpoints

### Skills

#### List Skills
```http
GET /api/skills
```

Returns all installed skills.

**Response:**
```json
{
  "skills": [
    {
      "name": "kubernetes",
      "description": "Kubernetes cluster management",
      "runtime": "native",
      "tools": [...]
    }
  ]
}
```

#### Get Skill Details
```http
GET /api/skills/{name}
```

**Response:**
```json
{
  "name": "kubernetes",
  "description": "Kubernetes cluster management",
  "runtime": "native",
  "tools": [
    {
      "name": "get",
      "description": "Get Kubernetes resources",
      "parameters": [...]
    }
  ]
}
```

### Execution

#### Execute Tool
```http
POST /api/execute
```

**Request:**
```json
{
  "skill_name": "kubernetes",
  "tool_name": "get",
  "parameters": {
    "resource": "pods",
    "namespace": "default"
  }
}
```

**Response:**
```json
{
  "execution_id": "exec_123",
  "status": "success",
  "output": "{...}",
  "duration_ms": 245
}
```

#### List Executions
```http
GET /api/executions?limit=10&offset=0
```

**Response:**
```json
{
  "executions": [
    {
      "id": "exec_123",
      "skill_name": "kubernetes",
      "tool_name": "get",
      "status": "success",
      "created_at": "2024-01-15T10:30:00Z",
      "duration_ms": 245
    }
  ],
  "total": 100
}
```

#### Get Execution
```http
GET /api/executions/{id}
```

### Search

#### Semantic Search
```http
POST /api/search
```

**Request:**
```json
{
  "query": "deploy to kubernetes",
  "top_k": 10
}
```

**Response:**
```json
{
  "results": [
    {
      "skill_name": "kubernetes",
      "tool_name": "apply",
      "description": "Apply Kubernetes manifests",
      "score": 0.95
    }
  ]
}
```

### Health

#### Health Check
```http
GET /api/health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "0.3.0",
  "uptime_seconds": 1234
}
```

#### Version Info
```http
GET /api/version
```

**Response:**
```json
{
  "version": "0.3.0",
  "commit": "abc123",
  "build_date": "2024-01-15"
}
```

## Error Responses

All errors follow this format:

```json
{
  "error": {
    "code": "TOOL_EXECUTION_FAILED",
    "message": "Tool execution failed: kubectl not found",
    "details": {
      "skill": "kubernetes",
      "tool": "get"
    }
  }
}
```

**Error Codes:**
- `SKILL_NOT_FOUND`: Skill doesn't exist
- `TOOL_NOT_FOUND`: Tool doesn't exist
- `INVALID_PARAMETERS`: Invalid tool parameters
- `TOOL_EXECUTION_FAILED`: Tool execution failed
- `TIMEOUT`: Execution timeout

## Rate Limiting

Default limits:
- 1000 requests per minute
- 10000 requests per hour

Configurable via server options.

## CORS

CORS enabled by default for local development. Configure for production use.

## WebSocket Support

Coming soon for streaming execution output.

## Examples

### cURL

```bash
# List skills
curl http://localhost:3000/api/skills

# Execute tool
curl -X POST http://localhost:3000/api/execute \
  -H "Content-Type: application/json" \
  -d '{
    "skill_name": "kubernetes",
    "tool_name": "get",
    "parameters": {"resource": "pods"}
  }'
```

### JavaScript/TypeScript

```typescript
const response = await fetch('http://localhost:3000/api/execute', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    skill_name: 'kubernetes',
    tool_name: 'get',
    parameters: { resource: 'pods' }
  })
});

const result = await response.json();
console.log(result);
```

### Python

```python
import requests

response = requests.post('http://localhost:3000/api/execute', json={
    'skill_name': 'kubernetes',
    'tool_name': 'get',
    'parameters': {'resource': 'pods'}
})

print(response.json())
```

## See Also

- [CLI Reference](./cli.md)
- [MCP Protocol](./mcp.md)
- [OpenAPI Specification](http://localhost:3000/docs/api) (when server running)
