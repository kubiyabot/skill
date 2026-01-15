//! Validator - Validate skills against Claude Agent Skills requirements
//!
//! Claude Agent Skills have specific requirements:
//! - Name: max 64 characters, lowercase, alphanumeric with hyphens
//! - Description: max 1024 characters
//! - Tool names: lowercase, alphanumeric with underscores

use anyhow::Result;

use super::types::{RawSkill, RawTool, ValidatedParameter, ValidatedSkill, ValidatedTool};

/// Maximum length for skill name
pub const MAX_NAME_LENGTH: usize = 64;

/// Maximum length for description
pub const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// Validator for Claude Agent Skills compliance
pub struct Validator {
    /// Whether to warn instead of error on validation issues
    pub lenient: bool,
}

impl Validator {
    /// Create a new validator with default (lenient) settings
    /// Lenient mode automatically cleans/truncates invalid values
    pub fn new() -> Self {
        Self { lenient: true }
    }

    /// Create a strict validator that errors on validation issues
    #[allow(dead_code)]
    pub fn strict() -> Self {
        Self { lenient: false }
    }

    /// Validate a raw skill and return a validated skill
    pub fn validate(&self, skill: &RawSkill) -> Result<ValidatedSkill> {
        // Validate name
        let name = self.validate_name(&skill.name)?;

        // Validate description
        let description = self.validate_description(skill.description.as_deref(), &skill.name)?;

        // Validate tools
        let tools = skill
            .tools
            .iter()
            .map(|t| self.validate_tool(t))
            .collect::<Result<Vec<_>>>()?;

        Ok(ValidatedSkill {
            name,
            description,
            source: skill.source.clone(),
            runtime: skill.runtime.clone(),
            tools,
            skill_md_content: skill.skill_md_content.clone(),
        })
    }

    /// Validate and normalize a skill name
    fn validate_name(&self, name: &str) -> Result<String> {
        let normalized = name.to_lowercase().replace(' ', "-");

        // Check length
        if normalized.len() > MAX_NAME_LENGTH {
            if self.lenient {
                tracing::warn!(
                    name = %name,
                    max = MAX_NAME_LENGTH,
                    "Skill name exceeds max length, truncating"
                );
                return Ok(normalized[..MAX_NAME_LENGTH].to_string());
            } else {
                anyhow::bail!(
                    "Skill name '{}' exceeds maximum length of {} characters",
                    name,
                    MAX_NAME_LENGTH
                );
            }
        }

        // Check characters (alphanumeric and hyphens only)
        if !normalized
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            if self.lenient {
                let cleaned: String = normalized
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
                    .collect();
                tracing::warn!(
                    original = %name,
                    cleaned = %cleaned,
                    "Skill name contains invalid characters, cleaning"
                );
                return Ok(cleaned);
            } else {
                anyhow::bail!(
                    "Skill name '{}' contains invalid characters. Use only lowercase letters, numbers, and hyphens.",
                    name
                );
            }
        }

        Ok(normalized)
    }

    /// Validate and normalize a description
    fn validate_description(&self, description: Option<&str>, skill_name: &str) -> Result<String> {
        let desc = description.unwrap_or_else(|| {
            tracing::warn!(skill = %skill_name, "No description provided, using default");
            "Skill for automation tasks"
        });

        if desc.len() > MAX_DESCRIPTION_LENGTH {
            if self.lenient {
                tracing::warn!(
                    skill = %skill_name,
                    max = MAX_DESCRIPTION_LENGTH,
                    "Description exceeds max length, truncating"
                );
                // Truncate at word boundary if possible
                let truncated = truncate_at_word_boundary(desc, MAX_DESCRIPTION_LENGTH - 3);
                return Ok(format!("{}...", truncated));
            } else {
                anyhow::bail!(
                    "Description for '{}' exceeds maximum length of {} characters",
                    skill_name,
                    MAX_DESCRIPTION_LENGTH
                );
            }
        }

        Ok(desc.to_string())
    }

    /// Validate a tool definition
    fn validate_tool(&self, tool: &RawTool) -> Result<ValidatedTool> {
        // Validate tool name (lowercase, alphanumeric with underscores)
        let name = self.validate_tool_name(&tool.name)?;

        // Validate description
        let description = if tool.description.is_empty() {
            format!("{} tool", name)
        } else if tool.description.len() > MAX_DESCRIPTION_LENGTH {
            truncate_at_word_boundary(&tool.description, MAX_DESCRIPTION_LENGTH - 3).to_string()
                + "..."
        } else {
            tool.description.clone()
        };

        // Validate parameters
        let parameters = tool
            .parameters
            .iter()
            .map(|p| ValidatedParameter {
                name: p.name.clone(),
                param_type: p.param_type.clone(),
                description: if p.description.is_empty() {
                    format!("{} parameter", p.name)
                } else {
                    p.description.clone()
                },
                required: p.required,
                default_value: p.default_value.clone(),
            })
            .collect();

        Ok(ValidatedTool {
            name,
            description,
            parameters,
            streaming: tool.streaming,
        })
    }

    /// Validate a tool name
    fn validate_tool_name(&self, name: &str) -> Result<String> {
        let normalized = name.to_lowercase().replace('-', "_");

        // Check characters
        if !normalized
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            if self.lenient {
                let cleaned: String = normalized
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                tracing::warn!(
                    original = %name,
                    cleaned = %cleaned,
                    "Tool name contains invalid characters, cleaning"
                );
                return Ok(cleaned);
            } else {
                anyhow::bail!(
                    "Tool name '{}' contains invalid characters. Use only lowercase letters, numbers, and underscores.",
                    name
                );
            }
        }

        Ok(normalized)
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

/// Truncate a string at a word boundary
fn truncate_at_word_boundary(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        return s;
    }

    // Find the last space before max_len
    let truncated = &s[..max_len];
    match truncated.rfind(' ') {
        Some(pos) if pos > max_len / 2 => &s[..pos],
        _ => truncated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::claude_bridge::types::{RawToolParameter, SkillRuntimeType};

    // === Skill Name Validation Tests ===

    #[test]
    fn test_validate_name_simple() {
        let validator = Validator::new();
        assert_eq!(validator.validate_name("kubernetes").unwrap(), "kubernetes");
    }

    #[test]
    fn test_validate_name_uppercase() {
        let validator = Validator::new();
        assert_eq!(validator.validate_name("Kubernetes").unwrap(), "kubernetes");
    }

    #[test]
    fn test_validate_name_with_spaces() {
        let validator = Validator::new();
        assert_eq!(
            validator.validate_name("my skill").unwrap(),
            "my-skill"
        );
    }

    #[test]
    fn test_validate_name_with_hyphens() {
        let validator = Validator::new();
        assert_eq!(validator.validate_name("my-skill").unwrap(), "my-skill");
        assert_eq!(validator.validate_name("skill-123").unwrap(), "skill-123");
    }

    #[test]
    fn test_validate_name_mixed_case() {
        let validator = Validator::new();
        assert_eq!(validator.validate_name("MySkill").unwrap(), "myskill");
        // Underscores are filtered out
        assert_eq!(validator.validate_name("MY_SKILL").unwrap(), "myskill");
    }

    #[test]
    fn test_validate_name_with_numbers() {
        let validator = Validator::new();
        assert_eq!(validator.validate_name("skill123").unwrap(), "skill123");
        assert_eq!(validator.validate_name("123skill").unwrap(), "123skill");
    }

    #[test]
    fn test_validate_name_with_underscores() {
        let validator = Validator::new();
        // Underscores are filtered out (not alphanumeric or hyphen)
        assert_eq!(validator.validate_name("my_skill").unwrap(), "myskill");
    }

    #[test]
    fn test_validate_name_with_special_characters() {
        let validator = Validator::new();
        // Special characters are filtered out in lenient mode
        let result = validator.validate_name("my@skill!").unwrap();
        assert_eq!(result, "myskill");

        let result = validator.validate_name("skill#name$test").unwrap();
        assert_eq!(result, "skillnametest");
    }

    #[test]
    fn test_validate_name_too_long_lenient() {
        let validator = Validator::new();
        let long_name = "a".repeat(100);
        let result = validator.validate_name(&long_name).unwrap();
        assert_eq!(result.len(), MAX_NAME_LENGTH);
    }

    #[test]
    fn test_validate_name_too_long_strict() {
        let strict = Validator::strict();
        let long_name = "a".repeat(100);
        let result = strict.validate_name(&long_name);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("exceeds maximum length"));
    }

    #[test]
    fn test_validate_name_exactly_max_length() {
        let validator = Validator::new();
        let exact_name = "a".repeat(MAX_NAME_LENGTH);
        let result = validator.validate_name(&exact_name).unwrap();
        assert_eq!(result.len(), MAX_NAME_LENGTH);
    }

    #[test]
    fn test_validate_name_strict_invalid_characters() {
        let strict = Validator::strict();
        let result = strict.validate_name("my@skill");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid characters"));
    }

    #[test]
    fn test_validate_name_empty() {
        let validator = Validator::new();
        let result = validator.validate_name("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_validate_name_unicode() {
        let validator = Validator::new();
        // Unicode characters should be filtered out
        let result = validator.validate_name("skill🚀test").unwrap();
        assert!(!result.contains("🚀"));
    }

    // === Description Validation Tests ===

    #[test]
    fn test_validate_description_simple() {
        let validator = Validator::new();
        let result = validator.validate_description(Some("A simple skill"), "test").unwrap();
        assert_eq!(result, "A simple skill");
    }

    #[test]
    fn test_validate_description_none() {
        let validator = Validator::new();
        let result = validator.validate_description(None, "test").unwrap();
        assert_eq!(result, "Skill for automation tasks");
    }

    #[test]
    fn test_validate_description_empty() {
        let validator = Validator::new();
        let result = validator.validate_description(Some(""), "test").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_validate_description_too_long_lenient() {
        let validator = Validator::new();
        let long_desc = "a".repeat(1500);
        let result = validator.validate_description(Some(&long_desc), "test").unwrap();
        assert!(result.len() <= MAX_DESCRIPTION_LENGTH);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_validate_description_too_long_strict() {
        let strict = Validator::strict();
        let long_desc = "a".repeat(1500);
        let result = strict.validate_description(Some(&long_desc), "test");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("exceeds maximum length"));
    }

    #[test]
    fn test_validate_description_exactly_max_length() {
        let validator = Validator::new();
        let exact_desc = "a".repeat(MAX_DESCRIPTION_LENGTH);
        let result = validator.validate_description(Some(&exact_desc), "test").unwrap();
        assert_eq!(result.len(), MAX_DESCRIPTION_LENGTH);
    }

    #[test]
    fn test_validate_description_with_special_characters() {
        let validator = Validator::new();
        let desc = "Description with 'quotes' and \"double\" and <>&";
        let result = validator.validate_description(Some(desc), "test").unwrap();
        assert_eq!(result, desc);
    }

    #[test]
    fn test_validate_description_with_newlines() {
        let validator = Validator::new();
        let desc = "Line one\nLine two\nLine three";
        let result = validator.validate_description(Some(desc), "test").unwrap();
        assert_eq!(result, desc);
    }

    // === Tool Name Validation Tests ===

    #[test]
    fn test_validate_tool_name_simple() {
        let validator = Validator::new();
        assert_eq!(validator.validate_tool_name("get").unwrap(), "get");
    }

    #[test]
    fn test_validate_tool_name_with_underscores() {
        let validator = Validator::new();
        assert_eq!(validator.validate_tool_name("get_pods").unwrap(), "get_pods");
    }

    #[test]
    fn test_validate_tool_name_with_hyphens() {
        let validator = Validator::new();
        // Hyphens are converted to underscores for tool names
        assert_eq!(validator.validate_tool_name("get-pods").unwrap(), "get_pods");
    }

    #[test]
    fn test_validate_tool_name_uppercase() {
        let validator = Validator::new();
        assert_eq!(validator.validate_tool_name("GetPods").unwrap(), "getpods");
    }

    #[test]
    fn test_validate_tool_name_with_numbers() {
        let validator = Validator::new();
        assert_eq!(validator.validate_tool_name("tool123").unwrap(), "tool123");
    }

    #[test]
    fn test_validate_tool_name_special_characters_lenient() {
        let validator = Validator::new();
        let result = validator.validate_tool_name("get@pods!").unwrap();
        assert_eq!(result, "getpods");
    }

    #[test]
    fn test_validate_tool_name_special_characters_strict() {
        let strict = Validator::strict();
        let result = strict.validate_tool_name("get@pods");
        assert!(result.is_err());
    }

    // === Tool Validation Tests ===

    #[test]
    fn test_validate_tool_basic() {
        let validator = Validator::new();
        let tool = RawTool {
            name: "get".to_string(),
            description: "Get resources".to_string(),
            parameters: vec![],
            streaming: false,
        };
        let validated = validator.validate_tool(&tool).unwrap();
        assert_eq!(validated.name, "get");
        assert_eq!(validated.description, "Get resources");
        assert_eq!(validated.parameters.len(), 0);
    }

    #[test]
    fn test_validate_tool_with_parameters() {
        let validator = Validator::new();
        let tool = RawTool {
            name: "get".to_string(),
            description: "Get resources".to_string(),
            parameters: vec![
                RawToolParameter {
                    name: "resource".to_string(),
                    param_type: "string".to_string(),
                    description: "Resource type".to_string(),
                    required: true,
                    default_value: None,
                },
                RawToolParameter {
                    name: "namespace".to_string(),
                    param_type: "string".to_string(),
                    description: "".to_string(), // Empty description
                    required: false,
                    default_value: Some("default".to_string()),
                },
            ],
            streaming: false,
        };
        let validated = validator.validate_tool(&tool).unwrap();
        assert_eq!(validated.parameters.len(), 2);
        assert_eq!(validated.parameters[0].name, "resource");
        assert_eq!(validated.parameters[0].required, true);
        // Empty description should be filled
        assert_eq!(validated.parameters[1].description, "namespace parameter");
    }

    #[test]
    fn test_validate_tool_empty_description() {
        let validator = Validator::new();
        let tool = RawTool {
            name: "get".to_string(),
            description: "".to_string(),
            parameters: vec![],
            streaming: false,
        };
        let validated = validator.validate_tool(&tool).unwrap();
        assert_eq!(validated.description, "get tool");
    }

    #[test]
    fn test_validate_tool_long_description() {
        let validator = Validator::new();
        let long_desc = "a".repeat(1500);
        let tool = RawTool {
            name: "get".to_string(),
            description: long_desc,
            parameters: vec![],
            streaming: false,
        };
        let validated = validator.validate_tool(&tool).unwrap();
        assert!(validated.description.len() <= MAX_DESCRIPTION_LENGTH);
        assert!(validated.description.ends_with("..."));
    }

    #[test]
    fn test_validate_tool_streaming() {
        let validator = Validator::new();
        let tool = RawTool {
            name: "stream".to_string(),
            description: "Streaming tool".to_string(),
            parameters: vec![],
            streaming: true,
        };
        let validated = validator.validate_tool(&tool).unwrap();
        assert_eq!(validated.streaming, true);
    }

    // === Full Skill Validation Tests ===

    #[test]
    fn test_validate_skill_complete() {
        let validator = Validator::new();
        let skill = RawSkill {
            name: "kubernetes".to_string(),
            description: Some("Kubernetes management".to_string()),
            source: "./kubernetes".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![
                RawTool {
                    name: "get-pods".to_string(),
                    description: "Get pods".to_string(),
                    parameters: vec![],
                    streaming: false,
                }
            ],
            skill_md_content: None,
        };

        let validated = validator.validate(&skill).unwrap();
        assert_eq!(validated.name, "kubernetes");
        assert_eq!(validated.description, "Kubernetes management");
        assert_eq!(validated.tools.len(), 1);
        assert_eq!(validated.tools[0].name, "get_pods");
    }

    #[test]
    fn test_validate_skill_no_description() {
        let validator = Validator::new();
        let skill = RawSkill {
            name: "test".to_string(),
            description: None,
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Native,
            tools: vec![],
            skill_md_content: None,
        };

        let validated = validator.validate(&skill).unwrap();
        assert_eq!(validated.description, "Skill for automation tasks");
    }

    #[test]
    fn test_validate_skill_invalid_name_lenient() {
        let validator = Validator::new();
        let skill = RawSkill {
            name: "My@Skill!".to_string(),
            description: Some("Test".to_string()),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![],
            skill_md_content: None,
        };

        let validated = validator.validate(&skill).unwrap();
        assert_eq!(validated.name, "myskill");
    }

    #[test]
    fn test_validate_skill_multiple_tools() {
        let validator = Validator::new();
        let skill = RawSkill {
            name: "test".to_string(),
            description: Some("Test skill".to_string()),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Docker,
            tools: vec![
                RawTool {
                    name: "tool1".to_string(),
                    description: "First tool".to_string(),
                    parameters: vec![],
                    streaming: false,
                },
                RawTool {
                    name: "tool2".to_string(),
                    description: "Second tool".to_string(),
                    parameters: vec![],
                    streaming: false,
                },
                RawTool {
                    name: "tool3".to_string(),
                    description: "Third tool".to_string(),
                    parameters: vec![],
                    streaming: true,
                },
            ],
            skill_md_content: Some("# Skill content".to_string()),
        };

        let validated = validator.validate(&skill).unwrap();
        assert_eq!(validated.tools.len(), 3);
        assert_eq!(validated.skill_md_content, Some("# Skill content".to_string()));
    }

    // === Truncate Helper Tests ===

    #[test]
    fn test_truncate_at_word_boundary() {
        assert_eq!(truncate_at_word_boundary("hello world", 8), "hello");
        assert_eq!(truncate_at_word_boundary("hello", 10), "hello");
        assert_eq!(truncate_at_word_boundary("helloworld", 5), "hello");
    }

    #[test]
    fn test_truncate_at_word_boundary_long_word() {
        let text = "supercalifragilisticexpialidocious is a long word";
        let result = truncate_at_word_boundary(text, 20);
        // Should truncate at character if no space found near boundary
        assert!(result.len() <= 20);
    }

    #[test]
    fn test_truncate_at_word_boundary_multiple_spaces() {
        let text = "one two three four five";
        let result = truncate_at_word_boundary(text, 15);
        assert_eq!(result, "one two three");
    }

    #[test]
    fn test_truncate_at_word_boundary_exact() {
        let text = "hello world";
        let result = truncate_at_word_boundary(text, 11);
        assert_eq!(result, "hello world");
    }

    // === Lenient vs Strict Mode Tests ===

    #[test]
    fn test_lenient_mode_cleans_invalid() {
        let validator = Validator::new();
        assert!(validator.lenient);

        // Invalid characters are cleaned
        let result = validator.validate_name("my@skill!").unwrap();
        assert_eq!(result, "myskill");

        // Long names are truncated
        let long_name = "a".repeat(100);
        let result = validator.validate_name(&long_name).unwrap();
        assert_eq!(result.len(), MAX_NAME_LENGTH);
    }

    #[test]
    fn test_strict_mode_errors_on_invalid() {
        let validator = Validator::strict();
        assert!(!validator.lenient);

        // Invalid characters cause error
        let result = validator.validate_name("my@skill!");
        assert!(result.is_err());

        // Long names cause error
        let long_name = "a".repeat(100);
        let result = validator.validate_name(&long_name);
        assert!(result.is_err());
    }

    // === Edge Cases ===

    #[test]
    fn test_validate_name_all_special_characters() {
        let validator = Validator::new();
        let result = validator.validate_name("@#$%^&*()").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_validate_description_unicode() {
        let validator = Validator::new();
        let desc = "Skill with emoji 🚀 and unicode €";
        let result = validator.validate_description(Some(desc), "test").unwrap();
        assert_eq!(result, desc);
    }

    #[test]
    fn test_validator_default() {
        let validator = Validator::default();
        assert!(validator.lenient);
    }
}
