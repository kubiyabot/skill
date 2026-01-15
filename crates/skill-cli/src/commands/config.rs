use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Input, Password, Select};
use skill_runtime::InstanceManager;
use skill_runtime::instance::ConfigValue;

use crate::ConfigAction;

pub async fn execute(
    skill: &str,
    instance: Option<&str>,
    action: Option<ConfigAction>,
) -> Result<()> {
    let instance_name = instance.unwrap_or("default");
    let instance_manager = InstanceManager::new()?;

    match action {
        Some(ConfigAction::Show) => show_config(skill, instance_name, &instance_manager).await,
        Some(ConfigAction::Set { pairs }) => {
            set_config(skill, instance_name, &instance_manager, pairs).await
        }
        Some(ConfigAction::Get { key }) => {
            get_config(skill, instance_name, &instance_manager, &key).await
        }
        None => interactive_config(skill, instance_name, &instance_manager).await,
    }
}

async fn show_config(
    skill: &str,
    instance: &str,
    manager: &InstanceManager,
) -> Result<()> {
    let config = manager
        .load_instance(skill, instance)
        .with_context(|| format!("Instance '{}' not found", instance))?;

    println!();
    println!(
        "{} Configuration for {}@{}",
        "→".cyan(),
        skill.yellow(),
        instance.cyan()
    );
    println!();

    println!("{}", "Metadata".bold().underline());
    println!("  {} {}", "Skill:".bold(), config.metadata.skill_name);
    println!("  {} {}", "Version:".bold(), config.metadata.skill_version);
    println!("  {} {}", "Instance:".bold(), config.metadata.instance_name);
    println!(
        "  {} {}",
        "Created:".bold(),
        config.metadata.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!();

    if !config.config.is_empty() {
        println!("{}", "Configuration".bold().underline());
        for (key, value) in &config.config {
            if value.secret {
                println!("  {} {} {}", key.bold(), "=".dimmed(), "[REDACTED]".red());
            } else {
                println!("  {} {} {}", key.bold(), "=".dimmed(), value.value.green());
            }
        }
        println!();
    }

    if !config.environment.is_empty() {
        println!("{}", "Environment Variables".bold().underline());
        for (key, value) in &config.environment {
            println!("  {} {} {}", key.bold(), "=".dimmed(), value.green());
        }
        println!();
    }

    println!("{}", "Capabilities".bold().underline());
    println!(
        "  {} {}",
        "Network Access:".bold(),
        if config.capabilities.network_access {
            "Enabled".green()
        } else {
            "Disabled".red()
        }
    );
    println!(
        "  {} {}",
        "Max Concurrent:".bold(),
        config.capabilities.max_concurrent_requests
    );
    println!();

    Ok(())
}

async fn set_config(
    skill: &str,
    instance: &str,
    manager: &InstanceManager,
    pairs: Vec<(String, String)>,
) -> Result<()> {
    let mut config = manager
        .load_instance(skill, instance)
        .with_context(|| format!("Instance '{}' not found", instance))?;

    for (key, value) in pairs {
        // Determine if this should be a secret based on key name
        let is_secret = key.to_lowercase().contains("secret")
            || key.to_lowercase().contains("password")
            || key.to_lowercase().contains("token")
            || key.to_lowercase().contains("key");

        if is_secret {
            // Store in keyring
            let keyring_ref =
                format!("keyring://skill-engine/{}/{}/{}", skill, instance, key);
            config.config.insert(
                key.clone(),
                ConfigValue {
                    value: keyring_ref,
                    secret: true,
                },
            );

            // Store actual value in keyring
            manager.update_secret(skill, instance, &key, &value)?;

            println!(
                "{} Set secret {} = [REDACTED]",
                "✓".green(),
                key.bold()
            );
        } else {
            // Store in config file
            config.set_config(key.clone(), value.clone(), false);
            println!("{} Set {} = {}", "✓".green(), key.bold(), value);
        }
    }

    manager.save_instance(skill, instance, &config)?;

    println!();
    println!("{} Configuration updated", "✓".green().bold());
    Ok(())
}

async fn get_config(
    skill: &str,
    instance: &str,
    manager: &InstanceManager,
    key: &str,
) -> Result<()> {
    let config = manager
        .load_instance(skill, instance)
        .with_context(|| format!("Instance '{}' not found", instance))?;

    if let Some(value) = config.config.get(key) {
        if value.secret {
            println!("{} = [REDACTED]", key.bold());
        } else {
            println!("{} = {}", key.bold(), value.value);
        }
    } else {
        anyhow::bail!("Configuration key '{}' not found", key);
    }

    Ok(())
}

async fn interactive_config(
    skill: &str,
    instance: &str,
    manager: &InstanceManager,
) -> Result<()> {
    println!();
    println!(
        "{} Interactive configuration for {}@{}",
        "→".cyan(),
        skill.yellow(),
        instance.cyan()
    );
    println!();

    let config_exists = manager.load_instance(skill, instance).is_ok();

    if config_exists {
        let choices = vec!["Add/Update value", "View configuration", "Exit"];
        let selection = Select::new()
            .with_prompt("What would you like to do?")
            .items(&choices)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                // Add/Update value
                let key: String = Input::new()
                    .with_prompt("Configuration key")
                    .interact_text()?;

                let is_secret_prompt = Select::new()
                    .with_prompt("Is this a secret value?")
                    .items(&["No", "Yes"])
                    .default(if key.to_lowercase().contains("secret")
                        || key.to_lowercase().contains("password")
                        || key.to_lowercase().contains("token")
                    {
                        1
                    } else {
                        0
                    })
                    .interact()?;

                let is_secret = is_secret_prompt == 1;

                let value = if is_secret {
                    Password::new()
                        .with_prompt("Secret value")
                        .interact()?
                } else {
                    Input::new()
                        .with_prompt("Configuration value")
                        .interact_text()?
                };

                set_config(skill, instance, manager, vec![(key, value)]).await?;
            }
            1 => {
                // View configuration
                show_config(skill, instance, manager).await?;
            }
            _ => {
                // Exit
                println!("Exiting...");
            }
        }
    } else {
        println!(
            "{} Instance '{}' does not exist yet. It will be created during installation.",
            "!".yellow(),
            instance
        );
    }

    Ok(())
}
