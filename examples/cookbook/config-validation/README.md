# Config Validation Pattern

This pattern demonstrates robust configuration validation in WASM skills.

## Key Concepts

### 1. Define a Schema

Document all configuration options:

```javascript
const CONFIG_SCHEMA = {
  api_key: {
    required: true,
    type: 'string',
    minLength: 10,
    description: 'API key for authentication'
  },
  base_url: {
    required: false,
    type: 'string',
    default: 'https://api.example.com',
    description: 'Base URL for API requests'
  }
};
```

### 2. Validate All Fields

Check each field against its schema:

```javascript
for (const [name, schema] of Object.entries(CONFIG_SCHEMA)) {
  const value = config[name];

  if (schema.required && !value) {
    errors.push(`'${name}' is required`);
  }

  if (schema.minLength && value.length < schema.minLength) {
    errors.push(`'${name}' must be at least ${schema.minLength} characters`);
  }
}
```

### 3. Apply Defaults

Use defaults for optional fields:

```javascript
config = {};
for (const [name, schema] of Object.entries(CONFIG_SCHEMA)) {
  config[name] = parsed[name] !== undefined ? parsed[name] : schema.default;
}
```

### 4. Mask Sensitive Values

Never expose full secrets:

```javascript
function maskValue(value) {
  if (value.length <= 8) return '****';
  return value.substring(0, 4) + '****' + value.substring(value.length - 4);
}

// api_key: "sk-1234****5678"
```

## Schema Options

| Option | Type | Description |
|--------|------|-------------|
| `required` | boolean | Is this field required? |
| `type` | string | Expected type: 'string', 'number', 'boolean' |
| `default` | any | Default value for optional fields |
| `minLength` | number | Minimum string length |
| `min`/`max` | number | Number range constraints |
| `pattern` | RegExp | String pattern validation |
| `description` | string | Human-readable description |

## Best Practices

1. **Document all options** - Include descriptions
2. **Validate early** - Check in `validateConfig()`
3. **Report all errors** - Don't stop at first error
4. **Use defaults** - Make optional fields truly optional
5. **Mask secrets** - Never log or display full API keys

## Usage

```bash
# Install the example
skill install ./examples/cookbook/config-validation

# Configure (will prompt for values)
skill config config-validation-example

# View config (masked)
skill run config-validation-example:show-config

# Test connection
skill run config-validation-example:test-connection
```

## Files

| File | Purpose |
|------|---------|
| `skill.js` | WASM skill implementation |
| `README.md` | This documentation |
