//! AI-enhanced example generation command
//!
//! Generates synthetic usage examples for installed skills using LLMs,
//! with real-time streaming progress display.

use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

#[cfg(feature = "ai-ingestion")]
use skill_runtime::{
    SearchConfig, SearchPipeline, GenerationEvent, ToolDocumentation,
    IndexDocument, DocumentMetadata,
};

#[cfg(feature = "ai-ingestion")]
use tokio_stream::StreamExt;

#[cfg(feature = "ai-ingestion")]
use std::path::PathBuf;

/// Execute the enhance command
pub async fn execute(
    skill_name: Option<&str>,
    all: bool,
    stream: bool,
    examples_per_tool: usize,
) -> Result<()> {
    #[cfg(not(feature = "ai-ingestion"))]
    {
        eprintln!(
            "{} AI ingestion feature not enabled.",
            "Error:".red().bold()
        );
        eprintln!(
            "{}",
            "Rebuild with: cargo build --features ai-ingestion".dimmed()
        );
        let _ = (skill_name, all, stream, examples_per_tool); // Suppress unused warnings
        return Ok(());
    }

    #[cfg(feature = "ai-ingestion")]
    {
        execute_with_ai(skill_name, all, stream, examples_per_tool).await
    }
}

#[cfg(feature = "ai-ingestion")]
async fn execute_with_ai(
    skill_name: Option<&str>,
    all: bool,
    stream: bool,
    _examples_per_tool: usize,
) -> Result<()> {
    // Load search config
    let config = load_config()?;

    // Check if AI ingestion is enabled
    if !config.ai_ingestion.enabled {
        eprintln!("{}", "AI ingestion not enabled in configuration.".yellow());
        eprintln!();
        eprintln!("{}", "To enable AI-enhanced example generation:".bold());
        eprintln!("  1. Run: {}", "skill setup".cyan());
        eprintln!("  2. Or edit: {}", "~/.skill-engine/search.toml".cyan());
        eprintln!("     Set [ai_ingestion] enabled = true");
        eprintln!();
        eprintln!("{}", "Providers available:".bold());
        eprintln!("  - {} (local, offline)", "ollama".green());
        eprintln!("  - {} (cloud API)", "openai".green());
        return Ok(());
    }

    // Get skills to enhance
    let skills = get_skills_to_enhance(skill_name, all)?;

    if skills.is_empty() {
        eprintln!("{}", "No skills found to enhance.".yellow());
        return Ok(());
    }

    // Header
    println!();
    println!("{}", "🤖 AI-Enhanced Example Generation".bold());
    println!("{}", "━".repeat(40));
    println!();

    // Show provider info
    println!(
        "{} {} / {}",
        "Provider:".dimmed(),
        format!("{:?}", config.ai_ingestion.provider).cyan(),
        config.ai_ingestion.model.cyan()
    );
    println!(
        "{} {} skills",
        "Enhancing:".dimmed(),
        skills.len().to_string().green()
    );
    println!();

    // Create search pipeline with AI ingestion
    let pipeline = SearchPipeline::from_config(config.clone())
        .await
        .context("Failed to create search pipeline")?;

    // Check if generator is available
    if !pipeline.has_example_generator() {
        eprintln!(
            "{} Failed to initialize LLM provider.",
            "Error:".red().bold()
        );
        eprintln!("{}", "Check your configuration and ensure the provider is accessible.".dimmed());
        return Ok(());
    }

    // Process each skill
    for skill_name in &skills {
        println!(
            " {} {}",
            "→".cyan(),
            skill_name.bold()
        );

        // Load tools for this skill
        let tools = load_skill_tools(skill_name)?;

        if tools.is_empty() {
            println!("   {} No tools found", "⚠".yellow());
            continue;
        }

        println!(
            "   {} {} tools",
            "Found:".dimmed(),
            tools.len()
        );

        // Create progress bar
        let pb = if stream {
            None
        } else {
            let bar = ProgressBar::new(tools.len() as u64);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("   {bar:30.cyan/blue} {pos}/{len} tools [{elapsed}]")
                    .unwrap()
                    .progress_chars("█▓░")
            );
            Some(bar)
        };

        // Convert to IndexDocument for pipeline
        let documents: Vec<IndexDocument> = tools
            .iter()
            .map(|t| IndexDocument {
                id: format!("{}:{}", skill_name, t.name),
                content: build_embedding_text(t),
                metadata: DocumentMetadata {
                    skill_name: Some(skill_name.clone()),
                    tool_name: Some(t.name.clone()),
                    ..Default::default()
                },
            })
            .collect();

        if stream {
            // Stream events to terminal
            let mut event_stream = Box::pin(pipeline.index_documents_stream(documents, tools.clone()));
            let mut examples_count = 0;
            let mut valid_count = 0;

            while let Some(event) = event_stream.next().await {
                match event {
                    GenerationEvent::Started { tool_name, .. } => {
                        println!("   {} {}", "▸".blue(), tool_name.dimmed());
                    }
                    GenerationEvent::Thinking { thought } => {
                        println!("     {} {}", "💭".dimmed(), thought.dimmed());
                    }
                    GenerationEvent::Example { example } => {
                        examples_count += 1;
                        println!(
                            "     {} {}",
                            "✓".green(),
                            truncate_command(&example.command, 60).green()
                        );
                    }
                    GenerationEvent::Validation { valid, errors, example_index } => {
                        if valid {
                            valid_count += 1;
                        } else {
                            println!(
                                "     {} Example {}: {}",
                                "⚠".yellow(),
                                example_index,
                                errors.join(", ").dimmed()
                            );
                        }
                    }
                    GenerationEvent::ToolCompleted { tool_name, examples_generated, valid_examples, duration_ms } => {
                        println!(
                            "   {} {} ({}/{} valid, {}ms)",
                            "✓".green(),
                            tool_name.bold(),
                            valid_examples,
                            examples_generated,
                            duration_ms
                        );
                    }
                    GenerationEvent::Error { message, tool_name, .. } => {
                        let prefix = tool_name.map(|n| format!(" [{}]", n)).unwrap_or_default();
                        eprintln!("   {}{} {}", "✗".red(), prefix, message.red());
                    }
                    GenerationEvent::Completed { total_examples, total_valid, total_tools, duration_ms } => {
                        println!();
                        println!(
                            "   {} {} examples generated ({} valid) from {} tools in {}ms",
                            "Done:".green().bold(),
                            total_examples,
                            total_valid,
                            total_tools,
                            duration_ms
                        );
                    }
                    _ => {}
                }
            }

            println!(
                "   {} Generated {} examples ({} valid)",
                "Summary:".dimmed(),
                examples_count,
                valid_count
            );
        } else {
            // Non-streaming: use index_documents_with_generation
            let (stats, examples) = pipeline
                .index_documents_with_generation(documents, tools.clone())
                .await
                .context("Failed to generate examples")?;

            if let Some(bar) = &pb {
                bar.finish_with_message("complete");
            }

            let valid_count = examples.iter().filter(|e| e.validated).count();
            println!(
                "   {} {} examples ({} valid), {} documents indexed",
                "✓".green(),
                examples.len(),
                valid_count,
                stats.total_documents
            );
        }

        println!();
    }

    // Summary
    println!("{}", "━".repeat(40));
    println!(
        "{} Enhanced {} skill(s)",
        "✓".green().bold(),
        skills.len()
    );
    println!();

    Ok(())
}

/// Get list of skill names to enhance
fn get_skills_to_enhance(skill_name: Option<&str>, all: bool) -> Result<Vec<String>> {
    if let Some(name) = skill_name {
        return Ok(vec![name.to_string()]);
    }

    if !all {
        anyhow::bail!(
            "Please specify a skill name or use --all to enhance all installed skills.\n\
             Example: skill enhance kubernetes\n\
             Example: skill enhance --all"
        );
    }

    // List all installed skills from registry directory
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let registry_dir = home.join(".skill-engine").join("registry");

    let mut skills = Vec::new();

    if registry_dir.exists() {
        for entry in std::fs::read_dir(&registry_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(skill_name) = entry.file_name().to_str() {
                    skills.push(skill_name.to_string());
                }
            }
        }
    }

    skills.sort();
    Ok(skills)
}

/// Load tools for a skill
#[cfg(feature = "ai-ingestion")]
fn load_skill_tools(skill_name: &str) -> Result<Vec<ToolDocumentation>> {
    use skill_runtime::parse_skill_md;

    // Get skill directory from registry
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let skill_dir = home.join(".skill-engine").join("registry").join(skill_name);

    if !skill_dir.exists() {
        anyhow::bail!("Skill '{}' not found in registry", skill_name);
    }

    // Look for SKILL.md
    let skill_md_path = skill_dir.join("SKILL.md");

    if !skill_md_path.exists() {
        // No SKILL.md, return empty
        return Ok(Vec::new());
    }

    // Parse SKILL.md
    let skill_md = parse_skill_md(&skill_md_path)
        .context(format!("Failed to parse SKILL.md for '{}'", skill_name))?;

    // Extract tools
    let tools: Vec<ToolDocumentation> = skill_md
        .tool_docs
        .into_values()
        .collect();

    Ok(tools)
}

/// Build embedding text for a tool
#[cfg(feature = "ai-ingestion")]
fn build_embedding_text(tool: &ToolDocumentation) -> String {
    let mut parts = vec![
        format!("Tool: {}", tool.name),
        format!("Description: {}", tool.description),
    ];

    // Add parameters
    if !tool.parameters.is_empty() {
        let params: Vec<String> = tool
            .parameters
            .iter()
            .map(|p| format!("{} ({})", p.name, p.param_type))
            .collect();
        parts.push(format!("Parameters: {}", params.join(", ")));
    }

    // Add usage
    if let Some(usage) = &tool.usage {
        parts.push(format!("Usage: {}", usage));
    }

    // Add examples
    for example in &tool.examples {
        parts.push(format!("Example: {}", example.code));
    }

    parts.join("\n")
}

/// Truncate command for display
fn truncate_command(cmd: &str, max_len: usize) -> String {
    if cmd.len() <= max_len {
        cmd.to_string()
    } else {
        format!("{}...", &cmd[..max_len - 3])
    }
}

/// Get config file path
#[cfg(feature = "ai-ingestion")]
fn get_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    Ok(home.join(".skill-engine").join("search.toml"))
}

/// Load search config from file or return default
#[cfg(feature = "ai-ingestion")]
fn load_config() -> Result<SearchConfig> {
    let config_path = get_config_path()?;
    if config_path.exists() {
        SearchConfig::from_toml_file(&config_path)
    } else {
        Ok(SearchConfig::default())
    }
}
