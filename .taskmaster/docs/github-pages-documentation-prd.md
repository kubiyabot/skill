# PRD: Skill Engine GitHub Pages Documentation Site

## Executive Summary

### Problem Statement
Skill Engine is a sophisticated universal runtime system for AI agents with 28,000+ lines of production-ready code across 6 crates, featuring WASM sandboxing, RAG/vector search, MCP protocol support, and a web UI. However, its current documentation (5,200+ lines) has critical gaps that create friction for developers, integrators, and operators:

**Quantified Pain Points:**
- **Discovery Time**: 15-20 minutes to understand what Skill Engine does and how it fits their use case
- **Time-to-First-Skill**: 30-45 minutes from installation to running first custom skill
- **API Discovery**: No complete REST API reference, forcing developers to read source code
- **Integration Confusion**: Claude Code integration takes 10+ minutes without clear step-by-step guide
- **Missing Features**: Claude Bridge (1,550 lines) completely undocumented despite being production-ready
- **Inline Documentation**: ~30% Rust doc coverage makes codebase navigation difficult
- **Search Quality**: No semantic search for finding relevant tools/skills quickly

**Impact:**
- Reduced adoption rate (high bounce rate on GitHub)
- Support burden (repetitive questions on integration)
- Slower contributor onboarding (lack of architecture docs)
- Missed opportunities (features like Claude Bridge hidden)

### Solution
Create a comprehensive, interactive GitHub Pages documentation site using VitePress that serves three primary audiences:

1. **Developers** building custom skills (time-to-first-skill < 10 minutes)
2. **Integrators** connecting AI tools like Claude Code (setup < 5 minutes)
3. **Operators** deploying and managing Skill Engine in production

**Key Features:**
- **Live WASM Playground**: Write and test skills in-browser without installation
- **API Explorer**: Interactive REST API testing with auto-generated OpenAPI spec
- **Configuration Generator**: Visual tool to build `.skill-engine.toml` manifests
- **RAG-Powered Search**: Semantic search with autocomplete leveraging existing vector pipeline
- **Complete API Reference**: 90%+ coverage for Rust, REST, MCP, and CLI
- **Interactive Tutorials**: Step-by-step guides with embedded code examples

### Success Metrics

**Primary KPIs:**
- **Time-to-First-Skill**: < 10 minutes (currently 30-45 minutes)
- **Discovery Time**: < 5 minutes (currently 15-20 minutes)
- **Rust Doc Coverage**: 90%+ (currently ~30%)
- **API Documentation**: 100% of REST endpoints (currently 0% formal docs)
- **Playground Usage**: 500+ executions/month within 3 months
- **GitHub Stars**: +300% increase in 3 months
- **Search Success Rate**: 85%+ queries find relevant results

**Secondary KPIs:**
- Site traffic: 500 visitors/month (Month 1) → 5,000 (Month 6)
- Pages/session: > 4 (indicating good content discovery)
- Bounce rate: < 40% (currently likely 60%+)
- Time on site: > 5 minutes average
- Mobile traffic: > 30% with responsive design

---

## Goals & Objectives

### User Goals

#### For Developers (Building Skills)
- **Goal**: Write and test first custom skill in < 10 minutes
- **Pain Point**: Current 30-45 minute learning curve discourages experimentation
- **Success Criteria**:
  - Live playground with pre-loaded examples
  - Zero-setup testing environment
  - Clear API documentation for getMetadata(), getTools(), executeTool()
  - TypeScript/JavaScript examples with auto-completion

#### For Integrators (AI Tool Connections)
- **Goal**: Integrate Skill Engine with Claude Code/Cursor in < 5 minutes
- **Pain Point**: Scattered MCP setup docs, unclear configuration
- **Success Criteria**:
  - Step-by-step integration guides with screenshots
  - One-command setup scripts
  - Troubleshooting section for common issues
  - Video walkthrough (< 3 minutes)

#### For Operators (Production Deployment)
- **Goal**: Deploy and monitor Skill Engine with confidence
- **Pain Point**: No deployment guides, unclear production best practices
- **Success Criteria**:
  - Docker/Kubernetes deployment guides
  - Monitoring and observability setup
  - Performance tuning documentation
  - Security hardening checklist

### Business Goals
- Increase GitHub stars by 300% (current ~500 → 2,000+)
- Reduce support burden by 50% (common questions answered in docs)
- Enable community contributions (clear architecture docs)
- Showcase unique features (WASM sandboxing, RAG search, Claude Bridge)

---

## Site Architecture & Information Hierarchy

### Navigation Structure

```
docs.skill-engine.dev/
├── Getting Started                    [Priority: P0, Week 1-2]
│   ├── What is Skill Engine?
│   ├── Quick Start (< 5 min)
│   ├── Installation (all platforms)
│   ├── Your First Skill (tutorial)
│   └── Core Concepts
│
├── User Guides                        [Priority: P1, Week 3-5]
│   ├── Building Skills
│   │   ├── JavaScript/TypeScript Skills
│   │   ├── Native Skills (SKILL.md)
│   │   ├── Docker Runtime Skills
│   │   ├── Configuration & Secrets
│   │   └── Testing Your Skills
│   ├── Integration
│   │   ├── Claude Code Setup
│   │   ├── Cursor Integration
│   │   ├── MCP Protocol Deep Dive
│   │   └── Custom AI Agent Integration
│   ├── RAG & Search
│   │   ├── Overview & Architecture
│   │   ├── Configuration Guide
│   │   ├── Embedding Models
│   │   ├── Hybrid Search Setup
│   │   └── Reranking & Compression
│   └── Advanced Topics
│       ├── Claude Bridge (Generate Agent Skills)
│       ├── Security & Sandboxing
│       ├── Performance Optimization
│       └── Multi-Environment Configs
│
├── API Reference                      [Priority: P1, Week 3-4]
│   ├── CLI Commands (24 commands)
│   ├── REST API (OpenAPI)
│   ├── MCP Protocol
│   └── Rust API (cargo doc)
│       ├── skill-runtime (CRITICAL)
│       ├── skill-cli
│       ├── skill-http
│       ├── skill-mcp
│       ├── skill-web
│       └── skill-context
│
├── Interactive Playground             [Priority: P2, Week 5-7]
│   ├── Live Editor (Monaco + WASM)
│   ├── Example Gallery
│   ├── Share & Export
│   └── Debugging Tools
│
├── Examples & Tutorials               [Priority: P2, Week 5-7]
│   ├── Example Skills
│   │   ├── WASM Skills (11 examples)
│   │   ├── Native Skills (5 examples)
│   │   └── Docker Skills (6 examples)
│   ├── Use Cases
│   │   ├── DevOps Automation
│   │   ├── Data Engineering Pipelines
│   │   ├── API Integration Hub
│   │   └── AI Agent Toolkit
│   └── Video Tutorials
│
├── Architecture                       [Priority: P2, Week 6-8]
│   ├── System Overview
│   ├── Component Model (WASM)
│   ├── Runtime Architecture
│   ├── Security Model
│   └── Contributing Guide
│
└── Resources                          [Priority: P3, Week 8-10]
    ├── FAQ
    ├── Troubleshooting
    ├── Migration Guides
    ├── Changelog
    └── Community & Support
```

### URL Structure
- Clean URLs without `.html` extensions
- Hierarchical structure matching navigation
- Examples:
  - `/getting-started/quick-start`
  - `/guides/building-skills/javascript`
  - `/api/rest` (interactive explorer)
  - `/playground`
  - `/examples/github-skill`

---

## Content Strategy & Migration Plan

### Content Types (Divio Framework)

#### 1. **Tutorials** (Learning-oriented)
- Goal: Take users by the hand through first experiences
- Examples:
  - "Your First Skill in 10 Minutes"
  - "Integrate Claude Code Step-by-Step"
  - "Build a GitHub Bot with Skill Engine"
- Format: Step-by-step, numbered instructions, expected outcomes
- Priority: **P0** (critical for onboarding)

#### 2. **How-To Guides** (Problem-oriented)
- Goal: Show how to solve specific problems
- Examples:
  - "How to Use OAuth2 Authentication"
  - "How to Deploy with Docker Compose"
  - "How to Configure Hybrid Search"
- Format: Clear steps, focused on one problem
- Priority: **P1** (high value after basic understanding)

#### 3. **Reference** (Information-oriented)
- Goal: Provide technical descriptions
- Examples:
  - CLI command reference (auto-generated)
  - REST API reference (OpenAPI)
  - Configuration schema reference
- Format: Dry, precise, comprehensive
- Priority: **P1** (developers need this constantly)

#### 4. **Explanation** (Understanding-oriented)
- Goal: Clarify and illuminate topics
- Examples:
  - "Why WASM for Skill Execution"
  - "Understanding the RAG Pipeline"
  - "Security Model Deep Dive"
- Format: Conceptual, architectural, discussion
- Priority: **P2** (valuable but not immediate blocker)

### Migration Plan for Existing Docs (5,200+ lines)

| Existing File | Lines | New Location | Action | Priority | Week |
|---------------|-------|--------------|--------|----------|------|
| `README.md` | 808 | `/` (home page) | Transform to landing page with hero, features, quick links | P0 | 1-2 |
| `docs/skill-development.md` | 523 | `/guides/building-skills/` | Split into JS, TS, Native sections | P0 | 1-2 |
| `docs/QUICK_START_CLAUDE_CODE.md` | 450 | `/getting-started/quick-start` + `/guides/integration/claude-code` | Enhance with screenshots, video | P0 | 1-2 |
| `docs/MANIFEST_GUIDE.md` | 1000+ | `/guides/configuration/` | Keep structure, add interactive examples | P1 | 3-4 |
| `docs/rag-search.md` | 485 | `/guides/rag-search/*` | **Split into 5 files**: overview, config, embeddings, hybrid, reranking | P1 | 3-4 |
| `docs/web-interface.md` | 412 | `/guides/web-ui` | Enhance with more screenshots, embed live demo | P2 | 5-6 |
| `docs/QUICK_START.md` | 380 | `/getting-started/installation` | Merge into installation guide | P0 | 1-2 |
| `docs/CLAUDE_CODE_INSTALLATION.md` | 320 | `/guides/integration/claude-code` | Combine with quick start | P0 | 1-2 |
| `docs/project-status.md` | 150 | `/resources/roadmap` | Update with latest status | P3 | 9-10 |
| `docs/design/*.md` | ~800 | `/architecture/*` | Complete and publish | P2 | 6-8 |
| Example skill READMEs | ~500 | `/examples/*` | Migrate with interactive demos | P2 | 5-7 |

**Total Migration**: 5,200+ lines → restructured, enhanced, and expanded

### New Content to Create (Estimated Lines)

| Topic | Est. Lines | Priority | Week |
|-------|-----------|----------|------|
| Claude Bridge Guide | 800 | P0 | 2-3 |
| Architecture Overview | 600 | P1 | 6-7 |
| REST API Reference (generated) | 1500 | P1 | 3-4 |
| MCP Protocol Spec | 500 | P1 | 3-4 |
| Deployment Guide (Docker/K8s) | 400 | P2 | 7-8 |
| Security Best Practices | 300 | P2 | 7-8 |
| Performance Tuning | 300 | P2 | 7-8 |
| FAQ & Troubleshooting | 400 | P3 | 9-10 |
| Video Tutorial Scripts | 600 | P3 | 9-10 |

**Total New Content**: ~5,400 lines

**Grand Total**: 10,600+ lines of documentation

---

## Interactive Features: Technical Specifications

### 1. Live WASM Playground

#### Overview
Browser-based code editor where users can write, test, and share Skill Engine skills without any local setup. Powered by Monaco Editor and skill-runtime compiled to WASM.

#### Architecture
```
┌─────────────────────────────────────────────────────┐
│ Monaco Editor (VS Code in browser)                  │
│ - TypeScript/JavaScript syntax highlighting         │
│ - Auto-completion with Skill API types              │
│ - Error checking and linting                        │
└──────────────┬──────────────────────────────────────┘
               │ User writes skill code
               ▼
┌─────────────────────────────────────────────────────┐
│ JCO Componentize (in WASM)                          │
│ - Compile JS → WASM Component                       │
│ - Runs entirely in browser via Emscripten           │
└──────────────┬──────────────────────────────────────┘
               │ Compiled .wasm component
               ▼
┌─────────────────────────────────────────────────────┐
│ skill-runtime.wasm (execution engine)               │
│ - Load WASM component                               │
│ - Execute with sandboxing                           │
│ - Return results to UI                              │
└──────────────┬──────────────────────────────────────┘
               │ Execution results
               ▼
┌─────────────────────────────────────────────────────┐
│ Output Panel                                         │
│ - JSON results formatted                            │
│ - Error messages with stack traces                  │
│ - Performance metrics (execution time, memory)      │
└─────────────────────────────────────────────────────┘
```

#### Implementation Details

**Technology Stack:**
- **Editor**: Monaco Editor (React component)
- **Compiler**: @bytecodealliance/jco (WASM build)
- **Runtime**: skill-runtime compiled to wasm32-unknown-unknown
- **State**: Zustand for editor state management
- **Sharing**: Base64 encode code → URL params → short URL service

**Example Component Structure:**
```typescript
// /docs-site/.vitepress/theme/components/Playground.vue
<template>
  <div class="playground">
    <div class="editor-container">
      <MonacoEditor
        v-model="code"
        language="javascript"
        :options="editorOptions"
        @change="onCodeChange"
      />
    </div>
    <div class="controls">
      <button @click="runSkill">Run</button>
      <button @click="shareSkill">Share</button>
      <button @click="exportSkill">Export</button>
      <select v-model="selectedExample">
        <option>Hello World</option>
        <option>GitHub Repo Lister</option>
        <option>AWS S3 Uploader</option>
      </select>
    </div>
    <div class="output">
      <pre>{{ output }}</pre>
      <div v-if="error" class="error">{{ error }}</div>
      <div class="metrics">
        Execution: {{ executionTime }}ms | Memory: {{ memoryUsed }}KB
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import MonacoEditor from 'monaco-editor-vue3'
import * as skillRuntime from '@skill-engine/wasm-runtime'

const code = ref(exampleCode)
const output = ref('')
const error = ref(null)
const executionTime = ref(0)

async function runSkill() {
  const start = performance.now()
  try {
    // Compile JS to WASM
    const component = await skillRuntime.compileSkill(code.value)

    // Execute tool
    const result = await skillRuntime.executeTool(component, 'greet', { name: 'World' })

    output.value = JSON.stringify(result, null, 2)
    executionTime.value = performance.now() - start
  } catch (e) {
    error.value = e.message
  }
}

function shareSkill() {
  const encoded = btoa(code.value)
  const url = `${window.location.origin}/playground?code=${encoded}`
  navigator.clipboard.writeText(url)
  // Show toast notification
}
</script>
```

**Pre-loaded Examples:**
1. **Hello World**: Basic greet tool (30 lines)
2. **HTTP Client**: Fetch GitHub user (60 lines)
3. **File Processor**: Parse JSON/CSV (80 lines)
4. **AWS S3**: List buckets (100 lines)
5. **Multi-Tool Skill**: GitHub repo + issue tracker (150 lines)

**Features:**
- Auto-save to localStorage (recover on page refresh)
- Syntax validation in real-time
- Import/export `.js` files
- Share via short URL
- Fork existing examples
- Dark/light theme toggle

**Performance Targets:**
- Editor load: < 1s
- Compilation: < 2s for 100-line skill
- Execution: < 100ms for simple skills
- Memory: < 50MB WASM heap

**Limitations & Caveats:**
- No network access from WASM (security limitation)
- File system simulated in-memory
- Max skill size: 10KB (prevents DoS)
- Execution timeout: 5 seconds

---

### 2. API Explorer with OpenAPI Spec

#### Overview
Interactive REST API documentation and testing interface (like Swagger UI) auto-generated from Rust code using `utoipa` annotations.

#### Architecture
```
┌─────────────────────────────────────────────────────┐
│ Rust HTTP Handlers (skill-http)                     │
│ #[utoipa::path(...)] annotations                    │
└──────────────┬──────────────────────────────────────┘
               │ cargo doc with utoipa
               ▼
┌─────────────────────────────────────────────────────┐
│ OpenAPI 3.1 Specification (openapi.json)            │
│ - All endpoints with request/response schemas       │
│ - Authentication methods                            │
│ - Example payloads                                  │
└──────────────┬──────────────────────────────────────┘
               │ Served at /api/openapi.json
               ▼
┌─────────────────────────────────────────────────────┐
│ Swagger UI / Redoc                                  │
│ - Interactive endpoint explorer                     │
│ - Try It Out functionality                          │
│ - Code generation (curl, JS, Python)               │
└─────────────────────────────────────────────────────┘
```

#### Implementation: utoipa Annotations

**Example Handler Annotation:**
```rust
// crates/skill-http/src/handlers.rs

use utoipa::{OpenApi, ToSchema};
use axum::{Json, extract::Path};

#[derive(ToSchema)]
pub struct SkillInfo {
    name: String,
    version: String,
    description: Option<String>,
    tools: Vec<ToolDefinition>,
}

#[utoipa::path(
    get,
    path = "/api/skills/{name}",
    tag = "Skills",
    summary = "Get skill information",
    description = "Retrieve detailed information about a specific skill including all available tools",
    params(
        ("name" = String, Path, description = "Skill name to retrieve")
    ),
    responses(
        (status = 200, description = "Skill found successfully", body = SkillInfo),
        (status = 404, description = "Skill not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_skill_info(
    Path(name): Path<String>,
) -> Result<Json<SkillInfo>, ErrorResponse> {
    // Handler implementation
}

// Generate OpenAPI spec
#[derive(OpenApi)]
#[openapi(
    paths(
        get_skill_info,
        list_skills,
        execute_skill_tool,
        // ... all other endpoints
    ),
    components(
        schemas(SkillInfo, ToolDefinition, ExecutionRequest, ExecutionResult)
    ),
    tags(
        (name = "Skills", description = "Skill management endpoints"),
        (name = "Execution", description = "Tool execution endpoints"),
        (name = "Search", description = "RAG search endpoints")
    )
)]
struct ApiDoc;
```

**REST Endpoints to Document (11 total):**

| Method | Path | Description | Priority |
|--------|------|-------------|----------|
| GET | `/api/skills` | List all skills (paginated) | P1 |
| GET | `/api/skills/{name}` | Get skill details | P1 |
| POST | `/api/skills/{name}/{tool}` | Execute tool | P1 |
| GET | `/api/skills/{name}/tools` | List skill tools | P1 |
| POST | `/api/search` | Semantic search | P1 |
| GET | `/api/health` | Health check | P1 |
| POST | `/api/auth/login` | Authentication | P2 |
| GET | `/api/services` | List required services | P2 |
| GET | `/api/services/{name}/status` | Service health | P2 |
| POST | `/api/skills/{name}/validate` | Validate config | P2 |
| GET | `/api/history` | Execution history | P3 |

**OpenAPI Spec Generation:**
```rust
// crates/skill-http/src/openapi.rs
pub fn generate_openapi_spec() -> String {
    ApiDoc::openapi().to_pretty_json().unwrap()
}

// Served at GET /api/openapi.json
pub async fn serve_openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
```

**Frontend Integration:**
```vue
<!-- /docs-site/api/rest.md -->
<script setup>
import SwaggerUI from 'swagger-ui'
import 'swagger-ui/dist/swagger-ui.css'

onMounted(() => {
  SwaggerUI({
    dom_id: '#swagger-ui',
    url: 'https://api.skill-engine.dev/api/openapi.json',
    tryItOutEnabled: true,
    persistAuthorization: true,
  })
})
</script>

<div id="swagger-ui"></div>
```

**Features:**
- Try It Out with user credentials
- Request/response examples in multiple languages
- Authentication with API keys or OAuth2
- Response schema validation
- Code generation (curl, JavaScript, Python, Rust)

---

### 3. Configuration Generator (.skill-engine.toml)

#### Overview
Visual form-based tool to build `.skill-engine.toml` manifest files with validation, examples, and TOML preview.

#### Architecture
```
┌─────────────────────────────────────────────────────┐
│ Rust Structs (manifest.rs)                          │
│ #[derive(JsonSchema)] annotations                   │
└──────────────┬──────────────────────────────────────┘
               │ cargo run --bin generate-schema
               ▼
┌─────────────────────────────────────────────────────┐
│ JSON Schema (.skill-engine.schema.json)             │
│ - All fields with types, descriptions, constraints  │
│ - Enum values (runtime types, source types)         │
│ - Required vs optional fields                       │
└──────────────┬──────────────────────────────────────┘
               │ Import schema into Vue form generator
               ▼
┌─────────────────────────────────────────────────────┐
│ Dynamic Vue Forms                                    │
│ - Auto-generated from JSON Schema                   │
│ - Conditional fields (e.g., show Docker config if   │
│   runtime = "docker")                               │
│ - Validation rules from schema                      │
└──────────────┬──────────────────────────────────────┘
               │ Form data
               ▼
┌─────────────────────────────────────────────────────┐
│ TOML Generator                                       │
│ - Convert form data to TOML structure               │
│ - Pretty-print with comments                        │
│ - Validate against schema                           │
└──────────────┬──────────────────────────────────────┘
               │ Generated .skill-engine.toml
               ▼
┌─────────────────────────────────────────────────────┐
│ Output Panel                                         │
│ - Live TOML preview                                 │
│ - Download button                                   │
│ - Copy to clipboard                                 │
│ - Validation errors (if any)                        │
└─────────────────────────────────────────────────────┘
```

#### Implementation: JSON Schema Generation

**Rust Schema Annotation:**
```rust
// crates/skill-runtime/src/manifest.rs

use schemars::{JsonSchema, schema_for};

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct SkillManifest {
    /// Version of the manifest format (currently "1")
    pub version: String,

    /// Global defaults for all skills
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defaults: Option<ManifestDefaults>,

    /// Skill definitions
    pub skills: HashMap<String, SkillDefinition>,
}

#[derive(JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillRuntime {
    /// WebAssembly Component Model (default)
    Wasm,
    /// Docker container runtime
    Docker,
    /// Native CLI wrapper
    Native,
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct SkillDefinition {
    /// Skill source: local path, git URL, or docker image
    ///
    /// Examples:
    /// - "./path/to/skill"
    /// - "github:user/repo@v1.0.0"
    /// - "docker:python:3.12-slim"
    pub source: String,

    /// Runtime type (defaults to wasm)
    #[serde(default)]
    pub runtime: SkillRuntime,

    /// Skill description
    pub description: Option<String>,

    /// Docker configuration (required if runtime = "docker")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docker: Option<DockerRuntimeConfig>,

    /// Instance configurations
    #[serde(default)]
    pub instances: HashMap<String, InstanceDefinition>,
}

// Generate schema
fn main() {
    let schema = schema_for!(SkillManifest);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    std::fs::write(".skill-engine.schema.json", json).unwrap();
}
```

**Vue Form Generator:**
```vue
<!-- /docs-site/.vitepress/theme/components/ConfigGenerator.vue -->
<template>
  <div class="config-generator">
    <div class="form-section">
      <h2>Add Skill</h2>

      <FormField
        v-model="skillName"
        label="Skill Name"
        :schema="schema.properties.skills.additionalProperties.properties.name"
      />

      <FormField
        v-model="skillSource"
        label="Source"
        :schema="schema.properties.skills.additionalProperties.properties.source"
        placeholder="./examples/native-skills/git-skill"
      />

      <FormSelect
        v-model="skillRuntime"
        label="Runtime"
        :options="['wasm', 'docker', 'native']"
        :schema="schema.properties.skills.additionalProperties.properties.runtime"
      />

      <!-- Conditional: Show Docker config if runtime = 'docker' -->
      <div v-if="skillRuntime === 'docker'" class="docker-config">
        <h3>Docker Configuration</h3>
        <FormField v-model="dockerImage" label="Image" />
        <FormField v-model="dockerEntrypoint" label="Entrypoint" />
        <FormField v-model="dockerMemory" label="Memory Limit" placeholder="512m" />
        <FormField v-model="dockerNetwork" label="Network" :options="['none', 'bridge', 'host']" />
      </div>

      <!-- Instance configuration -->
      <div class="instance-config">
        <h3>Instance: default</h3>
        <FormObject
          v-model="instanceConfig"
          label="Configuration"
          :schema="schema.properties.skills.additionalProperties.properties.instances"
        />
      </div>

      <button @click="addSkill">Add Skill</button>
    </div>

    <div class="preview-section">
      <h2>Generated TOML</h2>
      <CodeEditor
        v-model="generatedTOML"
        language="toml"
        readonly
      />
      <div class="actions">
        <button @click="downloadTOML">Download</button>
        <button @click="copyToClipboard">Copy</button>
        <button @click="validateTOML">Validate</button>
      </div>
      <div v-if="validationErrors.length" class="errors">
        <h3>Validation Errors:</h3>
        <ul>
          <li v-for="error in validationErrors">{{ error }}</li>
        </ul>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import TOML from '@iarna/toml'
import Ajv from 'ajv'
import schema from '../../../.skill-engine.schema.json'

const skills = ref([])
const skillName = ref('')
const skillSource = ref('')
const skillRuntime = ref('wasm')
const dockerImage = ref('')
const instanceConfig = ref({})

const generatedTOML = computed(() => {
  const manifest = {
    version: '1',
    skills: Object.fromEntries(
      skills.value.map(skill => [skill.name, {
        source: skill.source,
        runtime: skill.runtime,
        ...(skill.docker && { docker: skill.docker }),
        instances: { default: skill.instanceConfig }
      }])
    )
  }
  return TOML.stringify(manifest)
})

const validationErrors = ref([])

function validateTOML() {
  const ajv = new Ajv()
  const validate = ajv.compile(schema)
  const manifest = TOML.parse(generatedTOML.value)
  const valid = validate(manifest)
  if (!valid) {
    validationErrors.value = validate.errors.map(e => `${e.instancePath}: ${e.message}`)
  } else {
    validationErrors.value = []
  }
}

function addSkill() {
  skills.value.push({
    name: skillName.value,
    source: skillSource.value,
    runtime: skillRuntime.value,
    ...(skillRuntime.value === 'docker' && {
      docker: {
        image: dockerImage.value,
        entrypoint: dockerEntrypoint.value,
        memory: dockerMemory.value,
        network: dockerNetwork.value
      }
    }),
    instanceConfig: { ...instanceConfig.value }
  })
  // Reset form
  skillName.value = ''
  skillSource.value = ''
  skillRuntime.value = 'wasm'
  instanceConfig.value = {}
}
</script>
```

**Features:**
- Pre-filled templates (minimal, team, enterprise)
- Conditional fields based on runtime type
- Environment variable helper (suggests `${VAR:-default}` format)
- Validation with clear error messages
- Import existing TOML to edit
- Export to file or clipboard

---

### 4. RAG-Powered Search with Autocomplete

#### Overview
Leverage the existing RAG pipeline (fastembed, vector store, hybrid search) to provide semantic search with autocomplete for skills, tools, and documentation.

#### Architecture
```
┌─────────────────────────────────────────────────────┐
│ User Types Query                                     │
│ "deploy kubernetes pods"                            │
└──────────────┬──────────────────────────────────────┘
               │ Debounced input (300ms)
               ▼
┌─────────────────────────────────────────────────────┐
│ Query Processor                                      │
│ - Intent classification (tool_discovery)            │
│ - Entity extraction (kubernetes, deploy)            │
│ - Query expansion (synonyms)                        │
└──────────────┬──────────────────────────────────────┘
               │ Processed query
               ▼
┌─────────────────────────────────────────────────────┐
│ Hybrid Search (Dense + Sparse)                      │
│ - Dense: FastEmbed all-minilm embeddings           │
│ - Sparse: BM25 via Tantivy                         │
│ - Fusion: Reciprocal Rank Fusion (RRF)             │
└──────────────┬──────────────────────────────────────┘
               │ Top 100 candidates
               ▼
┌─────────────────────────────────────────────────────┐
│ Reranker (optional)                                 │
│ - BGE Reranker for precision scoring                │
│ - Final top-k selection                             │
└──────────────┬──────────────────────────────────────┘
               │ Top 5 results with scores
               ▼
┌─────────────────────────────────────────────────────┐
│ Autocomplete Dropdown                                │
│ - Skill name + tool name                            │
│ - Description snippet                               │
│ - Match score (0-100%)                              │
│ - Click to navigate                                 │
└─────────────────────────────────────────────────────┘
```

#### Implementation: Backend Search API

**Search Endpoint:**
```rust
// crates/skill-http/src/handlers/search.rs

use skill_runtime::search::{SearchPipeline, SearchRequest};

#[utoipa::path(
    post,
    path = "/api/search",
    tag = "Search",
    summary = "Semantic search for skills and tools",
    request_body = SearchRequest,
    responses(
        (status = 200, body = SearchResponse)
    )
)]
pub async fn search_skills(
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, ErrorResponse> {
    let pipeline = SearchPipeline::new()?;

    let results = pipeline
        .search(&request.query)
        .top_k(request.limit.unwrap_or(5))
        .with_reranking(request.rerank.unwrap_or(true))
        .execute()
        .await?;

    Ok(Json(SearchResponse {
        query: request.query,
        results: results.into_iter().map(|r| SearchResult {
            skill_name: r.skill_name,
            tool_name: r.tool_name,
            description: r.description,
            score: r.score,
            match_type: r.match_type, // dense, sparse, hybrid
        }).collect(),
        total: results.len(),
        execution_time_ms: 0, // TODO
    }))
}
```

**Frontend Search Component:**
```vue
<!-- /docs-site/.vitepress/theme/components/SearchBar.vue -->
<template>
  <div class="search-container">
    <input
      v-model="query"
      @input="onQueryChange"
      @keydown.down="selectNext"
      @keydown.up="selectPrevious"
      @keydown.enter="navigateToResult"
      placeholder="Search skills, tools, documentation..."
      class="search-input"
    />

    <div v-if="results.length" class="autocomplete-dropdown">
      <div
        v-for="(result, index) in results"
        :key="index"
        :class="['result-item', { selected: index === selectedIndex }]"
        @click="navigateTo(result)"
      >
        <div class="result-header">
          <span class="skill-name">{{ result.skill_name }}</span>
          <span class="tool-name">:{{ result.tool_name }}</span>
          <span class="score">{{ Math.round(result.score * 100) }}%</span>
        </div>
        <div class="result-description">{{ result.description }}</div>
        <div class="result-meta">
          <span class="badge">{{ result.match_type }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useDebounceFn } from '@vueuse/core'

const query = ref('')
const results = ref([])
const selectedIndex = ref(0)

const searchSkills = useDebounceFn(async (q: string) => {
  if (q.length < 2) {
    results.value = []
    return
  }

  const response = await fetch('/api/search', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      query: q,
      limit: 5,
      rerank: true
    })
  })

  const data = await response.json()
  results.value = data.results
  selectedIndex.value = 0
}, 300)

watch(query, searchSkills)

function navigateTo(result) {
  window.location.href = `/examples/${result.skill_name}#${result.tool_name}`
}
</script>
```

**Search Features:**
- **Autocomplete**: Show top 5 results as user types
- **Score Display**: Show match confidence (0-100%)
- **Match Type Badge**: Dense, Sparse, or Hybrid indicator
- **Keyboard Navigation**: Arrow keys + Enter
- **History**: Recent searches saved in localStorage
- **Filters**: By skill type (WASM, Native, Docker), category (DevOps, API, Data)
- **Advanced Search**: Dedicated page with more options (date range, author, tags)

**Algolia DocSearch Integration (Primary):**
For documentation pages (not skills), use Algolia DocSearch:
```javascript
// .vitepress/config.ts
export default {
  themeConfig: {
    search: {
      provider: 'algolia',
      options: {
        appId: 'SKILL_ENGINE_APP_ID',
        apiKey: 'SKILL_ENGINE_SEARCH_KEY',
        indexName: 'skill_engine_docs'
      }
    }
  }
}
```

**Custom RAG Search (Experimental):**
For skill/tool search, use custom RAG pipeline as shown above.

---

## API Documentation Strategy

### 1. Rust API Documentation (cargo doc)

#### Current State
- **Coverage**: ~30% (estimated from `grep -r "///" crates/ | wc -l`)
- **Missing**: Module-level docs, struct field descriptions, function examples

#### Target State
- **Coverage**: 90%+ across all public APIs
- **Format**: Rustdoc with examples, links, diagrams

#### Implementation Plan (5 weeks)

**Week 1: skill-runtime (CRITICAL)**
- **Priority**: HIGHEST (core execution engine)
- **Files** (~15 files, 400+ functions):
  - `lib.rs` - Module overview
  - `engine.rs` - Wasmtime setup
  - `executor.rs` - SkillExecutor
  - `sandbox.rs` - WASI sandboxing
  - `manifest.rs` - Configuration
  - `search/*.rs` - RAG pipeline (10 files)

**Example Documentation Template:**
```rust
/// Executes a tool within a skill's WASM component.
///
/// This function loads the specified skill component, sets up the sandbox
/// environment with the provided configuration, and executes the named tool
/// with the given arguments.
///
/// # Arguments
///
/// * `skill_path` - Path to the WASM component file
/// * `tool_name` - Name of the tool to execute
/// * `args` - JSON-serialized arguments for the tool
/// * `config` - Skill configuration and capabilities
///
/// # Returns
///
/// Returns a `Result` containing the tool execution result as a JSON string,
/// or an error if execution fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The skill component cannot be loaded
/// - The tool doesn't exist in the skill
/// - The arguments are invalid
/// - The tool execution fails or times out
///
/// # Examples
///
/// ```rust
/// use skill_runtime::{execute_tool, SkillConfig};
///
/// let config = SkillConfig::default();
/// let result = execute_tool(
///     "./examples/hello-skill.wasm",
///     "greet",
///     r#"{"name": "World"}"#,
///     &config
/// ).await?;
///
/// println!("Result: {}", result);
/// // Output: {"success": true, "output": "Hello, World!\n"}
/// ```
///
/// # Performance
///
/// - Cold start: ~100ms (includes AOT compilation)
/// - Warm start: <10ms (component cached)
/// - Memory: ~5MB per skill instance
///
/// # Security
///
/// Tools execute in a WASI Preview 2 sandbox with capability-based access:
/// - No network access by default
/// - Filesystem limited to pre-opened directories
/// - Memory limited to configured max
///
/// See [`SandboxBuilder`] for sandbox configuration.
pub async fn execute_tool(
    skill_path: &Path,
    tool_name: &str,
    args: &str,
    config: &SkillConfig,
) -> Result<String, ExecutionError> {
    // Implementation
}
```

**Week 2: skill-cli**
- **Priority**: HIGH (user-facing commands)
- **Files** (~25 command modules):
  - `main.rs` - CLI overview
  - `commands/*.rs` - Each command
  - Focus: Claude Bridge (currently ZERO docs)

**Week 3: skill-http + skill-mcp**
- **Priority**: HIGH (API integrations)
- **Files** (~10 files):
  - `server.rs` - HTTP server
  - `handlers.rs` - API endpoints (add OpenAPI too)
  - `mcp/server.rs` - MCP protocol

**Week 4: skill-web + skill-context**
- **Priority**: MEDIUM (internal APIs)
- **Files** (~20 files):
  - Web UI components
  - Context management utilities

**Week 5: Polish & CI Integration**
- Fix warnings from `cargo doc`
- Add `#![warn(missing_docs)]` to crate roots
- CI check: `cargo doc --no-deps --document-private-items`

**Documentation Guidelines:**
- Every public item needs docs (`pub fn`, `pub struct`, `pub mod`)
- Include examples for complex functions
- Link to related types/functions with `[`Type`]` syntax
- Use `# Examples`, `# Errors`, `# Panics`, `# Safety` sections
- Add module-level overview (`//!` at top of file)

**Automation:**
```bash
# Generate docs
cargo doc --no-deps --document-private-items --open

# Check coverage
cargo doc 2>&1 | grep warning | wc -l

# CI enforcement
if [ $(cargo doc 2>&1 | grep warning | wc -l) -gt 10 ]; then
  echo "Too many documentation warnings"
  exit 1
fi
```

---

### 2. REST API Documentation (OpenAPI 3.1)

#### Implementation with utoipa

**Dependencies:**
```toml
# crates/skill-http/Cargo.toml
[dependencies]
utoipa = { version = "5.0", features = ["axum_extras", "chrono"] }
utoipa-swagger-ui = { version = "8.0", features = ["axum"] }
```

**Example Annotated Handler:**
```rust
// crates/skill-http/src/handlers.rs

use utoipa::{OpenApi, ToSchema, IntoParams};
use axum::{Json, extract::{Path, Query}};

/// Request to execute a skill tool
#[derive(ToSchema, Deserialize)]
#[schema(example = json!({
    "args": {
        "resource": "pods",
        "namespace": "default"
    }
}))]
pub struct ExecutionRequest {
    /// Tool arguments as key-value pairs
    pub args: HashMap<String, String>,
}

/// Successful tool execution result
#[derive(ToSchema, Serialize)]
pub struct ExecutionResult {
    /// Whether the tool executed successfully
    pub success: bool,

    /// Tool output (stdout/result)
    pub output: String,

    /// Error message if execution failed
    pub error_message: Option<String>,

    /// Execution metrics
    pub metrics: ExecutionMetrics,
}

#[derive(ToSchema, Serialize)]
pub struct ExecutionMetrics {
    /// Execution time in milliseconds
    pub duration_ms: u64,

    /// Memory used in bytes
    pub memory_bytes: u64,
}

#[utoipa::path(
    post,
    path = "/api/skills/{skill}/{tool}",
    tag = "Execution",
    summary = "Execute a skill tool",
    description = "Execute a specific tool from a skill with provided arguments. \
                   The skill must be loaded and the tool must exist.",
    params(
        ("skill" = String, Path, description = "Skill name (e.g., 'kubernetes')"),
        ("tool" = String, Path, description = "Tool name (e.g., 'get')")
    ),
    request_body(
        content = ExecutionRequest,
        description = "Tool arguments and configuration"
    ),
    responses(
        (
            status = 200,
            description = "Tool executed successfully",
            body = ExecutionResult,
            example = json!({
                "success": true,
                "output": "NAME      READY   STATUS    RESTARTS   AGE\nfrontend   1/1    Running   0          5d",
                "error_message": null,
                "metrics": {
                    "duration_ms": 45,
                    "memory_bytes": 2048000
                }
            })
        ),
        (
            status = 400,
            description = "Invalid arguments or tool doesn't exist",
            body = ErrorResponse
        ),
        (
            status = 404,
            description = "Skill not found",
            body = ErrorResponse
        ),
        (
            status = 500,
            description = "Execution failed",
            body = ErrorResponse
        )
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn execute_tool(
    Path((skill, tool)): Path<(String, String)>,
    Json(request): Json<ExecutionRequest>,
) -> Result<Json<ExecutionResult>, ErrorResponse> {
    // Implementation
}
```

**OpenAPI Spec Generation:**
```rust
// crates/skill-http/src/openapi.rs

#[derive(OpenApi)]
#[openapi(
    paths(
        // Skills management
        handlers::list_skills,
        handlers::get_skill_info,
        handlers::get_skill_tools,
        handlers::validate_skill_config,

        // Tool execution
        handlers::execute_tool,
        handlers::get_execution_history,

        // Search
        handlers::search_skills,

        // Services
        handlers::list_services,
        handlers::get_service_status,

        // Health
        handlers::health_check,
    ),
    components(
        schemas(
            SkillInfo,
            ToolDefinition,
            ExecutionRequest,
            ExecutionResult,
            ExecutionMetrics,
            SearchRequest,
            SearchResponse,
            ServiceStatus,
            ErrorResponse,
        )
    ),
    tags(
        (name = "Skills", description = "Skill management and discovery"),
        (name = "Execution", description = "Tool execution endpoints"),
        (name = "Search", description = "Semantic search with RAG"),
        (name = "Services", description = "External service management"),
        (name = "Health", description = "Health and status checks"),
    ),
    info(
        title = "Skill Engine REST API",
        version = "0.3.0",
        description = "Universal runtime system for AI agents - REST API for skill management, \
                      tool execution, and semantic search.",
        contact(
            name = "Skill Engine Team",
            url = "https://github.com/kubiyabot/skill",
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT",
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development"),
        (url = "https://api.skill-engine.dev", description = "Production API"),
    )
)]
pub struct ApiDoc;

// Serve OpenAPI JSON
pub async fn serve_openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

// Serve Swagger UI
pub fn swagger_ui_router() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/docs/api").url("/api/openapi.json", ApiDoc::openapi()))
}
```

**Integration with Axum:**
```rust
// crates/skill-http/src/server.rs

use crate::openapi::{serve_openapi_spec, swagger_ui_router};

pub async fn start_server(port: u16) -> Result<()> {
    let app = Router::new()
        // API endpoints
        .route("/api/skills", get(handlers::list_skills))
        .route("/api/skills/:name", get(handlers::get_skill_info))
        .route("/api/skills/:name/:tool", post(handlers::execute_tool))
        // ... other routes

        // OpenAPI documentation
        .route("/api/openapi.json", get(serve_openapi_spec))
        .merge(swagger_ui_router()); // Swagger UI at /docs/api

    // Start server
    axum::Server::bind(&format!("127.0.0.1:{}", port).parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

**Result:**
- OpenAPI 3.1 spec at `/api/openapi.json`
- Interactive Swagger UI at `/docs/api`
- Embedded in VitePress docs site at `/api/rest`

---

### 3. MCP Protocol Documentation

#### Overview
Model Context Protocol (MCP) is used by Claude Code, Cursor, and other AI tools to discover and execute skills.

#### Documentation Structure

**Page**: `/guides/integration/mcp-protocol.md`

**Content:**

```markdown
# MCP Protocol Reference

## Overview

Skill Engine implements the [Model Context Protocol](https://modelcontextprotocol.io/) to expose skills as tools that AI agents can discover and execute.

## Connection Setup

### stdio Transport (Default)

Claude Code and most MCP clients use stdin/stdout:

\`\`\`bash
# Start MCP server
skill serve

# Or with custom manifest
skill serve --manifest /path/to/.skill-engine.toml
\`\`\`

Configuration in `.mcp.json`:
\`\`\`json
{
  "mcpServers": {
    "skill-engine": {
      "type": "stdio",
      "command": "skill",
      "args": ["serve"]
    }
  }
}
\`\`\`

### HTTP Transport

For remote connections:

\`\`\`bash
skill serve --http --port 3000
\`\`\`

Configuration:
\`\`\`json
{
  "mcpServers": {
    "skill-engine": {
      "type": "http",
      "url": "http://localhost:3000/mcp"
    }
  }
}
\`\`\`

## Protocol Messages

MCP uses JSON-RPC 2.0 over stdio or HTTP.

### Initialize

**Request:**
\`\`\`json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "0.1.0",
    "capabilities": {
      "tools": {}
    },
    "clientInfo": {
      "name": "claude-code",
      "version": "1.0.0"
    }
  }
}
\`\`\`

**Response:**
\`\`\`json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "0.1.0",
    "capabilities": {
      "tools": {
        "listChanged": true
      }
    },
    "serverInfo": {
      "name": "skill-engine",
      "version": "0.3.0"
    }
  }
}
\`\`\`

### List Tools

Discover available skills and tools:

**Request:**
\`\`\`json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list",
  "params": {
    "cursor": null
  }
}
\`\`\`

**Response:**
\`\`\`json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "execute",
        "description": "Execute a skill tool with provided arguments",
        "inputSchema": {
          "type": "object",
          "properties": {
            "skill": {
              "type": "string",
              "description": "Skill name (e.g., 'kubernetes')"
            },
            "tool": {
              "type": "string",
              "description": "Tool name (e.g., 'get')"
            },
            "args": {
              "type": "object",
              "description": "Tool arguments as key-value pairs"
            }
          },
          "required": ["skill", "tool", "args"]
        }
      },
      {
        "name": "list_skills",
        "description": "List all available skills with pagination",
        "inputSchema": {
          "type": "object",
          "properties": {
            "offset": { "type": "integer", "default": 0 },
            "limit": { "type": "integer", "default": 10 },
            "skill": { "type": "string", "description": "Filter by skill name" }
          }
        }
      },
      {
        "name": "search_skills",
        "description": "Semantic search for skills and tools using RAG",
        "inputSchema": {
          "type": "object",
          "properties": {
            "query": { "type": "string", "description": "Natural language query" },
            "top_k": { "type": "integer", "default": 5 }
          },
          "required": ["query"]
        }
      }
    ],
    "nextCursor": null
  }
}
\`\`\`

### Call Tool

Execute a skill tool:

**Request:**
\`\`\`json
{
  "jsonrpc": "2.0",
  "id": 3,
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
\`\`\`

**Response:**
\`\`\`json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "NAME      READY   STATUS    RESTARTS   AGE\\nfrontend   1/1    Running   0          5d\\nbackend    1/1    Running   0          5d"
      }
    ],
    "isError": false
  }
}
\`\`\`

### Error Handling

**Request:**
\`\`\`json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "execute",
    "arguments": {
      "skill": "nonexistent",
      "tool": "foo",
      "args": {}
    }
  }
}
\`\`\`

**Response:**
\`\`\`json
{
  "jsonrpc": "2.0",
  "id": 4,
  "error": {
    "code": -32000,
    "message": "Skill not found: nonexistent",
    "data": {
      "available_skills": ["kubernetes", "docker", "github"]
    }
  }
}
\`\`\`

## MCP Tools Catalog

### execute
Execute any skill tool with provided arguments.

**Parameters:**
- `skill` (string, required): Skill name
- `tool` (string, required): Tool name
- `args` (object, required): Tool-specific arguments

**Example:**
\`\`\`javascript
await callTool("execute", {
  skill: "github",
  tool: "list-repos",
  args: { org: "kubernetes" }
})
\`\`\`

### list_skills
List all available skills with optional filtering and pagination.

**Parameters:**
- `offset` (integer, optional): Starting index (default: 0)
- `limit` (integer, optional): Max results (default: 10)
- `skill` (string, optional): Filter by skill name

**Example:**
\`\`\`javascript
await callTool("list_skills", {
  limit: 5,
  skill: "kubernetes"
})
\`\`\`

### search_skills
Semantic search using RAG pipeline.

**Parameters:**
- `query` (string, required): Natural language query
- `top_k` (integer, optional): Number of results (default: 5)

**Example:**
\`\`\`javascript
await callTool("search_skills", {
  query: "deploy application to kubernetes",
  top_k: 3
})
\`\`\`

## Integration Examples

### Claude Code

**Setup:**
\`\`\`bash
# Automatic setup
skill claude setup

# Manual setup - add to .mcp.json
{
  "mcpServers": {
    "skill-engine": {
      "type": "stdio",
      "command": "skill",
      "args": ["serve"]
    }
  }
}
\`\`\`

**Usage:**
\`\`\`
claude> List all kubernetes pods in the default namespace

# Claude will use MCP to:
# 1. Call list_skills to discover kubernetes skill
# 2. Call execute with skill=kubernetes, tool=get, args={resource: "pods"}
# 3. Return formatted results
\`\`\`

### Cursor

**Setup:**
In Cursor settings, add MCP server:
\`\`\`json
{
  "mcp": {
    "servers": {
      "skill-engine": {
        "command": "skill",
        "args": ["serve"]
      }
    }
  }
}
\`\`\`

### Custom Python Client

\`\`\`python
import asyncio
import json
from mcp.client import ClientSession, StdioServerParameters

async def main():
    # Connect to Skill Engine MCP server
    server = StdioServerParameters(
        command="skill",
        args=["serve"]
    )

    async with ClientSession(server) as session:
        # Initialize
        await session.initialize()

        # List available tools
        tools = await session.list_tools()
        print(f"Available tools: {[t.name for t in tools]}")

        # Execute a skill
        result = await session.call_tool("execute", {
            "skill": "github",
            "tool": "list-repos",
            "args": {"org": "kubernetes"}
        })

        print(result.content[0].text)

asyncio.run(main())
\`\`\`

## Performance

- **Cold start**: ~100ms (MCP handshake + skill load)
- **Tool execution**: <500ms for most tools
- **Search**: <200ms (hybrid search + reranking)

## Security

- Skills execute in sandbox (WASI Preview 2)
- No access to host system by default
- Credentials managed via keyring, never exposed in MCP messages
- Audit logging of all tool executions

## Troubleshooting

### MCP Server Not Starting

\`\`\`bash
# Check if skill binary exists
which skill

# Test server manually
skill serve
# Should output: "MCP server ready - waiting for connections..."

# Check manifest loads
skill list
\`\`\`

### Claude Can't Discover Tools

1. Check `.mcp.json` configuration
2. Restart Claude Code
3. Run `skill claude status` to verify connection

### Tool Execution Fails

- Check skill exists: `skill list`
- Verify tool name: `skill info <skill-name>`
- Test directly: `skill run <skill>:<tool> arg=value`
\`\`\`

---

### 4. CLI Reference Documentation

#### Auto-Generation with clap-mangen

**Implementation:**
```rust
// crates/skill-cli/src/main.rs

use clap::{Parser, Subcommand};
use clap_mangen::Man;

#[derive(Parser)]
#[command(name = "skill")]
#[command(about = "Skill Engine CLI - Universal runtime for AI agents", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List installed skills and those from manifest
    #[command(about = "List all available skills with details")]
    List {
        /// Show only skills from manifest
        #[arg(long)]
        from_manifest: bool,

        /// Output format: table, json, yaml
        #[arg(short, long, default_value = "table")]
        output: String,
    },

    /// Run a skill tool with arguments
    #[command(about = "Execute a skill tool", long_about = "\
        Execute a tool from a skill with provided arguments.\n\n\
        Examples:\n  \
        skill run kubernetes:get resource=pods\n  \
        skill run ./my-skill greet name=World\n  \
        skill run github:list-repos org=kubernetes")]
    Run {
        /// Skill and tool in format 'skill:tool' or path to skill
        #[arg(value_name = "SKILL[:TOOL]")]
        skill: String,

        /// Tool arguments as key=value pairs
        #[arg(value_name = "ARG=VALUE")]
        args: Vec<String>,

        /// Use specific instance
        #[arg(long)]
        instance: Option<String>,
    },

    // ... 22 more commands
}

// Generate man pages
fn generate_man_pages() {
    let cmd = Cli::command();

    // Main page
    let man = Man::new(cmd.clone());
    let mut buffer = Vec::new();
    man.render(&mut buffer).unwrap();
    std::fs::write("/tmp/skill.1", buffer).unwrap();

    // Subcommand pages
    for subcmd in cmd.get_subcommands() {
        let man = Man::new(subcmd.clone());
        let mut buffer = Vec::new();
        man.render(&mut buffer).unwrap();
        std::fs::write(format!("/tmp/skill-{}.1", subcmd.get_name()), buffer).unwrap();
    }
}
```

**Build Integration:**
```toml
# Cargo.toml
[[bin]]
name = "generate-man-pages"
path = "src/bin/generate_man_pages.rs"

[build-dependencies]
clap = { version = "4.5", features = ["derive"] }
clap_mangen = "0.2"
```

```bash
# Generate during docs build
cargo run --bin generate-man-pages
# Output: skill.1, skill-list.1, skill-run.1, ... (25 man pages)

# Convert to markdown for docs site
for f in /tmp/skill*.1; do
  pandoc -f man -t markdown $f -o docs-site/cli/$(basename $f .1).md
done
```

**VitePress Integration:**
```markdown
<!-- docs-site/cli/run.md -->
---
title: skill run
description: Execute a skill tool with arguments
---

# skill run

Execute a tool from a skill with provided arguments.

## Synopsis

\`\`\`bash
skill run <SKILL[:TOOL]> [ARG=VALUE]... [OPTIONS]
\`\`\`

## Description

The `run` command executes a specific tool from a skill. Skills can be referenced by name (if installed or in manifest) or by path (for local development).

## Arguments

- `SKILL[:TOOL]` - Skill and tool name, or path to skill directory
  - Format: `skill-name:tool-name`
  - Example: `kubernetes:get`
  - Or: `./my-skill` (uses default tool)

- `ARG=VALUE` - Tool arguments as key-value pairs
  - Example: `resource=pods namespace=default`

## Options

- `--instance <INSTANCE>` - Use specific instance configuration
  - Example: `--instance prod`

- `--output <FORMAT>` - Output format: `text`, `json`, `yaml`
  - Default: `text`

- `-h, --help` - Print help
- `-V, --version` - Print version

## Examples

### Basic Usage

\`\`\`bash
# Run kubernetes get tool
skill run kubernetes:get resource=pods

# Run with specific namespace
skill run kubernetes:get resource=pods namespace=kube-system

# Run with specific instance
skill run kubernetes:prod get resource=deployments
\`\`\`

### Local Development

\`\`\`bash
# Run local skill (auto-compiles)
skill run ./my-skill greet name=Alice

# Run TypeScript skill
skill run ./my-skill.ts greet name=Bob
\`\`\`

### JSON Output

\`\`\`bash
skill run github:list-repos org=kubernetes --output json | jq '.results[].name'
\`\`\`

## Exit Status

- `0` - Success
- `1` - Skill or tool not found
- `2` - Invalid arguments
- `3` - Execution failed
- `4` - Timeout

## See Also

- [`skill list`](./list.md) - List available skills
- [`skill info`](./info.md) - Show skill details
- [`skill exec`](./exec.md) - Pass-through execution
\`\`\`

---

## Design System & VitePress Theme

### VitePress Configuration

**File**: `docs-site/.vitepress/config.ts`

```typescript
import { defineConfig } from 'vitepress'
import { SearchPlugin } from 'vitepress-plugin-search'

export default defineConfig({
  title: 'Skill Engine',
  description: 'Universal runtime for AI agents - WASM sandboxing, RAG search, MCP protocol',

  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }],
    ['meta', { name: 'theme-color', content: '#646cff' }],
    ['meta', { name: 'og:type', content: 'website' }],
    ['meta', { name: 'og:locale', content: 'en' }],
    ['meta', { name: 'og:site_name', content: 'Skill Engine' }],
    // Analytics
    ['script', { async: '', src: 'https://www.googletagmanager.com/gtag/js?id=G-XXXXXXXXXX' }],
  ],

  themeConfig: {
    logo: '/logo.svg',
    siteTitle: 'Skill Engine',

    nav: [
      { text: 'Guide', link: '/getting-started/quick-start' },
      { text: 'API', link: '/api/cli' },
      { text: 'Playground', link: '/playground' },
      { text: 'Examples', link: '/examples/' },
      {
        text: 'v0.3.0',
        items: [
          { text: 'Changelog', link: '/resources/changelog' },
          { text: 'Roadmap', link: '/resources/roadmap' },
        ]
      }
    ],

    sidebar: {
      '/getting-started/': [
        {
          text: 'Introduction',
          items: [
            { text: 'What is Skill Engine?', link: '/getting-started/' },
            { text: 'Quick Start', link: '/getting-started/quick-start' },
            { text: 'Installation', link: '/getting-started/installation' },
            { text: 'Core Concepts', link: '/getting-started/concepts' },
          ]
        },
        {
          text: 'First Steps',
          items: [
            { text: 'Your First Skill', link: '/getting-started/first-skill' },
            { text: 'Understanding Runtimes', link: '/getting-started/runtimes' },
          ]
        }
      ],

      '/guides/': [
        {
          text: 'Building Skills',
          items: [
            { text: 'JavaScript Skills', link: '/guides/building-skills/javascript' },
            { text: 'TypeScript Skills', link: '/guides/building-skills/typescript' },
            { text: 'Native Skills', link: '/guides/building-skills/native' },
            { text: 'Docker Skills', link: '/guides/building-skills/docker' },
            { text: 'Testing Your Skills', link: '/guides/building-skills/testing' },
          ]
        },
        {
          text: 'Integration',
          items: [
            { text: 'Claude Code Setup', link: '/guides/integration/claude-code' },
            { text: 'Cursor Integration', link: '/guides/integration/cursor' },
            { text: 'MCP Protocol', link: '/guides/integration/mcp-protocol' },
            { text: 'Custom Agents', link: '/guides/integration/custom-agents' },
          ]
        },
        {
          text: 'RAG & Search',
          items: [
            { text: 'Overview', link: '/guides/rag-search/' },
            { text: 'Configuration', link: '/guides/rag-search/configuration' },
            { text: 'Embedding Models', link: '/guides/rag-search/embeddings' },
            { text: 'Hybrid Search', link: '/guides/rag-search/hybrid' },
            { text: 'Reranking', link: '/guides/rag-search/reranking' },
          ]
        },
        {
          text: 'Advanced',
          items: [
            { text: 'Claude Bridge', link: '/guides/advanced/claude-bridge' },
            { text: 'Security & Sandboxing', link: '/guides/advanced/security' },
            { text: 'Performance Tuning', link: '/guides/advanced/performance' },
            { text: 'Multi-Environment', link: '/guides/advanced/multi-environment' },
          ]
        }
      ],

      '/api/': [
        {
          text: 'Reference',
          items: [
            { text: 'CLI Commands', link: '/api/cli' },
            { text: 'REST API', link: '/api/rest' },
            { text: 'MCP Protocol', link: '/api/mcp' },
            { text: 'Rust API', link: '/api/rust' },
          ]
        }
      ],

      '/examples/': [
        {
          text: 'Examples',
          items: [
            { text: 'Overview', link: '/examples/' },
            { text: 'WASM Skills', link: '/examples/wasm' },
            { text: 'Native Skills', link: '/examples/native' },
            { text: 'Docker Skills', link: '/examples/docker' },
          ]
        },
        {
          text: 'Use Cases',
          items: [
            { text: 'DevOps Automation', link: '/examples/use-cases/devops' },
            { text: 'Data Engineering', link: '/examples/use-cases/data' },
            { text: 'API Integration', link: '/examples/use-cases/api' },
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/kubiyabot/skill' },
      { icon: 'discord', link: 'https://discord.gg/skill-engine' },
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2024-present Kubiya'
    },

    search: {
      provider: 'algolia',
      options: {
        appId: 'SKILL_ENGINE_APP_ID',
        apiKey: 'SKILL_ENGINE_SEARCH_KEY',
        indexName: 'skill_engine_docs'
      }
    },

    editLink: {
      pattern: 'https://github.com/kubiyabot/skill/edit/main/docs-site/:path',
      text: 'Edit this page on GitHub'
    },

    lastUpdated: {
      text: 'Updated at',
      formatOptions: {
        dateStyle: 'full',
        timeStyle: 'medium'
      }
    }
  },

  markdown: {
    theme: {
      light: 'github-light',
      dark: 'github-dark'
    },
    lineNumbers: true,
    config: (md) => {
      // Custom markdown plugins
    }
  },

  vite: {
    plugins: [
      SearchPlugin({
        // Custom RAG search integration
        async search(query) {
          const response = await fetch('/api/search', {
            method: 'POST',
            body: JSON.stringify({ query, top_k: 5 })
          })
          return response.json()
        }
      })
    ]
  }
})
```

### Custom Theme Components

**Color Palette:**
```css
/* docs-site/.vitepress/theme/custom.css */
:root {
  /* Primary brand color - vibrant blue */
  --vp-c-brand-1: #646cff;
  --vp-c-brand-2: #747bff;
  --vp-c-brand-3: #535bf2;

  /* Accent - complementary orange for CTAs */
  --vp-c-accent-1: #ff7b00;
  --vp-c-accent-2: #ff8c1a;

  /* Backgrounds */
  --vp-c-bg: #ffffff;
  --vp-c-bg-soft: #f6f6f7;
  --vp-c-bg-mute: #f1f1f3;

  /* Text */
  --vp-c-text-1: #213547;
  --vp-c-text-2: #475569;
  --vp-c-text-3: #64748b;

  /* Code blocks */
  --vp-code-bg: #f8f9fa;
  --vp-code-color: #213547;
}

.dark {
  --vp-c-bg: #1a1a1a;
  --vp-c-bg-soft: #242424;
  --vp-c-bg-mute: #2f2f2f;
  --vp-c-text-1: #f1f1f3;
  --vp-c-text-2: #c9d1d9;
  --vp-c-text-3: #8b949e;
  --vp-code-bg: #161618;
}

/* Custom components */
.playground-container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  margin: 2rem 0;
}

.api-explorer {
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  padding: 1rem;
}

.feature-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 2rem;
  margin: 3rem 0;
}

.feature-card {
  padding: 2rem;
  background: var(--vp-c-bg-soft);
  border-radius: 12px;
  transition: transform 0.2s;
}

.feature-card:hover {
  transform: translateY(-4px);
}
```

**Vue Components:**
```typescript
// docs-site/.vitepress/theme/index.ts
import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'
import Playground from './components/Playground.vue'
import APIExplorer from './components/APIExplorer.vue'
import ConfigGenerator from './components/ConfigGenerator.vue'
import SearchBar from './components/SearchBar.vue'
import './custom.css'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    // Register global components
    app.component('Playground', Playground)
    app.component('APIExplorer', APIExplorer)
    app.component('ConfigGenerator', ConfigGenerator)
    app.component('SearchBar', SearchBar)
  }
} satisfies Theme
```

**Typography:**
- **Headings**: Inter (sans-serif, weight 600-700)
- **Body**: Inter (weight 400-500)
- **Code**: JetBrains Mono (monospace)
- **Base Size**: 16px
- **Line Height**: 1.6 for body, 1.2 for headings

**Code Highlighting:**
- **Languages**: JavaScript, TypeScript, Rust, Bash, TOML, JSON, YAML
- **Theme**: GitHub Light/Dark
- **Features**: Line numbers, diff highlighting, focused lines

---

## Technical Implementation Details

### Directory Structure

```
docs-site/
├── .vitepress/
│   ├── config.ts                  # VitePress configuration
│   ├── theme/
│   │   ├── index.ts              # Theme entry point
│   │   ├── custom.css            # Custom styles
│   │   └── components/
│   │       ├── Playground.vue
│   │       ├── APIExplorer.vue
│   │       ├── ConfigGenerator.vue
│   │       └── SearchBar.vue
│   └── dist/                      # Built site (gitignored)
├── public/
│   ├── favicon.ico
│   ├── logo.svg
│   ├── og-image.png
│   └── wasm/
│       └── skill-runtime.wasm     # Compiled runtime for playground
├── getting-started/
│   ├── index.md
│   ├── quick-start.md
│   ├── installation.md
│   ├── concepts.md
│   ├── first-skill.md
│   └── runtimes.md
├── guides/
│   ├── building-skills/
│   ├── integration/
│   ├── rag-search/
│   └── advanced/
├── api/
│   ├── cli/
│   ├── rest.md
│   ├── mcp.md
│   └── rust.md
├── playground/
│   └── index.md                   # Playground page
├── examples/
│   ├── index.md
│   ├── wasm/
│   ├── native/
│   ├── docker/
│   └── use-cases/
├── architecture/
│   ├── index.md
│   ├── system-overview.md
│   ├── component-model.md
│   ├── runtime.md
│   └── security.md
├── resources/
│   ├── faq.md
│   ├── troubleshooting.md
│   ├── changelog.md
│   └── roadmap.md
├── package.json
├── tsconfig.json
└── README.md
```

### Build Process

**package.json:**
```json
{
  "name": "skill-engine-docs",
  "version": "0.3.0",
  "type": "module",
  "scripts": {
    "dev": "vitepress dev",
    "build": "vitepress build",
    "preview": "vitepress preview",
    "generate-api-docs": "cargo doc --no-deps && cargo run --bin generate-man-pages"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "@bytecodealliance/jco": "^1.0.0",
    "monaco-editor": "^0.45.0",
    "monaco-editor-vue3": "^1.0.0",
    "swagger-ui": "^5.10.0",
    "vitepress": "^1.0.0",
    "vue": "^3.4.0",
    "@iarna/toml": "^2.2.5",
    "ajv": "^8.12.0"
  }
}
```

**GitHub Actions Workflow:**
```yaml
# .github/workflows/docs.yml
name: Deploy Documentation

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20

      # Generate Rust API docs
      - name: Generate Rust docs
        run: |
          cargo doc --no-deps --document-private-items
          mkdir -p docs-site/public/api/rust
          cp -r target/doc/* docs-site/public/api/rust/

      # Generate CLI man pages → markdown
      - name: Generate CLI docs
        run: |
          cargo run --bin generate-man-pages
          ./scripts/man-to-markdown.sh

      # Generate OpenAPI spec
      - name: Generate OpenAPI spec
        run: |
          cargo build --release --bin skill
          ./target/release/skill serve --http --port 3000 &
          sleep 5
          curl http://localhost:3000/api/openapi.json > docs-site/public/api/openapi.json

      # Build WASM playground
      - name: Build skill-runtime for WASM
        run: |
          cargo build --release --target wasm32-unknown-unknown
          wasm-opt -Oz target/wasm32-unknown-unknown/release/skill_runtime.wasm \
            -o docs-site/public/wasm/skill-runtime.wasm

      # Install docs dependencies
      - name: Install dependencies
        run: cd docs-site && npm ci

      # Build VitePress site
      - name: Build docs
        run: cd docs-site && npm run build

      # Deploy to GitHub Pages
      - name: Deploy to GitHub Pages
        if: github.ref == 'refs/heads/main'
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: docs-site/.vitepress/dist
          cname: docs.skill-engine.dev
```

### Performance Optimization

**Build Optimizations:**
- **Code Splitting**: Separate chunks for each major section
- **Lazy Loading**: Interactive components load on demand
- **Image Optimization**: WebP with fallbacks, responsive images
- **CSS Minification**: PostCSS with cssnano
- **WASM Compression**: wasm-opt -Oz reduces size by 50%

**Target Metrics:**
- **Lighthouse Score**: 95+ across all categories
- **First Contentful Paint (FCP)**: < 1.5s
- **Largest Contentful Paint (LCP)**: < 2.5s
- **Time to Interactive (TTI)**: < 3.5s
- **Total Blocking Time (TBT)**: < 300ms

**Caching Strategy:**
- **HTML**: No cache (always fresh)
- **JS/CSS**: Cache with content hash in filename (immutable)
- **WASM**: Cache with version hash (immutable)
- **Images**: Cache for 1 year
- **API responses**: Cache for 5 minutes

---

## Search & Discovery Integration

### Algolia DocSearch (Primary)

**Setup:**
```javascript
// Apply for DocSearch: https://docsearch.algolia.com/apply/

// .vitepress/config.ts
export default {
  themeConfig: {
    search: {
      provider: 'algolia',
      options: {
        appId: 'SKILL_ENGINE_APP_ID',
        apiKey: 'SKILL_ENGINE_SEARCH_KEY',
        indexName: 'skill_engine_docs',
        searchParameters: {
          facetFilters: ['version:v0.3'],
        }
      }
    }
  }
}
```

**Crawler Configuration:**
```json
{
  "index_name": "skill_engine_docs",
  "start_urls": [
    "https://docs.skill-engine.dev/"
  ],
  "selectors": {
    "lvl0": ".sidebar-heading.active",
    "lvl1": ".content h1",
    "lvl2": ".content h2",
    "lvl3": ".content h3",
    "lvl4": ".content h4",
    "text": ".content p, .content li"
  }
}
```

**Features:**
- Instant search with <100ms latency
- Keyboard shortcuts (Cmd+K)
- Search history
- Typo tolerance
- Synonyms (e.g., "CLI" → "command line")

### Custom RAG Search (Experimental)

**For Skill/Tool Discovery:**
```vue
<!-- SearchBar.vue with RAG integration -->
<script setup>
import { ref } from 'vue'

const query = ref('')
const results = ref([])

async function search(q: string) {
  const response = await fetch('/api/search', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      query: q,
      top_k: 10,
      rerank: true,
      filters: {
        skill_type: ['wasm', 'native', 'docker'],
        category: ['devops', 'api', 'data']
      }
    })
  })

  const data = await response.json()
  results.value = data.results
}
</script>
```

**Integration Points:**
- **Skill Pages**: Embed search widget to find related tools
- **Examples Gallery**: Filter by category/tags
- **Playground**: Search for example code snippets
- **API Reference**: Search across all endpoints

---

## Phases & Timeline (10 Weeks, 260 Hours)

### Phase 1: Foundation (Weeks 1-2, 60 hours)

**Week 1: VitePress Setup & Initial Migration**
- **Tasks**:
  - [ ] Initialize VitePress project with TypeScript
  - [ ] Configure theme and navigation structure
  - [ ] Set up GitHub Actions workflow (basic)
  - [ ] Migrate README.md → home page with hero section
  - [ ] Migrate quick start guides (2 files)
  - [ ] Create Getting Started section (5 pages)
  - [ ] Set up local development environment
- **Deliverable**: Deployed site with 10+ pages, basic navigation
- **Hours**: 30

**Week 2: Content Migration & Structure**
- **Tasks**:
  - [ ] Migrate skill-development.md → split into 3 guides
  - [ ] Migrate MANIFEST_GUIDE.md → configuration section
  - [ ] Create API reference structure (empty pages)
  - [ ] Add example skill pages (basic, no interactivity yet)
  - [ ] Implement responsive design and mobile navigation
  - [ ] SEO optimization (meta tags, sitemap, robots.txt)
- **Deliverable**: 30+ pages migrated, responsive design
- **Hours**: 30

**Phase 1 Checkpoint**:
- ✅ Site deployed to GitHub Pages
- ✅ 30+ pages live with navigation
- ✅ Mobile-friendly design
- ✅ Basic search (Algolia)

---

### Phase 2: API Documentation (Weeks 3-4, 60 hours)

**Week 3: Rust Docs & OpenAPI**
- **Tasks**:
  - [ ] Add Rust doc comments to skill-runtime (PRIORITY 1)
    - [ ] lib.rs, engine.rs, executor.rs, sandbox.rs
    - [ ] manifest.rs, search/*.rs (10 files)
  - [ ] Add utoipa annotations to skill-http handlers (11 endpoints)
  - [ ] Generate OpenAPI spec and integrate Swagger UI
  - [ ] Test OpenAPI spec with real API calls
  - [ ] Document authentication and error codes
- **Deliverable**: 80%+ Rust doc coverage on skill-runtime, OpenAPI spec live
- **Hours**: 30

**Week 4: MCP Protocol & CLI Reference**
- **Tasks**:
  - [ ] Add Rust doc comments to skill-cli (24 commands)
  - [ ] Write MCP protocol reference guide (from rmcp schemas)
  - [ ] Generate CLI man pages → convert to markdown
  - [ ] Create interactive CLI reference with examples
  - [ ] Add integration examples (Claude Code, Cursor, Python)
  - [ ] Test all documented examples
- **Deliverable**: Complete API reference for REST, MCP, CLI, Rust
- **Hours**: 30

**Phase 2 Checkpoint**:
- ✅ 90%+ Rust doc coverage on skill-runtime
- ✅ OpenAPI spec with Swagger UI
- ✅ MCP protocol documented
- ✅ CLI reference with 24 commands

---

### Phase 3: Interactive Features (Weeks 5-7, 80 hours)

**Week 5: Live WASM Playground**
- **Tasks**:
  - [ ] Build skill-runtime for wasm32-unknown-unknown target
  - [ ] Integrate Monaco Editor in Vue component
  - [ ] Implement JCO componentize (JS → WASM compilation)
  - [ ] Create execution sandbox with result display
  - [ ] Add 5 pre-loaded examples (hello, http, aws, github, multi-tool)
  - [ ] Implement share/export functionality
  - [ ] Performance optimization (lazy loading, caching)
- **Deliverable**: Functional playground at /playground
- **Hours**: 30

**Week 6: API Explorer & Config Generator**
- **Tasks**:
  - [ ] Integrate Swagger UI for REST API explorer
  - [ ] Add "Try It Out" functionality with authentication
  - [ ] Generate JSON Schema from Rust structs (schemars)
  - [ ] Build config generator Vue component
  - [ ] Implement TOML validation and preview
  - [ ] Add pre-filled templates (minimal, team, enterprise)
  - [ ] Test with real manifest examples
- **Deliverable**: API explorer + config generator functional
- **Hours**: 25

**Week 7: RAG Search Integration**
- **Tasks**:
  - [ ] Integrate custom RAG search API for skills/tools
  - [ ] Build search component with autocomplete
  - [ ] Add filters (skill type, category, tags)
  - [ ] Implement keyboard navigation
  - [ ] Add search analytics (track queries)
  - [ ] Optimize search performance (<200ms)
  - [ ] Fallback to Algolia for doc pages
- **Deliverable**: Semantic search with autocomplete
- **Hours**: 25

**Phase 3 Checkpoint**:
- ✅ Live playground functional
- ✅ API explorer with try-it
- ✅ Config generator working
- ✅ RAG search integrated

---

### Phase 4: Content & Polish (Weeks 8-9, 40 hours)

**Week 8: Advanced Guides & Tutorials**
- **Tasks**:
  - [ ] Write Claude Bridge guide (NEW feature, 0 docs currently)
  - [ ] Split rag-search.md → 5 separate guides
  - [ ] Write deployment guide (Docker, Kubernetes, systemd)
  - [ ] Write security best practices guide
  - [ ] Create 5 step-by-step tutorials
  - [ ] Add troubleshooting & FAQ sections
  - [ ] Review and update all existing content
- **Deliverable**: 15+ new guides, comprehensive tutorials
- **Hours**: 25

**Week 9: Examples Gallery & Use Cases**
- **Tasks**:
  - [ ] Create examples gallery with interactive demos
  - [ ] Document all 22 example skills (11 WASM, 5 Native, 6 Docker)
  - [ ] Write 3 use case guides (DevOps, Data, API integration)
  - [ ] Add code snippets for each example
  - [ ] Create video tutorial scripts (for future recording)
  - [ ] Implement dark mode theme (if not done)
  - [ ] Accessibility audit and fixes
- **Deliverable**: Examples gallery, use case guides, A11y compliant
- **Hours**: 15

**Phase 4 Checkpoint**:
- ✅ Claude Bridge documented
- ✅ 22 example skills documented
- ✅ 3 use case guides
- ✅ Accessible design (WCAG 2.1 AA)

---

### Phase 5: Launch Prep (Week 10, 20 hours)

**Week 10: Testing, Optimization, Launch**
- **Tasks**:
  - [ ] Comprehensive testing (manual + automated)
    - [ ] All links work (no 404s)
    - [ ] All code examples execute correctly
    - [ ] All interactive features functional
    - [ ] Mobile/tablet testing
  - [ ] Performance optimization
    - [ ] Lighthouse audit (target 95+ all categories)
    - [ ] Image optimization
    - [ ] Code splitting analysis
  - [ ] SEO final pass
    - [ ] Meta descriptions on all pages
    - [ ] Open Graph images
    - [ ] Sitemap validation
  - [ ] Analytics setup (Google Analytics, Plausible)
  - [ ] Write launch blog post
  - [ ] Create launch announcement (Twitter, Discord, Reddit)
  - [ ] Soft launch to beta users
  - [ ] Collect feedback and iterate
- **Deliverable**: Production-ready documentation site, launched
- **Hours**: 20

**Phase 5 Checkpoint**:
- ✅ Lighthouse 95+ on all metrics
- ✅ Zero broken links
- ✅ All features tested
- ✅ Site launched publicly

---

## Dependencies & Resource Requirements

### Technical Dependencies

**Build Tools:**
- Node.js 20+ (for VitePress)
- Rust 1.75+ (for cargo doc, OpenAPI generation)
- wasm-pack (for WASM builds)
- wasm-opt (for optimization)

**NPM Packages:**
```json
{
  "dependencies": {
    "vitepress": "^1.0.0",
    "vue": "^3.4.0",
    "monaco-editor": "^0.45.0",
    "swagger-ui": "^5.10.0",
    "@iarna/toml": "^2.2.5",
    "ajv": "^8.12.0"
  },
  "devDependencies": {
    "typescript": "^5.3.0",
    "@types/node": "^20.0.0",
    "@bytecodealliance/jco": "^1.0.0"
  }
}
```

**Rust Crates:**
```toml
[dependencies]
utoipa = "5.0"
utoipa-swagger-ui = "8.0"
schemars = "0.8"
clap_mangen = "0.2"
```

**External Services:**
- GitHub Pages (hosting) - FREE
- Algolia DocSearch (search) - FREE for open source
- Google Analytics (analytics) - FREE
- Vercel Blob (WASM hosting, optional) - FREE tier

### Team Requirements

**Primary Developer** (full-time, 10 weeks):
- **Skills**: Vue.js, VitePress, Rust, WASM, OpenAPI
- **Responsibilities**:
  - VitePress setup and configuration
  - Interactive features (playground, explorer, generator)
  - API documentation generation (rustdoc, OpenAPI)
  - Search integration
  - CI/CD pipeline

**Technical Writer** (part-time, 6 weeks, 50% capacity):
- **Skills**: Technical writing, Markdown, API documentation
- **Responsibilities**:
  - Content migration and restructuring
  - Tutorial and guide writing
  - Examples documentation
  - Proofreading and editing

**Designer** (review/advisory, 2 weeks, 25% capacity):
- **Skills**: UI/UX, design systems, accessibility
- **Responsibilities**:
  - Theme customization review
  - Component design feedback
  - Accessibility audit
  - Visual polish

**Total Effort**: ~260 hours over 10 weeks

---

## Risk Analysis & Mitigation

### Risk 1: WASM Playground Complexity
- **Impact**: HIGH
- **Likelihood**: MEDIUM
- **Description**: Compiling JS → WASM in-browser is complex, may have performance issues or compatibility problems
- **Mitigation**:
  - Start with simple editor + pre-compiled WASM examples
  - Use JCO (proven tool from Bytecode Alliance)
  - Extensive browser testing (Chrome, Firefox, Safari)
  - Fallback to server-side compilation if client-side fails
  - Set realistic expectations (3-5s compilation time acceptable)

### Risk 2: Rust Doc Coverage Gap
- **Impact**: MEDIUM
- **Likelihood**: HIGH
- **Description**: 70% of codebase lacks doc comments, takes significant time to add quality docs
- **Mitigation**:
  - Phased approach: skill-runtime first (CRITICAL), others later
  - Provide documentation template
  - Focus on public APIs (users don't need internal docs)
  - CI enforcement after Week 3 (no new code without docs)
  - Accept 80% coverage instead of 90% if time-constrained

### Risk 3: Documentation Drift
- **Impact**: HIGH
- **Likelihood**: HIGH
- **Description**: Docs become outdated as code changes, no automatic updates
- **Mitigation**:
  - Auto-generate wherever possible (rustdoc, OpenAPI, CLI)
  - CI checks for doc tests (ensure code examples compile)
  - Versioned docs (v0.3, v0.4) so old docs stay accurate
  - Contributor guidelines emphasize doc updates
  - Quarterly doc audit scheduled

### Risk 4: Search Quality
- **Impact**: MEDIUM
- **Likelihood**: MEDIUM
- **Description**: Custom RAG search may not perform well, users can't find content
- **Mitigation**:
  - Algolia as primary search (proven quality)
  - Custom RAG as experimental/supplementary
  - A/B test search implementations
  - Collect search analytics to identify gaps
  - Iterate on search based on actual queries

### Risk 5: Interactive Features Performance
- **Impact**: MEDIUM
- **Likelihood**: LOW
- **Description**: Playground/explorer may be slow on lower-end devices
- **Mitigation**:
  - Performance budget: <3s load, <5s compilation
  - Lazy loading for heavy components
  - WASM size optimization (wasm-opt -Oz)
  - Fallback to simplified UI on slow connections
  - Browser requirements clearly stated

### Risk 6: API Breaking Changes
- **Impact**: MEDIUM
- **Likelihood**: MEDIUM
- **Description**: Skill Engine APIs change, docs become inaccurate
- **Mitigation**:
  - Semantic versioning strictly enforced
  - Changelog with breaking changes highlighted
  - Deprecation warnings in docs (with migration guide)
  - Version dropdown to access old docs
  - API stability policy documented

### Risk 7: Content Quality
- **Impact**: LOW
- **Likelihood**: LOW
- **Description**: Docs have errors, confusing explanations, broken examples
- **Mitigation**:
  - Technical review by Skill Engine maintainers
  - User testing with 5 beta users
  - Feedback mechanism on every page
  - Doc tests for all code examples
  - Continuous iteration based on feedback

### Risk 8: Timeline Slippage
- **Impact**: LOW
- **Likelihood**: MEDIUM
- **Description**: 10-week timeline is ambitious, delays likely
- **Mitigation**:
  - Prioritization: P0 must-have vs P1 nice-to-have
  - Incremental delivery (ship Phase 1, then Phase 2, etc.)
  - Buffer built into each phase (~20%)
  - Can skip Phase 4 content if needed (add post-launch)
  - Parallelize work where possible (content + code)

### Risk 9: Hosting/Deployment Issues
- **Impact**: LOW
- **Likelihood**: LOW
- **Description**: GitHub Pages has limitations, deployment fails
- **Mitigation**:
  - Test deployment early (Week 1)
  - Backup plan: Vercel or Cloudflare Pages
  - CNAME setup for custom domain
  - CI/CD tested with multiple deploys
  - Rollback strategy (previous deployment preserved)

---

## Success Criteria & Launch Checklist

### Must Have (Launch Blockers)

**Content:**
- [ ] 50+ documentation pages published
- [ ] All existing 5,200+ lines of docs migrated
- [ ] Getting Started guide with quick start (<10 min)
- [ ] API reference for CLI (24 commands)
- [ ] At least 3 complete tutorials

**Technical:**
- [ ] VitePress site deployed to GitHub Pages
- [ ] Custom domain (docs.skill-engine.dev) working
- [ ] HTTPS enabled with valid certificate
- [ ] Mobile-responsive design (works on phone/tablet)
- [ ] Search functional (Algolia DocSearch)
- [ ] Zero broken links (link checker passes)

**API Documentation:**
- [ ] 80%+ Rust doc coverage on skill-runtime
- [ ] OpenAPI spec for all REST endpoints
- [ ] Swagger UI functional at /api/rest
- [ ] MCP protocol documented with examples

**Interactive Features:**
- [ ] Live WASM playground functional
- [ ] At least 3 pre-loaded examples work
- [ ] Playground share functionality works

**Performance:**
- [ ] Lighthouse score 90+ (mobile)
- [ ] First Contentful Paint < 2s
- [ ] Time to Interactive < 4s

**SEO & Analytics:**
- [ ] Meta descriptions on all pages
- [ ] Open Graph images
- [ ] Sitemap.xml generated
- [ ] Robots.txt configured
- [ ] Google Analytics integrated

---

### Should Have (High Priority, Post-Launch OK)

**Content:**
- [ ] Claude Bridge guide (NEW feature, 0 docs)
- [ ] 5+ step-by-step tutorials
- [ ] 3 use case guides (DevOps, Data, API)
- [ ] RAG search split into 5 guides
- [ ] Deployment guide (Docker, Kubernetes)

**API Documentation:**
- [ ] 90%+ Rust doc coverage across all crates
- [ ] CLI reference with examples for every command
- [ ] MCP integration guide for custom clients

**Interactive Features:**
- [ ] API explorer with try-it functionality
- [ ] Config generator with validation
- [ ] 5+ playground examples

**Quality:**
- [ ] Accessibility audit (WCAG 2.1 AA)
- [ ] Dark mode theme
- [ ] Keyboard navigation throughout site
- [ ] Print stylesheet for docs

---

### Nice to Have (Post-Launch)

**Content:**
- [ ] Video tutorials (5+ videos)
- [ ] Community contributions guide
- [ ] Case studies from real users
- [ ] Bi-weekly blog posts

**Interactive Features:**
- [ ] RAG search visualizer (show ranking)
- [ ] Multi-language code examples
- [ ] Playground with GitHub import
- [ ] Config generator with templates gallery

**Community:**
- [ ] Discord integration (live chat widget)
- [ ] User-submitted examples gallery
- [ ] Monthly office hours schedule
- [ ] Newsletter signup

---

## Monthly Targets (Post-Launch)

### Month 1 Targets
- **Traffic**: 500 unique visitors
- **Playground**: 100 skill executions
- **GitHub Stars**: +50 (10% increase)
- **Search Queries**: 200+ searches
- **Time on Site**: > 4 minutes average
- **Bounce Rate**: < 50%

### Month 3 Targets
- **Traffic**: 2,000 unique visitors
- **Playground**: 500 skill executions
- **GitHub Stars**: +300 (60% increase, +150% from baseline)
- **Search Success**: 80%+ queries find relevant results
- **Returning Visitors**: 30%
- **Documentation Contributions**: 5+ community PRs

### Month 6 Targets
- **Traffic**: 5,000 unique visitors
- **Playground**: 1,000 skill executions
- **GitHub Stars**: +500 (100% increase, +300% from baseline)
- **Search Success**: 85%+
- **Returning Visitors**: 40%
- **Newsletter Subscribers**: 500+

---

## Iteration Plan

### Weekly Iteration (During Development)
- Monday: Review previous week, plan current week
- Wednesday: Mid-week check-in, adjust priorities
- Friday: Demo progress, collect feedback

### Monthly Iteration (Post-Launch)
- **Metrics Review**:
  - Traffic (Google Analytics)
  - User behavior (heatmaps, scroll depth)
  - Search analytics (top queries, success rate)
  - Playground usage (executions, shared URLs)
- **Content Audit**:
  - Identify most/least visited pages
  - Update outdated content
  - Add requested documentation
- **Feature Prioritization**:
  - Review feedback and feature requests
  - Prioritize next month's additions
  - Allocate 20% time for iteration

### Quarterly Review
- Comprehensive site audit
- User survey (satisfaction, pain points)
- Competitive analysis (vs other doc sites)
- Roadmap update for next quarter

---

## Appendices

### Appendix A: Claude Bridge Documentation Plan

Claude Bridge is a NEW feature (1,550 lines added recently) with ZERO documentation. This is a high-priority gap.

**Content Needed:**

1. **Overview Page** (`/guides/advanced/claude-bridge.md`):
   - What is Claude Bridge?
   - Use cases (generate Claude Agent Skills from Skill Engine skills)
   - Architecture diagram
   - Benefits vs manual skill creation

2. **Getting Started**:
   - Installation (already in Skill Engine)
   - First skill generation
   - 5-minute tutorial

3. **Command Reference**:
   - `skill claude-bridge generate` options
   - All generation modes (single, batch, filter)
   - YAML frontmatter customization
   - Output formats

4. **Advanced Usage**:
   - Batch generation with filters
   - Custom templates
   - Integration with CI/CD
   - Validation and testing

5. **Examples**:
   - Generate single skill: `skill claude-bridge generate kubernetes`
   - Generate all skills: `skill claude-bridge generate --all`
   - Custom output: `skill claude-bridge generate --output ./skills/`

**Priority**: P0 (Week 2-3, part of content migration)

---

### Appendix B: Competitive Analysis

**Comparison with Other AI Tool Documentation:**

| Feature | Skill Engine Docs | E2B Docs | Modal Docs | Dagger Docs |
|---------|------------------|----------|------------|-------------|
| **Interactive Playground** | ✅ WASM in-browser | ❌ No playground | ✅ Python sandbox | ❌ No playground |
| **API Explorer** | ✅ Swagger UI | ✅ REST docs | ✅ gRPC docs | ✅ GraphQL playground |
| **Search Quality** | ✅ Algolia + RAG | ✅ Algolia | ✅ Algolia | ✅ Custom search |
| **Mobile Experience** | ✅ Responsive | ✅ Responsive | ⚠️ Limited | ✅ Responsive |
| **Video Tutorials** | ❌ Post-launch | ✅ 10+ videos | ✅ 5+ videos | ✅ 3+ videos |
| **Example Gallery** | ✅ 22 skills | ✅ 15+ examples | ✅ 20+ examples | ✅ 10+ examples |
| **Versioned Docs** | ⚠️ Phase 2 | ✅ Multi-version | ✅ Multi-version | ✅ Multi-version |

**Differentiation:**
- **Live WASM Playground**: Unique to Skill Engine (WASM sandboxing)
- **RAG-Powered Search**: Semantic search for skills/tools (vs keyword only)
- **Config Generator**: Visual tool for manifest creation (unique)
- **MCP Protocol**: Deep integration with Claude Code (unique positioning)

---

### Appendix C: Page Layout Wireframes

**Home Page:**
```
┌─────────────────────────────────────────────┐
│ Hero Section                                │
│ ┌─────────────────────────────────────────┐ │
│ │ Skill Engine                            │ │
│ │ Universal Runtime for AI Agents         │ │
│ │                                         │ │
│ │ [Get Started →]  [View Playground]     │ │
│ └─────────────────────────────────────────┘ │
├─────────────────────────────────────────────┤
│ Feature Grid (3 columns)                    │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│ │ WASM     │ │ RAG      │ │ MCP      │    │
│ │ Sandbox  │ │ Search   │ │ Protocol │    │
│ └──────────┘ └──────────┘ └──────────┘    │
├─────────────────────────────────────────────┤
│ Code Example                                │
│ // Write skills in JavaScript              │
│ export function executeTool(...) { }       │
├─────────────────────────────────────────────┤
│ Quick Links                                 │
│ Getting Started | Examples | API Ref       │
└─────────────────────────────────────────────┘
```

**Playground Page:**
```
┌─────────────────────────────────────────────┐
│ Header: "Live WASM Playground"              │
├────────────────────┬────────────────────────┤
│ Editor (Monaco)    │ Output Panel           │
│                    │                        │
│ export function    │ > Running skill...     │
│ executeTool() {    │                        │
│   return {         │ ✓ Success              │
│     success: true  │ Output:                │
│   }                │ Hello, World!          │
│ }                  │                        │
│                    │ Execution: 42ms        │
│                    │ Memory: 2.1MB          │
├────────────────────┴────────────────────────┤
│ Controls: [Run] [Share] [Export]            │
│ Examples: ▼ Hello World                     │
└─────────────────────────────────────────────┘
```

**API Explorer:**
```
┌─────────────────────────────────────────────┐
│ REST API Reference                          │
├─────────────────────────────────────────────┤
│ GET /api/skills/{name}                      │
│ ┌─────────────────────────────────────────┐ │
│ │ Parameters                              │ │
│ │ name: string (path, required)          │ │
│ │                                         │ │
│ │ [Try It Out]                           │ │
│ │ name: [kubernetes____________]         │ │
│ │ [Execute]                              │ │
│ │                                         │ │
│ │ Response: 200 OK                       │ │
│ │ {                                      │ │
│ │   "name": "kubernetes",                │ │
│ │   "tools": [...]                       │ │
│ │ }                                      │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

---

### Appendix D: SEO Strategy

**On-Page SEO:**
- **Title Tags**: `<title>Page Title | Skill Engine Docs</title>` (50-60 chars)
- **Meta Descriptions**: 150-160 characters, keyword-rich
- **Headings**: Proper H1→H2→H3 hierarchy
- **Images**: Alt text on all images
- **Internal Linking**: 3-5 links per page to related content
- **URL Structure**: Clean, descriptive URLs (/getting-started/quick-start)

**Technical SEO:**
- **Sitemap**: Auto-generated by VitePress
- **Robots.txt**: Allow all, disallow /api/internal
- **Structured Data**: Schema.org markup for articles
- **Canonical URLs**: Prevent duplicate content
- **Mobile-First**: Responsive design
- **Page Speed**: Lighthouse 90+

**Content SEO:**
- **Keywords**: "AI agent runtime", "WASM skills", "MCP protocol", "skill engine"
- **Long-Tail**: "how to create AI agent skills", "skill engine vs modal"
- **Freshness**: Update dates on pages
- **Comprehensive**: 1000+ word guides

**Off-Page SEO:**
- **Backlinks**: GitHub, dev.to, Hacker News posts
- **Social**: Twitter, LinkedIn shares
- **Community**: Discord, Reddit mentions

**Target Keywords** (with search volume estimates):
- "AI agent runtime" (100/mo)
- "WASM skills" (50/mo)
- "MCP protocol" (200/mo)
- "skill engine" (500/mo)
- "Claude Code integration" (300/mo)

---

## Conclusion

This PRD provides a comprehensive roadmap for creating a world-class documentation site for Skill Engine over 10 weeks. The phased approach ensures incremental value delivery while building toward the full vision of an interactive, searchable, comprehensive documentation platform.

**Key Success Factors:**
1. **User-Centric Design**: Every feature solves a real user pain point
2. **Interactive Learning**: Playground, explorer, generators for hands-on learning
3. **Complete Coverage**: 90%+ API documentation, all features documented
4. **Performance**: Fast, responsive, accessible
5. **Continuous Improvement**: Monthly iteration based on data

**Next Steps:**
1. Review and approve this PRD
2. Parse with Task Master to generate 30-40 implementation tasks
3. Begin Phase 1 (VitePress setup and migration)
4. Iterate based on feedback and metrics

**Total Effort**: 260 hours over 10 weeks
**Team**: 1 developer (full-time), 1 writer (part-time), 1 designer (advisory)
**Budget**: ~$0 (GitHub Pages, Algolia, all free for open source)
**Launch Target**: End of Week 10

This documentation site will transform Skill Engine from a hidden gem into a widely adopted, well-understood platform for AI agent development.
