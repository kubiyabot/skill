# HTTP Skill

A generic HTTP client skill for Skill Engine. This skill allows agents to make arbitrary HTTP requests (GET, POST, PUT, DELETE, etc.) which is useful for interacting with REST APIs that don't have a dedicated skill yet.

## Installation

```bash
skill install ./examples/wasm-skills/http-skill
```

## Tools

### `request`

Make a raw HTTP request.

**Parameters:**

- `method` (string, required): HTTP method (GET, POST, PUT, DELETE, PATCH, HEAD).
- `url` (string, required): The URL to request.
- `headers` (object, optional): JSON object of request headers.
- `body` (string, optional): Request body (for POST/PUT).

## Examples

### Simple GET Request

```bash
skill run http-skill request method=GET url=https://httpbin.org/get
```

### POST with JSON Body

```bash
skill run http-skill request \
  method=POST \
  url=https://httpbin.org/post \
  headers='{"Content-Type": "application/json"}' \
  body='{"hello": "world"}'
```

## Security Note

This skill is powerful as it can access any URL. When running in production, you might want to restrict deployments or carefully review usage logs.
