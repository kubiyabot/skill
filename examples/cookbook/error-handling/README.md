# Error Handling Pattern

This pattern demonstrates robust error handling in WASM skills.

## Key Concepts

### 1. Categorize Errors

Different errors need different handling:

| Error Type | Example | Retryable? | Action |
|------------|---------|------------|--------|
| Validation | Missing parameter | No | Return immediately |
| Auth (401/403) | Invalid API key | No | Return auth error |
| Not Found (404) | Wrong endpoint | No | Return not found |
| Rate Limit (429) | Too many requests | Yes | Retry with backoff |
| Server (5xx) | Service down | Yes | Retry with backoff |
| Network | Connection failed | Yes | Retry with backoff |

### 2. Return Meaningful Messages

Help users understand what went wrong:

```javascript
// Bad
return { success: false, errorMessage: "Error" };

// Good
return { success: false, errorMessage: "Email format invalid. Expected: user@domain.com" };
```

### 3. Retry with Exponential Backoff

For transient errors:

```javascript
for (let attempt = 1; attempt <= retries; attempt++) {
  try {
    return await fetch(url);
  } catch (error) {
    if (!isRetryable(error)) throw error;
    const delay = Math.pow(2, attempt) * 100; // 200ms, 400ms, 800ms...
    await sleep(delay);
  }
}
```

### 4. Validate All Inputs

Check inputs before processing:

```javascript
const errors = [];
if (!args.email) errors.push("Email is required");
if (!isValidEmail(args.email)) errors.push("Invalid email format");

if (errors.length > 0) {
  return { success: false, errorMessage: errors.join(", ") };
}
```

## Best Practices

1. **Fail fast** - Validate inputs before expensive operations
2. **Be specific** - Tell users exactly what's wrong
3. **Be helpful** - Suggest how to fix the issue
4. **Log for debugging** - Use `console.error()` for internal details
5. **Don't leak secrets** - Never include API keys in error messages

## Usage

```bash
# Install the example
skill install ./examples/cookbook/error-handling

# Test validation
skill run error-handling-example:validate-input email="invalid"
skill run error-handling-example:validate-input email="user@example.com" age=-5

# Test retry logic
skill run error-handling-example:fetch-with-retry url="https://httpstat.us/500" retries=2
```

## Files

| File | Purpose |
|------|---------|
| `skill.js` | WASM skill implementation |
| `README.md` | This documentation |
