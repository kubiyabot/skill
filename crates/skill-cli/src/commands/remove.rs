use anyhow::{Context, Result};
use colored::*;
use dialoguer::Confirm;
use skill_runtime::InstanceManager;
use std::fs;

pub async fn execute(skill: &str, instance: Option<&str>, force: bool) -> Result<()> {
    let _home = dirs::home_dir().context("Failed to get home directory")?;
    let instance_manager = InstanceManager::new()?;

    match instance {
        Some(instance_name) => {
            // Remove specific instance
            remove_instance(skill, instance_name, force, &instance_manager).await
        }
        None => {
            // Remove entire skill
            remove_skill(skill, force, &instance_manager).await
        }
    }
}

async fn remove_instance(
    skill: &str,
    instance: &str,
    force: bool,
    manager: &InstanceManager,
) -> Result<()> {
    // Check if instance exists
    let config = manager
        .load_instance(skill, instance)
        .with_context(|| format!("Instance '{}' not found for skill '{}'", instance, skill))?;

    println!();
    println!(
        "{} Removing instance {}@{}",
        "→".yellow(),
        skill.cyan(),
        instance.yellow()
    );
    println!();
    println!("  {} {}", "Skill:".bold(), skill);
    println!("  {} {}", "Instance:".bold(), instance);
    println!("  {} {}", "Version:".bold(), config.metadata.skill_version);
    println!();

    // Confirm deletion
    if !force {
        let confirmed = Confirm::new()
            .with_prompt("Are you sure you want to remove this instance?")
            .default(false)
            .interact()?;

        if !confirmed {
            println!("{} Cancelled", "!".yellow());
            return Ok(());
        }
    }

    // Delete instance (this also cleans up keyring entries)
    manager
        .delete_instance(skill, instance)
        .context("Failed to delete instance")?;

    println!();
    println!("{} Instance removed successfully", "✓".green().bold());
    Ok(())
}

async fn remove_skill(
    skill: &str,
    force: bool,
    manager: &InstanceManager,
) -> Result<()> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let skill_dir = home.join(".skill-engine").join("registry").join(skill);

    if !skill_dir.exists() {
        anyhow::bail!("Skill '{}' not installed", skill);
    }

    // Get all instances
    let instances = manager.list_instances(skill).unwrap_or_default();

    println!();
    println!("{} Removing skill {}", "→".yellow(), skill.cyan());
    println!();
    println!("  {} {}", "Skill:".bold(), skill);
    println!("  {} {}", "Instances:".bold(), instances.len());
    println!();

    if !instances.is_empty() {
        println!("  {}", "The following instances will be removed:".yellow());
        for instance in &instances {
            println!("    • {}", instance.yellow());
        }
        println!();
    }

    // Confirm deletion
    if !force {
        let confirmed = Confirm::new()
            .with_prompt("Are you sure you want to remove this skill and all its instances?")
            .default(false)
            .interact()?;

        if !confirmed {
            println!("{} Cancelled", "!".yellow());
            return Ok(());
        }
    }

    // Remove all instances first
    for instance in &instances {
        manager
            .delete_instance(skill, instance)
            .context("Failed to delete instance")?;
        println!("  {} Removed instance {}", "✓".green(), instance);
    }

    // Remove skill directory from registry
    fs::remove_dir_all(&skill_dir)
        .with_context(|| format!("Failed to remove skill directory: {}", skill_dir.display()))?;

    // Clean up cache directory
    let cache_dir = home.join(".skill-engine").join("cache");
    if cache_dir.exists() {
        for entry in fs::read_dir(&cache_dir)? {
            let entry = entry?;
            let filename = entry.file_name();
            if let Some(name) = filename.to_str() {
                // Remove cached files that start with skill name
                if name.starts_with(&format!("{}_", skill)) {
                    fs::remove_file(entry.path())?;
                    println!("  {} Removed cached module {}", "✓".green(), name);
                }
            }
        }
    }

    println!();
    println!("{} Skill removed successfully", "✓".green().bold());
    Ok(())
}
