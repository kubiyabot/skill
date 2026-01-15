# WASM Skills

WebAssembly-based skills that run in a sandboxed WASM runtime. These skills are compiled from JavaScript, TypeScript, or Python and execute with strong isolation guarantees.

## Examples

| Skill | Language | Description | Complexity |
|-------|----------|-------------|------------|
| [simple-skill](./simple-skill/) | JavaScript | Hello World, basic tools | Beginner |
| [aws-skill](./aws-skill/) | JavaScript | S3, EC2, Lambda operations | Intermediate |
| [github-skill](./github-skill/) | JavaScript | Repos, issues, PRs | Intermediate |
| [github-oauth-skill](./github-oauth-skill/) | TypeScript | OAuth2 device flow | Advanced |
| [slack-skill](./slack-skill/) | TypeScript | Channels, messages, threads | Advanced |
| [python-skill](./python-skill/) | Python | Python SDK example | Intermediate |

## How WASM Skills Work

```
Source Code (JS/TS/Python)
         ↓
    Compilation (jco/skill-sdk)
         ↓
    WASM Component (.wasm)
         ↓
    Skill Engine Runtime
         ↓
    Sandboxed Execution
```

## Creating a WASM Skill

### JavaScript (Simplest)

```javascript
// skill.js
export function getMetadata() {
  return JSON.stringify({
    name: "my-skill",
    version: "1.0.0",
    description: "My custom skill"
  });
}

export function getTools() {
  return JSON.stringify([{
    name: "greet",
    description: "Greet someone",
    parameters: [{
      name: "name",
      type: "string",
      required: true
    }]
  }]);
}

export function executeTool(toolName, argsJson) {
  const args = JSON.parse(argsJson);
  if (toolName === "greet") {
    return JSON.stringify({
      success: true,
      output: `Hello, ${args.name}!`
    });
  }
  return JSON.stringify({ success: false, error: "Unknown tool" });
}
```

### TypeScript (Type-safe)

```typescript
// src/skill.ts
import { Skill, tool, param } from '@anthropic/skill-sdk';

const skill = new Skill({
  name: 'my-skill',
  version: '1.0.0'
});

skill.addTool({
  name: 'greet',
  description: 'Greet someone',
  parameters: [
    param.string('name', 'Name to greet', { required: true })
  ],
  execute: async (args) => {
    return { output: `Hello, ${args.name}!` };
  }
});

export default skill;
```

### Python

```python
# src/main.py
from skill_sdk import Skill, tool

skill = Skill(name="my-skill", version="1.0.0")

@tool(description="Greet someone")
def greet(name: str) -> str:
    return f"Hello, {name}!"
```

## Build Commands

```bash
# JavaScript (auto-compiled on first run, or manually)
npx @bytecodealliance/jco componentize skill.js --wit skill.wit --out skill.wasm

# TypeScript
npm run build        # tsc
npm run build:wasm   # jco componentize

# Python
skill-sdk build
```

## Manifest Configuration

```toml
[skills.my-skill]
source = "./path/to/skill"
description = "My custom skill"

[skills.my-skill.instances.default]
config.api_key = "${MY_API_KEY}"
capabilities.network_access = true
```

## Security Model

WASM skills run in a sandboxed environment with:

- **No filesystem access** by default
- **No network access** by default (must enable in manifest)
- **Limited system calls** (WASI Preview 2)
- **Memory isolation** from host system
- **Capability-based security** for resources

## Best Practices

1. **Use TypeScript** for larger skills (better type safety)
2. **Keep skills focused** - one domain per skill
3. **Handle errors gracefully** - return structured error messages
4. **Use config for secrets** - never hardcode API keys
5. **Document tools well** - descriptions help AI agents use them correctly
