# Rust API Reference

Complete API documentation for Skill Engine's Rust crates.

## Overview

Skill Engine is built with Rust and consists of 6 core crates:

| Crate | Purpose | Docs |
|-------|---------|------|
| `skill-runtime` | WASM runtime, skill execution, sandboxing | [docs.rs](https://docs.rs/skill-runtime) |
| `skill-mcp` | Model Context Protocol server implementation | [docs.rs](https://docs.rs/skill-mcp) |
| `skill-http` | HTTP server and REST API | [docs.rs](https://docs.rs/skill-http) |
| `skill-context` | RAG search, embeddings, secrets management | [docs.rs](https://docs.rs/skill-context) |
| `skill-cli` | Command-line interface | [docs.rs](https://docs.rs/skill-cli) |
| `skill-web` | Web UI (Yew/WASM) | [docs.rs](https://docs.rs/skill-web) |

## Installation

### As Library Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
skill-runtime = "0.3"
skill-mcp = "0.3"
skill-http = "0.3"
skill-context = "0.3"
```

### From Source

```bash
git clone https://github.com/kubiyabot/skill.git
cd skill
cargo build --release
```

## skill-runtime

Core runtime for executing skills with WASM sandboxing.

### Key Types

#### SkillExecutor

Main executor for running skills:

```rust
use skill_runtime::{SkillExecutor, ExecuteOptions};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create executor
    let executor = SkillExecutor::new("./skills")?;

    // Execute skill tool
    let params: HashMap<String, String> = [
        ("resource".to_string(), "pods".to_string()),
        ("namespace".to_string(), "default".to_string())
    ].into();

    let result = executor.execute(
        "kubernetes",
        "get",
        params,
        ExecuteOptions::default()
    ).await?;

    println!("Output: {}", result.stdout);
    Ok(())
}
```

#### Sandbox

WASI sandbox for secure execution:

```rust
use skill_runtime::sandbox::{SandboxBuilder, HostState};
use std::path::PathBuf;

// Create sandboxed environment
let host_state = SandboxBuilder::new("my-instance", PathBuf::from("./instance"))
    .env("API_KEY", "secret")
    .args(vec!["--verbose".to_string()])
    .inherit_stdio(true)
    .build()?;

// Host state can be used with Wasmtime engine
```

#### SkillManifest

Skill manifest parsing and validation:

```rust
use skill_runtime::manifest::SkillManifest;

// Load manifest
let manifest = SkillManifest::load("skill.toml")?;

println!("Skill: {}", manifest.name);
println!("Runtime: {:?}", manifest.runtime);
println!("Tools: {}", manifest.tools.len());

// Access tool definitions
for tool in &manifest.tools {
    println!("  - {}: {}", tool.name, tool.description);
}
```

#### InstanceConfig

Skill instance configuration (for multi-instance support):

```rust
use skill_runtime::instance::{InstanceConfig, InstanceManager};

// Create instance configuration
let config = InstanceConfig {
    name: "prod".to_string(),
    skill_name: "kubernetes".to_string(),
    environment: [
        ("KUBECONFIG".to_string(), "/path/to/prod.kubeconfig".to_string())
    ].into(),
    ..Default::default()
};

// Save instance
let manager = InstanceManager::new("./config")?;
manager.save_instance(&config)?;

// Load instance
let loaded = manager.load_instance("kubernetes", "prod")?;
```

### Features

Enable optional features in `Cargo.toml`:

```toml
[dependencies.skill-runtime]
version = "0.3"
features = [
    "qdrant",              # Qdrant vector database
    "hybrid-search",       # Tantivy full-text search
    "reranker",            # Embedding reranking
    "context-compression", # Token counting
    "ai-ingestion",        # LLM-based ingestion
    "ollama",              # Ollama provider
    "openai",              # OpenAI provider
    "sqlite-storage",      # SQLite job queue
    "postgres-storage",    # PostgreSQL job queue
    "redis-storage"        # Redis job queue
]
```

## skill-mcp

MCP (Model Context Protocol) server implementation.

### Key Types

#### McpServer

MCP JSON-RPC 2.0 server:

```rust
use skill_mcp::McpServer;
use tokio::io::{stdin, stdout};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create MCP server
    let server = McpServer::new("./skills")?;

    // Run on stdio (for Claude Code, etc.)
    server.run_stdio(stdin(), stdout()).await?;

    Ok(())
}
```

#### MCP Tools

Exposed MCP tools:

```rust
use skill_mcp::tools::{ExecuteTool, ListSkillsTool, SearchSkillsTool};

// Execute tool
let execute = ExecuteTool {
    skill_name: "kubernetes".to_string(),
    tool_name: "get".to_string(),
    parameters: serde_json::json!({
        "resource": "pods",
        "namespace": "default"
    })
};

let result = execute.call(&server).await?;

// List skills
let list = ListSkillsTool {
    limit: Some(10),
    offset: Some(0)
};

let skills = list.call(&server).await?;

// Search skills
let search = SearchSkillsTool {
    query: "kubernetes pods".to_string(),
    top_k: Some(5)
};

let results = search.call(&server).await?;
```

### Protocol Types

```rust
use skill_mcp::protocol::{JsonRpcRequest, JsonRpcResponse, ToolCall};

// JSON-RPC request
let request = JsonRpcRequest {
    jsonrpc: "2.0".to_string(),
    id: serde_json::json!(1),
    method: "tools/call".to_string(),
    params: serde_json::json!({
        "name": "skill-engine/execute",
        "arguments": {
            "skill_name": "kubernetes",
            "tool_name": "get",
            "parameters": {
                "resource": "pods"
            }
        }
    })
};

// Process request
let response = server.handle_request(&request).await?;
```

## skill-http

HTTP server and REST API.

### Key Types

#### HttpServer

REST API server:

```rust
use skill_http::HttpServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create HTTP server
    let server = HttpServer::new("./skills", "0.0.0.0:3000")?;

    // Run server
    server.run().await?;

    Ok(())
}
```

#### API Handlers

```rust
use skill_http::handlers::{execute_handler, list_skills_handler, search_handler};
use axum::{Router, routing::{get, post}};

// Build router
let app = Router::new()
    .route("/api/execute", post(execute_handler))
    .route("/api/skills", get(list_skills_handler))
    .route("/api/search", post(search_handler));
```

#### Request/Response Types

```rust
use skill_http::types::{ExecuteRequest, ExecuteResponse, ErrorResponse};

// Execute request
let request = ExecuteRequest {
    skill_name: "kubernetes".to_string(),
    tool_name: "get".to_string(),
    parameters: serde_json::json!({
        "resource": "pods",
        "namespace": "default"
    })
};

// Execute response
let response = ExecuteResponse {
    execution_id: "exec_123".to_string(),
    status: "success".to_string(),
    output: "...".to_string(),
    duration_ms: 245
};

// Error response
let error = ErrorResponse {
    error: ErrorDetails {
        code: "TOOL_EXECUTION_FAILED".to_string(),
        message: "kubectl not found".to_string(),
        details: serde_json::json!({
            "skill": "kubernetes",
            "tool": "get"
        })
    }
};
```

### OpenAPI

Generate OpenAPI specification:

```rust
use skill_http::openapi::generate_openapi_spec;

// Generate spec
let spec = generate_openapi_spec()?;

// Write to file
std::fs::write("openapi.json", serde_json::to_string_pretty(&spec)?)?;
```

## skill-context

Context management, RAG search, and secrets.

### Key Types

#### EmbeddingGenerator

Generate embeddings for semantic search:

```rust
use skill_context::embeddings::EmbeddingGenerator;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create generator
    let generator = EmbeddingGenerator::new().await?;

    // Generate embedding
    let text = "List all Kubernetes pods";
    let embedding = generator.generate(text).await?;

    println!("Embedding dimensions: {}", embedding.len());

    Ok(())
}
```

#### VectorStore

Store and search embeddings:

```rust
use skill_context::vectorstore::{VectorStore, Document};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create vector store
    let store = VectorStore::new("./vectorstore").await?;

    // Index document
    let doc = Document {
        id: "kubernetes-get".to_string(),
        text: "Get Kubernetes resources".to_string(),
        metadata: serde_json::json!({
            "skill": "kubernetes",
            "tool": "get"
        })
    };

    store.index(&doc).await?;

    // Search
    let results = store.search("list pods", 5).await?;

    for result in results {
        println!("Score: {:.3} - {}", result.score, result.document.text);
    }

    Ok(())
}
```

#### SecretsManager

Encrypted credential storage:

```rust
use skill_context::secrets::SecretsManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create secrets manager
    let secrets = SecretsManager::new()?;

    // Store credential (encrypted)
    secrets.set("kubernetes", "kubeconfig", "...base64...").await?;

    // Retrieve credential (decrypted)
    let kubeconfig = secrets.get("kubernetes", "kubeconfig").await?;

    // Delete credential
    secrets.delete("kubernetes", "kubeconfig").await?;

    Ok(())
}
```

#### RuntimeContext

Runtime context for skill execution:

```rust
use skill_context::runtime::RuntimeContext;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create context
    let context = RuntimeContext::new()
        .with_skill("kubernetes")
        .with_instance("prod")
        .with_user("alice")
        .build()?;

    // Access context
    println!("Skill: {}", context.skill_name);
    println!("Instance: {}", context.instance_id);
    println!("User: {}", context.user);

    Ok(())
}
```

## skill-cli

Command-line interface (not typically used as library, but API available).

### Command Execution

```rust
use skill_cli::commands::{run_command, CommandType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Execute command programmatically
    let result = run_command(CommandType::Run {
        skill: "kubernetes".to_string(),
        tool: "get".to_string(),
        args: vec![
            "--resource".to_string(),
            "pods".to_string()
        ]
    }).await?;

    println!("{}", result);

    Ok(())
}
```

## skill-web

Web UI built with Yew (WASM frontend).

### Component API

```rust
use yew::prelude::*;
use skill_web::components::SkillList;

#[function_component(App)]
fn app() -> Html {
    html! {
        <div>
            <h1>{"Skill Engine"}</h1>
            <SkillList />
        </div>
    }
}
```

## Common Patterns

### Error Handling

All crates use `anyhow::Result` for error handling:

```rust
use anyhow::{Context, Result};

fn my_function() -> Result<String> {
    let data = std::fs::read_to_string("file.txt")
        .context("Failed to read configuration file")?;

    Ok(data)
}
```

### Async Execution

All I/O operations are async with Tokio:

```rust
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Async operations
    let result = execute_skill().await?;

    Ok(())
}
```

### Logging

Structured logging with `tracing`:

```rust
use tracing::{info, debug, warn, error};

#[tracing::instrument]
async fn execute_tool(skill: &str, tool: &str) -> Result<String> {
    debug!(skill, tool, "Starting execution");

    let result = do_execution(skill, tool).await?;

    info!(
        skill,
        tool,
        duration_ms = result.duration,
        "Execution completed"
    );

    Ok(result.output)
}
```

### Configuration

Configuration via structs with `serde`:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    skills_dir: PathBuf,
    max_concurrent: usize,
    timeout_ms: u64
}

// Load from TOML
let config: Config = toml::from_str(&contents)?;

// Save to TOML
let toml_str = toml::to_string_pretty(&config)?;
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parsing() {
        let toml = r#"
            name = "test"
            version = "0.1.0"
            runtime = "wasm"
        "#;

        let manifest: SkillManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.name, "test");
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_skill_execution() {
    let executor = SkillExecutor::new("./test-skills").unwrap();

    let result = executor.execute(
        "test-skill",
        "echo",
        [("message".to_string(), "hello".to_string())].into(),
        ExecuteOptions::default()
    ).await.unwrap();

    assert_eq!(result.stdout.trim(), "hello");
}
```

### Mock Dependencies

Using `mockall` for mocking:

```rust
use mockall::*;

#[automock]
trait SkillExecutor {
    async fn execute(&self, skill: &str, tool: &str) -> Result<String>;
}

#[tokio::test]
async fn test_with_mock() {
    let mut mock = MockSkillExecutor::new();

    mock.expect_execute()
        .with(eq("kubernetes"), eq("get"))
        .returning(|_, _| Ok("mocked output".to_string()));

    let result = mock.execute("kubernetes", "get").await.unwrap();
    assert_eq!(result, "mocked output");
}
```

## Examples

### Complete Skill Execution Example

```rust
use skill_runtime::{SkillExecutor, ExecuteOptions};
use skill_context::secrets::SecretsManager;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load secrets
    let secrets = SecretsManager::new()?;
    let kubeconfig = secrets.get("kubernetes", "kubeconfig").await?;

    // Create executor
    let executor = SkillExecutor::new("./skills")?;

    // Execute skill
    let mut params = HashMap::new();
    params.insert("resource".to_string(), "pods".to_string());
    params.insert("namespace".to_string(), "production".to_string());

    let result = executor.execute(
        "kubernetes",
        "get",
        params,
        ExecuteOptions {
            timeout_ms: Some(30000),
            instance: Some("prod".to_string()),
            ..Default::default()
        }
    ).await?;

    // Process result
    if result.exit_code == 0 {
        println!("Success: {}", result.stdout);
    } else {
        eprintln!("Failed: {}", result.stderr);
    }

    Ok(())
}
```

### MCP Server with Custom Tools

```rust
use skill_mcp::McpServer;
use tokio::io::{stdin, stdout};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create MCP server
    let mut server = McpServer::new("./skills")?;

    // Register custom tool
    server.register_tool("custom/my-tool", |params| {
        async move {
            // Custom logic here
            Ok(serde_json::json!({
                "result": "custom output"
            }))
        }
    });

    // Run server
    server.run_stdio(stdin(), stdout()).await?;

    Ok(())
}
```

### HTTP Server with Middleware

```rust
use skill_http::HttpServer;
use axum::{Router, middleware};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create server
    let server = HttpServer::builder()
        .skills_dir("./skills")
        .bind("0.0.0.0:3000")
        .middleware(TraceLayer::new_for_http())
        .build()?;

    // Run with graceful shutdown
    server.run_with_shutdown(shutdown_signal()).await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl-c");
}
```

## Documentation

### Generate Docs

```bash
# Generate all crate docs
cargo doc --no-deps --open

# Generate specific crate
cargo doc -p skill-runtime --open

# Include private items
cargo doc --document-private-items
```

### Inline Documentation

Follow Rust documentation standards:

```rust
/// Executes a skill tool with the provided parameters.
///
/// # Arguments
///
/// * `skill_name` - The name of the skill to execute
/// * `tool_name` - The tool within the skill to run
/// * `parameters` - Key-value parameters for the tool
/// * `options` - Execution options (timeout, instance, etc.)
///
/// # Returns
///
/// Returns `ExecutionResult` containing stdout, stderr, and exit code.
///
/// # Errors
///
/// Returns `Err` if:
/// - Skill is not found
/// - Tool does not exist
/// - Parameters are invalid
/// - Execution times out
/// - Runtime error occurs
///
/// # Examples
///
/// ```rust
/// use skill_runtime::{SkillExecutor, ExecuteOptions};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let executor = SkillExecutor::new("./skills")?;
///
/// let result = executor.execute(
///     "kubernetes",
///     "get",
///     [("resource".into(), "pods".into())].into(),
///     ExecuteOptions::default()
/// ).await?;
///
/// println!("Output: {}", result.stdout);
/// # Ok(())
/// # }
/// ```
pub async fn execute(
    &self,
    skill_name: &str,
    tool_name: &str,
    parameters: HashMap<String, String>,
    options: ExecuteOptions
) -> Result<ExecutionResult> {
    // Implementation
}
```

## Contributing

See [Contributing Guide](../../contributing.md) for development setup and guidelines.

### Building from Source

```bash
# Clone repository
git clone https://github.com/kubiyabot/skill.git
cd skill

# Build all crates
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy

# Format code
cargo fmt
```

## Related Documentation

- [CLI Reference](./cli.md) - Command-line interface
- [REST API](./rest.md) - HTTP API reference
- [MCP Protocol](./mcp.md) - MCP protocol specification
- [Security Model](../guides/advanced/security.md) - Security architecture

## External Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial) - Async programming
- [WASM Component Model](https://component-model.bytecodealliance.org/) - WASM components
- [Wasmtime](https://docs.wasmtime.dev/) - WASM runtime
