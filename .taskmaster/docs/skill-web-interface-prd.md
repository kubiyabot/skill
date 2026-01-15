# Product Requirements Document: Skill Web Interface

## Document Information

| Field | Value |
|-------|-------|
| **Document Title** | Skill Web Interface PRD |
| **Version** | 1.0 |
| **Date** | December 2025 |
| **Author** | AI Assistant |
| **Status** | Draft |

---

## Executive Summary

This PRD defines the requirements for a comprehensive web interface for the Skill Engine platform. The web interface will provide a modern, intuitive UI for managing skills end-to-end, including installation, configuration, execution, testing, and validation. Built using Yew (Rust WebAssembly framework), the interface will serve as both a management dashboard and an alternative onboarding experience to the current TUI-based interactive flow.

### Key Objectives

1. **Unified Management**: Single pane of glass for all skill operations
2. **Developer Experience**: Visual skill discovery, configuration, and testing
3. **Onboarding Replacement**: Web-based setup wizard replacing the CLI interactive flow
4. **Real-time Monitoring**: Live execution feedback and performance metrics
5. **Self-Hosted**: Runs locally via `skill web` command with zero external dependencies

---

## Background & Context

### Current State

The Skill Engine currently provides:

- **CLI (`skill`)**: Full-featured command-line interface with 20+ commands
- **MCP Server**: Model Context Protocol server for AI agent integration
- **HTTP Skeleton**: Basic Axum-based HTTP server (not yet implemented)

### Pain Points

1. **CLI Learning Curve**: Users must learn multiple commands and flags
2. **Configuration Complexity**: TOML manifests require manual editing
3. **No Visual Discovery**: Semantic search exists but lacks visual exploration
4. **Debugging Difficulty**: Execution errors require log parsing
5. **Onboarding Friction**: Interactive CLI setup is platform-dependent

### Why Yew?

| Criteria | Yew Advantage |
|----------|---------------|
| **Language Consistency** | Pure Rust codebase, shares types with runtime |
| **Performance** | WASM compilation, near-native speed |
| **Type Safety** | Compile-time guarantees across frontend/backend |
| **Bundle Size** | Small WASM binaries, fast loading |
| **No JS Toolchain** | Trunk handles everything, no npm required |
| **Offline First** | Embedded in binary, works without internet |

---

## User Personas

### 1. AI Developer (Primary)

- **Background**: Building AI agents using Claude, GPT, or custom models
- **Goals**: Quickly add capabilities to agents via skills
- **Pain Points**: Wants visual skill discovery, not CLI memorization
- **Usage Pattern**: Explores skills, configures instances, monitors executions

### 2. DevOps Engineer

- **Background**: Managing skill deployments across environments
- **Goals**: Configure instances, manage credentials, audit executions
- **Pain Points**: Needs centralized configuration management
- **Usage Pattern**: Sets up production instances, monitors resource usage

### 3. Skill Author

- **Background**: Creating custom skills for internal or public use
- **Goals**: Test skills, validate configurations, debug issues
- **Pain Points**: Rapid iteration cycle for skill development
- **Usage Pattern**: Frequent test executions, configuration tweaks

### 4. First-Time User

- **Background**: New to Skill Engine, wants to evaluate
- **Goals**: Quick setup, see value proposition immediately
- **Pain Points**: Overwhelmed by CLI options
- **Usage Pattern**: Guided onboarding, sample skill execution

---

## Feature Requirements

### Phase 1: Core Foundation (MVP)

#### F1.1 Dashboard Overview

**Priority**: P0 (Must Have)

**Description**: Landing page providing at-a-glance system status

**Functional Requirements**:
- Display total installed skills count
- Show skills by source (Git, Local, WASM, SKILL.md)
- Recent execution history (last 10 executions)
- System health indicators (RAG pipeline status, vector store)
- Quick action buttons (Install Skill, Run Skill, Open Settings)

**UI Components**:
```
┌─────────────────────────────────────────────────────────────┐
│  Skill Engine Dashboard                        [Settings] ⚙️ │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│ │   Skills    │ │ Executions  │ │   Search    │            │
│ │     12      │ │    Today    │ │   Ready     │            │
│ │  Installed  │ │     47      │ │   ✓ RAG     │            │
│ └─────────────┘ └─────────────┘ └─────────────┘            │
│                                                             │
│ Quick Actions                                               │
│ [+ Install Skill] [▶ Run Skill] [🔍 Search] [📚 Docs]      │
│                                                             │
│ Recent Activity                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ▶ kubernetes:get_pods          2s ago    ✓ Success     │ │
│ │ ▶ github:list_repos            5m ago    ✓ Success     │ │
│ │ ▶ aws:s3_list_buckets         12m ago    ✗ Error       │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] Dashboard loads within 500ms
- [ ] All metrics update in real-time via WebSocket
- [ ] Clicking activity item navigates to execution details
- [ ] Quick actions are contextually enabled/disabled

---

#### F1.2 Skill Browser & Discovery

**Priority**: P0 (Must Have)

**Description**: Visual interface for browsing and discovering skills

**Functional Requirements**:
- List all installed skills with metadata
- Search skills by name, description, or tags
- Semantic search using RAG pipeline
- Filter by source type (Git, Local, SKILL.md, WASM)
- Filter by status (Configured, Unconfigured, Error)
- Sort by name, last used, execution count

**UI Components**:
```
┌─────────────────────────────────────────────────────────────┐
│ Skills                                    [+ Install New]   │
├─────────────────────────────────────────────────────────────┤
│ 🔍 Search skills...           [Semantic] [Keyword]          │
│                                                             │
│ Filters: [All Sources ▼] [All Status ▼] [Sort: Name ▼]     │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 🎛️ kubernetes                              [3 instances] │ │
│ │ Kubernetes cluster management                           │ │
│ │ Source: github:skill-engine/kubernetes-skill            │ │
│ │ Tools: 18 | Last Used: 2h ago | Executions: 247        │ │
│ │ [Configure] [Run] [View Details]                        │ │
│ └─────────────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 🐙 github                                  [1 instance]  │ │
│ │ GitHub repository and issue management                  │ │
│ │ Source: local:./skills/github.wasm                      │ │
│ │ Tools: 12 | Last Used: 1d ago | Executions: 89         │ │
│ │ [Configure] [Run] [View Details]                        │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] Skills load with pagination (20 per page)
- [ ] Semantic search returns relevant results within 200ms
- [ ] Filters combine with AND logic
- [ ] Empty state shows "No skills found" with install CTA

---

#### F1.3 Skill Detail View

**Priority**: P0 (Must Have)

**Description**: Comprehensive skill information page

**Functional Requirements**:
- Display skill metadata (name, version, description, author)
- Render SKILL.md or description as formatted Markdown
- List all tools with parameters and descriptions
- Show instance configurations
- Display execution history for this skill
- Show usage examples (AI-generated via `skill enhance`)

**UI Components**:
```
┌─────────────────────────────────────────────────────────────┐
│ ← Back to Skills                                            │
├─────────────────────────────────────────────────────────────┤
│ 🎛️ kubernetes                                    v1.2.0     │
│ Kubernetes cluster management for AI agents                │
│                                                             │
│ ┌─────────┬───────────┬────────────┬──────────────────────┐ │
│ │ Overview│   Tools   │  Instances │ Execution History    │ │
│ └─────────┴───────────┴────────────┴──────────────────────┘ │
│                                                             │
│ ## Description                                              │
│                                                             │
│ This skill provides comprehensive Kubernetes cluster        │
│ management capabilities including pod operations,           │
│ deployment management, and service discovery.              │
│                                                             │
│ ### Features                                                │
│ - Pod lifecycle management (create, delete, list)          │
│ - Deployment rollouts and rollbacks                        │
│ - Service and ingress configuration                        │
│ - Namespace isolation support                              │
│                                                             │
│ ## Quick Start                                              │
│ ```bash                                                     │
│ skill run kubernetes:get_pods namespace=default            │
│ ```                                                         │
│                                                             │
│ [▶ Run Skill] [⚙️ Configure] [🗑️ Uninstall]                │
└─────────────────────────────────────────────────────────────┘
```

**Tabs Content**:

**Tools Tab**:
```
┌─────────────────────────────────────────────────────────────┐
│ Tools (18)                              🔍 Filter tools...  │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ get_pods                                      [▶ Run]   │ │
│ │ List pods in a namespace                                │ │
│ │                                                         │ │
│ │ Parameters:                                             │ │
│ │ • namespace (string, optional) - Target namespace       │ │
│ │   Default: "default"                                    │ │
│ │ • labels (string, optional) - Label selector            │ │
│ │ • field_selector (string, optional) - Field selector    │ │
│ │                                                         │ │
│ │ Example:                                                │ │
│ │ ```json                                                 │ │
│ │ { "namespace": "production", "labels": "app=web" }     │ │
│ │ ```                                                     │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Instances Tab**:
```
┌─────────────────────────────────────────────────────────────┐
│ Instances (3)                              [+ Add Instance] │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 🟢 prod                                    [Default]     │ │
│ │ Production cluster configuration                        │ │
│ │ Config: kubeconfig=/home/user/.kube/prod-config        │ │
│ │ Capabilities: network_access=true                       │ │
│ │ [Edit] [Delete] [Set Default]                          │ │
│ └─────────────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 🟢 staging                                               │ │
│ │ Staging cluster configuration                           │ │
│ │ Config: kubeconfig=/home/user/.kube/staging-config     │ │
│ │ [Edit] [Delete] [Set Default]                          │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] Markdown renders with syntax highlighting for code blocks
- [ ] Tool parameters show type, required/optional, defaults
- [ ] Instance status indicators (green=configured, red=error)
- [ ] Tab state persists during navigation

---

#### F1.4 Skill Installation Wizard

**Priority**: P0 (Must Have)

**Description**: Multi-step wizard for installing skills from various sources

**Functional Requirements**:
- Support installation from:
  - GitHub shorthand (`github:user/repo@version`)
  - Git URL (`https://github.com/user/repo.git`)
  - HTTP URL (direct WASM download)
  - Local path (for development)
- Validate skill before installation
- Preview skill metadata and tools
- Configure initial instance during installation
- Show installation progress with logs

**UI Components**:
```
┌─────────────────────────────────────────────────────────────┐
│ Install Skill                                        Step 1/4│
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ ○ Step 1: Source    ● Step 2: Validate    ○ Step 3: Config │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│ Select Installation Source                                  │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ○ GitHub Repository                                     │ │
│ │   github:user/repo@v1.0.0                              │ │
│ │   ┌─────────────────────────────────────────────────┐   │ │
│ │   │ github:skill-engine/kubernetes-skill@v1.2.0    │   │ │
│ │   └─────────────────────────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ○ Git URL                                               │ │
│ │   https://github.com/user/repo.git                     │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ○ HTTP/HTTPS URL                                        │ │
│ │   Direct link to .wasm file                            │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ○ Local Path                                            │ │
│ │   /path/to/skill.wasm or /path/to/SKILL.md             │ │
│ │   [Browse...]                                          │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│                              [Cancel] [Next →]              │
└─────────────────────────────────────────────────────────────┘
```

**Step 2: Validation**:
```
┌─────────────────────────────────────────────────────────────┐
│ Install Skill                                        Step 2/4│
├─────────────────────────────────────────────────────────────┤
│ Validating Skill...                                         │
│                                                             │
│ ✓ Repository cloned                                         │
│ ✓ SKILL.md found                                            │
│ ✓ Metadata parsed                                           │
│ ⟳ Validating tool definitions...                            │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│ Skill Preview                                               │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Name: kubernetes                                        │ │
│ │ Version: 1.2.0                                          │ │
│ │ Author: Skill Engine Team                               │ │
│ │ Description: Kubernetes cluster management              │ │
│ │ Tools: 18                                               │ │
│ │ Type: SKILL.md (Native Commands)                        │ │
│ │                                                         │ │
│ │ Tools Preview:                                          │ │
│ │ • get_pods - List pods in a namespace                   │ │
│ │ • get_deployments - List deployments                    │ │
│ │ • apply_manifest - Apply Kubernetes manifests           │ │
│ │ • ... 15 more tools                                     │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│                     [← Back] [Cancel] [Next →]              │
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] All four source types work correctly
- [ ] Validation shows clear error messages on failure
- [ ] Progress indicators update in real-time
- [ ] Can cancel installation at any step
- [ ] Installed skill appears in browser immediately

---

#### F1.5 Skill Execution Interface

**Priority**: P0 (Must Have)

**Description**: Interactive interface for executing skill tools

**Functional Requirements**:
- Select skill and tool from dropdowns
- Dynamic form generation based on tool parameters
- Parameter validation with inline errors
- Instance selection (if multiple configured)
- Execute and display results
- Output formatting options (JSON, raw, formatted)
- Copy output to clipboard
- Save execution to history

**UI Components**:
```
┌─────────────────────────────────────────────────────────────┐
│ Run Skill                                                   │
├─────────────────────────────────────────────────────────────┤
│ Skill: [kubernetes          ▼]  Tool: [get_pods         ▼] │
│ Instance: [prod (default)    ▼]                             │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│ Parameters                                                  │
│                                                             │
│ namespace                                                   │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ default                                                 │ │
│ └─────────────────────────────────────────────────────────┘ │
│ Target Kubernetes namespace (optional, default: "default") │
│                                                             │
│ labels                                                      │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ app=nginx                                               │ │
│ └─────────────────────────────────────────────────────────┘ │
│ Label selector for filtering pods (optional)               │
│                                                             │
│ ☐ Include metadata in output                                │
│ Output format: ○ JSON  ○ Raw  ○ Formatted                  │
│                                                             │
│              [Clear] [▶ Execute]                            │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│ Output                                       [📋 Copy] [↓]  │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ {                                                       │ │
│ │   "pods": [                                             │ │
│ │     {                                                   │ │
│ │       "name": "nginx-7d4b8f9c5-abc12",                 │ │
│ │       "namespace": "default",                          │ │
│ │       "status": "Running",                             │ │
│ │       "age": "5d"                                       │ │
│ │     },                                                  │ │
│ │     ...                                                 │ │
│ │   ]                                                     │ │
│ │ }                                                       │ │
│ └─────────────────────────────────────────────────────────┘ │
│ Execution time: 234ms | Status: Success                    │
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] Form fields dynamically match tool parameters
- [ ] Required fields show validation errors before submission
- [ ] Output updates in real-time during execution
- [ ] Long outputs are virtualized (not all rendered)
- [ ] Execution history saves last 100 runs

---

#### F1.6 Instance Configuration Editor

**Priority**: P0 (Must Have)

**Description**: Visual editor for skill instance configurations

**Functional Requirements**:
- Create new instances with unique names
- Edit configuration key-value pairs
- Support environment variable expansion preview
- Configure capabilities (network, filesystem, env)
- Validate configuration against skill requirements
- Test configuration before saving

**UI Components**:
```
┌─────────────────────────────────────────────────────────────┐
│ Configure Instance: kubernetes / prod                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Instance Name                                               │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ prod                                                    │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ Description                                                 │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Production Kubernetes cluster                           │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│ Configuration                                  [+ Add Key]  │
│                                                             │
│ ┌────────────────┬────────────────────────────┬───────────┐ │
│ │ Key            │ Value                      │           │ │
│ ├────────────────┼────────────────────────────┼───────────┤ │
│ │ kubeconfig     │ ${KUBECONFIG:-~/.kube/cfg} │ [🗑️]      │ │
│ │ context        │ production                 │ [🗑️]      │ │
│ │ namespace      │ ${KUBE_NAMESPACE}          │ [🗑️]      │ │
│ └────────────────┴────────────────────────────┴───────────┘ │
│                                                             │
│ Resolved Values Preview:                                    │
│ • kubeconfig → /home/user/.kube/prod-config                │
│ • context → production                                      │
│ • namespace → ⚠️ KUBE_NAMESPACE not set                    │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│ Capabilities                                                │
│ ☑ Network Access    ☐ Filesystem Access    ☑ Environment   │
│                                                             │
│ Network Allowlist (comma-separated)                        │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ kubernetes.default.svc, *.k8s.io                        │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│            [Cancel] [Test Configuration] [Save]             │
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] Environment variable expansion shows live preview
- [ ] Invalid configurations show clear error messages
- [ ] Test button validates config without saving
- [ ] Changes require confirmation before discarding

---

### Phase 2: Onboarding & Setup

#### F2.1 Web-Based Onboarding Wizard

**Priority**: P0 (Must Have)

**Description**: Replace CLI interactive flow with web-based setup

**Functional Requirements**:
- Welcome screen with value proposition
- RAG pipeline configuration:
  - Embedding provider selection (FastEmbed, OpenAI, Ollama)
  - Vector store selection (InMemory, Qdrant)
  - Hybrid search toggle
  - Reranker configuration
- Credential setup:
  - API keys for LLM providers
  - Authentication for skill registries
- Sample skill installation:
  - Curated list of starter skills
  - One-click install
- Claude Code integration setup:
  - Automatic `.mcp.json` configuration
  - Connection testing

**UI Components**:
```
┌─────────────────────────────────────────────────────────────┐
│                    Welcome to Skill Engine                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                    ┌───────────────────┐                    │
│                    │    🚀 Skill       │                    │
│                    │      Engine       │                    │
│                    └───────────────────┘                    │
│                                                             │
│         Give your AI agents superpowers with                │
│              sandboxed WASM skill execution                 │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│  What you'll configure:                                     │
│                                                             │
│  ✓ Search Pipeline - How skills are discovered              │
│  ✓ AI Integration - Connect LLM providers                   │
│  ✓ Starter Skills - Get productive immediately              │
│  ✓ Claude Code - Seamless integration                       │
│                                                             │
│  Estimated time: 3-5 minutes                                │
│                                                             │
│                    [Get Started →]                          │
│                                                             │
│  Already configured? [Skip to Dashboard]                    │
└─────────────────────────────────────────────────────────────┘
```

**Step: Search Configuration**:
```
┌─────────────────────────────────────────────────────────────┐
│ Setup: Search Pipeline                               Step 1/4│
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Choose how skills are discovered and searched               │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│ Embedding Provider                                          │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ● FastEmbed (Recommended)                               │ │
│ │   Local, offline, no API keys required                  │ │
│ │   Model: all-MiniLM-L6-v2 (384 dimensions)             │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ○ OpenAI                                                │ │
│ │   Cloud-based, requires API key                         │ │
│ │   Model: text-embedding-3-small                        │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ○ Ollama                                                │ │
│ │   Self-hosted, requires Ollama installation            │ │
│ │   Model: nomic-embed-text                              │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│ Vector Store                                                │
│                                                             │
│ ● In-Memory (Recommended for development)                  │
│ ○ Qdrant (Production, requires Qdrant server)              │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│ Advanced Options                                      [▼]   │
│ ☐ Enable Hybrid Search (BM25 + Vector)                     │
│ ☐ Enable Reranking (Cross-encoder)                         │
│                                                             │
│                     [← Back] [Next →]                       │
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] Onboarding auto-starts on first launch
- [ ] Each step validates before proceeding
- [ ] Can skip optional steps
- [ ] Configuration persists to `search.toml` and `.env`
- [ ] Progress saved if user exits mid-setup

---

#### F2.2 Credential Manager

**Priority**: P1 (Should Have)

**Description**: Secure management of API keys and credentials

**Functional Requirements**:
- Store credentials in system keyring
- Support multiple credential types:
  - API Keys (OpenAI, Anthropic, etc.)
  - OAuth2 tokens (GitHub, etc.)
  - AWS credentials (access key, secret key, session token)
- Test credential validity
- Show credential usage (which skills use which credentials)
- Secure display (masked by default, reveal on click)

**UI Components**:
```
┌─────────────────────────────────────────────────────────────┐
│ Credentials                                  [+ Add New]    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ API Keys                                                    │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 🔑 OPENAI_API_KEY                            [✓ Valid]  │ │
│ │    sk-...************************xyz                    │ │
│ │    Used by: search pipeline (embeddings)               │ │
│ │    [👁️ Reveal] [✏️ Edit] [🗑️ Delete]                   │ │
│ └─────────────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 🔑 ANTHROPIC_API_KEY                         [✓ Valid]  │ │
│ │    sk-ant-...*********************abc                   │ │
│ │    Used by: skill enhancement                          │ │
│ │    [👁️ Reveal] [✏️ Edit] [🗑️ Delete]                   │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ OAuth Tokens                                                │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 🔐 GitHub                                   [⚠️ Expiring]│ │
│ │    Expires: 2025-01-15                                  │ │
│ │    Scopes: repo, read:user                             │ │
│ │    Used by: github skill                               │ │
│ │    [🔄 Refresh] [🗑️ Revoke]                             │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ AWS Credentials                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ☁️ AWS (default profile)                     [✓ Valid]  │ │
│ │    Access Key: AKIA...************XYZ                   │ │
│ │    Region: us-west-2                                    │ │
│ │    Used by: aws skill                                   │ │
│ │    [✏️ Edit] [🗑️ Delete]                                │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] Credentials stored in OS keyring (not filesystem)
- [ ] Validation happens on save
- [ ] Expiring credentials show warnings
- [ ] Deletion requires confirmation

---

### Phase 3: Advanced Features

#### F3.1 Execution History & Analytics

**Priority**: P1 (Should Have)

**Description**: Comprehensive execution tracking and analytics

**Functional Requirements**:
- Persistent execution history (last 1000 executions)
- Filter by skill, tool, status, date range
- Execution details: input, output, timing, errors
- Re-run previous executions
- Export history (JSON, CSV)
- Analytics dashboard:
  - Executions over time
  - Success/failure rate
  - Average execution time by tool
  - Most used tools

**UI Components**:
```
┌─────────────────────────────────────────────────────────────┐
│ Execution History                              [📊 Analytics]│
├─────────────────────────────────────────────────────────────┤
│ 🔍 Search... [Skill ▼] [Status ▼] [Date Range ▼] [Export ▼]│
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ #1247  kubernetes:get_pods         2 min ago   ✓ 234ms │ │
│ │ Instance: prod | namespace=default, labels=app=nginx   │ │
│ │ [View Details] [Re-run]                                 │ │
│ └─────────────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ #1246  aws:s3_list_buckets         15 min ago  ✗ Error │ │
│ │ Instance: default | prefix=data-                       │ │
│ │ Error: AccessDenied: Access Denied                     │ │
│ │ [View Details] [Re-run]                                 │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│ Execution #1247 Details                                     │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Skill: kubernetes                                       │ │
│ │ Tool: get_pods                                          │ │
│ │ Instance: prod                                          │ │
│ │ Timestamp: 2025-12-22T10:30:45Z                        │ │
│ │ Duration: 234ms                                         │ │
│ │ Status: Success                                         │ │
│ │                                                         │ │
│ │ Input:                                                  │ │
│ │ { "namespace": "default", "labels": "app=nginx" }      │ │
│ │                                                         │ │
│ │ Output:                                                 │ │
│ │ { "pods": [...] }                                       │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] History persists across restarts (SQLite)
- [ ] Filtering is instantaneous (<100ms)
- [ ] Re-run pre-fills execution form
- [ ] Analytics charts render with 30-day data

---

#### F3.2 Skill Marketplace / Registry Browser

**Priority**: P2 (Nice to Have)

**Description**: Browse and install skills from public registries

**Functional Requirements**:
- Browse skill-engine official registry (when available)
- Browse GitHub skill repositories
- Search by name, category, tags
- Show skill ratings and download counts
- One-click install from marketplace
- Version selection

**UI Components**:
```
┌─────────────────────────────────────────────────────────────┐
│ Skill Marketplace                              [My Skills →]│
├─────────────────────────────────────────────────────────────┤
│ 🔍 Search marketplace...                                    │
│                                                             │
│ Categories: [All ▼] [Cloud] [DevOps] [Data] [AI] [Utils]   │
│                                                             │
│ Featured Skills                                             │
│ ┌───────────────────────────────────────────────────────┐   │
│ │ ⭐ kubernetes-skill                    ★★★★★ (128)    │   │
│ │ Official Kubernetes cluster management                │   │
│ │ by skill-engine | v1.2.0 | 2.3k installs             │   │
│ │ [View] [Install]                                      │   │
│ └───────────────────────────────────────────────────────┘   │
│ ┌───────────────────────────────────────────────────────┐   │
│ │ ⭐ aws-skill                           ★★★★☆ (89)     │   │
│ │ AWS services including S3, EC2, Lambda                │   │
│ │ by skill-engine | v2.0.1 | 1.8k installs             │   │
│ │ [View] [Install]                                      │   │
│ └───────────────────────────────────────────────────────┘   │
│                                                             │
│ Recently Added                                              │
│ ┌───────────────────────────────────────────────────────┐   │
│ │ 🆕 jira-skill                          ★★★☆☆ (12)     │   │
│ │ Jira issue tracking and project management            │   │
│ │ by community-user | v1.0.0 | 45 installs             │   │
│ │ [View] [Install]                                      │   │
│ └───────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] Marketplace data cached locally (5 min TTL)
- [ ] Shows "already installed" badge for installed skills
- [ ] Version comparison for updates available
- [ ] Fallback to GitHub search if registry unavailable

---

#### F3.3 RAG Pipeline Tuning Interface

**Priority**: P2 (Nice to Have)

**Description**: Visual interface for tuning search pipeline parameters

**Functional Requirements**:
- Adjust retrieval parameters:
  - Top-K results
  - Dense/sparse weight balance (hybrid search)
  - Reranking threshold
- Test queries with live results
- Compare different configurations
- Save presets

**UI Components**:
```
┌─────────────────────────────────────────────────────────────┐
│ Search Pipeline Tuning                        [Save Preset] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ ┌─────────────────────┐  ┌─────────────────────────────────┐│
│ │ Configuration       │  │ Test Query                      ││
│ │                     │  │                                 ││
│ │ Retrieval           │  │ ┌─────────────────────────────┐ ││
│ │ Top-K: [5    ]      │  │ │ list kubernetes pods       │ ││
│ │                     │  │ └─────────────────────────────┘ ││
│ │ Hybrid Search       │  │ [Search]                        ││
│ │ ☑ Enabled           │  │                                 ││
│ │                     │  │ Results (5)              [234ms]││
│ │ Dense Weight        │  │ ┌─────────────────────────────┐ ││
│ │ ├────●────────┤     │  │ │ 1. kubernetes:get_pods     │ ││
│ │ 0.7                 │  │ │    Score: 0.92             │ ││
│ │                     │  │ │    List pods in namespace  │ ││
│ │ Sparse Weight       │  │ ├─────────────────────────────┤ ││
│ │ ├──────────●──┤     │  │ │ 2. kubernetes:describe_pod │ ││
│ │ 0.3                 │  │ │    Score: 0.84             │ ││
│ │                     │  │ │    Describe a specific pod │ ││
│ │ Reranking           │  │ ├─────────────────────────────┤ ││
│ │ ☑ Enabled           │  │ │ 3. docker:ps               │ ││
│ │ Min Score: [0.5]    │  │ │    Score: 0.71             │ ││
│ │                     │  │ │    List running containers │ ││
│ │ [Apply Changes]     │  │ └─────────────────────────────┘ ││
│ └─────────────────────┘  └─────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] Changes apply in real-time for testing
- [ ] "Apply Changes" persists to `search.toml`
- [ ] Shows score breakdown (dense, sparse, rerank)
- [ ] Presets exportable as TOML snippets

---

#### F3.4 Skill Development Mode

**Priority**: P2 (Nice to Have)

**Description**: Tools for skill authors to develop and test skills

**Functional Requirements**:
- File watcher for local skill development
- Auto-reload on SKILL.md or WASM changes
- Validation report with errors and warnings
- Test runner for skill unit tests
- AI-powered example generation (`skill enhance`)
- SKILL.md preview with syntax highlighting

**UI Components**:
```
┌─────────────────────────────────────────────────────────────┐
│ Skill Development: ./my-skill                [📁 Open Folder]│
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Status: 🟢 Watching for changes                             │
│ Last reload: 2 seconds ago                                  │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│ ┌─────────┬──────────┬───────────┬─────────────────────────┐│
│ │  Editor │ Validate │   Test    │    Generate Examples    ││
│ └─────────┴──────────┴───────────┴─────────────────────────┘│
│                                                             │
│ SKILL.md Preview                                            │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ---                                                     │ │
│ │ name: my-skill                                          │ │
│ │ description: My custom skill                            │ │
│ │ allowed-tools: Bash                                     │ │
│ │ ---                                                     │ │
│ │                                                         │ │
│ │ ## Tools Provided                                       │ │
│ │                                                         │ │
│ │ ### hello                                               │ │
│ │ Say hello to someone.                                   │ │
│ │                                                         │ │
│ │ **Parameters**:                                         │ │
│ │ - `name` (required, string): Person to greet           │ │
│ │                                                         │ │
│ │ ```bash                                                 │ │
│ │ echo "Hello, $name!"                                    │ │
│ │ ```                                                     │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ Validation Results                              [✓ Valid]   │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ✓ Metadata valid                                        │ │
│ │ ✓ 1 tool defined                                        │ │
│ │ ⚠️ Tool 'hello' missing examples (optional)            │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] File watcher detects changes within 500ms
- [ ] Validation runs automatically on change
- [ ] SKILL.md renders with Markdown formatting
- [ ] Generate Examples calls AI endpoint

---

#### F3.5 Multi-User Support (Future)

**Priority**: P3 (Future)

**Description**: Support for multiple users with permissions

**Functional Requirements**:
- User authentication (local or SSO)
- Role-based access control:
  - Admin: Full access
  - Developer: Execute, configure
  - Viewer: Read-only
- Audit logging for compliance
- Shared skill configurations

*Note: This is a future consideration for enterprise deployments*

---

### Phase 4: Integration & Polish

#### F4.1 Claude Code Integration Panel

**Priority**: P1 (Should Have)

**Description**: One-click Claude Code integration management

**Functional Requirements**:
- Show current Claude Code integration status
- Generate/update `.mcp.json` configuration
- Test MCP connection
- Show registered tools in Claude Code
- Troubleshooting guide

**UI Components**:
```
┌─────────────────────────────────────────────────────────────┐
│ Claude Code Integration                                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Status: 🟢 Connected                                        │
│ MCP Server: Running on stdio                                │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│ Configuration                                               │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ .mcp.json                                               │ │
│ │ {                                                       │ │
│ │   "mcpServers": {                                       │ │
│ │     "skill-engine": {                                   │ │
│ │       "command": "skill",                              │ │
│ │       "args": ["serve"]                                │ │
│ │     }                                                   │ │
│ │   }                                                     │ │
│ │ }                                                       │ │
│ └─────────────────────────────────────────────────────────┘ │
│ [Copy to Clipboard] [Open File Location]                    │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│                                                             │
│ Registered Tools (32)                                       │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ • execute - Execute any skill tool                     │ │
│ │ • list_skills - List available skills                  │ │
│ │ • search_skills - Semantic skill search                │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ [🔄 Regenerate Config] [🧪 Test Connection]                 │
│                                                             │
│ Having issues? [View Troubleshooting Guide]                 │
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] Status updates in real-time
- [ ] Config generation handles existing `.mcp.json`
- [ ] Test connection shows detailed results
- [ ] Troubleshooting guide covers common issues

---

#### F4.2 Settings & Preferences

**Priority**: P1 (Should Have)

**Description**: Global application settings

**Functional Requirements**:
- Theme selection (Light, Dark, System)
- Default output format (JSON, Raw, Formatted)
- Execution timeout defaults
- Auto-update preferences
- Data retention settings
- Export/Import configuration

**UI Components**:
```
┌─────────────────────────────────────────────────────────────┐
│ Settings                                                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Appearance                                                  │
│ ─────────────────────────────────────────────────────────── │
│ Theme: ○ Light  ● Dark  ○ System                           │
│                                                             │
│ Execution                                                   │
│ ─────────────────────────────────────────────────────────── │
│ Default timeout: [30   ] seconds                           │
│ Default output format: [JSON         ▼]                     │
│ ☑ Include execution metadata by default                    │
│                                                             │
│ Data                                                        │
│ ─────────────────────────────────────────────────────────── │
│ History retention: [1000 ] executions                      │
│ [Clear History] [Export Data] [Import Data]                │
│                                                             │
│ Updates                                                     │
│ ─────────────────────────────────────────────────────────── │
│ ☑ Check for updates automatically                          │
│ Current version: 0.2.2                                     │
│ [Check for Updates]                                        │
│                                                             │
│ Advanced                                                    │
│ ─────────────────────────────────────────────────────────── │
│ Log level: [Info        ▼]                                  │
│ [Open Logs Folder] [View Configuration Files]               │
│                                                             │
│                              [Reset to Defaults] [Save]     │
└─────────────────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] Theme changes apply immediately
- [ ] Settings persist across restarts
- [ ] Export produces valid JSON/TOML
- [ ] Import validates before applying

---

#### F4.3 Responsive Design & Accessibility

**Priority**: P1 (Should Have)

**Description**: Ensure UI works across devices and is accessible

**Functional Requirements**:
- Responsive layout for desktop, tablet, mobile
- Keyboard navigation support
- Screen reader compatibility
- High contrast mode
- Focus indicators
- ARIA labels

**Acceptance Criteria**:
- [ ] Usable on screens 320px and wider
- [ ] All actions accessible via keyboard
- [ ] WCAG 2.1 AA compliance
- [ ] Tab order is logical

---

## Technical Architecture

### Technology Stack

| Layer | Technology | Justification |
|-------|------------|---------------|
| **Frontend Framework** | Yew 0.21+ | Rust WASM, React-like components |
| **State Management** | Yewdux | Redux-like global state |
| **Routing** | yew-router | SPA client-side routing |
| **Styling** | TailwindCSS | Utility-first, build-time CSS |
| **Build Tool** | Trunk | WASM bundling, dev server |
| **HTTP Client** | gloo-net | Rust WASM HTTP requests |
| **Backend** | Axum | Existing skill-http crate |
| **WebSocket** | tokio-tungstenite | Real-time updates |
| **Storage** | SQLite | Execution history persistence |

### Project Structure

```
crates/
├── skill-web/                    # New crate
│   ├── Cargo.toml
│   ├── Trunk.toml               # Build configuration
│   ├── index.html               # Entry point
│   ├── tailwind.config.js       # Tailwind configuration
│   ├── input.css                # Tailwind entry
│   └── src/
│       ├── main.rs              # App entry point
│       ├── app.rs               # Root component
│       ├── router.rs            # Route definitions
│       ├── store/               # Yewdux stores
│       │   ├── mod.rs
│       │   ├── skills.rs        # Skills state
│       │   ├── executions.rs    # Execution history
│       │   └── settings.rs      # App settings
│       ├── components/          # Reusable components
│       │   ├── mod.rs
│       │   ├── navbar.rs
│       │   ├── sidebar.rs
│       │   ├── skill_card.rs
│       │   ├── tool_form.rs
│       │   ├── output_viewer.rs
│       │   ├── markdown.rs
│       │   └── ...
│       ├── pages/               # Route pages
│       │   ├── mod.rs
│       │   ├── dashboard.rs
│       │   ├── skills.rs
│       │   ├── skill_detail.rs
│       │   ├── run.rs
│       │   ├── history.rs
│       │   ├── settings.rs
│       │   └── onboarding/
│       │       ├── mod.rs
│       │       ├── welcome.rs
│       │       ├── search_setup.rs
│       │       ├── credentials.rs
│       │       └── complete.rs
│       ├── api/                 # API client
│       │   ├── mod.rs
│       │   ├── client.rs
│       │   ├── skills.rs
│       │   ├── executions.rs
│       │   └── config.rs
│       └── utils/               # Utilities
│           ├── mod.rs
│           ├── markdown.rs
│           └── formatting.rs
│
├── skill-http/                   # Enhanced (existing)
│   └── src/
│       ├── lib.rs
│       ├── server.rs            # Axum server
│       ├── routes/              # API routes
│       │   ├── mod.rs
│       │   ├── skills.rs        # GET/POST /api/skills
│       │   ├── executions.rs    # POST /api/execute
│       │   ├── config.rs        # GET/PUT /api/config
│       │   ├── search.rs        # POST /api/search
│       │   └── ws.rs            # WebSocket handler
│       └── handlers/            # Request handlers
│
└── skill-cli/                    # Enhanced (existing)
    └── src/
        └── commands/
            └── web.rs           # New: `skill web` command
```

### API Endpoints

```
REST API (skill-http)
─────────────────────────────────────────────────────────────

Skills
  GET    /api/skills                    List all skills
  GET    /api/skills/:name              Get skill details
  POST   /api/skills/install            Install skill
  DELETE /api/skills/:name              Uninstall skill
  GET    /api/skills/:name/tools        List skill tools
  GET    /api/skills/:name/instances    List instances
  POST   /api/skills/:name/instances    Create instance
  PUT    /api/skills/:name/instances/:id Update instance
  DELETE /api/skills/:name/instances/:id Delete instance

Execution
  POST   /api/execute                   Execute tool
  GET    /api/executions                List history
  GET    /api/executions/:id            Get execution details

Search
  POST   /api/search                    Semantic search
  GET    /api/search/config             Get search config
  PUT    /api/search/config             Update search config

Configuration
  GET    /api/config                    Get app config
  PUT    /api/config                    Update app config
  GET    /api/credentials               List credentials (masked)
  POST   /api/credentials               Add credential
  DELETE /api/credentials/:key          Remove credential

System
  GET    /api/health                    Health check
  GET    /api/version                   Version info
  WS     /api/ws                        WebSocket (real-time updates)
```

### WebSocket Events

```json
// Server → Client
{ "type": "execution_started", "id": "exec-123", "skill": "kubernetes", "tool": "get_pods" }
{ "type": "execution_output", "id": "exec-123", "chunk": "..." }
{ "type": "execution_completed", "id": "exec-123", "status": "success", "duration_ms": 234 }
{ "type": "skill_installed", "name": "github", "version": "1.0.0" }
{ "type": "skill_removed", "name": "old-skill" }
{ "type": "config_changed", "key": "search.embedding.provider", "value": "openai" }

// Client → Server
{ "type": "subscribe", "channels": ["executions", "skills"] }
{ "type": "unsubscribe", "channels": ["executions"] }
```

### Shared Types (Cross-Crate)

Create `skill-types` crate for shared types:

```rust
// crates/skill-types/src/lib.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSummary {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: SkillSource,
    pub tools_count: usize,
    pub instances_count: usize,
    pub last_used: Option<DateTime<Utc>>,
    pub execution_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDetail {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub source: SkillSource,
    pub skill_type: SkillType,
    pub tools: Vec<ToolDefinition>,
    pub instances: Vec<InstanceConfig>,
    pub readme: Option<String>,  // Rendered markdown
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub skill: String,
    pub tool: String,
    pub instance: Option<String>,
    pub args: serde_json::Value,
    pub options: ExecutionOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub id: String,
    pub status: ExecutionStatus,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub metadata: Option<ExecutionMetadata>,
}
```

### Build & Distribution

**Development**:
```bash
# Start backend
cargo run -p skill-http -- --port 3000

# Start frontend (separate terminal)
cd crates/skill-web && trunk serve --port 8080 --proxy-backend=http://localhost:3000/api
```

**Production Build**:
```bash
# Build WASM bundle
cd crates/skill-web && trunk build --release

# Embed in skill-http binary
cargo build --release -p skill-http --features embedded-ui
```

**Launch Command**:
```bash
# New CLI command
skill web                    # Start web UI on default port (3000)
skill web --port 8080        # Custom port
skill web --open             # Open browser automatically
```

### Embedding Strategy

The WASM bundle will be embedded in the `skill` binary using `rust-embed`:

```rust
// crates/skill-http/src/embedded.rs
#[derive(RustEmbed)]
#[folder = "../skill-web/dist"]
struct Assets;

// Serve embedded assets
async fn serve_static(path: &str) -> impl IntoResponse {
    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => serve_index().await,  // SPA fallback
    }
}
```

---

## Non-Functional Requirements

### Performance

| Metric | Target |
|--------|--------|
| Initial load time | < 1s (WASM + assets) |
| API response time | < 100ms (p95) |
| Execution start latency | < 50ms |
| Search response time | < 200ms |
| Memory usage | < 100MB (browser tab) |
| WASM bundle size | < 2MB (gzipped) |

### Security

- All API endpoints require same-origin (localhost only by default)
- Credentials never sent to frontend (masked display only)
- CSRF protection for state-changing operations
- Input sanitization for all user inputs
- Content Security Policy headers

### Reliability

- Graceful degradation if backend unavailable
- Automatic reconnection for WebSocket
- Offline indicator and retry UI
- Error boundaries prevent full-page crashes

### Compatibility

| Environment | Support |
|-------------|---------|
| Chrome | 90+ |
| Firefox | 88+ |
| Safari | 14+ |
| Edge | 90+ |
| Screen readers | NVDA, VoiceOver |

---

## Implementation Phases

### Phase 1: Foundation (MVP) - 8-10 tasks

1. Set up `skill-web` crate with Yew + Trunk
2. Implement `skill-http` REST API
3. Create basic routing and layout
4. Dashboard page
5. Skill browser page
6. Skill detail page
7. Execution interface
8. Instance configuration
9. WebSocket integration
10. `skill web` CLI command

### Phase 2: Onboarding - 4-5 tasks

1. Onboarding wizard component
2. Search pipeline setup page
3. Credential manager
4. Claude Code integration panel
5. First-run detection and flow

### Phase 3: Advanced Features - 6-8 tasks

1. Execution history with persistence
2. Analytics dashboard
3. RAG pipeline tuning
4. Skill development mode
5. Marketplace browser (if registry available)
6. Settings page
7. Theme support (dark/light)
8. Responsive design polish

### Phase 4: Polish & Distribution - 3-4 tasks

1. Accessibility audit and fixes
2. Performance optimization
3. Documentation
4. Binary embedding and release

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Onboarding completion | > 80% | Users who complete setup wizard |
| Daily active usage | > 60% | Users who return to web UI |
| Execution success rate | > 95% | Successful executions / total |
| Time to first execution | < 5 min | New user to first skill run |
| Feature adoption | > 50% | Users who use semantic search |

---

## Open Questions

1. **Offline Support**: Should the web UI work fully offline (PWA) or require backend connection?
2. **Multi-User**: Is multi-user support needed for v1, or is single-user sufficient?
3. **Mobile**: How important is mobile responsiveness for the initial release?
4. **Skill Registry**: Will there be a public skill registry to integrate with?
5. **Telemetry**: Should we include anonymous usage analytics?

---

## Appendix

### A. Yew Framework References

- [Yew Official Documentation](https://yew.rs/)
- [Yew Tutorial](https://yew.rs/docs/tutorial)
- [Yewdux State Management](https://github.com/intendednull/yewdux)
- [yew-router Routing](https://docs.rs/yew-router)
- [Trunk Build Tool](https://trunkrs.dev/)
- [TailwindCSS with Yew](https://github.com/trunk-rs/trunk/tree/main/examples/yew-tailwindcss)

### B. Related PRDs

- `rag-end-to-end-prd.md` - RAG pipeline implementation
- `persistent-server-workers-prd.md` - Server architecture
- `docker-skill-prd.md` - Docker skill execution

### C. Glossary

| Term | Definition |
|------|------------|
| **Skill** | A sandboxed capability unit (WASM or SKILL.md) |
| **Tool** | A specific function within a skill |
| **Instance** | A configured deployment of a skill |
| **RAG** | Retrieval-Augmented Generation for search |
| **MCP** | Model Context Protocol for AI integration |
| **WASM** | WebAssembly binary format |
| **Yew** | Rust framework for WASM web applications |
| **Trunk** | Build tool for Rust WASM applications |

---

*End of PRD*
