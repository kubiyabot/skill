use anyhow::{Context, Result};
use colored::*;
use skill_runtime::{
    instance::ConfigValue, parse_git_url, DockerRuntime, GitSkillLoader, InstanceManager,
    LocalSkillLoader, SkillEngine, SkillExecutor, SkillManifest, SkillRuntime,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

/// Parse CLI arguments supporting multiple formats:
/// - `key=value` (original format)
/// - `--key value` (standard CLI style)
/// - `--key=value` (combined style)
/// - `--flag` (boolean flags without values)
/// - `-k value` (short flags)
/// - `-k` (short boolean flags)
fn parse_cli_args(args: &[String]) -> Vec<(String, String)> {
    let mut parsed = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg.starts_with("--") {
            let key = arg.trim_start_matches('-');

            // Check for --key=value format
            if let Some(pos) = key.find('=') {
                parsed.push((key[..pos].to_string(), key[pos + 1..].to_string()));
            }
            // Check if next arg is the value (not another flag)
            else if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                parsed.push((key.to_string(), args[i + 1].clone()));
                i += 1; // Skip the value
            }
            // Boolean flag
            else {
                parsed.push((key.to_string(), "true".to_string()));
            }
        } else if arg.starts_with('-') && arg.len() == 2 {
            // Short flag -k value or -k
            let key = &arg[1..];
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                parsed.push((key.to_string(), args[i + 1].clone()));
                i += 1;
            } else {
                parsed.push((key.to_string(), "true".to_string()));
            }
        } else if let Some(pos) = arg.find('=') {
            // key=value format (current behavior)
            parsed.push((arg[..pos].to_string(), arg[pos + 1..].to_string()));
        } else {
            // Positional argument
            parsed.push(("arg".to_string(), arg.clone()));
        }

        i += 1;
    }

    parsed
}

pub async fn execute(
    skill_spec: &str,
    tool: Option<&str>,
    config_overrides: &[(String, String)],
    args: &[String],
    manifest: Option<&SkillManifest>,
) -> Result<()> {
    let start = Instant::now();

    // Check if skill_spec is a local path (starts with ./ or / or ~)
    let is_local_path = skill_spec.starts_with("./")
        || skill_spec.starts_with("../")
        || skill_spec.starts_with('/')
        || skill_spec.starts_with('~');

    if is_local_path {
        // Local skill execution
        return execute_local_skill(skill_spec, tool, config_overrides, args, start).await;
    }

    // Check if skill_spec is a Git URL (ephemeral execution without install)
    // Supports: github:user/repo:tool, https://github.com/user/repo:tool
    if is_git_url_spec(skill_spec) {
        return execute_git_skill(skill_spec, tool, config_overrides, args, start).await;
    }

    // Parse skill[@instance]:tool or skill[@instance] tool
    let (skill_name, instance_name, tool_name) = parse_skill_spec(skill_spec, tool)?;

    // Check if skill is defined in manifest
    if let Some(manifest) = manifest {
        if manifest.get_skill(&skill_name).is_some() {
            return execute_manifest_skill(
                manifest,
                &skill_name,
                Some(&instance_name),
                &tool_name,
                config_overrides,
                args,
                start,
            )
            .await;
        }
    }

    println!(
        "{} Running {}@{} → {}",
        "→".cyan(),
        skill_name.yellow(),
        instance_name.cyan(),
        tool_name.green()
    );

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

    // Apply config overrides from command line
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

    // Parse arguments (supports key=value, --key value, --key=value, --flag, -k value, -k)
    let parsed_args = parse_cli_args(args);

    // Execute tool
    println!();
    let result = executor
        .execute_tool(&tool_name, parsed_args)
        .await
        .context("Tool execution failed")?;

    let duration = start.elapsed();

    println!();
    if result.success {
        println!("{}", "─".repeat(60).dimmed());
        println!("{}", result.output);
        println!("{}", "─".repeat(60).dimmed());
        println!();
        println!(
            "{} Tool executed successfully in {:.2}s",
            "✓".green().bold(),
            duration.as_secs_f64()
        );
    } else {
        println!("{} Tool execution failed", "✗".red().bold());
        if let Some(error) = result.error_message {
            println!();
            println!("{} {}", "Error:".red().bold(), error);
        }
        println!();
        std::process::exit(1);
    }

    Ok(())
}

/// Execute a skill from manifest definition
async fn execute_manifest_skill(
    manifest: &SkillManifest,
    skill_name: &str,
    instance_name: Option<&str>,
    tool_name: &str,
    config_overrides: &[(String, String)],
    args: &[String],
    start: Instant,
) -> Result<()> {
    // Resolve instance from manifest
    let resolved = manifest
        .resolve_instance(skill_name, instance_name)
        .context("Failed to resolve skill from manifest")?;

    // Display runtime type
    let runtime_str = match resolved.runtime {
        SkillRuntime::Wasm => "wasm",
        SkillRuntime::Docker => "docker",
        SkillRuntime::Native => "native",
    };

    println!(
        "{} Running {} (from manifest, runtime: {}) → {}",
        "→".cyan(),
        format!(
            "{}@{}",
            resolved.skill_name.yellow(),
            resolved.instance_name.cyan()
        ),
        runtime_str.magenta(),
        tool_name.green()
    );
    println!(
        "{} Source: {}",
        "→".dimmed(),
        resolved.source.dimmed()
    );

    // Handle Docker runtime separately (before moving config)
    if resolved.runtime == SkillRuntime::Docker {
        return execute_docker_skill(&resolved, tool_name, args, start).await;
    }

    // Handle Native runtime - execute CLI commands directly
    if resolved.runtime == SkillRuntime::Native {
        return execute_native_manifest_skill(&resolved, tool_name, args, start).await;
    }

    // Apply config overrides
    let mut instance_config = resolved.config;
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
    println!();

    // Determine source type and execute (for WASM and Native runtimes)
    let engine = Arc::new(SkillEngine::new().context("Failed to create skill engine")?);

    let executor = if resolved.source.starts_with("./")
        || resolved.source.starts_with("../")
        || resolved.source.starts_with('/')
    {
        // Local path source - use LocalSkillLoader to find wasm file
        let local_path = PathBuf::from(&resolved.source);
        let loader = LocalSkillLoader::new()?;

        // Load skill (will find wasm in directory or compile if needed)
        let _component = loader
            .load_skill(&local_path, &engine)
            .await
            .context("Failed to load local skill")?;

        // Find the actual wasm path for the executor
        let wasm_path = find_wasm_in_path(&local_path)?;

        SkillExecutor::load(
            engine.clone(),
            &wasm_path,
            resolved.skill_name.clone(),
            resolved.instance_name.clone(),
            instance_config,
        )
        .await
        .context("Failed to create executor for local skill")?
    } else if is_git_url_spec(&resolved.source) {
        // Git source - clone and build
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
        // Assume installed skill from registry
        let home = dirs::home_dir().context("Failed to get home directory")?;
        let skill_path = home
            .join(".skill-engine")
            .join("registry")
            .join(&resolved.skill_name)
            .join(format!("{}.wasm", resolved.skill_name));

        if !skill_path.exists() {
            anyhow::bail!(
                "Skill '{}' from manifest not found in registry. Install it first or provide a local/git source.",
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

    // Parse arguments (supports key=value, --key value, --key=value, --flag, -k value, -k)
    let parsed_args = parse_cli_args(args);

    // Execute tool
    let result = match executor.execute_tool(tool_name, parsed_args).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("\n{} Execution error:", "✗".red().bold());
            eprintln!("{:#}", e);
            anyhow::bail!("Tool execution failed");
        }
    };

    // Check if the result contains a command that should be executed natively
    let final_result = if result.success && result.output.starts_with("Command: ") {
        // Extract and execute the kubectl command natively
        execute_native_command(&result.output, start).await?
    } else {
        result
    };

    let duration = start.elapsed();

    println!();
    if final_result.success {
        println!("{}", "─".repeat(60).dimmed());
        println!("{}", final_result.output);
        println!("{}", "─".repeat(60).dimmed());
        println!();
        println!(
            "{} Tool executed successfully in {:.2}s",
            "✓".green().bold(),
            duration.as_secs_f64()
        );
    } else {
        println!("{} Tool execution failed", "✗".red().bold());
        if let Some(error) = final_result.error_message {
            println!();
            println!("{} {}", "Error:".red().bold(), error);
        }
        std::process::exit(1);
    }

    Ok(())
}

/// Execute a native command from skill output
async fn execute_native_command(
    output: &str,
    _start: Instant,
) -> Result<skill_runtime::ExecutionResult> {
    use std::process::Stdio;
    use tokio::process::Command;

    // Extract the command from "Command: kubectl ..."
    let first_line = output.lines().next().unwrap_or("");
    let command_str = first_line.strip_prefix("Command: ").unwrap_or(first_line);

    // Parse the command
    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(skill_runtime::ExecutionResult {
            success: false,
            output: String::new(),
            error_message: Some("Empty command".to_string()),
            metadata: None,
        });
    }

    let program = parts[0];
    let args = &parts[1..];

    // Security check: Only allow specific commands
    let allowed_commands = ["kubectl", "helm", "git", "curl", "jq", "aws", "gcloud", "az", "docker", "terraform"];
    if !allowed_commands.contains(&program) {
        return Ok(skill_runtime::ExecutionResult {
            success: false,
            output: String::new(),
            error_message: Some(format!(
                "Command '{}' not allowed. Allowed: {}",
                program,
                allowed_commands.join(", ")
            )),
            metadata: None,
        });
    }

    println!("{} Executing: {}", "→".cyan(), command_str.yellow());

    // Execute the command
    let result = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if output.status.success() {
                Ok(skill_runtime::ExecutionResult {
                    success: true,
                    output: stdout,
                    error_message: if stderr.is_empty() {
                        None
                    } else {
                        Some(stderr)
                    },
                    metadata: None,
                })
            } else {
                Ok(skill_runtime::ExecutionResult {
                    success: false,
                    output: stdout,
                    error_message: Some(if stderr.is_empty() {
                        format!("Command exited with status: {}", output.status)
                    } else {
                        stderr
                    }),
                    metadata: None,
                })
            }
        }
        Err(e) => Ok(skill_runtime::ExecutionResult {
            success: false,
            output: String::new(),
            error_message: Some(format!("Failed to execute command: {}", e)),
            metadata: None,
        }),
    }
}

/// Execute a Docker-based skill
async fn execute_docker_skill(
    resolved: &skill_runtime::ResolvedInstance,
    tool_name: &str,
    args: &[String],
    start: Instant,
) -> Result<()> {
    let docker_config = resolved
        .docker
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Docker runtime requires docker configuration in manifest"))?;

    println!(
        "{} Docker image: {}",
        "→".dimmed(),
        docker_config.image.cyan()
    );

    // Check Docker availability
    if !DockerRuntime::is_available() {
        anyhow::bail!(
            "Docker is not available. Please install Docker and ensure it's running.\n\
             Install: https://docs.docker.com/get-docker/"
        );
    }

    let runtime = DockerRuntime::new();

    // Ensure image exists (pull if needed)
    println!("{} Ensuring Docker image is available...", "→".dimmed());
    runtime
        .ensure_image(&docker_config.image)
        .context("Failed to ensure Docker image")?;

    // Build tool arguments
    // Format: tool_name followed by args in key=value format
    let mut tool_args = vec![tool_name.to_string()];
    tool_args.extend(args.iter().cloned());

    println!(
        "{} Executing in container: {} {}",
        "→".cyan(),
        tool_name.yellow(),
        args.join(" ").dimmed()
    );

    // Execute in Docker container
    let output = runtime
        .execute(docker_config, &tool_args)
        .context("Failed to execute Docker container")?;

    let duration = start.elapsed();

    println!();
    if output.success {
        println!("{}", "─".repeat(60).dimmed());
        println!("{}", output.stdout);
        if !output.stderr.is_empty() {
            eprintln!("{}", output.stderr.dimmed());
        }
        println!("{}", "─".repeat(60).dimmed());
        println!();
        println!(
            "{} Docker skill executed successfully in {:.2}s",
            "✓".green().bold(),
            duration.as_secs_f64()
        );
    } else {
        println!("{} Docker skill execution failed", "✗".red().bold());
        println!();
        if !output.stdout.is_empty() {
            println!("{}", output.stdout);
        }
        if !output.stderr.is_empty() {
            eprintln!("{} {}", "Error:".red().bold(), output.stderr);
        }
        println!();
        println!(
            "{} Exit code: {}",
            "→".dimmed(),
            output.exit_code.to_string().red()
        );
        std::process::exit(output.exit_code);
    }

    Ok(())
}

/// Execute a native skill (CLI commands like kubectl, docker, git, terraform)
async fn execute_native_manifest_skill(
    resolved: &skill_runtime::ResolvedInstance,
    tool_name: &str,
    args: &[String],
    start: Instant,
) -> Result<()> {
    use std::process::Stdio;
    use tokio::process::Command;

    let skill_name = &resolved.skill_name;

    // Parse arguments (supports key=value, --key value, --key=value, --flag, -k value, -k)
    let parsed_args = parse_cli_args(args);

    // Build the native command
    let command_str = build_native_command(skill_name, tool_name, &parsed_args)?;

    println!(
        "{} Executing: {}",
        "→".cyan(),
        command_str.yellow()
    );
    println!();

    // Parse the command
    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("Empty command generated");
    }

    let program = parts[0];
    let cmd_args = &parts[1..];

    // Security check: Only allow specific commands
    let allowed_commands = ["kubectl", "helm", "git", "curl", "jq", "aws", "gcloud", "az", "docker", "terraform", "psql"];
    if !allowed_commands.contains(&program) {
        anyhow::bail!(
            "Command '{}' not allowed. Allowed: {}",
            program,
            allowed_commands.join(", ")
        );
    }

    // Execute the command
    let result = Command::new(program)
        .args(cmd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    let duration = start.elapsed();

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            println!("{}", "─".repeat(60).dimmed());
            if !stdout.is_empty() {
                println!("{}", stdout);
            }
            if !stderr.is_empty() && output.status.success() {
                eprintln!("{}", stderr.dimmed());
            }
            println!("{}", "─".repeat(60).dimmed());
            println!();

            if output.status.success() {
                println!(
                    "{} Native skill executed successfully in {:.2}s",
                    "✓".green().bold(),
                    duration.as_secs_f64()
                );
            } else {
                println!("{} Native skill execution failed", "✗".red().bold());
                if !stderr.is_empty() {
                    eprintln!("{} {}", "Error:".red().bold(), stderr);
                }
                println!(
                    "{} Exit code: {}",
                    "→".dimmed(),
                    output.status.code().unwrap_or(-1).to_string().red()
                );
                std::process::exit(output.status.code().unwrap_or(1));
            }
        }
        Err(e) => {
            anyhow::bail!("Failed to execute command '{}': {}", program, e);
        }
    }

    Ok(())
}

/// Build a native command from skill name, tool name, and arguments
/// Uses generic passthrough: base_command + tool_name + args
/// Args convention:
///   - "arg" key or empty key: positional argument (value only)
///   - single char key: short flag (-k value)
///   - multi char key: long flag (--key value)
///   - value "true" with flag key: boolean flag (--key or -k without value)
fn build_native_command(
    skill_name: &str,
    tool_name: &str,
    args: &[(String, String)],
) -> Result<String> {
    // Map skill name to base CLI command
    let base_command = match skill_name {
        "kubernetes" => "kubectl",
        "aws" => "aws",
        "docker" => "docker",
        "terraform" => "terraform",
        "helm" => "helm",
        "git" => "git",
        "postgres-native" => "psql",
        _ => skill_name,
    };

    let mut cmd_parts = vec![base_command.to_string()];

    // Add tool name as subcommand
    cmd_parts.push(tool_name.to_string());

    // Process arguments generically
    for (key, value) in args {
        if key == "arg" || key == "resource" || key.is_empty() {
            // Positional argument - just add the value
            // Note: "resource" is special-cased for kubectl which expects resource type as positional
            cmd_parts.push(value.clone());
        } else if value == "true" {
            // Boolean flag
            if key.len() == 1 {
                cmd_parts.push(format!("-{}", key));
            } else {
                cmd_parts.push(format!("--{}", key));
            }
        } else if value == "false" {
            // Skip false boolean flags
            continue;
        } else if key.len() == 1 {
            // Short flag: -n value
            cmd_parts.push(format!("-{}", key));
            cmd_parts.push(value.clone());
        } else {
            // Long flag: --namespace value
            cmd_parts.push(format!("--{}", key));
            cmd_parts.push(value.clone());
        }
    }

    Ok(cmd_parts.join(" "))
}

/// Execute a skill from a local path (directory or file)
async fn execute_local_skill(
    path: &str,
    tool: Option<&str>,
    config_overrides: &[(String, String)],
    args: &[String],
    start: Instant,
) -> Result<()> {
    let tool_name = tool.ok_or_else(|| anyhow::anyhow!("Tool name required for local skills"))?;

    // Expand ~ to home directory
    let expanded_path = if path.starts_with('~') {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        home.join(&path[2..])
    } else {
        PathBuf::from(path)
    };

    println!(
        "{} Running local skill {} → {}",
        "→".cyan(),
        expanded_path.display().to_string().yellow(),
        tool_name.green()
    );
    println!();

    // Create engine and loader
    let engine = Arc::new(SkillEngine::new().context("Failed to create skill engine")?);
    let loader = LocalSkillLoader::new()?;

    // Load skill (will compile if needed)
    println!("{} Loading skill...", "→".dimmed());
    let _component = loader
        .load_skill(&expanded_path, &engine)
        .await
        .context("Failed to load local skill")?;

    // Create a temporary instance config for local execution
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

    // Apply config overrides from command line
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

    // Create executor
    let executor = SkillExecutor::load(
        engine.clone(),
        &expanded_path,
        "local-skill".to_string(),
        "default".to_string(),
        instance_config,
    )
    .await
    .context("Failed to create skill executor")?;

    // Parse arguments (supports key=value, --key value, --key=value, --flag, -k value, -k)
    let parsed_args = parse_cli_args(args);

    // Execute tool
    println!("{} Executing tool...", "→".dimmed());
    println!();
    let result = executor
        .execute_tool(tool_name, parsed_args)
        .await
        .map_err(|e| {
            eprintln!("Execution error details: {:?}", e);
            e
        })
        .context("Tool execution failed")?;

    let duration = start.elapsed();

    println!();
    if result.success {
        println!("{}", "─".repeat(60).dimmed());
        println!("{}", result.output);
        println!("{}", "─".repeat(60).dimmed());
        println!();
        println!(
            "{} Tool executed successfully in {:.2}s",
            "✓".green().bold(),
            duration.as_secs_f64()
        );
    } else {
        println!("{} Tool execution failed", "✗".red().bold());
        if let Some(error) = result.error_message {
            println!();
            println!("{} {}", "Error:".red().bold(), error);
        }
        println!();
        std::process::exit(1);
    }

    Ok(())
}

/// Parse skill specification:
/// - skill[@instance]:tool
/// - skill[@instance] tool
/// - skill tool (default instance)
fn parse_skill_spec(
    skill_spec: &str,
    tool: Option<&str>,
) -> Result<(String, String, String)> {
    // Check for skill[@instance]:tool format
    if let Some(colon_pos) = skill_spec.rfind(':') {
        let skill_part = &skill_spec[..colon_pos];
        let tool_name = skill_spec[colon_pos + 1..].to_string();

        if let Some(at_pos) = skill_part.find('@') {
            // skill@instance:tool
            let skill_name = skill_part[..at_pos].to_string();
            let instance_name = skill_part[at_pos + 1..].to_string();
            Ok((skill_name, instance_name, tool_name))
        } else {
            // skill:tool
            Ok((skill_part.to_string(), "default".to_string(), tool_name))
        }
    } else {
        // skill[@instance] tool format
        let tool_name = tool
            .ok_or_else(|| anyhow::anyhow!("Tool name required"))?
            .to_string();

        if let Some(at_pos) = skill_spec.find('@') {
            // skill@instance tool
            let skill_name = skill_spec[..at_pos].to_string();
            let instance_name = skill_spec[at_pos + 1..].to_string();
            Ok((skill_name, instance_name, tool_name))
        } else {
            // skill tool
            Ok((skill_spec.to_string(), "default".to_string(), tool_name))
        }
    }
}

/// Check if a skill spec looks like a Git URL (for ephemeral execution)
/// Handles: github:user/repo:tool, github:user/repo tool, https://github.com/user/repo:tool
fn is_git_url_spec(spec: &str) -> bool {
    // Check for Git URL prefixes
    spec.starts_with("github:")
        || spec.starts_with("gitlab:")
        || spec.starts_with("bitbucket:")
        || spec.starts_with("git@")
        || spec.starts_with("https://github.com")
        || spec.starts_with("https://gitlab.com")
}

/// Find the WASM file in a path (handles both files and directories)
fn find_wasm_in_path(path: &Path) -> Result<PathBuf> {
    // If it's a direct wasm file, return it
    if path.extension().map_or(false, |ext| ext == "wasm") && path.exists() {
        return Ok(path.to_path_buf());
    }

    // If it's a directory, search for wasm files
    if path.is_dir() {
        let candidates = vec![
            path.join("skill.wasm"),
            path.join("dist/skill.wasm"),
        ];

        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    anyhow::bail!("No WASM file found in: {}", path.display())
}

/// Execute a skill directly from a Git URL (ephemeral, no install required)
async fn execute_git_skill(
    git_spec: &str,
    tool: Option<&str>,
    config_overrides: &[(String, String)],
    args: &[String],
    start: Instant,
) -> Result<()> {
    // Parse: github:user/repo:tool_name or github:user/repo[@ref]:tool_name
    let (git_url, tool_name) = parse_git_tool_spec(git_spec, tool)?;

    let git_source = parse_git_url(&git_url)?;

    println!(
        "{} Running {} → {}",
        "→".cyan(),
        git_source.display_name().yellow(),
        tool_name.green()
    );
    println!();

    // Clone/update and build (uses cached clone if available)
    let loader = GitSkillLoader::new()?;

    println!("{} Fetching skill from Git...", "→".dimmed());
    let cloned = loader.clone_skill(&git_source, false).await?;

    println!(
        "{} Skill type: {}",
        "→".dimmed(),
        format!("{}", cloned.skill_type).cyan()
    );

    println!("{} Building...", "→".dimmed());
    let wasm_path = loader.build_skill(&cloned).await?;

    // Create engine and executor
    let engine = Arc::new(SkillEngine::new().context("Failed to create skill engine")?);

    let mut instance_config = skill_runtime::InstanceConfig::default();
    instance_config.metadata.skill_name = cloned.skill_name.clone();
    instance_config.metadata.instance_name = "ephemeral".to_string();

    // Apply config overrides from command line
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

    let executor = SkillExecutor::load(
        engine.clone(),
        &wasm_path,
        cloned.skill_name,
        "ephemeral".to_string(),
        instance_config,
    )
    .await
    .context("Failed to load skill")?;

    // Parse arguments (supports key=value, --key value, --key=value, --flag, -k value, -k)
    let parsed_args = parse_cli_args(args);

    // Execute tool
    println!("{} Executing...", "→".dimmed());
    println!();
    let result = match executor.execute_tool(&tool_name, parsed_args).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("\n{} Execution error:", "✗".red().bold());
            eprintln!("{:#}", e);
            anyhow::bail!("Tool execution failed");
        }
    };

    let duration = start.elapsed();

    println!();
    if result.success {
        println!("{}", "─".repeat(60).dimmed());
        println!("{}", result.output);
        println!("{}", "─".repeat(60).dimmed());
        println!();
        println!(
            "{} Tool executed successfully in {:.2}s",
            "✓".green().bold(),
            duration.as_secs_f64()
        );
    } else {
        println!("{} Tool execution failed", "✗".red().bold());
        if let Some(error) = result.error_message {
            println!();
            println!("{} {}", "Error:".red().bold(), error);
        }
        println!();
        std::process::exit(1);
    }

    Ok(())
}

/// Parse Git URL with tool name
/// Formats:
/// - github:user/repo:tool -> (github:user/repo, tool)
/// - github:user/repo@v1.0.0:tool -> (github:user/repo@v1.0.0, tool)
/// - github:user/repo tool -> (github:user/repo, tool)  [with separate tool arg]
fn parse_git_tool_spec(spec: &str, tool: Option<&str>) -> Result<(String, String)> {
    // Try to find tool name after the last colon (but not within the URL part)
    // github:user/repo:tool -> split at last colon after the repo part
    // We check for inline tool FIRST, even if a separate tool arg was provided

    // For shorthand format (github:user/repo:tool)
    if spec.starts_with("github:")
        || spec.starts_with("gitlab:")
        || spec.starts_with("bitbucket:")
    {
        // Find the prefix end
        let prefix_end = spec.find(':').unwrap() + 1;
        let rest = &spec[prefix_end..];

        // rest is now "user/repo[@ref]:tool" or "user/repo[@ref]"
        // Find tool after the last colon in rest
        if let Some(last_colon) = rest.rfind(':') {
            let repo_part = &rest[..last_colon];
            let tool_part = &rest[last_colon + 1..];

            // Make sure we're not splitting a ref like @v1:0:0
            if !tool_part.is_empty() && !tool_part.contains('/') {
                let prefix = &spec[..prefix_end];
                return Ok((format!("{}{}", prefix, repo_part), tool_part.to_string()));
            }
        }
    }

    // For HTTPS URLs
    if spec.starts_with("https://") {
        if let Some(last_colon) = spec.rfind(':') {
            // Make sure it's not the https:// colon
            if last_colon > 8 {
                let url_part = &spec[..last_colon];
                let tool_part = &spec[last_colon + 1..];
                if !tool_part.is_empty() && !tool_part.contains('/') {
                    return Ok((url_part.to_string(), tool_part.to_string()));
                }
            }
        }
    }

    // No inline tool found - check if tool is provided separately
    if let Some(t) = tool {
        return Ok((spec.to_string(), t.to_string()));
    }

    anyhow::bail!(
        "Tool name required for Git skills.\n\
         \n\
         Usage:\n\
         skill run github:user/repo:tool-name args...\n\
         skill run github:user/repo tool-name args..."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skill_spec() {
        // skill:tool
        let (skill, instance, tool) = parse_skill_spec("aws:s3-list", None).unwrap();
        assert_eq!(skill, "aws");
        assert_eq!(instance, "default");
        assert_eq!(tool, "s3-list");

        // skill@instance:tool
        let (skill, instance, tool) = parse_skill_spec("aws@prod:s3-list", None).unwrap();
        assert_eq!(skill, "aws");
        assert_eq!(instance, "prod");
        assert_eq!(tool, "s3-list");

        // skill tool
        let (skill, instance, tool) = parse_skill_spec("aws", Some("s3-list")).unwrap();
        assert_eq!(skill, "aws");
        assert_eq!(instance, "default");
        assert_eq!(tool, "s3-list");

        // skill@instance tool
        let (skill, instance, tool) = parse_skill_spec("aws@prod", Some("s3-list")).unwrap();
        assert_eq!(skill, "aws");
        assert_eq!(instance, "prod");
        assert_eq!(tool, "s3-list");
    }

    #[test]
    fn test_parse_git_tool_spec() {
        // github:user/repo:tool
        let (url, tool) = parse_git_tool_spec("github:user/repo:hello", None).unwrap();
        assert_eq!(url, "github:user/repo");
        assert_eq!(tool, "hello");

        // github:user/repo@v1.0.0:tool
        let (url, tool) = parse_git_tool_spec("github:user/repo@v1.0.0:hello", None).unwrap();
        assert_eq!(url, "github:user/repo@v1.0.0");
        assert_eq!(tool, "hello");

        // With separate tool argument
        let (url, tool) = parse_git_tool_spec("github:user/repo", Some("hello")).unwrap();
        assert_eq!(url, "github:user/repo");
        assert_eq!(tool, "hello");
    }

    #[test]
    fn test_is_git_url_spec() {
        assert!(is_git_url_spec("github:user/repo"));
        assert!(is_git_url_spec("github:user/repo:tool"));
        assert!(is_git_url_spec("https://github.com/user/repo"));
        assert!(!is_git_url_spec("aws:s3-list"));  // This is skill:tool, not Git
        assert!(!is_git_url_spec("my-skill"));
    }

    #[test]
    fn test_parse_cli_args_key_value_format() {
        // Original key=value format
        let args = vec!["resource=pods".to_string(), "namespace=kube-system".to_string()];
        let parsed = parse_cli_args(&args);
        assert_eq!(parsed, vec![
            ("resource".to_string(), "pods".to_string()),
            ("namespace".to_string(), "kube-system".to_string()),
        ]);
    }

    #[test]
    fn test_parse_cli_args_long_flag_value() {
        // --key value format
        let args = vec!["--resource".to_string(), "pods".to_string(), "--namespace".to_string(), "kube-system".to_string()];
        let parsed = parse_cli_args(&args);
        assert_eq!(parsed, vec![
            ("resource".to_string(), "pods".to_string()),
            ("namespace".to_string(), "kube-system".to_string()),
        ]);
    }

    #[test]
    fn test_parse_cli_args_long_flag_equals() {
        // --key=value format
        let args = vec!["--resource=pods".to_string(), "--namespace=kube-system".to_string()];
        let parsed = parse_cli_args(&args);
        assert_eq!(parsed, vec![
            ("resource".to_string(), "pods".to_string()),
            ("namespace".to_string(), "kube-system".to_string()),
        ]);
    }

    #[test]
    fn test_parse_cli_args_boolean_flags() {
        // --flag boolean flags
        let args = vec!["--all-namespaces".to_string(), "--wide".to_string()];
        let parsed = parse_cli_args(&args);
        assert_eq!(parsed, vec![
            ("all-namespaces".to_string(), "true".to_string()),
            ("wide".to_string(), "true".to_string()),
        ]);
    }

    #[test]
    fn test_parse_cli_args_short_flags() {
        // -k value and -k boolean flags
        let args = vec!["-n".to_string(), "kube-system".to_string(), "-A".to_string()];
        let parsed = parse_cli_args(&args);
        assert_eq!(parsed, vec![
            ("n".to_string(), "kube-system".to_string()),
            ("A".to_string(), "true".to_string()),
        ]);
    }

    #[test]
    fn test_parse_cli_args_positional() {
        // Positional arguments
        let args = vec!["pods".to_string(), "nginx".to_string()];
        let parsed = parse_cli_args(&args);
        assert_eq!(parsed, vec![
            ("arg".to_string(), "pods".to_string()),
            ("arg".to_string(), "nginx".to_string()),
        ]);
    }

    #[test]
    fn test_parse_cli_args_mixed() {
        // Mixed format: --resource pods --all-namespaces
        let args = vec![
            "--resource".to_string(),
            "pods".to_string(),
            "--all-namespaces".to_string(),
        ];
        let parsed = parse_cli_args(&args);
        assert_eq!(parsed, vec![
            ("resource".to_string(), "pods".to_string()),
            ("all-namespaces".to_string(), "true".to_string()),
        ]);
    }

    #[test]
    fn test_parse_cli_args_kubectl_get_example() {
        // Real kubectl example: skill run kubernetes get --resource pods --all-namespaces
        let args = vec![
            "--resource".to_string(),
            "pods".to_string(),
            "--all-namespaces".to_string(),
        ];
        let parsed = parse_cli_args(&args);

        // Should parse to: resource=pods, all-namespaces=true
        assert_eq!(parsed, vec![
            ("resource".to_string(), "pods".to_string()),
            ("all-namespaces".to_string(), "true".to_string()),
        ]);

        // When passed to build_native_command for kubernetes/get, should produce:
        // kubectl get pods --all-namespaces
        // Note: "resource" is special-cased as positional for kubectl compatibility
        let cmd = build_native_command("kubernetes", "get", &parsed).unwrap();
        assert_eq!(cmd, "kubectl get pods --all-namespaces");
    }

    #[test]
    fn test_build_native_command_with_positional_resource() {
        // Using positional argument for resource: skill run kubernetes get pods --all-namespaces
        let parsed = vec![
            ("arg".to_string(), "pods".to_string()),
            ("all-namespaces".to_string(), "true".to_string()),
        ];
        let cmd = build_native_command("kubernetes", "get", &parsed).unwrap();
        assert_eq!(cmd, "kubectl get pods --all-namespaces");
    }
}
