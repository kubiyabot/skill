# Getting Started with Skill Engine

Welcome to Skill Engine! This guide will help you create your first skill in minutes.

## Prerequisites

- **Mac, Linux, or Windows (WSL)**
- **Rust 1.75+** (required for installation via Cargo)
- **Node.js 18+** (optional, for creating JavaScript skills)

## 1. Installation

Install the latest version of Skill Engine via Cargo:

```bash
cargo install skill-cli
```

> **Prerequisites:** Rust is required. Install from [rustup.rs](https://rustup.rs/).

Verify the installation:

```bash
skill --version
```

## 2. Create Your First Skill

Skill Engine allows you to create skills using standard JavaScript/TypeScript without any complex build setup.

### Create a directory

```bash
mkdir my-first-skill
cd my-first-skill
```

### Create `skill.js`

Create a file named `skill.js` with the following content:

```javascript
// Metadata describes your skill
export function getMetadata() {
  return {
    name: "hello-skill",
    version: "1.0.0",
    description: "My first skill",
  };
}

// Tools are the actions your skill provides
export function getTools() {
  return [
    {
      name: "greet",
      description: "Greets a user by name",
      parameters: [
        {
          name: "name",
          paramType: "string", // 'string', 'number', 'boolean'
          description: "Name of the person to greet",
          required: true,
        },
      ],
    },
  ];
}

// Execution logic handles the tool calls
export async function executeTool(toolName, argsJson) {
  const args = JSON.parse(argsJson);

  if (toolName === "greet") {
    return {
      success: true,
      output: `Hello, ${args.name}! Welcome to Skill Engine.`,
    };
  }

  return {
    success: false,
    errorMessage: `Unknown tool: ${toolName}`,
  };
}
```

## 3. Run Your Skill

You can run your skill immediately. The runtime will compile it to WebAssembly (WASM) automatically.

```bash
skill run . greet name=Alice
```

_Output:_

```
Hello, Alice! Welcome to Skill Engine.
```

## 4. Using APIs (Optional)

You can use `fetch` to make HTTP requests.

Update your `skill.js`:

```javascript
export async function executeTool(toolName, argsJson) {
  // ... existing code ...

  if (toolName === "joke") {
    const response = await fetch("https://api.chucknorris.io/jokes/random");
    const data = await response.json();
    return {
      success: true,
      output: data.value,
    };
  }
  // ...
}
```

Don't forget to add the definition to `getTools()`!

## 5. Next Steps

- **[Skill Development Guide](skill-development.md)**: Learn about TypeScript, configuration, and advanced features.
- **[Explore Examples](../examples/)**: See real-world skills like AWS, GitHub, and Slack.
- **[Web Interface](web-interface.md)**: Manage skills visually.
