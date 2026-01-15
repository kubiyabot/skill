# PRD: Persistent Server Architecture with Background Workers

## Overview

Enhance the Skill Engine MCP server to support long-running background job processing, persistent state management, remote server capabilities, and configurable authentication. The architecture must support both minimal local-only deployments and full-featured distributed deployments.

## Goals

1. **Efficient Background Processing**: Handle long-running AI tasks (example generation, vector indexing) without blocking the main server
2. **Persistent State**: Maintain job state across server restarts using configurable storage backends
3. **Remote MCP Server**: Support HTTP-based MCP transport for remote clients with authentication
4. **Minimal Dependencies Mode**: Work fully offline with SQLite and local models
5. **Enterprise Mode**: Scale with Redis, PostgreSQL, and cloud LLM providers

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Skill Engine Server                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐  │
│  │   MCP Transport  │    │   HTTP Server    │    │   Auth Layer     │  │
│  │   - stdio        │    │   (axum)         │    │   - None         │  │
│  │   - HTTP/SSE     │    │                  │    │   - JWT          │  │
│  │   - Streamable   │    │   /api/v1/...    │    │   - OIDC         │  │
│  └────────┬─────────┘    └────────┬─────────┘    └────────┬─────────┘  │
│           │                       │                        │            │
│           └───────────────────────┼────────────────────────┘            │
│                                   │                                      │
│  ┌────────────────────────────────▼─────────────────────────────────┐   │
│  │                       Job Coordinator                              │   │
│  │   - Accepts job submissions                                        │   │
│  │   - Routes to appropriate workers                                  │   │
│  │   - Manages job lifecycle                                          │   │
│  └────────────────────────────────┬─────────────────────────────────┘   │
│                                   │                                      │
│           ┌───────────────────────┼───────────────────────┐             │
│           │                       │                       │             │
│  ┌────────▼────────┐    ┌────────▼────────┐    ┌────────▼────────┐    │
│  │  AI Worker      │    │  Index Worker   │    │  Skill Worker   │    │
│  │                 │    │                 │    │                 │    │
│  │  - Example gen  │    │  - Vector embed │    │  - Tool exec    │    │
│  │  - RAG queries  │    │  - BM25 index   │    │  - WASM runtime │    │
│  │  - Self-ask     │    │  - Incremental  │    │  - Sandboxed    │    │
│  └────────┬────────┘    └────────┬────────┘    └────────┬────────┘    │
│           │                       │                       │             │
│           └───────────────────────┼───────────────────────┘             │
│                                   │                                      │
│  ┌────────────────────────────────▼─────────────────────────────────┐   │
│  │                     Storage Backend (apalis)                       │   │
│  │   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐      │   │
│  │   │  SQLite  │   │ Postgres │   │  Redis   │   │  Memory  │      │   │
│  │   │ (local)  │   │ (scale)  │   │ (fast)   │   │ (dev)    │      │   │
│  │   └──────────┘   └──────────┘   └──────────┘   └──────────┘      │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Phase 1: Background Job Processing (P0)

### 1.1 Apalis Integration

Add apalis for job queue management:

```toml
# Cargo.toml
[dependencies]
apalis = { version = "0.7", features = ["limit", "retry"] }
apalis-sql = { version = "0.7", features = ["sqlite"] }  # Default local
apalis-redis = { version = "0.7", optional = true }       # Optional scale

[features]
default = ["sqlite-storage"]
sqlite-storage = ["apalis-sql/sqlite"]
postgres-storage = ["apalis-sql/postgres"]
redis-storage = ["apalis-redis"]
```

### 1.2 Job Types

```rust
/// AI example generation job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateExamplesJob {
    pub skill_name: String,
    pub tool_names: Option<Vec<String>>,  // None = all tools
    pub examples_per_tool: usize,
    pub priority: JobPriority,
}

/// Vector indexing job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocumentsJob {
    pub skill_name: String,
    pub documents: Vec<IndexDocument>,
    pub force_reindex: bool,
}

/// Skill execution job (for long-running tools)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteSkillJob {
    pub skill_name: String,
    pub tool_name: String,
    pub input: serde_json::Value,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum JobPriority {
    Low,
    Normal,
    High,
}
```

### 1.3 Worker Configuration

```rust
/// Worker pool configuration
#[derive(Debug, Clone, Deserialize)]
pub struct WorkerConfig {
    /// Number of AI worker threads
    pub ai_workers: usize,  // Default: 2

    /// Number of indexing worker threads
    pub index_workers: usize,  // Default: 1

    /// Number of skill execution workers
    pub skill_workers: usize,  // Default: 4

    /// Maximum concurrent jobs per worker
    pub concurrency_per_worker: usize,  // Default: 2

    /// Job timeout
    pub job_timeout_secs: u64,  // Default: 300

    /// Retry configuration
    pub max_retries: usize,  // Default: 3
    pub retry_delay_secs: u64,  // Default: 5
}
```

### 1.4 Job State Management

```rust
/// Job status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    pub id: String,
    pub job_type: String,
    pub status: JobState,
    pub progress: Option<f32>,  // 0.0 - 1.0
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}
```

## Phase 2: Storage Backends (P0)

### 2.1 Configuration Schema

```toml
# ~/.skill-engine/server.toml

[server]
# Server mode: local, remote
mode = "local"

# Bind address
host = "127.0.0.1"
port = 3000

[storage]
# Backend: memory, sqlite, postgres, redis
backend = "sqlite"

# SQLite configuration (local mode default)
[storage.sqlite]
path = "~/.skill-engine/jobs.db"

# PostgreSQL configuration (optional)
[storage.postgres]
url = "postgres://user:pass@localhost/skill_engine"
pool_size = 10

# Redis configuration (optional)
[storage.redis]
url = "redis://localhost:6379"
prefix = "skill-engine"

[workers]
ai_workers = 2
index_workers = 1
skill_workers = 4
concurrency_per_worker = 2
job_timeout_secs = 300
```

### 2.2 Storage Abstraction

```rust
/// Storage backend trait
#[async_trait]
pub trait JobStorage: Send + Sync {
    /// Push a job to the queue
    async fn push<J: Job>(&self, job: J) -> Result<String>;

    /// Get job status
    async fn status(&self, job_id: &str) -> Result<Option<JobStatus>>;

    /// List jobs with optional filters
    async fn list(&self, filter: JobFilter) -> Result<Vec<JobStatus>>;

    /// Cancel a pending job
    async fn cancel(&self, job_id: &str) -> Result<bool>;

    /// Get storage health
    async fn health(&self) -> HealthStatus;
}

/// Create storage from configuration
pub fn create_storage(config: &StorageConfig) -> Result<Box<dyn JobStorage>> {
    match config.backend {
        StorageBackend::Memory => Ok(Box::new(MemoryStorage::new())),
        StorageBackend::Sqlite => {
            let storage = SqliteStorage::connect(&config.sqlite.path)?;
            storage.setup_tables()?;
            Ok(Box::new(storage))
        }
        #[cfg(feature = "postgres-storage")]
        StorageBackend::Postgres => {
            let pool = PgPoolOptions::new()
                .max_connections(config.postgres.pool_size)
                .connect(&config.postgres.url)?;
            Ok(Box::new(PostgresStorage::new(pool)))
        }
        #[cfg(feature = "redis-storage")]
        StorageBackend::Redis => {
            let client = redis::Client::open(&config.redis.url)?;
            Ok(Box::new(RedisStorage::new(client, &config.redis.prefix)))
        }
    }
}
```

## Phase 3: Remote MCP Server (P1)

### 3.1 HTTP Transport

Support the new Streamable HTTP transport (MCP 2025-03-26):

```rust
/// MCP HTTP server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct McpHttpConfig {
    /// Enable HTTP transport
    pub enabled: bool,

    /// Endpoint path
    pub endpoint: String,  // Default: "/mcp"

    /// Enable legacy SSE transport (backwards compat)
    pub legacy_sse: bool,  // Default: true

    /// Maximum request body size
    pub max_body_size: usize,  // Default: 10MB

    /// Request timeout
    pub timeout_secs: u64,  // Default: 300
}
```

### 3.2 HTTP Routes

```rust
/// MCP HTTP routes
pub fn mcp_routes(state: AppState) -> Router {
    Router::new()
        // Streamable HTTP transport (2025-03-26)
        .route("/mcp", post(handle_mcp_post))
        .route("/mcp", get(handle_mcp_get))

        // Legacy SSE transport (2024-11-05)
        .route("/mcp/sse", get(handle_sse_stream))
        .route("/mcp/message", post(handle_sse_message))

        // Job management API
        .route("/api/v1/jobs", post(submit_job))
        .route("/api/v1/jobs", get(list_jobs))
        .route("/api/v1/jobs/:id", get(get_job_status))
        .route("/api/v1/jobs/:id", delete(cancel_job))
        .route("/api/v1/jobs/:id/stream", get(stream_job_events))

        .with_state(state)
}

/// Handle MCP POST (Streamable HTTP)
async fn handle_mcp_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Parse JSON-RPC request
    let request: JsonRpcRequest = serde_json::from_slice(&body)?;

    // Check Accept header for streaming capability
    let accepts_sse = headers.get(ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/event-stream"))
        .unwrap_or(false);

    // Handle request
    let response = state.mcp_handler.handle(request).await?;

    // Return JSON or SSE stream based on response type
    if accepts_sse && response.requires_streaming() {
        stream_response(response)
    } else {
        json_response(response)
    }
}
```

## Phase 4: Authentication (P1)

### 4.1 Auth Configuration

```toml
# ~/.skill-engine/server.toml

[auth]
# Auth mode: none, jwt, oidc
mode = "none"

# JWT configuration (simple API key/token auth)
[auth.jwt]
# Secret for signing (generate with: openssl rand -base64 32)
secret = "${JWT_SECRET}"

# Token validity
expiry_hours = 24

# Issuer
issuer = "skill-engine"

# Audience
audience = "skill-engine-api"

# OIDC configuration (enterprise SSO)
[auth.oidc]
# OIDC provider URL (supports auto-discovery)
provider_url = "https://accounts.google.com"

# Client credentials
client_id = "${OIDC_CLIENT_ID}"
client_secret = "${OIDC_CLIENT_SECRET}"

# Scopes to request
scopes = ["openid", "profile", "email"]

# Claims to extract
user_claim = "email"
roles_claim = "groups"
```

### 4.2 Auth Middleware

```rust
/// Authentication layer
pub fn auth_layer(config: &AuthConfig) -> AuthLayer {
    match config.mode {
        AuthMode::None => AuthLayer::none(),
        AuthMode::Jwt => {
            let validator = JwtValidator::new(&config.jwt)?;
            AuthLayer::jwt(validator)
        }
        AuthMode::Oidc => {
            let provider = OidcProvider::discover(&config.oidc.provider_url)?;
            AuthLayer::oidc(provider, &config.oidc)
        }
    }
}

/// JWT authentication using tower middleware
pub struct JwtAuthLayer {
    validator: JwtValidator,
}

impl<S> Layer<S> for JwtAuthLayer {
    type Service = JwtAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        JwtAuthMiddleware {
            inner,
            validator: self.validator.clone(),
        }
    }
}

/// Extract authenticated user
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub sub: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
}

impl<S> FromRequestParts<S> for AuthenticatedUser {
    // Extract from request extensions (set by middleware)
}
```

### 4.3 API Key Management

```rust
/// Simple API key authentication for local use
#[derive(Debug, Clone, Deserialize)]
pub struct ApiKeyConfig {
    /// API keys (name -> key hash)
    pub keys: HashMap<String, String>,

    /// Header name
    pub header: String,  // Default: "X-API-Key"
}

/// API key commands
// skill auth api-key create <name>
// skill auth api-key revoke <name>
// skill auth api-key list
```

## Phase 5: CLI Integration (P1)

### 5.1 Server Commands

```bash
# Start server (local mode by default)
skill serve
skill serve --mode=local              # SQLite + local workers
skill serve --mode=remote             # HTTP transport + auth
skill serve --config=/path/to/config  # Custom config

# Worker management
skill workers status                  # Show worker status
skill workers scale ai=4 index=2      # Scale workers dynamically

# Job management
skill jobs list                       # List all jobs
skill jobs list --status=running      # Filter by status
skill jobs show <job_id>              # Show job details
skill jobs cancel <job_id>            # Cancel job
skill jobs retry <job_id>             # Retry failed job

# Submit background jobs
skill enhance kubernetes --background  # Submit as background job
skill index --all --background         # Background indexing
```

### 5.2 Background Job Submission

```rust
/// Submit job to background queue
pub async fn submit_background_job<J: Job>(
    storage: &dyn JobStorage,
    job: J,
    priority: JobPriority,
) -> Result<String> {
    let job_id = storage.push(job, priority).await?;
    println!("Job submitted: {}", job_id);
    println!("Track progress: skill jobs show {}", job_id);
    Ok(job_id)
}
```

## Implementation Plan

### Milestone 1: Local Background Jobs (1 week)

1. Add apalis with SQLite backend
2. Implement job types for AI generation and indexing
3. Add worker pool with configurable concurrency
4. Integrate with existing `skill enhance` command
5. Add `skill jobs` CLI commands

### Milestone 2: Remote MCP Server (1 week)

1. Implement Streamable HTTP transport
2. Add legacy SSE support for backwards compatibility
3. Create HTTP routes for job management API
4. Add streaming job progress via SSE

### Milestone 3: Authentication (3-4 days)

1. Add JWT authentication middleware
2. Implement OIDC discovery and validation
3. Add API key management for local use
4. Create `skill auth` CLI commands

### Milestone 4: Storage Backends (3-4 days)

1. Abstract storage with trait
2. Add PostgreSQL backend (optional feature)
3. Add Redis backend (optional feature)
4. Add configuration validation

### Milestone 5: Production Hardening (1 week)

1. Add metrics/observability (OpenTelemetry)
2. Graceful shutdown handling
3. Health checks and readiness probes
4. Documentation and examples

## Configuration Examples

### Minimal Local Mode (Default)

```toml
# No configuration needed - defaults to:
# - SQLite storage at ~/.skill-engine/jobs.db
# - Local workers (2 AI, 1 index, 4 skill)
# - stdio MCP transport
# - No authentication
```

### Development with HTTP

```toml
[server]
mode = "remote"
host = "127.0.0.1"
port = 3000

[auth]
mode = "none"  # No auth for local dev

[storage]
backend = "sqlite"
```

### Production with Auth

```toml
[server]
mode = "remote"
host = "0.0.0.0"
port = 3000

[auth]
mode = "jwt"

[auth.jwt]
secret = "${JWT_SECRET}"
expiry_hours = 8

[storage]
backend = "postgres"

[storage.postgres]
url = "${DATABASE_URL}"
pool_size = 20

[workers]
ai_workers = 4
index_workers = 2
skill_workers = 8
concurrency_per_worker = 4
```

### Enterprise with OIDC

```toml
[server]
mode = "remote"

[auth]
mode = "oidc"

[auth.oidc]
provider_url = "https://login.company.com"
client_id = "${OIDC_CLIENT_ID}"
client_secret = "${OIDC_CLIENT_SECRET}"

[storage]
backend = "redis"

[storage.redis]
url = "${REDIS_URL}"
```

## Success Metrics

1. **Efficiency**: Background jobs don't block MCP responses
2. **Reliability**: Jobs survive server restarts with SQLite
3. **Scalability**: Can scale to 100+ concurrent jobs with Redis
4. **Latency**: Job submission < 10ms, status check < 5ms
5. **Security**: JWT/OIDC validation < 20ms overhead

## References

- [apalis - GitHub](https://github.com/geofmureithi/apalis)
- [apalis-sql](https://lib.rs/crates/apalis-sql) - SQLite/Postgres support
- [apalis-redis](https://lib.rs/crates/apalis-redis) - Redis support
- [MCP Transports Specification (2025-03-26)](https://modelcontextprotocol.io/specification/2025-03-26/basic/transports)
- [MCP Legacy SSE Transport (2024-11-05)](https://spec.modelcontextprotocol.io/specification/2024-11-05/basic/transports/)
- [jwt-authorizer](https://lib.rs/crates/jwt-authorizer) - JWT/OIDC for axum
- [axum-oidc](https://crates.io/crates/axum-oidc) - OpenID Connect integration
- [tower-oauth2-resource-server](https://crates.io/crates/tower-oauth2-resource-server) - OAuth2 middleware
