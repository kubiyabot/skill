# GitHub Pages Documentation Site PRD
# Skill Engine Project

**Status**: Draft  
**Created**: 2026-01-05  
**Author**: Claude Code Agent  
**Target Release**: Q1 2026  

---

## 1. Problem Statement

### Current Pain Points

**For Users (Developers & Integrators)**
- **Discovery friction**: Potential users can't quickly understand what Skill Engine does or why it matters
- **Fragmented documentation**: 5,200+ lines across 15+ markdown files with no clear navigation
- **Missing API reference**: No auto-generated docs for REST API, MCP protocol, or Rust internals
- **No interactive learning**: Users must install and experiment blindly without try-before-install experience
- **Integration confusion**: Unclear how to integrate with Claude Code, Cursor, custom agents
- **Poor SEO/discoverability**: GitHub README-only presence limits organic discovery

**For Maintainers**
- **Documentation drift**: No single source of truth, frequent updates needed across multiple files
- **Onboarding burden**: New contributors struggle to understand architecture without guided tour
- **Support overhead**: Common questions repeat due to missing comprehensive guides

**For the Project**
- **Adoption barrier**: Sophisticated RAG pipeline and hybrid search features under-documented
- **Competitive disadvantage**: Similar tools (e.g., E2B, Modal, Dagger) have polished documentation sites
- **Community growth**: No central hub for showcasing examples, use cases, or community contributions

### Quantifiable Impact

- **Current**: Average 15-20 minutes to understand project scope from README alone
- **Current**: Zero interactive demos or API explorers
- **Current**: Inline Rust docs coverage < 30% based on grep analysis
- **Target**: < 5 minutes to understand value proposition and get started
- **Target**: 90%+ API surface documented with examples
- **Target**: Interactive playground for all 20+ skill tools

---

## 2. Goals & Success Metrics

### Primary Goals

1. **Reduce Time-to-Value**: Get developers from discovery to first skill execution in < 10 minutes
2. **Comprehensive Reference**: Document 100% of public APIs (CLI, REST, MCP, Rust)
3. **Enable Self-Service**: Answer top 20 support questions through docs alone
4. **Showcase Innovation**: Highlight unique features (RAG pipeline, hybrid search, multi-runtime)
5. **Drive Adoption**: Increase GitHub stars by 300% within 3 months of launch

### Success Metrics

| Metric | Baseline | 3-Month Target | 6-Month Target |
|--------|----------|----------------|----------------|
| **GitHub Stars** | 150 | 450 | 800 |
| **Weekly Site Visits** | N/A | 1,000 | 2,500 |
| **Avg. Time on Site** | N/A | 8 min | 12 min |
| **Docs Coverage** | 30% | 85% | 95% |
| **Interactive Playground Usage** | 0 | 300/week | 800/week |
| **Search Quality (MRR)** | N/A | 0.75 | 0.85 |
| **Support Ticket Reduction** | Baseline | -40% | -60% |

### Key Performance Indicators (KPIs)

- **Documentation Completeness**: % of public API surface with docs + examples
- **User Engagement**: Bounce rate, pages per session, interactive feature usage
- **Developer Success**: % completing "Quick Start" within 10 minutes
- **Search Effectiveness**: Click-through rate on search results, zero-result queries
- **Community Contribution**: PRs to docs repo, user-submitted examples

---

## 3. Site Architecture

### Information Architecture

```
skill-engine-docs/
├── Home (/)
│   ├── Hero with live code demo
│   ├── Value proposition (3 pillars)
│   ├── Feature highlights
│   └── Quick start CTA
│
├── Getting Started (/guide/)
│   ├── Installation (/guide/installation)
│   ├── Quick Start (/guide/quick-start)
│   ├── Core Concepts (/guide/concepts)
│   └── First Skill (/guide/first-skill)
│
├── User Guides (/guides/)
│   ├── Creating Skills
│   │   ├── SKILL.md Format (/guides/skills/skill-md)
│   │   ├── WASM Components (/guides/skills/wasm)
│   │   ├── Docker Skills (/guides/skills/docker)
│   │   └── Best Practices (/guides/skills/best-practices)
│   ├── Integration
│   │   ├── Claude Code (/guides/integration/claude-code)
│   │   ├── Claude Desktop (/guides/integration/claude-desktop)
│   │   ├── Cursor (/guides/integration/cursor)
│   │   └── Custom Agents (/guides/integration/custom)
│   ├── RAG & Search
│   │   ├── Overview (/guides/rag/overview)
│   │   ├── Configuration (/guides/rag/configuration)
│   │   ├── Embedding Providers (/guides/rag/embeddings)
│   │   ├── Hybrid Search (/guides/rag/hybrid-search)
│   │   └── Reranking (/guides/rag/reranking)
│   └── Advanced
│       ├── Multi-Instance Config (/guides/advanced/instances)
│       ├── Security Model (/guides/advanced/security)
│       ├── Performance Tuning (/guides/advanced/performance)
│       └── Deployment (/guides/advanced/deployment)
│
├── API Reference (/api/)
│   ├── CLI Commands (/api/cli)
│   ├── REST API (/api/rest)
│   │   ├── OpenAPI Spec (interactive)
│   │   └── Endpoints by category
│   ├── MCP Protocol (/api/mcp)
│   │   ├── Tools reference
│   │   └── Message schemas
│   └── Rust Internals (/api/rust)
│       ├── skill-runtime
│       ├── skill-cli
│       ├── skill-http
│       ├── skill-mcp
│       ├── skill-web
│       └── skill-context
│
├── Interactive Tools (/playground/)
│   ├── Live Playground (/playground/live)
│   │   ├── WASM skill executor
│   │   ├── Pre-loaded examples
│   │   └── Share/export functionality
│   ├── API Explorer (/playground/api)
│   │   ├── REST endpoint tester
│   │   └── MCP tool simulator
│   ├── Config Generator (/playground/config)
│   │   ├── .skill-engine.toml builder
│   │   └── Visual schema editor
│   └── Search Tester (/playground/search)
│       ├── RAG pipeline visualizer
│       └── Query debugging tools
│
├── Examples (/examples/)
│   ├── By Runtime
│   │   ├── WASM Examples (/examples/wasm)
│   │   ├── Native Examples (/examples/native)
│   │   └── Docker Examples (/examples/docker)
│   ├── By Use Case
│   │   ├── DevOps & Infrastructure
│   │   ├── Data Processing
│   │   ├── API Integration
│   │   └── AI/ML Workflows
│   └── Community Gallery
│
├── Architecture (/architecture/)
│   ├── System Overview (/architecture/overview)
│   ├── Runtime Architecture (/architecture/runtime)
│   ├── RAG Pipeline Deep Dive (/architecture/rag)
│   ├── Security Model (/architecture/security)
│   └── Design Decisions (/architecture/decisions)
│
└── Resources (/resources/)
    ├── FAQ (/resources/faq)
    ├── Troubleshooting (/resources/troubleshooting)
    ├── Contributing Guide (/resources/contributing)
    ├── Roadmap (/resources/roadmap)
    ├── Changelog (/resources/changelog)
    └── Community (/resources/community)
```

### URL Structure

| Pattern | Example | Description |
|---------|---------|-------------|
| `/` | `/` | Home page |
| `/guide/*` | `/guide/installation` | Getting started tutorials |
| `/guides/*` | `/guides/skills/wasm` | In-depth user guides |
| `/api/*` | `/api/rest` | API reference docs |
| `/playground/*` | `/playground/live` | Interactive tools |
| `/examples/*` | `/examples/wasm` | Code examples & gallery |
| `/architecture/*` | `/architecture/rag` | Architecture deep dives |
| `/resources/*` | `/resources/faq` | Auxiliary resources |

### Navigation Structure

**Primary Nav (Top Bar)**
- Guide (Getting Started dropdown)
- User Guides (Mega-menu by category)
- API Reference
- Playground
- Examples
- Architecture
- [GitHub Icon]

**Secondary Nav (Sidebar, context-aware)**
- Current section's table of contents
- Related pages
- Quick links to playground

**Footer**
- Documentation sections
- Community links (GitHub, Discord, Twitter)
- Legal (License)
- Version selector

---

## 4. Content Strategy

### Documentation Types (Divio Framework)

| Type | Purpose | Tone | Examples |
|------|---------|------|----------|
| **Tutorials** | Learning-oriented, hands-on | Encouraging | Quick Start, First Skill |
| **How-To Guides** | Problem-oriented, practical | Direct | Integration guides, RAG config |
| **Reference** | Information-oriented, factual | Precise | CLI commands, API endpoints |
| **Explanation** | Understanding-oriented, conceptual | Thoughtful | Architecture, design decisions |

### Content Migration Plan

#### Phase 1: Foundation (Week 1-2)

**Migrate & Enhance**
1. `README.md` → `/` (home page)
   - Extract hero section
   - Condense features into 3 pillars
   - Add interactive demo embed

2. `docs/skill-development.md` → `/guides/skills/`
   - Split into SKILL.md, WASM, Docker guides
   - Add validation checklist
   - Include Claude Bridge integration

3. `docs/rag-search.md` → `/guides/rag/`
   - Split into overview + configuration + components
   - Add performance tuning section
   - Include troubleshooting flowchart

4. `docs/web-interface.md` → `/guides/web-ui/`
   - Update with latest screenshots
   - Add service management walkthrough
   - Include deployment guide

#### Phase 2: New Content (Week 3-4)

**Create from Scratch**
1. **Getting Started** (NEW)
   - 5-minute quick start
   - Core concepts explainer
   - Common pitfalls & solutions

2. **Integration Guides** (NEW)
   - Claude Code setup (expand from QUICK_START_CLAUDE_CODE.md)
   - Claude Desktop configuration
   - Cursor integration
   - Custom agent template

3. **Architecture Guides** (NEW)
   - System architecture diagram
   - Component interaction flows
   - Runtime lifecycle
   - Security model deep dive

4. **Troubleshooting & FAQ** (NEW)
   - Top 20 support questions
   - Error message decoder
   - Performance debugging
   - Common misconfigurations

#### Phase 3: API Documentation (Week 5-6)

**Auto-Generated + Hand-Written**
1. **CLI Reference** (Auto-generate from clap)
   - Command tree
   - Flag reference
   - Exit codes
   - Examples for each command

2. **REST API** (Auto-generate from OpenAPI)
   - Endpoint catalog
   - Request/response schemas
   - Authentication flows
   - Error codes

3. **MCP Protocol** (Hand-written from rmcp schemas)
   - Tool specifications
   - Message format
   - Transport modes (stdio, HTTP)
   - Client integration examples

4. **Rust Internals** (cargo doc)
   - Crate overview pages
   - Module documentation
   - Trait relationships
   - Extension points

### Content Priority Matrix

| Content | User Impact | Effort | Priority | Target Week |
|---------|-------------|--------|----------|-------------|
| Home page | Critical | Medium | P0 | Week 1 |
| Quick Start | Critical | Low | P0 | Week 1 |
| Installation guide | Critical | Low | P0 | Week 1 |
| CLI reference | High | Medium | P1 | Week 2 |
| SKILL.md format | High | Low | P1 | Week 2 |
| Claude Code integration | High | Low | P1 | Week 2 |
| REST API reference | High | High | P1 | Week 3 |
| Live playground | High | Very High | P1 | Week 5-6 |
| RAG configuration | Medium | Medium | P2 | Week 3 |
| WASM skill guide | Medium | Medium | P2 | Week 4 |
| Docker skill guide | Medium | Low | P2 | Week 4 |
| Architecture deep dive | Medium | High | P2 | Week 7 |
| MCP protocol docs | Medium | High | P2 | Week 3 |
| Rust API docs | Medium | High | P2 | Week 6 |
| Troubleshooting guide | Low | Medium | P3 | Week 8 |
| FAQ | Low | Low | P3 | Week 8 |
| Community gallery | Low | Low | P3 | Post-launch |

### Content Style Guide

**Voice & Tone**
- **Professional but approachable**: Technical accuracy without jargon overload
- **Action-oriented**: Start with verbs, focus on what users can accomplish
- **Confidence-building**: Acknowledge complexity, provide clear paths forward

**Writing Guidelines**
- Use active voice ("Run the command" not "The command is run")
- Code examples before explanation
- One concept per page (link to related topics)
- Assume intermediate developer knowledge
- Define domain-specific terms on first use

**Code Examples**
- Always include expected output
- Show both success and error cases
- Use realistic data, not foo/bar
- Provide copy-paste ready snippets
- Annotate complex examples

---

## 5. Interactive Features

### 5.1 Live Code Playground

**Purpose**: Let users execute WASM skills in-browser without installation

**Architecture**
```
┌─────────────────────────────────────────────────────────────┐
│                    Browser (VitePress Site)                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Monaco Editor                                      │    │
│  │  - Syntax highlighting (Rust/JS/TS)                │    │
│  │  - Auto-completion for skill API                   │    │
│  │  - Error squiggles                                 │    │
│  └────────────────────────────────────────────────────┘    │
│                         │                                   │
│                         ▼                                   │
│  ┌────────────────────────────────────────────────────┐    │
│  │  WASM Skill Runtime (skill-runtime compiled to Web)│    │
│  │  - Wasmtime WASM                                   │    │
│  │  - Sandboxed execution                             │    │
│  │  - Capability restrictions                         │    │
│  └────────────────────────────────────────────────────┘    │
│                         │                                   │
│                         ▼                                   │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Output Panel                                       │    │
│  │  - Stdout/stderr streams                           │    │
│  │  - JSON formatter                                  │    │
│  │  - Execution metrics (time, memory)                │    │
│  └────────────────────────────────────────────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Features**
- Pre-loaded example skills (hello-world, aws-s3, github-api)
- Import skill from GitHub URL
- Edit parameters dynamically
- Share playground state via URL hash
- Export to local project (download skill.js)

**Technical Requirements**
1. Compile `skill-runtime` to `wasm32-unknown-unknown` target
2. Bundle with VitePress using vite-plugin-wasm
3. Implement WebAssembly System Interface (WASI) polyfills for browser
4. Limit execution time (5s timeout) and memory (50MB)
5. Use Web Workers to avoid blocking main thread

**Implementation Steps**
1. Create `crates/skill-runtime-web` with wasm-bindgen bindings
2. Add WASI polyfills for filesystem (virtual), network (disabled)
3. Build skill examples to WASM and include in docs bundle
4. Develop Monaco-based editor component in VitePress
5. Add execution metrics (time, memory) via Performance API

**Example Use Cases**
- Try the hello-world skill without installing
- Modify AWS S3 skill parameters interactively
- Paste SKILL.md content and see parsed output
- Debug skill execution errors in real-time

### 5.2 API Explorer

**Purpose**: Interactive REST API testing without Postman/curl

**Architecture**
```
┌─────────────────────────────────────────────────────────────┐
│                      API Explorer UI                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │  OpenAPI Spec Loader                               │    │
│  │  - Auto-generated from skill-http handlers         │    │
│  │  - Kept in sync with code                          │    │
│  └────────────────────────────────────────────────────┘    │
│                         │                                   │
│                         ▼                                   │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Swagger UI / RapiDoc Integration                  │    │
│  │  - Endpoint catalog                                │    │
│  │  - Request builder with schema validation          │    │
│  │  - Auth token management                           │    │
│  │  - Response viewer (JSON, headers)                 │    │
│  └────────────────────────────────────────────────────┘    │
│                         │                                   │
│                         ▼                                   │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Live Skill Engine Instance                        │    │
│  │  - Can connect to localhost:3000                   │    │
│  │  - Or use public demo instance                     │    │
│  └────────────────────────────────────────────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Features**
- Browse all REST API endpoints
- Fill parameters with example values
- Execute requests against live instance
- View response with syntax highlighting
- Copy as curl/fetch/axios
- Authentication: API key input field

**Technical Requirements**
1. Generate OpenAPI 3.1 spec from Axum handlers using `utoipa` crate
2. Embed Swagger UI or RapiDoc in VitePress
3. Proxy API calls through docs site to avoid CORS
4. Provide example data generators for complex schemas
5. Save request history in localStorage

**Implementation Steps**
1. Add `utoipa` and `utoipa-swagger-ui` to skill-http dependencies
2. Annotate all handlers with OpenAPI derive macros
3. Generate openapi.json at build time
4. Create VitePress component that embeds Swagger UI
5. Add "Try It" buttons in API reference pages

**Example Endpoints to Document**
- POST `/api/execute` - Execute skill tool
- GET `/api/skills` - List installed skills
- POST `/api/skills` - Install new skill
- GET `/api/search` - Semantic search
- POST `/api/services/start` - Start required service

### 5.3 Configuration Generator

**Purpose**: Visual editor for `.skill-engine.toml` files

**Architecture**
```
┌─────────────────────────────────────────────────────────────┐
│                 Configuration Generator                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Schema Source                                      │    │
│  │  - Parsed from manifest.rs structs                 │    │
│  │  - Converted to JSON Schema                        │    │
│  └────────────────────────────────────────────────────┘    │
│                         │                                   │
│                         ▼                                   │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Form Builder (vue-form-json-schema)               │    │
│  │  - Auto-generated forms from JSON Schema           │    │
│  │  - Field validation                                │    │
│  │  - Conditional fields                              │    │
│  │  - Help tooltips                                   │    │
│  └────────────────────────────────────────────────────┘    │
│                         │                                   │
│                         ▼                                   │
│  ┌────────────────────────────────────────────────────┐    │
│  │  TOML Preview (live)                               │    │
│  │  - Real-time rendering                             │    │
│  │  - Syntax highlighting                             │    │
│  │  - Copy to clipboard                               │    │
│  │  - Download file                                   │    │
│  └────────────────────────────────────────────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Features**
- Step-by-step wizard mode
- Visual skill source selector (local path, GitHub URL, registry)
- Instance configuration builder
- Environment variable template
- Capability checkboxes with explanations
- Search pipeline configuration (embedding model, reranker, etc.)
- Validation against schema
- Export as .skill-engine.toml

**Technical Requirements**
1. Generate JSON Schema from Rust structs using `schemars`
2. Build Vue.js form component with vue-form-json-schema
3. Implement TOML serializer with comments preservation
4. Add field-level help text from doc comments
5. Validate config before export

**Implementation Steps**
1. Add `schemars` derive to all manifest structs in skill-runtime
2. Generate config-schema.json at build time
3. Create Vue component for form builder
4. Add TOML preview pane with @ltd/j-toml parser
5. Implement download and copy-to-clipboard functionality

**Example Wizards**
- New Skill Setup: source → capabilities → instance config
- RAG Configuration: embedding provider → vector store → reranker
- Multi-Instance Setup: production + staging + development configs

### 5.4 Search Tester

**Purpose**: Visualize and debug RAG pipeline behavior

**Architecture**
```
┌─────────────────────────────────────────────────────────────┐
│                      Search Tester UI                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Query Input                                        │    │
│  │  - Natural language search                         │    │
│  │  - Advanced options (top_k, filters)               │    │
│  └────────────────────────────────────────────────────┘    │
│                         │                                   │
│                         ▼                                   │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Pipeline Visualization                             │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │    │
│  │  │ Embedding    │→│ Dense Search │→│ Reranker │ │    │
│  │  │ (5ms)        │  │ (12ms)       │  │ (45ms)   │ │    │
│  │  └──────────────┘  └──────────────┘  └──────────┘ │    │
│  │  ┌──────────────┐  ┌──────────────┐                │    │
│  │  │ BM25 Search  │→│ RRF Fusion   │                │    │
│  │  │ (8ms)        │  │ (2ms)        │                │    │
│  │  └──────────────┘  └──────────────┘                │    │
│  └────────────────────────────────────────────────────┘    │
│                         │                                   │
│                         ▼                                   │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Results Panel                                      │    │
│  │  - Ranked list with scores                         │    │
│  │  - Highlight matched terms                         │    │
│  │  - Explain scores (dense vs sparse vs rerank)      │    │
│  │  - Compare with/without features                   │    │
│  └────────────────────────────────────────────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Features**
- Visual pipeline diagram with timing
- Score breakdown (dense, sparse, reranker contributions)
- Comparison mode (before/after reranking)
- Config tuning sliders (dense_weight, sparse_weight)
- Export query results as JSON
- Share search configuration via URL

**Technical Requirements**
1. Expose debug endpoint: POST `/api/search/debug`
2. Return intermediate results at each pipeline stage
3. Build D3.js/Mermaid pipeline visualization
4. Calculate score attributions
5. Allow dynamic config override via API

**Implementation Steps**
1. Add debug mode to RAG pipeline in skill-runtime
2. Create /api/search/debug endpoint in skill-http
3. Build Vue component for pipeline visualization
4. Add score breakdown table
5. Implement comparison mode (toggle features on/off)

**Example Queries**
- "deploy kubernetes application" (shows tool ranking)
- "convert video to gif" (shows semantic matching)
- "scale replicas" (shows keyword vs semantic balance)

---

## 6. API Documentation

### 6.1 Rust Documentation (cargo doc)

**Strategy**: Auto-generate from inline doc comments

**Requirements**
1. Add doc comments to all public items (90%+ coverage target)
2. Include examples in doc comments with `/// # Example`
3. Link between related types using `[Type]` syntax
4. Add module-level documentation explaining purpose
5. Use `#[doc(hidden)]` for internal-only items

**Priority Crates**
1. **skill-runtime** (CRITICAL)
   - VectorStore trait and backends
   - Embedding providers
   - Search pipeline components
   - Manifest structs
   - WASM execution engine

2. **skill-http** (HIGH)
   - Handler functions
   - Request/response types
   - Service management
   - Analytics types

3. **skill-mcp** (HIGH)
   - MCP tool definitions
   - Protocol message types
   - Transport implementations

4. **skill-cli** (MEDIUM)
   - Command implementations
   - Config management
   - Auth providers

5. **skill-web** (LOW)
   - Component docs (user-facing UI, less critical for API)

**Implementation Plan**
1. Week 1: Audit existing coverage with `cargo doc --open`
2. Week 2-3: Add docs to skill-runtime (highest priority)
3. Week 4: Document skill-http and skill-mcp
4. Week 5: Document skill-cli
5. Week 6: Polish and interlink docs

**Documentation Template**
```rust
/// Brief one-line summary of what this does.
///
/// More detailed explanation of the type/function, including:
/// - What problem it solves
/// - Key concepts or terminology
/// - Important behavior notes
///
/// # Example
///
/// ```
/// use skill_runtime::VectorStore;
///
/// let store = InMemoryVectorStore::new(384);
/// store.upsert(documents).await?;
/// let results = store.search(&query_embedding, 10, None).await?;
/// ```
///
/// # Errors
///
/// Returns `Err` if:
/// - Condition 1
/// - Condition 2
///
/// # Panics
///
/// Panics if invariant X is violated.
///
/// # Safety (if unsafe)
///
/// Caller must ensure...
pub fn function_name() {}
```

### 6.2 REST API (OpenAPI)

**Strategy**: Auto-generate from Axum handlers using utoipa

**Requirements**
1. Annotate all handlers with `#[utoipa::path]`
2. Derive OpenAPI schemas for request/response types
3. Add examples to schemas
4. Document error responses
5. Include authentication requirements

**Implementation Steps**
1. Add dependencies:
   ```toml
   utoipa = { version = "5", features = ["axum_extras"] }
   utoipa-swagger-ui = { version = "8", features = ["axum"] }
   ```

2. Annotate handlers:
   ```rust
   #[utoipa::path(
       post,
       path = "/api/execute",
       request_body = ExecuteRequest,
       responses(
           (status = 200, description = "Tool executed successfully", body = ExecuteResponse),
           (status = 400, description = "Invalid request", body = ErrorResponse),
           (status = 500, description = "Execution failed", body = ErrorResponse)
       ),
       tag = "execution"
   )]
   async fn execute_tool(
       State(state): State<AppState>,
       Json(req): Json<ExecuteRequest>,
   ) -> Result<Json<ExecuteResponse>, AppError> {
       // ...
   }
   ```

3. Generate spec:
   ```rust
   use utoipa::OpenApi;

   #[derive(OpenApi)]
   #[openapi(
       paths(execute_tool, list_skills, search_skills),
       components(schemas(ExecuteRequest, ExecuteResponse, Skill))
   )]
   struct ApiDoc;

   // Serve at /api/openapi.json
   ```

4. Embed Swagger UI in docs site

**Endpoints to Document**

| Endpoint | Method | Category | Priority |
|----------|--------|----------|----------|
| `/api/execute` | POST | Execution | P0 |
| `/api/skills` | GET | Skills | P0 |
| `/api/skills/:name` | GET | Skills | P0 |
| `/api/skills` | POST | Skills | P1 |
| `/api/skills/:name` | DELETE | Skills | P1 |
| `/api/search` | POST | Discovery | P0 |
| `/api/services` | GET | Services | P1 |
| `/api/services/start` | POST | Services | P1 |
| `/api/services/stop` | POST | Services | P1 |
| `/api/executions` | GET | History | P2 |
| `/api/executions/:id` | GET | History | P2 |

### 6.3 MCP Protocol

**Strategy**: Hand-written from rmcp schemas + examples

**Content Structure**
```
/api/mcp/
├── overview.md          # What is MCP, why use it
├── tools.md             # Tool catalog with schemas
├── resources.md         # Resource providers (if any)
├── prompts.md           # Prompt templates (if any)
├── transport.md         # Stdio vs HTTP streaming
└── integration.md       # Client integration guide
```

**Tool Documentation Template**
```markdown
## execute

Execute a skill tool with arguments.

### Schema

```json
{
  "name": "execute",
  "description": "Execute any installed skill tool",
  "inputSchema": {
    "type": "object",
    "properties": {
      "skill": { "type": "string", "description": "Skill name" },
      "tool": { "type": "string", "description": "Tool name" },
      "args": { "type": "object", "description": "Tool arguments" }
    },
    "required": ["skill", "tool"]
  }
}
```

### Examples

**Execute Kubernetes tool**
```json
{
  "method": "tools/call",
  "params": {
    "name": "execute",
    "arguments": {
      "skill": "kubernetes",
      "tool": "get",
      "args": {
        "resource": "pods",
        "namespace": "default"
      }
    }
  }
}
```

### Response

Success:
```json
{
  "content": [
    { "type": "text", "text": "NAME\t\tREADY\tSTATUS\n..." }
  ]
}
```

Error:
```json
{
  "error": {
    "code": -32603,
    "message": "Tool execution failed: kubectl not found"
  }
}
```
```

**Tools to Document**
1. `execute` - Run skill tools
2. `list_skills` - Discover skills
3. `search_skills` - Semantic search

### 6.4 CLI Reference

**Strategy**: Auto-generate from Clap structs + hand-written examples

**Implementation**
1. Use clap's markdown generation:
   ```rust
   use clap::CommandFactory;
   use clap_mangen::Man;

   let cmd = Cli::command();
   let man = Man::new(cmd);
   man.render(&mut output)?;
   ```

2. Convert man pages to markdown
3. Enhance with examples and notes

**Command Structure**
```
skill
├── install <source>           # Install skill
├── remove <skill>             # Uninstall skill
├── list                       # List skills
├── info <skill>               # Skill details
├── run <skill>:<tool> [args]  # Execute tool
├── find <query>               # Semantic search
├── serve [--http] [--port]    # Start MCP server
├── config <skill>             # Configure skill
├── claude                     # Claude Code integration
│   ├── setup                  # Auto-configure
│   ├── status                 # Check status
│   └── remove                 # Remove integration
├── upgrade                    # Self-update
└── init [path]                # Create new skill
```

**Documentation Template**
```markdown
## skill run

Execute a skill tool with arguments.

### Usage

```bash
skill run <skill>:<tool> [ARGS]...
skill run <skill>@<instance>:<tool> [ARGS]...
```

### Arguments

- `<skill>` - Skill name (required)
- `<instance>` - Instance name (optional, default: "default")
- `<tool>` - Tool name (required)
- `[ARGS]` - Tool arguments as key=value pairs

### Options

- `--json` - Output in JSON format
- `--timeout <SECONDS>` - Execution timeout (default: 30)
- `--env <KEY=VALUE>` - Additional environment variables

### Examples

**Basic execution**
```bash
skill run kubernetes:get resource=pods
```

**With instance**
```bash
skill run aws@prod:s3-list bucket=my-bucket
```

**JSON output**
```bash
skill run github:list-repos owner=kubiyabot --json
```

**Custom timeout**
```bash
skill run video-converter:to-gif input=video.mp4 --timeout 120
```

### Exit Codes

- `0` - Success
- `1` - General error
- `2` - Tool execution failed
- `3` - Skill not found
- `4` - Invalid arguments
```

---

## 7. Design System

### 7.1 VitePress Theme Customization

**Base Theme**: VitePress default theme with overrides

**Color Palette**
```css
:root {
  /* Primary (Brand) */
  --vp-c-brand-1: #6366f1;  /* Indigo 500 */
  --vp-c-brand-2: #4f46e5;  /* Indigo 600 */
  --vp-c-brand-3: #4338ca;  /* Indigo 700 */
  
  /* Accent (Interactive) */
  --vp-c-accent-1: #10b981; /* Emerald 500 */
  --vp-c-accent-2: #059669; /* Emerald 600 */
  
  /* Background */
  --vp-c-bg: #ffffff;
  --vp-c-bg-soft: #f9fafb;  /* Gray 50 */
  --vp-c-bg-mute: #f3f4f6;  /* Gray 100 */
  
  /* Text */
  --vp-c-text-1: #111827;   /* Gray 900 */
  --vp-c-text-2: #4b5563;   /* Gray 600 */
  --vp-c-text-3: #9ca3af;   /* Gray 400 */
  
  /* Code */
  --vp-c-code-bg: #f3f4f6;
  --vp-c-code-border: #e5e7eb;
}

.dark {
  --vp-c-bg: #0f172a;       /* Slate 900 */
  --vp-c-bg-soft: #1e293b;  /* Slate 800 */
  --vp-c-bg-mute: #334155;  /* Slate 700 */
  
  --vp-c-text-1: #f1f5f9;   /* Slate 100 */
  --vp-c-text-2: #cbd5e1;   /* Slate 300 */
  --vp-c-text-3: #64748b;   /* Slate 500 */
}
```

**Typography**
- Headings: Inter (sans-serif)
- Body: Inter
- Code: Fira Code (with ligatures)

**Component Overrides**
- Hero section: Custom gradient background
- Feature cards: Glass morphism effect
- Code blocks: Custom theme (Night Owl for dark, Light Owl for light)
- Search: Algolia DocSearch integration

### 7.2 Custom Components

**SkillPlayground.vue**
```vue
<template>
  <div class="skill-playground">
    <div class="editor-pane">
      <MonacoEditor v-model="code" language="javascript" />
    </div>
    <div class="controls">
      <select v-model="selectedExample">
        <option value="hello">Hello World</option>
        <option value="github">GitHub API</option>
        <option value="aws">AWS S3</option>
      </select>
      <button @click="execute">Run</button>
      <button @click="share">Share</button>
    </div>
    <div class="output-pane">
      <pre>{{ output }}</pre>
    </div>
  </div>
</template>
```

**ApiExplorer.vue**
```vue
<template>
  <div class="api-explorer">
    <SwaggerUI :spec="openApiSpec" />
  </div>
</template>
```

**ConfigGenerator.vue**
```vue
<template>
  <div class="config-generator">
    <div class="form-panel">
      <VueFormJsonSchema :schema="configSchema" v-model="config" />
    </div>
    <div class="preview-panel">
      <CodeBlock language="toml" :code="generatedToml" />
      <button @click="download">Download</button>
      <button @click="copy">Copy</button>
    </div>
  </div>
</template>
```

**SearchTester.vue**
```vue
<template>
  <div class="search-tester">
    <input v-model="query" placeholder="Search for tools..." />
    <div class="pipeline-viz">
      <PipelineStage name="Embedding" :duration="metrics.embedding" />
      <PipelineStage name="Dense Search" :duration="metrics.dense" />
      <PipelineStage name="BM25 Search" :duration="metrics.sparse" />
      <PipelineStage name="Fusion" :duration="metrics.fusion" />
      <PipelineStage name="Reranker" :duration="metrics.rerank" />
    </div>
    <div class="results">
      <SearchResult v-for="result in results" :key="result.id" :result="result" />
    </div>
  </div>
</template>
```

### 7.3 Code Highlighting

**Languages to Support**
- Rust
- JavaScript/TypeScript
- Bash/Shell
- TOML
- JSON
- YAML
- Markdown

**Syntax Themes**
- Light: GitHub Light
- Dark: Night Owl

**Features**
- Line numbers
- Copy button
- Highlighted lines
- Filename caption
- Language badge

---

## 8. Technical Implementation

### 8.1 VitePress Setup

**Project Structure**
```
docs/
├── .vitepress/
│   ├── config.ts           # VitePress config
│   ├── theme/
│   │   ├── index.ts        # Theme entry
│   │   ├── style.css       # Custom styles
│   │   └── components/     # Custom Vue components
│   ├── public/             # Static assets
│   │   ├── images/
│   │   ├── wasm/           # Pre-built WASM modules
│   │   └── openapi.json    # Generated API spec
│   └── cache/              # Build cache
├── guide/
│   ├── index.md
│   ├── installation.md
│   └── ...
├── guides/
├── api/
├── playground/
├── examples/
├── architecture/
├── resources/
└── index.md                # Home page
```

**Configuration** (`.vitepress/config.ts`)
```typescript
import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Skill Engine',
  description: 'Universal AI agent runtime with WASM, Docker, and Native execution',
  
  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:image', content: '/og-image.png' }],
  ],
  
  themeConfig: {
    logo: '/logo.svg',
    
    nav: [
      { text: 'Guide', link: '/guide/' },
      { text: 'API', link: '/api/' },
      { text: 'Playground', link: '/playground/live' },
      { text: 'Examples', link: '/examples/' },
      {
        text: 'v0.3.0',
        items: [
          { text: 'Changelog', link: '/resources/changelog' },
          { text: 'Roadmap', link: '/resources/roadmap' }
        ]
      }
    ],
    
    sidebar: {
      '/guide/': [
        {
          text: 'Getting Started',
          items: [
            { text: 'Introduction', link: '/guide/' },
            { text: 'Installation', link: '/guide/installation' },
            { text: 'Quick Start', link: '/guide/quick-start' },
            { text: 'Core Concepts', link: '/guide/concepts' },
          ]
        }
      ],
      // ... more sidebar configs
    },
    
    socialLinks: [
      { icon: 'github', link: 'https://github.com/kubiyabot/skill' }
    ],
    
    search: {
      provider: 'algolia',
      options: {
        appId: 'YOUR_APP_ID',
        apiKey: 'YOUR_API_KEY',
        indexName: 'skill-engine'
      }
    },
    
    footer: {
      message: 'Released under the Apache-2.0 License.',
      copyright: 'Copyright © 2026 Skill Engine Contributors'
    }
  },
  
  vite: {
    plugins: [
      // WASM plugin for playground
      wasmPlugin(),
    ]
  }
})
```

### 8.2 Build Process

**GitHub Actions Workflow** (`.github/workflows/docs.yml`)
```yaml
name: Deploy Docs

on:
  push:
    branches: [main]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: 'npm'
          cache-dependency-path: docs/package-lock.json
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown
      
      - name: Generate Rust docs
        run: |
          cargo doc --no-deps --all-features
          cp -r target/doc docs/.vitepress/dist/api/rust
      
      - name: Build WASM playground modules
        run: |
          cd crates/skill-runtime-web
          wasm-pack build --target web --out-dir ../../docs/.vitepress/public/wasm
      
      - name: Generate OpenAPI spec
        run: |
          cargo run -p skill-cli -- generate-openapi > docs/.vitepress/public/openapi.json
      
      - name: Install docs dependencies
        run: cd docs && npm ci
      
      - name: Build docs
        run: cd docs && npm run build
      
      - name: Setup Pages
        uses: actions/configure-pages@v4
      
      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: docs/.vitepress/dist
  
  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

**Build Optimization**
- Enable VitePress build cache
- Use incremental cargo doc builds
- CDN for heavy assets (WASM modules)
- Aggressive asset compression (Brotli)
- Image optimization (WebP, AVIF)

### 8.3 Deployment

**GitHub Pages Configuration**
- Repository: `kubiyabot/skill`
- Branch: `gh-pages` (auto-created by action)
- Custom domain: `docs.skill-engine.dev` (future)
- HTTPS: Enforced
- Build source: GitHub Actions

**Performance Targets**
- Lighthouse Score: 95+ (all categories)
- First Contentful Paint: < 1.5s
- Time to Interactive: < 3.5s
- Total Bundle Size: < 500KB (gzipped)
- WASM Playground Load: < 2s

**Analytics**
- Google Analytics 4 (privacy-friendly mode)
- Track: Page views, search queries, playground usage
- No PII collection

---

## 9. Search & Discovery

### 9.1 Integration with RAG Pipeline

**Strategy**: Use existing skill-runtime search components for docs search

**Architecture**
```
┌─────────────────────────────────────────────────────────────┐
│                    Docs Search System                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Algolia DocSearch (Primary)                       │    │
│  │  - Instant search as you type                      │    │
│  │  - Keyword-based ranking                           │    │
│  │  - Built-in analytics                              │    │
│  └────────────────────────────────────────────────────┘    │
│                         +                                   │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Skill Engine RAG (Secondary, Experimental)        │    │
│  │  - Semantic understanding                          │    │
│  │  - Code example search                             │    │
│  │  - Cross-reference suggestions                     │    │
│  └────────────────────────────────────────────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Algolia DocSearch**
- Primary search mechanism (proven, reliable)
- Free for open source projects
- Minimal setup (crawler config + JS widget)

**Custom RAG Search** (Experimental)
- Semantic search across docs content
- Find conceptually related pages
- Suggest relevant code examples
- Powered by existing skill-runtime RAG pipeline

### 9.2 Search UX

**Search Widget**
- Keyboard shortcut: `/` or `Cmd+K`
- Instant results (no "Enter" required)
- Category filters (Guide, API, Examples)
- Recent searches history
- Suggested queries

**Search Results**
- Highlighted matching text
- Breadcrumb navigation
- Page excerpt
- Relevance score (for debugging)

**Autocomplete**
- Command names (`skill run`, `skill find`)
- API endpoints (`/api/execute`)
- Common queries ("how to install", "create WASM skill")

---

## 10. Phases & Timeline

### Phase 1: Foundation (Weeks 1-2)

**Objectives**
- VitePress site scaffolding
- Content migration
- Basic navigation

**Deliverables**
- VitePress configured with theme
- Home page with hero section
- Getting Started guide migrated
- Installation guide
- Quick Start tutorial
- Basic search (Algolia)

**Success Criteria**
- Site builds successfully
- All links work
- Mobile responsive
- Lighthouse score > 90

### Phase 2: API Documentation (Weeks 3-4)

**Objectives**
- Auto-generate API references
- Document CLI commands
- REST API spec

**Deliverables**
- Rust docs with 80%+ coverage
- OpenAPI spec generated
- CLI command reference
- MCP protocol docs
- Example requests/responses

**Success Criteria**
- All public APIs documented
- cargo doc builds without warnings
- OpenAPI spec validates
- Examples are copy-paste ready

### Phase 3: Interactive Features (Weeks 5-7)

**Objectives**
- Build live playground
- API explorer
- Config generator

**Deliverables**
- WASM playground with Monaco editor
- Pre-loaded skill examples
- Swagger UI integration
- Config generator with schema validation
- Share/export functionality

**Success Criteria**
- Playground executes skills in < 2s
- API explorer works against demo instance
- Config generator produces valid TOML
- All interactive features mobile-friendly

### Phase 4: Content & Polish (Weeks 8-9)

**Objectives**
- Complete remaining content
- Advanced guides
- Architecture docs

**Deliverables**
- RAG pipeline guides (configuration, tuning)
- WASM skill development guide
- Docker skill development guide
- Architecture deep dives
- Troubleshooting guide
- FAQ

**Success Criteria**
- All high-priority content complete
- No broken links
- Consistent formatting
- Code examples tested

### Phase 5: Launch Prep (Week 10)

**Objectives**
- Final testing
- Performance optimization
- Launch preparation

**Deliverables**
- Comprehensive testing (cross-browser, mobile)
- Performance tuning (bundle size, load times)
- Analytics setup
- Launch announcement (blog post, social media)
- Submission to Algolia DocSearch

**Success Criteria**
- Lighthouse score 95+ (all categories)
- Zero critical bugs
- All interactive features tested
- Analytics tracking verified

---

## 11. Dependencies & Resources

### 11.1 Technical Dependencies

**Build Tools**
- Node.js 20+ (for VitePress)
- Rust 1.75+ (for cargo doc, WASM builds)
- wasm-pack (for WASM playground)
- Trunk (for skill-web builds)

**VitePress Plugins**
- vite-plugin-wasm
- @vueuse/core
- vue-form-json-schema
- @monaco-editor/vue

**External Services**
- GitHub Pages (hosting)
- Algolia DocSearch (search)
- Google Analytics (analytics)
- Cloudflare (CDN, optional)

### 11.2 Content Dependencies

**Existing Docs to Migrate**
- README.md (5 sections)
- docs/skill-development.md (435 lines)
- docs/rag-search.md (485 lines)
- docs/web-interface.md (263 lines)
- docs/QUICK_START_CLAUDE_CODE.md
- docs/MANIFEST_GUIDE.md (1000+ lines)

**New Content to Create**
- Getting Started (3 pages)
- Integration Guides (4 pages)
- Architecture Deep Dives (5 pages)
- Troubleshooting & FAQ (2 pages)

### 11.3 Effort Estimates

| Phase | Tasks | Estimated Hours | Risks |
|-------|-------|-----------------|-------|
| Phase 1 | VitePress setup, content migration | 40h | Low |
| Phase 2 | API documentation, inline docs | 60h | Medium (doc comment coverage) |
| Phase 3 | Interactive features, WASM playground | 80h | High (WASM complexity) |
| Phase 4 | Advanced content, guides | 50h | Low |
| Phase 5 | Testing, optimization, launch | 30h | Low |
| **Total** | | **260h** | |

**Team Requirements**
- 1 developer with Rust + Web (full-time, 6-8 weeks)
- 1 technical writer (part-time, 4 weeks)
- 1 designer (review only, 1 week)

---

## 12. Risks & Mitigation

### 12.1 Technical Risks

**Risk 1: WASM Playground Complexity**
- **Impact**: High (core feature)
- **Likelihood**: Medium
- **Mitigation**: 
  - Start with simple executor, iterate
  - Use existing wasmtime-js bindings
  - Limit execution scope initially
  - Have fallback to codepen embeds

**Risk 2: Rust Doc Coverage**
- **Impact**: Medium (quality issue)
- **Likelihood**: High (currently < 30%)
- **Mitigation**:
  - Prioritize public API surface
  - Use doc coverage tools (cargo-llvm-cov with --doc-coverage)
  - Set CI check for minimum coverage
  - Document incrementally per crate

**Risk 3: OpenAPI Generation**
- **Impact**: Medium (API docs quality)
- **Likelihood**: Low (utoipa is mature)
- **Mitigation**:
  - Test utoipa integration early
  - Hand-write schemas if auto-gen fails
  - Validate spec with OpenAPI tools

**Risk 4: Build Performance**
- **Impact**: Low (developer experience)
- **Likelihood**: Medium (cargo doc + WASM builds slow)
- **Mitigation**:
  - Cache aggressively in CI
  - Use incremental builds
  - Separate doc builds from site builds

### 12.2 Content Risks

**Risk 5: Documentation Drift**
- **Impact**: High (outdated docs hurt credibility)
- **Likelihood**: High (fast-moving project)
- **Mitigation**:
  - Link docs to code (via cargo doc)
  - Add CI check for broken links
  - Version docs with releases
  - Use "Last updated" timestamps

**Risk 6: Incomplete Examples**
- **Impact**: Medium (user confusion)
- **Likelihood**: Medium
- **Mitigation**:
  - Test all examples in CI
  - Use doc-comment examples (tested)
  - Maintain example repository
  - Community contribution guidelines

**Risk 7: Search Quality**
- **Impact**: Medium (poor discovery)
- **Likelihood**: Low (Algolia is reliable)
- **Mitigation**:
  - Optimize Algolia crawler config
  - Add manual synonyms
  - Monitor zero-result queries
  - Fallback to GitHub search

### 12.3 Operational Risks

**Risk 8: Hosting Costs**
- **Impact**: Low (GitHub Pages is free)
- **Likelihood**: Low
- **Mitigation**:
  - GitHub Pages free for public repos
  - Cloudflare CDN free tier
  - Algolia DocSearch free for OSS

**Risk 9: Maintenance Burden**
- **Impact**: Medium (ongoing effort)
- **Likelihood**: High
- **Mitigation**:
  - Automate doc generation
  - Community contributions
  - Regular review cycles
  - Clear contribution guidelines

---

## 13. Success Criteria

### 13.1 Launch Criteria

**Must Have (Blocking Launch)**
- [ ] Home page with clear value proposition
- [ ] Complete Getting Started guide (installation → first skill)
- [ ] CLI command reference (all 24 commands)
- [ ] REST API reference (top 10 endpoints)
- [ ] SKILL.md format documentation
- [ ] Claude Code integration guide
- [ ] Working Algolia search
- [ ] Mobile responsive
- [ ] Lighthouse score > 90

**Should Have (Launch Soon After)**
- [ ] Live WASM playground
- [ ] Full REST API coverage
- [ ] MCP protocol docs
- [ ] RAG configuration guide
- [ ] WASM skill development guide
- [ ] Rust API docs (80%+ coverage)

**Nice to Have (Iterate Post-Launch)**
- [ ] API explorer with Swagger UI
- [ ] Config generator
- [ ] Search tester
- [ ] Docker skill guide
- [ ] Architecture deep dives
- [ ] Community gallery

### 13.2 Post-Launch Metrics

**Month 1 Targets**
- 1,000 unique visitors
- 50 GitHub stars added
- < 3% bounce rate on Getting Started
- 100+ playground executions
- 10+ community PRs

**Month 3 Targets**
- 5,000 unique visitors
- 300 GitHub stars added
- 80%+ doc completeness
- 1,000+ playground executions
- 50+ community PRs

**Month 6 Targets**
- 15,000 unique visitors
- 500 GitHub stars added
- 95%+ doc completeness
- 5,000+ playground executions
- Active community contributions

### 13.3 Iteration Plan

**Weekly Reviews**
- Analytics dashboard review
- User feedback triage
- Content gap identification
- Search query analysis

**Monthly Updates**
- New feature documentation
- Example additions
- Community contributions
- Performance optimization

**Quarterly Audits**
- Comprehensive link checking
- Content freshness review
- API documentation sync
- UX/design refresh

---

## Appendix A: Claude Bridge Documentation

**Status**: NEW FEATURE (1,550 lines, zero docs)

**Required Docs**
1. **Overview** (`/guides/claude-bridge/overview.md`)
   - What is Claude Bridge
   - When to use it (skill generation from existing tools)
   - Architecture diagram

2. **Tutorial** (`/guides/claude-bridge/tutorial.md`)
   - Generate skill from Terraform CLI
   - Generate skill from kubectl
   - Generate skill from custom tool

3. **Reference** (`/api/claude-bridge.md`)
   - Command syntax
   - Loader behavior
   - Validator rules
   - Transformer logic
   - Renderer options

4. **Integration** (`/guides/claude-bridge/integration.md`)
   - Using generated skills
   - Customizing output
   - Troubleshooting

---

## Appendix B: Competitive Analysis

| Feature | Skill Engine Docs | E2B Docs | Modal Docs | Dagger Docs |
|---------|------------------|----------|------------|-------------|
| **Live Playground** | Planned | ✅ | ✅ | ❌ |
| **API Explorer** | Planned | ✅ | ✅ | ❌ |
| **Auto-Generated API Docs** | Planned | ✅ | ✅ | ✅ |
| **Search Quality** | Good (planned) | Excellent | Excellent | Good |
| **Mobile Experience** | Planned | Good | Excellent | Fair |
| **Interactive Examples** | Planned | ✅ | ✅ | ✅ |
| **Version Selector** | Planned | ✅ | ✅ | ✅ |
| **Dark Mode** | ✅ (VitePress) | ✅ | ✅ | ✅ |

**Takeaways**
- Must have live playground (table stakes)
- API explorer expected for developer tools
- Search quality is differentiator
- Mobile experience critical (60%+ mobile traffic)

---

## Appendix C: Example Page Layouts

### Home Page Wireframe
```
┌─────────────────────────────────────────────────────────────┐
│  [ Skill Engine Logo ]         [ Guide ] [ API ] [ GitHub ] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│              Give Your AI Agent Superpowers                 │
│        Universal runtime for WASM, Docker, Native skills    │
│                                                             │
│      [ Get Started →]         [ View Examples →]            │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  $ skill run kubernetes:get resource=pods           │   │
│  │                                                      │   │
│  │  NAME              READY   STATUS    RESTARTS       │   │
│  │  nginx-7d...       1/1     Running   0              │   │
│  │  postgres-9a...    1/1     Running   0              │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  Why Skill Engine?                                          │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Secure       │  │ Universal    │  │ Intelligent  │     │
│  │ WASM sandbox │  │ Multi-runtime│  │ RAG search   │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  Featured Examples                                          │
│  [ Kubernetes ] [ AWS ] [ GitHub ] [ Docker ] [ More → ]    │
└─────────────────────────────────────────────────────────────┘
```

### API Reference Page
```
┌─────────────────────────────────────────────────────────────┐
│  API Reference                                [ Search... ]  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [ CLI ] [ REST ] [ MCP ] [ Rust ]                          │
│                                                             │
│  ┌────────────────────┐  ┌──────────────────────────────┐  │
│  │ Sidebar            │  │ POST /api/execute            │  │
│  │                    │  │                              │  │
│  │ Execution          │  │ Execute a skill tool         │  │
│  │  • execute         │  │                              │  │
│  │  • executions      │  │ Request Body:                │  │
│  │                    │  │ {                            │  │
│  │ Skills             │  │   "skill": "string",         │  │
│  │  • list            │  │   "tool": "string",          │  │
│  │  • install         │  │   "args": {}                 │  │
│  │  • remove          │  │ }                            │  │
│  │                    │  │                              │  │
│  │ Discovery          │  │ [ Try It → ]                 │  │
│  │  • search          │  └──────────────────────────────┘  │
│  └────────────────────┘                                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Appendix D: SEO Strategy

**Target Keywords**
- "AI agent runtime"
- "WASM plugin system"
- "MCP protocol server"
- "skill engine"
- "AI tool execution"

**Meta Tags**
- Title: "Skill Engine - Universal AI Agent Runtime"
- Description: "Give AI agents superpowers with secure, portable WASM skills. Multi-runtime support, semantic search, MCP protocol."
- OG Image: Hero screenshot with code example

**Structured Data**
- SoftwareApplication schema
- HowTo schemas for guides
- FAQ schema

**Sitemap**
- Auto-generated by VitePress
- Submit to Google Search Console

---

**END OF PRD**
