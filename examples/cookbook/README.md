# Skill Cookbook

Common patterns and recipes for building skills.

## Patterns

| Pattern | Directory | Description |
|---------|-----------|-------------|
| [API with Auth](./api-with-auth/) | `api-with-auth/` | API key and OAuth patterns |
| [Error Handling](./error-handling/) | `error-handling/` | Proper error responses |
| [Multi-Tool](./multi-tool/) | `multi-tool/` | Skills with related tools |
| [Config Validation](./config-validation/) | `config-validation/` | Input validation patterns |

## How to Use

Each pattern directory contains:
- `skill.js` - JavaScript implementation (WASM skills)
- `SKILL.md` - Native skill implementation (where applicable)
- `README.md` - Pattern explanation and usage

Browse the patterns and copy the code snippets you need into your own skills.

## Quick Reference

### API Authentication

```javascript
// Get API key from config
const apiKey = config.api_key;
if (!apiKey) {
  return { success: false, errorMessage: "API key not configured" };
}
```

### Error Handling

```javascript
try {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return { success: true, output: await response.text() };
} catch (error) {
  return { success: false, errorMessage: error.message };
}
```

### Parameter Validation

```javascript
if (!args.required_param) {
  return { success: false, errorMessage: "required_param is required" };
}
```

### Multiple Tools

```javascript
export function executeTool(name, argsJson) {
  const handlers = {
    'list': handleList,
    'get': handleGet,
    'create': handleCreate,
    'delete': handleDelete
  };

  const handler = handlers[name];
  if (!handler) {
    return { success: false, errorMessage: `Unknown tool: ${name}` };
  }

  return handler(JSON.parse(argsJson));
}
```
