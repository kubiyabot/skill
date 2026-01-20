//! Integration Tests for Claude Bridge Skill Generation
//!
//! These tests validate end-to-end skill generation workflows including:
//! - Complete skill generation (all skills)
//! - Single skill filtering
//! - Project-local generation
//! - Force overwrite behavior
//! - No-scripts mode
//! - Dry-run mode
//! - YAML frontmatter validation

#![allow(deprecated)] // cargo_bin is deprecated but still functional
//! - File structure compliance
//! - Specification compliance validation

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// Import spec validator
#[path = "spec_validator.rs"]
mod spec_validator;
use spec_validator::SpecValidator;

/// Test helper: Create a temporary directory with a test manifest
fn create_test_manifest(temp_dir: &Path, manifest_content: &str) -> PathBuf {
    let manifest_path = temp_dir.join(".skill-engine.toml");
    fs::write(&manifest_path, manifest_content).expect("Failed to write test manifest");
    manifest_path
}

/// Test helper: Simple manifest with one skill
fn simple_manifest() -> &'static str {
    r#"
[skills.test-skill]
source = "./test-skill"
runtime = "wasm"
description = "A simple test skill"
"#
}

/// Test helper: Multi-skill manifest
fn multi_skill_manifest() -> &'static str {
    r#"
[skills.kubernetes]
source = "./kubernetes"
runtime = "wasm"
description = "Kubernetes cluster management"

[skills.docker]
source = "./docker"
runtime = "native"
description = "Docker container operations"

[skills.terraform]
source = "./terraform"
runtime = "native"
description = "Infrastructure as Code with Terraform"
"#
}

/// Test helper: Verify skill directory structure
fn verify_skill_structure(skill_dir: &Path, skill_name: &str, expect_scripts: bool) {
    let skill_path = skill_dir.join(skill_name);

    assert!(
        skill_path.exists(),
        "Skill directory {} should exist",
        skill_name
    );

    let skill_md = skill_path.join("SKILL.md");
    assert!(skill_md.exists(), "SKILL.md should exist");
    assert!(
        fs::metadata(&skill_md).unwrap().len() > 0,
        "SKILL.md should not be empty"
    );

    let tools_md = skill_path.join("TOOLS.md");
    assert!(tools_md.exists(), "TOOLS.md should exist");

    if expect_scripts {
        let scripts_dir = skill_path.join("scripts");
        assert!(
            scripts_dir.exists(),
            "scripts/ directory should exist when scripts are enabled"
        );
    } else {
        let scripts_dir = skill_path.join("scripts");
        assert!(
            !scripts_dir.exists(),
            "scripts/ directory should NOT exist when --no-scripts is used"
        );
    }
}

/// Test helper: Parse YAML frontmatter from SKILL.md
fn parse_yaml_frontmatter(skill_md_path: &Path) -> serde_yaml::Value {
    let content = fs::read_to_string(skill_md_path).expect("Failed to read SKILL.md");

    // Find YAML frontmatter between --- delimiters
    let yaml_start = content.find("---\n").expect("No YAML frontmatter start found");
    let yaml_end = content[yaml_start + 4..]
        .find("\n---")
        .expect("No YAML frontmatter end found")
        + yaml_start
        + 4;

    let yaml = &content[yaml_start + 4..yaml_end];
    serde_yaml::from_str(yaml).expect("Failed to parse YAML frontmatter")
}

/// Test helper: Validate YAML frontmatter compliance
fn validate_yaml_frontmatter(yaml: &serde_yaml::Value) {
    // Required fields
    assert!(yaml.get("name").is_some(), "name field is required");
    assert!(
        yaml.get("description").is_some(),
        "description field is required"
    );

    // Field length constraints
    if let Some(name) = yaml["name"].as_str() {
        assert!(
            name.len() <= 64,
            "name must be <= 64 characters, got {}",
            name.len()
        );
    }

    if let Some(description) = yaml["description"].as_str() {
        assert!(
            description.len() <= 1024,
            "description must be <= 1024 characters, got {}",
            description.len()
        );
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary to be built
fn test_generate_all_skills() {
    let temp = TempDir::new().unwrap();
    create_test_manifest(temp.path(), multi_skill_manifest());
    let output_dir = temp.path().join("skills");

    // Generate all skills from manifest (manifest auto-detected in current dir)
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path()) // Set working directory to where manifest is
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Verify all skills were generated
    verify_skill_structure(&output_dir, "kubernetes", true);
    verify_skill_structure(&output_dir, "docker", true);
    verify_skill_structure(&output_dir, "terraform", true);
}

#[test]
#[ignore] // Requires skill binary to be built
fn test_generate_single_skill() {
    let temp = TempDir::new().unwrap();
    create_test_manifest(temp.path(), multi_skill_manifest());
    let output_dir = temp.path().join("skills");

    // Generate only kubernetes skill
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--skill")
        .arg("kubernetes")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Verify only kubernetes was generated
    assert!(output_dir.join("kubernetes").exists());
    assert!(!output_dir.join("docker").exists());
    assert!(!output_dir.join("terraform").exists());
}

#[test]
#[ignore] // Requires skill binary to be built
fn test_generate_force_overwrite() {
    let temp = TempDir::new().unwrap();
    create_test_manifest(temp.path(), simple_manifest());
    let output_dir = temp.path().join("skills");

    // Generate once
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Modify SKILL.md
    let skill_md = output_dir.join("test-skill").join("SKILL.md");
    fs::write(&skill_md, "MODIFIED CONTENT").unwrap();

    // Generate again without --force (should preserve)
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    let content = fs::read_to_string(&skill_md).unwrap();
    assert_eq!(
        content, "MODIFIED CONTENT",
        "File should be preserved without --force"
    );

    // Generate with --force (should overwrite)
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .arg("--force")
        .assert()
        .success();

    let content = fs::read_to_string(&skill_md).unwrap();
    assert_ne!(
        content, "MODIFIED CONTENT",
        "File should be overwritten with --force"
    );
    assert!(content.contains("---"), "Should have YAML frontmatter");
}

#[test]
#[ignore] // Requires skill binary to be built
fn test_generate_no_scripts() {
    let temp = TempDir::new().unwrap();
    create_test_manifest(temp.path(), simple_manifest());
    let output_dir = temp.path().join("skills");

    // Generate with --no-scripts
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .arg("--no-scripts")
        .assert()
        .success();

    // Verify skill structure without scripts
    verify_skill_structure(&output_dir, "test-skill", false);
}

#[test]
#[ignore] // Requires skill binary to be built
fn test_generate_dry_run() {
    let temp = TempDir::new().unwrap();
    create_test_manifest(temp.path(), simple_manifest());
    let output_dir = temp.path().join("skills");

    // Run with --dry-run
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Would generate"));

    // Verify no files were actually created
    assert!(
        !output_dir.exists(),
        "No files should be created in dry-run mode"
    );
}

#[test]
#[ignore] // Requires skill binary to be built
fn test_validate_yaml_frontmatter() {
    let temp = TempDir::new().unwrap();
    create_test_manifest(temp.path(), simple_manifest());
    let output_dir = temp.path().join("skills");

    // Generate skill
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Parse and validate YAML frontmatter
    let skill_md = output_dir.join("test-skill").join("SKILL.md");
    let yaml = parse_yaml_frontmatter(&skill_md);
    validate_yaml_frontmatter(&yaml);
}

#[test]
#[ignore] // Requires skill binary to be built
fn test_validate_skill_md_structure() {
    let temp = TempDir::new().unwrap();
    create_test_manifest(temp.path(), simple_manifest());
    let output_dir = temp.path().join("skills");

    // Generate skill
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Read SKILL.md
    let skill_md = output_dir.join("test-skill").join("SKILL.md");
    let content = fs::read_to_string(&skill_md).unwrap();

    // Verify structure
    assert!(content.contains("---"), "Should have YAML frontmatter");
    assert!(
        content.contains("# "),
        "Should have markdown headings"
    );

    // Check for common sections
    let content_lower = content.to_lowercase();
    assert!(
        content_lower.contains("usage") || content_lower.contains("how to use"),
        "Should have usage section"
    );
}

#[test]
#[ignore] // Requires skill binary to be built
fn test_validate_tools_md_format() {
    let temp = TempDir::new().unwrap();
    create_test_manifest(temp.path(), simple_manifest());
    let output_dir = temp.path().join("skills");

    // Generate skill
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Read TOOLS.md
    let tools_md = output_dir.join("test-skill").join("TOOLS.md");
    let content = fs::read_to_string(&tools_md).unwrap();

    // Verify TOOLS.md format
    assert!(
        content.contains("# "),
        "TOOLS.md should have headings"
    );
}

#[test]
#[ignore] // Requires skill binary to be built
fn test_generate_with_invalid_manifest() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join(".skill-engine.toml"), "invalid { toml [ syntax").unwrap();
    let output_dir = temp.path().join("skills");

    // Should fail with invalid manifest
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .failure();
}

#[test]
#[ignore] // Requires skill binary to be built
fn test_generate_nonexistent_skill() {
    let temp = TempDir::new().unwrap();
    create_test_manifest(temp.path(), simple_manifest());
    let output_dir = temp.path().join("skills");

    // Try to generate a skill that doesn't exist in manifest
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--skill")
        .arg("nonexistent-skill")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .failure();
}

#[test]
#[ignore] // Requires skill binary to be built
fn test_generated_skills_are_spec_compliant() {
    let temp = TempDir::new().unwrap();
    create_test_manifest(temp.path(), multi_skill_manifest());
    let output_dir = temp.path().join("skills");

    // Generate all skills from manifest
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Validate each generated skill against specification
    let validator = SpecValidator::new();

    for entry in fs::read_dir(&output_dir).unwrap() {
        let entry = entry.unwrap();
        let skill_dir = entry.path();

        if !skill_dir.is_dir() {
            continue;
        }

        let skill_name = skill_dir.file_name().unwrap().to_str().unwrap();
        println!("Validating skill: {}", skill_name);

        let result = validator.validate_skill_directory(&skill_dir);

        assert!(
            result.is_ok(),
            "Skill '{}' failed specification validation:\n{:#?}",
            skill_name,
            result.err().unwrap()
        );
    }
}
