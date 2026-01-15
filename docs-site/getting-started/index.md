# Getting Started with Skill Engine

Welcome to Skill Engine! This guide will help you understand what Skill Engine is, how it works, and get you up and running in minutes.

## What is Skill Engine?

Skill Engine is a universal runtime for AI agent tools and skills. It provides:

- **Universal Tool Runtime**: Execute tools from any source (WASM, Docker, native binaries, MCP servers)
- **Claude Code Integration**: Native integration with Claude Code and other AI agents
- **Multiple Interfaces**: CLI, HTTP API, and MCP protocol support
- **Skill Marketplace**: Discover and install pre-built skills
- **Developer Framework**: Build and share your own skills

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      AI Agents Layer                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Claude Code  │  │   Gemini     │  │   OpenAI     │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
└─────────┼──────────────────┼──────────────────┼──────────────┘
          │                  │                  │
          └──────────────────┴──────────────────┘
                             │
┌────────────────────────────┼────────────────────────────────┐
│                    Skill Engine Core                         │
│                             │                                │
│  ┌──────────────────────────┴────────────────────────────┐  │
│  │              Interface Layer                          │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐           │  │
│  │  │   CLI    │  │ HTTP API │  │   MCP    │           │  │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘           │  │
│  └───────┼─────────────┼─────────────┼──────────────────┘  │
│          │             │             │                      │
│  ┌───────┴─────────────┴─────────────┴──────────────────┐  │
│  │            Skill Runtime Engine                       │  │
│  │  • Manifest parsing & validation                      │  │
│  │  • Tool discovery & execution                         │  │
│  │  • Parameter validation & type conversion             │  │
│  │  • Execution history & analytics                      │  │
│  └───────────────────────┬───────────────────────────────┘  │
└──────────────────────────┼──────────────────────────────────┘
                           │
┌──────────────────────────┼──────────────────────────────────┐
│                   Runtime Layer                              │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐            │
│  │    WASM    │  │   Docker   │  │   Native   │            │
│  │ Components │  │ Containers │  │  Binaries  │            │
│  └────────────┘  └────────────┘  └────────────┘            │
└──────────────────────────────────────────────────────────────┘
```

## Key Concepts

### Skills
A **skill** is a packaged collection of tools with a manifest (`.skill-engine.toml` or `SKILL.md`) that defines:
- Tool names and descriptions
- Parameter schemas
- Runtime requirements (WASM, Docker, native)
- Environment variables
- Documentation

### Tools
A **tool** is an individual function that agents can call. Each tool has:
- A unique name
- Input parameters with types
- Output format
- Description for AI agents

### Runtimes
Skill Engine supports multiple runtime environments:
- **WASM Component Model**: Sandboxed WebAssembly components
- **Docker**: Containerized tools (FFmpeg, ImageMagick, PostgreSQL, etc.)
- **Native**: Local binaries on your system

### Manifests
Skills are defined using manifests in two formats:
1. **TOML** (`.skill-engine.toml`): Traditional configuration format
2. **Markdown** (`SKILL.md`): Documentation-first format with YAML front matter

## Use Cases

### For AI Agent Developers
- Add specialized capabilities to your agents
- Access pre-built integrations (GitHub, Slack, Kubernetes, etc.)
- Ensure consistent tool execution across different AI models

### For Tool Developers
- Package your tools for AI consumption
- Reach users across multiple AI platforms
- Maintain a single source of truth for tool definitions

### For DevOps Teams
- Standardize infrastructure operations
- Provide safe, sandboxed access to production tools
- Enable AI-assisted troubleshooting and automation

## Quick Navigation

- **[Quick Start Guide](./quick-start.md)**: Get running in under 10 minutes
- **[Installation](./installation.md)**: Detailed installation instructions for all platforms
- **[Building Your First Skill](../guides/building-skills/)**: Create a custom skill
- **[Claude Code Integration](../guides/claude-code-integration.md)**: Use with Claude Code
- **[API Reference](../api/)**: Complete REST API documentation

## Next Steps

Ready to get started? Head to the **[Quick Start Guide](./quick-start.md)** to install Skill Engine and run your first skill in under 10 minutes.

Have questions? Check out the **[FAQ](../guides/faq.md)** or explore the **[Example Skills](../examples/)** for inspiration.
