# HTTP Skill

Universal HTTP client for API requests with flexible authentication support.

## Overview

This skill provides comprehensive HTTP capabilities for making API requests. It supports all standard HTTP methods, multiple authentication schemes, and specialized request types like GraphQL.

## Requirements

- No external dependencies required
- Authentication is configured per-request

## Tools

### Basic HTTP Methods

#### get
Send HTTP GET request.

**Parameters:**
- `url` (string, required): Request URL
- `headers` (string, optional): Custom headers as JSON object
- `auth_type` (string, optional): Authentication type: bearer, basic, api-key, none
- `auth_value` (string, optional): Auth value (format depends on auth_type)
- `timeout` (number, optional): Request timeout in milliseconds (default: 30000)

**Example:**
```json
{
  "url": "https://api.example.com/users",
  "auth_type": "bearer",
  "auth_value": "your-api-token"
}
```

#### post
Send HTTP POST request.

**Parameters:**
- All parameters from `get`
- `body` (string, optional): Request body
- `content_type` (string, optional): Content-Type header (default: application/json)

**Example:**
```json
{
  "url": "https://api.example.com/users",
  "body": "{\"name\": \"John\", \"email\": \"john@example.com\"}",
  "auth_type": "bearer",
  "auth_value": "your-api-token"
}
```

#### put
Send HTTP PUT request.

**Parameters:** Same as `post`

**Example:**
```json
{
  "url": "https://api.example.com/users/123",
  "body": "{\"name\": \"John Updated\"}"
}
```

#### patch
Send HTTP PATCH request.

**Parameters:** Same as `post`

**Example:**
```json
{
  "url": "https://api.example.com/users/123",
  "body": "{\"status\": \"active\"}"
}
```

#### delete
Send HTTP DELETE request.

**Parameters:** Same as `get`

**Example:**
```json
{
  "url": "https://api.example.com/users/123",
  "auth_type": "bearer",
  "auth_value": "admin-token"
}
```

#### head
Send HTTP HEAD request (returns headers only, no body).

**Parameters:** Same as `get`

**Example:**
```json
{
  "url": "https://example.com/file.zip"
}
```

#### options
Send HTTP OPTIONS request (returns allowed methods and CORS info).

**Parameters:** Same as `get`

**Example:**
```json
{
  "url": "https://api.example.com/users"
}
```

### Specialized Requests

#### json-get
GET request with JSON response parsing and optional path filtering.

**Parameters:**
- All parameters from `get`
- `filter` (string, optional): JSON path filter (e.g., ".data.users", "[0].name")

**Examples:**
```json
// Get all users
{
  "url": "https://api.example.com/users"
}

// Get just the data field
{
  "url": "https://api.example.com/users",
  "filter": ".data"
}

// Get first user's name
{
  "url": "https://api.example.com/users",
  "filter": ".data[0].name"
}
```

#### json-post
POST request with JSON body (validates JSON before sending).

**Parameters:**
- All parameters from `get`
- `data` (string, required): JSON data to send

**Example:**
```json
{
  "url": "https://api.example.com/users",
  "data": "{\"name\": \"John\", \"role\": \"admin\"}"
}
```

#### form-post
POST request with form-urlencoded data.

**Parameters:**
- All parameters from `get`
- `data` (string, required): Form data as JSON object

**Example:**
```json
{
  "url": "https://api.example.com/login",
  "data": "{\"username\": \"john\", \"password\": \"secret\"}"
}
```

#### graphql
Send GraphQL query or mutation.

**Parameters:**
- All parameters from `get`
- `query` (string, required): GraphQL query or mutation
- `variables` (string, optional): GraphQL variables as JSON
- `operation_name` (string, optional): Operation name for multi-operation documents

**Examples:**
```json
// Simple query
{
  "url": "https://api.example.com/graphql",
  "query": "{ users { id name email } }"
}

// Query with variables
{
  "url": "https://api.example.com/graphql",
  "query": "query GetUser($id: ID!) { user(id: $id) { name email } }",
  "variables": "{\"id\": \"123\"}"
}

// Mutation
{
  "url": "https://api.example.com/graphql",
  "query": "mutation CreateUser($input: UserInput!) { createUser(input: $input) { id } }",
  "variables": "{\"input\": {\"name\": \"John\", \"email\": \"john@example.com\"}}"
}
```

#### download
Download file content from a URL.

**Parameters:**
- All parameters from `get`
- `as_text` (boolean, optional): Return content as text (default: true)

**Example:**
```json
{
  "url": "https://example.com/data.json",
  "as_text": true
}
```

#### upload
Upload content to a URL via PUT or POST.

**Parameters:**
- All parameters from `get`
- `content` (string, required): Content to upload
- `method` (string, optional): HTTP method: PUT or POST (default: PUT)
- `content_type` (string, optional): Content-Type header (default: application/octet-stream)

**Example:**
```json
{
  "url": "https://storage.example.com/files/data.json",
  "content": "{\"data\": [1, 2, 3]}",
  "content_type": "application/json",
  "method": "PUT"
}
```

#### test-url
Test if a URL is reachable and return status info.

**Parameters:**
- `url` (string, required): URL to test
- `timeout` (number, optional): Timeout in milliseconds (default: 5000)

**Example:**
```json
{
  "url": "https://api.example.com/health"
}
```

**Output:**
```
URL: https://api.example.com/health
Status: 200 OK
Reachable: Yes
Response Time: 150ms
```

#### websocket-send
Send a WebSocket message.

**Note:** This is limited in WASM context as WebSocket connections require persistent connections that aren't fully supported.

**Parameters:**
- `url` (string, required): WebSocket URL (ws:// or wss://)
- `message` (string, required): Message to send

## Authentication

The skill supports multiple authentication schemes:

### Bearer Token
```json
{
  "auth_type": "bearer",
  "auth_value": "your-api-token"
}
```

### Basic Auth
```json
{
  "auth_type": "basic",
  "auth_value": "username:password"
}
```

### API Key
```json
{
  "auth_type": "api-key",
  "auth_value": "X-API-Key:your-api-key"
}
```

The format for API key is `HeaderName:value`, allowing you to specify any header name for the key.

## Custom Headers

Add custom headers using a JSON object:
```json
{
  "url": "https://api.example.com/data",
  "headers": "{\"X-Custom-Header\": \"value\", \"Accept-Language\": \"en-US\"}"
}
```

## Configuration

No configuration required. All authentication is per-request.

## Error Handling

The skill handles common HTTP errors:
- **4xx errors**: Client errors (authentication, validation, not found)
- **5xx errors**: Server errors
- **Timeout**: Request exceeded timeout limit
- **Network errors**: Connection failures

GraphQL-specific errors are extracted from the response and reported with context.
