//! Claude Agent Skills Specification Validator
//!
//! This module validates generated Claude Agent Skills against Anthropic's
//! official specification, ensuring compliance with:
//!
//! - Naming conventions (lowercase alphanumeric + hyphens, max 64 chars)
//! - YAML frontmatter requirements (name, description fields)
//! - File structure (SKILL.md, TOOLS.md, scripts/)
//! - Script requirements (executable, proper shebang, skill run command)
//! - Markdown structure (required sections)

#![allow(deprecated)] // cargo_bin is deprecated but still functional
#![allow(dead_code)] // Validator fields and methods are reserved for future tests
#![allow(clippy::new_without_default)] // Validator doesn't need Default impl
//!
//! # Usage
//!
//! ```rust,no_run
//! use spec_validator::SpecValidator;
//! use std::path::Path;
//!
//! let validator = SpecValidator::new();
//! let skill_dir = Path::new("~/.claude/skills/kubernetes");
//!
//! match validator.validate_skill_directory(skill_dir) {
//!     Ok(()) => println!("Skill is compliant"),
//!     Err(errors) => {
//!         for error in errors {
//!             eprintln!("  - {}", error);
//!         }
//!     }
//! }
//! ```

use regex::Regex;
use serde_yaml::Value;
use std::path::Path;

/// Maximum length for skill names (per Claude Agent Skills spec)
const MAX_NAME_LENGTH: usize = 64;

/// Maximum length for skill descriptions (per Claude Agent Skills spec)
const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// Validates generated Claude Agent Skills against specification
///
/// This validator performs comprehensive checks to ensure generated skills
/// comply with Anthropic's Claude Agent Skills specification including
/// naming conventions, YAML frontmatter, file structure, and script requirements.
pub struct SpecValidator {
    /// Regex for valid skill names: lowercase alphanumeric + hyphens
    skill_name_regex: Regex,
    /// Regex for valid tool names: lowercase alphanumeric + underscores
    tool_name_regex: Regex,
}

impl SpecValidator {
    /// Create a new specification validator
    pub fn new() -> Self {
        Self {
            // Skill names: lowercase letters, numbers, hyphens only, 1-64 chars
            skill_name_regex: Regex::new(r"^[a-z0-9-]{1,64}$").unwrap(),
            // Tool names: lowercase letters, numbers, underscores only, 1-64 chars
            tool_name_regex: Regex::new(r"^[a-z0-9_]{1,64}$").unwrap(),
        }
    }

    /// Validate an entire skill directory structure
    ///
    /// Performs all validation checks:
    /// - File structure (SKILL.md, TOOLS.md)
    /// - YAML frontmatter in SKILL.md
    /// - Markdown sections in SKILL.md
    /// - Script validation (if scripts/ exists)
    ///
    /// # Returns
    ///
    /// - `Ok(())` if skill is fully compliant
    /// - `Err(Vec<String>)` with list of all validation errors
    pub fn validate_skill_directory(&self, skill_dir: &Path) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // 1. Check required files exist
        if !skill_dir.exists() {
            errors.push(format!("Skill directory does not exist: {:?}", skill_dir));
            return Err(errors);
        }

        if !skill_dir.join("SKILL.md").exists() {
            errors.push("Missing required file: SKILL.md".to_string());
        }

        if !skill_dir.join("TOOLS.md").exists() {
            errors.push("Missing required file: TOOLS.md".to_string());
        }

        // 2. Validate SKILL.md content
        let skill_md = skill_dir.join("SKILL.md");
        if skill_md.exists() {
            if let Err(e) = self.validate_skill_md(&skill_md) {
                errors.extend(e);
            }
        }

        // 3. Validate scripts directory (if present)
        let scripts_dir = skill_dir.join("scripts");
        if scripts_dir.exists() {
            if let Err(e) = self.validate_scripts(&scripts_dir) {
                errors.extend(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate SKILL.md file
    ///
    /// Checks:
    /// - YAML frontmatter exists and is valid
    /// - Required fields (name, description)
    /// - Field constraints (lengths, format)
    /// - Required markdown sections
    fn validate_skill_md(&self, skill_md: &Path) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        let content = match std::fs::read_to_string(skill_md) {
            Ok(c) => c,
            Err(e) => return Err(vec![format!("Cannot read SKILL.md: {}", e)]),
        };

        // 1. Extract and parse YAML frontmatter
        let yaml_frontmatter = match self.extract_yaml_frontmatter(&content) {
            Some(y) => y,
            None => {
                errors.push("No YAML frontmatter found (must start with ---)".to_string());
                return Err(errors);
            }
        };

        let yaml: Value = match serde_yaml::from_str(&yaml_frontmatter) {
            Ok(y) => y,
            Err(e) => {
                errors.push(format!("Invalid YAML frontmatter: {}", e));
                return Err(errors);
            }
        };

        // 2. Validate 'name' field
        match yaml["name"].as_str() {
            None => errors.push("Missing required field 'name' in YAML frontmatter".to_string()),
            Some(name) => {
                if !self.skill_name_regex.is_match(name) {
                    errors.push(format!(
                        "Invalid skill name '{}': must be lowercase alphanumeric + hyphens, max {} chars",
                        name, MAX_NAME_LENGTH
                    ));
                }
            }
        }

        // 3. Validate 'description' field
        match yaml["description"].as_str() {
            None => errors.push("Missing required field 'description' in YAML frontmatter".to_string()),
            Some(desc) => {
                if desc.is_empty() {
                    errors.push("Description cannot be empty".to_string());
                }
                if desc.len() > MAX_DESCRIPTION_LENGTH {
                    errors.push(format!(
                        "Description too long: {} chars (max {})",
                        desc.len(),
                        MAX_DESCRIPTION_LENGTH
                    ));
                }
                if desc.contains('<') || desc.contains('>') {
                    errors.push("Description contains XML/HTML tags (< or >)".to_string());
                }
            }
        }

        // 4. Validate markdown structure
        if !content.contains("## When to Use") && !content.contains("## Usage") {
            errors.push("Missing required section: '## When to Use' or '## Usage'".to_string());
        }

        if !content.contains("## Available Tools") && !content.contains("## Tools") {
            errors.push(
                "Missing required section: '## Available Tools' or '## Tools'".to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Extract YAML frontmatter from SKILL.md content
    ///
    /// Looks for content between --- delimiters at the start of the file
    fn extract_yaml_frontmatter(&self, content: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();

        // Check first line is ---
        if lines.first() != Some(&"---") {
            return None;
        }

        // Find closing ---
        let end_idx = lines[1..].iter().position(|&l| l == "---")?;

        // Extract YAML content between delimiters
        Some(lines[1..=end_idx].join("\n"))
    }

    /// Validate scripts directory
    ///
    /// Checks:
    /// - Scripts have .sh extension
    /// - Scripts are executable (on Unix)
    /// - Scripts have proper shebang
    /// - Scripts contain 'skill run' command
    fn validate_scripts(&self, scripts_dir: &Path) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        let entries = match std::fs::read_dir(scripts_dir) {
            Ok(e) => e,
            Err(e) => return Err(vec![format!("Cannot read scripts directory: {}", e)]),
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    errors.push(format!("Error reading directory entry: {}", e));
                    continue;
                }
            };

            let path = entry.path();

            // Only validate .sh files
            if path.extension().and_then(|s| s.to_str()) != Some("sh") {
                continue;
            }

            // Check executable permissions (Unix only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = std::fs::metadata(&path).unwrap();
                let mode = metadata.permissions().mode();
                if mode & 0o111 == 0 {
                    errors.push(format!("Script not executable: {:?}", path.file_name()));
                }
            }

            // Validate script content
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(format!("Cannot read script {:?}: {}", path.file_name(), e));
                    continue;
                }
            };

            // Check for proper shebang
            if !content.starts_with("#!/bin/bash") && !content.starts_with("#!/usr/bin/env bash")
            {
                errors.push(format!(
                    "Script {:?} missing bash shebang (#!/bin/bash or #!/usr/bin/env bash)",
                    path.file_name()
                ));
            }

            // Check for 'skill run' command
            if !content.contains("skill run") {
                errors.push(format!(
                    "Script {:?} doesn't call 'skill run' command",
                    path.file_name()
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate a skill name against naming conventions
    ///
    /// Skill names must be:
    /// - Lowercase letters, numbers, and hyphens only
    /// - 1-64 characters long
    pub fn is_valid_skill_name(&self, name: &str) -> bool {
        self.skill_name_regex.is_match(name)
    }

    /// Validate a tool name against naming conventions
    ///
    /// Tool names must be:
    /// - Lowercase letters, numbers, and underscores only
    /// - 1-64 characters long
    pub fn is_valid_tool_name(&self, name: &str) -> bool {
        self.tool_name_regex.is_match(name)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_valid_skill() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("kubernetes");
        std::fs::create_dir(&skill_dir).unwrap();

        // Create valid SKILL.md
        let skill_md_content = r#"---
name: kubernetes
description: Kubernetes cluster management and operations
---

# Kubernetes Skill

## When to Use

Use this skill for Kubernetes cluster operations.

## Available Tools

### get

Get Kubernetes resources.
"#;
        std::fs::write(skill_dir.join("SKILL.md"), skill_md_content).unwrap();
        std::fs::write(skill_dir.join("TOOLS.md"), "# Tools\n").unwrap();

        let validator = SpecValidator::new();
        let result = validator.validate_skill_directory(&skill_dir);
        assert!(result.is_ok(), "Valid skill should pass: {:?}", result);
    }

    #[test]
    fn test_validate_invalid_skill_name() {
        let validator = SpecValidator::new();

        // Invalid: uppercase
        assert!(!validator.is_valid_skill_name("Kubernetes"));

        // Invalid: underscores
        assert!(!validator.is_valid_skill_name("kube_skill"));

        // Invalid: special characters
        assert!(!validator.is_valid_skill_name("kube@skill"));
        assert!(!validator.is_valid_skill_name("kube.skill"));

        // Invalid: too long
        let long_name = "a".repeat(65);
        assert!(!validator.is_valid_skill_name(&long_name));

        // Invalid: empty
        assert!(!validator.is_valid_skill_name(""));

        // Valid examples
        assert!(validator.is_valid_skill_name("kubernetes"));
        assert!(validator.is_valid_skill_name("kube-skill"));
        assert!(validator.is_valid_skill_name("k8s"));
        assert!(validator.is_valid_skill_name("my-skill-123"));
    }

    #[test]
    fn test_validate_description_length() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("test");
        std::fs::create_dir(&skill_dir).unwrap();

        // Description exceeds 1024 characters
        let long_desc = "a".repeat(1025);
        let skill_md_content = format!(
            r#"---
name: test
description: {}
---

# Test

## When to Use

Test

## Available Tools

Test
"#,
            long_desc
        );

        std::fs::write(skill_dir.join("SKILL.md"), skill_md_content).unwrap();
        std::fs::write(skill_dir.join("TOOLS.md"), "# Tools\n").unwrap();

        let validator = SpecValidator::new();
        let result = validator.validate_skill_directory(&skill_dir);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("too long")),
            "Should detect description length violation: {:?}",
            errors
        );
    }

    #[test]
    fn test_validate_missing_yaml_frontmatter() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("test");
        std::fs::create_dir(&skill_dir).unwrap();

        // SKILL.md without YAML frontmatter
        let skill_md_content = r#"# Test Skill

This skill has no YAML frontmatter.
"#;

        std::fs::write(skill_dir.join("SKILL.md"), skill_md_content).unwrap();
        std::fs::write(skill_dir.join("TOOLS.md"), "# Tools\n").unwrap();

        let validator = SpecValidator::new();
        let result = validator.validate_skill_directory(&skill_dir);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("frontmatter")),
            "Should detect missing frontmatter: {:?}",
            errors
        );
    }

    #[test]
    fn test_validate_missing_required_fields() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("test");
        std::fs::create_dir(&skill_dir).unwrap();

        // YAML frontmatter missing 'description'
        let skill_md_content = r#"---
name: test
---

# Test

## When to Use

Test

## Available Tools

Test
"#;

        std::fs::write(skill_dir.join("SKILL.md"), skill_md_content).unwrap();
        std::fs::write(skill_dir.join("TOOLS.md"), "# Tools\n").unwrap();

        let validator = SpecValidator::new();
        let result = validator.validate_skill_directory(&skill_dir);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("description")),
            "Should detect missing description: {:?}",
            errors
        );
    }

    #[test]
    fn test_validate_html_in_description() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("test");
        std::fs::create_dir(&skill_dir).unwrap();

        let skill_md_content = r#"---
name: test
description: This has <html> tags in it
---

# Test

## When to Use

Test

## Available Tools

Test
"#;

        std::fs::write(skill_dir.join("SKILL.md"), skill_md_content).unwrap();
        std::fs::write(skill_dir.join("TOOLS.md"), "# Tools\n").unwrap();

        let validator = SpecValidator::new();
        let result = validator.validate_skill_directory(&skill_dir);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("XML/HTML")),
            "Should detect HTML tags: {:?}",
            errors
        );
    }

    #[test]
    fn test_validate_missing_sections() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("test");
        std::fs::create_dir(&skill_dir).unwrap();

        // Missing "Available Tools" section
        let skill_md_content = r#"---
name: test
description: Test skill
---

# Test Skill

## When to Use

Use this for testing.
"#;

        std::fs::write(skill_dir.join("SKILL.md"), skill_md_content).unwrap();
        std::fs::write(skill_dir.join("TOOLS.md"), "# Tools\n").unwrap();

        let validator = SpecValidator::new();
        let result = validator.validate_skill_directory(&skill_dir);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("Available Tools")),
            "Should detect missing section: {:?}",
            errors
        );
    }

    #[test]
    fn test_validate_script_permissions() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("test");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::create_dir(skill_dir.join("scripts")).unwrap();

        // Create SKILL.md and TOOLS.md
        let skill_md_content = r#"---
name: test
description: Test skill
---

# Test

## When to Use

Test

## Available Tools

Test
"#;
        std::fs::write(skill_dir.join("SKILL.md"), skill_md_content).unwrap();
        std::fs::write(skill_dir.join("TOOLS.md"), "# Tools\n").unwrap();

        // Create script without executable permissions
        let script_content = r#"#!/bin/bash
skill run test tool --arg value
"#;
        let script_path = skill_dir.join("scripts/test.sh");
        std::fs::write(&script_path, script_content).unwrap();

        // Note: On Unix, newly created files are not executable by default
        #[cfg(unix)]
        {
            let validator = SpecValidator::new();
            let result = validator.validate_skill_directory(&skill_dir);

            assert!(result.is_err());
            let errors = result.unwrap_err();
            assert!(
                errors.iter().any(|e| e.contains("not executable")),
                "Should detect non-executable script: {:?}",
                errors
            );
        }
    }

    #[test]
    fn test_validate_script_missing_shebang() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("test");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::create_dir(skill_dir.join("scripts")).unwrap();

        let skill_md_content = r#"---
name: test
description: Test skill
---

# Test

## When to Use

Test

## Available Tools

Test
"#;
        std::fs::write(skill_dir.join("SKILL.md"), skill_md_content).unwrap();
        std::fs::write(skill_dir.join("TOOLS.md"), "# Tools\n").unwrap();

        // Create script without shebang
        let script_content = r#"skill run test tool --arg value
"#;
        let script_path = skill_dir.join("scripts/test.sh");
        std::fs::write(&script_path, script_content).unwrap();

        // Make it executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms).unwrap();
        }

        let validator = SpecValidator::new();
        let result = validator.validate_skill_directory(&skill_dir);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("shebang")),
            "Should detect missing shebang: {:?}",
            errors
        );
    }
}
