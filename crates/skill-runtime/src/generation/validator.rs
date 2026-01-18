//! Example validator for AI-generated tool examples
//!
//! Validates generated examples against tool parameter schemas and
//! checks for diversity across examples.

use std::collections::HashMap;
use crate::skill_md::{ToolDocumentation, ParameterType};
use super::streaming::GeneratedExample;

/// Result of validating an example
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the example is valid
    pub valid: bool,
    /// Validation errors (if any)
    pub errors: Vec<String>,
    /// Validation warnings (non-fatal)
    pub warnings: Vec<String>,
    /// Adjusted confidence score
    pub confidence: f32,
}

impl ValidationResult {
    /// Create a valid result
    pub fn valid(confidence: f32) -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            confidence,
        }
    }

    /// Create an invalid result
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
            confidence: 0.0,
        }
    }

    /// Add a warning
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }
}

/// Parsed command representation
#[derive(Debug, Clone)]
pub struct ParsedCommand {
    /// Skill name (e.g., "kubernetes")
    pub skill: Option<String>,
    /// Tool name (e.g., "apply")
    pub tool: Option<String>,
    /// Positional arguments
    pub positional: Vec<String>,
    /// Named parameters (--param=value or --param value)
    pub parameters: HashMap<String, String>,
    /// Flags (--flag without value)
    pub flags: Vec<String>,
}

impl ParsedCommand {
    /// Check if a parameter is present (by name)
    pub fn has_param(&self, name: &str) -> bool {
        self.parameters.contains_key(name) || self.flags.contains(&name.to_string())
    }

    /// Get parameter value
    pub fn get_param(&self, name: &str) -> Option<&String> {
        self.parameters.get(name)
    }
}

/// Validator for generated examples
pub struct ExampleValidator {
    /// Minimum diversity score threshold
    pub diversity_threshold: f32,
    /// Strict mode - fail on warnings
    pub strict: bool,
}

impl Default for ExampleValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ExampleValidator {
    /// Create a new validator with default settings
    pub fn new() -> Self {
        Self {
            diversity_threshold: 0.7,
            strict: false,
        }
    }

    /// Create a strict validator
    pub fn strict() -> Self {
        Self {
            diversity_threshold: 0.8,
            strict: true,
        }
    }

    /// Set diversity threshold
    pub fn with_diversity_threshold(mut self, threshold: f32) -> Self {
        self.diversity_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Validate a single example against a tool's documentation
    pub fn validate_example(
        &self,
        example: &GeneratedExample,
        tool: &ToolDocumentation,
    ) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Parse the command
        let parsed = match self.parse_command(&example.command) {
            Ok(p) => p,
            Err(e) => {
                return ValidationResult::invalid(vec![format!("Failed to parse command: {}", e)]);
            }
        };

        // Validate tool name matches (if extractable)
        if let Some(ref tool_name) = parsed.tool {
            let expected_name = &tool.name;
            if !tool_name.eq_ignore_ascii_case(expected_name) &&
               !tool_name.contains(expected_name) &&
               !expected_name.contains(tool_name) {
                warnings.push(format!(
                    "Tool name mismatch: expected '{}', got '{}'",
                    expected_name, tool_name
                ));
            }
        }

        // Validate required parameters are present
        for param in &tool.parameters {
            if param.required && !parsed.has_param(&param.name) {
                // Check for common aliases
                let has_alias = param.name.chars().next()
                    .map(|c| parsed.flags.contains(&c.to_string()))
                    .unwrap_or(false);

                if !has_alias {
                    errors.push(format!("Missing required parameter: {}", param.name));
                }
            }
        }

        // Validate parameter types if possible
        for (name, value) in &parsed.parameters {
            if let Some(param) = tool.parameters.iter().find(|p| p.name == *name) {
                if let Err(e) = self.validate_param_type(value, &param.param_type) {
                    warnings.push(format!("Parameter '{}': {}", name, e));
                }
            }
        }

        // Check for unknown parameters
        for name in parsed.parameters.keys() {
            if !tool.parameters.iter().any(|p| p.name == *name) {
                // Not necessarily an error, could be a valid flag
                warnings.push(format!("Unknown parameter: {}", name));
            }
        }

        // Validate explanation is not empty
        if example.explanation.trim().is_empty() {
            errors.push("Example explanation is empty".to_string());
        }

        // Calculate final validity
        let valid = errors.is_empty() && (!self.strict || warnings.is_empty());

        // Adjust confidence based on warnings
        let confidence = if valid {
            let warning_penalty = 0.1 * warnings.len() as f32;
            (example.confidence - warning_penalty).max(0.1)
        } else {
            0.0
        };

        ValidationResult {
            valid,
            errors,
            warnings,
            confidence,
        }
    }

    /// Validate multiple examples and return batch results
    pub fn validate_batch(
        &self,
        examples: &[GeneratedExample],
        tool: &ToolDocumentation,
    ) -> Vec<ValidationResult> {
        examples
            .iter()
            .map(|e| self.validate_example(e, tool))
            .collect()
    }

    /// Calculate diversity score for a set of examples
    /// Returns a score from 0.0 (all identical) to 1.0 (completely diverse)
    pub fn calculate_diversity(&self, examples: &[GeneratedExample]) -> f32 {
        if examples.len() < 2 {
            return 1.0; // Single example is "diverse" by default
        }

        // Use simple command similarity as a proxy for diversity
        let mut total_similarity = 0.0;
        let mut pairs = 0;

        for i in 0..examples.len() {
            for j in (i + 1)..examples.len() {
                let similarity = self.command_similarity(&examples[i].command, &examples[j].command);
                total_similarity += similarity;
                pairs += 1;
            }
        }

        if pairs == 0 {
            return 1.0;
        }

        // Average similarity, converted to diversity (1 - similarity)
        1.0 - (total_similarity / pairs as f32)
    }

    /// Check if diversity meets threshold
    pub fn check_diversity(&self, examples: &[GeneratedExample]) -> bool {
        self.calculate_diversity(examples) >= self.diversity_threshold
    }

    /// Calculate simple command similarity using Jaccard index
    fn command_similarity(&self, cmd1: &str, cmd2: &str) -> f32 {
        let tokens1: std::collections::HashSet<_> = cmd1.split_whitespace().collect();
        let tokens2: std::collections::HashSet<_> = cmd2.split_whitespace().collect();

        let intersection = tokens1.intersection(&tokens2).count();
        let union = tokens1.union(&tokens2).count();

        if union == 0 {
            return 1.0;
        }

        intersection as f32 / union as f32
    }

    /// Parse a command string into components
    ///
    /// Supports formats:
    /// - `skill run tool:name --param=value`
    /// - `skill run skill:tool param=value`
    /// - `tool --flag --param value`
    pub fn parse_command(&self, command: &str) -> Result<ParsedCommand, String> {
        let mut parsed = ParsedCommand {
            skill: None,
            tool: None,
            positional: Vec::new(),
            parameters: HashMap::new(),
            flags: Vec::new(),
        };

        let tokens: Vec<&str> = command.split_whitespace().collect();

        if tokens.is_empty() {
            return Err("Empty command".to_string());
        }

        let mut i = 0;

        // Skip "skill run" prefix if present
        if tokens.get(0) == Some(&"skill") {
            i += 1;
            if tokens.get(i) == Some(&"run") {
                i += 1;
            }
        }

        // Parse tool identifier (skill:tool or just tool)
        if let Some(tool_part) = tokens.get(i) {
            if tool_part.contains(':') {
                let parts: Vec<&str> = tool_part.splitn(2, ':').collect();
                parsed.skill = Some(parts[0].to_string());
                parsed.tool = Some(parts.get(1).unwrap_or(&"").to_string());
            } else if !tool_part.starts_with('-') {
                parsed.tool = Some(tool_part.to_string());
            }
            i += 1;
        }

        // Parse remaining arguments
        while i < tokens.len() {
            let token = tokens[i];

            if token.starts_with("--") {
                // Long parameter
                let param = &token[2..];
                if let Some((name, value)) = param.split_once('=') {
                    parsed.parameters.insert(name.to_string(), value.to_string());
                } else if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                    // Next token is the value
                    parsed.parameters.insert(param.to_string(), tokens[i + 1].to_string());
                    i += 1;
                } else {
                    // Flag without value
                    parsed.flags.push(param.to_string());
                }
            } else if token.starts_with('-') && token.len() == 2 {
                // Short flag
                let flag = &token[1..];
                if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                    parsed.parameters.insert(flag.to_string(), tokens[i + 1].to_string());
                    i += 1;
                } else {
                    parsed.flags.push(flag.to_string());
                }
            } else if token.contains('=') {
                // key=value format (without --)
                if let Some((name, value)) = token.split_once('=') {
                    parsed.parameters.insert(name.to_string(), value.to_string());
                }
            } else {
                // Positional argument
                parsed.positional.push(token.to_string());
            }

            i += 1;
        }

        Ok(parsed)
    }

    /// Validate a parameter value against a ParameterType
    fn validate_param_type(&self, value: &str, param_type: &ParameterType) -> Result<(), String> {
        match param_type {
            ParameterType::String => Ok(()),
            ParameterType::Integer => {
                value.parse::<i64>()
                    .map(|_| ())
                    .map_err(|_| format!("expected integer, got '{}'", value))
            }
            ParameterType::Number => {
                value.parse::<f64>()
                    .map(|_| ())
                    .map_err(|_| format!("expected number, got '{}'", value))
            }
            ParameterType::Boolean => {
                match value.to_lowercase().as_str() {
                    "true" | "false" | "yes" | "no" | "1" | "0" => Ok(()),
                    _ => Err(format!("expected boolean, got '{}'", value)),
                }
            }
            ParameterType::Array => Ok(()), // Can't easily validate array syntax
            ParameterType::Object => Ok(()), // Can't easily validate object syntax
        }
    }

    /// Validate a parameter value against a type hint string (for tests)
    #[allow(dead_code)]
    fn validate_type(&self, value: &str, type_hint: &str) -> Result<(), String> {
        let type_lower = type_hint.to_lowercase();

        match type_lower.as_str() {
            "int" | "integer" | "number" => {
                value.parse::<i64>()
                    .map(|_| ())
                    .map_err(|_| format!("expected integer, got '{}'", value))
            }
            "float" | "decimal" => {
                value.parse::<f64>()
                    .map(|_| ())
                    .map_err(|_| format!("expected number, got '{}'", value))
            }
            "bool" | "boolean" => {
                match value.to_lowercase().as_str() {
                    "true" | "false" | "yes" | "no" | "1" | "0" => Ok(()),
                    _ => Err(format!("expected boolean, got '{}'", value)),
                }
            }
            "path" | "file" => {
                // Basic path validation
                if value.is_empty() {
                    Err("empty path".to_string())
                } else {
                    Ok(())
                }
            }
            "url" => {
                if value.starts_with("http://") || value.starts_with("https://") {
                    Ok(())
                } else {
                    Err(format!("expected URL, got '{}'", value))
                }
            }
            _ => Ok(()), // Unknown types pass
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tool() -> ToolDocumentation {
        ToolDocumentation {
            name: "apply".to_string(),
            description: "Apply a Kubernetes manifest".to_string(),
            usage: None,
            parameters: vec![
                ParameterDoc {
                    name: "file".to_string(),
                    param_type: ParameterType::String,
                    description: "Path to manifest file".to_string(),
                    required: true,
                    default: None,
                    allowed_values: vec![],
                },
                ParameterDoc {
                    name: "namespace".to_string(),
                    param_type: ParameterType::String,
                    description: "Target namespace".to_string(),
                    required: false,
                    default: Some("default".to_string()),
                    allowed_values: vec![],
                },
                ParameterDoc {
                    name: "dry-run".to_string(),
                    param_type: ParameterType::Boolean,
                    description: "Perform dry run".to_string(),
                    required: false,
                    default: None,
                    allowed_values: vec![],
                },
            ],
            examples: vec![],
        }
    }

    #[test]
    fn test_parse_command_basic() {
        let validator = ExampleValidator::new();
        let parsed = validator.parse_command("skill run k8s:apply --file=deploy.yaml").unwrap();

        assert_eq!(parsed.skill, Some("k8s".to_string()));
        assert_eq!(parsed.tool, Some("apply".to_string()));
        assert_eq!(parsed.get_param("file"), Some(&"deploy.yaml".to_string()));
    }

    #[test]
    fn test_parse_command_separate_value() {
        let validator = ExampleValidator::new();
        let parsed = validator.parse_command("skill run apply --file deploy.yaml --namespace prod").unwrap();

        assert_eq!(parsed.tool, Some("apply".to_string()));
        assert_eq!(parsed.get_param("file"), Some(&"deploy.yaml".to_string()));
        assert_eq!(parsed.get_param("namespace"), Some(&"prod".to_string()));
    }

    #[test]
    fn test_parse_command_flags() {
        let validator = ExampleValidator::new();
        let parsed = validator.parse_command("apply --dry-run --file=test.yaml").unwrap();

        assert!(parsed.flags.contains(&"dry-run".to_string()));
        assert!(parsed.has_param("dry-run"));
    }

    #[test]
    fn test_parse_command_key_value() {
        let validator = ExampleValidator::new();
        let parsed = validator.parse_command("skill run tool namespace=default file=app.yaml").unwrap();

        assert_eq!(parsed.get_param("namespace"), Some(&"default".to_string()));
        assert_eq!(parsed.get_param("file"), Some(&"app.yaml".to_string()));
    }

    #[test]
    fn test_validate_example_valid() {
        let validator = ExampleValidator::new();
        let tool = create_test_tool();

        let example = GeneratedExample {
            command: "skill run k8s:apply --file=deploy.yaml".to_string(),
            explanation: "Apply deployment manifest".to_string(),
            confidence: 0.9,
            validated: false,
            category: None,
            parameters: None,
        };

        let result = validator.validate_example(&example, &tool);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_example_missing_required() {
        let validator = ExampleValidator::new();
        let tool = create_test_tool();

        let example = GeneratedExample {
            command: "skill run k8s:apply --namespace=prod".to_string(),
            explanation: "Apply to prod namespace".to_string(),
            confidence: 0.8,
            validated: false,
            category: None,
            parameters: None,
        };

        let result = validator.validate_example(&example, &tool);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("file")));
    }

    #[test]
    fn test_validate_example_empty_explanation() {
        let validator = ExampleValidator::new();
        let tool = create_test_tool();

        let example = GeneratedExample {
            command: "skill run k8s:apply --file=test.yaml".to_string(),
            explanation: "  ".to_string(),
            confidence: 0.9,
            validated: false,
            category: None,
            parameters: None,
        };

        let result = validator.validate_example(&example, &tool);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("explanation")));
    }

    #[test]
    fn test_diversity_identical() {
        let validator = ExampleValidator::new();
        let examples = vec![
            GeneratedExample::new("skill run apply --file=a.yaml", "Apply a"),
            GeneratedExample::new("skill run apply --file=a.yaml", "Apply a"),
        ];

        let diversity = validator.calculate_diversity(&examples);
        assert!(diversity < 0.5); // Low diversity for identical commands
    }

    #[test]
    fn test_diversity_different() {
        let validator = ExampleValidator::new();
        let examples = vec![
            GeneratedExample::new("skill run apply --file=deploy.yaml", "Deploy app"),
            GeneratedExample::new("skill run delete --namespace=prod --all", "Delete all in prod"),
            GeneratedExample::new("skill run get pods --output=json", "List pods as JSON"),
        ];

        let diversity = validator.calculate_diversity(&examples);
        assert!(diversity > 0.5); // High diversity for different commands
    }

    #[test]
    fn test_validate_type_integer() {
        let validator = ExampleValidator::new();

        assert!(validator.validate_type("123", "integer").is_ok());
        assert!(validator.validate_type("-42", "int").is_ok());
        assert!(validator.validate_type("abc", "integer").is_err());
    }

    #[test]
    fn test_validate_type_boolean() {
        let validator = ExampleValidator::new();

        assert!(validator.validate_type("true", "boolean").is_ok());
        assert!(validator.validate_type("false", "bool").is_ok());
        assert!(validator.validate_type("yes", "boolean").is_ok());
        assert!(validator.validate_type("maybe", "boolean").is_err());
    }

    #[test]
    fn test_validate_type_url() {
        let validator = ExampleValidator::new();

        assert!(validator.validate_type("https://example.com", "url").is_ok());
        assert!(validator.validate_type("http://localhost:8080", "url").is_ok());
        assert!(validator.validate_type("not-a-url", "url").is_err());
    }

    #[test]
    fn test_batch_validation() {
        let validator = ExampleValidator::new();
        let tool = create_test_tool();

        let examples = vec![
            GeneratedExample::new("skill run apply --file=a.yaml", "Apply a"),
            GeneratedExample::new("skill run apply --namespace=prod", "Missing file"),
        ];

        let results = validator.validate_batch(&examples, &tool);
        assert_eq!(results.len(), 2);
        assert!(results[0].valid);
        assert!(!results[1].valid);
    }
}
