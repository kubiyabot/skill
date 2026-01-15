---
layout: home

hero:
  name: "Skill"
  text: "Give your AI agent superpowers through the terminal"
  tagline: A universal skill runtime that runs anywhere. No servers, no context bloat, just tools that work.
  actions:
    - theme: brand
      text: Get Started
      link: /getting-started/quick-start
    - theme: alt
      text: View on GitHub
      link: https://github.com/kubiyabot/skill

features:
  - icon: 🚀
    title: Single Binary
    details: No servers required. Install once, works everywhere. Runs in CI/CD, terminal agents, or IDEs.
  - icon: 🔐
    title: WASM Sandbox
    details: Capability-based security. Skills can't escape their boundaries. Declare permissions upfront.
  - icon: 🔍
    title: Local Semantic Search
    details: Find tools using natural language. Zero context cost. Works offline. No API keys needed.
  - icon: 🌐
    title: Universal Runtime
    details: Same skills work with Claude Code, MCP protocol, or CLI commands. Write once, run everywhere.
---

## The Problem

**AI agents need to do things in the real world.** Query databases. Deploy code. Manage infrastructure. Call APIs.

Today, you have two choices:

1. **Stuff tool docs into prompts** → Context bloat, hallucinated flags, model confusion
2. **Run MCP servers** → Complex setup, always-on processes, protocol overhead

**Skill is a third way.** A universal skill runtime that runs anywhere, stays secure, finds tools intelligently, and works with any agent.

## Quick Start

```bash
# Install
curl -fsSL https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/install.sh | sh

# Discover tools with natural language
skill find "manage kubernetes pods"

# Execute in sandboxed WASM
skill run kubernetes:get --resource pods --all-namespaces
```

[Read the full installation guide →](/getting-started/quick-start)

## Two Ways to Use

### CLI Mode (Default)

Perfect for Claude Code, Aider, or any terminal-based agent:

```bash
skill find "<what you want to do>"   # Discover tools
skill run <skill>:<tool> [args]      # Execute tools
skill list                           # See what's installed
```

The agent uses shell commands. No SDK, no protocol, no complexity.

### MCP Server Mode

For agents that speak Model Context Protocol:

```bash
# Start the MCP server
skill serve
```

Add to your Claude Desktop or IDE config:

```json
{
  "mcpServers": {
    "skill": {
      "command": "skill",
      "args": ["serve"]
    }
  }
}
```

The same skills work in both modes. Write once, use everywhere.

[Learn more about MCP integration →](/guides/mcp)

## Why Skill?

<div class="comparison-grid">

<div class="comparison-card">

### 📝 Traditional Approach
**Stuff docs into prompts**

- ❌ Context window bloat
- ❌ Hallucinated command flags
- ❌ Model confusion with complex tools
- ❌ Slow, expensive API calls

</div>

<div class="comparison-card">

### 🔌 MCP Servers
**Always-on processes**

- ❌ Complex setup per tool
- ❌ Memory overhead for servers
- ❌ Protocol translation costs
- ❌ Hard to debug failures

</div>

<div class="comparison-card highlight">

### ⚡ Skill Runtime
**Best of both worlds**

- ✅ Zero context cost (local search)
- ✅ Single binary, no servers
- ✅ WASM sandbox security
- ✅ <50ms tool discovery

</div>

</div>

## Performance

| Metric | Value |
|--------|-------|
| Cold start | ~100ms (includes AOT compilation) |
| Warm start | <10ms (cached) |
| Vector search | <50ms (local FastEmbed) |
| MCP tool call | <100ms typical |

All operations run locally. No API calls. Works offline.

## Security Model

Skills declare capabilities at install time and can't escape their boundaries:

```yaml
capabilities:
  network:
    - "*.amazonaws.com"
  filesystem:
    - read: "${args.file}"
  allowed-tools:
    - kubectl
    - helm
```

### What Skills Cannot Do

- ❌ Read arbitrary files (WASI filesystem not mounted)
- ❌ Access unrequested network (WASI sockets allowlist)
- ❌ Run arbitrary commands (command allowlist)
- ❌ Persist state (memory cleared after execution)

## Who Uses Skill?

<div class="persona-cards">

<div class="persona-card">

### 🤖 AI Agent Developers
Use Skill with Claude Code, Aider, Cursor, or custom agents. No integration code needed—just shell commands.

[Quick start guide →](/getting-started/quick-start)

</div>

<div class="persona-card">

### 💻 DevOps Teams
Standardize tool access across your organization. Audit logging and RBAC built-in.

[Enterprise features →](/guides/developing-skills)

</div>

<div class="persona-card">

### 🛠️ Tool Developers
Write your skill once in Rust, JavaScript, or Python. Runs everywhere without modification.

[Build a skill →](/guides/developing-skills)

</div>

</div>

## Get Started

<div class="cta-primary">

### Install Skill

Run your first skill in under 5 minutes.

<a href="/getting-started/quick-start" class="button-primary">Get Started →</a>

</div>

**Or explore:**
[Build a Skill](/guides/developing-skills) · [API Reference](/api/) · [Browse Examples](/examples/)

<style>
.comparison-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 1.5rem;
  margin: 3rem 0;
}

.comparison-card {
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 12px;
  padding: 2rem;
}

.comparison-card.highlight {
  border: 2px solid var(--vp-c-accent);
  background: linear-gradient(135deg, var(--vp-c-accent-soft) 0%, var(--vp-c-bg-soft) 100%);
}

.comparison-card h3 {
  margin-top: 0;
  margin-bottom: 0.5rem;
  font-size: 1.125rem;
}

.comparison-card h3 + p {
  font-size: 0.875rem;
  color: var(--vp-c-text-3);
  margin-bottom: 1rem;
}

.comparison-card ul {
  list-style: none;
  padding: 0;
  margin: 0;
}

.comparison-card li {
  padding: 0.375rem 0;
  font-size: 0.9375rem;
  line-height: 1.6;
}
</style>
