# Contributing to Skill Engine

Thank you for your interest in contributing to Skill Engine! This guide will help you get started.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Release Process](#release-process)

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please:

- Be respectful and professional
- Welcome newcomers and help them get started
- Focus on constructive feedback
- Respect differing viewpoints and experiences

Report unacceptable behavior to security@kubiya.ai.

## Getting Started

### Prerequisites

- **Rust** 1.75.0 or higher
- **Cargo** (comes with Rust)
- **Git**
- **Node.js** 18+ (for documentation site)

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/skill.git
   cd skill
   ```
3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/kubiyabot/skill.git
   ```

## Development Setup

### Build from Source

```bash
# Install dependencies and build
cargo build

# Run tests
cargo test

# Run CLI in development
cargo run -- --help
```

### Install Development Version

```bash
# Install locally
cargo install --path crates/skill-cli

# Verify installation
skill --version
```

### IDE Setup

**VS Code** (Recommended):
```bash
# Install Rust analyzer
code --install-extension rust-lang.rust-analyzer

# Open workspace
code .
```

**RustRover / IntelliJ**:
- Open the project directory
- Rust plugin should auto-detect the project

## Project Structure

```
skill/
├── crates/
│   ├── skill-cli/           # CLI binary and commands
│   │   ├── src/
│   │   │   ├── commands/    # Command implementations
│   │   │   ├── main.rs      # CLI entry point
│   │   │   └── config.rs    # Configuration
│   │   └── Cargo.toml
│   │
│   ├── skill-runtime/       # Core runtime and skill execution
│   │   ├── src/
│   │   │   ├── manifest.rs  # Skill manifest parsing
│   │   │   ├── executor.rs  # Execution logic
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── skill-mcp/          # Model Context Protocol implementation
│   │   ├── src/
│   │   │   ├── server.rs   # MCP server
│   │   │   ├── protocol.rs # Protocol definitions
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── skill-http/         # HTTP server and REST API
│   │   ├── src/
│   │   │   ├── handlers.rs # API handlers
│   │   │   ├── routes.rs   # Route definitions
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── skill-web/          # Web UI (WASM)
│   │   ├── src/
│   │   │   ├── main.rs     # Yew app
│   │   │   └── components/ # UI components
│   │   └── Cargo.toml
│   │
│   └── skill-context/      # RAG and semantic search
│       ├── src/
│       │   ├── embeddings.rs # Embedding generation
│       │   ├── search.rs     # Search logic
│       │   └── lib.rs
│       └── Cargo.toml
│
├── docs-site/              # VitePress documentation
│   ├── .vitepress/
│   │   ├── config.ts
│   │   └── theme/
│   ├── getting-started/
│   ├── guides/
│   ├── api/
│   └── examples/
│
├── examples/               # Example skills
│   ├── native-skills/
│   ├── docker-skills/
│   └── wasm-skills/
│
└── tests/                  # Integration tests
    ├── mcp_integration_tests.sh
    └── test-manifests.sh
```

## Making Changes

### Branching Strategy

- `main` - stable release branch
- `develop` - integration branch for next release
- `feature/*` - feature branches
- `fix/*` - bug fix branches
- `docs/*` - documentation changes

### Create a Feature Branch

```bash
# Update your fork
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/my-awesome-feature
```

### Development Workflow

1. **Make Changes**: Edit code following our style guide
2. **Add Tests**: Write tests for new functionality
3. **Run Tests**: Ensure all tests pass
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```
4. **Update Docs**: Add/update documentation
5. **Commit**: Write clear commit messages

### Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements

**Examples**:
```bash
feat(cli): add semantic search command

Implements `skill find` command with RAG-based search.
Uses OpenAI embeddings for tool discovery.

Closes #42
```

```bash
fix(runtime): resolve WASM memory leak

Fixed memory leak in long-running WASM skills by properly
cleaning up instances after execution.

Fixes #156
```

## Testing

### Run All Tests

```bash
# Unit tests
cargo test

# Integration tests
./tests/mcp_integration_tests.sh
./tests/test-manifests.sh
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage
```

### Writing Tests

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parsing() {
        let manifest = r#"
            name = "test-skill"
            version = "0.1.0"
        "#;

        let parsed = parse_manifest(manifest).unwrap();
        assert_eq!(parsed.name, "test-skill");
    }
}
```

**Integration Tests**:
```bash
#!/bin/bash
# tests/test_new_feature.sh

skill install ./test-skill
output=$(skill run test-skill:test-tool)
[[ "$output" == *"expected"* ]] || exit 1
```

## Documentation

### Code Documentation

Add doc comments to all public items:

```rust
/// Executes a skill tool with the given parameters.
///
/// # Arguments
///
/// * `skill_name` - Name of the skill to execute
/// * `tool_name` - Name of the tool within the skill
/// * `params` - Tool parameters as key-value pairs
///
/// # Returns
///
/// Returns the tool output as a string or an error.
///
/// # Examples
///
/// ```rust
/// let output = execute_tool("kubernetes", "get", params)?;
/// println!("Output: {}", output);
/// ```
pub fn execute_tool(
    skill_name: &str,
    tool_name: &str,
    params: HashMap<String, String>
) -> Result<String> {
    // Implementation
}
```

### Documentation Site

Update the VitePress documentation site:

```bash
cd docs-site

# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build
```

Add new pages in `docs-site/`:
- Getting Started: `getting-started/*.md`
- Guides: `guides/*.md`
- API Reference: `api/*.md`
- Examples: `examples/*.md`

## Submitting Changes

### Pre-Submission Checklist

- [ ] All tests pass (`cargo test`)
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Commit messages follow convention

### Create Pull Request

1. **Push Changes**:
   ```bash
   git push origin feature/my-awesome-feature
   ```

2. **Open PR**: Go to GitHub and create a pull request

3. **PR Description**: Include:
   - What changes were made
   - Why the changes were needed
   - How to test the changes
   - Related issues (Closes #123)

4. **Review Process**:
   - Maintainers will review your PR
   - Address feedback and push updates
   - Once approved, PR will be merged

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Tests pass locally
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
```

## Release Process

**For Maintainers**:

1. **Version Bump**:
   ```bash
   # Update version in all Cargo.toml files
   ./scripts/bump-version.sh 0.4.0
   ```

2. **Update Changelog**:
   ```bash
   # Update CHANGELOG.md with release notes
   vim CHANGELOG.md
   ```

3. **Create Release**:
   ```bash
   git tag -a v0.4.0 -m "Release v0.4.0"
   git push origin v0.4.0
   ```

4. **Publish**:
   ```bash
   # Publish to crates.io
   cargo publish -p skill-runtime
   cargo publish -p skill-mcp
   cargo publish -p skill-http
   cargo publish -p skill-context
   cargo publish -p skill-cli
   ```

5. **GitHub Release**:
   - Create release on GitHub
   - Upload binary artifacts
   - Publish release notes

## Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and community discussions
- **Discord**: Real-time chat (link in README)

### Questions?

If you have questions about contributing:

1. Check existing documentation
2. Search GitHub issues
3. Ask in GitHub Discussions
4. Reach out on Discord

## Recognition

All contributors are recognized in:
- GitHub contributors page
- Release notes
- Project README

Thank you for contributing to Skill Engine! 🚀
