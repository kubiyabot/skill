# Implementation Plan: Skill Workflows

## Overview

Extend the Hybrid Bridge Generator with workflow support:
- **A) Procedural workflows in SKILL.md** - Claude follows step-by-step instructions
- **B) Compound scripts** - Multi-tool orchestration scripts

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         WORKFLOW ARCHITECTURE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  INPUT: Workflow Definitions                                                │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                             │
│  Option 1: In SKILL.md (existing)     Option 2: In TOML (new)               │
│  ┌─────────────────────────────┐      ┌─────────────────────────────┐      │
│  │ ## Workflows                │      │ [workflows.troubleshoot]    │      │
│  │                             │      │ description = "..."         │      │
│  │ ### Troubleshoot Pod        │      │ steps = [                   │      │
│  │ 1. Get pod status           │      │   {tool="get", args={...}}, │      │
│  │ 2. Describe pod             │      │   {tool="describe",...},    │      │
│  │ 3. Check logs               │      │ ]                           │      │
│  └─────────────────────────────┘      └─────────────────────────────┘      │
│              │                                    │                         │
│              └────────────────┬───────────────────┘                         │
│                               ▼                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      WORKFLOW GENERATOR                              │   │
│  │                                                                      │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │   │
│  │  │ Parse        │─►│ Validate     │─►│ Generate     │               │   │
│  │  │ Definitions  │  │ Steps        │  │ Outputs      │               │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘               │   │
│  │                                                                      │   │
│  └──────────────────────────────────┬───────────────────────────────────┘   │
│                                     │                                       │
│                                     ▼                                       │
│  OUTPUT                                                                     │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                             │
│  ~/.claude/skills/kubernetes/                                               │
│  ├── SKILL.md                    # Includes workflow documentation          │
│  │   └── ## Workflows            # Procedural steps Claude follows          │
│  ├── TOOLS.md                                                               │
│  ├── scripts/                    # Individual tool scripts                  │
│  │   ├── get.sh                                                             │
│  │   ├── describe.sh                                                        │
│  │   └── logs.sh                                                            │
│  └── workflows/                  # Compound workflow scripts (NEW)          │
│      ├── troubleshoot-pod.sh     # Chains: get → describe → logs            │
│      ├── deploy-app.sh           # Chains: apply → rollout → verify         │
│      └── cleanup-namespace.sh    # Chains: delete resources                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Workflow Definition Format

### Option 1: TOML in Skill Directory

```toml
# kubernetes-skill/workflows.toml

[workflows.troubleshoot-pod]
description = "Diagnose a failing pod by checking status, events, and logs"
parameters = [
    { name = "pod", type = "string", required = true, description = "Pod name" },
    { name = "namespace", type = "string", default = "default", description = "Namespace" },
]
steps = [
    { tool = "get", args = { resource = "pods", name = "${pod}", namespace = "${namespace}" }, label = "Pod Status" },
    { tool = "describe", args = { resource = "pod", name = "${pod}", namespace = "${namespace}" }, label = "Pod Events", extract = "Events:" },
    { tool = "logs", args = { pod = "${pod}", namespace = "${namespace}", tail = 50 }, label = "Recent Logs" },
]

[workflows.restart-deployment]
description = "Restart a deployment with rollout and status verification"
parameters = [
    { name = "deployment", type = "string", required = true },
    { name = "namespace", type = "string", default = "default" },
]
steps = [
    { tool = "rollout", args = { resource = "deployment", name = "${deployment}", namespace = "${namespace}", action = "restart" }, label = "Restart" },
    { tool = "rollout", args = { resource = "deployment", name = "${deployment}", namespace = "${namespace}", action = "status" }, label = "Wait for Rollout", wait = true },
]

[workflows.cleanup-namespace]
description = "Delete all resources in a namespace (destructive)"
parameters = [
    { name = "namespace", type = "string", required = true },
    { name = "confirm", type = "bool", required = true, description = "Must be true to proceed" },
]
confirm = "This will delete ALL resources in namespace '${namespace}'. Continue?"
steps = [
    { tool = "delete", args = { resource = "deployments", namespace = "${namespace}", all = true }, label = "Delete Deployments" },
    { tool = "delete", args = { resource = "services", namespace = "${namespace}", all = true }, label = "Delete Services" },
    { tool = "delete", args = { resource = "configmaps", namespace = "${namespace}", all = true }, label = "Delete ConfigMaps" },
    { tool = "delete", args = { resource = "secrets", namespace = "${namespace}", all = true }, label = "Delete Secrets" },
]
```

### Option 2: Inline in SKILL.md (Simpler)

```markdown
<!-- workflows:
troubleshoot-pod:
  description: Diagnose a failing pod
  params: [pod, namespace?=default]
  steps:
    - get resource=pods name=${pod} namespace=${namespace}
    - describe resource=pod name=${pod} namespace=${namespace}
    - logs pod=${pod} namespace=${namespace} tail=50
-->
```

---

## Output: Generated Files

### SKILL.md - Workflow Section

```markdown
## Workflows

Pre-defined multi-step workflows for common tasks.

### Troubleshoot Pod

Diagnose a failing pod by checking status, events, and logs.

**MCP Sequence:**
```
1. execute(skill='kubernetes', tool='get', args={resource:'pods', name:'<pod>', namespace:'<ns>'})
2. execute(skill='kubernetes', tool='describe', args={resource:'pod', name:'<pod>', namespace:'<ns>'})
3. execute(skill='kubernetes', tool='logs', args={pod:'<pod>', namespace:'<ns>', tail:50})
```

**Script:**
```bash
./workflows/troubleshoot-pod.sh <pod> [namespace]
```

**When to use:**
- User reports "pod not working" or "app is down"
- Need to diagnose CrashLoopBackOff or ImagePullBackOff
- Investigating pod failures

---

### Restart Deployment

Restart a deployment and wait for rollout to complete.

**MCP Sequence:**
```
1. execute(skill='kubernetes', tool='rollout', args={resource:'deployment', name:'<name>', action:'restart'})
2. execute(skill='kubernetes', tool='rollout', args={resource:'deployment', name:'<name>', action:'status'})
```

**Script:**
```bash
./workflows/restart-deployment.sh <deployment> [namespace]
```

---

### Cleanup Namespace (Destructive)

Delete all resources in a namespace.

**Script:**
```bash
./workflows/cleanup-namespace.sh <namespace> --confirm
```

**Warning:** This is destructive and cannot be undone.
```

### Compound Script: workflows/troubleshoot-pod.sh

```bash
#!/bin/bash
# workflows/troubleshoot-pod.sh - Generated by Skill Engine
#
# Workflow: Troubleshoot Pod
# Diagnose a failing pod by checking status, events, and logs
#
# Usage:
#   ./troubleshoot-pod.sh <pod-name> [namespace]
#
# Parameters:
#   pod        - Pod name (required)
#   namespace  - Kubernetes namespace (default: default)
#
# Steps:
#   1. Get pod status
#   2. Describe pod for events
#   3. Show recent logs

set -euo pipefail

# Parse arguments
POD="${1:?Error: Pod name required. Usage: ./troubleshoot-pod.sh <pod> [namespace]}"
NAMESPACE="${2:-default}"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Troubleshooting pod: $POD in namespace: $NAMESPACE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo ""
echo "▶ Step 1: Pod Status"
echo "─────────────────────────────────────────────────────────────"
skill run kubernetes get resource=pods name="$POD" namespace="$NAMESPACE" || true

echo ""
echo "▶ Step 2: Pod Events"
echo "─────────────────────────────────────────────────────────────"
skill run kubernetes describe resource=pod name="$POD" namespace="$NAMESPACE" 2>/dev/null | grep -A 30 "Events:" || echo "No events found"

echo ""
echo "▶ Step 3: Recent Logs (last 50 lines)"
echo "─────────────────────────────────────────────────────────────"
skill run kubernetes logs pod="$POD" namespace="$NAMESPACE" tail=50 || echo "Could not fetch logs"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Troubleshooting complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
```

---

## Implementation Plan

### Phase 1: Workflow Definition Parser (2 days)

```
crates/skill-cli/src/claude_bridge/
├── workflows/
│   ├── mod.rs              # Module exports
│   ├── types.rs            # Workflow, Step, Parameter types
│   ├── parser.rs           # Parse workflows.toml
│   └── validator.rs        # Validate workflow definitions
```

**Tasks:**
- [ ] Define `Workflow`, `WorkflowStep`, `WorkflowParameter` types
- [ ] Parse `workflows.toml` from skill directory
- [ ] Validate step references (tool must exist)
- [ ] Validate parameter interpolation (`${param}`)

**Types:**

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub description: String,
    pub parameters: Vec<WorkflowParameter>,
    pub steps: Vec<WorkflowStep>,
    pub confirm: Option<String>,  // Confirmation prompt for destructive ops
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowParameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub default: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowStep {
    pub tool: String,
    pub args: HashMap<String, String>,
    pub label: Option<String>,
    pub extract: Option<String>,  // Extract specific section from output
    pub wait: Option<bool>,       // Wait for completion
    pub on_error: Option<String>, // continue, stop, skip
}
```

---

### Phase 2: SKILL.md Workflow Documentation (1 day)

**Tasks:**
- [ ] Add `workflows` section to SKILL.md template
- [ ] Document each workflow with MCP + Script usage
- [ ] Add "When to use" guidance
- [ ] Add warnings for destructive workflows

**Template addition:**

```handlebars
{{#if workflows}}
## Workflows

Pre-defined multi-step workflows for common tasks.

{{#each workflows}}
### {{titlecase name}}

{{description}}

**MCP Sequence:**
```
{{#each steps}}
{{@index}}. execute(skill='{{../../skill_name}}', tool='{{tool}}', args={{json args}})
{{/each}}
```

**Script:**
```bash
./workflows/{{name}}.sh {{#each parameters}}{{#if required}}<{{name}}>{{else}}[{{name}}]{{/if}} {{/each}}
```

{{#if confirm}}
**Warning:** {{confirm}}
{{/if}}

---
{{/each}}
{{/if}}
```

---

### Phase 3: Compound Script Generator (2 days)

**File:** `claude_bridge/workflows/script_gen.rs`

**Tasks:**
- [ ] Generate workflow scripts with header/usage/params
- [ ] Implement parameter parsing (positional + named)
- [ ] Implement step execution with labels
- [ ] Handle `extract` (grep/filter output)
- [ ] Handle `on_error` (continue/stop)
- [ ] Add confirmation prompt for destructive workflows

**Generator:**

```rust
pub struct WorkflowScriptGenerator {
    skill_name: String,
}

impl WorkflowScriptGenerator {
    pub fn generate(&self, workflow: &Workflow) -> String {
        let mut script = String::new();

        // Shebang
        script.push_str("#!/bin/bash\n");

        // Header
        script.push_str(&format!("# workflows/{}.sh - Generated by Skill Engine\n", workflow.name));
        script.push_str("#\n");
        script.push_str(&format!("# Workflow: {}\n", titlecase(&workflow.name)));
        script.push_str(&format!("# {}\n", workflow.description));
        script.push_str("#\n");

        // Usage
        self.generate_usage(&mut script, workflow);

        // Body
        script.push_str("\nset -euo pipefail\n\n");

        // Parameter parsing
        self.generate_param_parsing(&mut script, workflow);

        // Confirmation prompt if needed
        if let Some(confirm) = &workflow.confirm {
            self.generate_confirmation(&mut script, confirm);
        }

        // Steps
        self.generate_steps(&mut script, workflow);

        script
    }

    fn generate_steps(&self, script: &mut String, workflow: &Workflow) {
        for (i, step) in workflow.steps.iter().enumerate() {
            let label = step.label.as_deref().unwrap_or(&step.tool);

            script.push_str(&format!("\necho \"\"\n"));
            script.push_str(&format!("echo \"▶ Step {}: {}\"\n", i + 1, label));
            script.push_str("echo \"─────────────────────────────────────────\"\n");

            // Build command
            let args = step.args.iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, self.interpolate(v)))
                .collect::<Vec<_>>()
                .join(" ");

            let cmd = format!("skill run {} {} {}", self.skill_name, step.tool, args);

            // Handle extract
            if let Some(extract) = &step.extract {
                script.push_str(&format!("{} | grep -A 30 \"{}\" || true\n", cmd, extract));
            } else {
                let on_error = step.on_error.as_deref().unwrap_or("continue");
                match on_error {
                    "stop" => script.push_str(&format!("{}\n", cmd)),
                    _ => script.push_str(&format!("{} || true\n", cmd)),
                }
            }
        }
    }
}
```

---

### Phase 4: CLI Integration (1 day)

**Tasks:**
- [ ] Add `--workflows` flag to `skill claude generate`
- [ ] Add workflow loading to generator pipeline
- [ ] Generate `workflows/` directory
- [ ] List workflows in generation summary

**CLI Update:**

```rust
/// Generate Claude Agent Skills (with workflows)
Generate {
    // ... existing options ...

    /// Include workflow scripts
    #[arg(long, default_value = "true")]
    workflows: bool,

    /// Only generate workflows (skip individual tool scripts)
    #[arg(long)]
    workflows_only: bool,
}
```

---

### Phase 5: Built-in Workflows (1 day)

Create default workflows for common skills:

**Kubernetes:**
- `troubleshoot-pod` - Diagnose failing pod
- `restart-deployment` - Restart and verify
- `scale-deployment` - Scale and verify
- `cleanup-namespace` - Delete all resources
- `check-cluster-health` - Node and pod status

**Docker:**
- `cleanup-containers` - Remove stopped containers
- `cleanup-images` - Remove dangling images
- `restart-container` - Stop, remove, run
- `view-logs` - Tail logs with follow

**Git:**
- `sync-branch` - Fetch, pull, status
- `create-feature` - Branch, checkout, push
- `cleanup-branches` - Delete merged branches

---

## Output Structure

```
~/.claude/skills/kubernetes/
├── SKILL.md                      # With ## Workflows section
├── TOOLS.md
├── scripts/                      # Individual tools
│   ├── get.sh
│   ├── describe.sh
│   ├── logs.sh
│   └── ...
└── workflows/                    # Compound workflows (NEW)
    ├── troubleshoot-pod.sh
    ├── restart-deployment.sh
    ├── scale-deployment.sh
    ├── cleanup-namespace.sh
    └── check-cluster-health.sh
```

---

## Timeline

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Phase 1 | 2 days | Workflow parser & types |
| Phase 2 | 1 day | SKILL.md workflow docs |
| Phase 3 | 2 days | Compound script generator |
| Phase 4 | 1 day | CLI integration |
| Phase 5 | 1 day | Built-in workflows |
| **Total** | **7 days** | Full workflow support |

---

## Combined Timeline (Bridge + Workflows)

| Feature | Duration |
|---------|----------|
| Hybrid Bridge (base) | 8-12 days |
| Workflows Extension | 7 days |
| **Total** | **15-19 days** |

**Or parallel development:**
- Bridge core (Phase 1-2): 5-7 days
- Bridge scripts + Workflows: 8-10 days (parallel)
- Integration & testing: 2-3 days
- **Total parallel**: **10-14 days**

---

## Usage Examples

### CLI

```bash
# Generate with workflows (default)
skill claude generate

# Generate without workflows
skill claude generate --no-workflows

# Generate only workflows (assume tools exist)
skill claude generate --workflows-only
```

### Claude Interaction

```
User: "My nginx pod keeps crashing"

Claude: I'll troubleshoot this pod. Let me run the troubleshoot workflow.

[Reads SKILL.md → sees troubleshoot-pod workflow]

Option 1 (MCP - preferred):
execute(skill='kubernetes', tool='get', args={resource:'pods', name:'nginx'})
execute(skill='kubernetes', tool='describe', args={resource:'pod', name:'nginx'})
execute(skill='kubernetes', tool='logs', args={pod:'nginx', tail:50})

Option 2 (Script - fallback):
./workflows/troubleshoot-pod.sh nginx default
```

---

## Success Criteria

1. **Workflow definitions** parsed from `workflows.toml`
2. **SKILL.md** includes workflow documentation
3. **Compound scripts** generated in `workflows/`
4. **Scripts execute correctly** - chain tools together
5. **Claude can follow** either MCP sequence or run script
