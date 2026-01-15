# Technical Specification: Hybrid Bridge Generator

## Overview

Generate Claude Agent Skills with **dual-mode execution**: MCP tools (preferred) + executable scripts (fallback). This achieves 100% compliance with Anthropic's Agent Skills specification while preserving Skill Engine's context engineering capabilities.

### Key Features

- **100% Agent Skills Compliance**: Filesystem discovery, SKILL.md, scripts/
- **Dual Execution Modes**: MCP (with grep/jq/max_output) or scripts (portable)
- **Single Source of Truth**: Scripts wrap `skill run`, no logic duplication

---

## Table of Contents

1. [Architecture](#architecture)
2. [Data Flow](#data-flow)
3. [CLI Command Design](#cli-command-design)
4. [SKILL.md Template Structure](#skillmd-template-structure)
5. [Generator Implementation](#generator-implementation)
6. [Configuration](#configuration)
7. [Output Structure](#output-structure)
8. [Validation](#validation)
9. [Edge Cases](#edge-cases)
10. [Testing Strategy](#testing-strategy)

---

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         HYBRID BRIDGE ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        INPUT SOURCES                                 │   │
│  │                                                                      │   │
│  │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐   │   │
│  │  │.skill-engine.toml│  │    SKILL.md      │  │    skill.wit     │   │   │
│  │  │                  │  │  (existing docs) │  │ (tool interface) │   │   │
│  │  │ - skill configs  │  │                  │  │                  │   │   │
│  │  │ - instances      │  │ - description    │  │ - tool names     │   │   │
│  │  │ - services       │  │ - tool docs      │  │ - parameters     │   │   │
│  │  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘   │   │
│  │           │                     │                     │              │   │
│  └───────────┼─────────────────────┼─────────────────────┼──────────────┘   │
│              │                     │                     │                  │
│              └─────────────────────┼─────────────────────┘                  │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      BRIDGE GENERATOR                                │   │
│  │                                                                      │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │   │
│  │  │ Skill Loader │─►│  Transformer │─►│  Renderer    │               │   │
│  │  │              │  │              │  │              │               │   │
│  │  │ - Parse TOML │  │ - Validate   │  │ - Templates  │               │   │
│  │  │ - Load tools │  │ - Normalize  │  │ - Markdown   │               │   │
│  │  │ - Read docs  │  │ - Categorize │  │ - YAML       │               │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘               │   │
│  │                                                                      │   │
│  └──────────────────────────────────┬───────────────────────────────────┘   │
│                                     │                                       │
│                                     ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        OUTPUT (Claude Agent Skills)                  │   │
│  │                                                                      │   │
│  │  ~/.claude/skills/kubernetes/                                        │   │
│  │  ├── SKILL.md              # Instructions (MCP preferred, scripts)   │   │
│  │  ├── TOOLS.md              # Detailed reference (Level 3)            │   │
│  │  └── scripts/              # Executable wrappers (100% compliance)   │   │
│  │      ├── get.sh                → skill run kubernetes get "$@"       │   │
│  │      ├── describe.sh           → skill run kubernetes describe "$@"  │   │
│  │      ├── logs.sh               → skill run kubernetes logs "$@"      │   │
│  │      └── ...                                                         │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility |
|-----------|----------------|
| **Skill Loader** | Parse manifest, load tool definitions, read existing SKILL.md |
| **Transformer** | Validate names, normalize data, categorize tools |
| **Renderer** | Generate instruction-oriented SKILL.md and TOOLS.md |
| **Script Generator** | Generate executable scripts that wrap `skill run` |

### Dual-Mode Execution Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         DUAL-MODE EXECUTION                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Claude Code (MCP available)              claude.ai (No MCP)                │
│  ────────────────────────────             ──────────────────                │
│                                                                             │
│  SKILL.md instructs:                      SKILL.md instructs:               │
│  "Prefer MCP for large outputs"           "Use scripts"                     │
│           │                                        │                        │
│           ▼                                        ▼                        │
│  mcp__skill-engine__execute(              ./scripts/get.sh resource=pods    │
│    skill='kubernetes',                             │                        │
│    tool='get',                                     │                        │
│    args={resource:'pods'},                         │                        │
│    grep='Running',  ◄── context eng.               │                        │
│    max_output=4000                                 │                        │
│  )                                                 │                        │
│           │                                        │                        │
│           └────────────────┬───────────────────────┘                        │
│                            ▼                                                │
│                  ┌─────────────────┐                                        │
│                  │  Skill Engine   │                                        │
│                  │  (execution)    │                                        │
│                  └─────────────────┘                                        │
│                            │                                                │
│                            ▼                                                │
│                  kubectl get pods                                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Data Flow

### Generation Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            DATA FLOW                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. LOAD PHASE                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                             │
│  .skill-engine.toml ──────────────────────────────────────────────────┐    │
│       │                                                                │    │
│       ▼                                                                │    │
│  ┌─────────────────────────────────────────────────────────────────┐  │    │
│  │  Parse Manifest                                                  │  │    │
│  │  ┌─────────────────────────────────────────────────────────┐    │  │    │
│  │  │  skills: [                                               │    │  │    │
│  │  │    { name: "kubernetes", source: "...", description }    │    │  │    │
│  │  │    { name: "docker", source: "...", description }        │    │  │    │
│  │  │  ]                                                       │    │  │    │
│  │  └─────────────────────────────────────────────────────────┘    │  │    │
│  └─────────────────────────────────────────────────────────────────┘  │    │
│       │                                                                │    │
│       ▼                                                                │    │
│  ┌─────────────────────────────────────────────────────────────────┐  │    │
│  │  For each skill:                                                 │  │    │
│  │  ┌───────────────────────────────────────────────────────────┐  │  │    │
│  │  │  Load Tools (via get_tools() or skill.wit)                │  │  │    │
│  │  │  ┌─────────────────────────────────────────────────────┐  │  │  │    │
│  │  │  │  tools: [                                            │  │  │  │    │
│  │  │  │    { name: "get", params: [...], description }       │  │  │  │    │
│  │  │  │    { name: "describe", params: [...], description }  │  │  │  │    │
│  │  │  │  ]                                                   │  │  │  │    │
│  │  │  └─────────────────────────────────────────────────────┘  │  │  │    │
│  │  └───────────────────────────────────────────────────────────┘  │  │    │
│  └─────────────────────────────────────────────────────────────────┘  │    │
│       │                                                                │    │
│       │  Load existing SKILL.md (if exists) ◄──────────────────────────┘    │
│       │  for: description, examples, when-to-use                            │
│       ▼                                                                     │
│                                                                             │
│  2. TRANSFORM PHASE                                                         │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────┐       │
│  │  Validate & Normalize                                            │       │
│  │  ┌─────────────────────────────────────────────────────────┐    │       │
│  │  │  - Validate name (max 64 chars, lowercase, hyphens)     │    │       │
│  │  │  - Truncate description (max 1024 chars)                │    │       │
│  │  │  - Categorize tools by function                         │    │       │
│  │  │  - Generate quick reference examples                    │    │       │
│  │  └─────────────────────────────────────────────────────────┘    │       │
│  └─────────────────────────────────────────────────────────────────┘       │
│       │                                                                     │
│       ▼                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐       │
│  │  Intermediate Representation                                     │       │
│  │  ┌─────────────────────────────────────────────────────────┐    │       │
│  │  │  ClaudeSkill {                                           │    │       │
│  │  │    name: "kubernetes",                                   │    │       │
│  │  │    description: "...",                                   │    │       │
│  │  │    when_to_use: ["..."],                                 │    │       │
│  │  │    tool_categories: {                                    │    │       │
│  │  │      "Resource Viewing": [get, describe, logs],          │    │       │
│  │  │      "Resource Management": [create, delete, apply],     │    │       │
│  │  │    },                                                    │    │       │
│  │  │    quick_reference: [...],                               │    │       │
│  │  │    context_features: [...],                              │    │       │
│  │  │  }                                                       │    │       │
│  │  └─────────────────────────────────────────────────────────┘    │       │
│  └─────────────────────────────────────────────────────────────────┘       │
│       │                                                                     │
│       ▼                                                                     │
│                                                                             │
│  3. RENDER PHASE                                                            │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────┐       │
│  │  Apply Templates                                                 │       │
│  │  ┌─────────────────────────────────────────────────────────┐    │       │
│  │  │  SKILL.md Template ──► Instruction-oriented markdown    │    │       │
│  │  │  TOOLS.md Template ──► Detailed reference (Level 3)     │    │       │
│  │  └─────────────────────────────────────────────────────────┘    │       │
│  └─────────────────────────────────────────────────────────────────┘       │
│       │                                                                     │
│       ▼                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐       │
│  │  Write Output                                                    │       │
│  │  ┌─────────────────────────────────────────────────────────┐    │       │
│  │  │  ~/.claude/skills/kubernetes/SKILL.md                   │    │       │
│  │  │  ~/.claude/skills/kubernetes/TOOLS.md                   │    │       │
│  │  └─────────────────────────────────────────────────────────┘    │       │
│  └─────────────────────────────────────────────────────────────────┘       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Runtime Flow (After Generation)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          RUNTIME FLOW                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  User: "Show me all failing pods"                                           │
│                     │                                                       │
│                     ▼                                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  LEVEL 1: Metadata (System Prompt)                                   │   │
│  │  ┌───────────────────────────────────────────────────────────────┐  │   │
│  │  │  Available Skills:                                             │  │   │
│  │  │  - kubernetes: Kubernetes cluster management. Use for pods...  │  │   │
│  │  │  - docker: Docker container management...                      │  │   │
│  │  └───────────────────────────────────────────────────────────────┘  │   │
│  │                                                                      │   │
│  │  Claude matches: "pods" → kubernetes skill                           │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                     │                                                       │
│                     ▼                                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  LEVEL 2: Instructions (Claude reads SKILL.md)                       │   │
│  │  ┌───────────────────────────────────────────────────────────────┐  │   │
│  │  │  $ cat ~/.claude/skills/kubernetes/SKILL.md                    │  │   │
│  │  │                                                                │  │   │
│  │  │  → Claude learns: "Use mcp__skill-engine__execute"             │  │   │
│  │  │  → Claude sees quick reference for 'get' tool                  │  │   │
│  │  └───────────────────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                     │                                                       │
│                     ▼                                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  EXECUTION: MCP Tool Call                                            │   │
│  │  ┌───────────────────────────────────────────────────────────────┐  │   │
│  │  │  mcp__skill-engine__execute(                                   │  │   │
│  │  │    skill='kubernetes',                                         │  │   │
│  │  │    tool='get',                                                 │  │   │
│  │  │    args={resource: 'pods', output: 'wide'},                    │  │   │
│  │  │    grep='0/1|Error|CrashLoop'   ◄── Context engineering!       │  │   │
│  │  │  )                                                             │  │   │
│  │  └───────────────────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                     │                                                       │
│                     ▼                                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  RESULT: Filtered output returned to Claude                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## CLI Command Design

### Integration with Existing `skill claude` Command

The bridge generator integrates with the existing `skill claude` command structure:

```
skill claude
├── setup          # Existing: Configure MCP server in .mcp.json
├── status         # Existing: Check integration status
├── remove         # Existing: Remove MCP configuration
└── generate       # NEW: Generate Claude Agent Skills (bridge)
```

### Command Signature

```bash
skill claude generate [OPTIONS]
```

### Options

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--output` | `-o` | path | `~/.claude/skills` | Output directory |
| `--skill` | `-s` | string | (all) | Generate for specific skill only |
| `--project` | `-p` | flag | false | Output to `.claude/skills/` (project-local) |
| `--force` | `-f` | flag | false | Overwrite existing files |
| `--dry-run` | | flag | false | Show what would be generated |
| `--format` | | enum | `standard` | Output format: `standard`, `minimal`, `verbose` |
| `--include-tools-md` | | flag | true | Generate TOOLS.md reference file |
| `--no-scripts` | | flag | false | Skip scripts/ directory (MCP-only mode) |
| `--scripts-only` | | flag | false | Skip MCP instructions (scripts-only mode) |

### Usage Examples

```bash
# Generate all skills (with scripts for 100% compliance)
skill claude generate

# Generate only kubernetes skill
skill claude generate --skill kubernetes

# Generate to project directory
skill claude generate --project

# Preview without writing
skill claude generate --dry-run

# Force overwrite existing
skill claude generate --force --skill docker

# Minimal output (smaller SKILL.md)
skill claude generate --format minimal

# MCP-only (no scripts, smaller output)
skill claude generate --no-scripts

# Full setup: MCP + Agent Skills
skill claude setup && skill claude generate
```

### Relationship to Existing Commands

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CLAUDE COMMAND RESPONSIBILITIES                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  skill claude setup                                                         │
│  ─────────────────                                                          │
│  • Configures .mcp.json (MCP server registration)                           │
│  • Enables: mcp__skill-engine__execute, list_skills, search_skills          │
│  • Output: .mcp.json or ~/.config/claude/mcp.json                           │
│                                                                             │
│  skill claude generate     (NEW)                                            │
│  ────────────────────────                                                   │
│  • Generates Claude Agent Skills (SKILL.md files)                           │
│  • Enables: Filesystem-based skill discovery                                │
│  • Output: .claude/skills/ or ~/.claude/skills/                             │
│                                                                             │
│  Combined Effect:                                                           │
│  ┌───────────────────────────────────────────────────────────────────┐     │
│  │  1. Claude discovers skills via filesystem (Agent Skills spec)    │     │
│  │  2. SKILL.md instructs Claude to use MCP tools                    │     │
│  │  3. MCP server executes actual commands                           │     │
│  └───────────────────────────────────────────────────────────────────┘     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Manifest not found |
| 4 | Skill not found |
| 5 | Output directory not writable |
| 6 | Validation errors |

---

## SKILL.md Template Structure

### Template Variables

```rust
struct SkillTemplateContext {
    // Frontmatter (Level 1)
    name: String,                    // Validated skill name
    description: String,             // Truncated to 1024 chars

    // Metadata
    version: Option<String>,
    skill_engine_version: String,
    generated_at: DateTime,

    // When to use (for instructions)
    when_to_use: Vec<String>,        // Extracted or generated triggers

    // Tools organized by category
    tool_categories: HashMap<String, Vec<Tool>>,

    // Quick reference examples
    quick_reference: Vec<QuickRef>,

    // Context engineering features
    context_features: Vec<ContextFeature>,

    // Advanced features (Level 3 references)
    has_tools_md: bool,
    additional_resources: Vec<String>,
}

struct Tool {
    name: String,
    description: String,
    parameters: Vec<Parameter>,
    examples: Vec<Example>,
}

struct QuickRef {
    task: String,           // "List all pods"
    command: String,        // Full MCP call
}

struct ContextFeature {
    name: String,           // "grep"
    description: String,    // "Filter output lines"
    example: String,        // "grep='Running'"
}
```

### SKILL.md Template (Dual-Mode - Standard Format)

```markdown
---
name: {{name}}
description: {{description}}
---

# {{title}} Skill

{{description}}

## When to Use This Skill

Use this skill when the user:
{{#each when_to_use}}
- {{this}}
{{/each}}

## How to Execute

This skill supports two execution methods. **Choose based on your environment:**

### Method 1: MCP Tools (Preferred in Claude Code)

Use MCP when available - enables output filtering and context engineering:

```
mcp__skill-engine__execute(
  skill='{{name}}',
  tool='<tool-name>',
  args={<parameters>},
  grep='<filter>',      // Optional: filter output
  max_output=4000       // Optional: limit output size
)
```

### Method 2: Scripts (Fallback / claude.ai)

Use bundled scripts when MCP is not available:

```bash
./scripts/<tool-name>.sh <param>=<value> ...
```

## Quick Reference

| Task | MCP | Script |
|------|-----|--------|
{{#each quick_reference}}
| {{task}} | `execute(skill='{{../name}}', tool='{{tool}}', args={ {{args}} })` | `./scripts/{{tool}}.sh {{args}}` |
{{/each}}

{{#each tool_categories}}
## {{@key}}

{{#each this}}
### {{name}}

{{description}}

**MCP:**
```
execute(skill='{{../../name}}', tool='{{name}}', args={
{{#each parameters}}
  {{name}}: <{{type}}>{{#if required}} (required){{/if}}{{#unless @last}},{{/unless}}
{{/each}}
})
```

**Script:**
```bash
./scripts/{{name}}.sh{{#each parameters}}{{#if required}} {{name}}=<value>{{/if}}{{/each}}
```

{{/each}}
{{/each}}

## Context Engineering (MCP Only)

When using MCP, these parameters help handle large outputs efficiently:

| Parameter | Description | Example |
|-----------|-------------|---------|
{{#each context_features}}
| `{{name}}` | {{description}} | `{{example}}` |
{{/each}}

**Example - Get only failing pods:**
```
execute(
  skill='{{name}}',
  tool='get',
  args={resource: 'pods', namespace: 'all'},
  grep='CrashLoop|Error|0/',
  max_output=4000
)
```

{{#if has_tools_md}}
## Detailed Reference

For complete parameter documentation, see [TOOLS.md](TOOLS.md).
{{/if}}

{{#if has_scripts}}
## Available Scripts

{{#each tools}}
- `./scripts/{{name}}.sh` - {{description}}
{{/each}}
{{/if}}

---
*Generated by Skill Engine v{{skill_engine_version}} at {{generated_at}}*
```

### TOOLS.md Template (Reference Document)

```markdown
# {{title}} - Tool Reference

Complete parameter reference for all {{name}} skill tools.

{{#each tool_categories}}
## {{@key}}

{{#each this}}
### {{name}}

{{description}}

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
{{#each parameters}}
| `{{name}}` | {{type}} | {{#if required}}Yes{{else}}No{{/if}} | {{default}} | {{description}} |
{{/each}}

#### Examples

{{#each examples}}
**{{title}}**

```
execute(skill='{{../../name}}', tool='{{../name}}', args={
{{#each args}}
  {{@key}}: {{this}}{{#unless @last}},{{/unless}}
{{/each}}
})
```

{{/each}}

---

{{/each}}
{{/each}}

*Generated by Skill Engine v{{skill_engine_version}}*
```

---

## Generator Implementation

### Module Structure

```
crates/skill-cli/src/
├── commands/
│   ├── mod.rs
│   └── claude.rs                     # Existing + add generate() function
├── claude_bridge/                    # New module
│   ├── mod.rs
│   ├── loader.rs                     # Load skills from manifest
│   ├── transformer.rs                # Transform to Claude format
│   ├── renderer.rs                   # Render templates
│   ├── validator.rs                  # Validate output
│   └── templates/
│       ├── mod.rs
│       ├── skill_md.hbs              # SKILL.md Handlebars template
│       └── tools_md.hbs              # TOOLS.md Handlebars template
```

### Integration with Existing claude.rs

Add to existing `crates/skill-cli/src/commands/claude.rs`:

```rust
// Add to existing imports
use crate::claude_bridge::{self, GenerateOptions, OutputFormat};

/// Generate Claude Agent Skills from Skill Engine skills
pub async fn generate(
    output: Option<PathBuf>,
    skill_filter: Option<&str>,
    project: bool,
    force: bool,
    dry_run: bool,
    format: OutputFormat,
    include_tools_md: bool,
) -> Result<()> {
    // Determine output directory
    let output_dir = if project {
        get_project_skills_path()?
    } else {
        output.unwrap_or_else(|| get_global_skills_path().unwrap())
    };

    let options = GenerateOptions {
        output_dir: output_dir.clone(),
        skill_filter: skill_filter.map(String::from),
        force,
        dry_run,
        format,
        include_tools_md,
    };

    // Run the bridge generator
    let generated = claude_bridge::generate(options).await?;

    // Print summary
    println!();
    if dry_run {
        println!("{} Dry run - no files written", "ℹ".blue());
    } else {
        println!(
            "{} Generated {} Claude Agent Skills to {}",
            "✓".green().bold(),
            generated.len(),
            output_dir.display()
        );
    }

    println!();
    println!("Generated skills:");
    for skill in &generated {
        println!("  • {}/SKILL.md", skill.cyan());
    }

    if !dry_run {
        println!();
        println!(
            "{} Claude Code will now discover these skills automatically.",
            "✓".green()
        );
        println!();
        println!("The skills instruct Claude to use MCP tools:");
        println!("  mcp__skill-engine__execute(skill='{}', tool='...', args={{...}})",
            generated.first().unwrap_or(&"kubernetes".to_string())
        );
    }

    Ok(())
}

/// Get project-level .claude/skills path
fn get_project_skills_path() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("Unable to get current directory")?;
    Ok(cwd.join(".claude/skills"))
}

/// Get global ~/.claude/skills path
fn get_global_skills_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    Ok(PathBuf::from(home).join(".claude/skills"))
}
```

### main.rs Subcommand Addition

```rust
// In main.rs, extend the Claude subcommand enum

#[derive(Subcommand)]
enum ClaudeCommands {
    /// Configure Claude Code MCP integration
    Setup {
        /// Configure globally instead of project-local
        #[arg(long)]
        global: bool,
        /// Custom server name
        #[arg(long)]
        name: Option<String>,
        /// Custom binary path
        #[arg(long)]
        binary: Option<String>,
    },
    /// Check Claude Code integration status
    Status,
    /// Remove Skill Engine from Claude Code
    Remove {
        #[arg(long)]
        global: bool,
        #[arg(long)]
        name: Option<String>,
    },
    /// Generate Claude Agent Skills (NEW)
    Generate {
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Generate for specific skill only
        #[arg(short, long)]
        skill: Option<String>,
        /// Output to .claude/skills/ (project-local)
        #[arg(short, long)]
        project: bool,
        /// Overwrite existing files
        #[arg(short, long)]
        force: bool,
        /// Preview without writing
        #[arg(long)]
        dry_run: bool,
        /// Output format: standard, minimal, verbose
        #[arg(long, default_value = "standard")]
        format: String,
        /// Skip generating TOOLS.md
        #[arg(long)]
        no_tools_md: bool,
    },
}
```

### Core Data Structures

```rust
// claude_bridge/mod.rs

/// Intermediate representation of a Claude-compatible skill
#[derive(Debug, Clone)]
pub struct ClaudeSkill {
    pub name: String,
    pub description: String,
    pub when_to_use: Vec<String>,
    pub tool_categories: IndexMap<String, Vec<ClaudeTool>>,
    pub quick_reference: Vec<QuickReference>,
    pub context_features: Vec<ContextFeature>,
    pub source_skill: SkillMetadata,
}

#[derive(Debug, Clone)]
pub struct ClaudeTool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
    pub examples: Vec<ToolExample>,
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub default: Option<String>,
    pub description: String,
    pub allowed_values: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct QuickReference {
    pub task: String,
    pub tool: String,
    pub args: String,
}

#[derive(Debug, Clone)]
pub struct ContextFeature {
    pub name: String,
    pub description: String,
    pub example: String,
}

/// Generation options
#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub output_dir: PathBuf,
    pub skill_filter: Option<String>,
    pub force: bool,
    pub dry_run: bool,
    pub format: OutputFormat,
    pub include_tools_md: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Standard,
    Minimal,
    Verbose,
}
```

### Loader Implementation

```rust
// claude_bridge/loader.rs

use crate::manifest::SkillManifest;
use crate::runtime::SkillRuntime;

pub struct SkillLoader {
    manifest: SkillManifest,
    runtime: SkillRuntime,
}

impl SkillLoader {
    pub fn new(manifest_path: &Path) -> Result<Self> {
        let manifest = SkillManifest::load(manifest_path)?;
        let runtime = SkillRuntime::new()?;
        Ok(Self { manifest, runtime })
    }

    /// Load all skills from manifest
    pub async fn load_all(&self) -> Result<Vec<LoadedSkill>> {
        let mut skills = Vec::new();

        for (name, config) in &self.manifest.skills {
            let skill = self.load_skill(name, config).await?;
            skills.push(skill);
        }

        Ok(skills)
    }

    /// Load a single skill
    pub async fn load_skill(&self, name: &str, config: &SkillConfig) -> Result<LoadedSkill> {
        // Get tools from runtime (calls get_tools())
        let tools = self.runtime.get_tools(name).await?;

        // Load existing SKILL.md if present
        let skill_md = self.load_skill_md(&config.source)?;

        // Extract metadata from existing docs
        let when_to_use = self.extract_when_to_use(&skill_md);
        let examples = self.extract_examples(&skill_md);

        Ok(LoadedSkill {
            name: name.to_string(),
            config: config.clone(),
            tools,
            existing_skill_md: skill_md,
            when_to_use,
            examples,
        })
    }

    fn load_skill_md(&self, source: &str) -> Option<String> {
        let skill_md_path = Path::new(source).join("SKILL.md");
        fs::read_to_string(skill_md_path).ok()
    }

    fn extract_when_to_use(&self, skill_md: &Option<String>) -> Vec<String> {
        // Parse "When to Use" section from existing SKILL.md
        // Or generate sensible defaults based on skill name
        // ...
    }
}
```

### Transformer Implementation

```rust
// claude_bridge/transformer.rs

pub struct Transformer {
    tool_categorizer: ToolCategorizer,
    validator: Validator,
}

impl Transformer {
    pub fn transform(&self, loaded: LoadedSkill) -> Result<ClaudeSkill> {
        // Validate and normalize name
        let name = self.validator.validate_name(&loaded.name)?;

        // Truncate description
        let description = self.validator.validate_description(
            &loaded.config.description
        )?;

        // Categorize tools
        let tool_categories = self.tool_categorizer.categorize(&loaded.tools);

        // Generate quick reference
        let quick_reference = self.generate_quick_reference(&loaded);

        // Standard context features
        let context_features = self.standard_context_features();

        Ok(ClaudeSkill {
            name,
            description,
            when_to_use: loaded.when_to_use,
            tool_categories,
            quick_reference,
            context_features,
            source_skill: loaded.into(),
        })
    }

    fn standard_context_features(&self) -> Vec<ContextFeature> {
        vec![
            ContextFeature {
                name: "max_output".into(),
                description: "Limit output size (chars)".into(),
                example: "max_output=4000".into(),
            },
            ContextFeature {
                name: "grep".into(),
                description: "Filter output lines by regex".into(),
                example: "grep='Running|Completed'".into(),
            },
            ContextFeature {
                name: "head".into(),
                description: "Return first N lines".into(),
                example: "head=10".into(),
            },
            ContextFeature {
                name: "tail".into(),
                description: "Return last N lines".into(),
                example: "tail=20".into(),
            },
            ContextFeature {
                name: "jq".into(),
                description: "Extract JSON fields".into(),
                example: "jq='.items[].metadata.name'".into(),
            },
        ]
    }
}

/// Categorize tools by function
pub struct ToolCategorizer {
    // Category rules based on tool name patterns
    rules: Vec<CategoryRule>,
}

impl ToolCategorizer {
    pub fn categorize(&self, tools: &[Tool]) -> IndexMap<String, Vec<ClaudeTool>> {
        let mut categories: IndexMap<String, Vec<ClaudeTool>> = IndexMap::new();

        for tool in tools {
            let category = self.detect_category(tool);
            categories
                .entry(category)
                .or_default()
                .push(tool.into());
        }

        categories
    }

    fn detect_category(&self, tool: &Tool) -> String {
        // Match against rules
        for rule in &self.rules {
            if rule.matches(tool) {
                return rule.category.clone();
            }
        }
        "Other".into()
    }
}

// Default categorization rules
fn default_category_rules() -> Vec<CategoryRule> {
    vec![
        CategoryRule::new("Viewing", &["get", "list", "show", "describe", "status"]),
        CategoryRule::new("Management", &["create", "delete", "update", "apply", "remove"]),
        CategoryRule::new("Execution", &["run", "exec", "execute", "invoke"]),
        CategoryRule::new("Logs & Debugging", &["logs", "debug", "trace", "inspect"]),
        CategoryRule::new("Configuration", &["config", "set", "configure"]),
        CategoryRule::new("Scaling", &["scale", "resize", "replicas"]),
    ]
}
```

### Renderer Implementation

```rust
// claude_bridge/renderer.rs

use handlebars::Handlebars;

pub struct Renderer {
    handlebars: Handlebars<'static>,
    format: OutputFormat,
}

impl Renderer {
    pub fn new(format: OutputFormat) -> Self {
        let mut handlebars = Handlebars::new();

        // Register templates
        handlebars.register_template_string(
            "skill_md",
            include_str!("templates/skill_md.hbs")
        ).unwrap();

        handlebars.register_template_string(
            "tools_md",
            include_str!("templates/tools_md.hbs")
        ).unwrap();

        // Register helpers
        handlebars.register_helper("titlecase", Box::new(titlecase_helper));

        Self { handlebars, format }
    }

    pub fn render_skill_md(&self, skill: &ClaudeSkill) -> Result<String> {
        let template = match self.format {
            OutputFormat::Standard => "skill_md",
            OutputFormat::Minimal => "skill_md_minimal",
            OutputFormat::Verbose => "skill_md_verbose",
        };

        let context = self.build_context(skill);
        self.handlebars.render(template, &context)
            .map_err(|e| anyhow!("Template rendering failed: {}", e))
    }

    pub fn render_tools_md(&self, skill: &ClaudeSkill) -> Result<String> {
        let context = self.build_context(skill);
        self.handlebars.render("tools_md", &context)
            .map_err(|e| anyhow!("Template rendering failed: {}", e))
    }

    fn build_context(&self, skill: &ClaudeSkill) -> serde_json::Value {
        json!({
            "name": skill.name,
            "title": titlecase(&skill.name),
            "description": skill.description,
            "when_to_use": skill.when_to_use,
            "tool_categories": skill.tool_categories,
            "quick_reference": skill.quick_reference,
            "context_features": skill.context_features,
            "has_tools_md": true,
            "skill_engine_version": env!("CARGO_PKG_VERSION"),
            "generated_at": chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        })
    }
}
```

### Script Generator Implementation

```rust
// claude_bridge/script_generator.rs

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub struct ScriptGenerator {
    skill_name: String,
}

impl ScriptGenerator {
    pub fn new(skill_name: &str) -> Self {
        Self {
            skill_name: skill_name.to_string(),
        }
    }

    /// Generate all scripts for a skill
    pub fn generate_scripts(
        &self,
        tools: &[ClaudeTool],
        output_dir: &Path,
    ) -> Result<Vec<String>> {
        let scripts_dir = output_dir.join("scripts");
        fs::create_dir_all(&scripts_dir)?;

        let mut generated = Vec::new();

        for tool in tools {
            let script_path = scripts_dir.join(format!("{}.sh", tool.name));
            let script_content = self.generate_script(tool);

            fs::write(&script_path, &script_content)?;

            // Make executable (chmod +x)
            let mut perms = fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms)?;

            generated.push(tool.name.clone());
        }

        Ok(generated)
    }

    /// Generate a single script
    fn generate_script(&self, tool: &ClaudeTool) -> String {
        let mut script = String::new();

        // Shebang
        script.push_str("#!/bin/bash\n");

        // Header comment
        script.push_str(&format!(
            "# {}/scripts/{}.sh - Generated by Skill Engine\n",
            self.skill_name, tool.name
        ));
        script.push_str("#\n");
        script.push_str(&format!("# {}\n", tool.description));
        script.push_str("#\n");

        // Usage
        script.push_str("# Usage:\n");
        let example_args = self.generate_example_args(tool);
        script.push_str(&format!("#   ./{}.sh {}\n", tool.name, example_args));
        script.push_str("#\n");

        // Parameters documentation
        script.push_str("# Parameters:\n");
        for param in &tool.parameters {
            let required = if param.required { "(required)" } else { "" };
            script.push_str(&format!(
                "#   {:12} - {} {}\n",
                param.name, param.description, required
            ));
        }
        script.push_str("\n");

        // Script body
        script.push_str("set -euo pipefail\n");
        script.push_str(&format!(
            "exec skill run {} {} \"$@\"\n",
            self.skill_name, tool.name
        ));

        script
    }

    fn generate_example_args(&self, tool: &ClaudeTool) -> String {
        tool.parameters
            .iter()
            .filter(|p| p.required)
            .map(|p| format!("{}=<{}>", p.name, p.param_type))
            .collect::<Vec<_>>()
            .join(" ")
    }
}
```

### Main Command Implementation

```rust
// commands/generate_claude_skills.rs

use clap::Args;
use crate::claude_bridge::{SkillLoader, Transformer, Renderer, GenerateOptions};

#[derive(Args)]
pub struct GenerateClaudeSkillsArgs {
    /// Output directory
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Generate for specific skill only
    #[arg(short, long)]
    skill: Option<String>,

    /// Output to .claude/skills/ (project-local)
    #[arg(short, long)]
    project: bool,

    /// Overwrite existing files
    #[arg(short, long)]
    force: bool,

    /// Preview without writing
    #[arg(long)]
    dry_run: bool,

    /// Output format
    #[arg(long, default_value = "standard")]
    format: OutputFormat,

    /// Include TOOLS.md reference
    #[arg(long, default_value = "true")]
    include_tools_md: bool,
}

pub async fn run(args: GenerateClaudeSkillsArgs) -> Result<()> {
    // Determine output directory
    let output_dir = if args.project {
        PathBuf::from(".claude/skills")
    } else {
        args.output.unwrap_or_else(|| {
            dirs::home_dir()
                .expect("Could not find home directory")
                .join(".claude/skills")
        })
    };

    let options = GenerateOptions {
        output_dir: output_dir.clone(),
        skill_filter: args.skill,
        force: args.force,
        dry_run: args.dry_run,
        format: args.format,
        include_tools_md: args.include_tools_md,
    };

    // Load manifest
    let manifest_path = find_manifest()?;
    let loader = SkillLoader::new(&manifest_path)?;

    // Load skills
    let loaded_skills = loader.load_all().await?;

    // Filter if requested
    let skills_to_generate: Vec<_> = if let Some(ref filter) = options.skill_filter {
        loaded_skills.into_iter()
            .filter(|s| s.name == *filter)
            .collect()
    } else {
        loaded_skills
    };

    if skills_to_generate.is_empty() {
        if options.skill_filter.is_some() {
            bail!("Skill '{}' not found", options.skill_filter.unwrap());
        }
        bail!("No skills found in manifest");
    }

    // Transform and render
    let transformer = Transformer::new();
    let renderer = Renderer::new(options.format);

    for loaded in skills_to_generate {
        let skill = transformer.transform(loaded)?;
        generate_skill_files(&skill, &renderer, &options)?;
    }

    // Summary
    if options.dry_run {
        println!("\n[Dry run] No files written");
    } else {
        println!("\nClaude Agent Skills generated at: {}", output_dir.display());
        println!("Claude Code will automatically discover these skills.");
    }

    Ok(())
}

fn generate_skill_files(
    skill: &ClaudeSkill,
    renderer: &Renderer,
    options: &GenerateOptions,
) -> Result<()> {
    let skill_dir = options.output_dir.join(&skill.name);

    // Check if exists
    if skill_dir.exists() && !options.force {
        println!("⚠ Skipping {} (exists, use --force to overwrite)", skill.name);
        return Ok(());
    }

    // Render files
    let skill_md = renderer.render_skill_md(skill)?;
    let tools_md = if options.include_tools_md {
        Some(renderer.render_tools_md(skill)?)
    } else {
        None
    };

    if options.dry_run {
        println!("Would generate: {}/", skill_dir.display());
        println!("  - SKILL.md ({} bytes)", skill_md.len());
        if let Some(ref tools) = tools_md {
            println!("  - TOOLS.md ({} bytes)", tools.len());
        }
        return Ok(());
    }

    // Write files
    fs::create_dir_all(&skill_dir)?;

    fs::write(skill_dir.join("SKILL.md"), skill_md)?;
    println!("✓ Generated {}/SKILL.md", skill.name);

    if let Some(tools) = tools_md {
        fs::write(skill_dir.join("TOOLS.md"), tools)?;
        println!("✓ Generated {}/TOOLS.md", skill.name);
    }

    Ok(())
}
```

---

## Configuration

### Auto-Sync Configuration (Optional)

Add to `.skill-engine.toml`:

```toml
[claude-bridge]
# Enable automatic regeneration on skill changes
enabled = true

# Output directory (relative to project root)
output = ".claude/skills"

# Auto-sync when skills change
auto_sync = true

# Output format: "standard", "minimal", "verbose"
format = "standard"

# Include TOOLS.md reference files
include_tools_md = true

# Skills to exclude from generation
exclude = ["internal-skill", "deprecated-skill"]
```

### Per-Skill Overrides

```toml
[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"
description = "Kubernetes cluster management"

[skills.kubernetes.claude-bridge]
# Override when-to-use for this skill
when_to_use = [
    "Asks about pods, deployments, services, or Kubernetes resources",
    "Wants to check application status or container logs",
    "Needs to scale applications up or down",
    "Is troubleshooting container or cluster issues",
]

# Custom category overrides
[skills.kubernetes.claude-bridge.categories]
"Resource Viewing" = ["get", "describe", "logs", "top"]
"Resource Management" = ["create", "delete", "apply", "scale"]
"Node Operations" = ["cordon", "uncordon", "drain", "taint"]
```

---

## Output Structure

### Standard Output (Full - 100% Compliance)

```
~/.claude/skills/
├── kubernetes/
│   ├── SKILL.md          # Level 2: Instructions (< 5k tokens)
│   ├── TOOLS.md          # Level 3: Full reference
│   └── scripts/          # Executable scripts (Claude can run directly)
│       ├── get.sh            → skill run kubernetes get "$@"
│       ├── describe.sh       → skill run kubernetes describe "$@"
│       ├── logs.sh           → skill run kubernetes logs "$@"
│       ├── apply.sh          → skill run kubernetes apply "$@"
│       ├── delete.sh         → skill run kubernetes delete "$@"
│       ├── scale.sh          → skill run kubernetes scale "$@"
│       └── ...
├── docker/
│   ├── SKILL.md
│   ├── TOOLS.md
│   └── scripts/
│       ├── ps.sh
│       ├── images.sh
│       └── ...
└── ...
```

### MCP-Only Output (--no-scripts)

```
~/.claude/skills/
├── kubernetes/
│   ├── SKILL.md          # Instructions for MCP usage only
│   └── TOOLS.md          # Full reference
├── docker/
│   ├── SKILL.md
│   └── TOOLS.md
└── ...
```

### Project-Local Output

```
.claude/skills/
├── kubernetes/
│   ├── SKILL.md
│   ├── TOOLS.md
│   └── scripts/
│       └── ...
└── ...
```

### Script Structure

Each generated script is a thin wrapper around `skill run`:

```bash
#!/bin/bash
# kubernetes/scripts/get.sh
# Generated by Skill Engine - DO NOT EDIT
#
# Get Kubernetes resources (pods, services, deployments, etc.)
#
# Usage:
#   ./get.sh resource=pods
#   ./get.sh resource=pods namespace=kube-system
#   ./get.sh resource=deployments output=wide
#
# Parameters:
#   resource  - Resource type (required)
#   name      - Specific resource name
#   namespace - Kubernetes namespace (default: default)
#   selector  - Label selector
#   output    - Output format (wide, yaml, json)

set -euo pipefail
exec skill run kubernetes get "$@"
```

### Generated SKILL.md Size Targets

| Format | Target Size | Use Case |
|--------|-------------|----------|
| Minimal | < 2k tokens | Many skills, limited context |
| Standard | < 5k tokens | Balanced detail |
| Verbose | < 10k tokens | Comprehensive, few skills |

---

## Validation

### Name Validation Rules

```rust
fn validate_name(name: &str) -> Result<String> {
    // Max 64 characters
    if name.len() > 64 {
        return Err(ValidationError::NameTooLong(name.len()));
    }

    // Lowercase only
    if name != name.to_lowercase() {
        let fixed = name.to_lowercase();
        warn!("Name '{}' converted to lowercase: '{}'", name, fixed);
        return Ok(fixed);
    }

    // Only alphanumeric and hyphens
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(ValidationError::InvalidCharacters(name.into()));
    }

    // No reserved words
    let reserved = ["anthropic", "claude", "skill"];
    for word in reserved {
        if name.contains(word) {
            return Err(ValidationError::ReservedWord(word.into()));
        }
    }

    // Cannot start/end with hyphen
    if name.starts_with('-') || name.ends_with('-') {
        return Err(ValidationError::InvalidHyphenPosition);
    }

    Ok(name.to_string())
}
```

### Description Validation Rules

```rust
fn validate_description(desc: &str) -> Result<String> {
    // Non-empty
    if desc.trim().is_empty() {
        return Err(ValidationError::EmptyDescription);
    }

    // Max 1024 characters
    let truncated = if desc.len() > 1024 {
        warn!("Description truncated from {} to 1024 chars", desc.len());
        format!("{}...", &desc[..1021])
    } else {
        desc.to_string()
    };

    // No XML tags
    if truncated.contains('<') && truncated.contains('>') {
        return Err(ValidationError::ContainsXmlTags);
    }

    Ok(truncated)
}
```

---

## Edge Cases

### Handling

| Edge Case | Handling |
|-----------|----------|
| Skill name too long | Truncate with warning |
| Skill name has uppercase | Convert to lowercase with warning |
| Skill name has reserved word | Error, require rename |
| No tools in skill | Generate minimal SKILL.md with warning |
| Very long description | Truncate to 1024 chars |
| Existing SKILL.md in output | Skip unless `--force` |
| Manifest not found | Error with helpful message |
| Invalid skill source | Error, skip skill |
| WASM skill fails to load | Warn, skip skill |

### Error Messages

```
Error: Skill name 'my-Claude-skill' contains reserved word 'claude'
  → Rename skill in .skill-engine.toml

Error: Could not load tools for skill 'broken-skill'
  → Check skill source path and ensure it compiles

Warning: Skill 'long-name-...' truncated to 64 characters
  → Consider using a shorter name

Warning: Skipping 'kubernetes' (exists), use --force to overwrite
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("kubernetes").is_ok());
        assert!(validate_name("my-skill-123").is_ok());
    }

    #[test]
    fn test_validate_name_too_long() {
        let long_name = "a".repeat(65);
        assert!(matches!(
            validate_name(&long_name),
            Err(ValidationError::NameTooLong(_))
        ));
    }

    #[test]
    fn test_validate_name_reserved() {
        assert!(matches!(
            validate_name("my-claude-skill"),
            Err(ValidationError::ReservedWord(_))
        ));
    }

    #[test]
    fn test_categorize_tools() {
        let categorizer = ToolCategorizer::default();
        let tools = vec![
            Tool { name: "get".into(), .. },
            Tool { name: "create".into(), .. },
            Tool { name: "logs".into(), .. },
        ];

        let categories = categorizer.categorize(&tools);
        assert!(categories["Viewing"].iter().any(|t| t.name == "get"));
        assert!(categories["Management"].iter().any(|t| t.name == "create"));
    }

    #[test]
    fn test_render_skill_md() {
        let renderer = Renderer::new(OutputFormat::Standard);
        let skill = ClaudeSkill {
            name: "test-skill".into(),
            description: "Test description".into(),
            ..Default::default()
        };

        let output = renderer.render_skill_md(&skill).unwrap();
        assert!(output.contains("name: test-skill"));
        assert!(output.contains("mcp__skill-engine__execute"));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_generate_kubernetes_skill() {
    let temp_dir = tempdir().unwrap();

    let args = GenerateClaudeSkillsArgs {
        output: Some(temp_dir.path().to_path_buf()),
        skill: Some("kubernetes".into()),
        force: true,
        dry_run: false,
        ..Default::default()
    };

    run(args).await.unwrap();

    // Verify output
    let skill_md = temp_dir.path().join("kubernetes/SKILL.md");
    assert!(skill_md.exists());

    let content = fs::read_to_string(skill_md).unwrap();
    assert!(content.contains("name: kubernetes"));
    assert!(content.contains("mcp__skill-engine__execute"));
    assert!(content.contains("tool='get'"));
}
```

### End-to-End Test

```bash
#!/bin/bash
# test_claude_bridge.sh

set -e

# Setup
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Generate skills
skill generate-claude-skills --output "$TEMP_DIR" --skill kubernetes

# Verify structure
test -f "$TEMP_DIR/kubernetes/SKILL.md"
test -f "$TEMP_DIR/kubernetes/TOOLS.md"

# Verify content
grep -q "name: kubernetes" "$TEMP_DIR/kubernetes/SKILL.md"
grep -q "mcp__skill-engine__execute" "$TEMP_DIR/kubernetes/SKILL.md"

# Verify YAML frontmatter is valid
python3 -c "
import yaml
with open('$TEMP_DIR/kubernetes/SKILL.md') as f:
    content = f.read()
    frontmatter = content.split('---')[1]
    data = yaml.safe_load(frontmatter)
    assert 'name' in data
    assert 'description' in data
    assert len(data['name']) <= 64
    assert len(data['description']) <= 1024
"

echo "All tests passed!"
```

---

## Summary

| Component | Status | Priority |
|-----------|--------|----------|
| CLI Command | Design complete | P0 |
| Loader | Design complete | P0 |
| Transformer | Design complete | P0 |
| Renderer | Design complete | P0 |
| Validator | Design complete | P1 |
| Templates | Design complete | P0 |
| Auto-sync | Design complete | P2 |
| Tests | Strategy defined | P1 |

### Estimated Implementation Effort

| Phase | Effort |
|-------|--------|
| Core implementation (Loader, Transformer, Renderer) | 3-4 days |
| Templates and formatting | 1-2 days |
| Validation and error handling | 1 day |
| Testing | 1-2 days |
| Documentation | 1 day |
| **Total** | **7-10 days** |
