# Claude Bridge Testing Suite

This directory contains comprehensive end-to-end tests for the Claude Bridge feature, which generates Claude Agent Skills from Skill Engine capabilities.

## Overview

The Claude Bridge testing suite validates:
- Skill generation in all modes (all skills, single skill, project-local, force, no-scripts)
- MCP server integration and tool execution
- Claude Code integration and skill discovery
- Real-world usage scenarios
- Security, performance, and compliance

## Test Environment Requirements

### macOS Environment (GitHub Actions: macos-latest)

**Requirements**:
- macOS 12+ (Monterey or later)
- Rust stable toolchain (1.75+)
- Xcode Command Line Tools
- Homebrew (for dependencies)

**Setup**:
```bash
# Install Rust if not present
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install system dependencies
brew install openssl pkg-config

# Build skill-cli
cargo build -p skill-cli --release

# Verify installation
./target/release/skill --version
```

### Linux Environment (GitHub Actions: ubuntu-22.04)

**Requirements**:
- Ubuntu 22.04 LTS (or compatible)
- Rust stable toolchain (1.75+)
- Build essentials
- OpenSSL development libraries

**Setup**:
```bash
# Install Rust if not present
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install system dependencies
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev

# Build skill-cli
cargo build -p skill-cli --release

# Verify installation
./target/release/skill --version
```

### Docker Environment (rust:1.75-slim)

**Requirements**:
- Docker 20.10+
- docker-compose 2.0+ (optional, for orchestrated tests)

**Setup**:
```bash
# Build test container
docker build -f tests/claude_bridge/Dockerfile -t skill-cli-test .

# Run tests in container
docker run --rm -v $(pwd):/workspace skill-cli-test cargo test --package skill-cli

# Or use docker-compose
docker-compose -f tests/claude_bridge/docker-compose.yml up --build
```

## Test Categories

### 1. Unit Tests (`cargo test`)
Location: `crates/skill-cli/src/commands/claude_bridge/*/tests.rs`

Tests individual modules:
- Loader: Manifest parsing and skill discovery
- Validator: Name/description validation and sanitization
- Transformer: Type conversions and categorization
- Renderer: SKILL.md and TOOLS.md generation
- Script Generator: Bash script generation

**Run**:
```bash
cargo test -p skill-cli --lib -- claude_bridge
```

### 2. Integration Tests (Bash)
Location: `tests/claude_bridge/test-*.sh`

Tests end-to-end workflows:
- `test-skill-generation.sh`: All generation modes
- `test-mcp-integration.sh`: MCP server and tool execution
- `test-claude-code.sh`: Claude Code integration
- `test-performance.sh`: Performance benchmarks

**Run**:
```bash
# All integration tests
./tests/claude_bridge/test-all.sh

# Individual test suites
./tests/claude_bridge/test-skill-generation.sh
./tests/claude_bridge/test-mcp-integration.sh
```

### 3. Acceptance Tests (Rust + Manual)
Location: `tests/claude_bridge/acceptance_tests/`

Tests real-world scenarios:
- Kubernetes pod investigation
- Docker container debugging
- Git repository analysis
- AWS infrastructure review
- Terraform plan validation

**Run**:
```bash
# Automated acceptance tests
cargo test --test scenarios

# Manual tests (requires Claude Code)
# Follow: tests/claude_bridge/acceptance_tests/MANUAL_TESTS.md
```

### 4. Security Tests
Location: `tests/claude_bridge/security/`

Tests security vectors:
- API key leak prevention
- Command injection prevention
- Path traversal protection
- XSS prevention
- Privilege escalation prevention

**Run**:
```bash
./tests/claude_bridge/security/test-security.sh
cargo audit
cargo clippy -- -D warnings
```

### 5. Performance Tests
Location: `tests/claude_bridge/performance/`

Benchmarks:
- Skill generation speed (1, 10, 50 skills)
- Memory usage profiling
- Scalability tests

**Run**:
```bash
./tests/claude_bridge/performance/test-performance.sh
cargo bench --bench claude_bridge_bench
```

## Test Fixtures

### Manifest Fixtures (`fixtures/manifests/`)
- `minimal.toml`: 1 skill, 1 tool (baseline)
- `small.toml`: 3 skills, 5 tools each (typical)
- `medium.toml`: 10 skills, 10-20 tools each (production)
- `large.toml`: 50 skills, varying tools (stress test)
- `complex.toml`: Skills with all features (parameters, streaming, examples)

### Reference Skills (`fixtures/skills/`)
- Pre-generated SKILL.md files for validation
- Golden files for regression testing
- Example outputs for documentation

## CI/CD Integration

### GitHub Actions Workflow
Location: `.github/workflows/claude-bridge-tests.yml`

**Triggers**:
- Push to any branch
- Pull request creation/update
- Scheduled nightly builds

**Jobs**:
1. **unit-tests**: Rust unit tests on macOS and Linux
2. **integration-tests**: Bash integration tests on Linux
3. **acceptance-tests**: Real-world scenario tests
4. **security-tests**: Security scanning and audits
5. **performance-tests**: Benchmarks (scheduled only)

**Coverage**:
- Target: 80%+ unit coverage, 70%+ integration coverage
- Tool: cargo-tarpaulin
- Reports: Codecov integration

## Running Tests Locally

### Quick Start
```bash
# 1. Setup environment
./tests/claude_bridge/setup-env.sh

# 2. Run all tests
./tests/claude_bridge/test-all.sh

# 3. View coverage report
cargo tarpaulin --workspace --exclude-files 'tests/*' --out Html
open tarpaulin-report.html
```

### Individual Test Suites
```bash
# Unit tests only
cargo test -p skill-cli --lib -- claude_bridge

# Integration tests only
./tests/claude_bridge/test-skill-generation.sh

# Acceptance tests only
cargo test --test scenarios

# Security tests only
./tests/claude_bridge/security/test-security.sh

# Performance tests only
./tests/claude_bridge/performance/test-performance.sh
```

### With Docker
```bash
# Build test container
docker build -f tests/claude_bridge/Dockerfile -t skill-cli-test .

# Run all tests in container
docker run --rm -v $(pwd):/workspace skill-cli-test \
  ./tests/claude_bridge/test-all.sh

# Interactive testing
docker run --rm -it -v $(pwd):/workspace skill-cli-test bash
```

## Debugging Tests

### Enable Debug Logging
```bash
RUST_LOG=debug cargo test -p skill-cli -- --nocapture
```

### Run Specific Test
```bash
cargo test -p skill-cli test_validate_name_valid -- --nocapture
```

### Generate Test Artifacts
```bash
# Generate skills for inspection
cargo run -p skill-cli -- claude generate --output /tmp/test-skills --dry-run

# Run with verbose output
./tests/claude_bridge/test-skill-generation.sh -v
```

## Test Data Management

### Cleaning Test Artifacts
```bash
# Remove generated test files
rm -rf /tmp/skill-test-*
rm -rf ~/.claude/skills-test/

# Clean build artifacts
cargo clean
```

### Regenerating Fixtures
```bash
# Update manifest fixtures
./tests/claude_bridge/fixtures/generate-fixtures.sh

# Regenerate reference skills
./tests/claude_bridge/fixtures/generate-reference-skills.sh
```

## Troubleshooting

### Common Issues

**Issue**: `cargo build` fails with OpenSSL errors
**Solution**:
```bash
# macOS
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)

# Linux
sudo apt-get install libssl-dev pkg-config
```

**Issue**: Permission denied when running scripts
**Solution**:
```bash
chmod +x tests/claude_bridge/*.sh
```

**Issue**: Docker build fails
**Solution**:
```bash
# Clear Docker cache
docker system prune -a
docker build --no-cache -f tests/claude_bridge/Dockerfile -t skill-cli-test .
```

**Issue**: Tests fail with "skill command not found"
**Solution**:
```bash
# Build and add to PATH
cargo build -p skill-cli --release
export PATH=$PATH:$(pwd)/target/release
```

## Environment-Specific Notes

### macOS Quirks
- Requires Xcode Command Line Tools for compilation
- OpenSSL from Homebrew may need explicit `OPENSSL_DIR` env var
- File system is case-insensitive by default (may affect tests)

### Linux Quirks
- Requires explicit installation of `build-essential`
- Some tests may require `sudo` for Docker operations
- File descriptor limits may need adjustment for large tests

### Docker Quirks
- Volume mounts may have different permissions than host
- Network access for MCP tests requires `--network host`
- Some tests may be slower in containerized environment

## Contributing

### Adding New Tests

1. **Unit Test**: Add to relevant module's `tests.rs` file
2. **Integration Test**: Create new bash script in `tests/claude_bridge/`
3. **Acceptance Test**: Add scenario to `tests/claude_bridge/acceptance_tests/scenarios.rs`
4. **Update CI**: Add to `.github/workflows/claude-bridge-tests.yml` if needed

### Test Conventions

- Use descriptive test names: `test_validate_skill_name_with_uppercase`
- Include comments explaining what is being tested
- Clean up test artifacts in teardown
- Use fixtures instead of hardcoding test data
- Follow existing patterns in test organization

## Test Coverage Goals

- **Unit Tests**: 80%+ line coverage
- **Integration Tests**: 70%+ scenario coverage
- **Acceptance Tests**: 95%+ of real-world scenarios
- **Security Tests**: 100% of known vulnerability vectors
- **Performance Tests**: All critical paths benchmarked

## References

- [Claude Agent Skills Specification](https://docs.anthropic.com/claude/docs/claude-agent-skills)
- [Skill Engine Documentation](../docs/)
- [PRD: Claude Bridge Testing](.taskmaster/docs/claude-bridge-testing-prd.txt)
- [Complexity Analysis Report](.taskmaster/reports/task-complexity-report.json)

## Support

For issues or questions:
1. Check this README and troubleshooting section
2. Review PRD and task details: `task-master show 1.1`
3. Check existing test examples
4. Open an issue on GitHub
