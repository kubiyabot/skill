# Tutorial: Creating Your First Skill

This tutorial guides you through creating a new skill for Skill Engine from scratch. We will cover both the native `SKILL.md` format (for CLI tools) and the WASM format (JavaScript/TypeScript).

## Part 1: Native Skills (SKILL.md)

The easiest way to make existing CLI tools available to your agents is using `SKILL.md`.

### 1. Create a `SKILL.md` File

Create a file named `SKILL.md` in a new directory:

````markdown
---
name: file-util
description: Simple file utility wrapper
allowed-tools:
  - ls
  - cat
---

# File Utility Skill

This skill allows agents to list files and read content safely.

## Tools

### list_files

List files in the current directory.

**Parameters:**

- `path` (optional, string): The path to list. Defaults to ".".

**Example:**

```bash
ls -la ${path:-.}
```
````

### read_file

Read contents of a file.

**Parameters:**

- `file` (required, string): The file to read.

**Example:**

```bash
cat ${file}
```

````

### 2. Install and Run

```bash
# Install the skill from the current directory
skill install .

# Test it
skill run file-util list_files
````

## Part 2: WASM Skills (JavaScript)

For custom logic, API integrations, or platform-independent code, use JavaScript.

### 1. Create `skill.js`

```javascript
export function getMetadata() {
  return {
    name: "math_skill",
    version: "0.1.0",
    description: "Basic math operations",
  };
}

export function getTools() {
  return [
    {
      name: "add",
      description: "Add two numbers",
      parameters: [
        { name: "a", paramType: "number", required: true },
        { name: "b", paramType: "number", required: true },
      ],
    },
  ];
}

export async function executeTool(name, argsJson) {
  const args = JSON.parse(argsJson);
  if (name === "add") {
    return { success: true, output: String(args.a + args.b) };
  }
}
```

### 2. Run It

```bash
skill run . add a=10 b=20
```

## Part 3: Testing

It's important to test your skills before sharing.

1. **Manual Testing**: Use `skill run` as shown above.
2. **Automated Testing**: Create a simple shell script to verify outputs.

```bash
#!/bin/bash
OUTPUT=$(skill run . add a=2 b=2)
if [[ "$OUTPUT" == *"4"* ]]; then
  echo "Test Passed!"
else
  echo "Test Failed: $OUTPUT"
fi
```

## Next Steps

- Publish your skill to a GitHub repository.
- Share it with others using `skill install github:username/repo`.
