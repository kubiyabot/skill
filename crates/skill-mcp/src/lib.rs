//! Skill MCP Server - Model Context Protocol integration
//!
//! This crate provides an MCP server that exposes installed skills as MCP tools,
//! allowing AI agents like Claude to discover and execute skill tools.
//!
//! # Features
//!
//! - **Dynamic Tool Discovery**: Automatically discovers tools from installed skills
//! - **SKILL.md Integration**: Uses SKILL.md documentation for rich tool descriptions
//! - **Manifest Support**: Works with `.skill-engine.toml` declarative manifests
//! - **Stdio Transport**: Uses stdio for direct Claude Code integration
//!
//! # Usage
//!
//! ```bash
//! # Start MCP server
//! skill serve
//!
//! # Or programmatically
//! use skill_mcp::McpServer;
//! let server = McpServer::new()?;
//! server.run().await?;
//! ```

pub mod server;

pub use server::{DiscoveredTool, McpServer, ToolParameter};

use anyhow::Result;
use skill_runtime::SkillManifest;

/// Start the MCP server with default configuration
pub async fn serve() -> Result<()> {
    let server = McpServer::new()?;
    server.run().await
}

/// Start the MCP server with a manifest
pub async fn serve_with_manifest(manifest: SkillManifest) -> Result<()> {
    let server = McpServer::with_manifest(manifest)?;
    server.run().await
}
