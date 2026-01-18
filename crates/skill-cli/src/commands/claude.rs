//! Claude Code integration commands
//!
//! Provides commands to integrate Skill Engine with Claude Code:
//! - `skill claude setup` - Configure Claude Code to use the Skill Engine MCP server
//! - `skill claude status` - Check Claude Code integration status
//! - `skill claude remove` - Remove Skill Engine from Claude Code
//! - `skill claude generate` - Generate Claude Agent Skills from installed skills

use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::commands::claude_bridge::{self, GenerateOptions};

/// MCP server configuration for Claude Code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub server_type: Option<String>,
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

/// .mcp.json file structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

/// Setup Claude Code integration
pub async fn setup(
    global: bool,
    server_name: Option<&str>,
    binary_path: Option<&str>,
) -> Result<()> {
    let server_name = server_name.unwrap_or("skill-engine");

    // Find the skill binary path
    let skill_binary = if let Some(path) = binary_path {
        PathBuf::from(path)
    } else {
        find_skill_binary()?
    };

    // Verify the binary exists
    if !skill_binary.exists() {
        anyhow::bail!(
            "Skill binary not found at: {}\n\
             Install with: curl -fsSL https://raw.githubusercontent.com/kubiyabot/skill/main/install.sh | sh",
            skill_binary.display()
        );
    }

    // Determine config file location
    let config_path = if global {
        get_global_mcp_config_path()?
    } else {
        get_project_mcp_config_path()?
    };

    // Load or create config
    let mut config = load_mcp_config(&config_path)?;

    // Add or update skill-engine server
    let server_config = McpServerConfig {
        server_type: Some("stdio".to_string()),
        command: skill_binary.to_string_lossy().to_string(),
        args: vec!["serve".to_string()],
        env: HashMap::new(),
    };

    let was_existing = config.mcp_servers.contains_key(server_name);
    config.mcp_servers.insert(server_name.to_string(), server_config);

    // Save config
    save_mcp_config(&config_path, &config)?;

    // Print success message
    println!();
    if was_existing {
        println!(
            "{} Updated {} in {}",
            "✓".green().bold(),
            server_name.cyan(),
            config_path.display()
        );
    } else {
        println!(
            "{} Added {} to {}",
            "✓".green().bold(),
            server_name.cyan(),
            config_path.display()
        );
    }

    println!();
    println!("{}", "Claude Code Integration Configured!".green().bold());
    println!();
    println!("Claude Code now has access to these MCP tools:");
    println!("  • {} - Run any skill tool", "execute".cyan());
    println!("  • {} - List available skills and tools", "list_skills".cyan());
    println!("  • {} - Find tools by natural language", "search_skills".cyan());
    println!();

    if !global {
        println!(
            "{} Restart Claude Code or run {} to apply changes",
            "Note:".yellow().bold(),
            "/mcp".cyan()
        );
    }

    println!();
    println!("Example prompts for Claude:");
    println!(
        "  \"{}\"",
        "List all available Kubernetes tools".italic()
    );
    println!(
        "  \"{}\"",
        "Get pods from the default namespace".italic()
    );
    println!(
        "  \"{}\"",
        "Find a tool to convert video to GIF".italic()
    );

    Ok(())
}

/// Check Claude Code integration status
pub async fn status() -> Result<()> {
    println!();
    println!("{}", "Claude Code Integration Status".bold());
    println!("{}", "─".repeat(50));

    // Check project-level config
    let project_config_path = get_project_mcp_config_path()?;
    if project_config_path.exists() {
        let config = load_mcp_config(&project_config_path)?;
        println!();
        println!(
            "{} Project config: {}",
            "📁".to_string(),
            project_config_path.display()
        );

        if let Some(server) = config.mcp_servers.get("skill-engine") {
            println!(
                "   {} skill-engine: {} serve",
                "✓".green(),
                server.command
            );
        } else {
            println!("   {} skill-engine: {}", "✗".red(), "not configured".dimmed());
        }

        // List other MCP servers
        for (name, _server) in config.mcp_servers.iter() {
            if name != "skill-engine" {
                println!("   • {}", name.dimmed());
            }
        }
    } else {
        println!();
        println!(
            "{} Project config: {} (not found)",
            "📁".to_string(),
            project_config_path.display().to_string().dimmed()
        );
    }

    // Check global config
    if let Ok(global_config_path) = get_global_mcp_config_path() {
        if global_config_path.exists() {
            let config = load_mcp_config(&global_config_path)?;
            println!();
            println!(
                "{} Global config: {}",
                "🌐".to_string(),
                global_config_path.display()
            );

            if let Some(server) = config.mcp_servers.get("skill-engine") {
                println!(
                    "   {} skill-engine: {} serve",
                    "✓".green(),
                    server.command
                );
            } else {
                println!("   {} skill-engine: {}", "✗".red(), "not configured".dimmed());
            }
        }
    }

    // Check skill binary
    println!();
    println!("{}", "Skill Binary".bold());
    if let Ok(binary_path) = find_skill_binary() {
        if binary_path.exists() {
            println!(
                "   {} Found: {}",
                "✓".green(),
                binary_path.display()
            );
        } else {
            println!(
                "   {} Not installed: {}",
                "✗".red(),
                binary_path.display()
            );
        }
    } else {
        println!("   {} Unable to locate skill binary", "✗".red());
    }

    println!();

    Ok(())
}

/// Remove Skill Engine from Claude Code
pub async fn remove(global: bool, server_name: Option<&str>) -> Result<()> {
    let server_name = server_name.unwrap_or("skill-engine");

    let config_path = if global {
        get_global_mcp_config_path()?
    } else {
        get_project_mcp_config_path()?
    };

    if !config_path.exists() {
        println!(
            "{} Config file not found: {}",
            "⚠".yellow(),
            config_path.display()
        );
        return Ok(());
    }

    let mut config = load_mcp_config(&config_path)?;

    if config.mcp_servers.remove(server_name).is_some() {
        save_mcp_config(&config_path, &config)?;
        println!(
            "{} Removed {} from {}",
            "✓".green().bold(),
            server_name.cyan(),
            config_path.display()
        );
    } else {
        println!(
            "{} {} not found in {}",
            "⚠".yellow(),
            server_name,
            config_path.display()
        );
    }

    Ok(())
}

/// Find the skill binary path
fn find_skill_binary() -> Result<PathBuf> {
    // First try: Check if we're running from the built binary
    if let Ok(current_exe) = std::env::current_exe() {
        if current_exe.file_name().map(|n| n.to_string_lossy().contains("skill")).unwrap_or(false) {
            return Ok(current_exe);
        }
    }

    // Second try: Check ~/.skill-engine/bin/skill
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let install_path = PathBuf::from(&home).join(".skill-engine/bin/skill");
    if install_path.exists() {
        return Ok(install_path);
    }

    // Third try: Check PATH
    if let Ok(output) = std::process::Command::new("which").arg("skill").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }
    }

    // Default to install path (may not exist)
    Ok(install_path)
}

/// Get project-level .mcp.json path
fn get_project_mcp_config_path() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("Unable to get current directory")?;
    Ok(cwd.join(".mcp.json"))
}

/// Get global Claude Code config path
fn get_global_mcp_config_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;

    // Claude Code uses different paths on different platforms
    #[cfg(target_os = "macos")]
    let config_dir = PathBuf::from(&home).join(".config/claude");

    #[cfg(target_os = "linux")]
    let config_dir = PathBuf::from(&home).join(".config/claude");

    #[cfg(target_os = "windows")]
    let config_dir = PathBuf::from(&home).join("AppData/Roaming/claude");

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let config_dir = PathBuf::from(&home).join(".config/claude");

    Ok(config_dir.join("mcp.json"))
}

/// Load MCP config from file
fn load_mcp_config(path: &PathBuf) -> Result<McpConfig> {
    if !path.exists() {
        return Ok(McpConfig::default());
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))
}

/// Save MCP config to file
fn save_mcp_config(path: &PathBuf, config: &McpConfig) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let content = serde_json::to_string_pretty(config)
        .context("Failed to serialize MCP config")?;

    std::fs::write(path, content)
        .with_context(|| format!("Failed to write {}", path.display()))?;

    Ok(())
}

/// Generate Claude Agent Skills from installed skills
pub async fn generate(
    skill: Option<String>,
    output: Option<PathBuf>,
    force: bool,
    dry_run: bool,
    no_scripts: bool,
    project: bool,
) -> Result<()> {
    // Determine output directory
    let output_dir = if let Some(path) = output {
        path
    } else if project {
        std::env::current_dir()?.join(".claude").join("skills")
    } else {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        PathBuf::from(home).join(".claude").join("skills")
    };

    println!();
    println!("{}", "Claude Agent Skills Generator".bold());
    println!("{}", "─".repeat(50));

    if dry_run {
        println!("{} Dry run mode - no files will be written", "ℹ".blue());
    }

    println!();
    println!("Output directory: {}", output_dir.display().to_string().cyan());

    if let Some(ref name) = skill {
        println!("Generating skill: {}", name.cyan());
    } else {
        println!("Generating: {}", "all skills".cyan());
    }

    println!();

    // Build options
    let options = GenerateOptions {
        output_dir: output_dir.clone(),
        skill_name: skill,
        manifest_path: None,
        force,
        dry_run,
        no_scripts,
        project,
    };

    // Generate
    let result = claude_bridge::generate(options).await?;

    // Print results
    if dry_run {
        println!("{}", "Would generate:".bold());
        for output in &result.dry_run_output {
            println!("  {} {}", "→".cyan(), output);
        }
    } else {
        println!("{}", "Generated Claude Agent Skills:".green().bold());
        for skill_name in &result.generated_skills {
            let _skill_dir = output_dir.join(skill_name);
            println!();
            println!("  {} {}", "✓".green(), skill_name.cyan().bold());
            println!("    {} SKILL.md", "├─".dimmed());
            println!("    {} TOOLS.md", "├─".dimmed());
            if !no_scripts {
                println!("    {} scripts/", "└─".dimmed());
            }
        }
    }

    // Print warnings
    if !result.warnings.is_empty() {
        println!();
        println!("{}", "Warnings:".yellow().bold());
        for warning in &result.warnings {
            println!("  {} {}", "⚠".yellow(), warning);
        }
    }

    println!();

    if !dry_run && !result.generated_skills.is_empty() {
        println!("{}", "Next steps:".bold());
        println!();
        println!("  1. Claude Code will automatically discover skills in:");
        println!("     {}", output_dir.display().to_string().cyan());
        println!();
        println!("  2. Test by asking Claude:");
        println!("     \"{}\"", "What skills do you have available?".italic());
        println!();
        println!("  3. Claude will use MCP tools when available, scripts as fallback");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_config_serialization() {
        let mut config = McpConfig::default();
        config.mcp_servers.insert(
            "skill-engine".to_string(),
            McpServerConfig {
                server_type: Some("stdio".to_string()),
                command: "/usr/local/bin/skill".to_string(),
                args: vec!["serve".to_string()],
                env: HashMap::new(),
            },
        );

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("skill-engine"));
        assert!(json.contains("mcpServers"));
        assert!(json.contains("serve"));

        let parsed: McpConfig = serde_json::from_str(&json).unwrap();
        assert!(parsed.mcp_servers.contains_key("skill-engine"));
    }
}
