//! Renderer - Render SKILL.md and TOOLS.md using templates
//!
//! This module generates the Claude Agent Skills markdown files using
//! handlebars templates for consistent formatting.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::types::ClaudeSkill;

/// Renders Claude Agent Skills markdown files
pub struct Renderer {
    output_dir: PathBuf,
}

impl Renderer {
    /// Create a new renderer with the specified output directory
    pub fn new(output_dir: &Path) -> Result<Self> {
        fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;

        Ok(Self {
            output_dir: output_dir.to_path_buf(),
        })
    }

    /// Render all files for a Claude Skill
    pub fn render(&self, skill: &ClaudeSkill) -> Result<()> {
        let skill_dir = self.output_dir.join(&skill.name);
        fs::create_dir_all(&skill_dir)?;

        // Render SKILL.md
        let skill_md = self.render_skill_md(skill);
        let skill_md_path = skill_dir.join("SKILL.md");
        fs::write(&skill_md_path, skill_md)
            .with_context(|| format!("Failed to write SKILL.md: {}", skill_md_path.display()))?;

        // Render TOOLS.md
        let tools_md = self.render_tools_md(skill);
        let tools_md_path = skill_dir.join("TOOLS.md");
        fs::write(&tools_md_path, tools_md)
            .with_context(|| format!("Failed to write TOOLS.md: {}", tools_md_path.display()))?;

        Ok(())
    }

    /// Render SKILL.md content
    fn render_skill_md(&self, skill: &ClaudeSkill) -> String {
        let mut md = String::new();

        // YAML frontmatter (required for Claude Agent Skills)
        md.push_str("---\n");
        md.push_str(&format!("name: {}\n", skill.name));
        md.push_str(&format!("description: {}\n", escape_yaml(&skill.description)));
        md.push_str("---\n\n");

        // Title
        md.push_str(&format!("# {}\n\n", titlecase(&skill.name)));

        // Description
        md.push_str(&format!("{}\n\n", skill.description));

        // When to use
        md.push_str("## When to Use\n\n");
        for trigger in &skill.when_to_use {
            md.push_str(&format!("- {}\n", trigger));
        }
        md.push('\n');

        // Execution modes
        md.push_str("## How to Execute\n\n");
        md.push_str("This skill supports two execution methods. **Choose based on your environment:**\n\n");

        md.push_str("### Method 1: MCP Tools (Preferred in Claude Code)\n\n");
        md.push_str("When running in Claude Code with MCP enabled, use the `execute` tool:\n\n");
        md.push_str("```\n");
        md.push_str(&format!(
            "mcp__skill-engine__execute(\n  skill='{}',\n  tool='<tool_name>',\n  args={{...}}\n)\n",
            skill.name
        ));
        md.push_str("```\n\n");
        md.push_str("**Context Engineering Options:**\n");
        md.push_str("- `grep='pattern'` - Filter output to lines matching pattern\n");
        md.push_str("- `jq='.field'` - Extract JSON fields from output\n");
        md.push_str("- `max_output=4000` - Limit output size (prevents context overflow)\n");
        md.push_str("- `head=10` / `tail=10` - Show first/last N lines\n\n");

        md.push_str("### Method 2: Scripts (Fallback / claude.ai)\n\n");
        md.push_str("When MCP is not available, use the shell scripts:\n\n");
        md.push_str("```bash\n");
        md.push_str("./scripts/<tool_name>.sh arg1=value1 arg2=value2\n");
        md.push_str("```\n\n");

        // Quick reference table
        md.push_str("## Quick Reference\n\n");
        md.push_str("| Tool | Description | Category |\n");
        md.push_str("|------|-------------|----------|\n");
        for tool in &skill.tools {
            let category = tool.category.as_deref().unwrap_or("General");
            let desc = truncate(&tool.description, 50);
            md.push_str(&format!("| `{}` | {} | {} |\n", tool.name, desc, category));
        }
        md.push('\n');

        // Tools by category
        md.push_str("## Tools by Category\n\n");
        for (category, tool_names) in &skill.categories {
            md.push_str(&format!("### {}\n\n", category));

            for tool_name in tool_names {
                if let Some(tool) = skill.tools.iter().find(|t| &t.name == tool_name) {
                    md.push_str(&format!("#### `{}`\n\n", tool.name));
                    md.push_str(&format!("{}\n\n", tool.description));

                    // Parameters summary
                    if !tool.parameters.is_empty() {
                        md.push_str("**Parameters:**\n");
                        for param in &tool.parameters {
                            let required = if param.required { " *(required)*" } else { "" };
                            let default = param
                                .default_value
                                .as_ref()
                                .map(|v| format!(" (default: `{}`)", v))
                                .unwrap_or_default();

                            md.push_str(&format!(
                                "- `{}`: {}{}{}\n",
                                param.name, param.description, required, default
                            ));
                        }
                        md.push('\n');
                    }

                    // Example
                    md.push_str("**Example:**\n\n");
                    md.push_str("```\n");
                    md.push_str(&format!(
                        "# MCP\nexecute(skill='{}', tool='{}', args={{{}}})\n\n",
                        skill.name,
                        tool.name,
                        self.format_example_args(&tool.parameters)
                    ));
                    md.push_str(&format!(
                        "# Script\n./scripts/{}.sh {}\n",
                        tool.name,
                        self.format_script_args(&tool.parameters)
                    ));
                    md.push_str("```\n\n");
                }
            }
        }

        // Scripts list
        md.push_str("## Available Scripts\n\n");
        md.push_str("All scripts are in the `scripts/` directory:\n\n");
        for tool in &skill.tools {
            md.push_str(&format!("- `scripts/{}.sh` - {}\n", tool.name, tool.description));
        }
        md.push('\n');

        // Related resources
        md.push_str("## Related Resources\n\n");
        md.push_str("- [TOOLS.md](TOOLS.md) - Detailed parameter documentation\n");

        md
    }

    /// Render TOOLS.md content
    fn render_tools_md(&self, skill: &ClaudeSkill) -> String {
        let mut md = String::new();

        // Header
        md.push_str(&format!("# {} - Tool Reference\n\n", titlecase(&skill.name)));
        md.push_str("Detailed parameter documentation for all tools.\n\n");

        // Table of contents
        md.push_str("## Table of Contents\n\n");
        for tool in &skill.tools {
            md.push_str(&format!("- [{}](#{})\n", tool.name, tool.name.replace('_', "-")));
        }
        md.push_str("\n---\n\n");

        // Tool details
        for tool in &skill.tools {
            md.push_str(&format!("## {}\n\n", tool.name));
            md.push_str(&format!("{}\n\n", tool.description));

            // Parameters table
            if !tool.parameters.is_empty() {
                md.push_str("### Parameters\n\n");
                md.push_str("| Parameter | Type | Required | Default | Description |\n");
                md.push_str("|-----------|------|----------|---------|-------------|\n");

                for param in &tool.parameters {
                    let required = if param.required { "Yes" } else { "No" };
                    let default = param.default_value.as_deref().unwrap_or("-");

                    md.push_str(&format!(
                        "| `{}` | {} | {} | {} | {} |\n",
                        param.name, param.param_type, required, default, param.description
                    ));
                }
                md.push('\n');
            } else {
                md.push_str("*No parameters required.*\n\n");
            }

            // Examples
            md.push_str("### Examples\n\n");

            md.push_str("**MCP:**\n```\n");
            md.push_str(&format!(
                "execute(\n  skill='{}',\n  tool='{}',\n  args={{{}}}\n)\n",
                skill.name,
                tool.name,
                self.format_example_args(&tool.parameters)
            ));
            md.push_str("```\n\n");

            md.push_str("**Script:**\n```bash\n");
            md.push_str(&format!(
                "./scripts/{}.sh {}\n",
                tool.name,
                self.format_script_args(&tool.parameters)
            ));
            md.push_str("```\n\n");

            md.push_str("---\n\n");
        }

        md
    }

    /// Format example args for MCP call
    fn format_example_args(&self, params: &[super::types::ClaudeToolParameter]) -> String {
        if params.is_empty() {
            return String::new();
        }

        params
            .iter()
            .filter(|p| p.required)
            .map(|p| format!("{}: '<value>'", p.name))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Format script args
    fn format_script_args(&self, params: &[super::types::ClaudeToolParameter]) -> String {
        if params.is_empty() {
            return String::new();
        }

        params
            .iter()
            .filter(|p| p.required)
            .map(|p| format!("{}=<value>", p.name))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Convert a string to title case
fn titlecase(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Truncate a string to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Escape special characters in YAML strings
fn escape_yaml(s: &str) -> String {
    if s.contains(':') || s.contains('#') || s.contains('\n') || s.starts_with(' ') {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::claude_bridge::types::{
        ClaudeTool, ClaudeToolParameter, SkillRuntimeType,
    };
    use std::collections::HashMap;
    use tempfile::TempDir;

    // === Helper Function Tests ===

    #[test]
    fn test_titlecase() {
        assert_eq!(titlecase("kubernetes"), "Kubernetes");
        assert_eq!(titlecase("my-skill"), "My Skill");
        assert_eq!(titlecase("aws-s3-ops"), "Aws S3 Ops");
    }

    #[test]
    fn test_titlecase_single_word() {
        assert_eq!(titlecase("test"), "Test");
        assert_eq!(titlecase("docker"), "Docker");
    }

    #[test]
    fn test_titlecase_empty() {
        assert_eq!(titlecase(""), "");
    }

    #[test]
    fn test_titlecase_multiple_hyphens() {
        assert_eq!(titlecase("one-two-three-four"), "One Two Three Four");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a longer string", 15), "this is a lo...");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate("exactly ten", 11), "exactly ten");
    }

    #[test]
    fn test_truncate_empty() {
        assert_eq!(truncate("", 10), "");
    }

    #[test]
    fn test_truncate_very_short_max() {
        assert_eq!(truncate("hello", 5), "hello");
        assert_eq!(truncate("hello world", 5), "he...");
    }

    #[test]
    fn test_escape_yaml() {
        assert_eq!(escape_yaml("simple"), "simple");
        assert_eq!(escape_yaml("has: colon"), "\"has: colon\"");
        assert_eq!(escape_yaml("has # comment"), "\"has # comment\"");
    }

    #[test]
    fn test_escape_yaml_newline() {
        assert_eq!(escape_yaml("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn test_escape_yaml_leading_space() {
        assert_eq!(escape_yaml(" leading"), "\" leading\"");
    }

    #[test]
    fn test_escape_yaml_quotes() {
        // Quotes alone don't trigger escaping - only when combined with special chars
        assert_eq!(escape_yaml("has \"quotes\""), "has \"quotes\"");
        // But quotes with colon do
        assert_eq!(escape_yaml("has: \"quotes\""), "\"has: \\\"quotes\\\"\"");
    }

    #[test]
    fn test_escape_yaml_empty() {
        assert_eq!(escape_yaml(""), "");
    }

    // === Renderer Creation Tests ===

    #[test]
    fn test_renderer_new_creates_directory() {
        let temp = TempDir::new().unwrap();
        let output_dir = temp.path().join("output");

        let renderer = Renderer::new(&output_dir);
        assert!(renderer.is_ok());
        assert!(output_dir.exists());
    }

    #[test]
    fn test_renderer_new_existing_directory() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path()).unwrap();

        let renderer = Renderer::new(temp.path());
        assert!(renderer.is_ok());
    }

    // === SKILL.md Rendering Tests ===

    #[test]
    fn test_render_skill_md_basic() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let skill = ClaudeSkill {
            name: "test-skill".to_string(),
            description: "Test skill description".to_string(),
            tools: vec![],
            categories: HashMap::new(),
            when_to_use: vec!["When user needs test".to_string()],
            runtime: SkillRuntimeType::Wasm,
        };

        let md = renderer.render_skill_md(&skill);

        // Check YAML frontmatter
        assert!(md.starts_with("---\n"));
        assert!(md.contains("name: test-skill"));
        assert!(md.contains("description: Test skill description"));

        // Check title
        assert!(md.contains("# Test Skill"));

        // Check sections
        assert!(md.contains("## When to Use"));
        assert!(md.contains("## How to Execute"));
        assert!(md.contains("## Quick Reference"));
    }

    #[test]
    fn test_render_skill_md_with_tools() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let mut categories = HashMap::new();
        categories.insert("Read Operations".to_string(), vec!["get_pods".to_string()]);

        let skill = ClaudeSkill {
            name: "kubernetes".to_string(),
            description: "Kubernetes management".to_string(),
            tools: vec![ClaudeTool {
                name: "get_pods".to_string(),
                description: "Get pods from cluster".to_string(),
                parameters: vec![],
                examples: vec![],
                category: Some("Read Operations".to_string()),
                streaming: false,
            }],
            categories,
            when_to_use: vec!["When working with pods".to_string()],
            runtime: SkillRuntimeType::Native,
        };

        let md = renderer.render_skill_md(&skill);

        // Check tool is listed
        assert!(md.contains("get_pods"));
        assert!(md.contains("Get pods from cluster"));
        assert!(md.contains("Read Operations"));
    }

    #[test]
    fn test_render_skill_md_with_parameters() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let mut categories = HashMap::new();
        categories.insert("General".to_string(), vec!["get_resource".to_string()]);

        let skill = ClaudeSkill {
            name: "test".to_string(),
            description: "Test".to_string(),
            tools: vec![ClaudeTool {
                name: "get_resource".to_string(),
                description: "Get a resource".to_string(),
                parameters: vec![
                    ClaudeToolParameter {
                        name: "resource_type".to_string(),
                        param_type: "string".to_string(),
                        description: "Type of resource".to_string(),
                        required: true,
                        default_value: None,
                        enum_values: None,
                    },
                    ClaudeToolParameter {
                        name: "namespace".to_string(),
                        param_type: "string".to_string(),
                        description: "Namespace".to_string(),
                        required: false,
                        default_value: Some("default".to_string()),
                        enum_values: None,
                    },
                ],
                examples: vec![],
                category: Some("General".to_string()),
                streaming: false,
            }],
            categories,
            when_to_use: vec!["Test trigger".to_string()],
            runtime: SkillRuntimeType::Wasm,
        };

        let md = renderer.render_skill_md(&skill);

        // Check parameters are documented
        assert!(md.contains("resource_type"));
        assert!(md.contains("*(required)*"));
        assert!(md.contains("namespace"));
        assert!(md.contains("(default: `default`)"));
    }

    #[test]
    fn test_render_skill_md_yaml_escaping() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let skill = ClaudeSkill {
            name: "test".to_string(),
            description: "Description with: colon and # comment".to_string(),
            tools: vec![],
            categories: HashMap::new(),
            when_to_use: vec![],
            runtime: SkillRuntimeType::Wasm,
        };

        let md = renderer.render_skill_md(&skill);

        // Check YAML is properly escaped
        assert!(md.contains("description: \"Description with: colon and # comment\""));
    }

    // === TOOLS.md Rendering Tests ===

    #[test]
    fn test_render_tools_md_basic() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let skill = ClaudeSkill {
            name: "test".to_string(),
            description: "Test".to_string(),
            tools: vec![ClaudeTool {
                name: "tool1".to_string(),
                description: "First tool".to_string(),
                parameters: vec![],
                examples: vec![],
                category: None,
                streaming: false,
            }],
            categories: HashMap::new(),
            when_to_use: vec![],
            runtime: SkillRuntimeType::Wasm,
        };

        let md = renderer.render_tools_md(&skill);

        // Check header
        assert!(md.contains("# Test - Tool Reference"));
        assert!(md.contains("## Table of Contents"));

        // Check tool section
        assert!(md.contains("## tool1"));
        assert!(md.contains("First tool"));
    }

    #[test]
    fn test_render_tools_md_with_parameters() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let skill = ClaudeSkill {
            name: "test".to_string(),
            description: "Test".to_string(),
            tools: vec![ClaudeTool {
                name: "get".to_string(),
                description: "Get resource".to_string(),
                parameters: vec![
                    ClaudeToolParameter {
                        name: "id".to_string(),
                        param_type: "string".to_string(),
                        description: "Resource ID".to_string(),
                        required: true,
                        default_value: None,
                        enum_values: None,
                    },
                    ClaudeToolParameter {
                        name: "format".to_string(),
                        param_type: "string".to_string(),
                        description: "Output format".to_string(),
                        required: false,
                        default_value: Some("json".to_string()),
                        enum_values: None,
                    },
                ],
                examples: vec![],
                category: None,
                streaming: false,
            }],
            categories: HashMap::new(),
            when_to_use: vec![],
            runtime: SkillRuntimeType::Native,
        };

        let md = renderer.render_tools_md(&skill);

        // Check parameters table
        assert!(md.contains("### Parameters"));
        assert!(md.contains("| Parameter | Type | Required | Default | Description |"));
        assert!(md.contains("| `id` | string | Yes | - | Resource ID |"));
        assert!(md.contains("| `format` | string | No | json | Output format |"));
    }

    #[test]
    fn test_render_tools_md_no_parameters() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let skill = ClaudeSkill {
            name: "test".to_string(),
            description: "Test".to_string(),
            tools: vec![ClaudeTool {
                name: "list_all".to_string(),
                description: "List everything".to_string(),
                parameters: vec![],
                examples: vec![],
                category: None,
                streaming: false,
            }],
            categories: HashMap::new(),
            when_to_use: vec![],
            runtime: SkillRuntimeType::Wasm,
        };

        let md = renderer.render_tools_md(&skill);

        // Check no parameters message
        assert!(md.contains("*No parameters required.*"));
    }

    #[test]
    fn test_render_tools_md_table_of_contents() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let skill = ClaudeSkill {
            name: "test".to_string(),
            description: "Test".to_string(),
            tools: vec![
                ClaudeTool {
                    name: "tool_one".to_string(),
                    description: "First".to_string(),
                    parameters: vec![],
                    examples: vec![],
                    category: None,
                    streaming: false,
                },
                ClaudeTool {
                    name: "tool_two".to_string(),
                    description: "Second".to_string(),
                    parameters: vec![],
                    examples: vec![],
                    category: None,
                    streaming: false,
                },
            ],
            categories: HashMap::new(),
            when_to_use: vec![],
            runtime: SkillRuntimeType::Docker,
        };

        let md = renderer.render_tools_md(&skill);

        // Check TOC has links
        assert!(md.contains("- [tool_one](#tool-one)"));
        assert!(md.contains("- [tool_two](#tool-two)"));
    }

    // === Full Render Tests ===

    #[test]
    fn test_render_creates_files() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let skill = ClaudeSkill {
            name: "test-skill".to_string(),
            description: "Test".to_string(),
            tools: vec![],
            categories: HashMap::new(),
            when_to_use: vec![],
            runtime: SkillRuntimeType::Wasm,
        };

        let result = renderer.render(&skill);
        assert!(result.is_ok());

        // Check files were created
        let skill_dir = temp.path().join("test-skill");
        assert!(skill_dir.exists());
        assert!(skill_dir.join("SKILL.md").exists());
        assert!(skill_dir.join("TOOLS.md").exists());
    }

    #[test]
    fn test_render_file_contents() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let skill = ClaudeSkill {
            name: "myskill".to_string(),
            description: "My test skill".to_string(),
            tools: vec![],
            categories: HashMap::new(),
            when_to_use: vec!["Test trigger".to_string()],
            runtime: SkillRuntimeType::Native,
        };

        renderer.render(&skill).unwrap();

        // Read and verify SKILL.md
        let skill_md = fs::read_to_string(temp.path().join("myskill/SKILL.md")).unwrap();
        assert!(skill_md.contains("name: myskill"));
        assert!(skill_md.contains("# Myskill"));

        // Read and verify TOOLS.md
        let tools_md = fs::read_to_string(temp.path().join("myskill/TOOLS.md")).unwrap();
        assert!(tools_md.contains("# Myskill - Tool Reference"));
    }

    // === Argument Formatting Tests ===

    #[test]
    fn test_format_example_args_empty() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let result = renderer.format_example_args(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_example_args_required_only() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let params = vec![
            ClaudeToolParameter {
                name: "param1".to_string(),
                param_type: "string".to_string(),
                description: "First".to_string(),
                required: true,
                default_value: None,
                enum_values: None,
            },
            ClaudeToolParameter {
                name: "param2".to_string(),
                param_type: "string".to_string(),
                description: "Second".to_string(),
                required: false,
                default_value: Some("default".to_string()),
                enum_values: None,
            },
        ];

        let result = renderer.format_example_args(&params);
        assert_eq!(result, "param1: '<value>'");
    }

    #[test]
    fn test_format_script_args_empty() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let result = renderer.format_script_args(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_script_args_required_only() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let params = vec![
            ClaudeToolParameter {
                name: "arg1".to_string(),
                param_type: "string".to_string(),
                description: "First".to_string(),
                required: true,
                default_value: None,
                enum_values: None,
            },
            ClaudeToolParameter {
                name: "arg2".to_string(),
                param_type: "string".to_string(),
                description: "Second".to_string(),
                required: false,
                default_value: Some("default".to_string()),
                enum_values: None,
            },
        ];

        let result = renderer.format_script_args(&params);
        assert_eq!(result, "arg1=<value>");
    }

    #[test]
    fn test_format_script_args_multiple_required() {
        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();

        let params = vec![
            ClaudeToolParameter {
                name: "arg1".to_string(),
                param_type: "string".to_string(),
                description: "First".to_string(),
                required: true,
                default_value: None,
                enum_values: None,
            },
            ClaudeToolParameter {
                name: "arg2".to_string(),
                param_type: "string".to_string(),
                description: "Second".to_string(),
                required: true,
                default_value: None,
                enum_values: None,
            },
        ];

        let result = renderer.format_script_args(&params);
        assert_eq!(result, "arg1=<value> arg2=<value>");
    }
}
