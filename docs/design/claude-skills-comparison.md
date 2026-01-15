# Claude Agent Skills vs Skill Engine: A Comprehensive Comparison

This document explains the difference between Anthropic's official Claude Agent Skills and this project's Skill Engine, including protocol analysis and compatibility assessment.

## Table of Contents

- [Overview](#overview)
- [Claude Agent Skills (Anthropic Official)](#claude-agent-skills-anthropic-official)
- [Skill Engine (This Project)](#skill-engine-this-project)
- [Side-by-Side Comparison](#side-by-side-comparison)
- [Protocol Compliance Analysis](#protocol-compliance-analysis)
- [Use Cases](#use-cases)
- [Integration Possibilities](#integration-possibilities)

---

## Overview

Both systems share the name "Skills" but serve fundamentally different purposes:

| Aspect | Claude Agent Skills | Skill Engine |
|--------|---------------------|--------------|
| **Purpose** | Give Claude **instructions & expertise** | Give Claude **executable tools** |
| **Paradigm** | "Here's HOW to do X" | "Here's a TOOL to do X" |
| **Execution** | Claude interprets & writes code | Engine executes commands |

---

## Claude Agent Skills (Anthropic Official)

Claude Agent Skills are modular capabilities that extend Claude's functionality through instructions, workflows, and bundled resources.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        CLAUDE AGENT SKILLS PROTOCOL                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │  SKILL.md (Required)                                              │  │
│  │  ┌─────────────────────────────────────────────────────────────┐  │  │
│  │  │  ---                                                        │  │  │
│  │  │  name: pdf-processing           ◄── Level 1: Always loaded  │  │  │
│  │  │  description: Extract text...       (~100 tokens)           │  │  │
│  │  │  ---                                                        │  │  │
│  │  ├─────────────────────────────────────────────────────────────┤  │  │
│  │  │  # PDF Processing               ◄── Level 2: On-demand      │  │  │
│  │  │  ## Instructions                    (<5k tokens)            │  │  │
│  │  │  Use pdfplumber to extract...                               │  │  │
│  │  │  For forms, see [FORMS.md]                                  │  │  │
│  │  └─────────────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │  Bundled Resources (Optional)       ◄── Level 3: As needed        │  │
│  │  ├── FORMS.md                           (unlimited)               │  │
│  │  ├── REFERENCE.md                                                 │  │
│  │  └── scripts/                                                     │  │
│  │      └── fill_form.py              ◄── Code Claude can execute    │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Progressive Disclosure Model

Claude Agent Skills use a three-level loading strategy to optimize context usage:

| Level | When Loaded | Token Cost | Content |
|-------|-------------|------------|---------|
| **Level 1: Metadata** | Always (at startup) | ~100 tokens | `name` and `description` from YAML frontmatter |
| **Level 2: Instructions** | When Skill is triggered | <5k tokens | SKILL.md body with instructions and guidance |
| **Level 3: Resources** | As needed | Unlimited | Bundled files, scripts, reference materials |

### How Claude Uses Agent Skills

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           HOW CLAUDE USES IT                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   User: "Extract text from invoice.pdf"                                     │
│                          │                                                  │
│                          ▼                                                  │
│   ┌──────────────────────────────────────┐                                  │
│   │ System prompt includes:              │                                  │
│   │ "pdf-processing - Extract text..."   │  ◄── Metadata always present     │
│   └──────────────────────────────────────┘                                  │
│                          │                                                  │
│                          ▼ (matches!)                                       │
│   ┌──────────────────────────────────────┐                                  │
│   │ bash: cat ~/.claude/skills/          │                                  │
│   │       pdf-skill/SKILL.md             │  ◄── Claude READS instructions   │
│   └──────────────────────────────────────┘                                  │
│                          │                                                  │
│                          ▼                                                  │
│   ┌──────────────────────────────────────┐                                  │
│   │ Claude FOLLOWS the instructions:     │                                  │
│   │ import pdfplumber                    │  ◄── Claude writes/runs code     │
│   │ pdf.pages[0].extract_text()          │      based on skill guidance     │
│   └──────────────────────────────────────┘                                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### SKILL.md Structure (Official Protocol)

```yaml
---
name: your-skill-name          # Required: max 64 chars, lowercase, hyphens only
description: Brief description # Required: max 1024 chars, when to use it
---

# Your Skill Name

## Instructions
[Clear, step-by-step guidance for Claude to follow]

## Examples
[Concrete examples of using this Skill]
```

### Pre-built Agent Skills

Anthropic provides these ready-to-use skills:

- **PowerPoint (pptx)**: Create/edit presentations
- **Excel (xlsx)**: Spreadsheets, data analysis, charts
- **Word (docx)**: Document creation and editing
- **PDF (pdf)**: Generate formatted PDF documents

---

## Skill Engine (This Project)

Skill Engine is a runtime that provides Claude with executable tools through the MCP (Model Context Protocol).

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         SKILL ENGINE ARCHITECTURE                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │  skill.wit (Tool Interface Definition)                            │  │
│  │  ┌─────────────────────────────────────────────────────────────┐  │  │
│  │  │  world skill-basic {                                        │  │  │
│  │  │    export get-metadata: func() -> string;                   │  │  │
│  │  │    export get-tools: func() -> string;      ◄── Tool schema │  │  │
│  │  │    export execute-tool: func(name, args);   ◄── Execution   │  │  │
│  │  │  }                                                          │  │  │
│  │  └─────────────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │  SKILL.md (Documentation - Claude-compatible frontmatter)         │  │
│  │  ┌─────────────────────────────────────────────────────────────┐  │  │
│  │  │  ---                                                        │  │  │
│  │  │  name: kubernetes                                           │  │  │
│  │  │  description: Kubernetes cluster management...              │  │  │
│  │  │  allowed-tools: Bash, skill-run     ◄── Extension field     │  │  │
│  │  │  ---                                                        │  │  │
│  │  │  # Kubernetes Skill                                         │  │  │
│  │  │  ## Tools Provided                  ◄── Tool documentation  │  │  │
│  │  │  ### get, describe, logs...                                 │  │  │
│  │  └─────────────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │  .skill-engine.toml (Manifest)                                    │  │
│  │  ┌─────────────────────────────────────────────────────────────┐  │  │
│  │  │  [skills.kubernetes]                                        │  │  │
│  │  │  source = "./examples/native-skills/kubernetes-skill"       │  │  │
│  │  │  description = "Kubernetes cluster management"              │  │  │
│  │  │                                                             │  │  │
│  │  │  [skills.kubernetes.instances.default]                      │  │  │
│  │  │  config.cluster = "production"      ◄── Instance configs    │  │  │
│  │  └─────────────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Three Runtime Types

| Type | Description | Network | Security | Use Case |
|------|-------------|---------|----------|----------|
| **WASM** | WebAssembly components | Configurable | Sandboxed | Custom logic, API integrations |
| **Native** | CLI wrappers | Full access | Command allowlist | DevOps tools (kubectl, docker) |
| **Docker** | Container-based | Isolated | Container sandbox | Heavy processing (ffmpeg) |

### How Claude Uses Skill Engine

```
┌───────────────────────────────────────────────────────────────────────────┐
│                           HOW CLAUDE USES IT                              │
├───────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│   User: "Show me all pods in kube-system"                                 │
│                          │                                                │
│                          ▼                                                │
│   ┌──────────────────────────────────────┐                                │
│   │ MCP Server exposes tools:            │                                │
│   │ mcp__skill-engine__execute           │  ◄── Tools in system prompt    │
│   │ mcp__skill-engine__list_skills       │                                │
│   │ mcp__skill-engine__search_skills     │                                │
│   └──────────────────────────────────────┘                                │
│                          │                                                │
│                          ▼                                                │
│   ┌──────────────────────────────────────┐                                │
│   │ Claude CALLS the MCP tool:           │                                │
│   │ execute(                             │                                │
│   │   skill='kubernetes',                │  ◄── Claude invokes tool       │
│   │   tool='get',                        │                                │
│   │   args={resource:'pods',             │                                │
│   │         namespace:'kube-system'}     │                                │
│   │ )                                    │                                │
│   └──────────────────────────────────────┘                                │
│                          │                                                │
│                          ▼                                                │
│   ┌──────────────────────────────────────┐                                │
│   │ Skill Engine EXECUTES:               │                                │
│   │ kubectl get pods -n kube-system      │  ◄── Engine runs command       │
│   └──────────────────────────────────────┘                                │
│                          │                                                │
│                          ▼                                                │
│   ┌──────────────────────────────────────┐                                │
│   │ Returns output to Claude             │  ◄── Claude receives results   │
│   └──────────────────────────────────────┘                                │
│                                                                           │
└───────────────────────────────────────────────────────────────────────────┘
```

### Available Skills

| Category | Skills |
|----------|--------|
| **WASM** | simple, github, http, slack, prometheus, circleci, jira |
| **Native** | kubernetes, docker, git, terraform, postgres-native |
| **Docker** | ffmpeg, python-runner, node-runner, imagemagick, postgres, redis |

---

## Side-by-Side Comparison

```
┌────────────────────────────────────┬────────────────────────────────────────┐
│     CLAUDE AGENT SKILLS            │         SKILL ENGINE                   │
│     (Anthropic Official)           │         (This Project)                 │
├────────────────────────────────────┼────────────────────────────────────────┤
│                                    │                                        │
│  Purpose: INSTRUCT Claude          │  Purpose: EXECUTE for Claude           │
│           ▲                        │           ▲                            │
│           │                        │           │                            │
│  "Here's HOW to do X"              │  "Here's a TOOL to do X"               │
│                                    │                                        │
├────────────────────────────────────┼────────────────────────────────────────┤
│                                    │                                        │
│  SKILL.md                          │  skill.wit + SKILL.md                  │
│  ┌──────────────┐                  │  ┌──────────────┐                      │
│  │ Instructions │ ──► Claude       │  │ WIT Interface│ ──► Runtime          │
│  │ Workflows    │     reads &      │  │ Tool Schema  │     executes         │
│  │ Examples     │     follows      │  │ Validation   │     commands         │
│  └──────────────┘                  │  └──────────────┘                      │
│                                    │  ┌──────────────┐                      │
│                                    │  │ SKILL.md     │ ──► Documentation    │
│                                    │  │ (optional)   │     for humans       │
│                                    │  └──────────────┘                      │
│                                    │                                        │
├────────────────────────────────────┼────────────────────────────────────────┤
│                                    │                                        │
│  Discovery: Claude's VM filesystem │  Discovery: MCP protocol               │
│                                    │                                        │
│  ~/.claude/skills/                 │  mcp__skill-engine__list_skills        │
│       └── my-skill/                │  mcp__skill-engine__search_skills      │
│           └── SKILL.md             │                                        │
│                                    │                                        │
├────────────────────────────────────┼────────────────────────────────────────┤
│                                    │                                        │
│  Execution:                        │  Execution:                            │
│  ┌────────────────────────┐        │  ┌────────────────────────┐            │
│  │ Claude reads SKILL.md  │        │  │ Claude calls MCP tool  │            │
│  │ Claude writes code     │        │  │ Engine runs kubectl    │            │
│  │ Claude runs code       │        │  │ Returns output         │            │
│  └────────────────────────┘        │  └────────────────────────┘            │
│                                    │                                        │
├────────────────────────────────────┼────────────────────────────────────────┤
│                                    │                                        │
│  Runtimes: Claude's VM only        │  Runtimes: WASM, Native, Docker        │
│                                    │                                        │
│  ┌─────────┐                       │  ┌─────────┐ ┌────────┐ ┌───────────┐  │
│  │Claude VM│                       │  │  WASM   │ │ Native │ │   Docker  │  │
│  │(sandbox)│                       │  │(sandbox)│ │ (CLI)  │ │(container)│  │
│  └─────────┘                       │  └─────────┘ └────────┘ └───────────┘  │
│                                    │                                        │
├────────────────────────────────────┼────────────────────────────────────────┤
│                                    │                                        │
│  Token Usage:                      │  Token Usage:                          │
│  Progressive disclosure            │  Tool list in system prompt            │
│  (load only what's needed)         │  (all tools always visible)            │
│                                    │                                        │
└────────────────────────────────────┴────────────────────────────────────────┘
```

### Key Differences Table

| Aspect | Claude Agent Skills | Skill Engine |
|--------|---------------------|--------------|
| **Core file** | `SKILL.md` (instructions) | `skill.wit` (interface) + `SKILL.md` (docs) |
| **What Claude does** | Reads & follows instructions | Calls MCP tools |
| **Code execution** | Claude writes and runs code | Engine executes predefined commands |
| **Discovery** | Filesystem (`~/.claude/skills/`) | MCP protocol |
| **Token optimization** | Progressive disclosure (3 levels) | All tools listed upfront |
| **Sandboxing** | Claude's VM sandbox | WASM/Docker/command allowlist |
| **Extensibility** | Markdown + scripts | WIT interfaces + multiple runtimes |

---

## Protocol Compliance Analysis

### What Skill Engine Implements

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          PROTOCOL COMPLIANCE                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ✅ SKILL.md with YAML frontmatter                                           │
│                                                                             │
│     Skill Engine SKILL.md files include:                                    │
│     ---                                                                     │
│     name: kubernetes                      ◄── Matches Claude protocol       │
│     description: Kubernetes cluster...   ◄── Matches Claude protocol        │
│     allowed-tools: Bash, skill-run       ◄── Extension field                │
│     ---                                                                     │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ⚠️  DIFFERENT PURPOSE                                                       │
│                                                                             │
│     Claude Agent Skills:         Skill Engine:                              │
│     ┌─────────────────────┐      ┌─────────────────────┐                    │
│     │ Instructions for    │      │ Tool definitions &  │                    │
│     │ Claude to follow    │      │ execution runtime   │                    │
│     └─────────────────────┘      └─────────────────────┘                    │
│              │                            │                                 │
│              ▼                            ▼                                 │
│     Claude reads SKILL.md        Claude calls MCP tools                     │
│     and generates code           Engine executes commands                   │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│  ❌ NOT DIRECTLY COMPATIBLE                                                  │
│                                                                             │
│     Discovery mechanism:                                                    │
│     - Claude Skills: filesystem (~/.claude/skills/)                         │
│     - Skill Engine: MCP protocol (mcp__skill-engine__*)                     │
│                                                                             │
│     Loading model:                                                          │
│     - Claude Skills: progressive disclosure (3 levels)                      │
│     - Skill Engine: all tools exposed via MCP, docs on-demand               │
│                                                                             │
│     Execution model:                                                        │
│     - Claude Skills: Claude interprets instructions, writes code            │
│     - Skill Engine: Engine executes predefined tool commands                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Compliance Summary

| Protocol Element | Claude Agent Skills | Skill Engine | Compatible? |
|------------------|---------------------|--------------|-------------|
| YAML frontmatter | Required | Implemented | ✅ Format matches |
| `name` field | Required | Present | ✅ |
| `description` field | Required | Present | ✅ |
| `allowed-tools` field | Not in spec | Extension | ⚠️ Custom field |
| Filesystem discovery | `~/.claude/skills/` | Not used | ❌ |
| MCP discovery | Not used | Primary method | ❌ |
| Progressive disclosure | 3 levels | Not implemented | ❌ |
| Instructions body | Claude reads & follows | Documentation only | ⚠️ Different purpose |

---

## Use Cases

### When to Use Claude Agent Skills

- Teaching Claude domain-specific workflows
- Providing best practices and guidelines
- Bundling reference documentation
- Including utility scripts Claude can run
- Creating reusable instruction sets

**Example**: A "code-review" skill that teaches Claude your team's review standards.

### When to Use Skill Engine

- Executing external CLI tools (kubectl, docker, terraform)
- Calling external APIs (GitHub, Slack, Jira)
- Running sandboxed computations (WASM, Docker)
- Providing pre-built, validated tool interfaces
- Enforcing security through command allowlists

**Example**: A "kubernetes" skill that lets Claude run kubectl commands safely.

---

## Integration Possibilities

The two systems are **complementary, not competing**. They can work together:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      COMPLEMENTARY ARCHITECTURE                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   User: "Deploy my app to Kubernetes following our team's best practices"   │
│                          │                                                  │
│                          ▼                                                  │
│   ┌──────────────────────────────────────────────────────────────────┐      │
│   │  Claude Agent Skill: deployment-practices                        │      │
│   │  ┌────────────────────────────────────────────────────────────┐  │      │
│   │  │ SKILL.md:                                                  │  │      │
│   │  │ # Deployment Best Practices                                │  │      │
│   │  │ 1. Always check current state first                        │  │      │
│   │  │ 2. Use rolling updates                                     │  │      │
│   │  │ 3. Verify health after deployment                          │  │      │
│   │  │ 4. Use the kubernetes skill for all kubectl operations     │  │      │
│   │  └────────────────────────────────────────────────────────────┘  │      │
│   └──────────────────────────────────────────────────────────────────┘      │
│                          │                                                  │
│                          │ Claude follows instructions                      │
│                          ▼                                                  │
│   ┌──────────────────────────────────────────────────────────────────┐      │
│   │  Skill Engine: kubernetes                                        │      │
│   │  ┌────────────────────────────────────────────────────────────┐  │      │
│   │  │ execute(skill='kubernetes', tool='get', args={...})        │  │      │
│   │  │ execute(skill='kubernetes', tool='apply', args={...})      │  │      │
│   │  │ execute(skill='kubernetes', tool='rollout', args={...})    │  │      │
│   │  └────────────────────────────────────────────────────────────┘  │      │
│   └──────────────────────────────────────────────────────────────────┘      │
│                          │                                                  │
│                          ▼                                                  │
│   ┌──────────────────────────────────────────────────────────────────┐      │
│   │  Result: Best practices applied + actual kubectl execution       │      │
│   └──────────────────────────────────────────────────────────────────┘      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Hybrid Approach Benefits

| Layer | System | Provides |
|-------|--------|----------|
| **Knowledge** | Claude Agent Skills | Best practices, workflows, guidelines |
| **Execution** | Skill Engine | Safe, validated tool execution |

---

## References

- [Claude Agent Skills Documentation](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview)
- [Claude Agent Skills Best Practices](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices)
- [MCP (Model Context Protocol)](https://modelcontextprotocol.io/)
- [WebAssembly Interface Types (WIT)](https://component-model.bytecodealliance.org/design/wit.html)
