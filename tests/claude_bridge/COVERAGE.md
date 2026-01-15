# Code Coverage Guide for Claude Bridge

This document explains how to run, interpret, and maintain code coverage for the Claude Bridge feature.

## Quick Start

```bash
# Generate HTML coverage report and open it
make coverage-open

# Run coverage with threshold check
make coverage-check

# Unit tests coverage only
make coverage-unit
```

## Coverage Targets

### Global Targets
- **Project-wide**: 70% minimum coverage
- **Claude Bridge**: 80% minimum coverage (new code)
- **Patches (new code)**: 80% target

### Rationale
- **70% global**: Realistic for entire codebase with legacy code
- **80% Claude Bridge**: Higher standard for new, well-tested features
- **80% patches**: Ensures new code maintains quality

## Running Coverage Locally

### Prerequisites

Install cargo-tarpaulin:
```bash
cargo install cargo-tarpaulin
```

### Basic Usage

```bash
# All tests with all output formats (XML, HTML, Lcov)
make coverage

# Unit tests only
make coverage-unit

# Integration tests only
make coverage-integration

# Open HTML report in browser
make coverage-open
```

### Advanced Usage

```bash
# Specific package
cargo tarpaulin -p skill-cli --out Html

# Specific module
cargo tarpaulin -p skill-cli --lib -- claude_bridge --out Html

# With verbosity
cargo tarpaulin --workspace --out Html --verbose

# Fail if below threshold
cargo tarpaulin --workspace --out Xml --fail-under 70
```

## Interpreting Coverage Reports

### HTML Report

The HTML report (`tarpaulin-report.html`) shows:
- **Green lines**: Covered by tests
- **Red lines**: Not covered by tests
- **Yellow lines**: Partially covered (e.g., match arms)

### Coverage Percentage

```
Lines: 450/500 (90%)
  ├─ Covered: 450
  ├─ Missed: 50
  └─ Percentage: 90%
```

### What to Look For

1. **Uncovered Error Paths**: Error handling often uncovered
2. **Edge Cases**: Boundary conditions might be missed
3. **Helper Functions**: Small utilities sometimes skipped
4. **Debug Code**: Code only in debug builds

## Configuration Files

### tarpaulin.toml

Location: `/tarpaulin.toml`

Key settings:
- `exclude-files`: Files excluded from coverage
- `fail-under`: Minimum coverage threshold (70%)
- `packages.fail-under`: Per-package thresholds

### .codecov.yml

Location: `/.codecov.yml`

Configures Codecov integration:
- Target coverage: 70%
- Patch coverage: 80%
- Ignore patterns
- Comment behavior

## Handling Uncoverable Code

### Using Attributes

```rust
// Skip this function from coverage
#[cfg(not(tarpaulin_include))]
fn debug_only_function() {
    // ...
}

// Skip specific branches
fn example() {
    if cfg!(test) {
        #[cfg(not(tarpaulin_include))]
        {
            // Test-only code
        }
    }
}
```

### Common Uncoverable Patterns

1. **Main functions**: Entry points usually uncovered
2. **Debug impls**: `fmt::Debug` often not tested
3. **Error conversions**: Simple `From` impls
4. **Platform-specific code**: OS-specific branches

## CI/CD Integration

### GitHub Actions

Coverage runs automatically on:
- Pull requests
- Pushes to main/develop
- Scheduled nightly builds

Workflow: `.github/workflows/claude-bridge-tests.yml`

### Coverage Job

```yaml
coverage:
  runs-on: ubuntu-22.04
  steps:
    - Install cargo-tarpaulin
    - Generate coverage
    - Upload to Codecov
    - Check thresholds
```

### Artifacts

Coverage reports are uploaded as artifacts:
- Retention: 30 days
- Formats: XML, HTML, Lcov

## Codecov Integration

### Setup

1. Sign up at [codecov.io](https://codecov.io)
2. Enable repository
3. Add `CODECOV_TOKEN` to GitHub secrets

### Features

- **PR Comments**: Automatic coverage comments on PRs
- **Coverage Diff**: Shows coverage changes
- **Trend Graphs**: Track coverage over time
- **Flags**: Separate coverage for different test types

### Viewing Reports

1. Go to codecov.io/gh/kubiyabot/skill
2. Navigate to specific commits or PRs
3. View file-level coverage details

## Best Practices

### Writing Testable Code

```rust
// Good: Testable function
pub fn parse_manifest(path: &Path) -> Result<Manifest> {
    let content = std::fs::read_to_string(path)?;
    parse_manifest_str(&content)
}

// Internal function is easier to test
fn parse_manifest_str(content: &str) -> Result<Manifest> {
    // Implementation
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_manifest_str() {
        let result = parse_manifest_str("[[skill]]\nname = 'test'");
        assert!(result.is_ok());
    }
}
```

### Testing Error Paths

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_error_handling() {
        // Test the error path
        let result = parse_manifest_str("invalid toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_edge_cases() {
        // Empty input
        assert!(parse_manifest_str("").is_err());

        // Very large input
        let large = "x".repeat(10_000);
        assert!(parse_manifest_str(&large).is_err());
    }
}
```

### Coverage-Driven Development

1. **Write tests first**: Start with test cases
2. **Implement feature**: Write minimal code to pass tests
3. **Check coverage**: Run `make coverage-open`
4. **Add missing tests**: Cover red lines
5. **Refactor**: Improve code with confidence

## Troubleshooting

### Coverage Too Low

**Problem**: Coverage below threshold

**Solutions**:
1. Add tests for uncovered lines
2. Remove dead code
3. Mark uncoverable code with `#[cfg(not(tarpaulin_include))]`
4. Add integration tests for complex flows

### Slow Coverage Generation

**Problem**: Coverage takes too long

**Solutions**:
1. Run specific packages: `cargo tarpaulin -p skill-cli`
2. Run specific tests: `cargo tarpaulin -- test_name`
3. Skip doctests: Remove `Doctests` from `tarpaulin.toml`
4. Use `--skip-clean` flag

### Coverage Report Issues

**Problem**: Report shows wrong coverage

**Solutions**:
1. Clean build: `make clean && make coverage`
2. Update tarpaulin: `cargo install cargo-tarpaulin --force`
3. Check exclusions in `tarpaulin.toml`
4. Verify test actually runs: add `println!` statements

## Maintenance

### Weekly Tasks

- Review coverage trends on Codecov
- Identify low-coverage modules
- Plan tests for uncovered code

### Before Release

```bash
# Full coverage check
make coverage-check

# Review HTML report
make coverage-open

# Verify CI passes
git push && check GitHub Actions
```

### Updating Thresholds

If coverage consistently exceeds targets:

1. Update `tarpaulin.toml`:
   ```toml
   fail-under = 75.0  # Increased from 70%
   ```

2. Update `.codecov.yml`:
   ```yaml
   target: 75%  # Increased from 70%
   ```

3. Update this document

## Resources

- [cargo-tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [Codecov Documentation](https://docs.codecov.com)
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [PRD: Claude Bridge Testing](.taskmaster/docs/claude-bridge-testing-prd.txt)

## Support

For coverage-related issues:
1. Check this guide
2. Review tarpaulin documentation
3. Ask in team chat
4. Open GitHub issue with `coverage` label
