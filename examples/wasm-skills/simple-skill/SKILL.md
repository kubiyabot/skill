---
name: simple-skill
version: 1.0.0
description: A minimal example demonstrating how to create skills with Skill Engine
author: Skill Engine Team
allowed-tools: ["hello", "echo", "calculate"]
---

# Simple Skill

A minimal example demonstrating how easy it is to create skills with Skill Engine. This skill showcases the basic patterns for defining tools, handling parameters, and returning results - perfect for learning or as a template for new skills.

## Overview

This skill requires **no build steps** - just write JavaScript and run! The runtime automatically compiles to WASM on first run and caches the result for fast subsequent executions.

## When to Use This Skill

**Use this skill to**:
- Learn the basics of Skill Engine development
- Test skill execution and parameter handling
- Understand the skill.js file structure
- Use as a template for creating new skills

**Use this skill as a reference when**:
- Starting a new skill from scratch
- Understanding parameter types and validation
- Learning error handling patterns
- Setting up tool metadata

## Features

- **No build steps required** - Just write JavaScript and run
- **Auto-compilation** - Runtime compiles to WASM on first run (~2-3 seconds) and caches
- **Fast execution** - Subsequent runs use cached WASM (<100ms startup)
- **Auto-recompile** - Modifications to skill.js trigger automatic recompilation
- **Simple API** - Export functions matching the skill interface
- **Type-safe** - Tool definitions provide schema for arguments

## Tools

### hello
Greet someone with a friendly message.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| name | string | Yes | - | Name of the person to greet |
| greeting | string | No | Hello | Custom greeting message |

**Example**:
```bash
# Basic greeting
skill run simple-skill hello name=World
# Output: Hello, World! 👋

# Custom greeting
skill run simple-skill hello name=Alice greeting="Hi"
# Output: Hi, Alice! 👋

# Direct path execution
skill run ./examples/wasm-skills/simple-skill hello name=Bob
```

### echo
Echo back the provided message, optionally repeating it multiple times.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| message | string | Yes | - | Message to echo back |
| repeat | number | No | 1 | Number of times to repeat the message |

**Example**:
```bash
# Simple echo
skill run simple-skill echo message="Hello World"
# Output: Hello World

# Repeat 3 times
skill run simple-skill echo message="Testing" repeat=3
# Output:
# Testing
# Testing
# Testing

# Echo with special characters
skill run simple-skill echo message="Skills are great!" repeat=2
```

### calculate
Perform basic arithmetic operations on two numbers.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| operation | string | Yes | Operation: add, subtract, multiply, divide |
| a | number | Yes | First number |
| b | number | Yes | Second number |

**Example**:
```bash
# Addition
skill run simple-skill calculate operation=add a=10 b=5
# Output: 10 add 5 = 15

# Subtraction
skill run simple-skill calculate operation=subtract a=20 b=8
# Output: 20 subtract 8 = 12

# Multiplication
skill run simple-skill calculate operation=multiply a=7 b=6
# Output: 7 multiply 6 = 42

# Division
skill run simple-skill calculate operation=divide a=100 b=4
# Output: 100 divide 4 = 25

# Decimals work too
skill run simple-skill calculate operation=divide a=22 b=7
# Output: 22 divide 7 = 3.142857142857143
```

## How It Works

### First Run (Compilation)
1. Runtime detects `skill.js` needs compilation
2. Compiles JavaScript → WASM (takes ~2-3 seconds)
3. Stores compiled WASM in `~/.skill-engine/local-cache/`
4. Executes the tool

### Subsequent Runs (Cached)
1. Runtime finds cached WASM
2. Loads from cache (<100ms startup)
3. Executes immediately

### After Modifications
1. Runtime detects `skill.js` changed (via hash)
2. Automatically recompiles to WASM
3. Updates cache
4. Executes with new code

## Skill Structure

This skill demonstrates the minimal required structure:

```javascript
// 1. Metadata - Basic information about your skill
export function getMetadata() {
  return {
    name: "simple-skill",
    version: "1.0.0",
    description: "A simple example skill",
    author: "Skill Engine Team"
  };
}

// 2. Tool Definitions - Declare available tools and their parameters
export function getTools() {
  return [
    {
      name: "hello",
      description: "Greet someone",
      parameters: [
        {
          name: "name",
          paramType: "string",        // Type: string, number, boolean
          description: "Name to greet",
          required: true,              // Is this parameter required?
          defaultValue: "World"        // Optional default value
        }
      ]
    }
  ];
}

// 3. Tool Execution - Handle tool invocations
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

// 4. Config Validation (Optional) - Validate configuration
export async function validateConfig() {
  return { ok: null };
}
```

## Parameter Types Reference

When defining tools in `getTools()`, use these parameter types:

| paramType | JavaScript Type | Example Values | Validation |
|-----------|----------------|----------------|------------|
| `string` | String | "hello", "world" | Any text |
| `number` | Number | 42, 3.14, -10 | Parsed as float |
| `boolean` | Boolean | true, false | true/false only |

### Parameter Properties

```javascript
{
  name: "param_name",           // Parameter name (use snake_case)
  paramType: "string",          // Type: string, number, boolean
  description: "What it does",  // Clear description for users
  required: true,               // true = required, false = optional
  defaultValue: "default"       // Optional: default if not provided
}
```

## Development Workflow

### 1. Create Your Skill

```bash
# Create directory
mkdir my-skill
cd my-skill

# Create skill.js (copy from examples/wasm-skills/simple-skill/skill.js)
cat > skill.js << 'EOF'
export function getMetadata() {
  return {
    name: "my-skill",
    version: "1.0.0",
    description: "My first skill"
  };
}
// ... add getTools() and executeTool() ...
EOF
```

### 2. Test Locally

```bash
# Run directly without installing
skill run ./my-skill tool-name param1=value1

# Check tool list
skill info ./my-skill

# View parameter help
skill run ./my-skill tool-name --help
```

### 3. Iterate Quickly

```bash
# 1. Edit skill.js
vim skill.js

# 2. Run immediately (auto-recompiles)
skill run ./my-skill tool-name

# No build steps, no npm install, no package.json needed!
```

### 4. Install (Optional)

```bash
# Once ready, install globally
skill install ./my-skill

# Now run from anywhere
skill run my-skill tool-name
```

## Using Configuration (Advanced)

If your skill needs configuration (API keys, URLs, etc.):

### 1. Create skill.config.toml

```toml
[config]
api_key = "your-key-here"
api_url = "https://api.example.com"
timeout = 30
```

### 2. Access via Environment Variables

Configuration is automatically exposed as environment variables with `SKILL_` prefix:

```javascript
export async function executeTool(toolName, argsJson) {
  const apiKey = process.env.SKILL_API_KEY;
  const apiUrl = process.env.SKILL_API_URL;
  const timeout = parseInt(process.env.SKILL_TIMEOUT || "30");

  // Use configuration in your tool logic
}
```

### 3. Validate Configuration

```javascript
export async function validateConfig() {
  const apiKey = process.env.SKILL_API_KEY;

  if (!apiKey) {
    return {
      ok: null,
      error: "SKILL_API_KEY is required"
    };
  }

  return { ok: null };
}
```

## TypeScript Support

You can write skills in TypeScript - just change the extension:

```bash
# Rename to .ts
mv skill.js skill.ts

# Runtime automatically compiles TypeScript → JavaScript → WASM
skill run ./my-skill tool-name
```

## Error Handling Best Practices

### Return Proper Error Messages

```javascript
// Bad - unclear error
return { success: false, output: "", errorMessage: "Error" };

// Good - specific error
return {
  success: false,
  output: "",
  errorMessage: "Invalid operation: 'foo'. Use: add, subtract, multiply, or divide"
};
```

### Validate Input Early

```javascript
function handleCalculate(args) {
  // Validate first
  if (isNaN(parseFloat(args.a)) || isNaN(parseFloat(args.b))) {
    return {
      success: false,
      output: "",
      errorMessage: "Both 'a' and 'b' must be valid numbers"
    };
  }

  // Then process
  const result = parseFloat(args.a) + parseFloat(args.b);
  return { success: true, output: `${result}\n`, errorMessage: null };
}
```

### Handle Edge Cases

```javascript
function handleDivide(args) {
  const a = parseFloat(args.a);
  const b = parseFloat(args.b);

  // Check for division by zero
  if (b === 0) {
    return {
      success: false,
      output: "",
      errorMessage: "Cannot divide by zero"
    };
  }

  return { success: true, output: `${a / b}\n`, errorMessage: null };
}
```

## Best Practices

1. **Keep it simple** - Skills should be focused and do one thing well
2. **Document parameters** - Use clear, descriptive parameter descriptions
3. **Validate input** - Check required parameters and types early
4. **Return meaningful errors** - Help users understand what went wrong
5. **Use descriptive names** - Tool and parameter names should be self-explanatory
6. **Provide examples** - Show real usage examples in SKILL.md
7. **Test edge cases** - Handle invalid input gracefully
8. **Follow conventions** - Use snake_case for parameters, lowercase for tool names

## Testing Your Skill

```bash
# List available tools
skill info simple-skill

# Test each tool
skill run simple-skill hello name=Test
skill run simple-skill echo message="Testing 123"
skill run simple-skill calculate operation=add a=2 b=2

# Check parameter validation
skill run simple-skill hello                    # Missing required param
skill run simple-skill calculate operation=add  # Missing required params

# Test edge cases
skill run simple-skill calculate operation=divide a=10 b=0  # Division by zero
skill run simple-skill echo message="Test" repeat=-1         # Negative repeat
```

## Next Steps

After mastering this simple skill:

1. **Study More Examples**:
   - `github-skill` - HTTP API integration
   - `slack-skill` - OAuth authentication
   - `kubernetes-skill` - Native tool execution

2. **Learn the SDK**:
   - Type-safe parameter validation
   - HTTP client utilities
   - Error handling helpers
   - Authentication patterns

3. **Build Real Skills**:
   - Integrate with APIs you use
   - Automate your workflows
   - Share with the community

## Resources

- [Skill Engine Documentation](https://skill-engine.dev/docs)
- [SDK Reference](https://skill-engine.dev/docs/sdk)
- [Example Skills](https://github.com/kubiyabot/skill/tree/main/examples)
- [Community Skills](https://skill-engine.dev/marketplace)

## License

MIT - Use as a template for your own skills!
