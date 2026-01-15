//! Skill info command - shows SKILL.md documentation and metadata
use anyhow::{Context, Result};
use colored::*;
use skill_runtime::{find_skill_md, parse_skill_md, SkillManifest, LocalSkillLoader};
use std::path::PathBuf;

pub async fn execute(skill_name: &str, manifest: Option<&SkillManifest>) -> Result<()> {
    // Find the skill path
    let skill_path = find_skill_path(skill_name, manifest)?;

    // Load SKILL.md
    let loader = LocalSkillLoader::new()?;
    let skill_md = loader.load_skill_md(&skill_path);

    if let Some(md) = skill_md {
        print_skill_info(&md, &skill_path);
    } else {
        // Try to find SKILL.md directly
        if let Some(skill_md_path) = find_skill_md(&skill_path) {
            let md = parse_skill_md(&skill_md_path)
                .with_context(|| format!("Failed to parse SKILL.md: {}", skill_md_path.display()))?;
            print_skill_info(&md, &skill_path);
        } else {
            println!("{} No SKILL.md found for '{}'", "!".yellow(), skill_name);
            println!();
            println!("  Searched in: {}", skill_path.display());
            println!();
            println!(
                "  Create a {} file in the skill directory with:",
                "SKILL.md".cyan()
            );
            println!("  - YAML frontmatter (name, description, allowed-tools)");
            println!("  - Tool documentation sections");
            println!("  - Usage examples");
        }
    }

    Ok(())
}

fn find_skill_path(skill_name: &str, manifest: Option<&SkillManifest>) -> Result<PathBuf> {
    // Check manifest first
    if let Some(manifest) = manifest {
        if let Some(skill) = manifest.get_skill(skill_name) {
            let source = &skill.source;

            // Resolve relative paths
            let resolved = if source.starts_with("./") || source.starts_with("../") {
                manifest.base_dir.join(source)
            } else {
                PathBuf::from(source)
            };

            if resolved.exists() {
                return Ok(resolved);
            }
        }
    }

    // Check installed skills
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let registry_dir = home.join(".skill-engine").join("registry");
    let skill_dir = registry_dir.join(skill_name);

    if skill_dir.exists() {
        return Ok(skill_dir);
    }

    // Try as a direct path
    let direct_path = PathBuf::from(skill_name);
    if direct_path.exists() {
        return Ok(direct_path);
    }

    anyhow::bail!(
        "Skill '{}' not found. Check installed skills with: skill list",
        skill_name
    )
}

fn print_skill_info(md: &skill_runtime::SkillMdContent, path: &std::path::Path) {
    println!();
    println!(
        "{} {}",
        "Skill:".bold(),
        if md.frontmatter.name.is_empty() {
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            md.frontmatter.name.clone()
        }
        .cyan()
        .bold()
    );
    println!("{}", "─".repeat(70).dimmed());

    // Description
    if !md.frontmatter.description.is_empty() {
        println!();
        println!("{}", md.frontmatter.description);
    }

    // Allowed tools
    if let Some(ref tools) = md.frontmatter.allowed_tools {
        println!();
        println!("{}: {}", "Allowed Tools".bold(), tools.yellow());
    }

    // When to use
    if let Some(ref when_to_use) = md.when_to_use {
        println!();
        println!("{}", "When to Use".bold());
        for line in when_to_use.lines() {
            println!("  {}", line.dimmed());
        }
    }

    // Tools provided
    if !md.tool_docs.is_empty() {
        println!();
        println!("{} ({} tools)", "Tools Provided".bold(), md.tool_docs.len().to_string().yellow());
        println!();

        let mut tools: Vec<_> = md.tool_docs.iter().collect();
        tools.sort_by_key(|(name, _)| *name);

        for (name, tool) in tools {
            println!("  {} {}", "•".cyan(), name.cyan().bold());
            if !tool.description.is_empty() {
                println!("    {}", tool.description.dimmed());
            }
            if !tool.examples.is_empty() {
                println!(
                    "    {} {} example(s)",
                    "→".dimmed(),
                    tool.examples.len()
                );
            }
        }
    }

    // Code examples summary
    if !md.examples.is_empty() {
        println!();
        println!(
            "{}: {} code example(s) in documentation",
            "Examples".bold(),
            md.examples.len().to_string().yellow()
        );
    }

    // Configuration
    if let Some(ref config) = md.configuration {
        println!();
        println!("{}", "Configuration".bold());
        for line in config.lines().take(5) {
            println!("  {}", line.dimmed());
        }
        if config.lines().count() > 5 {
            println!("  {}", "...".dimmed());
        }
    }

    // Path
    println!();
    println!("{}: {}", "Path".dimmed(), path.display().to_string().dimmed());
    println!();
}
