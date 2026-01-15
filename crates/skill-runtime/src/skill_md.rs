//! SKILL.md parser for extracting rich skill documentation.
//!
//! This module parses SKILL.md files following Anthropic's skills format:
//! - YAML frontmatter with name, description, allowed-tools
//! - Markdown content with tool documentation, usage examples
//! - Parameter tables and code blocks
//!
//! # Example SKILL.md format
//!
//! ```markdown
//! ---
//! name: kubernetes-skill
//! description: Kubernetes cluster management with kubectl
//! allowed-tools: Read, Bash, skill-run
//! ---
//!
//! # Kubernetes Skill
//!
//! ## Tools Provided
//!
//! ### get
//! Get Kubernetes resources (pods, services, deployments)
//!
//! **Parameters**:
//! - `resource` (required): Resource type
//! - `namespace` (optional): Kubernetes namespace
//!
//! **Example**:
//! ```bash
//! skill run kubernetes get resource=pods namespace=default
//! ```
//! ```

use anyhow::{Context, Result};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// YAML frontmatter from SKILL.md
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillMdFrontmatter {
    /// Skill name
    pub name: String,

    /// Short description for discovery
    pub description: String,

    /// Comma-separated list of allowed tools
    #[serde(default, rename = "allowed-tools")]
    pub allowed_tools: Option<String>,

    /// Additional metadata
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

/// Parsed SKILL.md content
#[derive(Debug, Clone, Default)]
pub struct SkillMdContent {
    /// YAML frontmatter
    pub frontmatter: SkillMdFrontmatter,

    /// Full markdown body (after frontmatter)
    pub body: String,

    /// Tool-specific documentation extracted from markdown
    pub tool_docs: HashMap<String, ToolDocumentation>,

    /// Code examples extracted from the document
    pub examples: Vec<CodeExample>,

    /// When to use this skill (extracted from ## When to Use section)
    pub when_to_use: Option<String>,

    /// Configuration documentation
    pub configuration: Option<String>,
}

/// Documentation for a specific tool
#[derive(Debug, Clone, Default)]
pub struct ToolDocumentation {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Usage instructions
    pub usage: Option<String>,

    /// Parameter documentation
    pub parameters: Vec<ParameterDoc>,

    /// Code examples for this tool
    pub examples: Vec<CodeExample>,
}

/// Parameter type enumeration
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ParameterType {
    #[default]
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Object,
}

impl std::fmt::Display for ParameterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterType::String => write!(f, "string"),
            ParameterType::Integer => write!(f, "integer"),
            ParameterType::Number => write!(f, "number"),
            ParameterType::Boolean => write!(f, "boolean"),
            ParameterType::Array => write!(f, "array"),
            ParameterType::Object => write!(f, "object"),
        }
    }
}

/// Parameter documentation from markdown
#[derive(Debug, Clone)]
pub struct ParameterDoc {
    /// Parameter name
    pub name: String,

    /// Whether the parameter is required
    pub required: bool,

    /// Parameter type (string, integer, boolean, etc.)
    pub param_type: ParameterType,

    /// Parameter description
    pub description: String,

    /// Default value if any
    pub default: Option<String>,

    /// Allowed values (enum)
    pub allowed_values: Vec<String>,
}

/// Code example extracted from markdown
#[derive(Debug, Clone)]
pub struct CodeExample {
    /// Language hint (bash, json, etc.)
    pub language: Option<String>,

    /// The code content
    pub code: String,

    /// Optional description/title
    pub description: Option<String>,
}

/// Parse a SKILL.md file
pub fn parse_skill_md(path: &Path) -> Result<SkillMdContent> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read SKILL.md: {}", path.display()))?;

    parse_skill_md_content(&content)
}

/// Parse SKILL.md content from a string
pub fn parse_skill_md_content(content: &str) -> Result<SkillMdContent> {
    // Split frontmatter and body
    let (frontmatter, body) = extract_frontmatter(content)?;

    // Parse the markdown body
    let tool_docs = extract_tool_sections(&body);
    let examples = extract_code_examples(&body);
    let when_to_use = extract_section(&body, "When to Use");
    let configuration = extract_section(&body, "Configuration");

    Ok(SkillMdContent {
        frontmatter,
        body,
        tool_docs,
        examples,
        when_to_use,
        configuration,
    })
}

/// Extract YAML frontmatter from markdown content
fn extract_frontmatter(content: &str) -> Result<(SkillMdFrontmatter, String)> {
    let content = content.trim();

    // Check for frontmatter delimiter
    if !content.starts_with("---") {
        // No frontmatter, return empty frontmatter and full content as body
        return Ok((SkillMdFrontmatter::default(), content.to_string()));
    }

    // Find the closing delimiter
    let after_first = &content[3..];
    let end_pos = after_first
        .find("\n---")
        .or_else(|| after_first.find("\r\n---"))
        .context("SKILL.md has opening --- but no closing ---")?;

    let yaml_content = &after_first[..end_pos].trim();
    let body_start = 3 + end_pos + 4; // Skip past closing ---
    let body = if body_start < content.len() {
        content[body_start..].trim().to_string()
    } else {
        String::new()
    };

    // Parse YAML frontmatter
    let frontmatter: SkillMdFrontmatter = serde_yaml::from_str(yaml_content)
        .with_context(|| format!("Failed to parse SKILL.md frontmatter: {}", yaml_content))?;

    Ok((frontmatter, body))
}

/// Extract tool documentation sections from markdown
fn extract_tool_sections(markdown: &str) -> HashMap<String, ToolDocumentation> {
    let mut tools = HashMap::new();
    let parser = Parser::new(markdown);

    let mut _current_h2: Option<String> = None;
    let mut _current_h3: Option<String> = None;
    let mut current_tool: Option<ToolDocumentation> = None;
    let mut in_tools_section = false;
    let mut collecting_text = false;
    let mut current_text = String::new();
    let mut in_code_block = false;
    let mut code_lang: Option<String> = None;
    let mut code_content = String::new();
    let mut h3_tool_candidate: Option<ToolDocumentation> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                // Save previous tool if we have one
                if let Some(tool) = current_tool.take() {
                    if !tool.name.is_empty() {
                        tools.insert(tool.name.clone(), tool);
                    }
                }

                collecting_text = true;
                current_text.clear();

                match level {
                    HeadingLevel::H2 => {
                        _current_h3 = None;
                    }
                    HeadingLevel::H3 => {}
                    _ => {}
                }
            }
            Event::End(TagEnd::Heading(level)) => {
                collecting_text = false;
                let heading = current_text.trim().to_string();

                match level {
                    HeadingLevel::H2 => {
                        // Commit H3 candidate if we had one (no H4 followed it)
                        if let Some(h3_tool) = h3_tool_candidate.take() {
                            if !h3_tool.name.is_empty() {
                                tools.insert(h3_tool.name.clone(), h3_tool);
                            }
                        }
                        _current_h2 = Some(heading.clone());
                        in_tools_section = heading.to_lowercase().contains("tools");
                    }
                    HeadingLevel::H3 if in_tools_section => {
                        // Commit previous H3 candidate if we had one (no H4 followed it)
                        if let Some(h3_tool) = h3_tool_candidate.take() {
                            if !h3_tool.name.is_empty() {
                                tools.insert(h3_tool.name.clone(), h3_tool);
                            }
                        }
                        // Save new H3 as candidate
                        _current_h3 = Some(heading.clone());
                        h3_tool_candidate = Some(ToolDocumentation {
                            name: heading,
                            ..Default::default()
                        });
                    }
                    HeadingLevel::H4 if in_tools_section => {
                        // H4 found - discard H3 candidate (it was a category)
                        h3_tool_candidate = None;
                        // H4 is the actual tool/command
                        current_tool = Some(ToolDocumentation {
                            name: heading,
                            ..Default::default()
                        });
                    }
                    _ => {}
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                        let lang_str = lang.to_string();
                        if lang_str.is_empty() {
                            None
                        } else {
                            Some(lang_str)
                        }
                    }
                    _ => None,
                };
                code_content.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                if let Some(ref mut tool) = current_tool {
                    tool.examples.push(CodeExample {
                        language: code_lang.take(),
                        code: code_content.clone(),
                        description: None,
                    });
                }
            }
            Event::Text(text) => {
                if collecting_text {
                    current_text.push_str(&text);
                } else if in_code_block {
                    code_content.push_str(&text);
                } else if let Some(ref mut tool) = current_tool {
                    // Add to tool description if we're in a tool section
                    if tool.description.is_empty() && !text.trim().is_empty() {
                        tool.description = text.trim().to_string();
                    }
                }
            }
            Event::Code(code) => {
                if collecting_text {
                    current_text.push_str(&code);
                }
            }
            _ => {}
        }
    }

    // Save last tool (either H4-based or H3 candidate)
    if let Some(tool) = current_tool {
        if !tool.name.is_empty() {
            tools.insert(tool.name.clone(), tool);
        }
    } else if let Some(h3_tool) = h3_tool_candidate {
        // No H4 followed the last H3, so it was a tool
        if !h3_tool.name.is_empty() {
            tools.insert(h3_tool.name.clone(), h3_tool);
        }
    }

    // CRITICAL FIX: Extract parameters for each tool by parsing their sections
    extract_tool_parameters(markdown, &mut tools);

    tools
}

/// Extract and parse parameters for each tool from the markdown content
/// This function finds the **Parameters**: section under each tool heading
fn extract_tool_parameters(markdown: &str, tools: &mut HashMap<String, ToolDocumentation>) {
    for (tool_name, tool_doc) in tools.iter_mut() {
        // Find the tool section in markdown
        if let Some(tool_section) = extract_tool_section_content(markdown, tool_name) {
            // Look for **Parameters**: section
            if let Some(params_text) = extract_parameters_section(&tool_section) {
                tool_doc.parameters = parse_parameters(&params_text);
            }
        }
    }
}

/// Extract the content of a specific tool section (from heading to next same-level heading)
fn extract_tool_section_content(markdown: &str, tool_name: &str) -> Option<String> {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut start_idx: Option<usize> = None;
    let mut section_level: Option<usize> = None;

    // Find the tool heading
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Check for H3 (###) or H4 (####) headings matching tool name
        if (trimmed.starts_with("### ") || trimmed.starts_with("#### "))
            && trimmed.trim_start_matches('#').trim() == tool_name {
            start_idx = Some(idx);
            section_level = Some(trimmed.chars().take_while(|c| *c == '#').count());
            break;
        }
    }

    let start_idx = start_idx?;
    let section_level = section_level?;

    // Find the end of this section (next heading at same or higher level)
    let mut end_idx = lines.len();
    for (idx, line) in lines.iter().enumerate().skip(start_idx + 1) {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|c| *c == '#').count();
            if level <= section_level {
                end_idx = idx;
                break;
            }
        }
    }

    // Extract the section content
    let section_lines = &lines[start_idx..end_idx];
    Some(section_lines.join("\n"))
}

/// Extract the **Parameters**: section content from a tool section
fn extract_parameters_section(tool_section: &str) -> Option<String> {
    let lines: Vec<&str> = tool_section.lines().collect();
    let mut params_start: Option<usize> = None;

    // Find **Parameters**: line
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("**Parameters") && trimmed.contains(':') {
            params_start = Some(idx);
            break;
        }
    }

    let params_start = params_start?;

    // Find the end of parameters section (next **Section** or empty line followed by heading)
    let mut params_end = lines.len();
    for (idx, line) in lines.iter().enumerate().skip(params_start + 1) {
        let trimmed = line.trim();
        // Stop at next bold section or example section
        if trimmed.starts_with("**") && !trimmed.starts_with("**Parameters") {
            params_end = idx;
            break;
        }
        // Stop at code block
        if trimmed.starts_with("```") {
            params_end = idx;
            break;
        }
    }

    let params_lines = &lines[params_start..params_end];
    Some(params_lines.join("\n"))
}

/// Extract all code examples from markdown
fn extract_code_examples(markdown: &str) -> Vec<CodeExample> {
    let parser = Parser::new(markdown);
    let mut examples = Vec::new();
    let mut in_code_block = false;
    let mut code_lang: Option<String> = None;
    let mut code_content = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                        let lang_str = lang.to_string();
                        if lang_str.is_empty() {
                            None
                        } else {
                            Some(lang_str)
                        }
                    }
                    _ => None,
                };
                code_content.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                examples.push(CodeExample {
                    language: code_lang.take(),
                    code: code_content.clone(),
                    description: None,
                });
            }
            Event::Text(text) if in_code_block => {
                code_content.push_str(&text);
            }
            _ => {}
        }
    }

    examples
}

/// Extract a specific section by heading name
fn extract_section(markdown: &str, section_name: &str) -> Option<String> {
    let parser = Parser::new(markdown);
    let mut in_target_section = false;
    let mut content = String::new();
    let mut collecting_heading = false;
    let mut heading_text = String::new();
    let mut target_level: Option<HeadingLevel> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                if in_target_section {
                    // Check if this heading ends our section
                    if let Some(target) = target_level {
                        if level <= target {
                            break;
                        }
                    }
                }
                collecting_heading = true;
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(level)) => {
                collecting_heading = false;
                if heading_text.to_lowercase().contains(&section_name.to_lowercase()) {
                    in_target_section = true;
                    target_level = Some(level);
                }
            }
            Event::Text(text) => {
                if collecting_heading {
                    heading_text.push_str(&text);
                } else if in_target_section {
                    content.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak if in_target_section => {
                content.push('\n');
            }
            Event::Start(Tag::Paragraph) if in_target_section => {}
            Event::End(TagEnd::Paragraph) if in_target_section => {
                content.push('\n');
            }
            Event::Start(Tag::Item) if in_target_section => {
                content.push_str("- ");
            }
            Event::End(TagEnd::Item) if in_target_section => {
                content.push('\n');
            }
            _ => {}
        }
    }

    if content.trim().is_empty() {
        None
    } else {
        Some(content.trim().to_string())
    }
}

/// Parse parameter documentation from a tool section
///
/// Supports formats:
/// - `name` (required): description
/// - `name` (optional, string): description
/// - `name` (required, integer): description
/// - `name` (optional, boolean, default: true): description
/// - `name` (required, enum: value1|value2|value3): description
pub fn parse_parameters(text: &str) -> Vec<ParameterDoc> {
    let mut params = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with('-') && !line.starts_with('*') {
            continue;
        }

        // Remove leading bullet
        let line = line.trim_start_matches('-').trim_start_matches('*').trim();

        // Try to extract parameter name (in backticks or bold)
        let (name, rest) = if line.starts_with('`') {
            if let Some(end) = line[1..].find('`') {
                let name = &line[1..=end];
                let rest = &line[end + 2..];
                (name.to_string(), rest.trim())
            } else {
                continue;
            }
        } else if line.starts_with("**") {
            if let Some(end) = line[2..].find("**") {
                let name = &line[2..end + 2];
                let rest = &line[end + 4..];
                (name.to_string(), rest.trim())
            } else {
                continue;
            }
        } else {
            continue;
        };

        let rest_lower = rest.to_lowercase();

        // Check for required/optional
        let required = rest_lower.contains("required");

        // Extract type from parentheses content
        // Patterns: (required), (optional, string), (required, integer), etc.
        let param_type = if rest_lower.contains("integer") || rest_lower.contains("int)") {
            ParameterType::Integer
        } else if rest_lower.contains("number") || rest_lower.contains("float") {
            ParameterType::Number
        } else if rest_lower.contains("boolean") || rest_lower.contains("bool") {
            ParameterType::Boolean
        } else if rest_lower.contains("array") || rest_lower.contains("list") {
            ParameterType::Array
        } else if rest_lower.contains("object") || rest_lower.contains("json") {
            ParameterType::Object
        } else {
            ParameterType::String
        };

        // Extract default value if present
        // Pattern: default: value or default=value
        let default = if let Some(pos) = rest_lower.find("default:") {
            let after = &rest[pos + 8..];
            // Find the end (comma, paren, or end of parentheses block)
            let end = after.find(|c: char| c == ',' || c == ')').unwrap_or(after.len());
            Some(after[..end].trim().to_string())
        } else if let Some(pos) = rest_lower.find("default=") {
            let after = &rest[pos + 8..];
            let end = after.find(|c: char| c == ',' || c == ')').unwrap_or(after.len());
            Some(after[..end].trim().to_string())
        } else {
            None
        };

        // Extract allowed values (enum)
        // Pattern: enum: value1|value2|value3 or values: [a, b, c]
        let allowed_values = if let Some(pos) = rest_lower.find("enum:") {
            let after = &rest[pos + 5..];
            let end = after.find(')').unwrap_or(after.len());
            after[..end]
                .split('|')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        };

        // Extract description (after the colon following parentheses)
        let description = if let Some(colon_pos) = rest.find(':') {
            // Skip if the colon is inside parentheses (part of default: or enum:)
            let before_colon = &rest[..colon_pos];
            let open_parens = before_colon.matches('(').count();
            let close_parens = before_colon.matches(')').count();

            if open_parens > close_parens {
                // Colon is inside parentheses, look for next colon after closing paren
                if let Some(paren_end) = rest.find(')') {
                    if let Some(next_colon) = rest[paren_end..].find(':') {
                        rest[paren_end + next_colon + 1..].trim().to_string()
                    } else {
                        rest[paren_end + 1..].trim().to_string()
                    }
                } else {
                    rest.to_string()
                }
            } else {
                rest[colon_pos + 1..].trim().to_string()
            }
        } else {
            rest.to_string()
        };

        params.push(ParameterDoc {
            name,
            required,
            param_type,
            description,
            default,
            allowed_values,
        });
    }

    params
}

/// Find SKILL.md file in a skill directory
pub fn find_skill_md(skill_dir: &Path) -> Option<std::path::PathBuf> {
    let skill_md = skill_dir.join("SKILL.md");
    if skill_md.exists() {
        return Some(skill_md);
    }

    // Try lowercase
    let skill_md_lower = skill_dir.join("skill.md");
    if skill_md_lower.exists() {
        return Some(skill_md_lower);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: test-skill
description: A test skill for unit testing
allowed-tools: Read, Bash
---

# Test Skill

This is the body content.
"#;

        let result = parse_skill_md_content(content).unwrap();
        assert_eq!(result.frontmatter.name, "test-skill");
        assert_eq!(result.frontmatter.description, "A test skill for unit testing");
        assert_eq!(result.frontmatter.allowed_tools, Some("Read, Bash".to_string()));
        assert!(result.body.contains("# Test Skill"));
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = r#"# Just a Markdown File

No frontmatter here.
"#;

        let result = parse_skill_md_content(content).unwrap();
        assert!(result.frontmatter.name.is_empty());
        assert!(result.body.contains("# Just a Markdown File"));
    }

    #[test]
    fn test_extract_tool_sections() {
        let markdown = r#"
# Skill

## Tools Provided

### get
Get resources from the cluster.

### delete
Delete resources from the cluster.

## Configuration

Some config info.
"#;

        let tools = extract_tool_sections(markdown);
        assert!(tools.contains_key("get"));
        assert!(tools.contains_key("delete"));
        assert_eq!(tools.get("get").unwrap().description, "Get resources from the cluster.");
    }

    #[test]
    fn test_extract_code_examples() {
        let markdown = r#"
# Example

```bash
skill run kubernetes get resource=pods
```

Some text.

```json
{"key": "value"}
```
"#;

        let examples = extract_code_examples(markdown);
        assert_eq!(examples.len(), 2);
        assert_eq!(examples[0].language, Some("bash".to_string()));
        assert!(examples[0].code.contains("skill run"));
        assert_eq!(examples[1].language, Some("json".to_string()));
    }

    #[test]
    fn test_extract_section() {
        let markdown = r#"
# Skill

## When to Use

Use this skill when you need to:
- Manage Kubernetes resources
- Deploy applications

## Configuration

Set up credentials first.
"#;

        let when_to_use = extract_section(markdown, "When to Use").unwrap();
        assert!(when_to_use.contains("Manage Kubernetes"));
        assert!(when_to_use.contains("Deploy applications"));
    }

    #[test]
    fn test_parse_parameters() {
        let text = r#"
**Parameters**:
- `resource` (required): The resource type to get
- `namespace` (optional): Kubernetes namespace
- `output` (optional): Output format
"#;

        let params = parse_parameters(text);
        assert_eq!(params.len(), 3);
        assert_eq!(params[0].name, "resource");
        assert!(params[0].required);
        assert_eq!(params[0].param_type, ParameterType::String);
        assert_eq!(params[1].name, "namespace");
        assert!(!params[1].required);
    }

    #[test]
    fn test_parse_parameters_with_types() {
        let text = r#"
**Parameters**:
- `count` (required, integer): Number of items to return
- `enabled` (optional, boolean, default: true): Enable feature
- `replicas` (required, integer): Desired replica count
- `format` (optional, enum: json|yaml|table): Output format
"#;

        let params = parse_parameters(text);
        assert_eq!(params.len(), 4);

        assert_eq!(params[0].name, "count");
        assert!(params[0].required);
        assert_eq!(params[0].param_type, ParameterType::Integer);

        assert_eq!(params[1].name, "enabled");
        assert!(!params[1].required);
        assert_eq!(params[1].param_type, ParameterType::Boolean);
        assert_eq!(params[1].default, Some("true".to_string()));

        assert_eq!(params[2].name, "replicas");
        assert_eq!(params[2].param_type, ParameterType::Integer);

        assert_eq!(params[3].name, "format");
        assert_eq!(params[3].allowed_values, vec!["json", "yaml", "table"]);
    }
}
