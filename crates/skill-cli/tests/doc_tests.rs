//! Documentation Accuracy and Quality Tests
//!
//! This module tests the accuracy and quality of project documentation including:
//! - README code examples validation
//! - Internal link validation
//! - CLI help text completeness
//! - Error message quality and helpfulness
//!
//! # Running Tests
//!
//! ```bash
//! cargo test --test doc_tests -- --ignored
//! ```

use assert_cmd::Command;
use regex::Regex;
use std::fs;
use std::path::Path;

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract code blocks of a specific language from markdown
fn extract_code_blocks(markdown: &str, language: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current_block = String::new();
    let block_start = format!("```{}", language);

    for line in markdown.lines() {
        if line.starts_with(&block_start) {
            in_block = true;
            current_block.clear();
        } else if line.starts_with("```") && in_block {
            in_block = false;
            blocks.push(current_block.clone());
        } else if in_block {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }

    blocks
}

// ============================================================================
// README Code Example Tests
// ============================================================================

#[test]
#[ignore] // Requires README.md in project root
fn test_readme_bash_examples() {
    let readme_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("README.md");

    if !readme_path.exists() {
        println!("README.md not found at {:?}, skipping test", readme_path);
        return;
    }

    let readme = fs::read_to_string(&readme_path)
        .expect("Failed to read README.md");

    let bash_blocks = extract_code_blocks(&readme, "bash");

    println!("Found {} bash code blocks in README", bash_blocks.len());

    for (i, block) in bash_blocks.iter().enumerate() {
        // Skip blocks with placeholders or ellipsis
        if block.contains('<')
            || block.contains("...")
            || block.contains("$YOUR_")
            || block.contains("your-")
            || block.trim().is_empty() {
            println!("Skipping block {} (contains placeholders)", i);
            continue;
        }

        println!("Testing README bash example {}", i);
        println!("Code: {}", block.lines().take(2).collect::<Vec<_>>().join(" "));

        // Test bash syntax validity
        let result = std::process::Command::new("bash")
            .arg("-n") // Check syntax only, don't execute
            .arg("-c")
            .arg(block)
            .output()
            .expect("Failed to execute bash syntax check");

        assert!(
            result.status.success(),
            "README bash example {} has syntax errors:\n{}\n\nCode:\n{}",
            i,
            String::from_utf8_lossy(&result.stderr),
            block
        );
    }
}

#[test]
#[ignore] // Requires README.md in project root
fn test_readme_shell_examples() {
    let readme_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("README.md");

    if !readme_path.exists() {
        println!("README.md not found, skipping test");
        return;
    }

    let readme = fs::read_to_string(&readme_path).unwrap();

    // Also test ```sh blocks
    let sh_blocks = extract_code_blocks(&readme, "sh");
    println!("Found {} sh code blocks in README", sh_blocks.len());

    for (i, block) in sh_blocks.iter().enumerate() {
        if block.contains('<') || block.contains("...") || block.trim().is_empty() {
            continue;
        }

        let result = std::process::Command::new("bash")
            .arg("-n")
            .arg("-c")
            .arg(block)
            .output()
            .unwrap();

        assert!(
            result.status.success(),
            "README sh example {} has syntax errors:\n{}",
            i,
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

// ============================================================================
// Documentation Link Validation Tests
// ============================================================================

#[test]
#[ignore] // Requires README.md in project root
fn test_documentation_links() {
    let readme_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("README.md");

    if !readme_path.exists() {
        println!("README.md not found, skipping test");
        return;
    }

    let readme = fs::read_to_string(&readme_path).unwrap();
    let project_root = readme_path.parent().unwrap();

    // Extract markdown links: [text](link)
    let link_regex = Regex::new(r"\[([^\]]+)\]\(([^\)]+)\)").unwrap();

    let mut broken_links = Vec::new();

    for cap in link_regex.captures_iter(&readme) {
        let link_text = cap.get(1).unwrap().as_str();
        let link = cap.get(2).unwrap().as_str();

        // Skip external links
        if link.starts_with("http://") || link.starts_with("https://") {
            continue;
        }

        // Skip mailto links
        if link.starts_with("mailto:") {
            continue;
        }

        println!("Validating link: {} -> {}", link_text, link);

        // Check internal links
        if link.starts_with('#') {
            // Anchor link - check heading exists in README
            let anchor = link.trim_start_matches('#');
            let heading_pattern = format!("# {}", anchor.replace('-', " "));

            if !readme.to_lowercase().contains(&heading_pattern.to_lowercase()) {
                // Also try with ##, ###, etc.
                let mut found = false;
                for prefix in &["# ", "## ", "### ", "#### "] {
                    let pattern = format!("{}{}", prefix, anchor.replace('-', " "));
                    if readme.to_lowercase().contains(&pattern.to_lowercase()) {
                        found = true;
                        break;
                    }
                }

                if !found {
                    broken_links.push(format!("Broken anchor link: {} -> {}", link_text, link));
                }
            }
        } else {
            // File link - resolve relative to README location
            let file_path = project_root.join(link);

            if !file_path.exists() {
                broken_links.push(format!(
                    "Broken file link: {} -> {} (resolved to {:?})",
                    link_text, link, file_path
                ));
            }
        }
    }

    if !broken_links.is_empty() {
        panic!(
            "Found {} broken links in README:\n{}",
            broken_links.len(),
            broken_links.join("\n")
        );
    }
}

// ============================================================================
// CLI Help Text Completeness Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary
fn test_claude_generate_help_complete() {
    let result = Command::cargo_bin("skill")
        .unwrap()
        .arg("claude")
        .arg("generate")
        .arg("--help")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Verify all flags are documented
    let required_flags = vec![
        "--skill",
        "--output",
        "--force",
        "--dry-run",
        "--no-scripts",
        "--project",
    ];

    for flag in required_flags {
        assert!(
            output.contains(flag),
            "Help text missing required flag: {}",
            flag
        );
    }

    // Verify examples section exists
    assert!(
        output.to_lowercase().contains("examples:") || output.to_lowercase().contains("example:"),
        "Help text should include examples section"
    );

    // Verify description is present
    assert!(
        output.len() > 200,
        "Help text seems too short (only {} chars)",
        output.len()
    );
}

#[test]
#[ignore] // Requires skill binary
fn test_claude_command_help_text() {
    let result = Command::cargo_bin("skill")
        .unwrap()
        .arg("claude")
        .arg("--help")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Should mention generate subcommand
    assert!(output.contains("generate"), "Help should mention 'generate' subcommand");

    // Should have description
    assert!(
        output.len() > 100,
        "Claude command help text too short"
    );
}

#[test]
#[ignore] // Requires skill binary
fn test_main_help_includes_claude() {
    let result = Command::cargo_bin("skill")
        .unwrap()
        .arg("--help")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Main help should mention claude subcommand
    assert!(
        output.contains("claude"),
        "Main help text should mention 'claude' command"
    );
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
#[ignore] // Requires skill binary
fn test_error_messages_are_helpful() {
    struct ErrorTest {
        args: Vec<&'static str>,
        expected_keywords: Vec<&'static str>,
        description: &'static str,
    }

    let tests = vec![
        ErrorTest {
            args: vec!["claude", "generate", "--skill", "nonexistent-skill-xyz"],
            expected_keywords: vec!["not found", "nonexistent"],
            description: "Nonexistent skill error",
        },
        ErrorTest {
            args: vec!["claude", "generate", "--output", "/root/protected-dir"],
            expected_keywords: vec!["permission", "denied"],
            description: "Permission denied error",
        },
    ];

    for test in tests {
        println!("Testing error: {}", test.description);

        let result = Command::cargo_bin("skill")
            .unwrap()
            .args(&test.args)
            .assert()
            .failure();

        let stderr = String::from_utf8_lossy(&result.get_output().stderr);
        let stderr_lower = stderr.to_lowercase();

        // Check for expected keywords
        let mut missing_keywords = Vec::new();
        for keyword in &test.expected_keywords {
            if !stderr_lower.contains(keyword) {
                missing_keywords.push(*keyword);
            }
        }

        if !missing_keywords.is_empty() {
            println!("Error message: {}", stderr);
            panic!(
                "{} missing keywords: {:?}",
                test.description, missing_keywords
            );
        }

        // Error should not contain raw stack traces (unless --debug)
        assert!(
            !stderr.contains("panicked at") && !stderr.contains("stack backtrace:"),
            "Error message should not contain stack trace:\n{}",
            stderr
        );

        // Error should be reasonably concise
        let line_count = stderr.lines().count();
        assert!(
            line_count < 50,
            "Error message too verbose ({} lines):\n{}",
            line_count,
            stderr
        );
    }
}

#[test]
#[ignore] // Requires skill binary
fn test_error_message_provides_suggestions() {
    // Test that error for missing manifest provides helpful guidance
    let temp = tempfile::TempDir::new().unwrap();

    let result = Command::cargo_bin("skill")
        .unwrap()
        .current_dir(temp.path())
        .arg("claude")
        .arg("generate")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&result.get_output().stderr);
    let stderr_lower = stderr.to_lowercase();

    // Should suggest how to fix the issue
    let has_helpful_suggestion = stderr_lower.contains("create")
        || stderr_lower.contains("initialize")
        || stderr_lower.contains(".skill-engine.toml")
        || stderr_lower.contains("manifest");

    assert!(
        has_helpful_suggestion,
        "Error should provide helpful suggestions:\n{}",
        stderr
    );
}

// ============================================================================
// Documentation Completeness Tests
// ============================================================================

#[test]
#[ignore]
fn test_readme_has_essential_sections() {
    let readme_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("README.md");

    if !readme_path.exists() {
        println!("README.md not found, skipping test");
        return;
    }

    let readme = fs::read_to_string(&readme_path).unwrap();
    let readme_lower = readme.to_lowercase();

    // Essential sections
    let essential_sections = vec![
        ("installation", "Installation or Quick Start section"),
        ("usage", "Usage section"),
        ("example", "Examples section"),
    ];

    for (keyword, description) in essential_sections {
        assert!(
            readme_lower.contains(keyword),
            "README should contain {}: '{}'",
            description,
            keyword
        );
    }

    // README should be substantial
    let line_count = readme.lines().count();
    assert!(
        line_count > 100,
        "README seems too short: only {} lines",
        line_count
    );
}

#[test]
#[ignore]
fn test_readme_mentions_key_features() {
    let readme_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("README.md");

    if !readme_path.exists() {
        return;
    }

    let readme = fs::read_to_string(&readme_path).unwrap();
    let readme_lower = readme.to_lowercase();

    // Key features that should be mentioned
    let key_features = vec![
        "claude",
        "skill",
        "generate",
    ];

    for feature in key_features {
        assert!(
            readme_lower.contains(feature),
            "README should mention key feature: '{}'",
            feature
        );
    }
}
