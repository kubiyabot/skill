# PRD: RAG End-to-End Integration & Model Management

## Overview

Complete the RAG (Retrieval-Augmented Generation) pipeline integration to enable true end-to-end functionality out of the box. This includes creating a unified SearchPipeline, interactive setup wizards, model management for Ollama/HuggingFace, MCP search tool, and CLI/HTTP interfaces.

## Problem Statement

The current RAG implementation has well-designed components (VectorStore, EmbeddingProviders, BM25, Reranker, etc.) but they are not wired together into a cohesive system:

1. **No unified orchestrator**: `find.rs` duplicates logic instead of using runtime abstractions
2. **No MCP search tool**: Claude cannot invoke semantic search through MCP protocol
3. **No setup wizard**: Users must manually configure embedding providers and understand the system
4. **No model management**: No way to pull Ollama models, download FastEmbed models, or verify HuggingFace availability
5. **No HTTP API**: REST endpoints for search are not implemented

## Goals

1. **Zero-config startup**: RAG should work immediately with sensible defaults (FastEmbed local embeddings)
2. **Interactive setup**: Wizard to configure providers, download models, test connectivity
3. **Model management**: Pull/download/verify embedding models from Ollama, HuggingFace, FastEmbed
4. **Unified pipeline**: Single SearchPipeline that orchestrates all RAG components
5. **MCP integration**: Expose semantic search to Claude and other MCP clients
6. **HTTP API**: REST endpoints for programmatic access

## Non-Goals

- Fine-tuning or training custom embedding models
- Supporting every possible embedding provider (focus on Ollama, FastEmbed, OpenAI)
- Real-time index updates (batch sync is sufficient)

---

## Requirements

### 1. SearchPipeline Orchestrator

Create a high-level `SearchPipeline` struct in `skill-runtime` that:

- Orchestrates VectorStore, EmbeddingProvider, BM25Index, Reranker, ContextCompressor
- Provides a simple `search(query: &str) -> Vec<SearchResult>` interface
- Handles initialization from SearchConfig
- Manages component lifecycle (lazy loading, health checks)
- Supports both sync and async operations

**File**: `crates/skill-runtime/src/search/pipeline.rs`

```rust
pub struct SearchPipeline {
    config: SearchConfig,
    vector_store: Box<dyn VectorStore>,
    embedding_provider: Box<dyn EmbeddingProvider>,
    bm25_index: Option<BM25Index>,      // if hybrid-search enabled
    reranker: Option<Box<dyn Reranker>>, // if reranker enabled
    compressor: Option<ContextCompressor>, // if context-compression enabled
    query_processor: QueryProcessor,
}

impl SearchPipeline {
    pub async fn from_config(config: SearchConfig) -> Result<Self>;
    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>>;
    pub async fn index_skills(&mut self, skills: Vec<SkillMetadata>) -> Result<IndexStats>;
    pub async fn health_check(&self) -> Result<HealthStatus>;
}
```

### 2. Refactor find.rs to Use SearchPipeline

The `skill find` command should become a thin wrapper around SearchPipeline:

- Remove direct Rig library usage
- Initialize SearchPipeline from config
- Delegate search to pipeline
- Keep formatting/output logic in CLI

### 3. MCP Search Tool

Add `search_skills` tool to MCP server:

**File**: `crates/skill-mcp/src/server.rs`

```json
{
  "name": "search_skills",
  "description": "Semantic search for tools across all installed skills",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": { "type": "string", "description": "Natural language query" },
      "top_k": { "type": "integer", "default": 5 }
    },
    "required": ["query"]
  }
}
```

### 4. Interactive Setup Wizard

Add `skill setup` command with sub-commands:

```bash
skill setup              # Full interactive wizard
skill setup search       # Configure RAG/search settings
skill setup embedding    # Configure embedding provider
skill setup check        # Verify all dependencies
```

**Wizard Flow**:

1. Detect environment (Ollama running? OpenAI key set? Network available?)
2. Present provider options based on detection
3. Download/pull required models
4. Test embedding generation
5. Generate `skill.toml` configuration
6. Index installed skills

### 5. Model Management Commands

```bash
# Ollama integration
skill models ollama list          # List available Ollama models
skill models ollama pull <model>  # Pull embedding model (e.g., nomic-embed-text)
skill models ollama status        # Check Ollama service status

# FastEmbed (local)
skill models fastembed list       # List available FastEmbed models
skill models fastembed download <model>  # Pre-download model
skill models fastembed cache      # Show cache location and size

# General
skill models list                 # List all configured models
skill models test                 # Test current embedding provider
skill models benchmark            # Compare latency across providers
```

### 6. HTTP API Endpoints

Implement in `crates/skill-http/src/handlers.rs`:

```
POST /api/search
  Body: { "query": "...", "top_k": 5 }
  Response: { "results": [...] }

GET /api/search/health
  Response: { "status": "ok", "provider": "fastembed", "indexed": 150 }

POST /api/index/sync
  Response: { "added": 5, "updated": 2, "removed": 1 }

GET /api/models
  Response: { "current": "all-minilm", "available": [...] }
```

### 7. Configuration Management

```bash
skill config search              # View current search config
skill config search --edit       # Open in editor
skill config search --reset      # Reset to defaults
skill config search provider=ollama model=nomic-embed-text  # Inline set
```

### 8. Auto-Setup on First Run

When `skill find` is run for the first time:

1. Check if SearchConfig exists
2. If not, run minimal auto-setup:
   - Use FastEmbed (no external dependencies)
   - Download default model (all-MiniLM-L6-v2)
   - Create default config
   - Index installed skills
3. Show progress bar for model download
4. Cache everything for subsequent runs

---

## Technical Specifications

### Dependencies

| Provider | Required Dependencies | Auto-Detection |
|----------|----------------------|----------------|
| FastEmbed | None (downloads on first use) | Always available |
| Ollama | Ollama service running | Check `localhost:11434` |
| OpenAI | `OPENAI_API_KEY` env var | Check env var |
| HuggingFace | `HF_TOKEN` for private models | Check env var |

### Model Download Locations

| Provider | Cache Location | Approximate Size |
|----------|---------------|------------------|
| FastEmbed | `~/.fastembed_cache/` | 50-400MB per model |
| Ollama | Managed by Ollama | 100-500MB per model |
| OpenAI | N/A (API-based) | N/A |

### Feature Flag Matrix

| Feature | Default | Dependency |
|---------|---------|------------|
| Basic search | Yes | None |
| Hybrid search | No | `tantivy` |
| Reranking | No | `fastembed` |
| Context compression | No | `tiktoken-rs` |
| Qdrant backend | No | `qdrant-client` |

---

## User Stories

### Story 1: First-Time User
> As a new user, I want `skill find "kubernetes"` to just work without configuration.

**Acceptance Criteria**:
- First run auto-downloads FastEmbed model with progress bar
- Creates default config automatically
- Indexes installed skills
- Returns results in <5 seconds (after model download)

### Story 2: Ollama User
> As a user with Ollama installed, I want to use my local Ollama embeddings.

**Acceptance Criteria**:
- `skill setup` detects running Ollama
- Offers to use Ollama as embedding provider
- Pulls required model if not present
- Tests embedding generation before completing

### Story 3: Claude Code Integration
> As a Claude Code user, I want Claude to find relevant tools through MCP.

**Acceptance Criteria**:
- `search_skills` MCP tool available when `skill serve` running
- Claude can invoke semantic search
- Results include tool names, descriptions, and usage examples

### Story 4: CI/CD Integration
> As a DevOps engineer, I want to programmatically search skills via HTTP.

**Acceptance Criteria**:
- `skill serve --http` exposes REST endpoints
- `/api/search` returns JSON results
- Health check endpoint for monitoring

---

## Implementation Phases

### Phase 1: Core Pipeline (Priority: P0)
1. Create SearchPipeline orchestrator
2. Refactor find.rs to use SearchPipeline
3. Implement auto-setup on first run
4. Add progress bar for model downloads

### Phase 2: MCP Integration (Priority: P0)
1. Add search_skills tool to MCP server
2. Test with Claude Code
3. Handle streaming results

### Phase 3: Setup Wizard (Priority: P1)
1. Implement `skill setup` command
2. Add provider detection logic
3. Create interactive prompts
4. Generate configuration file

### Phase 4: Model Management (Priority: P1)
1. Implement `skill models` command group
2. Add Ollama integration (status, pull, list)
3. Add FastEmbed cache management
4. Add model benchmarking

### Phase 5: HTTP API (Priority: P2)
1. Implement search handler
2. Add health check endpoint
3. Add index sync endpoint
4. Document API with examples

### Phase 6: Advanced Features (Priority: P2)
1. HuggingFace model support
2. Model auto-update checking
3. Embedding cache sharing across projects
4. Custom model registration

---

## Success Metrics

| Metric | Target |
|--------|--------|
| First-run to search result | <30 seconds (including model download) |
| Subsequent search latency | <500ms |
| Setup wizard completion rate | >90% |
| MCP search tool adoption | Used in >50% of Claude sessions |

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Large model downloads on first run | Show progress bar, offer smaller models |
| Ollama not installed | Graceful fallback to FastEmbed |
| Network issues during setup | Offline mode with pre-downloaded models |
| Breaking changes in model APIs | Version lock dependencies |

---

## Open Questions

1. Should we support custom embedding models from HuggingFace Hub?
2. Should the HTTP API require authentication?
3. Should we provide a Docker image with pre-downloaded models?
4. Should index be stored globally or per-project?

---

## Appendix: Current Implementation Status

### Completed (Tasks 41-50)
- VectorStore trait + InMemory/Qdrant backends
- Embedding providers (FastEmbed, OpenAI, Ollama)
- BM25 index with Tantivy
- Hybrid search with RRF
- Cross-encoder reranking
- Context compression
- Query understanding
- Persistent index manager
- Configuration schema

### Missing (This PRD)
- SearchPipeline orchestrator
- find.rs refactor to use runtime
- MCP search_skills tool
- Setup wizard
- Model management CLI
- HTTP API handlers
- Auto-setup on first run
