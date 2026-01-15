# Your First WASM Skill

In this tutorial, you'll create a simple WASM skill using JavaScript. By the end, you'll have a working skill that can be installed and run with the Skill CLI.

**Time:** 15 minutes

## What You'll Build

A "greeting" skill with two tools:
- `hello` - Returns a greeting message
- `goodbye` - Returns a farewell message

## Prerequisites

- Skill CLI installed (`skill --version`)
- Node.js 18+ installed (`node --version`)
- A text editor

## Step 1: Create the Project Directory

```bash
mkdir greeting-skill
cd greeting-skill
```

## Step 2: Create the Skill File

Create a file called `skill.js`:

```javascript
// skill.js - A simple greeting skill

// Define the tools this skill provides
export function getTools() {
  return [
    {
      name: "hello",
      description: "Say hello to someone",
      parameters: [
        {
          name: "name",
          paramType: "string",
          description: "Name of the person to greet",
          required: true
        }
      ]
    },
    {
      name: "goodbye",
      description: "Say goodbye to someone",
      parameters: [
        {
          name: "name",
          paramType: "string",
          description: "Name of the person",
          required: true
        },
        {
          name: "formal",
          paramType: "boolean",
          description: "Use formal language",
          required: false
        }
      ]
    }
  ];
}

// Return skill metadata
export function getMetadata() {
  return {
    name: "greeting",
    version: "1.0.0",
    description: "A friendly greeting skill",
    author: "Your Name"
  };
}

// Execute a tool
export function executeTool(toolName, argsJson) {
  const args = JSON.parse(argsJson);

  if (toolName === "hello") {
    return {
      success: true,
      output: `Hello, ${args.name}! Welcome to Skill Engine.\n`,
      errorMessage: null
    };
  }

  if (toolName === "goodbye") {
    const message = args.formal
      ? `Farewell, ${args.name}. It was a pleasure.`
      : `Bye, ${args.name}! See you later!`;

    return {
      success: true,
      output: message + "\n",
      errorMessage: null
    };
  }

  return {
    success: false,
    output: "",
    errorMessage: `Unknown tool: ${toolName}`
  };
}

// Validate configuration (optional)
export function validateConfig() {
  return { ok: null };
}
```

## Step 3: Create the SKILL.md Documentation

Create a file called `SKILL.md`:

```markdown
---
name: greeting
description: A friendly greeting skill for demonstrations
---

# Greeting Skill

A simple skill that demonstrates the basics of WASM skill development.

## Tools

### hello

Say hello to someone.

**Parameters:**
- \`name\` (required, string): Name of the person to greet

**Example:**
\`\`\`bash
skill run greeting:hello name="World"
\`\`\`

### goodbye

Say goodbye to someone.

**Parameters:**
- \`name\` (required, string): Name of the person
- \`formal\` (optional, boolean): Use formal language

**Example:**
\`\`\`bash
skill run greeting:goodbye name="Alice" formal=true
\`\`\`
```

## Step 4: Install the Skill

From the `greeting-skill` directory:

```bash
skill install .
```

You should see:
```
✓ Skill 'greeting' installed successfully
```

## Step 5: Test Your Skill

List installed skills:
```bash
skill list
```

Run the hello tool:
```bash
skill run greeting:hello name="World"
```

Expected output:
```
Hello, World! Welcome to Skill Engine.
```

Try the goodbye tool:
```bash
skill run greeting:goodbye name="Alice" formal=true
```

Expected output:
```
Farewell, Alice. It was a pleasure.
```

## Step 6: View Skill Info

```bash
skill info greeting
```

This shows the skill metadata and available tools.

## Understanding the Code

### Tool Definition

Each tool has:
- `name` - Unique identifier for the tool
- `description` - What the tool does (used in search)
- `parameters` - Input parameters with types and requirements

### Parameter Types

Supported types:
- `string` - Text values
- `boolean` - true/false
- `number` - Numeric values
- `integer` - Whole numbers
- `array` - Lists of values
- `object` - Structured data

### Return Values

Tools return an object with:
- `success` - Whether the tool succeeded
- `output` - The output to display
- `errorMessage` - Error details if failed

## Next Steps

- Add more tools to your skill
- Learn about [configuration](/guides/environment)
- Build a [native skill](./first-native-skill)
- Explore [API integration](./api-integration)

## Troubleshooting

**"Skill not found" error:**
- Make sure you ran `skill install .` from the skill directory
- Check that `skill.js` exists in the directory

**"Unknown tool" error:**
- Verify the tool name matches exactly
- Check that `getTools()` returns the tool definition

**Parameters not working:**
- Use the format `name="value"` (with quotes for strings)
- For booleans, use `param=true` or `param=false`
