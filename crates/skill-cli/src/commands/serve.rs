use anyhow::{Context, Result};
use colored::*;
use skill_mcp::McpServer;
use skill_runtime::SkillManifest;
use std::process::{Child, Command};

pub async fn execute(skill: Option<&str>, host: &str, port: u16, http: bool, with_web: bool) -> Result<()> {
    // Start trunk serve if --with-web flag is set
    let mut trunk_process: Option<Child> = None;
    if with_web {
        trunk_process = start_trunk_serve()?;
    }
    // Load manifest if available
    let manifest = load_manifest_for_serve()?;

    if http {
        // HTTP streaming mode
        println!("{} Starting Skill Engine MCP Server (HTTP Streaming)...", "🚀".green());
        println!();
        println!("Server will be available at: {}", format!("http://{}:{}/mcp", host, port).cyan());
        println!();
        println!("This endpoint supports:");
        println!("  • Server-Sent Events (SSE) for real-time streaming");
        println!("  • Session-based connections for stateful interactions");
        println!("  • Standard MCP protocol over HTTP");
        println!();

        if let Some(ref m) = manifest {
            println!("{} Loaded manifest with {} skills", "✓".green(), m.skill_names().len());
        } else {
            println!("{} No manifest found, using installed skills only", "ℹ".blue());
        }

        println!();
        println!("{} MCP HTTP server starting...", "✓".green());

        // Run HTTP server
        McpServer::run_http(host, port, manifest).await?;
    } else {
        // Stdio mode (default for Claude Code)
        if skill.is_some() {
            println!("{} skill filter option not yet supported", "Note:".yellow().bold());
            println!();
        }

        println!("{} Starting Skill Engine MCP Server...", "🚀".green());
        println!();
        println!("Add to your {} configuration:", ".mcp.json".cyan());
        println!();
        println!(r#"  {{
    "mcpServers": {{
      "skill-engine": {{
        "command": "skill",
        "args": ["serve"]
      }}
    }}
  }}"#);
        println!();

        // Create and run the MCP server
        let server = if let Some(manifest) = manifest {
            println!("{} Loaded manifest with {} skills", "✓".green(), manifest.skill_names().len());
            McpServer::with_manifest(manifest)?
        } else {
            println!("{} No manifest found, using installed skills only", "ℹ".blue());
            McpServer::new()?
        };

        println!();
        println!("{} MCP server ready - waiting for connections...", "✓".green());
        println!();

        // Run the server (this blocks until the client disconnects)
        server.run().await?;
    }

    // Cleanup: kill trunk process if it was started
    if let Some(mut child) = trunk_process {
        eprintln!();
        eprintln!("{} Shutting down web interface...", "🛑".yellow());
        let _ = child.kill();
    }

    Ok(())
}

/// Load manifest from current directory or parent directories
fn load_manifest_for_serve() -> Result<Option<SkillManifest>> {
    let cwd = std::env::current_dir()?;

    // Check common manifest locations
    let candidates = vec![
        cwd.join(".skill-engine.toml"),
        cwd.join("skill-engine.toml"),
    ];

    for path in candidates {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let mut manifest: SkillManifest = toml::from_str(&content)?;
            manifest.base_dir = path.parent().unwrap_or(&cwd).to_path_buf();
            return Ok(Some(manifest));
        }
    }

    Ok(None)
}

/// Start trunk serve for the web interface
fn start_trunk_serve() -> Result<Option<Child>> {
    eprintln!("{} Starting web interface (trunk serve)...", "🌐".cyan());

    // Find the skill-web crate directory
    let workspace_root = std::env::current_dir()?;
    let web_dir = workspace_root.join("crates/skill-web");

    if !web_dir.exists() {
        eprintln!("{} Web interface directory not found at: {}", "⚠".yellow(), web_dir.display());
        eprintln!("{} Continuing without web interface", "ℹ".blue());
        return Ok(None);
    }

    // Check if trunk is installed
    let trunk_check = Command::new("trunk")
        .arg("--version")
        .output();

    if trunk_check.is_err() {
        eprintln!("{} trunk not found. Install with: cargo install trunk", "⚠".yellow());
        eprintln!("{} Continuing without web interface", "ℹ".blue());
        return Ok(None);
    }

    // Start trunk serve
    let child = Command::new("trunk")
        .arg("serve")
        .arg("--port")
        .arg("8080")
        .arg("--open")
        .current_dir(&web_dir)
        .spawn()
        .context("Failed to start trunk serve")?;

    eprintln!("{} Web interface starting at: {}", "✓".green(), "http://localhost:8080".cyan());
    eprintln!();

    Ok(Some(child))
}
