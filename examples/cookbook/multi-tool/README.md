# Multi-Tool Pattern

This pattern demonstrates how to organize skills with multiple related tools.

## Key Concepts

### 1. Handler Dispatch

Use an object map for clean routing:

```javascript
const handlers = {
  'list': handleList,
  'get': handleGet,
  'create': handleCreate,
  'update': handleUpdate,
  'delete': handleDelete
};

const handler = handlers[name];
if (!handler) {
  return { success: false, errorMessage: `Unknown tool: ${name}` };
}
return handler(args);
```

### 2. Consistent Response Format

All tools should return the same structure:

```javascript
function formatResponse(data) {
  return {
    success: true,
    output: JSON.stringify(data, null, 2),
    errorMessage: null
  };
}

function formatError(message) {
  return {
    success: false,
    output: "",
    errorMessage: message
  };
}
```

### 3. CRUD Operations

Standard operations for resource management:

| Tool | HTTP Equivalent | Description |
|------|-----------------|-------------|
| `list` | GET /items | List all items |
| `get` | GET /items/:id | Get single item |
| `create` | POST /items | Create new item |
| `update` | PATCH /items/:id | Update existing |
| `delete` | DELETE /items/:id | Delete item |

### 4. Partial Updates

Only update provided fields:

```javascript
if (name !== undefined) {
  item.name = name;
}
if (data !== undefined) {
  item.data = data;
}
```

## Best Practices

1. **Group related tools** - Keep related operations together
2. **Use consistent naming** - `list`, `get`, `create`, `update`, `delete`
3. **Share code** - Extract common logic into helper functions
4. **Document relationships** - Explain how tools work together

## Usage

```bash
# Install the example
skill install ./examples/cookbook/multi-tool

# Create items
skill run multi-tool-example:create name="First Item"
skill run multi-tool-example:create name="Second Item" data='{"key":"value"}'

# List items
skill run multi-tool-example:list
skill run multi-tool-example:list limit=5

# Get specific item
skill run multi-tool-example:get id="item-xxx"

# Update item
skill run multi-tool-example:update id="item-xxx" name="Updated Name"

# Delete item
skill run multi-tool-example:delete id="item-xxx"
```

## Files

| File | Purpose |
|------|---------|
| `skill.js` | WASM skill implementation |
| `README.md` | This documentation |
