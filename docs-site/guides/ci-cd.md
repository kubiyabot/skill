# CI/CD Pipeline

Skill uses GitHub Actions for continuous integration, testing, and deployment. This page documents all workflows and how to use them.

## Workflows Overview

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| [claude-bridge-tests.yml](#claude-bridge-tests) | Manual | Unit tests, integration tests, code coverage |
| [docs.yml](#documentation-deployment) | Push to main | Build and deploy documentation site |
| [release.yml](#release-pipeline) | Tag push (`v*`) | Multi-platform binary release |
| [publish.yml](#cratesio-publishing) | Release / Manual | Publish crates to crates.io |
| [security-tests.yml](#security-tests) | Manual | Security scanning and vulnerability detection |
| [performance-tests.yml](#performance-tests) | Manual | Benchmarks and scalability testing |
| [web-ui-tests.yml](#web-ui-tests) | Manual | Frontend and backend integration tests |

## Claude Bridge Tests

**File:** `.github/workflows/claude-bridge-tests.yml`

The main test workflow for the Claude Code integration.

### What It Tests

| Job | Platform | Description |
|-----|----------|-------------|
| Unit Tests | Linux + macOS | Tests `claude_bridge` module |
| Doc Tests | Linux + macOS | Documentation examples |
| Integration Tests | Linux | Shell-based integration tests |
| Code Coverage | Linux | Coverage with tarpaulin, uploads to Codecov |
| Performance Tests | Linux | Optional, triggered with `[perf]` in commit |

### Running Locally

```bash
# Unit tests
cargo test -p skill-cli --lib -- claude_bridge --nocapture

# Doc tests
cargo test -p skill-cli --doc -- claude_bridge

# Integration tests
./tests/claude_bridge/test-fresh-install.sh
./tests/claude_bridge/test-skill-generation.sh
```

### Manual Trigger

1. Go to **Actions** → **Claude Bridge Tests**
2. Click **Run workflow**
3. Optionally enable performance tests

## Documentation Deployment

**File:** `.github/workflows/docs.yml`

Automatically builds and deploys the documentation site to GitHub Pages.

### Triggers

- Push to `main` branch (with changes in `docs-site/` or `crates/`)
- Pull requests to `main` (build only, no deploy)
- Manual dispatch

### What It Does

1. Builds Rust API documentation (`cargo doc`)
2. Compiles skill-runtime to WASM for playground
3. Builds VitePress site
4. Deploys to GitHub Pages (on main branch only)

### Running Locally

```bash
cd docs-site

# Install dependencies
npm install

# Development server
npm run dev

# Production build
npm run build
```

## Release Pipeline

**File:** `.github/workflows/release.yml`

Creates releases when a version tag is pushed.

### Supported Platforms

| Platform | Architecture | Binary |
|----------|-------------|--------|
| Linux | x86_64 (glibc) | `skill-x86_64-unknown-linux-gnu.tar.gz` |
| macOS | aarch64 (Apple Silicon) | `skill-aarch64-apple-darwin.tar.gz` |

> **Note:** For other platforms (Linux ARM, macOS Intel), build from source with `cargo install`.

### Creating a Release

```bash
# Update version in Cargo.toml files
# Update CHANGELOG.md

# Create and push tag
git tag v0.4.0
git push origin v0.4.0
```

### What It Does

1. Builds binaries for all platforms
2. Builds Web UI with Tailwind CSS
3. Creates SHA256 checksums
4. Uploads to Vercel Blob storage
5. Creates GitHub Release with download links

### Download URLs

After release, binaries are available at:
- Versioned: `https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/releases/v{VERSION}/skill-{target}.tar.gz`
- Latest: `https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/releases/latest/skill-{target}.tar.gz`

## Crates.io Publishing

**File:** `.github/workflows/publish.yml`

Publishes Skill crates to [crates.io](https://crates.io) for Rust developers.

### Triggers

- **Automatic:** When a GitHub Release is published
- **Manual:** Workflow dispatch with optional dry-run mode

### Published Crates

Crates are published in dependency order:

| Order | Crate | Description |
|-------|-------|-------------|
| 1 | [skill-context](https://crates.io/crates/skill-context) | Execution context management |
| 2 | [skill-runtime](https://crates.io/crates/skill-runtime) | Core execution engine (WASM, Docker, native) |
| 3 | [skill-mcp](https://crates.io/crates/skill-mcp) | MCP server implementation |
| 4 | [skill-http](https://crates.io/crates/skill-http) | HTTP streaming server |
| 5 | [skill-cli](https://crates.io/crates/skill-cli) | Command-line interface |

> **Note:** `skill-web` is not published to crates.io as it's a WASM-only frontend crate.

### Manual Publishing (Dry Run)

1. Go to **Actions** → **Publish to crates.io**
2. Click **Run workflow**
3. Enable **Dry run** to test without publishing
4. Click **Run workflow**

### Publishing a Release

When you create a GitHub Release, the workflow automatically publishes all crates:

```bash
# Ensure version is updated in Cargo.toml
# Create and push a tag
git tag v0.4.0
git push origin v0.4.0

# Create a GitHub Release from the tag
# → This triggers the publish workflow
```

### Requirements

| Secret | Purpose |
|--------|---------|
| `CARGO_REGISTRY_TOKEN` | API token from [crates.io](https://crates.io/settings/tokens) |

### Troubleshooting

**Version mismatch error:**
Ensure the tag version matches `Cargo.toml` workspace version.

**Crate already exists:**
Each version can only be published once. Bump the version and create a new release.

**Dependency not found:**
Crates are published with 30-second waits between each to allow crates.io index propagation.

## Security Tests

**File:** `.github/workflows/security-tests.yml`

Comprehensive security scanning workflow (manual trigger only).

### Security Checks

| Job | Tool | Purpose |
|-----|------|---------|
| Security Tests | cargo test | Custom security test suite |
| Cargo Audit | cargo-audit | Known vulnerability detection |
| Cargo Deny | cargo-deny | License and advisory checks |
| Secrets Scan | gitleaks | Hardcoded secrets detection |
| SAST Scan | CodeQL | Static application security testing |
| Permissions Check | find | File permission audit |

### Running Locally

```bash
# Security tests
cargo test --test security_tests --package skill-cli -- --ignored --nocapture

# Cargo audit
cargo install cargo-audit
cargo audit

# Cargo deny
cargo install cargo-deny
cargo deny check
```

### Severity Responses

| Check | On Failure |
|-------|------------|
| Security Tests | Block release |
| Cargo Audit | Block release |
| Secrets Scan | Block release |
| SAST Scan | Review findings |

## Performance Tests

**File:** `.github/workflows/performance-tests.yml`

Benchmarking and scalability testing (manual trigger only).

### Test Suites

| Job | Description |
|-----|-------------|
| Criterion Benchmarks | Micro-benchmarks with regression detection |
| Performance Test Suite | End-to-end performance tests |
| Memory Profiling | Valgrind/massif memory analysis |
| Scalability Test | 100-skill generation test |

### Running Locally

```bash
# Criterion benchmarks
cargo bench --bench claude_bridge_bench

# Performance tests
./tests/claude_bridge/test-performance.sh

# Generate large manifests for testing
./tests/claude_bridge/generate-large-manifest.sh
```

### Performance Targets

| Metric | Target |
|--------|--------|
| Cold start | < 100ms |
| Warm start | < 10ms |
| Semantic search | < 50ms |
| 100 skills generation | < 60s |

## Web UI Tests

**File:** `.github/workflows/web-ui-tests.yml`

Frontend and backend integration tests (manual trigger only).

### Test Coverage

| Component | Tests | Framework |
|-----------|-------|-----------|
| Backend Integration | 62 tests | cargo test |
| Frontend WASM | 67 tests | wasm-pack test |

### Running Locally

```bash
# Backend tests
cargo test -p skill-http --tests

# Frontend WASM tests (requires Chrome)
cd crates/skill-web
wasm-pack test --headless --chrome
```

## Running All Tests

The complete test suite can be run with:

```bash
# Quick test (most common)
cargo test --workspace

# Full test suite with integration tests
./tests/run-all-tests.sh

# MCP protocol tests
./tests/mcp_integration_tests.sh
```

## CI Badges

Add these badges to your README:

```markdown
[![CI](https://github.com/kubiyabot/skill/actions/workflows/claude-bridge-tests.yml/badge.svg)](https://github.com/kubiyabot/skill/actions)
[![Security](https://github.com/kubiyabot/skill/actions/workflows/security-tests.yml/badge.svg)](https://github.com/kubiyabot/skill/actions)
```

## Required Secrets

Configure these in repository settings:

| Secret | Purpose |
|--------|---------|
| `CODECOV_TOKEN` | Code coverage uploads |
| `BLOB_READ_WRITE_TOKEN` | Vercel Blob storage for releases |
| `CARGO_REGISTRY_TOKEN` | Publishing to crates.io |
| `GITHUB_TOKEN` | Automatic (GitHub Actions) |

## Troubleshooting CI

### Build Fails on Linux

Some dependencies have proc-macro compatibility issues on Linux. The workflows use macOS as a workaround for security and performance tests.

### Coverage Report Missing

Ensure `CODECOV_TOKEN` is set in repository secrets.

### Release Artifacts Not Uploading

Check that `BLOB_READ_WRITE_TOKEN` is configured for Vercel Blob storage.

### Tests Timing Out

Increase the timeout in the workflow file:
```yaml
timeout-minutes: 60
```
