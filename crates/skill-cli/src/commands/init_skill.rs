use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::Path;

/// Generate a SKILL.md template for a new or existing skill
pub fn execute(
    skill_name: &str,
    output_path: Option<&str>,
    tools: Option<Vec<String>>,
) -> Result<()> {
    let tools = tools.unwrap_or_else(|| vec!["example-tool".to_string()]);

    let skill_md_content = generate_skill_md_template(skill_name, &tools);

    let output = output_path
        .map(|p| Path::new(p).to_path_buf())
        .unwrap_or_else(|| Path::new(".").join("SKILL.md"));

    // Check if file already exists
    if output.exists() {
        println!(
            "{} SKILL.md already exists at {}",
            "⚠".yellow(),
            output.display()
        );
        println!("   Use --force to overwrite");
        return Ok(());
    }

    fs::write(&output, skill_md_content)
        .with_context(|| format!("Failed to write SKILL.md to {}", output.display()))?;

    println!("{} Generated SKILL.md template", "✓".green().bold());
    println!("   Location: {}", output.display().to_string().cyan());
    println!();
    println!("{} Next steps:", "→".cyan());
    println!("   1. Edit SKILL.md to add your skill description");
    println!("   2. Document each tool with parameters and examples");
    println!("   3. Place SKILL.md alongside your skill.wasm file");
    println!();

    Ok(())
}

/// Generate SKILL.md template content
fn generate_skill_md_template(skill_name: &str, tools: &[String]) -> String {
    let mut content = String::new();

    // YAML Frontmatter
    content.push_str("---\n");
    content.push_str(&format!("name: {}\n", skill_name));
    content.push_str(&format!(
        "description: {} - brief description of what this skill does\n",
        skill_name
    ));
    content.push_str("allowed-tools:\n");
    content.push_str("  - Read\n");
    content.push_str("  - Bash\n");
    content.push_str("# Optional metadata:\n");
    content.push_str("# version: 1.0.0\n");
    content.push_str("# author: Your Name\n");
    content.push_str("# category: utility\n");
    content.push_str("# tags:\n");
    content.push_str("#   - example\n");
    content.push_str("#   - template\n");
    content.push_str("---\n\n");

    // Title
    let title = skill_name
        .split('-')
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    content.push_str(&format!("# {}\n\n", title));
    content.push_str("Brief description of what this skill does and its main purpose.\n\n");

    // When to Use section
    content.push_str("## When to Use\n\n");
    content.push_str("- Use case 1: Describe when to use this skill\n");
    content.push_str("- Use case 2: Another scenario\n");
    content.push_str("- Use case 3: Additional context\n\n");

    // Tools Provided section
    content.push_str("## Tools Provided\n\n");

    for tool in tools {
        content.push_str(&format!("### {}\n", tool));
        content.push_str(&format!("Description of what {} does.\n\n", tool));

        content.push_str("**Usage**:\n");
        content.push_str("```bash\n");
        content.push_str(&format!(
            "skill run {}:{} --param1 value1 --param2 value2\n",
            skill_name, tool
        ));
        content.push_str("```\n\n");

        content.push_str("**Parameters**:\n");
        content.push_str("- `param1` (required): Description of first parameter\n");
        content.push_str("- `param2` (optional): Description of second parameter\n\n");

        content.push_str("**Example**:\n");
        content.push_str("```bash\n");
        content.push_str(&format!(
            "# Example usage with realistic values\n\
             skill run {}@default:{} --param1 \"example value\"\n",
            skill_name, tool
        ));
        content.push_str("```\n\n");
    }

    // Configuration section
    content.push_str("## Configuration\n\n");
    content.push_str("Describe any configuration required for this skill:\n\n");
    content.push_str("```bash\n");
    content.push_str(&format!(
        "skill config {} --set api_key=YOUR_API_KEY\n",
        skill_name
    ));
    content.push_str(&format!(
        "skill config {} --set region=us-east-1\n",
        skill_name
    ));
    content.push_str("```\n\n");

    // Examples section
    content.push_str("## Examples\n\n");
    content.push_str("### Example Workflow 1\n");
    content.push_str("```bash\n");
    content.push_str(&format!(
        "skill run {}@default:{} --param1 value\n",
        skill_name,
        tools.first().unwrap_or(&"tool".to_string())
    ));
    content.push_str("```\n\n");

    // Security Notes section
    content.push_str("## Security Notes\n\n");
    content.push_str("- Credentials are stored securely in the system keychain\n");
    content.push_str("- Each instance has isolated configuration\n");
    content.push_str("- All API calls use TLS encryption\n");

    content
}

/// Generate SKILL.md from existing skill metadata (introspection)
pub async fn generate_from_skill(skill_name: &str, output_path: Option<&str>) -> Result<()> {
    use skill_runtime::{SkillEngine, SkillExecutor};
    use std::sync::Arc;

    // Find the skill
    let skill_dir = dirs::home_dir()
        .context("Failed to get home directory")?
        .join(".skill-engine")
        .join("registry")
        .join(skill_name);

    let skill_path = skill_dir.join(format!("{}.wasm", skill_name));

    if !skill_path.exists() {
        return Err(anyhow::anyhow!(
            "Skill '{}' not found. Install it first with: skill install {}",
            skill_name,
            skill_name
        ));
    }

    println!("{} Loading skill to extract metadata...", "→".cyan());

    // Load skill to get tools
    let engine = Arc::new(SkillEngine::new()?);
    let executor = SkillExecutor::load(
        engine,
        &skill_path,
        skill_name.to_string(),
        "default".to_string(),
        Default::default(),
    )
    .await?;

    let tools = executor.get_tools().await?;
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();

    println!(
        "{} Found {} tools: {}",
        "✓".green(),
        tool_names.len(),
        tool_names.join(", ").cyan()
    );

    // Generate template with actual tool names
    execute(skill_name, output_path, Some(tool_names))
}
