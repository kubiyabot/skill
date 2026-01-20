# Getting Started with Skill Engine

Welcome to Skill Engine! This guide will help you create your first skill in minutes.

## Prerequisites

- **Mac, Linux, or Windows (WSL)**
- **Rust 1.75+** (required for installation via Cargo)
- **Node.js 18+** (optional, for creating JavaScript skills)

## 1. Installation

### Install via Cargo (Recommended)

Install the latest version of Skill Engine:

```bash
cargo install skill-cli
```

> **Prerequisites:** Rust is required. Install from [rustup.rs](https://rustup.rs/).

### Install from Source

```bash
git clone https://github.com/kubiyabot/skill
cd skill
cargo install --path crates/skill-cli
```

### Verify Installation

```bash
skill --version
```

## 2. Your First Skill

Let's install a simple example skill to verify everything is working.

```bash
# Install the simple-skill example
skill install https://github.com/kubiyabot/skill/releases/download/latest/simple-skill.wasm

# Run the hello tool
skill run simple-skill hello name=World
```

**Output:**

```
Hello, World!
```

## 3. Creating a Local Skill

Skill Engine allows you to create skills using standard JavaScript/TypeScript without complex build steps.

### Create a directory

```bash
mkdir my-skill
cd my-skill
```

### Create `skill.js`

```javascript
// Metadata describes your skill
export function getMetadata() {
  return {
    name: "my-skill",
    version: "1.0.0",
    description: "My first skill",
  };
}

// Tools are the actions your skill provides
export function getTools() {
  return [
    {
      name: "greet",
      description: "Greets a user",
      parameters: [
        {
          name: "name",
          paramType: "string",
          description: "Name to greet",
          required: true,
        },
      ],
    },
  ];
}

// Execution logic
export async function executeTool(toolName, argsJson) {
  const args = JSON.parse(argsJson);
  if (toolName === "greet") {
    return { success: true, output: `Hello, ${args.name}!` };
  }
}
```

### Run it

```bash
skill run . greet name=Developer
```

The runtime will automatically compile your JavaScript to WASM on the first run.

## 4. Integration with Claude Code

Skill Engine is an MCP (Model Context Protocol) server, which means you can use it directly with Claude.

```bash
skill claude setup
```

This will configure Claude Code to use your installed skills. You can now ask Claude:

> "Use the greet tool in my-skill to say hello to the team"

## Next Steps

- **[Creating Skills](tutorials/creating-your-first-skill.md)**: A step-by-step tutorial.
- **[CLI Reference](api/cli-reference.md)**: Explore all commands.
- **[Examples](../examples/README.md)**: Browse the skill catalog.
