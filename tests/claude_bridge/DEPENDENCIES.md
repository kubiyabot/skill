# Test Dependencies for Claude Bridge

This document describes all test dependencies used in the Claude Bridge testing infrastructure.

## Overview

The Claude Bridge test suite uses a comprehensive set of Rust testing libraries to ensure full coverage of functionality, performance, and edge cases.

## Core Test Dependencies

### assert_cmd (v2.0)

**Purpose:** CLI testing framework for testing command-line applications

**Use Cases:**
- Testing `skill claude generate` command execution
- Verifying command-line argument parsing
- Testing exit codes and error messages
- Validating stdout/stderr output

**Example Usage:**
```rust
use assert_cmd::Command;

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("skill").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("skill"));
}
```

**Why v2.0:** Latest stable version with full async support and improved ergonomics.

### predicates (v3.0)

**Purpose:** Assertion helpers and matchers for flexible test assertions

**Use Cases:**
- String pattern matching in output
- File existence and content validation
- Complex boolean assertions
- Composable matchers

**Example Usage:**
```rust
use predicates::prelude::*;

#[test]
fn test_generated_file_contains_yaml() {
    let content = std::fs::read_to_string("SKILL.md").unwrap();
    assert!(predicate::str::contains("---").eval(&content));
    assert!(predicate::str::contains("name:").eval(&content));
}
```

**Why v3.0:** Latest version with improved type safety and better error messages.

### mockall (v0.12)

**Purpose:** Mocking framework for creating mock objects in tests

**Use Cases:**
- Mocking file system operations
- Mocking external API calls (if any)
- Testing error handling paths
- Isolating units under test

**Example Usage:**
```rust
use mockall::predicate::*;
use mockall::*;

#[automock]
trait FileWriter {
    fn write(&self, path: &str, content: &str) -> Result<()>;
}

#[test]
fn test_renderer_with_mock() {
    let mut mock = MockFileWriter::new();
    mock.expect_write()
        .with(eq("SKILL.md"), always())
        .times(1)
        .returning(|_, _| Ok(()));

    // Test with mock
}
```

**Why v0.12:** Stable release with support for async traits and lifetime parameters.

### tempfile (workspace)

**Purpose:** Create temporary files and directories for testing

**Use Cases:**
- Creating temporary test environments
- Testing file generation without polluting workspace
- Cleaning up test artifacts automatically
- Isolating test runs

**Example Usage:**
```rust
use tempfile::TempDir;

#[test]
fn test_skill_generation() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("skills");

    // Generate skills to temp directory
    // ...

    // Temp directory automatically cleaned up when dropped
}
```

**Version:** Managed by workspace, currently 3.8+

## Advanced Testing Dependencies

### tokio-test (v0.4)

**Purpose:** Testing utilities for async Tokio code

**Use Cases:**
- Testing async functions
- Asserting async behavior
- Mocking async I/O operations
- Testing concurrent operations

**Example Usage:**
```rust
use tokio_test::{assert_ok, assert_err};

#[tokio::test]
async fn test_async_load_manifest() {
    let loader = Loader::new(Some("manifest.toml")).unwrap();
    let result = loader.load_all_skills().await;
    assert_ok!(result);
}
```

**Why v0.4:** Compatible with tokio 1.x, provides essential async test utilities.

### proptest (v1.4)

**Purpose:** Property-based testing framework

**Use Cases:**
- Testing with random inputs
- Finding edge cases automatically
- Validating invariants
- Fuzzing input validation

**Example Usage:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_skill_name_validation(name in "[a-z][a-z0-9-]{0,50}") {
        // Test that any valid skill name passes validation
        assert!(validate_skill_name(&name).is_ok());
    }
}
```

**Why v1.4:** Latest stable with shrinking improvements and better ergonomics.

### insta (v1.34)

**Purpose:** Snapshot testing for large outputs

**Use Cases:**
- Testing generated markdown files
- Comparing YAML frontmatter
- Validating script generation
- Regression testing

**Features Enabled:** `yaml` - YAML snapshot support

**Example Usage:**
```rust
use insta::assert_snapshot;

#[test]
fn test_generated_skill_md() {
    let skill = generate_test_skill();
    let markdown = render_skill_md(&skill);

    // First run creates snapshot, subsequent runs compare
    assert_snapshot!(markdown);
}
```

**Why v1.34 with yaml:** Latest version, YAML feature for structured snapshot comparisons.

## Performance Testing Dependencies

### criterion (v0.5)

**Purpose:** Statistical benchmarking framework

**Use Cases:**
- Performance regression detection
- Comparing implementation approaches
- Identifying bottlenecks
- Generating benchmark reports

**Features Enabled:** `html_reports` - Generate HTML visualization

**Example Usage:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_manifest_parsing(c: &mut Criterion) {
    c.bench_function("parse manifest", |b| {
        b.iter(|| {
            let parsed = parse_manifest(black_box("manifest.toml"));
            black_box(parsed)
        });
    });
}

criterion_group!(benches, bench_manifest_parsing);
criterion_main!(benches);
```

**Running Benchmarks:**
```bash
cargo bench --bench claude_bridge_bench
```

**View Reports:**
```bash
open target/criterion/report/index.html
```

**Why v0.5 with html_reports:** Latest version with improved statistical analysis and beautiful HTML reports.

## Dependency Compatibility

### Version Constraints

All dependencies use conservative version constraints:
- **Major version pinned** (e.g., `2.0`) for stable APIs
- **Workspace managed** where possible for consistency
- **Feature flags** only enabled when needed to reduce compilation time

### Rust Version Requirements

- **Minimum Rust Version:** 1.75+ (for edition2024 support)
- **Recommended:** Rust nightly for edition2024 features
- All dependencies compatible with Rust 1.75+

### Conflict Resolution

**Current State:** No known dependency conflicts

**If Conflicts Arise:**
```bash
# Check dependency tree
cargo tree -p skill-cli --edges dev

# Find duplicate dependencies
cargo tree -p skill-cli -d

# Update specific dependency
cargo update -p <package-name>
```

## Usage Examples

### Running Different Test Types

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Claude Bridge tests only
cargo test -p skill-cli -- claude_bridge

# With coverage
cargo tarpaulin -p skill-cli --lib

# Benchmarks
cargo bench --bench claude_bridge_bench

# Property tests with verbose output
cargo test proptest -- --nocapture
```

### Using Test Utilities

```rust
use crate::commands::claude_bridge::test_utils::*;

#[test]
fn test_with_utilities() {
    // Create mock filesystem
    let fs = MockFileSystem::new();

    // Write test manifest
    let manifest = create_minimal_manifest();
    let (_temp, path) = write_manifest_to_temp(&manifest);

    // Validate generated output
    fs.write_skill_file("SKILL.md", "content").unwrap();
    assert!(fs.skill_file_exists("SKILL.md"));

    let content = fs.read_skill_file("SKILL.md").unwrap();
    validate_skill_md_structure(&content).unwrap();
}
```

### Snapshot Testing Workflow

```bash
# Run tests and review snapshots
cargo test

# Review new/changed snapshots
cargo insta review

# Accept all snapshots
cargo insta accept

# Reject all snapshots
cargo insta reject
```

## Development Workflow

### Adding a New Test Dependency

1. Add to `Cargo.toml`:
```toml
[dev-dependencies]
new-crate = "1.0"
```

2. Document in this file:
   - Purpose and use cases
   - Version rationale
   - Example usage
   - Compatibility notes

3. Update CI if needed:
   - `.github/workflows/claude-bridge-tests.yml`
   - Check for platform-specific issues

4. Run dependency check:
```bash
cargo tree -p skill-cli --edges dev
cargo test --lib
```

### Updating Dependencies

```bash
# Check for updates
cargo outdated

# Update specific dependency
cargo update -p assert_cmd

# Update all dependencies
cargo update

# Test after update
make test
make coverage
```

## CI/CD Integration

### GitHub Actions

Dependencies are automatically cached in CI:
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    shared-key: "rust-deps"
```

### Build Times

Approximate clean build times:
- **Unit tests:** ~2-3 minutes
- **Integration tests:** ~3-5 minutes
- **Benchmarks:** ~5-7 minutes

**Caching Impact:**
- First build: ~10 minutes
- Cached builds: ~30 seconds

## Troubleshooting

### Common Issues

#### Issue: "could not compile `mockall`"

**Cause:** Version incompatibility or missing proc-macro dependencies

**Solution:**
```bash
cargo clean
cargo update -p mockall
cargo test --lib
```

#### Issue: "criterion requires nightly"

**Cause:** Some criterion features require nightly Rust

**Solution:**
```bash
rustup default nightly
cargo bench
```

#### Issue: "snapshot test failed"

**Cause:** Generated output changed

**Solution:**
```bash
# Review changes
cargo insta review

# Accept if expected
cargo insta accept
```

#### Issue: Slow test execution

**Cause:** Running all tests including benchmarks

**Solution:**
```bash
# Run only unit tests
cargo test --lib

# Skip slow tests
cargo test --lib -- --skip slow_
```

### Getting Help

1. Check this documentation
2. Review dependency documentation:
   - [assert_cmd](https://docs.rs/assert_cmd)
   - [predicates](https://docs.rs/predicates)
   - [mockall](https://docs.rs/mockall)
   - [proptest](https://docs.rs/proptest)
   - [insta](https://docs.rs/insta)
   - [criterion](https://docs.rs/criterion)
3. Check GitHub issues for known problems
4. Ask in team chat

## Maintenance

### Weekly Tasks

- Review dependency updates: `cargo outdated`
- Check for security advisories: `cargo audit`
- Update snapshot tests if needed: `cargo insta review`

### Monthly Tasks

- Update dependencies: `cargo update`
- Review benchmark trends
- Update this documentation

### Before Release

```bash
# Full dependency check
cargo tree -p skill-cli --edges dev
cargo audit

# Full test suite
make test
make coverage-check
cargo bench

# Verify clean build
cargo clean
cargo build --release
```

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Cargo Book - Dev Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#development-dependencies)
- [assert_cmd Documentation](https://docs.rs/assert_cmd)
- [Property Testing Guide](https://proptest-rs.github.io/proptest/)
- [Criterion User Guide](https://bheisler.github.io/criterion.rs/book/)
- [Insta Snapshot Testing](https://insta.rs/)

## Summary

| Dependency | Version | Purpose | Key Use Cases |
|------------|---------|---------|---------------|
| assert_cmd | 2.0 | CLI testing | Command execution, output validation |
| predicates | 3.0 | Assertions | Pattern matching, flexible assertions |
| mockall | 0.12 | Mocking | Unit test isolation, error paths |
| tokio-test | 0.4 | Async testing | Testing async functions |
| tempfile | workspace | Temp files | Test isolation, cleanup |
| proptest | 1.4 | Property testing | Fuzzing, edge cases |
| insta | 1.34 | Snapshots | Regression testing, large outputs |
| criterion | 0.5 | Benchmarking | Performance tracking |

All dependencies are production-ready, actively maintained, and widely used in the Rust ecosystem.
