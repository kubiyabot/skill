//! MCP Integration Tests for Claude Bridge Context Engineering
//!
//! These tests validate the MCP (Model Context Protocol) server implementation,
//! focusing on context engineering features that help AI agents manage output
//! size and extract relevant information from skill executions.
//!
//! # Overview
//!
//! The MCP server exposes skills as tools that can be executed by AI agents like
//! Claude. These tests validate the complete JSON-RPC 2.0 protocol implementation
//! including initialization, tool discovery, tool execution, and context
//! engineering features.
//!
//! # Test Framework
//!
//! - `McpTestServer`: Manages MCP server process lifecycle
//!   - Spawns `skill serve` command as subprocess
//!   - Handles JSON-RPC communication via stdin/stdout
//!   - Automatic cleanup via `kill_on_drop`
//!   - Timeout protection for hanging operations
//! - JSON-RPC 2.0 protocol compliance validation
//! - Async test execution with tokio runtime
//!
//! # Running Tests
//!
//! Tests are marked `#[ignore]` because they require:
//! - A built `skill` binary
//! - A running Kubernetes cluster (for kubernetes skill tests)
//!
//! ```bash
//! # Build the skill binary first
//! cargo build --bin skill
//!
//! # Run all MCP tests
//! cargo test --test mcp_claude_bridge_tests -- --ignored
//!
//! # Run a specific test
//! cargo test test_mcp_tool_execution -- --ignored --nocapture
//!
//! # Compile tests without running
//! cargo test --test mcp_claude_bridge_tests --no-run
//! ```
//!
//! # MCP Protocol Reference
//!
//! - **Specification**: MCP 2024-11-05
//! - **Transport**: stdio (JSON-RPC over stdin/stdout)
//! - **Format**: JSON-RPC 2.0
//! - **Methods**:
//!   - `initialize`: Protocol handshake
//!   - `tools/list`: List available tools
//!   - `tools/call`: Execute a tool
//!
//! ## Protocol Flow
//!
//! 1. Client sends `initialize` request with protocol version
//! 2. Server responds with capabilities and server info
//! 3. Client sends `notifications/initialized` notification
//! 4. Client can now call `tools/list` and `tools/call`
//!
//! # Context Engineering Features Tested
//!
//! Context engineering helps AI agents manage token limits and extract relevant
//! information from potentially large tool outputs:
//!
//! - **`grep`**: Filter output lines by regex pattern (like grep command)
//! - **`head`**: Limit output to first N lines (like head -n N)
//! - **`tail`**: Limit output to last N lines (like tail -n N)
//! - **`jq`**: Extract data from JSON output using jq expressions
//! - **`max_output`**: Truncate output to maximum characters with smart truncation
//!
//! # Test Coverage
//!
//! ## Basic Protocol Tests
//! - `test_mcp_tool_execution`: Validates initialization and basic tool execution
//!
//! ## Context Engineering Tests
//! - `test_mcp_context_engineering_grep`: Validates grep filtering
//! - `test_mcp_context_engineering_head`: Validates head line limiting
//! - `test_mcp_context_engineering_jq`: Validates jq JSON extraction
//! - `test_mcp_context_engineering_max_output`: Validates output truncation
//!
//! ## Error Handling Tests
//! - `test_mcp_error_invalid_skill`: Non-existent skill error
//! - `test_mcp_error_invalid_tool`: Non-existent tool error
//! - `test_mcp_error_missing_params`: Missing required parameter error
//!
//! # Implementation Details
//!
//! ## McpTestServer
//!
//! The test server manages a child process running `skill serve`:
//!
//! - **Process Management**: Uses `tokio::process::Command` with piped stdio
//! - **Communication**: Async stdin writes and stdout reads via `tokio::io`
//! - **Timeout**: All operations wrapped in `tokio::time::timeout` (10s default)
//! - **Cleanup**: Process killed automatically via `kill_on_drop = true`
//!
//! ## Request/Response Pattern
//!
//! ```rust,ignore
//! // 1. Initialize server
//! let mut server = McpTestServer::new().await?;
//! server.initialize().await?;
//!
//! // 2. Execute tool with context engineering
//! let response = server.execute_tool(
//!     "kubernetes",
//!     "get",
//!     json!({"resource": "pods"}),
//!     Some(json!({"grep": "Running", "head": 10}))
//! ).await?;
//!
//! // 3. Validate response
//! assert_eq!(response["jsonrpc"], "2.0");
//! assert!(response["result"]["content"][0]["text"].as_str().unwrap().contains("Running"));
//! ```
//!
//! # Troubleshooting
//!
//! ## Tests Hanging
//! - Check if kubernetes cluster is running: `kubectl cluster-info`
//! - Verify skill binary exists: `which skill` or `cargo build --bin skill`
//! - Increase timeout if cluster is slow
//!
//! ## Parse Errors
//! - Server stderr is suppressed (`.stderr(Stdio::null())`)
//! - Run `skill serve` manually to see error messages
//! - Check JSON-RPC format is correct
//!
//! ## Process Leaks
//! - Ensure tests don't panic before cleanup
//! - Use `kill_on_drop = true` to ensure cleanup
//! - Check for zombie processes: `ps aux | grep skill`

use serde_json::{json, Value};
use std::io::Write;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::timeout;

/// MCP Test Server - manages MCP server process and JSON-RPC communication
///
/// This struct spawns the `skill serve` command and provides methods for
/// sending JSON-RPC requests and receiving responses over stdin/stdout.
///
/// # Example
///
/// ```no_run
/// let mut server = McpTestServer::new().await.unwrap();
/// server.initialize().await.unwrap();
/// let response = server.execute_tool("kubernetes", "get", json!({"resource": "pods"})).await.unwrap();
/// ```
pub struct McpTestServer {
    process: Child,
    next_id: i64,
}

impl McpTestServer {
    /// Spawn a new MCP server process
    ///
    /// Starts `skill serve` with stdin/stdout piped for JSON-RPC communication.
    /// The server process will be automatically cleaned up when this struct is dropped.
    pub async fn new() -> anyhow::Result<Self> {
        let process = Command::new(env!("CARGO_BIN_EXE_skill"))
            .arg("serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()) // Suppress stderr noise in tests
            .kill_on_drop(true)
            .spawn()?;

        Ok(Self {
            process,
            next_id: 1,
        })
    }

    /// Send a JSON-RPC request and read the response
    ///
    /// # Arguments
    ///
    /// * `request` - JSON-RPC request object
    ///
    /// # Returns
    ///
    /// The JSON-RPC response as a `Value`
    async fn send_request(&mut self, request: Value) -> anyhow::Result<Value> {
        // Get stdin and stdout handles
        let stdin = self
            .process
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdin"))?;
        let stdout = self
            .process
            .stdout
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdout"))?;

        // Send request
        let request_str = serde_json::to_string(&request)? + "\n";
        stdin.write_all(request_str.as_bytes()).await?;
        stdin.flush().await?;

        // Read response with timeout
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        timeout(Duration::from_secs(10), reader.read_line(&mut line)).await??;

        // Parse and return response
        let response: Value = serde_json::from_str(&line)?;
        Ok(response)
    }

    /// Send MCP initialize request
    ///
    /// Must be called before any other MCP operations.
    /// Performs the MCP handshake with protocol version 2024-11-05.
    pub async fn initialize(&mut self) -> anyhow::Result<Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "id": self.next_id,
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        });
        self.next_id += 1;

        let response = self.send_request(request).await?;

        // Send initialized notification
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });

        let notification_str = serde_json::to_string(&notification)? + "\n";
        let stdin = self
            .process
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdin"))?;
        stdin.write_all(notification_str.as_bytes()).await?;
        stdin.flush().await?;

        // Give server time to process notification
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(response)
    }

    /// List available MCP tools
    pub async fn list_tools(&mut self) -> anyhow::Result<Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "id": self.next_id,
            "params": {}
        });
        self.next_id += 1;

        self.send_request(request).await
    }

    /// Execute a skill tool via MCP
    ///
    /// # Arguments
    ///
    /// * `skill` - Skill name (e.g., "kubernetes")
    /// * `tool` - Tool name (e.g., "get")
    /// * `args` - Tool arguments as JSON object
    /// * `context_opts` - Optional context engineering options (grep, head, jq, etc.)
    pub async fn execute_tool(
        &mut self,
        skill: &str,
        tool: &str,
        args: Value,
        context_opts: Option<Value>,
    ) -> anyhow::Result<Value> {
        let mut arguments = json!({
            "skill": skill,
            "tool": tool,
            "args": args
        });

        // Merge context engineering options if provided
        if let Some(opts) = context_opts {
            if let (Some(args_obj), Some(opts_obj)) = (arguments.as_object_mut(), opts.as_object())
            {
                for (key, value) in opts_obj {
                    args_obj.insert(key.clone(), value.clone());
                }
            }
        }

        let request = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "id": self.next_id,
            "params": {
                "name": "execute",
                "arguments": arguments
            }
        });
        self.next_id += 1;

        self.send_request(request).await
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
#[ignore] // Requires skill binary to be built and kubernetes cluster available
async fn test_mcp_tool_execution() {
    let mut server = McpTestServer::new().await.unwrap();

    // Initialize MCP protocol
    let init_response = server.initialize().await.unwrap();
    assert_eq!(init_response["jsonrpc"], "2.0");
    assert!(init_response["result"]["protocolVersion"]
        .as_str()
        .unwrap()
        .starts_with("2024"));

    // Execute kubernetes:get tool
    let response = server
        .execute_tool(
            "kubernetes",
            "get",
            json!({"resource": "namespaces"}),
            None,
        )
        .await
        .unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());
    assert!(response.get("error").is_none());
}

#[tokio::test]
#[ignore] // Requires skill binary to be built and kubernetes cluster available
async fn test_mcp_context_engineering_grep() {
    let mut server = McpTestServer::new().await.unwrap();
    server.initialize().await.unwrap();

    // Execute with grep filter
    let response = server
        .execute_tool(
            "kubernetes",
            "get",
            json!({"resource": "namespaces"}),
            Some(json!({"grep": "default"})),
        )
        .await
        .unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());

    // Verify grep filtered the output
    let result_str = response["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert!(
        result_str.contains("default"),
        "Grep filter should include 'default'"
    );
}

#[tokio::test]
#[ignore] // Requires skill binary to be built and kubernetes cluster available
async fn test_mcp_context_engineering_head() {
    let mut server = McpTestServer::new().await.unwrap();
    server.initialize().await.unwrap();

    // Execute with head limit
    let response = server
        .execute_tool(
            "kubernetes",
            "get",
            json!({"resource": "namespaces"}),
            Some(json!({"head": 3})),
        )
        .await
        .unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());

    // Verify output is limited
    let result_str = response["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    let line_count = result_str.lines().count();
    assert!(
        line_count <= 3,
        "Head should limit output to 3 lines, got {}",
        line_count
    );
}

#[tokio::test]
#[ignore] // Requires skill binary to be built and kubernetes cluster available
async fn test_mcp_context_engineering_jq() {
    let mut server = McpTestServer::new().await.unwrap();
    server.initialize().await.unwrap();

    // Execute with JSON output and jq extraction
    let response = server
        .execute_tool(
            "kubernetes",
            "get",
            json!({"resource": "namespaces", "output": "json"}),
            Some(json!({"jq": ".items[].metadata.name"})),
        )
        .await
        .unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());

    // Verify jq extracted namespace names
    let result_str = response["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert!(
        result_str.contains("default") || result_str.contains("kube-"),
        "JQ should extract namespace names"
    );
}

#[tokio::test]
#[ignore] // Requires skill binary to be built and kubernetes cluster available
async fn test_mcp_context_engineering_max_output() {
    let mut server = McpTestServer::new().await.unwrap();
    server.initialize().await.unwrap();

    // Execute with max_output truncation
    let response = server
        .execute_tool(
            "kubernetes",
            "get",
            json!({"resource": "pods", "all-namespaces": "true"}),
            Some(json!({"max_output": 500})),
        )
        .await
        .unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());

    // Verify output is truncated
    let result_str = response["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert!(
        result_str.len() <= 600, // Some buffer for metadata
        "Max output should truncate to ~500 chars, got {}",
        result_str.len()
    );
}

#[tokio::test]
#[ignore] // Requires skill binary to be built
async fn test_mcp_error_invalid_skill() {
    let mut server = McpTestServer::new().await.unwrap();
    server.initialize().await.unwrap();

    // Execute with non-existent skill
    let response = server
        .execute_tool("nonexistent_skill_xyz", "get", json!({}), None)
        .await
        .unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("error").is_some());
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("skill"));
}

#[tokio::test]
#[ignore] // Requires skill binary to be built and kubernetes cluster available
async fn test_mcp_error_invalid_tool() {
    let mut server = McpTestServer::new().await.unwrap();
    server.initialize().await.unwrap();

    // Execute with non-existent tool
    let response = server
        .execute_tool("kubernetes", "nonexistent_tool_xyz", json!({}), None)
        .await
        .unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("error").is_some());
}

#[tokio::test]
#[ignore] // Requires skill binary to be built and kubernetes cluster available
async fn test_mcp_error_missing_params() {
    let mut server = McpTestServer::new().await.unwrap();
    server.initialize().await.unwrap();

    // Execute kubernetes:get without required 'resource' parameter
    let response = server
        .execute_tool("kubernetes", "get", json!({}), None)
        .await
        .unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(
        response.get("error").is_some() || response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("error"),
        "Should return error for missing required parameter"
    );
}
