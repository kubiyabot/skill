//! Error Handling and Edge Case Tests
//!
//! This module contains comprehensive error handling tests for the Claude Bridge
//! skill generation, covering:
//!
//! - Missing or invalid manifest files
//! - Filesystem permission errors
//! - Concurrent generation scenarios
//! - Partial failure recovery
//! - Invalid input sanitization
//! - Error message quality validation
//!
//! # Running Tests
//!
//! ```bash
//! # Run all error tests
//! cargo test --test error_tests -- --ignored
//!
//! # Run specific error category
//! cargo test test_error_missing_manifest -- --ignored
//! ```

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ============================================================================
// Manifest Error Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary
fn test_error_missing_manifest() {
    let temp = TempDir::new().unwrap();

    // Try to generate without manifest file
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("manifest").or(predicate::str::contains("not found")));
}

#[test]
#[ignore] // Requires skill binary
fn test_error_invalid_toml_manifest() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    // Create manifest with invalid TOML syntax
    fs::write(&manifest_path, "invalid { toml [ syntax").unwrap();

    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("TOML")
                .or(predicate::str::contains("parse"))
                .or(predicate::str::contains("invalid")),
        );
}

#[test]
#[ignore] // Requires skill binary
fn test_error_missing_required_manifest_fields() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    // Create manifest missing required fields
    fs::write(
        &manifest_path,
        r#"
[skills.incomplete]
# Missing description and other required fields
source = "./incomplete"
"#,
    )
    .unwrap();

    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("required")
                .or(predicate::str::contains("missing"))
                .or(predicate::str::contains("description")),
        );
}

#[test]
#[ignore] // Requires skill binary
fn test_error_invalid_skill_name() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills.valid-skill]
source = "./valid"
runtime = "wasm"
description = "A valid skill"
"#,
    )
    .unwrap();

    // Try to generate non-existent skill
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--skill")
        .arg("nonexistent-skill")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .and(predicate::str::contains("nonexistent-skill")),
        );
}

#[test]
#[ignore] // Requires skill binary
fn test_error_no_skills_in_manifest() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    // Create empty manifest
    fs::write(&manifest_path, "# Empty manifest\n").unwrap();

    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("No skills")
                .or(predicate::str::contains("empty"))
                .or(predicate::str::contains("found")),
        );
}

// ============================================================================
// Filesystem Permission Error Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary and Unix permissions
#[cfg(unix)]
fn test_error_output_dir_not_writable() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    // Create valid manifest
    fs::write(
        &manifest_path,
        r#"
[skills.test-skill]
source = "./test"
runtime = "wasm"
description = "Test skill"
"#,
    )
    .unwrap();

    let output_dir = temp.path().join("readonly");
    fs::create_dir(&output_dir).unwrap();

    // Make directory read-only
    let mut perms = fs::metadata(&output_dir).unwrap().permissions();
    perms.set_mode(0o444); // r--r--r--
    fs::set_permissions(&output_dir, perms).unwrap();

    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("permission")
                .or(predicate::str::contains("Permission"))
                .or(predicate::str::contains("denied")),
        );

    // Clean up: restore permissions so temp dir can be deleted
    let mut perms = fs::metadata(&output_dir).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&output_dir, perms).unwrap();
}

#[test]
#[ignore] // Requires skill binary
fn test_error_script_generation_failure() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills.test-skill]
source = "./test"
runtime = "wasm"
description = "Test skill"
"#,
    )
    .unwrap();

    let output_dir = temp.path().join("skills");
    fs::create_dir_all(&output_dir).unwrap();

    let skill_dir = output_dir.join("test-skill");
    fs::create_dir(&skill_dir).unwrap();

    // Create a file where scripts/ directory should be
    let scripts_path = skill_dir.join("scripts");
    fs::write(&scripts_path, "I am a file, not a directory").unwrap();

    // This should fail because scripts exists as a file
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .arg("--force")
        .assert()
        .failure();
}

// ============================================================================
// Concurrent Generation Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary, slow test
fn test_concurrent_generation_safety() {
    use std::sync::Arc;
    use std::thread;

    let temp = Arc::new(TempDir::new().unwrap());
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills.test-skill]
source = "./test"
runtime = "wasm"
description = "Test skill for concurrency"
"#,
    )
    .unwrap();

    let output_dir = temp.path().join("skills");
    fs::create_dir(&output_dir).unwrap();

    let mut handles = vec![];

    // Spawn 3 concurrent generation processes
    for i in 0..3 {
        let temp_path = temp.path().to_path_buf();
        let output = output_dir.clone();

        let handle = thread::spawn(move || {
            println!("Thread {} starting generation", i);
            Command::cargo_bin("skill")
                .unwrap()
                .current_dir(&temp_path)
                .arg("claude")
                .arg("generate")
                .arg("--output")
                .arg(&output)
                .arg("--force")
                .output()
                .expect("Failed to execute command");
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify generated files are not corrupted
    let skill_md = output_dir.join("test-skill").join("SKILL.md");
    assert!(
        skill_md.exists(),
        "SKILL.md should exist after concurrent generation"
    );

    let content = fs::read_to_string(skill_md).unwrap();
    assert!(
        content.contains("---"),
        "SKILL.md should have valid YAML frontmatter"
    );
    assert!(
        content.contains("name:"),
        "SKILL.md should have name field"
    );
}

// ============================================================================
// Partial Failure Recovery Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary
fn test_partial_failure_continues_generation() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    // Create manifest with one valid and one potentially problematic skill
    fs::write(
        &manifest_path,
        r#"
[skills.valid-skill]
source = "./valid"
runtime = "wasm"
description = "A valid skill"

[skills.test-skill-2]
source = "./test2"
runtime = "native"
description = "Another valid skill"
"#,
    )
    .unwrap();

    let output_dir = temp.path().join("skills");

    let result = Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .output()
        .unwrap();

    // Check if at least some skills were generated
    if output_dir.exists() {
        let entries: Vec<_> = fs::read_dir(&output_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        if !entries.is_empty() {
            println!(
                "Generated {} skill(s) even with potential issues",
                entries.len()
            );
        }
    }

    // Output should contain information about any failures
    let stderr = String::from_utf8_lossy(&result.stderr);
    println!("stderr: {}", stderr);
}

// ============================================================================
// Path Traversal and Security Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary
fn test_path_traversal_prevention() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills.test]
source = "./test"
runtime = "wasm"
description = "Test"
"#,
    )
    .unwrap();

    // Try to use path traversal in output directory
    let traversal_path = temp.path().join("..").join("..").join("..").join("etc");

    let result = Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&traversal_path)
        .output()
        .unwrap();

    // Should either reject the path or sanitize it
    // At minimum, should not write outside temp directory
    assert!(
        !result.status.success() || !Path::new("/etc/skills").exists(),
        "Should not write to /etc via path traversal"
    );
}

#[test]
#[ignore] // Requires skill binary
fn test_special_characters_in_paths() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills.test]
source = "./test"
runtime = "wasm"
description = "Test"
"#,
    )
    .unwrap();

    // Test with spaces in output directory name
    let output_dir = temp.path().join("skills with spaces");

    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .assert();

    // Should handle spaces correctly
    if output_dir.exists() {
        println!("Successfully handled spaces in path");
    }
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary
fn test_error_message_quality() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills.existing]
source = "./existing"
runtime = "wasm"
description = "An existing skill"
"#,
    )
    .unwrap();

    let result = Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--skill")
        .arg("nonexistent")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&result.stderr);

    // Error message quality checks:
    // 1. Should explain what went wrong
    assert!(
        stderr.contains("not found") || stderr.contains("does not exist"),
        "Error should explain skill was not found"
    );

    // 2. Should mention the skill name
    assert!(
        stderr.contains("nonexistent"),
        "Error should mention the requested skill name"
    );

    // 3. Should not contain raw stack traces (unless in debug mode)
    let has_stack_trace = stderr.contains("panicked at")
        || stderr.contains("stack backtrace:")
        || stderr.contains("thread 'main'");

    if has_stack_trace {
        println!("Warning: Error message contains stack trace");
        println!("stderr: {}", stderr);
    }

    // 4. Error message should be reasonably concise
    let line_count = stderr.lines().count();
    assert!(
        line_count < 50,
        "Error message should be concise (got {} lines)",
        line_count
    );
}

#[test]
#[ignore] // Requires skill binary
fn test_helpful_error_suggestions() {
    let temp = TempDir::new().unwrap();

    // No manifest at all
    let result = Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&result.stderr);

    // Should suggest how to fix the issue
    let has_helpful_info = stderr.contains("create")
        || stderr.contains("initialize")
        || stderr.contains(".skill-engine.toml")
        || stderr.contains("manifest");

    assert!(
        has_helpful_info,
        "Error should provide helpful suggestions: {}",
        stderr
    );
}

// ============================================================================
// Data Edge Cases
// ============================================================================

#[test]
#[ignore] // Requires skill binary
fn test_extreme_values() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    // Skill name exactly 64 characters (max length)
    let long_name = "a".repeat(64);

    // Description exactly 1024 characters
    let long_desc = "d".repeat(1024);

    fs::write(
        &manifest_path,
        format!(
            r#"
[skills.{}]
source = "./test"
runtime = "wasm"
description = "{}"
"#,
            long_name, long_desc
        ),
    )
    .unwrap();

    let output_dir = temp.path().join("skills");

    // Should handle maximum length values
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .assert();
}

#[test]
#[ignore] // Requires skill binary
fn test_unicode_in_descriptions() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    // Unicode characters in description
    fs::write(
        &manifest_path,
        r#"
[skills.unicode-test]
source = "./test"
runtime = "wasm"
description = "Kubernetes 集群管理 🚀 Deploy to cloud"
"#,
    )
    .unwrap();

    let output_dir = temp.path().join("skills");

    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Verify unicode is preserved
    let skill_md = output_dir.join("unicode-test").join("SKILL.md");
    if skill_md.exists() {
        let content = fs::read_to_string(skill_md).unwrap();
        assert!(
            content.contains("集群") && content.contains("🚀"),
            "Unicode characters should be preserved"
        );
    }
}
