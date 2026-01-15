use anyhow::{Context, Result};
use colored::*;
use skill_runtime::{InstanceManager, SkillManifest};
use std::fs;

pub async fn execute(format: &str, manifest: Option<&SkillManifest>) -> Result<()> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let registry_dir = home.join(".skill-engine").join("registry");

    // Scan registry directory for installed skills
    let mut installed_skills = Vec::new();

    if registry_dir.exists() {
        for entry in fs::read_dir(&registry_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(skill_name) = entry.file_name().to_str() {
                    installed_skills.push(skill_name.to_string());
                }
            }
        }
    }

    installed_skills.sort();

    // Get manifest skills
    let manifest_skills = manifest.map(|m| m.list_skills()).unwrap_or_default();

    // Check if we have anything to show
    if installed_skills.is_empty() && manifest_skills.is_empty() {
        println!("{} No skills found", "!".yellow());
        println!();
        println!("Install a skill with: {} install <path>", "skill".cyan());
        println!(
            "Or create a {} file to define skills",
            ".skill-engine.toml".cyan()
        );
        return Ok(());
    }

    match format {
        "json" => list_json(&installed_skills, &manifest_skills).await,
        "table" | _ => list_table(&installed_skills, &manifest_skills).await,
    }
}

async fn list_table(
    installed_skills: &[String],
    manifest_skills: &[skill_runtime::manifest::SkillInfo],
) -> Result<()> {
    let instance_manager = InstanceManager::new()?;

    let total_skills = installed_skills.len() + manifest_skills.len();
    let manifest_only: Vec<_> = manifest_skills
        .iter()
        .filter(|m| !installed_skills.contains(&m.name))
        .collect();

    println!();
    println!(
        "{} {} skill(s) available",
        "→".cyan(),
        total_skills.to_string().yellow()
    );

    // Show manifest skills first (if any)
    if !manifest_only.is_empty() {
        println!();
        println!(
            "  {} (from manifest)",
            "Skills from .skill-engine.toml".bold()
        );
        println!("  {}", "─".repeat(70).dimmed());
        println!(
            "  {:<20} {:<30} {}",
            "SKILL".bold(),
            "SOURCE".bold(),
            "INSTANCES".bold()
        );
        println!("  {}", "─".repeat(70).dimmed());

        for skill in &manifest_only {
            let instances_str = if skill.instances.is_empty() {
                format!("default={}", skill.default_instance).dimmed().to_string()
            } else {
                skill.instances.join(", ")
            };

            // Truncate source if too long
            let source = if skill.source.len() > 28 {
                format!("{}...", &skill.source[..25])
            } else {
                skill.source.clone()
            };

            println!(
                "  {:<20} {:<30} {}",
                skill.name.cyan(),
                source.dimmed(),
                instances_str.yellow()
            );
        }
    }

    // Show installed skills
    if !installed_skills.is_empty() {
        println!();
        println!("  {}", "Installed Skills".bold());
        println!("  {}", "─".repeat(70).dimmed());
        println!(
            "  {:<20} {:<15} {:<15} {}",
            "SKILL".bold(),
            "INSTANCE".bold(),
            "VERSION".bold(),
            "STATUS".bold()
        );
        println!("  {}", "─".repeat(70).dimmed());

        for skill_name in installed_skills {
            // Check if this skill is also in manifest
            let in_manifest = manifest_skills.iter().any(|m| &m.name == skill_name);

            // Get instances for this skill
            let instances = instance_manager
                .list_instances(skill_name)
                .unwrap_or_default();

            if instances.is_empty() {
                // Skill installed but no instances
                let status_extra = if in_manifest {
                    " (in manifest)".dimmed().to_string()
                } else {
                    String::new()
                };
                println!(
                    "  {:<20} {:<15} {:<15} {}{}",
                    skill_name.cyan(),
                    "-".dimmed(),
                    "-".dimmed(),
                    "No instances".yellow(),
                    status_extra
                );
            } else {
                for (i, instance_name) in instances.iter().enumerate() {
                    let config = instance_manager
                        .load_instance(skill_name, instance_name)
                        .ok();

                    let version = config
                        .as_ref()
                        .map(|c| c.metadata.skill_version.as_str())
                        .unwrap_or("-");

                    let status = if config.is_some() {
                        "Ready".green()
                    } else {
                        "Error".red()
                    };

                    // Only print skill name on first row
                    let skill_display = if i == 0 {
                        let suffix = if in_manifest { "*" } else { "" };
                        format!("{}{}", skill_name.cyan(), suffix.dimmed())
                    } else {
                        " ".repeat(20)
                    };

                    println!(
                        "  {:<20} {:<15} {:<15} {}",
                        skill_display, instance_name.yellow(), version, status
                    );
                }
            }
        }
    }

    println!();

    // Show legend if we have manifest skills
    if !manifest_only.is_empty() || manifest_skills.iter().any(|m| installed_skills.contains(&m.name)) {
        println!(
            "  {} Skills with * are also defined in manifest",
            "Note:".dimmed()
        );
        println!();
    }

    Ok(())
}

async fn list_json(
    installed_skills: &[String],
    manifest_skills: &[skill_runtime::manifest::SkillInfo],
) -> Result<()> {
    use serde_json::json;

    let instance_manager = InstanceManager::new()?;

    // Build installed skills info
    let mut installed_list = Vec::new();
    for skill_name in installed_skills {
        let instances = instance_manager
            .list_instances(skill_name)
            .unwrap_or_default();

        let instance_info: Vec<serde_json::Value> = instances
            .iter()
            .map(|instance_name| {
                let config = instance_manager
                    .load_instance(skill_name, instance_name)
                    .ok();

                json!({
                    "name": instance_name,
                    "version": config.as_ref().map(|c| &c.metadata.skill_version),
                    "created_at": config.as_ref().map(|c| c.metadata.created_at.to_rfc3339()),
                })
            })
            .collect();

        installed_list.push(json!({
            "skill": skill_name,
            "source": "installed",
            "instances": instance_info
        }));
    }

    // Build manifest skills info
    let manifest_list: Vec<serde_json::Value> = manifest_skills
        .iter()
        .filter(|m| !installed_skills.contains(&m.name))
        .map(|skill| {
            json!({
                "skill": skill.name,
                "source": skill.source,
                "description": skill.description,
                "instances": skill.instances,
                "default_instance": skill.default_instance
            })
        })
        .collect();

    let output = json!({
        "installed": installed_list,
        "manifest": manifest_list
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
