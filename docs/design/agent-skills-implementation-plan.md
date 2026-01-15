# Implementing Anthropic's Agent Skills Specification

This document outlines what changes are needed to make Skill Engine compatible with Anthropic's Agent Skills specification.

## Table of Contents

- [Current State Gap Analysis](#current-state-gap-analysis)
- [Implementation Options](#implementation-options)
- [Option A: Full Protocol Compliance](#option-a-full-protocol-compliance)
- [Option B: Hybrid Bridge Approach](#option-b-hybrid-bridge-approach)
- [Recommended Approach](#recommended-approach)
- [Implementation Roadmap](#implementation-roadmap)

---

## Current State Gap Analysis

### What Skill Engine Already Has

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         CURRENT IMPLEMENTATION                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ✅ SKILL.md with YAML frontmatter                                          │
│     ---                                                                     │
│     name: kubernetes                                                        │
│     description: Kubernetes cluster management...                           │
│     ---                                                                     │
│                                                                             │
│  ✅ Tool definitions (via skill.wit / getTools())                           │
│                                                                             │
│  ✅ Execution runtime (WASM, Native, Docker)                                │
│                                                                             │
│  ✅ MCP integration for Claude Code                                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### What's Missing for Agent Skills Compliance

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              GAPS TO FILL                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ❌ Filesystem-based discovery                                              │
│     Required: ~/.claude/skills/ or .claude/skills/                          │
│     Current:  MCP protocol only                                             │
│                                                                             │
│  ❌ Progressive disclosure (3-level loading)                                │
│     Required: Metadata → Instructions → Resources                           │
│     Current:  All tools exposed via MCP at once                             │
│                                                                             │
│  ❌ Instruction-oriented SKILL.md                                           │
│     Required: Workflows Claude reads and follows                            │
│     Current:  Tool documentation (reference material)                       │
│                                                                             │
│  ❌ Bundled executable scripts                                              │
│     Required: scripts/ directory Claude can execute                         │
│     Current:  Tools executed by Skill Engine, not Claude                    │
│                                                                             │
│  ⚠️  Field validation                                                       │
│     Required: name (max 64 chars, lowercase, hyphens)                       │
│     Current:  No validation enforced                                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Detailed Gap Matrix

| Requirement | Claude Agent Skills Spec | Skill Engine Current | Gap |
|-------------|-------------------------|---------------------|-----|
| **Discovery** | Filesystem (`~/.claude/skills/`) | MCP protocol | Major |
| **SKILL.md location** | Root of skill directory | Root of skill directory | None |
| **YAML frontmatter** | `name`, `description` required | Has both fields | None |
| **name validation** | Max 64 chars, lowercase, hyphens, no reserved words | No validation | Minor |
| **description validation** | Max 1024 chars, non-empty | No validation | Minor |
| **Progressive disclosure** | 3-level loading model | Not implemented | Major |
| **Instructions body** | Procedural guidance for Claude | Tool documentation | Conceptual |
| **Bundled scripts** | `scripts/` Claude executes | Engine executes tools | Architectural |
| **Additional resources** | Reference .md files | SKILL.md only typically | Minor |

---

## Implementation Options

There are two main approaches to achieve compatibility:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        IMPLEMENTATION OPTIONS                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────┐    ┌─────────────────────────────┐        │
│  │  OPTION A                   │    │  OPTION B                   │        │
│  │  Full Protocol Compliance   │    │  Hybrid Bridge Approach     │        │
│  ├─────────────────────────────┤    ├─────────────────────────────┤        │
│  │                             │    │                             │        │
│  │  • Filesystem discovery     │    │  • Keep MCP execution       │        │
│  │  • Claude reads SKILL.md    │    │  • Generate Claude Skills   │        │
│  │  • Claude runs scripts      │    │    that wrap MCP tools      │        │
│  │  • Full progressive         │    │  • Best of both worlds      │        │
│  │    disclosure               │    │                             │        │
│  │                             │    │                             │        │
│  │  Effort: HIGH               │    │  Effort: MEDIUM             │        │
│  │  Compatibility: 100%        │    │  Compatibility: ~90%        │        │
│  └─────────────────────────────┘    └─────────────────────────────┘        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Option A: Full Protocol Compliance

Transform Skill Engine skills into standard Claude Agent Skills.

### Architecture Change

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    OPTION A: FULL COMPLIANCE ARCHITECTURE                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Before (Current):                                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  .skill-engine.toml                                                  │   │
│  │         │                                                            │   │
│  │         ▼                                                            │   │
│  │  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐            │   │
│  │  │ Skill Engine│────►│ MCP Server  │────►│ Claude Code │            │   │
│  │  │   Runtime   │     │             │     │             │            │   │
│  │  └─────────────┘     └─────────────┘     └─────────────┘            │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  After (Option A):                                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  ~/.claude/skills/kubernetes/                                        │   │
│  │         │                                                            │   │
│  │         ├── SKILL.md          ◄── Claude reads this                 │   │
│  │         ├── REFERENCE.md      ◄── Additional docs                   │   │
│  │         └── scripts/                                                 │   │
│  │             ├── get.sh        ◄── Claude executes these             │   │
│  │             ├── describe.sh                                          │   │
│  │             └── apply.sh                                             │   │
│  │                    │                                                 │   │
│  │                    ▼                                                 │   │
│  │  ┌─────────────────────────────────────────────────────────┐        │   │
│  │  │  Claude reads SKILL.md, runs scripts via bash           │        │   │
│  │  └─────────────────────────────────────────────────────────┘        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Required Changes

#### 1. Skill Installation Command

```bash
# New command to install skills to Claude's filesystem
skill install kubernetes --target ~/.claude/skills/

# Result:
~/.claude/skills/
└── kubernetes/
    ├── SKILL.md           # Instructions for Claude
    ├── TOOLS.md           # Tool reference
    └── scripts/
        ├── get.sh
        ├── describe.sh
        ├── logs.sh
        └── ...
```

#### 2. SKILL.md Transformation

Transform from tool documentation to Claude instructions:

**Current (Tool Documentation):**
```yaml
---
name: kubernetes
description: Kubernetes cluster management...
---

# Kubernetes Skill

## Tools Provided

### get
Get Kubernetes resources...

**Parameters**:
- `resource` (required): Resource type
```

**Required (Claude Instructions):**
```yaml
---
name: kubernetes
description: Kubernetes cluster management. Use when you need to manage pods, deployments, services, or any K8s resources.
---

# Kubernetes Skill

## Quick Start

To get pods in a namespace:
```bash
./scripts/get.sh resource=pods namespace=default
```

## When to Use This Skill

Use this skill when the user asks about:
- Viewing Kubernetes resources (pods, services, deployments)
- Checking pod logs or status
- Scaling deployments
- Managing cluster configuration

## Available Scripts

### Getting Resources

Use `scripts/get.sh` to retrieve Kubernetes resources:

```bash
# Get all pods
./scripts/get.sh resource=pods

# Get pods in specific namespace
./scripts/get.sh resource=pods namespace=kube-system

# Get with wide output
./scripts/get.sh resource=pods output=wide
```

For detailed parameter reference, see [TOOLS.md](TOOLS.md).
```

#### 3. Script Generation

Generate executable scripts that wrap Skill Engine:

```bash
#!/bin/bash
# scripts/get.sh - Generated by Skill Engine
# Wraps: skill run kubernetes get

# Parse arguments
declare -A args
for arg in "$@"; do
    key="${arg%%=*}"
    value="${arg#*=}"
    args[$key]="$value"
done

# Build command
cmd="skill run kubernetes get"
[[ -n "${args[resource]}" ]] && cmd+=" resource=${args[resource]}"
[[ -n "${args[namespace]}" ]] && cmd+=" namespace=${args[namespace]}"
[[ -n "${args[output]}" ]] && cmd+=" output=${args[output]}"

# Execute
eval $cmd
```

#### 4. Validation Layer

Add YAML frontmatter validation:

```rust
// In skill-runtime or skill-cli

struct SkillMetadata {
    name: String,        // max 64 chars, lowercase, hyphens, no reserved words
    description: String, // max 1024 chars, non-empty
}

fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name.len() > 64 {
        return Err(ValidationError::NameTooLong);
    }
    if !name.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '-') {
        return Err(ValidationError::InvalidCharacters);
    }
    if name.contains("anthropic") || name.contains("claude") {
        return Err(ValidationError::ReservedWord);
    }
    Ok(())
}
```

### Pros & Cons

| Pros | Cons |
|------|------|
| 100% Claude Agent Skills compatible | Major architectural change |
| Works with claude.ai, API, and Claude Code | Loses MCP benefits (context engineering) |
| Progressive disclosure built-in | Scripts less efficient than direct execution |
| Can be shared via Claude ecosystem | Duplicates tool logic in scripts |

---

## Option B: Hybrid Bridge Approach

Generate Claude Agent Skills that instruct Claude to use MCP tools.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    OPTION B: HYBRID BRIDGE ARCHITECTURE                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  ~/.claude/skills/kubernetes/                                        │   │
│  │         │                                                            │   │
│  │         └── SKILL.md    ◄── Instructions to use MCP tools           │   │
│  │                │                                                     │   │
│  │                ▼                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  Claude reads SKILL.md:                                      │    │   │
│  │  │  "Use mcp__skill-engine__execute for Kubernetes operations"  │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  │                │                                                     │   │
│  │                ▼                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  mcp__skill-engine__execute(                                 │    │   │
│  │  │    skill='kubernetes',                                       │    │   │
│  │  │    tool='get',                                               │    │   │
│  │  │    args={resource: 'pods'}                                   │    │   │
│  │  │  )                                                           │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  │                │                                                     │   │
│  │                ▼                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  Skill Engine executes: kubectl get pods                     │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Benefits:                                                                  │
│  • Claude discovers skill via filesystem (Agent Skills spec)               │
│  • Execution via MCP (context engineering, filtering, etc.)                │
│  • Best of both worlds                                                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Bridge SKILL.md Template

```yaml
---
name: kubernetes
description: Kubernetes cluster management. Use when you need to manage pods, deployments, services, configmaps, secrets, or any K8s resources.
---

# Kubernetes Skill

This skill provides Kubernetes cluster management through the Skill Engine MCP integration.

## How to Use

**IMPORTANT**: This skill uses the `mcp__skill-engine__execute` tool. Do NOT use bash/kubectl directly.

### Basic Pattern

```
mcp__skill-engine__execute(
  skill='kubernetes',
  tool='<tool-name>',
  args={<parameters>}
)
```

## Quick Reference

### Viewing Resources

| Task | Command |
|------|---------|
| List pods | `execute(skill='kubernetes', tool='get', args={resource:'pods'})` |
| List all namespaces | `execute(skill='kubernetes', tool='get', args={resource:'namespaces'})` |
| Get deployments | `execute(skill='kubernetes', tool='get', args={resource:'deployments'})` |

### Common Operations

**Get pods in a namespace:**
```
execute(skill='kubernetes', tool='get', args={
  resource: 'pods',
  namespace: 'kube-system'
})
```

**View pod logs:**
```
execute(skill='kubernetes', tool='logs', args={
  pod: 'nginx-xxxxx',
  namespace: 'default',
  tail: 100
})
```

**Scale deployment:**
```
execute(skill='kubernetes', tool='scale', args={
  resource: 'deployment',
  name: 'nginx',
  replicas: 3
})
```

## Context Engineering Features

Use these parameters for efficient output handling:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `max_output` | Limit output size | `max_output=4000` |
| `grep` | Filter output lines | `grep='Running'` |
| `head` | First N lines | `head=10` |
| `tail` | Last N lines | `tail=20` |
| `jq` | Extract JSON fields | `jq='.items[].metadata.name'` |

**Example with filtering:**
```
execute(
  skill='kubernetes',
  tool='get',
  args={resource: 'pods', namespace: 'default'},
  grep='Running',
  head=10
)
```

## Available Tools

For complete tool documentation, use:
```
mcp__skill-engine__list_skills(skill='kubernetes')
```

Or search semantically:
```
mcp__skill-engine__search_skills(query='scale deployment replicas')
```
```

### Required Implementation

#### 1. Bridge Generator Command

```bash
# Generate Claude Agent Skills from Skill Engine skills
skill generate-claude-skills --output ~/.claude/skills/

# Or for project-local skills
skill generate-claude-skills --output .claude/skills/
```

#### 2. Generator Implementation

```rust
// New crate or module: skill-claude-bridge

pub fn generate_claude_skill(skill: &Skill, output_dir: &Path) -> Result<()> {
    let skill_dir = output_dir.join(&skill.name);
    fs::create_dir_all(&skill_dir)?;

    // Generate SKILL.md
    let skill_md = generate_skill_md(skill)?;
    fs::write(skill_dir.join("SKILL.md"), skill_md)?;

    // Generate TOOLS.md (detailed reference)
    let tools_md = generate_tools_md(skill)?;
    fs::write(skill_dir.join("TOOLS.md"), tools_md)?;

    Ok(())
}

fn generate_skill_md(skill: &Skill) -> Result<String> {
    let mut content = String::new();

    // YAML frontmatter
    content.push_str("---\n");
    content.push_str(&format!("name: {}\n", validate_name(&skill.name)?));
    content.push_str(&format!("description: {}\n", truncate(&skill.description, 1024)));
    content.push_str("---\n\n");

    // Instructions header
    content.push_str(&format!("# {} Skill\n\n", titlecase(&skill.name)));
    content.push_str("This skill uses the Skill Engine MCP integration.\n\n");

    // Usage pattern
    content.push_str("## How to Use\n\n");
    content.push_str("Use `mcp__skill-engine__execute` with:\n");
    content.push_str(&format!("- `skill='{}'`\n", skill.name));
    content.push_str("- `tool='<tool-name>'`\n");
    content.push_str("- `args={{<parameters>}}`\n\n");

    // Quick reference for each tool
    content.push_str("## Quick Reference\n\n");
    for tool in &skill.tools {
        content.push_str(&format!("### {}\n\n", tool.name));
        content.push_str(&format!("{}\n\n", tool.description));
        content.push_str("```\n");
        content.push_str(&format!(
            "execute(skill='{}', tool='{}', args={{...}})\n",
            skill.name, tool.name
        ));
        content.push_str("```\n\n");
    }

    // Reference to detailed docs
    content.push_str("## Detailed Reference\n\n");
    content.push_str("See [TOOLS.md](TOOLS.md) for complete parameter documentation.\n");

    Ok(content)
}
```

#### 3. Auto-sync Hook (Optional)

Keep Claude Skills in sync when Skill Engine config changes:

```toml
# .skill-engine.toml

[claude-bridge]
enabled = true
output = ".claude/skills"
auto_sync = true  # Regenerate on skill changes
```

### Pros & Cons

| Pros | Cons |
|------|------|
| Maintains MCP execution benefits | Not 100% spec compliant (no scripts) |
| Context engineering features preserved | Requires MCP server running |
| Less code duplication | Claude.ai without MCP won't work |
| Easier to implement | Two systems to maintain |
| Progressive disclosure via filesystem | |

---

## Recommended Approach

**Recommendation: Start with Option B (Hybrid Bridge), with Option A as future enhancement.**

### Rationale

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         RECOMMENDATION RATIONALE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. IMMEDIATE VALUE (Option B)                                              │
│     ┌───────────────────────────────────────────────────────────────────┐  │
│     │ • Works with existing Skill Engine architecture                    │  │
│     │ • Adds Claude Agent Skills discovery                               │  │
│     │ • Preserves MCP context engineering (grep, jq, max_output)        │  │
│     │ • Lower implementation effort                                      │  │
│     └───────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  2. FUTURE ENHANCEMENT (Option A)                                           │
│     ┌───────────────────────────────────────────────────────────────────┐  │
│     │ • Add script generation for claude.ai compatibility               │  │
│     │ • Full offline support without MCP                                 │  │
│     │ • Share skills via Claude ecosystem                                │  │
│     └───────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  3. PROGRESSIVE APPROACH                                                    │
│                                                                             │
│     Phase 1: Bridge Generator (Option B)                                    │
│         │                                                                   │
│         ▼                                                                   │
│     Phase 2: Add Validation Layer                                           │
│         │                                                                   │
│         ▼                                                                   │
│     Phase 3: Script Generation (Option A features)                          │
│         │                                                                   │
│         ▼                                                                   │
│     Phase 4: Full Standalone Mode                                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Roadmap

### Phase 1: Bridge Generator (Option B Core)

**Effort: 1-2 weeks**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 1 DELIVERABLES                                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. New CLI command: `skill generate-claude-skills`                         │
│                                                                             │
│  2. SKILL.md template generator                                             │
│     • YAML frontmatter with validation                                      │
│     • MCP usage instructions                                                │
│     • Quick reference for tools                                             │
│                                                                             │
│  3. TOOLS.md reference generator                                            │
│     • Detailed parameter documentation                                      │
│     • Examples for each tool                                                │
│                                                                             │
│  4. Output to ~/.claude/skills/ or .claude/skills/                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Files to create/modify:**
- `crates/skill-cli/src/commands/generate_claude_skills.rs` (new)
- `crates/skill-cli/src/templates/skill_md.rs` (new)
- `crates/skill-cli/src/templates/tools_md.rs` (new)

### Phase 2: Validation Layer

**Effort: 3-5 days**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 2 DELIVERABLES                                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. YAML frontmatter validation                                             │
│     • name: max 64 chars, lowercase, hyphens, no reserved words            │
│     • description: max 1024 chars, non-empty                                │
│                                                                             │
│  2. CLI validation command: `skill validate`                                │
│                                                                             │
│  3. Validation on skill load (warn if non-compliant)                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Phase 3: Script Generation (Option A Features)

**Effort: 1-2 weeks**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 3 DELIVERABLES                                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. Script generator for each tool                                          │
│     • Bash scripts that wrap `skill run`                                    │
│     • Parameter parsing                                                     │
│     • Error handling                                                        │
│                                                                             │
│  2. Updated SKILL.md to reference scripts                                   │
│                                                                             │
│  3. Hybrid mode: scripts OR MCP (configurable)                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Phase 4: Full Standalone Mode

**Effort: 2-3 weeks**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 4 DELIVERABLES                                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. Standalone skill packages (no MCP required)                             │
│                                                                             │
│  2. Skill distribution via:                                                 │
│     • ZIP files for claude.ai upload                                        │
│     • Claude API Skills endpoint                                            │
│     • npm/cargo packages                                                    │
│                                                                             │
│  3. Progressive disclosure optimization                                     │
│     • Separate SKILL.md sections by loading level                           │
│     • Lazy-load reference documentation                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Quick Start: Phase 1 Implementation

### 1. Add CLI Command

```rust
// crates/skill-cli/src/commands/mod.rs
mod generate_claude_skills;

// crates/skill-cli/src/commands/generate_claude_skills.rs
use clap::Args;

#[derive(Args)]
pub struct GenerateClaudeSkillsArgs {
    /// Output directory (default: ~/.claude/skills)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Specific skill to generate (default: all)
    #[arg(short, long)]
    skill: Option<String>,

    /// Overwrite existing files
    #[arg(long)]
    force: bool,
}

pub async fn run(args: GenerateClaudeSkillsArgs) -> Result<()> {
    let output = args.output.unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap()
            .join(".claude")
            .join("skills")
    });

    let manifest = load_manifest()?;

    for (name, skill) in manifest.skills {
        if args.skill.is_none() || args.skill.as_ref() == Some(&name) {
            generate_skill(&skill, &output, args.force)?;
            println!("Generated: {}/", output.join(&name).display());
        }
    }

    println!("\nClaude Agent Skills generated at: {}", output.display());
    Ok(())
}
```

### 2. Run Generator

```bash
# Generate all skills
skill generate-claude-skills

# Generate specific skill
skill generate-claude-skills --skill kubernetes

# Custom output location
skill generate-claude-skills --output .claude/skills
```

### 3. Verify in Claude Code

```bash
# Claude Code will now discover the skills
claude

# Claude should see and use the kubernetes skill
> "Show me all pods in kube-system"
# Claude reads ~/.claude/skills/kubernetes/SKILL.md
# Claude uses mcp__skill-engine__execute
```

---

## Summary

| Phase | Deliverable | Effort | Compatibility |
|-------|-------------|--------|---------------|
| **Phase 1** | Bridge Generator | 1-2 weeks | ~80% |
| **Phase 2** | Validation Layer | 3-5 days | ~85% |
| **Phase 3** | Script Generation | 1-2 weeks | ~95% |
| **Phase 4** | Standalone Mode | 2-3 weeks | 100% |

**Recommended starting point**: Phase 1 + Phase 2 provides immediate value with moderate effort.
