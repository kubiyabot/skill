//! Claude Code Simulator - Test Framework for Acceptance Testing
//!
//! This module provides a test harness that simulates Claude Code's interaction
//! with the Skill Engine MCP server, enabling automated end-to-end acceptance
//! testing of real-world usage scenarios.
//!
//! # Overview
//!
//! The `ClaudeCodeSimulator` acts as a mock Claude Code client, performing the
//! same MCP protocol operations that Claude Code would:
//!
//! 1. **Skill Discovery**: Search for relevant skills using semantic search
//! 2. **Tool Execution**: Execute skill tools with arguments
//! 3. **Context Engineering**: Apply filters (grep, jq, head, tail, max_output)
//!
//! # Usage
//!
//! ```rust,no_run
//! use framework::ClaudeCodeSimulator;
//! use serde_json::json;
//!
//! #[tokio::test]
//! async fn test_scenario() {
//!     let mut sim = ClaudeCodeSimulator::new().await.unwrap();
//!
//!     // Discover relevant skills
//!     let skills = sim.discover_skill("kubernetes pods").await.unwrap();
//!     assert!(skills.contains(&"kubernetes".to_string()));
//!
//!     // Execute tool
//!     let result = sim.execute_tool(
//!         "kubernetes",
//!         "get",
//!         json!({"resource": "pods"}),
//!         None
//!     ).await.unwrap();
//!
//!     assert!(!result.is_empty());
//! }
//! ```

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::timeout;

/// Claude Code Simulator - simulates Claude Code's MCP client behavior
///
/// This struct manages an MCP server process and provides high-level methods
/// that mirror the operations Claude Code performs when interacting with skills.
///
/// # Lifecycle
///
/// 1. `new()` - Spawns MCP server and performs initialization handshake
/// 2. Various test methods (discover_skill, execute_tool, etc.)
/// 3. Automatic cleanup on drop
pub struct ClaudeCodeSimulator {
    /// MCP server process handle
    process: Child,
    /// Next JSON-RPC request ID
    next_id: i64,
}

impl ClaudeCodeSimulator {
    /// Create a new Claude Code simulator
    ///
    /// Spawns the MCP server process (`skill serve`) and performs the
    /// initialization handshake required by the MCP protocol.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - skill binary cannot be found or executed
    /// - MCP initialization fails
    /// - Server doesn't respond within timeout
    pub async fn new() -> Result<Self> {
        let process = Command::new(env!("CARGO_BIN_EXE_skill"))
            .arg("serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to spawn MCP server")?;

        let mut simulator = Self {
            process,
            next_id: 1,
        };

        // Perform MCP initialization handshake
        simulator.initialize().await?;

        Ok(simulator)
    }

    /// Send MCP initialize request and notifications/initialized
    async fn initialize(&mut self) -> Result<()> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "id": self.next_id,
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "claude-code-simulator",
                    "version": "1.0.0"
                }
            }
        });
        self.next_id += 1;

        let response = self.send_request(request).await?;

        // Validate initialization response
        if response.get("error").is_some() {
            anyhow::bail!("MCP initialization failed: {:?}", response["error"]);
        }

        // Send notifications/initialized
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });

        let notification_str = serde_json::to_string(&notification)? + "\n";
        let stdin = self
            .process
            .stdin
            .as_mut()
            .context("Failed to get stdin")?;
        stdin.write_all(notification_str.as_bytes()).await?;
        stdin.flush().await?;

        // Give server time to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    /// Send a JSON-RPC request and read response
    async fn send_request(&mut self, request: Value) -> Result<Value> {
        let stdin = self
            .process
            .stdin
            .as_mut()
            .context("Failed to get stdin")?;
        let stdout = self
            .process
            .stdout
            .as_mut()
            .context("Failed to get stdout")?;

        // Send request
        let request_str = serde_json::to_string(&request)? + "\n";
        stdin.write_all(request_str.as_bytes()).await?;
        stdin.flush().await?;

        // Read response with timeout
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        timeout(Duration::from_secs(15), reader.read_line(&mut line))
            .await
            .context("Timeout waiting for MCP response")??;

        // Parse response
        let response: Value = serde_json::from_str(&line)
            .context(format!("Failed to parse MCP response: {}", line))?;

        Ok(response)
    }

    /// Discover relevant skills using semantic search
    ///
    /// Simulates Claude Code's skill discovery process by querying the
    /// `search_skills` MCP tool with a user prompt.
    ///
    /// # Arguments
    ///
    /// * `query` - User's natural language query (e.g., "kubernetes pods")
    ///
    /// # Returns
    ///
    /// Vector of skill names that are relevant to the query
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let skills = sim.discover_skill("show me docker containers").await?;
    /// assert!(skills.contains(&"docker".to_string()));
    /// ```
    pub async fn discover_skill(&mut self, query: &str) -> Result<Vec<String>> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "id": self.next_id,
            "params": {
                "name": "search_skills",
                "arguments": {
                    "query": query,
                    "top_k": 5
                }
            }
        });
        self.next_id += 1;

        let response = self.send_request(request).await?;

        if let Some(error) = response.get("error") {
            anyhow::bail!("search_skills failed: {:?}", error);
        }

        // Parse search results to extract skill names
        let result_text = response["result"]["content"][0]["text"]
            .as_str()
            .context("Missing result text")?;

        // Extract skill names from search results
        // Format: "**skill_name**: description"
        let skills: Vec<String> = result_text
            .lines()
            .filter_map(|line| {
                if line.starts_with("**") {
                    let skill_name = line
                        .trim_start_matches("**")
                        .split("**")
                        .next()?
                        .split(':')
                        .next()?
                        .trim();
                    Some(skill_name.to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(skills)
    }

    /// Execute a skill tool
    ///
    /// Simulates Claude Code executing a tool with the provided arguments.
    ///
    /// # Arguments
    ///
    /// * `skill` - Skill name (e.g., "kubernetes")
    /// * `tool` - Tool name (e.g., "get")
    /// * `args` - Tool arguments as JSON
    /// * `context_opts` - Optional context engineering options
    ///
    /// # Returns
    ///
    /// The tool's output as a string
    pub async fn execute_tool(
        &mut self,
        skill: &str,
        tool: &str,
        args: Value,
        context_opts: Option<Value>,
    ) -> Result<String> {
        let mut arguments = json!({
            "skill": skill,
            "tool": tool,
            "args": args
        });

        // Merge context engineering options
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

        let response = self.send_request(request).await?;

        if let Some(error) = response.get("error") {
            anyhow::bail!("Tool execution failed: {:?}", error);
        }

        let result_text = response["result"]["content"][0]["text"]
            .as_str()
            .context("Missing result text")?
            .to_string();

        Ok(result_text)
    }

    /// Execute a tool with context engineering filters
    ///
    /// Convenience method for executing tools with common context engineering
    /// options like grep filtering and jq JSON extraction.
    ///
    /// # Arguments
    ///
    /// * `skill` - Skill name
    /// * `tool` - Tool name
    /// * `args` - Tool arguments
    /// * `grep` - Optional grep pattern to filter output
    /// * `jq` - Optional jq expression for JSON extraction
    /// * `head` - Optional limit to first N lines
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// // Get only running pods
    /// let output = sim.apply_context_engineering(
    ///     "kubernetes",
    ///     "get",
    ///     json!({"resource": "pods"}),
    ///     Some("Running"),
    ///     None,
    ///     None
    /// ).await?;
    /// ```
    pub async fn apply_context_engineering(
        &mut self,
        skill: &str,
        tool: &str,
        args: Value,
        grep: Option<&str>,
        jq: Option<&str>,
        head: Option<usize>,
    ) -> Result<String> {
        let mut context_opts = json!({});

        if let Some(pattern) = grep {
            context_opts["grep"] = json!(pattern);
        }

        if let Some(expr) = jq {
            context_opts["jq"] = json!(expr);
        }

        if let Some(n) = head {
            context_opts["head"] = json!(n);
        }

        self.execute_tool(skill, tool, args, Some(context_opts))
            .await
    }

    /// List all available skills
    ///
    /// Queries the MCP server for the list of all installed skills.
    ///
    /// # Returns
    ///
    /// Vector of skill names
    pub async fn list_skills(&mut self) -> Result<Vec<String>> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "id": self.next_id,
            "params": {
                "name": "list_skills",
                "arguments": {}
            }
        });
        self.next_id += 1;

        let response = self.send_request(request).await?;

        if let Some(error) = response.get("error") {
            anyhow::bail!("list_skills failed: {:?}", error);
        }

        let result_text = response["result"]["content"][0]["text"]
            .as_str()
            .context("Missing result text")?;

        // Extract skill names from list output
        let skills: Vec<String> = result_text
            .lines()
            .filter_map(|line| {
                // Format: "skill_name: description"
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    Some(parts[0].trim().to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(skills)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires skill binary and skills installed
    async fn test_simulator_initialization() {
        let sim = ClaudeCodeSimulator::new().await;
        assert!(sim.is_ok(), "Simulator should initialize successfully");
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_skills() {
        let mut sim = ClaudeCodeSimulator::new().await.unwrap();
        let skills = sim.list_skills().await.unwrap();
        assert!(
            !skills.is_empty(),
            "Should find at least one installed skill"
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_discover_skill() {
        let mut sim = ClaudeCodeSimulator::new().await.unwrap();
        let skills = sim.discover_skill("kubernetes").await.unwrap();
        assert!(
            skills.iter().any(|s| s.contains("kubernetes")),
            "Should discover kubernetes skill"
        );
    }
}
