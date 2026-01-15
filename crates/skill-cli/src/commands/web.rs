//! Web command - Start the Skill Engine web interface
//!
//! This command starts an HTTP server that serves both the REST API
//! and an embedded web UI built with Yew/WASM.

use anyhow::Result;
use colored::*;

/// Execute the web command
pub async fn execute(host: &str, port: u16, open_browser: bool) -> Result<()> {
    let url = format!("http://{}:{}", host, port);

    // Print startup banner
    println!();
    println!("{}", "Skill Engine Web Interface".cyan().bold());
    println!("{}", "━".repeat(40).dimmed());
    println!();

    // Open browser if requested
    if open_browser {
        println!("{} Opening browser at {}", "→".blue(), url.cyan());
        if let Err(e) = open_url(&url) {
            eprintln!("{} Failed to open browser: {}", "!".yellow(), e);
        }
    }

    println!("{} Starting server on {}", "→".blue(), url.cyan());
    println!();
    println!("  {} {}/", "Web UI:".dimmed(), url);
    println!("  {} {}/api/...", "API:".dimmed(), url);
    println!();
    println!("{}", "Press Ctrl+C to stop".dimmed());
    println!();

    // Start the server with web UI enabled
    skill_http::serve_with_ui(host, port).await
}

/// Open a URL in the default browser
fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()?;
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        eprintln!("Auto-open not supported on this platform. Please open {} manually.", url);
    }

    Ok(())
}
