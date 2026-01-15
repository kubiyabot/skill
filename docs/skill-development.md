# Skill Development Guide

## Quick Start

The easiest way to create a skill is to write a simple JavaScript file:

```bash
# Create a new skill
mkdir my-first-skill
cd my-first-skill
```

Create `skill.js`:

```javascript
export function getMetadata() {
  return {
    name: "my-first-skill",
    version: "1.0.0",
    description: "My first skill",
    author: "Your Name"
  };
}

export function getTools() {
  return [
    {
      name: "greet",
      description: "Greet someone",
      parameters: [
        { name: "name", paramType: "string", description: "Name to greet", required: true }
      ]
    }
  ];
}

export async function executeTool(toolName, argsJson) {
  const args = JSON.parse(argsJson);

  if (toolName === "greet") {
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

export async function validateConfig() {
  return { ok: null };
}
```

Run it immediately:

```bash
skill run . greet name=World
```

That's it! No build steps, no package.json, no configuration needed.

## How It Works

### 1. First Run (Compilation)

When you run a skill for the first time:

```
skill run ./my-skill tool-name
  ↓
  Runtime detects skill.js
  ↓
  Compiles to WASM using jco componentize (~2-3 seconds)
  ↓
  Caches compiled WASM in ~/.skill-engine/local-cache/
  ↓
  Executes tool
```

### 2. Subsequent Runs (Cached)

On subsequent runs:

```
skill run ./my-skill tool-name
  ↓
  Runtime checks cache
  ↓
  Finds cached WASM (file hasn't changed)
  ↓
  Loads from cache (<100ms)
  ↓
  Executes tool
```

### 3. Auto-Recompilation

If you modify the skill:

```
skill run ./my-skill tool-name
  ↓
  Runtime detects file changed (compares modification time)
  ↓
  Recompiles to WASM
  ↓
  Updates cache
  ↓
  Executes tool
```

## Skill API

Your skill must export four functions:

### `getMetadata()`

Returns information about your skill:

```javascript
export function getMetadata() {
  return {
    name: "skill-name",           // Required: unique identifier
    version: "1.0.0",             // Required: semver version
    description: "Description",    // Required: what the skill does
    author: "Your Name"            // Required: skill author
  };
}
```

### `getTools()`

Returns array of tools your skill provides:

```javascript
export function getTools() {
  return [
    {
      name: "tool-name",                    // Required: tool identifier
      description: "What this tool does",   // Required: tool description
      parameters: [                          // Required: array of parameters
        {
          name: "param-name",               // Required: parameter name
          paramType: "string",              // Required: string, number, boolean
          description: "Param description", // Required: what parameter does
          required: true,                   // Required: is parameter required?
          defaultValue: "default"           // Optional: default value if not provided
        }
      ]
    }
  ];
}
```

### `executeTool(toolName, argsJson)`

Executes a tool with provided arguments:

```javascript
export async function executeTool(toolName, argsJson) {
  // Parse arguments
  const args = JSON.parse(argsJson);

  // Route to tool handler
  if (toolName === "my-tool") {
    return {
      success: true,                      // Required: did tool succeed?
      output: "Tool output\n",           // Required: output text
      errorMessage: null                  // Optional: error message if failed
    };
  }

  // Unknown tool
  return {
    success: false,
    output: "",
    errorMessage: `Unknown tool: ${toolName}`
  };
}
```

### `validateConfig()`

Optional validation of instance configuration:

```javascript
export async function validateConfig() {
  // Check if required config is present
  if (!process.env.SKILL_API_KEY) {
    return {
      err: "API_KEY configuration is required"
    };
  }

  // Configuration is valid
  return { ok: null };
}
```

## Configuration

Skills can use configuration via environment variables.

### Creating Configuration

Create `skill.config.toml` in your skill directory:

```toml
[config]
api_key = "your-api-key"
region = "us-east-1"
timeout = "30"
```

### Accessing Configuration

Configuration is passed as environment variables with `SKILL_` prefix:

```javascript
export async function executeTool(toolName, argsJson) {
  // Access configuration
  const apiKey = process.env.SKILL_API_KEY;
  const region = process.env.SKILL_REGION;
  const timeout = parseInt(process.env.SKILL_TIMEOUT || "30", 10);

  // Use configuration...
}
```

## TypeScript Support

You can write skills in TypeScript:

```typescript
interface SkillMetadata {
  name: string;
  version: string;
  description: string;
  author: string;
}

export function getMetadata(): SkillMetadata {
  return {
    name: "my-skill",
    version: "1.0.0",
    description: "TypeScript skill",
    author: "Your Name"
  };
}

export async function executeTool(
  toolName: string,
  argsJson: string
): Promise<ExecutionResult> {
  const args = JSON.parse(argsJson);
  // Your logic...
}
```

The runtime automatically compiles TypeScript to JavaScript first, then to WASM.

## Directory Structure

Skills can be organized in different ways:

### Simple (Single File)

```
my-skill/
└── skill.js
```

### Standard (With Config)

```
my-skill/
├── skill.js
└── skill.config.toml
```

### Advanced (TypeScript + Modules)

```
my-skill/
├── skill.ts
├── skill.config.toml
├── lib/
│   ├── helper.ts
│   └── types.ts
└── README.md
```

## Best Practices

### 1. Error Handling

Always handle errors gracefully:

```javascript
export async function executeTool(toolName, argsJson) {
  try {
    const args = JSON.parse(argsJson);

    // Validate required parameters
    if (!args.name) {
      return {
        success: false,
        output: "",
        errorMessage: "Parameter 'name' is required"
      };
    }

    // Your logic...

    return {
      success: true,
      output: "Success!\n",
      errorMessage: null
    };
  } catch (error) {
    return {
      success: false,
      output: "",
      errorMessage: `Error: ${error.message}`
    };
  }
}
```

### 2. Clear Output

Make output easy to read:

```javascript
return {
  success: true,
  output: `
✓ Operation completed successfully

Results:
  - Files processed: 42
  - Total time: 1.2s
  - Status: Ready

`,
  errorMessage: null
};
```

### 3. Helpful Descriptions

Write clear parameter descriptions:

```javascript
{
  name: "format",
  paramType: "string",
  description: "Output format (json, yaml, or table). Default: table",
  required: false,
  defaultValue: "table"
}
```

### 4. Async Operations

Skills support async/await:

```javascript
export async function executeTool(toolName, argsJson) {
  const args = JSON.parse(argsJson);

  // Async operations work fine
  const data = await fetchDataFromAPI(args.url);
  const processed = await processData(data);

  return {
    success: true,
    output: JSON.stringify(processed, null, 2) + "\n",
    errorMessage: null
  };
}
```

## Examples

Check out the examples directory for more:

- `examples/simple-skill/` - Basic example with multiple tools
- `examples/aws-skill/` - Complex example with external APIs (coming soon)
- `examples/typescript-skill/` - TypeScript example (coming soon)

## Testing

Test your skill locally before installation:

```bash
# Run directly from directory
skill run ./my-skill tool-name arg1=value1 arg2=value2

# Test with different arguments
skill run ./my-skill tool-name arg1=test
skill run ./my-skill tool-name arg1=production

# Test error handling
skill run ./my-skill invalid-tool-name
```

## Installing Skills

Once you're happy with your skill, install it:

```bash
# Compile to WASM first (optional but faster)
cd my-skill
# ... compile ...

# Install from directory
skill install .

# Or install pre-compiled WASM
skill install ./my-skill.wasm --instance production
```

## Next Steps

- Read the [Simple Skill Example](examples/simple-skill/README.md)
- Explore the [WIT Interface](wit/skill-interface.wit)
- Check out [Advanced Features](#advanced-features) (coming soon)
