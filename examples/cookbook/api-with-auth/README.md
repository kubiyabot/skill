# API with Authentication Pattern

This pattern demonstrates secure API authentication in WASM skills.

## Key Concepts

### 1. Credential Storage

Never hardcode API keys. Use the config system:

```bash
skill config my-skill
# Enter credentials when prompted
```

Credentials are stored in your system keyring, not in files.

### 2. Credential Validation

Always check for credentials before making requests:

```javascript
if (!config.api_key) {
  return {
    success: false,
    errorMessage: "API key not configured. Run: skill config my-skill"
  };
}
```

### 3. Authentication Headers

Common patterns for adding auth to requests:

```javascript
// Bearer token (most common for OAuth/JWT)
headers: { "Authorization": `Bearer ${config.api_key}` }

// API key header
headers: { "X-API-Key": config.api_key }

// Basic auth
headers: { "Authorization": `Basic ${btoa(username + ':' + password)}` }
```

### 4. Error Handling

Handle auth-specific errors:

```javascript
if (response.status === 401) {
  return { success: false, errorMessage: "Authentication failed" };
}

if (response.status === 403) {
  return { success: false, errorMessage: "Access forbidden" };
}
```

## Usage

```bash
# Install the example
skill install ./examples/cookbook/api-with-auth

# Configure credentials
skill config api-auth-example

# Use the skill
skill run api-auth-example:fetch-data endpoint="/users"
skill run api-auth-example:check-auth
```

## Files

| File | Purpose |
|------|---------|
| `skill.js` | WASM skill implementation |
| `README.md` | This documentation |

## See Also

- [Error Handling Pattern](../error-handling/)
- [Config Validation Pattern](../config-validation/)
