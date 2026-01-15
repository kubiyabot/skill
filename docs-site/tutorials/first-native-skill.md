# Your First Native Skill

In this tutorial, you'll create a native skill using the SKILL.md format. Native skills wrap existing CLI tools, making them accessible to AI agents.

**Time:** 10 minutes

## What You'll Build

A "system-info" skill that provides system information tools:
- `hostname` - Get the system hostname
- `uptime` - Get system uptime
- `disk` - Check disk usage

## Prerequisites

- Skill CLI installed (`skill --version`)
- A text editor
- Basic terminal commands available (hostname, uptime, df)

## Step 1: Create the Project Directory

```bash
mkdir system-info-skill
cd system-info-skill
```

## Step 2: Create the SKILL.md File

Create a file called `SKILL.md`:

```markdown
---
name: system-info
description: System information utilities for checking host status
allowed-tools: Bash
---

# System Info Skill

Get information about the system where Skill is running.

## When to Use

- Check system hostname
- Monitor system uptime
- Review disk space usage

## Tools

### hostname

Get the system hostname.

**Example:**
\`\`\`bash
skill run system-info:hostname
\`\`\`

### uptime

Get system uptime and load average.

**Example:**
\`\`\`bash
skill run system-info:uptime
\`\`\`

### disk

Check disk usage.

**Parameters:**
- \`path\` (optional, string): Path to check (default: /)

**Example:**
\`\`\`bash
skill run system-info:disk
skill run system-info:disk path="/home"
\`\`\`

### memory

Show memory usage.

**Example:**
\`\`\`bash
skill run system-info:memory
\`\`\`
```

## Step 3: Install the Skill

```bash
skill install .
```

You should see:
```
✓ Skill 'system-info' installed successfully
```

## Step 4: Test Your Skill

List your skills:
```bash
skill list
```

Run the hostname tool:
```bash
skill run system-info:hostname
```

Check uptime:
```bash
skill run system-info:uptime
```

Check disk usage:
```bash
skill run system-info:disk
```

## Understanding SKILL.md Format

### Frontmatter

The YAML frontmatter defines skill metadata:

```yaml
---
name: system-info           # Skill identifier
description: System info    # Description for search
allowed-tools: Bash         # Commands that can be executed
---
```

### Tool Definitions

Each `### tool-name` heading defines a tool:

```markdown
### disk

Check disk usage.

**Parameters:**
- \`path\` (optional, string): Path to check
```

### Parameter Format

Parameters follow this pattern:
```
- `name` (required|optional, type): Description
```

Types: `string`, `integer`, `number`, `boolean`, `array`

### Allowed Tools

The `allowed-tools` field controls what commands can run:
- `Bash` - Shell commands
- `kubectl` - Kubernetes commands
- `git` - Git commands
- `docker` - Docker commands

Multiple tools: `allowed-tools: Bash, kubectl, git`

## Adding More Tools

Let's add a `processes` tool. Edit your SKILL.md:

```markdown
### processes

List running processes.

**Parameters:**
- \`count\` (optional, integer): Number of processes to show (default: 10)

**Example:**
\`\`\`bash
skill run system-info:processes count=5
\`\`\`
```

Reinstall to pick up changes:
```bash
skill install .
```

## Best Practices

### 1. Clear Descriptions

Write descriptions that help AI agents understand when to use each tool:

```markdown
### disk
Check available disk space. Use when users ask about storage, disk space, or filesystem capacity.
```

### 2. Document Parameters Well

Include type, requirements, defaults, and examples:

```markdown
**Parameters:**
- \`path\` (optional, string, default: "/"): Filesystem path to check
```

### 3. Include Examples

Show realistic usage:

```markdown
**Example:**
\`\`\`bash
# Check root filesystem
skill run system-info:disk

# Check home directory
skill run system-info:disk path="/home"
\`\`\`
```

### 4. Security Considerations

Only include `allowed-tools` that are necessary. Don't use `Bash` if you only need `kubectl`.

## Next Steps

- Create a more complex skill with the [kubernetes example](/examples/kubernetes)
- Learn about [skill instances](/guides/skill-instances) for configuration
- Build a [WASM skill](./first-wasm-skill) for custom logic

## Troubleshooting

**"Command not allowed" error:**
- Check your `allowed-tools` in frontmatter
- Make sure the command exists on your system

**Tool not appearing:**
- Verify the `### tool-name` heading format
- Check for YAML frontmatter syntax errors

**Parameters not parsed:**
- Use the `**Parameters:**` format exactly
- Follow the backtick format for parameter names
