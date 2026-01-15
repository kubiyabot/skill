use anyhow::{Context, Result};
use colored::*;
use skill_runtime::{
    instance::ConfigValue, parse_git_url, GitSkillLoader, InstanceManager, SkillEngine,
    SkillExecutor, SkillManifest,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

/// Execute a skill with pass-through arguments.
///
/// This command is designed for skills that wrap CLI tools, allowing
/// you to pass arbitrary arguments directly to the skill's entry point.
///
/// The skill should implement an "exec" or "cli" tool that accepts
/// a single "args" parameter containing all pass-through arguments.
pub async fn execute(
    skill_spec: &str,
    config_overrides: &[(String, String)],
    args: &[String],
    manifest: Option<&SkillManifest>,
) -> Result<()> {
    let start = Instant::now();

    // Check if skill_spec is a local path
    let is_local_path = skill_spec.starts_with("./")
        || skill_spec.starts_with("../")
        || skill_spec.starts_with('/')
        || skill_spec.starts_with('~');

    if is_local_path {
        return execute_local_skill(skill_spec, config_overrides, args, start).await;
    }

    // Parse skill[@instance] (no tool - exec uses special "exec" tool)
    let (skill_name, instance_name) = parse_skill_spec(skill_spec)?;

    // Check if skill is defined in manifest
    if let Some(manifest) = manifest {
        if manifest.get_skill(&skill_name).is_some() {
            return execute_manifest_skill(
                manifest,
                &skill_name,
                Some(&instance_name),
                config_overrides,
                args,
                start,
            )
            .await;
        }
    }

    println!(
        "{} Executing {} (pass-through mode)",
        "→".cyan(),
        format!("{}@{}", skill_name, instance_name).yellow(),
    );

    if !args.is_empty() {
        println!(
            "{} Args: {}",
            "→".dimmed(),
            args.join(" ").dimmed()
        );
    }
    println!();

    // Load skill from registry
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let skill_path = home
        .join(".skill-engine")
        .join("registry")
        .join(&skill_name)
        .join(format!("{}.wasm", skill_name));

    if !skill_path.exists() {
        anyhow::bail!(
            "Skill '{}' not found. Install it with: skill install <path>",
            skill_name
        );
    }

    // Load instance configuration
    let instance_manager = InstanceManager::new()?;
    let mut instance_config = instance_manager
        .load_instance(&skill_name, &instance_name)
        .with_context(|| {
            format!(
                "Instance '{}' not found for skill '{}'. Create it with: skill config {} -i {}",
                instance_name, skill_name, skill_name, instance_name
            )
        })?;

    // Apply config overrides
    if !config_overrides.is_empty() {
        println!(
            "{} Applying {} config override(s)",
            "→".dimmed(),
            config_overrides.len()
        );
        for (key, value) in config_overrides {
            instance_config.config.insert(
                key.clone(),
                ConfigValue {
                    value: value.clone(),
                    secret: false,
                },
            );
        }
    }

    // Create skill engine and executor
    let engine = Arc::new(SkillEngine::new().context("Failed to create skill engine")?);

    let executor = SkillExecutor::load(
        engine.clone(),
        &skill_path,
        skill_name.clone(),
        instance_name.clone(),
        instance_config,
    )
    .await
    .context("Failed to load skill")?;

    // Get available tools to find exec/cli tool
    let tools = executor.get_tools().await?;

    // Look for exec, cli, or main tool (common names for pass-through execution)
    let exec_tool = tools
        .iter()
        .find(|t| t.name == "exec" || t.name == "cli" || t.name == "main" || t.name == "run")
        .map(|t| t.name.clone());

    let tool_name = exec_tool.ok_or_else(|| {
        let available = tools.iter().map(|t| t.name.as_str()).collect::<Vec<_>>().join(", ");
        anyhow::anyhow!(
            "Skill '{}' does not have an 'exec', 'cli', 'main', or 'run' tool for pass-through execution.\n\
            Available tools: {}\n\n\
            Hint: Use 'skill run {}:<tool-name>' to run a specific tool instead.",
            skill_name, available, skill_name
        )
    })?;

    println!("{} Using tool: {}", "→".dimmed(), tool_name.cyan());

    // Build arguments: pass all args as a single "args" parameter
    // Also pass individual args with numeric keys for flexibility
    let mut parsed_args: Vec<(String, String)> = vec![
        ("args".to_string(), args.join(" ")),
        ("argv".to_string(), serde_json::to_string(args).unwrap_or_default()),
    ];

    // Also add individual arguments with positional keys
    for (i, arg) in args.iter().enumerate() {
        parsed_args.push((format!("arg{}", i), arg.clone()));
    }

    // Execute tool
    println!("{} Executing...", "→".dimmed());
    println!();

    let result = match executor.execute_tool(&tool_name, parsed_args).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("\n{} Execution error:", "✗".red().bold());
            eprintln!("{:#}", e);
            anyhow::bail!("Exec failed");
        }
    };

    let duration = start.elapsed();

    println!();
    if result.success {
        // For exec, just print output without decoration
        if !result.output.is_empty() {
            println!("{}", result.output);
        }
        println!();
        println!(
            "{} Completed in {:.2}s",
            "✓".green().bold(),
            duration.as_secs_f64()
        );
    } else {
        println!("{} Exec failed", "✗".red().bold());
        if let Some(error) = result.error_message {
            eprintln!("{}", error);
        }
        std::process::exit(1);
    }

    Ok(())
}

/// Execute a skill from manifest definition (pass-through mode)
async fn execute_manifest_skill(
    manifest: &SkillManifest,
    skill_name: &str,
    instance_name: Option<&str>,
    config_overrides: &[(String, String)],
    args: &[String],
    start: Instant,
) -> Result<()> {
    // Resolve instance from manifest
    let resolved = manifest
        .resolve_instance(skill_name, instance_name)
        .context("Failed to resolve skill from manifest")?;

    println!(
        "{} Executing {} (from manifest, pass-through mode)",
        "→".cyan(),
        format!(
            "{}@{}",
            resolved.skill_name.yellow(),
            resolved.instance_name.cyan()
        ),
    );
    println!(
        "{} Source: {}",
        "→".dimmed(),
        resolved.source.dimmed()
    );

    // Apply config overrides
    let mut instance_config = resolved.config;
    if !config_overrides.is_empty() {
        for (key, value) in config_overrides {
            instance_config.config.insert(
                key.clone(),
                ConfigValue {
                    value: value.clone(),
                    secret: false,
                },
            );
        }
    }
    println!();

    // Determine source type and execute
    let engine = Arc::new(SkillEngine::new().context("Failed to create skill engine")?);

    let executor = if resolved.source.starts_with("./")
        || resolved.source.starts_with("../")
        || resolved.source.starts_with('/')
    {
        // Local path source
        let local_path = PathBuf::from(&resolved.source);
        SkillExecutor::load(
            engine.clone(),
            &local_path,
            resolved.skill_name.clone(),
            resolved.instance_name.clone(),
            instance_config,
        )
        .await
        .context("Failed to load local skill from manifest")?
    } else if is_git_url_spec(&resolved.source) {
        // Git source
        let loader = GitSkillLoader::new()?;
        let git_source = parse_git_url(&resolved.source)?;

        println!("{} Fetching from Git...", "→".dimmed());
        let cloned = loader.clone_skill(&git_source, false).await?;

        println!("{} Building...", "→".dimmed());
        let wasm_path = loader.build_skill(&cloned).await?;

        SkillExecutor::load(
            engine.clone(),
            &wasm_path,
            resolved.skill_name.clone(),
            resolved.instance_name.clone(),
            instance_config,
        )
        .await
        .context("Failed to load git skill from manifest")?
    } else {
        // Installed skill
        let home = dirs::home_dir().context("Failed to get home directory")?;
        let skill_path = home
            .join(".skill-engine")
            .join("registry")
            .join(&resolved.skill_name)
            .join(format!("{}.wasm", resolved.skill_name));

        if !skill_path.exists() {
            anyhow::bail!(
                "Skill '{}' from manifest not found in registry",
                resolved.skill_name
            );
        }

        SkillExecutor::load(
            engine.clone(),
            &skill_path,
            resolved.skill_name.clone(),
            resolved.instance_name.clone(),
            instance_config,
        )
        .await
        .context("Failed to load installed skill from manifest")?
    };

    // Get available tools to find exec/cli tool
    let tools = executor.get_tools().await?;
    let exec_tool = tools
        .iter()
        .find(|t| t.name == "exec" || t.name == "cli" || t.name == "main" || t.name == "run")
        .map(|t| t.name.clone());

    let tool_name = exec_tool.ok_or_else(|| {
        anyhow::anyhow!(
            "Skill '{}' does not have an 'exec', 'cli', 'main', or 'run' tool",
            skill_name
        )
    })?;

    // Build arguments
    let mut parsed_args: Vec<(String, String)> = vec![
        ("args".to_string(), args.join(" ")),
        ("argv".to_string(), serde_json::to_string(args).unwrap_or_default()),
    ];
    for (i, arg) in args.iter().enumerate() {
        parsed_args.push((format!("arg{}", i), arg.clone()));
    }

    // Execute
    println!("{} Executing (tool: {})...", "→".dimmed(), tool_name.cyan());
    println!();

    let result = match executor.execute_tool(&tool_name, parsed_args).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("\n{} Execution error:", "✗".red().bold());
            eprintln!("{:#}", e);
            anyhow::bail!("Exec failed");
        }
    };

    let duration = start.elapsed();

    if result.success {
        if !result.output.is_empty() {
            println!("{}", result.output);
        }
        println!(
            "\n{} Completed in {:.2}s",
            "✓".green().bold(),
            duration.as_secs_f64()
        );
    } else {
        println!("{} Exec failed", "✗".red().bold());
        if let Some(error) = result.error_message {
            eprintln!("{}", error);
        }
        std::process::exit(1);
    }

    Ok(())
}

/// Check if a spec looks like a Git URL
fn is_git_url_spec(spec: &str) -> bool {
    spec.starts_with("github:")
        || spec.starts_with("gitlab:")
        || spec.starts_with("bitbucket:")
        || spec.starts_with("git@")
        || spec.starts_with("https://github.com")
        || spec.starts_with("https://gitlab.com")
}

/// Execute a local skill in pass-through mode
async fn execute_local_skill(
    path: &str,
    config_overrides: &[(String, String)],
    args: &[String],
    start: Instant,
) -> Result<()> {
    // Expand ~ to home directory
    let expanded_path = if path.starts_with('~') {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        home.join(&path[2..])
    } else {
        PathBuf::from(path)
    };

    println!(
        "{} Executing local skill {} (pass-through mode)",
        "→".cyan(),
        expanded_path.display().to_string().yellow(),
    );
    println!();

    // Create engine
    let engine = Arc::new(SkillEngine::new().context("Failed to create skill engine")?);

    // Create a temporary instance config
    let mut instance_config = skill_runtime::InstanceConfig::default();
    instance_config.metadata.skill_name = "local-skill".to_string();
    instance_config.metadata.instance_name = "default".to_string();

    // Load config from skill directory if it exists
    if expanded_path.is_dir() {
        let config_file = expanded_path.join("skill.config.toml");
        if config_file.exists() {
            if let Ok(config) = skill_runtime::InstanceConfig::load(&config_file) {
                instance_config = config;
            }
        }
    }

    // Apply config overrides
    if !config_overrides.is_empty() {
        for (key, value) in config_overrides {
            instance_config.config.insert(
                key.clone(),
                ConfigValue {
                    value: value.clone(),
                    secret: false,
                },
            );
        }
    }

    // Create executor
    let executor = SkillExecutor::load(
        engine.clone(),
        &expanded_path,
        "local-skill".to_string(),
        "default".to_string(),
        instance_config,
    )
    .await
    .context("Failed to load skill")?;

    // Get available tools
    let tools = executor.get_tools().await?;
    let exec_tool = tools
        .iter()
        .find(|t| t.name == "exec" || t.name == "cli" || t.name == "main" || t.name == "run")
        .map(|t| t.name.clone());

    let tool_name = exec_tool.ok_or_else(|| {
        anyhow::anyhow!(
            "Local skill does not have an 'exec', 'cli', 'main', or 'run' tool.\n\
            Use 'skill run <path> <tool-name>' instead."
        )
    })?;

    // Build arguments
    let mut parsed_args: Vec<(String, String)> = vec![
        ("args".to_string(), args.join(" ")),
        ("argv".to_string(), serde_json::to_string(args).unwrap_or_default()),
    ];
    for (i, arg) in args.iter().enumerate() {
        parsed_args.push((format!("arg{}", i), arg.clone()));
    }

    // Execute
    let result = match executor.execute_tool(&tool_name, parsed_args).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("\n{} Execution error:", "✗".red().bold());
            eprintln!("{:#}", e);
            anyhow::bail!("Exec failed");
        }
    };

    let duration = start.elapsed();

    if result.success {
        if !result.output.is_empty() {
            println!("{}", result.output);
        }
        println!(
            "\n{} Completed in {:.2}s",
            "✓".green().bold(),
            duration.as_secs_f64()
        );
    } else {
        println!("{} Exec failed", "✗".red().bold());
        if let Some(error) = result.error_message {
            eprintln!("{}", error);
        }
        std::process::exit(1);
    }

    Ok(())
}

/// Parse skill specification for exec:
/// - skill -> (skill, "default")
/// - skill@instance -> (skill, instance)
fn parse_skill_spec(skill_spec: &str) -> Result<(String, String)> {
    if let Some(at_pos) = skill_spec.find('@') {
        let skill_name = skill_spec[..at_pos].to_string();
        let instance_name = skill_spec[at_pos + 1..].to_string();
        Ok((skill_name, instance_name))
    } else {
        Ok((skill_spec.to_string(), "default".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skill_spec() {
        let (skill, instance) = parse_skill_spec("aws").unwrap();
        assert_eq!(skill, "aws");
        assert_eq!(instance, "default");

        let (skill, instance) = parse_skill_spec("aws@prod").unwrap();
        assert_eq!(skill, "aws");
        assert_eq!(instance, "prod");
    }
}
