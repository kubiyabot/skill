//! Claude Bridge - Generate Claude Agent Skills from Skill Engine skills
//!
//! This module generates Claude Agent Skills that integrate with Skill Engine,
//! providing dual-mode execution (MCP preferred, scripts as fallback).
//!
//! # Features
//!
//! - 100% Claude Agent Skills compliance (YAML frontmatter, filesystem discovery)
//! - Dual execution: MCP tools with context engineering OR standalone scripts
//! - Progressive disclosure: Level 1 (frontmatter), Level 2 (SKILL.md), Level 3 (TOOLS.md)
//! - Single source of truth: scripts wrap `skill run`, no logic duplication
//!
//! # Generated Structure
//!
//! ```text
//! ~/.claude/skills/kubernetes/
//! ├── SKILL.md              # Instructions with MCP + script usage
//! ├── TOOLS.md              # Detailed parameter reference
//! └── scripts/
//!     ├── get.sh            # skill run kubernetes get "$@"
//!     ├── describe.sh
//!     └── ...
//! ```

mod loader;
mod renderer;
mod script_gen;
mod transformer;
mod types;
mod validator;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod edge_cases;

pub use loader::Loader;
pub use renderer::Renderer;
pub use script_gen::ScriptGenerator;
pub use transformer::Transformer;
pub use types::*;
pub use validator::Validator;

use anyhow::Result;

/// Generate Claude Agent Skills for all skills in the manifest
pub async fn generate(options: GenerateOptions) -> Result<GenerateResult> {
    // Load skills from manifest
    let loader = Loader::new(options.manifest_path.as_deref())?;
    let skills = loader.load_all_skills().await?;

    // Filter by skill name if specified
    let skills = if let Some(ref name) = options.skill_name {
        skills.into_iter().filter(|s| &s.name == name).collect()
    } else {
        skills
    };

    if skills.is_empty() {
        if options.skill_name.is_some() {
            anyhow::bail!(
                "Skill '{}' not found in manifest",
                options.skill_name.as_ref().unwrap()
            );
        } else {
            anyhow::bail!("No skills found in manifest");
        }
    }

    // Validate and transform
    let validator = Validator::new();
    let transformer = Transformer::new();

    let mut result = GenerateResult::default();

    for skill in skills {
        // Validate
        let validated = validator.validate(&skill)?;

        // Transform to Claude format
        let claude_skill = transformer.transform(validated)?;

        // Render output
        let renderer = Renderer::new(&options.output_dir)?;
        let script_gen = ScriptGenerator::new(&claude_skill.name);

        if options.dry_run {
            result.dry_run_output.push(format!(
                "Would generate: {}/{}",
                options.output_dir.display(),
                claude_skill.name
            ));
        } else {
            // Generate SKILL.md and TOOLS.md
            renderer.render(&claude_skill)?;

            // Generate scripts (unless disabled)
            if !options.no_scripts {
                script_gen.generate(&claude_skill, &options.output_dir)?;
            }

            result.generated_skills.push(claude_skill.name.clone());
        }
    }

    Ok(result)
}
