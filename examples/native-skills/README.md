# Native Skills

Native skills wrap command-line tools and execute them directly on the host system. They're ideal for DevOps tools that need full system access.

## Examples

| Skill | CLI Tool | Tools Count | Description |
|-------|----------|-------------|-------------|
| [docker-skill](./docker-skill/) | `docker` | 30+ | Container and image management |
| [kubernetes-skill](./kubernetes-skill/) | `kubectl` | 18+ | Kubernetes cluster management |

## How Native Skills Work

```
WASM Skill Component
        ↓
    Outputs: "Command: kubectl get pods"
        ↓
    Skill Engine Detects "Command:" Prefix
        ↓
    Validates Against Allowlist
        ↓
    Executes Native Command
        ↓
    Returns Output
```

## Security Model

Native skills have access to system commands, so they use a **command allowlist** for security:

```rust
// crates/skill-cli/src/commands/run.rs
let allowed_commands = [
    "kubectl", "helm",      // Kubernetes
    "docker",               // Containers
    "git",                  // Version control
    "curl", "jq",           // Data tools
    "aws", "gcloud", "az",  // Cloud CLIs
    "terraform"             // Infrastructure
];
```

**Only commands in this list can be executed.**

## Creating a Native Skill

Native skills follow the same WASM interface but output commands for native execution:

```javascript
// skill.js
export function executeTool(toolName, argsJson) {
  const args = JSON.parse(argsJson);

  if (toolName === "get") {
    const cmd = `kubectl get ${args.resource}`;
    return JSON.stringify({
      success: true,
      output: `Command: ${cmd}`  // "Command:" prefix triggers native execution
    });
  }
}
```

### SKILL.md Documentation

Native skills use `SKILL.md` to document their tools:

```markdown
---
name: my-cli-skill
description: Wrapper for my-cli tool
allowed-tools: Bash
---

# My CLI Skill

## Tools Provided

### get
Get resources from the system.

**Parameters:**
- `resource` (required): Resource type to get
- `format` (optional): Output format (json, yaml, table)

**Example:**
```bash
skill run my-cli get resource=pods format=json
```
```

## Manifest Configuration

```toml
[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"
description = "Kubernetes cluster management"

[skills.kubernetes.instances.default]
# No special config needed - uses system kubectl

[skills.kubernetes.instances.prod]
config.kubeconfig = "${KUBECONFIG:-~/.kube/config}"
config.context = "production-cluster"
```

## Comparison: Native vs Docker Runtime

| Aspect | Native Skills | Docker Runtime Skills |
|--------|---------------|----------------------|
| Execution | Direct on host | Inside container |
| Network | Full access | Configurable (none default) |
| Filesystem | Full access | Volume mounts only |
| Dependencies | Must be installed | Bundled in image |
| Performance | Fastest | Container overhead |
| Isolation | None | Container sandbox |

**Use Native Skills when:**
- Tool is already installed and configured (kubectl, docker)
- Performance is critical
- Need full system access (SSH, file operations)

**Use Docker Runtime Skills when:**
- Tool isn't installed
- Need isolation/security
- Want reproducible environments

## Best Practices

1. **Document all tools** in SKILL.md
2. **Validate inputs** before building commands
3. **Block dangerous operations** (--privileged, sensitive mounts)
4. **Use parameterized commands** to prevent injection
5. **Return structured output** when possible (JSON)

## Adding New Commands to Allowlist

To add a new command to the allowlist, modify:
- `crates/skill-cli/src/commands/run.rs:380`
- `crates/skill-mcp/src/server.rs:895`

```rust
let allowed_commands = [
    "kubectl", "helm", "git", "curl", "jq",
    "aws", "gcloud", "az", "docker", "terraform",
    "my-new-command"  // Add here
];
```
