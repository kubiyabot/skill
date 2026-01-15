# Contributing to Skill

Thank you for your interest in contributing to Skill! This document provides guidelines and information for contributors.

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Getting Started

### Prerequisites

- **Rust 1.75+** - Install via [rustup](https://rustup.rs/)
- **wasm32-wasip1 target** - For WASM skill compilation
- **Node.js 18+** - For JavaScript/TypeScript skill development
- **Git** - For version control

### Development Setup

```bash
# Clone the repository
git clone https://github.com/kubiyabot/skill.git
cd skill

# Add WASM target
rustup target add wasm32-wasip1

# Build the project
cargo build

# Run tests
cargo test

# Install locally for testing
cargo install --path crates/skill-cli
```

### Project Structure

```
skill/
├── crates/
│   ├── skill-cli/        # CLI binary
│   ├── skill-runtime/    # WASM execution engine
│   ├── skill-mcp/        # MCP server
│   ├── skill-http/       # HTTP server
│   ├── skill-web/        # Web UI
│   └── skill-context/    # Shared types
├── docs/                 # Documentation
├── examples/             # Example skills
├── sdk/                  # Language SDKs
├── tests/                # Integration tests
└── wit/                  # WIT interface definitions
```

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check existing issues to avoid duplicates.

When filing a bug report, include:
- Skill version (`skill --version`)
- Operating system and version
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs or error messages

### Suggesting Features

Feature requests are welcome! Please:
- Check existing issues and discussions first
- Describe the use case and problem you're solving
- Explain your proposed solution
- Consider alternative approaches

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Follow the code style** (see below)
3. **Add tests** for new functionality
4. **Update documentation** as needed
5. **Ensure CI passes** before requesting review

#### PR Checklist

- [ ] Code compiles without warnings (`cargo build`)
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Clippy passes (`cargo clippy`)
- [ ] Documentation is updated (if applicable)
- [ ] CHANGELOG.md is updated (for user-facing changes)

## Code Style

### Rust

We follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy --all-targets --all-features -- -D warnings
```

Key guidelines:
- Use `rustfmt` defaults
- Prefer explicit error handling over `.unwrap()`
- Write doc comments for public APIs
- Keep functions focused and small

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, no code change
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Performance improvement
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(cli): add --verbose flag to run command
fix(runtime): handle empty WASM modules gracefully
docs: update SKILL.md format documentation
```

## Testing

### Running Tests

```bash
# All unit tests
cargo test

# Specific crate
cargo test -p skill-runtime

# Integration tests
./tests/run-all-tests.sh

# MCP integration tests
./tests/mcp_integration_tests.sh
```

### Writing Tests

- Place unit tests in the same file as the code
- Use `#[cfg(test)]` modules
- Integration tests go in `tests/`
- Test both success and error cases

## Documentation

- Update relevant docs when changing functionality
- Use rustdoc comments (`///`) for public APIs
- Keep README.md and docs/ in sync
- Include code examples where helpful

## Release Process

Releases are automated via GitHub Actions when tags are pushed:

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create and push a tag: `git tag v0.x.y && git push --tags`

## Getting Help

- **Questions**: Open a [Discussion](https://github.com/kubiyabot/skill/discussions)
- **Bugs**: Open an [Issue](https://github.com/kubiyabot/skill/issues)
- **Security**: See [SECURITY.md](SECURITY.md)

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
