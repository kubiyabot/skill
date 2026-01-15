# Simple Skill Example

A minimal example demonstrating how easy it is to create skills with Skill Engine.

## Features

- **No build steps required** - Just write JavaScript and run
- **Auto-compilation** - Runtime compiles to WASM on first run and caches
- **Simple API** - Export functions matching the skill interface
- **Type-safe** - Tool definitions provide schema for arguments

## Usage

### Run the skill directly:

```bash
# Hello tool
skill run ./examples/simple-skill hello name=Alice greeting="Hi"

# Echo tool
skill run ./examples/simple-skill echo message="Hello World" repeat=3

# Calculate tool
skill run ./examples/simple-skill calculate operation=add a=10 b=5
skill run ./examples/simple-skill calculate operation=multiply a=7 b=6
```

### How it works:

1. **First run**: Runtime compiles `skill.js` to WASM (takes ~2-3 seconds)
2. **Cached**: Compiled WASM is cached in `~/.skill-engine/local-cache/`
3. **Fast runs**: Subsequent executions use cached WASM (<100ms startup)
4. **Auto-recompile**: If you modify `skill.js`, it automatically recompiles

## Skill Structure

```javascript
// Metadata about your skill
export function getMetadata() {
  return {
    name: "simple-skill",
    version: "1.0.0",
    description: "A simple example skill",
    author: "Your Name"
  };
}

// Define available tools
export function getTools() {
  return [
    {
      name: "hello",
      description: "Greet someone",
      parameters: [
        {
          name: "name",
          paramType: "string",
          description: "Name to greet",
          required: true
        }
      ]
    }
  ];
}

// Execute tools
export async function executeTool(toolName, argsJson) {
  const args = JSON.parse(argsJson);

  if (toolName === "hello") {
    return {
      success: true,
      output: `Hello, ${args.name}!\n`,
      errorMessage: null
    };
  }

  return {
    success: false,
    output: "",
    errorMessage: `Unknown tool: ${toolName}`
  };
}

// Validate config (optional)
export async function validateConfig() {
  return { ok: null };
}
```

## Development Workflow

1. **Create** a new directory with `skill.js`
2. **Write** your tool logic
3. **Run** immediately: `skill run ./my-skill tool-name`
4. **Iterate** - modify and run again

No package.json, no npm install, no build scripts needed!

## Advanced Features

### Using Configuration

Create a `skill.config.toml` in your skill directory:

```toml
[config]
api_key = "your-key"
region = "us-east-1"
```

Access in your skill via environment variables:

```javascript
export async function executeTool(toolName, argsJson) {
  const apiKey = process.env.SKILL_API_KEY;
  const region = process.env.SKILL_REGION;
  // Use configuration...
}
```

### TypeScript Support

You can also write skills in TypeScript:

```bash
# Just change the extension
mv skill.js skill.ts
skill run ./my-skill tool-name
```

The runtime automatically compiles TypeScript to JavaScript, then to WASM.

## Best Practices

1. **Keep it simple** - Skills should be focused and do one thing well
2. **Error handling** - Always return proper error messages
3. **Documentation** - Use descriptive parameter descriptions
4. **Testing** - Test locally before installing: `skill run ./my-skill`
