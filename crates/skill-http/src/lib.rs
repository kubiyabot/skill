//! Skill HTTP Server - High-performance REST API with streaming
//!
//! This crate provides a complete HTTP REST API for the Skill Engine.
//!
//! ## Features
//!
//! - **Skills Management**: List, install, uninstall skills
//! - **Tool Execution**: Execute skill tools with arguments
//! - **Search**: Semantic search across skills and tools
//! - **Configuration**: Runtime configuration management
//! - **Health Checks**: Monitor server and component health
//!
//! ## API Endpoints
//!
//! ### Skills
//! - `GET /api/skills` - List all installed skills
//! - `POST /api/skills` - Install a new skill
//! - `GET /api/skills/{name}` - Get skill details
//! - `DELETE /api/skills/{name}` - Uninstall a skill
//!
//! ### Execution
//! - `POST /api/execute` - Execute a tool
//! - `GET /api/executions` - List execution history
//! - `GET /api/executions/{id}` - Get execution details
//!
//! ### Search
//! - `POST /api/search` - Semantic search for skills/tools
//! - `GET /api/search/config` - Get search configuration
//! - `PUT /api/search/config` - Update search configuration
//!
//! ### Configuration
//! - `GET /api/config` - Get application configuration
//! - `PUT /api/config` - Update application configuration
//!
//! ### Health
//! - `GET /api/health` - Health check
//! - `GET /api/version` - Version information
//!
//! ## Example
//!
//! ```ignore
//! use skill_http::{HttpServer, HttpServerConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = HttpServerConfig {
//!         host: "127.0.0.1".to_string(),
//!         port: 3000,
//!         enable_cors: true,
//!         enable_tracing: true,
//!     };
//!
//!     let server = HttpServer::with_config(config)?;
//!     server.run().await
//! }
//! ```

pub mod analytics;
pub mod embedded;
pub mod execution_history;
pub mod handlers;
pub mod middleware;
pub mod openapi;
pub mod routes;
pub mod server;
pub mod types;

pub use server::{AppState, HttpServer, HttpServerConfig};
pub use types::*;

use anyhow::Result;

/// Start the HTTP server with default configuration (API only)
pub async fn serve(host: &str, port: u16) -> Result<()> {
    let config = HttpServerConfig {
        host: host.to_string(),
        port,
        enable_cors: true,
        enable_tracing: true,
        enable_web_ui: false,
        working_dir: None,
    };
    let server = HttpServer::with_config(config)?;
    server.run().await
}

/// Start the HTTP server with embedded web UI
pub async fn serve_with_ui(host: &str, port: u16) -> Result<()> {
    let config = HttpServerConfig {
        host: host.to_string(),
        port,
        enable_cors: true,
        enable_tracing: true,
        enable_web_ui: true,
        working_dir: None,
    };
    let server = HttpServer::with_config(config)?;
    server.run().await
}
