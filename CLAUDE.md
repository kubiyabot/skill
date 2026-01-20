# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Skill is a universal skill runtime for AI agents that executes tools in a sandboxed WASM environment. It provides both CLI commands and MCP (Model Context Protocol) server capabilities for AI agent integration.

## Build & Development Commands

```bash
# Setup (one-time)
rustup target add wasm32-wasip1  # Required for WASM skill compilation

# Build
cargo build                    # Debug build
cargo build --release          # Release build (optimized for size)
cargo build --profile release-fast  # Fast compile with speed optimization

# Test
cargo test --workspace         # Run all tests
cargo test -p skill-cli        # Test specific crate
cargo test -p skill-cli test_name  # Run single test by name
cargo test -p skill-cli --lib -- claude_bridge  # Test claude_bridge module
cargo test -p skill-runtime -- --test-threads=1  # Serial execution for flaky tests
./tests/mcp_integration_tests.sh  # MCP protocol tests
./tests/run-all-tests.sh       # Full test suite

# Snapshot testing (uses insta crate)
cargo insta test               # Run snapshot tests
cargo insta review             # Review and accept new snapshots

# Benchmarks
cargo bench -p skill-cli       # Run benchmarks (claude_bridge_bench)

# Lint & Format
cargo fmt --all                # Format code
cargo clippy --all-targets -- -D warnings  # Lint

# Coverage
cargo tarpaulin --workspace --out Html  # Generate HTML coverage report

# Install locally
cargo install --path crates/skill-cli

# Documentation site (VitePress)
cd docs-site && npm install    # One-time setup
cd docs-site && npm run dev    # Local dev server
cd docs-site && npm run build  # Production build
```

## Architecture

### Crate Structure

```
crates/
├── skill-cli/        # Main binary - CLI commands, Claude bridge, auth
├── skill-runtime/    # Core execution engine - WASM, Docker, native runners
├── skill-mcp/        # MCP server implementation (stdio transport)
├── skill-http/       # HTTP streaming server for web clients
├── skill-web/        # Yew-based WebAssembly UI
└── skill-context/    # Shared types and context
```

### Key Components

**skill-runtime** (`crates/skill-runtime/src/`) is the core engine:
- `engine.rs` - `SkillEngine` orchestrates execution and search
- `manifest.rs` - `SkillManifest` for `.skill-engine.toml` parsing
- `executor.rs` - `SkillExecutor` WASM Component Model execution via Wasmtime
- `docker_runtime.rs` - Containerized skill execution with security policies
- `skill_md.rs` - SKILL.md parser for native command-based skills
- `local_loader.rs` / `git_loader.rs` - Skill installation from various sources
- `instance.rs` - Multi-instance management (dev/staging/prod)
- `credentials.rs` - Secure credential storage via keyring
- `audit.rs` - Execution audit logging

**skill-runtime search subsystem** (`crates/skill-runtime/src/search/`):
- `pipeline.rs` - `SearchPipeline` coordinates the full RAG pipeline
- `index_manager.rs` - Persistent index management with incremental updates
- `hybrid.rs` - Dense + BM25 fusion search
- `reranker.rs` - Cross-encoder reranking for precision
- `context.rs` - Token-aware context compression
- `query_processor.rs` - Intent classification and entity extraction

**skill-cli** (`crates/skill-cli/src/`):
- `commands/` - CLI command implementations
- `auth/` - Authentication providers (OAuth2, API keys, AWS)
- `config.rs` - CLI configuration management

**skill-cli commands** (aliases in parentheses):
- `install` - Install from local, HTTP, Git sources
- `run` - Execute skill tools with arguments
- `exec` - Pass-through execution (like `docker exec`)
- `list` (`ls`) - List installed skills
- `remove` (`rm`) - Uninstall skills
- `info` - Show skill details
- `find` - Semantic search for tools
- `search` - Registry search
- `config` - Configure skill credentials
- `serve` - Start MCP server (stdio or HTTP)
- `setup` - Configure search/RAG settings
- `enhance` - AI-generated examples
- `init` - Initialize new skill project
- `init-skill` - Generate SKILL.md template
- `claude setup/status/remove/generate` - Claude Code integration
- `auth login/status/logout/providers` - Authentication management
- `web` - Start embedded web interface
- `upgrade` - Self-update CLI

**skill-mcp** exposes three MCP tools:
- `execute` - Run any skill tool
- `list_skills` - Paginated skill listing
- `search_skills` - Semantic search

### Runtime Feature Flags

The runtime has optional features that enable additional capabilities:
```bash
cargo build -p skill-runtime --features hybrid-search,reranker,context-compression,qdrant
```

- `hybrid-search` - BM25 + vector fusion (tantivy)
- `reranker` - Cross-encoder reranking (fastembed)
- `context-compression` - Token-aware compression (tiktoken-rs)
- `qdrant` - Production vector database
- `ai-ingestion` - LLM providers for example generation
- `job-queue` - Async job scheduling (apalis)

## Key Patterns

### SKILL.md Format

Native skills use SKILL.md markdown files with YAML frontmatter:
```markdown
---
name: kubernetes
description: Kubernetes management
allowed-tools: Bash
---

### get
Get Kubernetes resources

**Parameters:**
- `resource` (required, string): Resource type
- `namespace` (optional, string): Namespace
```

Parameter parsing rules in `skill-runtime/src/skill_md.rs`.

### Manifest Configuration

`.skill-engine.toml` declares skills and instances:
```toml
[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"

[skills.kubernetes.instances.prod]
config = { context = "prod-cluster" }
```

Manifest loading in `skill-cli/src/commands/manifest.rs`.

### MCP Protocol

MCP server uses stdio for Claude Code integration. **Critical: all logs must go to stderr** (stdout is reserved for JSON-RPC). Use `tracing` macros which are configured to write to stderr. Entry point in `skill-mcp/src/server.rs`.

## Testing

- Unit tests: Same file with `#[cfg(test)]` modules
- Integration tests: `crates/*/tests/`
- Shell tests: `tests/` directory (MCP, security, e2e)
- Snapshot tests: Uses `insta` crate for YAML snapshots

Test directory structure:
```
tests/
├── mcp_integration_tests.sh     # MCP protocol tests (45 tests)
├── run-all-tests.sh             # Full test suite
├── claude_bridge/               # Claude bridge integration tests
├── e2e/                         # End-to-end tests
├── integration/                 # Runtime integration (docker, native, wasm)
├── security/                    # Security tests (injection, path traversal, etc.)
└── unit/                        # CLI command unit tests
```

Run specific test:
```bash
cargo test -p skill-cli test_name
cargo test -p skill-runtime -- --test-threads=1  # Serial execution
```

## Execution Model

Skills have three execution modes:
1. **WASM** - Sandboxed via Wasmtime Component Model, portable across platforms
2. **Native** - SKILL.md files that generate shell commands (kubectl, git, etc.) with command allowlists
3. **Docker** - Containerized execution for isolated environments

The `SkillEngine` in skill-runtime orchestrates which executor to use based on skill type.

### Data Flow

1. **Skill Installation**: `install.rs` → `local_loader.rs`/`git_loader.rs` → skills stored in `~/.skill-engine/skills/`
2. **Tool Execution**: `run.rs` → `SkillEngine::execute()` → `SkillExecutor` (WASM) or `skill_md.rs` (native)
3. **MCP Requests**: `skill-mcp/server.rs` → JSON-RPC handler → `SkillEngine` → response via stdout
4. **Search Pipeline**: Query → `embeddings.rs` → `vector_store.rs` → optional reranker → results

### Claude Bridge

The `claude_bridge/` module in skill-cli (`crates/skill-cli/src/commands/claude_bridge/`) generates Claude Code-compatible skill definitions:
- `loader.rs` - Loads SKILL.md files and extracts tool metadata
- `transformer.rs` - Converts skill tools to Claude-compatible format
- `renderer.rs` - Generates the final skill definition output
- `validator.rs` - Validates parameter types and constraints
- `script_gen.rs` - Generates shell scripts for skill execution
- `edge_cases.rs` - Handles special parameter types and edge cases
- `types.rs` - Shared type definitions

Run `skill claude setup` to auto-configure Claude Code integration.

### Authentication System

The auth module (`crates/skill-cli/src/auth/`) supports multiple providers:
- `providers/oauth2.rs` - OAuth2 Device Flow (GitHub, Google)
- `providers/api_key.rs` - API key storage (OpenAI, Anthropic)
- `providers/aws.rs` - AWS IAM credentials
- `token_store.rs` - Secure token persistence via keyring

## Task Master AI Instructions
**Import Task Master's development workflow commands and guidelines, treat as if import is in the main CLAUDE.md file.**
@./.taskmaster/CLAUDE.md
