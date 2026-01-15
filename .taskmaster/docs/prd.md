# Skill Engine PRD - Implementation Plan

## Executive Summary

Building a **high-performance, secure skill execution engine** using Rust + WASM that enables "docker pull for skills" - a universal plugin system where developers create JavaScript/TypeScript skills that compile to portable WASM binaries, installable as both CLI tools and MCP servers.

## Key Decisions (From User Input)

1. **Distribution**: Hybrid model - central registry + Git/local installation
2. **Languages**: Multi-language support from start via WASM Component Model
3. **MCP Integration**: Dual-mode - CLI and MCP are equal first-class interfaces
4. **Configuration**: Instance-based (multiple installations of same skill with different configs)

## Research Foundation

### Anthropic Skills System Analysis
- Skills use **SKILL.md** format with YAML frontmatter (name, description, allowed-tools)
- **Progressive disclosure**: metadata (always) → instructions (when triggered) → resources (on-demand)
- Model-invoked discovery (not user-invoked like slash commands)
- Security: audit all files, scripts, external URLs before installing
- MCP integration: skills provide "how", MCP provides "what/where"

### WASM/Rust Architecture Findings
- **WASI Preview 2**: Capability-based security with resource handles (modern, secure)
- **Component Model**: WIT interfaces enable language-agnostic plugins
- **Performance**: AOT compilation + wasm-opt + Wizer pre-initialization
- **Security**: Sandboxed linear memory, no direct hardware access, explicit capabilities
- **CLI Patterns**: Command (stateless), Reactor (stateful), Library (host-integrated)

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Skill Engine CLI (skill command)                       │
│  - Rust binary                                          │
│  - Commands: install, list, run, remove, publish       │
└────────────────┬────────────────────────────────────────┘
                 │
      ┌──────────┴──────────┐
      │   Core Components    │
      ├─────────────────────┤
      │ 1. Registry Client  │ ← Hybrid: registry + git + local
      │ 2. WASM Runtime     │ ← Wasmtime + Component Model
      │ 3. Instance Manager │ ← Multiple configs per skill
      │ 4. MCP Server       │ ← Expose tools dynamically
      │ 5. CLI Executor     │ ← Direct binary execution
      └─────────────────────┘
                 │
      ┌──────────┴────────────────────────┐
      │                                    │
┌─────▼─────┐                    ┌────────▼────────┐
│ WASM Skill│                    │  MCP Protocol   │
│ Component │                    │  Tool Exposure  │
│ (.wasm)   │                    └─────────────────┘
└───────────┘
      │
      │ Implements WIT interface:
      │
┌─────▼──────────────────────────────────────────┐
│ skill-interface.wit                             │
│ ─────────────────────────────────────────────  │
│ world skill {                                   │
│   import wasi:cli/environment@0.2.0            │
│   import wasi:io/streams@0.2.0                 │
│   import wasi:filesystem/types@0.2.0           │
│                                                 │
│   export skill-metadata: func() -> metadata    │
│   export skill-tools: func() -> list<tool>     │
│   export skill-execute: func(tool, args) -> result │
│   export skill-validate: func() -> result      │
│ }                                               │
└─────────────────────────────────────────────────┘
```

## Core Components Breakdown

### 1. Skill Engine CLI (`skill` binary)

**Commands** (Modern, intuitive CLI design):
```bash
# ============================================
# INSTALLATION & DISCOVERY
# ============================================
skill search <query>                     # Search registry for skills
skill info <name>                        # Show detailed skill information
skill install <name>[@version]           # From registry (smart default)
skill install <name> -i <alias>          # Install with instance name
skill install <git-url>                  # From Git repository
skill install ./path                     # From local filesystem
skill pull-run <name> <tool> [args]      # One-time execution (docker run style)
  --config key=value                     # Temporary configuration
  --no-cache                             # Skip cache, always fresh
  --keep                                 # Don't cleanup after execution

# ============================================
# MANAGEMENT
# ============================================
skill list                               # Pretty table of all skills
skill list --json                        # JSON output for scripting
skill ls -l                              # Alias with long format
skill instances                          # Show all instances across skills
skill instances <name>                   # Show instances for specific skill
skill update <name>                      # Update to latest version
skill update --all                       # Update all skills
skill remove <name>                      # Remove skill (prompts for confirmation)
skill rm <name> -f                       # Force remove without prompt
skill prune                              # Clean up unused cached skills

# ============================================
# EXECUTION (Ergonomic & Flexible)
# ============================================
skill run <name>:<tool> [args]           # Run tool (preferred syntax)
skill run <name>@<instance>:<tool> [args] # Run with specific instance
skill exec <name> -- [cli-args]          # Pass-through to skill's native CLI
skill <name>:<tool> [args]               # Shorthand (when skill in PATH)

# Examples:
skill run aws:s3-upload --bucket my-bucket --file data.json
skill run aws@prod:s3-list mybucket
skill run aws@staging:ec2-list --region us-east-1
skill exec aws -- s3 ls s3://mybucket

# ============================================
# CONFIGURATION (Interactive & Scriptable)
# ============================================
skill config <name>                      # Interactive TUI wizard
skill config <name> -i <instance>        # Configure specific instance
skill config <name> --show               # Display current config
skill config <name> --edit               # Open in $EDITOR
skill config <name> --set key=value      # Set single value
skill config <name> --get key            # Get single value
skill config <name> --export > config.toml # Export configuration
skill config <name> --import config.toml # Import configuration

# ============================================
# DEVELOPMENT & SCAFFOLDING
# ============================================
skill init                               # Interactive project creation
skill init <name> -t typescript          # Create from specific template
skill init --list                        # List available templates
skill new <name>                         # Alias for init
skill build                              # Build current skill (in skill dir)
skill test                               # Run skill tests
skill watch                              # Watch mode for development
skill validate                           # Validate skill.yaml + WIT interface

# ============================================
# PUBLISHING
# ============================================
skill login                              # Authenticate with registry
skill publish                            # Publish current skill
skill publish --dry-run                  # Test publishing without uploading
skill yank <version>                     # Remove version from registry

# ============================================
# HTTP SERVER & MCP
# ============================================
skill serve                              # Start HTTP + MCP server
skill serve --http-only                  # HTTP REST API only
skill serve --mcp-only                   # MCP protocol only
skill serve --port 8080                  # Custom port (default: 3000)
skill serve --host 0.0.0.0               # Bind to specific host
skill serve --stream                     # Enable SSE streaming (on by default)
skill serve <name>                       # Serve specific skill only
skill serve --tls-cert cert.pem --tls-key key.pem # HTTPS support

# Server modes:
# - REST API: /api/v1/skills, /api/v1/execute, /api/v1/stream
# - MCP: stdio, HTTP SSE, WebSocket
# - Metrics: /metrics (Prometheus format)
# - Health: /health, /ready

# ============================================
# UTILITIES
# ============================================
skill version                            # Show skill-engine version
skill doctor                             # Diagnose installation issues
skill completion bash > ~/.bashrc        # Shell completion
skill help                               # Show help
skill <command> --help                   # Command-specific help
```

**Critical Files**:
- `~/.skill-engine/config.toml` - Global configuration
- `~/.skill-engine/registry/` - Downloaded skill binaries
- `~/.skill-engine/instances/` - Instance configurations
- `/usr/local/bin/skill` - Main CLI binary

### 2. Registry Architecture

**Hybrid Model Implementation**:

**A. Central Registry (like npm/crates.io)**
```
Registry API Endpoints:
- GET  /api/v1/skills?search=aws         # Search skills
- GET  /api/v1/skills/<name>             # Skill metadata
- GET  /api/v1/skills/<name>/versions    # Version list
- GET  /api/v1/skills/<name>/<version>   # Download binary
- POST /api/v1/skills                    # Publish (auth required)

Metadata Structure:
{
  "name": "aws-skill",
  "version": "1.2.3",
  "description": "AWS CLI integration skill",
  "author": "username",
  "repository": "https://github.com/user/aws-skill",
  "wasm_url": "https://registry.skill-engine.io/aws-skill-1.2.3.wasm",
  "wit_interface": "skill-interface@1.0.0",
  "dependencies": [],
  "tools": ["s3-upload", "ec2-list", "lambda-invoke"],
  "config_schema": { "aws_access_key_id": "string", ... }
}
```

**B. Git Installation**
```bash
skill install https://github.com/user/aws-skill.git
# Clones repo, runs build script, installs binary
```

**C. Local Installation**
```bash
skill install ./my-skill/
# Validates, compiles (if source), installs
```

### 3. WASM Runtime (Wasmtime + Component Model)

**Key Technologies**:
- **Wasmtime**: Bytecode Alliance's secure WASM runtime
- **WASI Preview 2**: Modern capability-based system interface
- **Component Model**: WIT-based language-agnostic components
- **wasmtime-wasi crate**: Rust bindings for WASI

**Security Sandbox**:
```rust
// Pseudo-code for skill execution
let engine = Engine::default();
let mut config = Config::new();
config.wasm_component_model(true);
config.async_support(true);

let component = Component::from_file(&engine, "skill.wasm")?;
let mut linker = Linker::new(&engine);

// Grant ONLY required capabilities
wasmtime_wasi::add_to_linker_async(&mut linker)?;

// Restrict filesystem access
let preopened_dirs = vec![
    "/tmp/skill-sandbox",           // Temporary workspace
    "~/.skill-engine/instances/aws-prod/"  // Instance config
];

let wasi = WasiCtxBuilder::new()
    .inherit_stdio()
    .env("INSTANCE_ID", "aws-prod")
    .preopened_dirs(preopened_dirs)
    .build();

let mut store = Store::new(&engine, wasi);
let instance = linker.instantiate_async(&mut store, &component).await?;
```

### 4. Instance Management System

**Instance Model** (inspired by Docker containers):
- Each instance = separate installation with unique configuration
- Multiple instances of same skill = different credentials/settings
- Instances stored in `~/.skill-engine/instances/<skill>/<instance-name>/`

**Example: AWS Skill with Multiple Accounts**
```bash
# Install and configure production account
skill install aws-skill --instance prod
skill config aws-skill --instance prod --set aws_access_key_id=AKIAXXXPROD
skill config aws-skill --instance prod --set aws_secret_access_key=***
skill config aws-skill --instance prod --set region=us-east-1

# Install and configure staging account
skill install aws-skill --instance staging
skill config aws-skill --instance staging --set aws_access_key_id=AKIAXXXSTAGE
skill config aws-skill --instance staging --set region=eu-west-1

# Use specific instance
skill run aws-skill --instance prod s3-list mybucket
skill run aws-skill --instance staging ec2-list
```

**Instance Configuration Structure**:
```toml
# ~/.skill-engine/instances/aws-skill/prod/config.toml
[metadata]
skill_name = "aws-skill"
skill_version = "1.2.3"
instance_name = "prod"
created_at = "2025-12-18T10:00:00Z"

[config]
aws_access_key_id = "AKIAXXXPROD"
aws_secret_access_key = "***"  # Encrypted at rest
region = "us-east-1"
output_format = "json"

[capabilities]
allowed_services = ["s3", "ec2", "lambda"]
max_concurrent_requests = 10

[environment]
# Virtual env vars passed to WASM module
AWS_PROFILE = "production"
AWS_DEFAULT_REGION = "us-east-1"
```

**Virtual Environment Transfer**:
- Configuration values mapped to environment variables
- Files mounted as virtual filesystem paths
- Credentials encrypted at rest, decrypted only during execution
- WASI capability grants restrict access to instance directory only

### 5. HTTP Server & REST API

**High-Performance Rust HTTP Server** (using Axum + Tokio):

```rust
// Modern async Rust web framework with best practices
use axum::{
    Router, extract::{State, Path, Json},
    response::{IntoResponse, Sse, sse::Event},
    http::StatusCode,
};
use tokio::sync::mpsc;
use tower_http::{
    trace::TraceLayer,
    cors::CorsLayer,
    compression::CompressionLayer,
};

// High-performance server with streaming
#[tokio::main]
async fn main() {
    let app = Router::new()
        // REST API endpoints
        .route("/api/v1/skills", get(list_skills))
        .route("/api/v1/skills/:name", get(get_skill_info))
        .route("/api/v1/execute", post(execute_skill))
        .route("/api/v1/stream", post(execute_skill_stream))

        // Health & metrics
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/metrics", get(metrics))

        // Middleware stack (best practices)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(AppState::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

**REST API Endpoints**:

```
GET  /api/v1/skills
Response: List of all available skills
{
  "skills": [
    {
      "name": "aws-skill",
      "version": "1.2.3",
      "instances": ["prod", "staging"],
      "tools": ["s3-upload", "s3-list", "ec2-list"]
    }
  ]
}

GET  /api/v1/skills/:name
Response: Detailed skill information
{
  "name": "aws-skill",
  "version": "1.2.3",
  "description": "AWS CLI integration",
  "tools": [
    {
      "name": "s3-upload",
      "description": "Upload file to S3",
      "parameters": [...]
    }
  ],
  "config_schema": {...}
}

POST /api/v1/execute
Request:
{
  "skill": "aws-skill",
  "instance": "prod",
  "tool": "s3-upload",
  "args": {
    "bucket": "mybucket",
    "key": "file.txt",
    "file_path": "/tmp/data.json"
  }
}
Response:
{
  "success": true,
  "output": "Uploaded successfully",
  "execution_time_ms": 234
}

POST /api/v1/stream (Server-Sent Events)
Request: Same as /execute
Response: SSE stream
event: start
data: {"skill": "aws-skill", "tool": "s3-upload"}

event: progress
data: {"percent": 25, "message": "Uploading..."}

event: progress
data: {"percent": 75, "message": "Finalizing..."}

event: done
data: {"success": true, "output": "Upload complete"}

event: close
```

**Streaming Support** (Server-Sent Events):

```rust
async fn execute_skill_stream(
    State(state): State<AppState>,
    Json(req): Json<ExecuteRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::channel(100);

    // Spawn background task for skill execution
    tokio::spawn(async move {
        // Send start event
        tx.send(Event::default()
            .event("start")
            .json_data(json!({"skill": req.skill, "tool": req.tool}))
        ).await;

        // Execute skill with progress callbacks
        let result = execute_skill_with_progress(&req, |progress| {
            tx.send(Event::default()
                .event("progress")
                .json_data(progress)
            ).await;
        }).await;

        // Send completion event
        tx.send(Event::default()
            .event("done")
            .json_data(result)
        ).await;

        // Close stream
        tx.send(Event::default().event("close")).await;
    });

    Sse::new(ReceiverStream::new(rx))
}
```

**Performance Optimizations**:
- Tokio async runtime (multi-threaded work-stealing)
- Zero-copy where possible (bytes crate)
- Connection pooling for database/registry
- Response compression (gzip, br)
- HTTP/2 support
- Graceful shutdown
- Request timeouts and backpressure
- Efficient JSON serialization (serde_json)

**Observability**:
```rust
// Structured logging with tracing
use tracing::{info, warn, error, instrument};

#[instrument(skip(state))]
async fn execute_skill(
    State(state): State<AppState>,
    Json(req): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, AppError> {
    info!(skill = %req.skill, tool = %req.tool, "Executing skill");

    let start = Instant::now();
    let result = state.runtime.execute(&req).await?;
    let duration = start.elapsed();

    info!(
        skill = %req.skill,
        tool = %req.tool,
        duration_ms = duration.as_millis(),
        "Skill execution completed"
    );

    Ok(Json(result))
}

// Prometheus metrics
use prometheus::{Counter, Histogram, Registry};

lazy_static! {
    static ref SKILL_EXECUTIONS: Counter =
        Counter::new("skill_executions_total", "Total skill executions").unwrap();
    static ref EXECUTION_DURATION: Histogram =
        Histogram::new("skill_execution_duration_seconds", "Execution duration").unwrap();
}
```

**WebSocket Support** (for bidirectional streaming):
```rust
// WebSocket endpoint for interactive skills
.route("/api/v1/ws", get(websocket_handler))

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                // Parse execute request
                let req: ExecuteRequest = serde_json::from_str(&text)?;

                // Execute with streaming
                let mut stream = state.runtime.execute_stream(&req).await?;

                while let Some(chunk) = stream.next().await {
                    sender.send(Message::Text(
                        serde_json::to_string(&chunk)?
                    )).await?;
                }
            }
            _ => {}
        }
    }
}
```

**TLS/HTTPS Support**:
```rust
use axum_server::tls_rustls::RustlsConfig;

// Load TLS certificates
let config = RustlsConfig::from_pem_file(
    "cert.pem",
    "key.pem",
).await?;

// Serve with TLS
axum_server::bind_rustls("0.0.0.0:443".parse()?, config)
    .serve(app.into_make_service())
    .await?;
```

### 6. MCP Server Integration

**Dynamic Tool Exposure**:
```json
// MCP Server exposes tools based on installed skills
{
  "tools": [
    {
      "name": "mcp__skill_aws-skill_prod__s3-upload",
      "description": "Upload file to S3 bucket (prod account)",
      "inputSchema": {
        "type": "object",
        "properties": {
          "bucket": {"type": "string"},
          "key": {"type": "string"},
          "file_path": {"type": "string"}
        }
      }
    },
    {
      "name": "mcp__skill_aws-skill_staging__s3-upload",
      "description": "Upload file to S3 bucket (staging account)",
      "inputSchema": { /* same schema */ }
    }
  ]
}
```

**MCP Server Implementation**:
```rust
// skill serve → starts MCP server
// Discovers all installed skill instances
// Queries each skill for exposed tools via WIT interface
// Dynamically constructs MCP tool catalog
// Routes tool calls to appropriate skill instance
```

**Tool Naming Convention**:
```
mcp__skill_<skill-name>_<instance>__<tool-name>

Examples:
- mcp__skill_aws-skill_prod__s3-upload
- mcp__skill_github-skill_work__create-issue
- mcp__skill_slack-skill_team__send-message
```

### 7. WIT Interface Definition (Enhanced)

**Standard Skill Interface** (`skill-interface.wit`):
```wit
// skill-interface.wit v1.0.0
// Complete, production-ready WIT interface

package skill-engine:interface@1.0.0;

// ============================================
// Core Types
// ============================================

record skill-metadata {
    name: string,
    version: string,
    description: string,
    author: string,
    repository: option<string>,
    license: option<string>,
}

record tool-definition {
    name: string,
    description: string,
    parameters: list<parameter>,
    streaming: bool,  // Does this tool support streaming?
}

record parameter {
    name: string,
    param-type: param-type-enum,
    description: string,
    required: bool,
    default-value: option<string>,
}

enum param-type-enum {
    string,
    number,
    boolean,
    file,
    json,
    array,
}

record execution-result {
    success: bool,
    output: string,
    error-message: option<string>,
    metadata: option<list<tuple<string, string>>>,  // Key-value pairs
}

record stream-chunk {
    chunk-type: stream-chunk-type,
    data: string,
}

enum stream-chunk-type {
    stdout,
    stderr,
    progress,
    metadata,
}

record config-value {
    key: string,
    value: string,
    secret: bool,
}

record dependency {
    skill-name: string,
    version-constraint: string,  // Semver constraint like "^1.0.0"
    optional: bool,
}

// ============================================
// World Definition - Skills implement this
// ============================================

world skill {
    // Import WASI capabilities (Preview 2)
    import wasi:cli/environment@0.2.0;
    import wasi:io/streams@0.2.0;
    import wasi:filesystem/types@0.2.0;
    import wasi:filesystem/preopens@0.2.0;
    import wasi:random/random@0.2.0;
    import wasi:clocks/wall-clock@0.2.0;
    import wasi:clocks/monotonic-clock@0.2.0;

    // Optional: Network capabilities (if skill needs HTTP)
    // import wasi:http/types@0.2.0;
    // import wasi:http/outgoing-handler@0.2.0;

    // ============================================
    // Required Exports - All skills must implement
    // ============================================

    // Skill metadata
    export get-metadata: func() -> skill-metadata;

    // Tool discovery
    export get-tools: func() -> list<tool-definition>;

    // Tool execution (non-streaming)
    export execute-tool: func(
        tool-name: string,
        args: list<tuple<string, string>>  // Key-value arguments
    ) -> result<execution-result, string>;

    // Configuration validation
    export validate-config: func(
        config: list<config-value>
    ) -> result<_, string>;

    // ============================================
    // Optional Exports - Skills can implement
    // ============================================

    // Streaming execution
    export execute-tool-stream: func(
        tool-name: string,
        args: list<tuple<string, string>>
    ) -> result<list<stream-chunk>, string>;

    // Dependency declaration
    export get-dependencies: func() -> list<dependency>;

    // Health check
    export health-check: func() -> result<_, string>;

    // Lifecycle hooks
    export on-install: func() -> result<_, string>;
    export on-configure: func(config: list<config-value>) -> result<_, string>;
    export on-uninstall: func() -> result<_, string>;
}

// ============================================
// Host Interface - What the host provides
// ============================================

world host {
    // Host provides these capabilities to skills
    export get-config: func(key: string) -> option<string>;
    export set-config: func(key: string, value: string, secret: bool) -> result<_, string>;
    export log: func(level: log-level, message: string);
    export emit-progress: func(percent: u8, message: string);
}

enum log-level {
    trace,
    debug,
    info,
    warn,
    error,
}
```

## Skill Authoring Experience

### JavaScript/TypeScript Skill Creation

**Project Structure**:
```
my-skill/
├── skill.yaml              # Skill manifest
├── src/
│   ├── index.ts           # Main entry point
│   ├── tools/
│   │   ├── upload.ts      # Tool implementations
│   │   └── list.ts
│   └── config.ts          # Config schema
├── package.json
├── tsconfig.json
└── README.md
```

**Skill Manifest** (`skill.yaml`):
```yaml
name: aws-skill
version: 1.2.3
description: AWS service integration for S3, EC2, Lambda
author: username
repository: https://github.com/user/aws-skill

# Component Model settings
component_model: true
wit_interface: skill-engine:interface@1.0.0

# Configuration schema
config:
  aws_access_key_id:
    type: string
    required: true
    secret: true
  aws_secret_access_key:
    type: string
    required: true
    secret: true
  region:
    type: string
    required: false
    default: us-east-1

# Tools exposed
tools:
  - name: s3-upload
    description: Upload file to S3 bucket
    handler: tools/upload.ts:s3Upload
  - name: s3-list
    description: List S3 bucket contents
    handler: tools/list.ts:s3List

# Package dependencies (npm/cargo packages available during compilation)
dependencies:
  npm:
    - "@aws-sdk/client-s3": "^3.0.0"

# Skill dependencies (other skills required by this skill)
skill_dependencies:
  - name: auth-skill
    version: "^1.0.0"
    optional: false
  - name: logging-skill
    version: ">=2.1.0"
    optional: true
```

**TypeScript Implementation** (`src/tools/upload.ts`):
```typescript
import { S3Client, PutObjectCommand } from '@aws-sdk/client-s3';
import { getConfig } from '../config';
import * as fs from 'fs';

export async function s3Upload(args: {
  bucket: string;
  key: string;
  filePath: string;
}): Promise<{ success: boolean; output: string }> {
  const config = getConfig();

  const client = new S3Client({
    credentials: {
      accessKeyId: config.aws_access_key_id,
      secretAccessKey: config.aws_secret_access_key,
    },
    region: config.region,
  });

  const fileContent = fs.readFileSync(args.filePath);

  const command = new PutObjectCommand({
    Bucket: args.bucket,
    Key: args.key,
    Body: fileContent,
  });

  await client.send(command);

  return {
    success: true,
    output: `Uploaded ${args.filePath} to s3://${args.bucket}/${args.key}`,
  };
}
```

**Build Process**:
```bash
# Developer builds skill locally
npm install
npm run build  # Compiles TS → JS → WASM via javy/componentize-js

# Output: dist/aws-skill.wasm (component)

# Test locally
skill install ./dist/aws-skill.wasm --instance test
skill run aws-skill --instance test s3-list my-bucket

# Publish to registry
skill publish
```

### Multi-Language Support (Rust Example)

**Rust Skill** (`src/lib.rs`):
```rust
use skill_engine_sdk::*;

wit_bindgen::generate!({
    world: "skill",
    exports: {
        world: Component,
    },
});

struct Component;

impl Guest for Component {
    fn get_metadata() -> SkillMetadata {
        SkillMetadata {
            name: "rust-skill".to_string(),
            version: "0.1.0".to_string(),
            description: "High-performance skill in Rust".to_string(),
            author: "username".to_string(),
        }
    }

    fn get_tools() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "process-data".to_string(),
                description: "Process large datasets efficiently".to_string(),
                parameters: vec![
                    Parameter {
                        name: "input".to_string(),
                        param_type: "file".to_string(),
                        description: "Input data file".to_string(),
                        required: true,
                    }
                ],
            }
        ]
    }

    fn execute_tool(tool_name: String, args: Vec<String>) -> ExecutionResult {
        match tool_name.as_str() {
            "process-data" => {
                // High-performance Rust implementation
                ExecutionResult {
                    success: true,
                    output: "Data processed".to_string(),
                    error_message: None,
                }
            }
            _ => ExecutionResult {
                success: false,
                output: "".to_string(),
                error_message: Some(format!("Unknown tool: {}", tool_name)),
            }
        }
    }

    fn validate_config() -> Result<(), String> {
        // Validate instance configuration
        Ok(())
    }
}
```

## Security Model

### Capability-Based Access Control

**Principles**:
1. **Least Privilege**: Skills only get capabilities they explicitly request
2. **Explicit Grants**: Host must approve all filesystem/network access
3. **Instance Isolation**: Each instance has separate sandbox
4. **Audit Trail**: All capability grants logged

**Pre-installation Audit**:
```bash
skill install aws-skill
# Output:
#
# ⚠️  Skill Capabilities Review:
#
# aws-skill v1.2.3 requests:
#   ✓ Filesystem: Read/write to instance config directory
#   ✓ Network: HTTPS access to *.amazonaws.com
#   ✓ Environment: AWS_* environment variables
#   ⚠️ Secrets: Will store AWS credentials (encrypted)
#
# Install? [y/N]
```

**Runtime Sandboxing**:
- WASM linear memory isolation (no cross-module access)
- WASI Preview 2 resource handles (unforgeable capabilities)
- No direct syscalls (all through WASI interface)
- Filesystem access limited to pre-opened directories
- Network access controlled by host (if enabled)

### Credential Encryption

**At Rest**:
- Secrets encrypted using system keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Master key derived from user password/system entropy
- Per-instance encryption keys

**In Transit**:
- Environment variables passed securely to WASM module
- No plaintext credentials in process arguments
- Memory cleared after execution

## Performance Optimization

### Compilation Strategy
1. **AOT Compilation**: Skills pre-compiled to native code during installation
2. **Wizer Pre-initialization**: Expensive setup (loading configs, init libraries) done once
3. **Module Caching**: Compiled modules cached for fast subsequent runs

**Build Pipeline**:
```
Source (JS/TS/Rust)
  → WASM Component (.wasm)
  → wasm-opt optimization (-Os/-Oz)
  → Wizer pre-initialization
  → Wasmtime AOT compilation (.cwasm)
  → Installed binary
```

### Startup Time Targets
- Cold start: <100ms (first run after install)
- Warm start: <10ms (cached compilation)
- MCP tool call: <50ms (excluding skill execution time)

### Binary Size Optimization
- wasm-opt aggressive optimization
- Strip unused code/data
- Gzip compression for distribution
- Target: <5MB for average skill

## CLI Interface Design

### Dual-Mode Execution

**A. Direct CLI Mode** (skill runs as native binary):
```bash
# Transparent pass-through
skill exec aws-skill --instance prod -- aws s3 ls s3://mybucket

# Equivalent to running a native AWS CLI with profile
```

**B. Tool Mode** (skill exposes individual tools):
```bash
# Structured tool invocation
skill run aws-skill --instance prod s3-upload \
  --bucket mybucket \
  --key file.txt \
  --file-path ./local-file.txt
```

**C. Pull-Run Mode** (one-time execution, like `docker run`):
```bash
# Install temporarily, execute, and cleanup
skill pull-run aws-skill s3-list --bucket mybucket

# With instance configuration
skill pull-run aws-skill --config aws_access_key_id=XXX s3-list mybucket

# From specific source
skill pull-run https://github.com/user/aws-skill.git s3-list mybucket

# Behavior:
# 1. Check if skill is cached in /tmp/skill-engine/cache/
# 2. If not cached, download/install to temp location
# 3. Execute the tool with provided arguments
# 4. Optionally cleanup (--rm flag, default true)
# 5. Keep in cache for subsequent pull-runs (expires after 24h)
```

### Skill Template Generation

**Template System** (`skill init`):
```bash
# List available templates
skill init --list
# Output:
#   Available templates:
#   - javascript    Basic JavaScript skill with single tool
#   - typescript    TypeScript skill with build tooling
#   - rust          Rust skill with Component Model bindings
#   - go            Go skill with TinyGo WASM compilation
#   - python        Python skill with componentize-py
#   - multi-tool-js Complex JS skill with multiple tools and config

# Create new skill from template
skill init my-new-skill --template typescript
# Creates:
#   my-new-skill/
#   ├── skill.yaml
#   ├── src/
#   │   ├── index.ts
#   │   └── tools/
#   │       └── example-tool.ts
#   ├── package.json
#   ├── tsconfig.json
#   ├── README.md
#   └── .gitignore

# Interactive mode (asks questions, generates custom template)
skill init
# Prompts:
#   - Skill name?
#   - Language? [JavaScript/TypeScript/Rust/Go/Python]
#   - Does it need configuration? [y/N]
#   - How many tools? [1]
#   - Tool names? (comma separated)
```

**Template Features**:
- Pre-configured build scripts and toolchains
- Example tool implementations with comments
- Configuration schema examples
- README with usage instructions
- Build + test + publish workflow
- GitHub Actions CI/CD templates

### PATH Integration

**Symlink Pattern**:
```bash
# After installation, create symlinks:
~/.skill-engine/bin/aws-skill-prod -> skill exec aws-skill --instance prod
~/.skill-engine/bin/aws-skill-staging -> skill exec aws-skill --instance staging

# Add to PATH in ~/.bashrc:
export PATH="$HOME/.skill-engine/bin:$PATH"

# Now can call directly:
aws-skill-prod s3 ls
aws-skill-staging ec2 describe-instances
```

## Implementation Phases

### Phase 1: Core Runtime (MVP)
**Goal**: Basic skill execution with single instance support

**Deliverables**:
1. Rust CLI (`skill` binary) with basic commands (install local, run, remove)
2. Wasmtime + Component Model integration
3. WIT interface definition (skill-interface.wit)
4. WASI Preview 2 capability system
5. Basic configuration management (TOML files)
6. JavaScript → WASM build toolchain (using javy/componentize-js)
7. Sample skill (simple-skill) demonstrating pattern

**Success Metrics**:
- Install skill from local path
- Execute skill tool via CLI
- Configuration persists between runs
- Sandbox prevents unauthorized file access

### Phase 2: Instance Management
**Goal**: Multiple instances with credential isolation

**Deliverables**:
1. Instance creation/deletion commands
2. Per-instance configuration storage
3. Credential encryption (system keychain integration)
4. Virtual environment mapping (config → env vars)
5. Instance-aware CLI execution
6. Sample skill with multi-account support (aws-skill)

**Success Metrics**:
- Multiple instances of same skill work independently
- Credentials encrypted at rest
- No cross-instance data leakage

### Phase 3: MCP Server Integration
**Goal**: Skills exposed as MCP tools

**Deliverables**:
1. MCP server implementation (`skill serve`)
2. Dynamic tool discovery from installed skills
3. Tool naming convention (mcp__skill_...)
4. MCP tool invocation → skill execution bridge
5. Error handling and logging
6. Integration test with Claude Code

**Success Metrics**:
- Claude Code can discover and call skill tools
- Tool calls execute correctly with proper authentication
- Errors surface appropriately in Claude Code

### Phase 4: Registry System
**Goal**: Central registry for skill distribution

**Deliverables**:
1. Registry API server (REST endpoints)
2. Skill metadata database
3. Binary storage (S3/CDN)
4. `skill publish` command (with authentication)
5. `skill search` command
6. Version management and updates
7. Basic web UI for browsing skills

**Success Metrics**:
- Publish skill to registry
- Install skill by name from registry
- Update to newer version works correctly

### Phase 5: Git/Hybrid Installation
**Goal**: Install skills from Git repositories

**Deliverables**:
1. Git clone and build pipeline
2. Build script detection (npm, cargo, etc.)
3. Source validation and sandboxed builds
4. Local path installation with validation
5. Update detection for Git-installed skills

**Success Metrics**:
- Install skill from GitHub URL
- Install skill from local directory
- Automatic rebuilds on source changes (dev mode)

### Phase 6: Advanced Features
**Goal**: Production-ready ecosystem

**Deliverables**:
1. **Dependency resolution between skills**
   - Skill-to-skill dependency graph
   - Automatic installation of required skills
   - Version compatibility checking
   - Circular dependency detection
   - Optional vs required dependencies
2. **Skill versioning with semver**
   - Semantic versioning enforcement
   - Lock files for reproducible builds
   - Version constraints (^, ~, >=, etc.)
3. **Pull-run feature** (one-time execution)
   - Temporary skill cache in /tmp
   - Automatic cleanup after execution
   - Cache expiration (24h default)
4. **Skill template system** (`skill init`)
   - Multiple language templates (JS/TS/Rust/Go/Python)
   - Interactive scaffolding
   - Pre-configured build tooling
5. **Rollback/pinning to specific versions**
6. **Performance monitoring and profiling**
7. **Audit logging for security**
8. **Documentation generator**
9. **CI/CD integration examples**

### Phase 7: Multi-Language SDK
**Goal**: First-class support for Rust, Go, Python skills

**Deliverables**:
1. Rust SDK (`skill-engine-sdk` crate)
2. Go SDK (WIT bindings)
3. Python SDK (componentize-py)
4. Build tooling for each language
5. Example skills in each language
6. Language-specific optimization guides

## Critical Files to Create

### Project Structure
```
skill-engine/
├── Cargo.toml                    # Workspace definition
├── README.md                     # Project overview
├── LICENSE                       # Open source license
│
├── crates/
│   ├── skill-cli/               # Main CLI binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs          # Entry point
│   │       ├── commands/        # Command implementations
│   │       └── config.rs        # Config management
│   │
│   ├── skill-runtime/           # WASM execution engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── component.rs     # Component Model integration
│   │       ├── sandbox.rs       # Capability control
│   │       └── instance.rs      # Instance management
│   │
│   ├── skill-mcp/               # MCP server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── server.rs        # MCP protocol
│   │       └── tools.rs         # Tool exposure
│   │
│   └── skill-registry-client/   # Registry API client
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── api.rs           # HTTP client
│
├── wit/
│   └── skill-interface.wit      # Component interface
│
├── examples/
│   ├── simple-skill/            # Basic example (JS)
│   ├── aws-skill/               # Complex example (JS + AWS SDK)
│   └── rust-skill/              # Native Rust example
│
├── sdk/
│   ├── javascript/              # JS/TS SDK
│   │   ├── package.json
│   │   └── src/
│   ├── rust/                    # Rust SDK
│   │   ├── Cargo.toml
│   │   └── src/
│   └── python/                  # Python SDK (future)
│
└── registry-server/             # Registry backend (Phase 4)
    ├── Cargo.toml
    └── src/
```

## Open Questions & Risks

### Technical Risks
1. **WASM Startup Performance**: Can we achieve <100ms cold start?
   - Mitigation: AOT compilation + Wizer pre-init + caching
2. **JavaScript WASM Size**: JS skills may be large (QuickJS embedded)
   - Mitigation: Aggressive wasm-opt, consider SpiderMonkey bindings
3. **Network Access in WASI**: WASI Preview 2 sockets still experimental
   - Mitigation: Use preview builds or polyfill via host imports
4. **Cross-Platform Credential Storage**: Different keychains per OS
   - Mitigation: Use keyring-rs crate (abstraction)

### Product Questions
1. **Should we support skill dependencies?** (One skill importing another)
   - Decision: Yes, but Phase 6 (not MVP)
2. **Versioning strategy?** Semver, lock files, etc.
   - Decision: Follow npm/cargo patterns (Phase 4)
3. **Sandboxing strictness?** Allow network by default or opt-in?
   - Decision: Explicit opt-in for network (security first)

## Success Criteria

### Phase 1 (MVP) Success
- [ ] Developer can create JS skill in <30 minutes
- [ ] Skill installs and runs correctly from local path
- [ ] Configuration persists between executions
- [ ] Sandbox blocks unauthorized filesystem access
- [ ] Documentation covers end-to-end workflow

### Long-Term Success
- [ ] 50+ skills in registry within 6 months
- [ ] <100ms cold start for average skill
- [ ] Skills work seamlessly in Claude Code via MCP
- [ ] Community contributions (PRs, skills, docs)
- [ ] Production usage by 100+ developers

## Phase 8: Anthropic SKILL.md Format & Enhanced Discovery

### NEW REQUIREMENT (2025-12-18): Skill Metadata & Documentation

**Goal**: Integrate Anthropic's SKILL.md format for richer skill metadata and enhanced semantic discovery

**Context**: Anthropic's Claude Skills use a SKILL.md format with YAML frontmatter that provides:
- Progressive disclosure (metadata always visible, instructions on-demand)
- Token-efficient discovery (few dozen tokens per skill)
- Rich documentation with examples and guidelines
- Tool restrictions (allowed-tools field)

**Reference**: https://github.com/anthropics/skills

#### SKILL.md Structure

Skills will support an optional `SKILL.md` file alongside `skill.yaml`:

```yaml
---
name: aws-skill
description: AWS service integration for S3, EC2, and Lambda. Use when you need to interact with AWS cloud resources.
allowed-tools: Read, Bash, skill-run
---

# AWS Skill

Provides comprehensive AWS cloud service integration through the AWS SDK.

## When to Use

- Managing S3 buckets and objects
- Listing and managing EC2 instances
- Invoking Lambda functions
- Working with AWS resources programmatically

## Tools Provided

### s3-upload
Upload files to S3 buckets with automatic multipart upload for large files.

**Usage**:
```bash
skill run aws-skill:s3-upload --bucket my-bucket --key path/to/file.txt --file ./local-file.txt
```

**Parameters**:
- `bucket` (required): S3 bucket name
- `key` (required): Object key/path in bucket
- `file` (required): Local file path to upload

**Example**:
```bash
# Upload a JSON file
skill run aws-skill@prod:s3-upload \
  --bucket analytics-data \
  --key uploads/2025/data.json \
  --file ./exports/data.json
```

### s3-list
List objects in an S3 bucket with optional prefix filtering.

**Usage**:
```bash
skill run aws-skill:s3-list --bucket my-bucket --prefix path/
```

**Parameters**:
- `bucket` (required): S3 bucket name
- `prefix` (optional): Filter by object key prefix

### ec2-list
List EC2 instances with optional filtering by state or tags.

**Usage**:
```bash
skill run aws-skill:ec2-list --region us-east-1 --state running
```

**Parameters**:
- `region` (optional): AWS region (defaults to config)
- `state` (optional): Filter by instance state (running, stopped, etc.)

## Configuration

This skill requires AWS credentials. Configure via:

```bash
skill config aws-skill --set aws_access_key_id=AKIAXXXXX
skill config aws-skill --set aws_secret_access_key=secretkey
skill config aws-skill --set region=us-east-1
```

Or use multiple instances for different AWS accounts:

```bash
skill install aws-skill --instance prod
skill config aws-skill --instance prod --set aws_access_key_id=PROD_KEY

skill install aws-skill --instance staging
skill config aws-skill --instance staging --set aws_access_key_id=STAGING_KEY
```

## Examples

### Upload build artifacts to S3
```bash
skill run aws-skill@prod:s3-upload \
  --bucket ci-artifacts \
  --key builds/v1.2.3/app.zip \
  --file ./dist/app.zip
```

### List running EC2 instances
```bash
skill run aws-skill@prod:ec2-list --state running
```

### Invoke Lambda function
```bash
skill run aws-skill@prod:lambda-invoke \
  --function process-data \
  --payload '{"key": "value"}'
```

## Security Notes

- Credentials are encrypted at rest using system keychain
- Each instance has isolated configuration
- AWS SDK uses TLS for all API calls
- Temporary credentials (STS) supported via IAM roles
```

#### Integration with Semantic Search

**Enhanced `skill find` command** will:

1. **Index SKILL.md content** alongside tool definitions
   - Embed: tool name, description, SKILL.md body, examples, usage patterns
   - Weight: Tool descriptions > examples > guidelines
   - Context window: Include 3-5 sentences around matches

2. **Return contextual results** with rich metadata:
   ```
   ✓ Top 5 matching tools:

   1. [95% match] aws-skill@prod → s3-upload
      Upload files to S3 buckets with automatic multipart upload

      📋 Context from SKILL.md:
      "Upload build artifacts to S3. Supports large files via multipart upload.
       Automatically detects content type from file extension."

      ⚙️  Usage:
      skill run aws-skill@prod:s3-upload --bucket my-bucket --key file.txt --file ./local.txt

      📝 Parameters:
      - bucket (required): S3 bucket name
      - key (required): Object key/path in bucket
      - file (required): Local file path to upload

      💡 Example:
      skill run aws-skill@prod:s3-upload \
        --bucket ci-artifacts \
        --key builds/v1.2.3/app.zip \
        --file ./dist/app.zip
   ```

3. **Contextual answer format**:
   - Similarity score (0-100%)
   - Tool identifier (skill@instance:tool)
   - Short description
   - Relevant SKILL.md excerpt (3-5 lines)
   - Full command with all flags
   - Parameter definitions
   - Concrete usage example
   - Configuration requirements (if any)

#### Implementation Tasks

**8.1. SKILL.md Parser**
- Read SKILL.md from skill package/directory
- Parse YAML frontmatter (name, description, allowed-tools)
- Extract tool documentation sections
- Build rich metadata structure

**8.2. Enhanced Embedding Index**
```rust
struct ToolDocument {
    // Basic metadata
    skill_name: String,
    instance_name: String,
    tool_name: String,
    description: String,

    // NEW: SKILL.md content
    skill_md_description: Option<String>,  // From SKILL.md frontmatter
    tool_documentation: Option<String>,    // Tool-specific section
    usage_examples: Vec<String>,           // Extracted examples
    parameters: Vec<ParameterDoc>,         // Parameter docs

    // Embedding text (combines all above)
    full_text: String,
}

struct ParameterDoc {
    name: String,
    required: bool,
    param_type: String,
    description: String,
}
```

**8.3. Rich Search Results Formatter**
- Template-based result display
- Syntax-highlighted code blocks
- Emoji indicators (📋 📝 ⚙️ 💡)
- Multi-line formatting for readability
- Truncation for long results

**8.4. Testing**
- Unit tests for SKILL.md parser
- Integration tests with sample skills
- Embedding quality tests (relevance scoring)
- Result formatting tests

#### Success Metrics

- [ ] SKILL.md files parsed correctly from skill packages
- [ ] Search results include SKILL.md context
- [ ] Parameter documentation visible in results
- [ ] Usage examples shown for top matches
- [ ] Search relevance improved (user testing)

## Phase 9: Multi-Provider Embedding Support ✅ COMPLETED

**Goal**: Eliminate vendor lock-in for semantic search with local embedding options

**Status**: ✅ Implemented (2025-12-18)

**Deliverables**:
1. ✅ FastEmbed integration (local, no API key required)
2. ✅ Ollama integration (local inference server)
3. ✅ OpenAI integration (existing)
4. ✅ CLI flags for provider selection (`--provider`, `--model`)
5. ✅ Default to FastEmbed (all-minilm model)

**Usage**:
```bash
# FastEmbed (default - no API key)
skill find "s3 upload" --provider fastembed --model all-minilm

# Ollama (requires local server)
skill find "s3 upload" --provider ollama --model nomic-embed-text

# OpenAI (requires API key)
skill find "s3 upload" --provider openai
```

**Available Models**:
- FastEmbed: all-minilm (default), bge-small, bge-base, bge-large
- Ollama: nomic-embed-text, mxbai-embed-large, all-minilm
- OpenAI: text-embedding-ada-002

**Benefits**:
- ✅ Zero API costs with local models
- ✅ Complete data privacy
- ✅ No internet required (FastEmbed)
- ✅ Fast inference (<100ms)
- ✅ Flexible provider switching

**Implementation**: `crates/skill-cli/src/commands/find.rs`

## Next Steps (PRD → Tasks)

1. ✅ **Parse this PRD with Task Master** to generate implementation tasks
2. Continue Phase 1-7 implementation (Core Runtime → Multi-Language SDK)
3. **NEW**: Implement Phase 8 (SKILL.md support + Enhanced Discovery)
   - Parse SKILL.md files from skill packages
   - Enhance ToolDocument structure with rich metadata
   - Update embedding generation to include SKILL.md content
   - Implement rich result formatter for `skill find`
4. ✅ **Phase 9 COMPLETED**: Multi-provider embeddings (FastEmbed/Ollama/OpenAI)
5. Iterate on WIT interface based on real usage

---

**Document Status**: Updated with New Requirements
**Last Updated**: 2025-12-18 (Added Phase 8 & 9)
**Author**: Shaked (with Claude assistance)
