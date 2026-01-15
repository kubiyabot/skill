# SKILL.md Specification

`SKILL.md` is a markdown-based format for defining native skills that execute shell commands.

## Frontmatter

The file must start with a YAML frontmatter block:

```yaml
---
name: my-skill # Name of the skill (required)
description: My skill desc # Description (required)
version: 1.0.0 # Version (optional)
allowed-tools: # Allowlist of binaries (required for security)
  - ls
  - grep
---
```

## Tool Definitions

Tools are defined using Level 3 Headers (`###`).

### Syntax

```markdown
### tool_name

Description of the tool.

**Parameters:**

- `param1` (type, required/optional): Description
- `param2`: Description

**Example:**
\`\`\`bash
command to execute
\`\`\`
```

## Parameter Types

- `string`: Text input
- `integer`: Whole numbers
- `number`: Decimal numbers
- `boolean`: true/false
- `enum`: `enum:val1|val2`

## Variable Substitution

Variables are substituted into the command block using bash syntax:

- `${param}`: value of param
- `${param:-default}`: value with default

## Security

Only commands listed in `allowed-tools` can be executed. Attempts to run other commands (e.g., via chaining `|` or `;`) will be blocked by the runtime if they are not in the allowlist.
