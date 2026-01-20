use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use skill_runtime::{
    is_git_url, parse_git_url, GitSkillLoader, InstanceConfig, InstanceManager, SkillEngine,
};
use std::path::PathBuf;
use std::time::Instant;

pub async fn execute(source: &str, instance: Option<&str>, force: bool, enhance: bool) -> Result<()> {
    println!("{} Installing skill from: {}", "→".cyan(), source.yellow());

    let start = Instant::now();

    // Determine source type and get WASM path + skill name
    let (wasm_path, skill_name, version) = if is_git_url(source) {
        install_from_git(source, force).await?
    } else {
        install_from_local(source)?
    };

    // Create progress spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    // Step 1: Load component and validate
    pb.set_message("Loading WASM component...");
    let engine = SkillEngine::new().context("Failed to create skill engine")?;

    let component = engine
        .load_component(&wasm_path)
        .await
        .context("Failed to load WASM component")?;

    pb.set_message("Validating component...");
    engine
        .validate_component(&component)
        .await
        .context("Component validation failed")?;

    let instance_name = instance.unwrap_or("default");

    pb.set_message(format!("Creating instance '{}'...", instance_name));

    // Step 2: Create registry directory and copy binary
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let registry_dir = home.join(".skill-engine").join("registry").join(&skill_name);
    std::fs::create_dir_all(&registry_dir)
        .with_context(|| format!("Failed to create registry directory: {}", registry_dir.display()))?;

    let dest_path = registry_dir.join(format!("{}.wasm", skill_name));
    std::fs::copy(&wasm_path, &dest_path)
        .with_context(|| format!("Failed to copy skill binary to: {}", dest_path.display()))?;

    // Step 3: Create default instance
    let instance_manager = InstanceManager::new()?;

    let mut config = InstanceConfig::default();
    config.metadata.skill_name = skill_name.clone();
    config.metadata.skill_version = version.unwrap_or_else(|| "0.1.0".to_string());
    config.metadata.instance_name = instance_name.to_string();
    config.metadata.created_at = chrono::Utc::now();
    config.metadata.updated_at = chrono::Utc::now();

    // Create instance with empty secrets (will be configured later)
    instance_manager
        .create_instance(
            &skill_name,
            instance_name,
            config,
            std::collections::HashMap::new(),
        )
        .context("Failed to create instance")?;

    // Step 4: Pre-compile component for fast execution
    pb.set_message("Pre-compiling for fast execution...");
    let cache_dir = home.join(".skill-engine").join("cache");
    std::fs::create_dir_all(&cache_dir)?;

    pb.finish_and_clear();

    let duration = start.elapsed();
    println!();
    println!("{} Skill installed successfully", "✓".green().bold());
    println!();
    println!("  {} {}", "Skill:".bold(), skill_name.cyan());
    println!("  {} {}", "Instance:".bold(), instance_name.yellow());
    println!("  {} {}", "Location:".bold(), dest_path.display());
    println!(
        "  {} {:.2}s",
        "Duration:".bold(),
        duration.as_secs_f64()
    );

    // Enhancement step (if requested)
    if enhance {
        println!();
        enhance_skill(&skill_name, &registry_dir).await?;
    }

    println!();
    println!("{} Next steps:", "→".cyan());
    println!("  • Configure: {} config {} -i {}", "skill".cyan(), skill_name, instance_name);
    println!("  • Run tool:  {} run {}", "skill".cyan(), skill_name);
    if !enhance {
        println!("  • Enhance:   {} enhance {}", "skill".cyan(), skill_name);
    }
    println!();

    Ok(())
}

/// Enhance skill with AI-generated examples
async fn enhance_skill(_skill_name: &str, skill_dir: &PathBuf) -> Result<()> {
    #[allow(unused_imports)]
    use skill_runtime::parse_skill_md;

    println!("{}", "🤖 Enhancing with AI examples...".bold());

    // Check for SKILL.md
    let skill_md_path = skill_dir.join("SKILL.md");
    if !skill_md_path.exists() {
        println!(
            "  {} No SKILL.md found - skipping enhancement",
            "⚠".yellow()
        );
        println!(
            "  {} Create a SKILL.md to enable AI enhancement",
            "→".dimmed()
        );
        return Ok(());
    }

    // Try to load config and check if AI ingestion is enabled
    #[cfg(feature = "ai-ingestion")]
    {
        use skill_runtime::{SearchConfig, SearchPipeline, GenerationEvent, IndexDocument, DocumentMetadata};
        use tokio_stream::StreamExt;

        let config = load_search_config()?;

        if !config.ai_ingestion.enabled {
            println!(
                "  {} AI ingestion not configured",
                "⚠".yellow()
            );
            println!(
                "  {} Run: {} to enable",
                "→".dimmed(),
                "skill setup".cyan()
            );
            return Ok(());
        }

        // Parse SKILL.md
        let skill_md = parse_skill_md(&skill_md_path)
            .context("Failed to parse SKILL.md")?;

        let tools: Vec<_> = skill_md.tool_docs.into_values().collect();

        if tools.is_empty() {
            println!(
                "  {} No tools found in SKILL.md",
                "⚠".yellow()
            );
            return Ok(());
        }

        println!(
            "  {} Found {} tools to enhance",
            "→".dimmed(),
            tools.len()
        );

        // Create pipeline
        let pipeline = SearchPipeline::from_config(config)
            .await
            .context("Failed to create search pipeline")?;

        if !pipeline.has_example_generator() {
            println!(
                "  {} LLM provider not available",
                "⚠".yellow()
            );
            return Ok(());
        }

        // Build documents
        let documents: Vec<IndexDocument> = tools
            .iter()
            .map(|t| IndexDocument {
                id: format!("{}:{}", skill_name, t.name),
                content: build_tool_content(t),
                metadata: DocumentMetadata {
                    skill_name: Some(skill_name.to_string()),
                    tool_name: Some(t.name.clone()),
                    ..Default::default()
                },
            })
            .collect();

        // Stream generation
        let mut stream = Box::pin(pipeline.index_documents_stream(documents, tools.clone()));
        let mut total_examples = 0;
        let mut total_valid = 0;

        while let Some(event) = stream.next().await {
            match event {
                GenerationEvent::Started { tool_name, .. } => {
                    print!("  {} {}...", "→".dimmed(), tool_name);
                }
                GenerationEvent::Example { example } => {
                    total_examples += 1;
                    if example.validated {
                        total_valid += 1;
                    }
                }
                GenerationEvent::ToolCompleted { examples_generated, valid_examples, .. } => {
                    println!(
                        " {} ({}/{})",
                        "✓".green(),
                        valid_examples,
                        examples_generated
                    );
                }
                GenerationEvent::Error { message, .. } => {
                    println!(" {} {}", "✗".red(), message);
                }
                GenerationEvent::Completed { .. } => {
                    println!(
                        "  {} Generated {} examples ({} valid)",
                        "✓".green().bold(),
                        total_examples,
                        total_valid
                    );
                }
                _ => {}
            }
        }
    }

    #[cfg(not(feature = "ai-ingestion"))]
    {
        let _ = skill_dir; // Suppress warning
        println!(
            "  {} AI ingestion feature not enabled",
            "⚠".yellow()
        );
        println!(
            "  {} Rebuild with: {}",
            "→".dimmed(),
            "cargo build --features ai-ingestion".cyan()
        );
    }

    Ok(())
}

/// Build content string for a tool
#[cfg(feature = "ai-ingestion")]
fn build_tool_content(tool: &skill_runtime::ToolDocumentation) -> String {
    let mut parts = vec![
        format!("Tool: {}", tool.name),
        format!("Description: {}", tool.description),
    ];

    if !tool.parameters.is_empty() {
        let params: Vec<String> = tool
            .parameters
            .iter()
            .map(|p| format!("{} ({})", p.name, p.param_type))
            .collect();
        parts.push(format!("Parameters: {}", params.join(", ")));
    }

    if let Some(usage) = &tool.usage {
        parts.push(format!("Usage: {}", usage));
    }

    parts.join("\n")
}

/// Load search config
#[cfg(feature = "ai-ingestion")]
fn load_search_config() -> Result<skill_runtime::SearchConfig> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let config_path = home.join(".skill-engine").join("search.toml");

    if config_path.exists() {
        skill_runtime::SearchConfig::from_toml_file(&config_path)
    } else {
        Ok(skill_runtime::SearchConfig::default())
    }
}

/// Install skill from a Git URL
async fn install_from_git(source: &str, force: bool) -> Result<(PathBuf, String, Option<String>)> {
    let git_source = parse_git_url(source)?;

    println!(
        "{} Detected Git source: {}",
        "→".dimmed(),
        git_source.display_name().cyan()
    );

    let loader = GitSkillLoader::new()?;

    // Clone/update repository
    println!("{} Cloning repository...", "→".dimmed());
    let cloned = loader.clone_skill(&git_source, force).await?;

    println!(
        "{} Detected skill type: {}",
        "→".dimmed(),
        format!("{}", cloned.skill_type).yellow()
    );

    // Build if needed
    println!("{} Building skill...", "→".dimmed());
    let wasm_path = loader.build_skill(&cloned).await?;

    println!(
        "{} Build complete: {}",
        "✓".green(),
        wasm_path.display()
    );

    Ok((wasm_path, cloned.skill_name, cloned.version))
}

/// Install skill from a local file
fn install_from_local(source: &str) -> Result<(PathBuf, String, Option<String>)> {
    let source_path = PathBuf::from(source);

    if !source_path.exists() {
        anyhow::bail!(
            "Skill file not found: {}\n\
             \n\
             Did you mean to install from Git? Try:\n\
             skill install github:user/repo",
            source
        );
    }

    if !source_path.extension().is_some_and(|ext| ext == "wasm") {
        anyhow::bail!(
            "Invalid file type. Expected .wasm file.\n\
             \n\
             For source directories, use:\n\
             skill install github:user/repo\n\
             skill run ./path/to/directory"
        );
    }

    let skill_name = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
        .to_string();

    Ok((source_path, skill_name, None))
}
