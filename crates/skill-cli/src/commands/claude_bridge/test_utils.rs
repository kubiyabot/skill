//! Test utilities for Claude Bridge testing
//!
//! This module provides common fixtures, helpers, and mock implementations
//! for testing the Claude Bridge functionality.

#![cfg(test)]

use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Creates a temporary directory for testing
pub fn create_test_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Creates a minimal valid skill manifest for testing
pub fn create_minimal_manifest() -> String {
    r#"[[skill]]
name = "test-skill"
description = "A test skill for unit testing"
version = "1.0.0"

[[skill.tool]]
name = "test-tool"
description = "A test tool"
command = "echo {{message}}"

[[skill.tool.parameter]]
name = "message"
type = "string"
description = "Message to echo"
required = true
"#
    .to_string()
}

/// Creates an invalid skill manifest for negative testing
pub fn create_invalid_manifest() -> String {
    r#"[[skill]]
name = "test-skill"
# Missing required description field
version = "1.0.0"

[[skill.tool]]
name = "test-tool"
# Missing required command field
"#
    .to_string()
}

/// Creates a manifest with multiple skills
pub fn create_multi_skill_manifest() -> String {
    r#"[[skill]]
name = "skill-one"
description = "First test skill"
version = "1.0.0"

[[skill.tool]]
name = "tool-one"
description = "First tool"
command = "echo one"

[[skill]]
name = "skill-two"
description = "Second test skill"
version = "1.0.0"

[[skill.tool]]
name = "tool-two"
description = "Second tool"
command = "echo two"
"#
    .to_string()
}

/// Writes a manifest to a temporary file and returns the path
pub fn write_manifest_to_temp(manifest_content: &str) -> (TempDir, PathBuf) {
    let temp_dir = create_test_dir();
    let manifest_path = temp_dir.path().join("manifest.toml");
    std::fs::write(&manifest_path, manifest_content)
        .expect("Failed to write manifest to temp file");
    (temp_dir, manifest_path)
}

/// Expected YAML frontmatter for minimal test skill
pub fn expected_minimal_frontmatter() -> &'static str {
    r#"---
name: test-skill
description: A test skill for unit testing
version: 1.0.0
---"#
}

/// Expected tool documentation section for minimal test skill
pub fn expected_minimal_tool_section() -> &'static str {
    r#"## test-tool

**Description:** A test tool

**Parameters:**
- `message` (string, required): Message to echo

**Usage:**
```bash
test-tool --message="value"
```"#
}

/// Validates that a generated SKILL.md file contains required sections
pub fn validate_skill_md_structure(content: &str) -> Result<(), String> {
    let required_sections = vec![
        "---",  // YAML frontmatter start
        "name:",
        "description:",
        "# ",  // Skill title
        "## When to Use This Skill",
        "## Quick Reference",
        "## Tools by Category",
    ];

    for section in required_sections {
        if !content.contains(section) {
            return Err(format!("Missing required section: {}", section));
        }
    }

    Ok(())
}

/// Validates that a generated TOOLS.md file contains required sections
pub fn validate_tools_md_structure(content: &str) -> Result<(), String> {
    let required_sections = vec![
        "# Tools Reference",
        "## Tools",
    ];

    for section in required_sections {
        if !content.contains(section) {
            return Err(format!("Missing required section: {}", section));
        }
    }

    Ok(())
}

/// Validates that generated scripts have proper structure
pub fn validate_script_structure(script_content: &str) -> Result<(), String> {
    // Check for shebang
    if !script_content.starts_with("#!/usr/bin/env bash") &&
       !script_content.starts_with("#!/bin/bash") {
        return Err("Script missing shebang".to_string());
    }

    // Check for error handling
    if !script_content.contains("set -e") && !script_content.contains("set -euo pipefail") {
        return Err("Script missing error handling (set -e)".to_string());
    }

    Ok(())
}

/// Creates a test output directory structure
pub fn create_test_output_structure(base: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(base.join("skills"))?;
    std::fs::create_dir_all(base.join("skills/scripts"))?;
    Ok(())
}

/// Mock environment for testing file operations
pub struct MockFileSystem {
    pub temp_dir: TempDir,
    pub skills_dir: PathBuf,
    pub scripts_dir: PathBuf,
}

impl MockFileSystem {
    /// Creates a new mock filesystem with typical Claude skills directory structure
    pub fn new() -> Self {
        let temp_dir = create_test_dir();
        let skills_dir = temp_dir.path().join("skills");
        let scripts_dir = skills_dir.join("scripts");

        std::fs::create_dir_all(&skills_dir).expect("Failed to create skills dir");
        std::fs::create_dir_all(&scripts_dir).expect("Failed to create scripts dir");

        Self {
            temp_dir,
            skills_dir,
            scripts_dir,
        }
    }

    /// Returns the base path for the mock filesystem
    pub fn base_path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Returns the skills directory path
    pub fn skills_path(&self) -> &Path {
        &self.skills_dir
    }

    /// Returns the scripts directory path
    pub fn scripts_path(&self) -> &Path {
        &self.scripts_dir
    }

    /// Writes a file to the skills directory
    pub fn write_skill_file(&self, filename: &str, content: &str) -> std::io::Result<PathBuf> {
        let path = self.skills_dir.join(filename);
        std::fs::write(&path, content)?;
        Ok(path)
    }

    /// Writes a file to the scripts directory
    pub fn write_script_file(&self, filename: &str, content: &str) -> std::io::Result<PathBuf> {
        let path = self.scripts_dir.join(filename);
        std::fs::write(&path, content)?;
        Ok(path)
    }

    /// Reads a file from the skills directory
    pub fn read_skill_file(&self, filename: &str) -> std::io::Result<String> {
        std::fs::read_to_string(self.skills_dir.join(filename))
    }

    /// Reads a file from the scripts directory
    pub fn read_script_file(&self, filename: &str) -> std::io::Result<String> {
        std::fs::read_to_string(self.scripts_dir.join(filename))
    }

    /// Checks if a skill file exists
    pub fn skill_file_exists(&self, filename: &str) -> bool {
        self.skills_dir.join(filename).exists()
    }

    /// Checks if a script file exists
    pub fn script_file_exists(&self, filename: &str) -> bool {
        self.scripts_dir.join(filename).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_dir() {
        let dir = create_test_dir();
        assert!(dir.path().exists());
        assert!(dir.path().is_dir());
    }

    #[test]
    fn test_minimal_manifest_is_valid_toml() {
        let manifest = create_minimal_manifest();
        let parsed: Result<toml::Value, _> = toml::from_str(&manifest);
        assert!(parsed.is_ok(), "Minimal manifest should be valid TOML");
    }

    #[test]
    fn test_multi_skill_manifest_is_valid_toml() {
        let manifest = create_multi_skill_manifest();
        let parsed: Result<toml::Value, _> = toml::from_str(&manifest);
        assert!(parsed.is_ok(), "Multi-skill manifest should be valid TOML");
    }

    #[test]
    fn test_write_manifest_to_temp() {
        let manifest = create_minimal_manifest();
        let (_temp_dir, path) = write_manifest_to_temp(&manifest);
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, manifest);
    }

    #[test]
    fn test_validate_skill_md_structure_valid() {
        let valid_content = r#"---
name: test
description: Test skill
---

# Test Skill

## When to Use This Skill
Some content

## Quick Reference
Some content

## Tools by Category
Some content
"#;
        assert!(validate_skill_md_structure(valid_content).is_ok());
    }

    #[test]
    fn test_validate_skill_md_structure_invalid() {
        let invalid_content = r#"---
name: test
---

# Test Skill
"#;
        assert!(validate_skill_md_structure(invalid_content).is_err());
    }

    #[test]
    fn test_validate_script_structure_valid() {
        let valid_script = r#"#!/usr/bin/env bash
set -euo pipefail

echo "Hello"
"#;
        assert!(validate_script_structure(valid_script).is_ok());
    }

    #[test]
    fn test_validate_script_structure_missing_shebang() {
        let invalid_script = r#"set -euo pipefail
echo "Hello"
"#;
        assert!(validate_script_structure(invalid_script).is_err());
    }

    #[test]
    fn test_mock_filesystem_creation() {
        let fs = MockFileSystem::new();
        assert!(fs.base_path().exists());
        assert!(fs.skills_path().exists());
        assert!(fs.scripts_path().exists());
    }

    #[test]
    fn test_mock_filesystem_write_read() {
        let fs = MockFileSystem::new();
        let content = "test content";

        fs.write_skill_file("test.md", content).unwrap();
        assert!(fs.skill_file_exists("test.md"));

        let read_content = fs.read_skill_file("test.md").unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_mock_filesystem_scripts() {
        let fs = MockFileSystem::new();
        let script = "#!/bin/bash\necho test";

        fs.write_script_file("test.sh", script).unwrap();
        assert!(fs.script_file_exists("test.sh"));

        let read_script = fs.read_script_file("test.sh").unwrap();
        assert_eq!(read_script, script);
    }
}
