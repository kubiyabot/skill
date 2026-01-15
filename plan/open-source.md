# Plan: Best-in-Class Professional Open Source Project

## Overview

Transform Skill into a premier open source project with professional documentation, examples, and community infrastructure. Live site: https://www.skill-ai.dev/

## Current Strengths (Already Excellent)

- Dual licensing (Apache 2.0 + MIT) with both LICENSE and LICENSE-MIT files
- Professional VitePress docs site with custom design
- One-liner installation with platform detection
- 22 real-world examples across 3 runtime types
- 6 GitHub workflows: release, security-tests, docs, claude-bridge-tests, performance-tests, web-ui-tests
- Issue templates with bug report, feature request, and Q&A links
- Clear value proposition and architecture docs
- Professional README with badges
- Comprehensive test suite (cargo tests, MCP integration, shell tests)

## Gap Analysis

### Tier 1: Critical Gaps

1. **SECURITY.md** - Contains GitHub template text, wrong version numbers
2. **CHANGELOG.md** - Only one entry, no historical context
3. **Missing Files**: SUPPORT.md, ROADMAP.md
4. **CODE_OF_CONDUCT.md** - Has placeholder contact method
5. **Sparse Examples**: node-runner, python-runner barely documented

### Tier 2: Professional Polish

1. No contributor recognition system
2. No community engagement (Discord/Discussions)
3. README missing: code coverage badge, community links
4. No interactive playground or sandbox
5. Limited tutorials (only 1)
6. No video content or GIFs
7. No FAQ section in docs

### Tier 3: World-Class Additions

1. No governance documentation
2. No release process documentation
3. No migration guides
4. No deprecation policy
5. No "Who Uses This" / testimonials section
6. No benchmarks page with reproducible tests

---

## Phase 1: Community Health Files (Foundation)

### 1.1 Rewrite SECURITY.md

**File:** `SECURITY.md`

```markdown
# Security Policy

## Supported Versions

| Version | Supported                     |
| ------- | ----------------------------- |
| 0.3.x   | :white_check_mark:            |
| 0.2.x   | :warning: Security fixes only |
| < 0.2   | :x:                           |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Email: security@kubiya.ai

Include:

- Description of the vulnerability
- Steps to reproduce
- Affected versions
- Potential impact

### Response Timeline

- **Initial Response**: 3 business days
- **Status Update**: 7 business days
- **Fix Timeline**: Based on severity (see below)

### Severity Levels

| Severity | Description                          | Target Fix Time |
| -------- | ------------------------------------ | --------------- |
| Critical | Remote code execution, data breach   | 24-48 hours     |
| High     | Privilege escalation, sandbox escape | 7 days          |
| Medium   | Information disclosure, DoS          | 30 days         |
| Low      | Minor issues, hardening              | Next release    |

### Responsible Disclosure

We follow a 90-day disclosure policy. We will:

1. Acknowledge receipt within 3 business days
2. Validate and assess the vulnerability
3. Develop and test a fix
4. Release a security update
5. Credit you in the advisory (unless you prefer anonymity)

## Security Best Practices for Contributors

- Never commit secrets, API keys, or credentials
- Use `keyring` for credential storage in skills
- Follow WASM sandbox boundaries
- Validate all user inputs in skill implementations
- Review security implications of new features

## Security Features

- **WASM Sandbox**: All skills run in isolated WebAssembly environment
- **Capability-Based Security**: Skills declare required permissions
- **Credential Isolation**: Each skill instance has separate credential storage
- **Audit Logging**: Execution events are logged for security review
```

### 1.2 Expand CHANGELOG.md

**File:** `CHANGELOG.md`

Structure:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- (Track upcoming features here)

### Changed

- (Track changes here)

## [0.3.4] - 2025-12-23

### Added

- Initial open source release
- WASM Component Model execution via Wasmtime
- MCP server with stdio and HTTP streaming
- Semantic search with FastEmbed
- SKILL.md native command execution
- Claude Code integration
- Web UI with skill browser

### Security

- WASM sandbox isolation
- Capability-based permissions model
- Credential storage via system keyring

## [0.3.0] - 2025-12-01

### Added

- RAG search pipeline with hybrid search
- Docker runtime execution mode
- Multi-instance skill configuration

### Changed

- Improved installation script
- Enhanced error messages

## [0.2.0] - 2025-11-01

### Added

- Initial skill-runtime implementation
- Basic CLI commands (install, run, list)
- SKILL.md format specification

## [0.1.0] - 2025-10-01

### Added

- Project initialization
- Core architecture design
- WIT interface definition
```

### 1.3 Create SUPPORT.md

**File:** `SUPPORT.md` (new)

```markdown
# Getting Help

## Documentation

- **Official Docs**: https://www.skill-ai.dev/
- **Quick Start**: https://www.skill-ai.dev/getting-started/quick-start
- **API Reference**: https://www.skill-ai.dev/api/

## Community

### GitHub Discussions

For questions, ideas, and community support:

- **Q&A**: Ask questions and get help
- **Ideas**: Propose new features
- **Show and Tell**: Share your skills and projects
- **General**: Chat with the community

👉 [Join Discussions](https://github.com/kubiyabot/skill/discussions)

### GitHub Issues

For bug reports and feature requests:

- [Report a Bug](https://github.com/kubiyabot/skill/issues/new?template=bug_report.yml)
- [Request a Feature](https://github.com/kubiyabot/skill/issues/new?template=feature_request.yml)

## Before Asking for Help

1. **Check the docs** - Your answer might already be documented
2. **Search existing issues** - Someone may have had the same problem
3. **Minimal reproduction** - If reporting a bug, provide a minimal example

## Response Times

| Channel           | Expected Response    |
| ----------------- | -------------------- |
| Security issues   | 3 business days      |
| Bug reports       | 7 business days      |
| Feature requests  | Community discussion |
| General questions | Community-driven     |

## Commercial Support

For enterprise support, custom skill development, or consulting:

- Contact: support@kubiya.ai
- Website: https://kubiya.ai
```

### 1.4 Create ROADMAP.md

**File:** `ROADMAP.md` (new)

Transform `ideas.md` (80 skill ideas) into a structured roadmap:

```markdown
# Roadmap

This document outlines the future direction of Skill Engine. Dates are estimates, not commitments.

## Vision

Make Skill the universal runtime for AI agent tools - secure, portable, and intelligent.

## Current Focus: v0.4.x (Q1 2025)

### Platform Stability

- [ ] Improved error messages and debugging
- [ ] Performance optimizations
- [ ] Enhanced test coverage (target: 80%)
- [ ] Documentation polish

### Developer Experience

- [ ] `skill init` project scaffolding
- [ ] Hot reload during development
- [ ] Better VS Code integration
- [ ] Skill template gallery

## Next: v0.5.x (Q2 2025)

### Core Skills (Community Priority)

Priority skills based on community feedback:

| Skill            | Status         | Maintainer             |
| ---------------- | -------------- | ---------------------- |
| terraform-skill  | ✅ Available   | Core team              |
| github-skill     | ✅ Available   | Core team              |
| aws-skill        | 🚧 In progress | @contributor           |
| slack-skill      | ✅ Available   | Core team              |
| prometheus-skill | ✅ Available   | Core team              |
| vault-skill      | 📋 Planned     | Looking for maintainer |
| openai-skill     | 📋 Planned     | Looking for maintainer |

### Platform Features

- [ ] Skill registry and discovery
- [ ] Skill versioning and updates
- [ ] Homebrew tap
- [ ] Docker image publication

## Future: v1.0 and Beyond

### Enterprise Features

- [ ] Role-based access control
- [ ] Audit logging dashboard
- [ ] Multi-tenant support
- [ ] SSO integration

### Ecosystem

- [ ] Plugin marketplace
- [ ] Visual skill builder
- [ ] Mobile companion app
- [ ] IDE plugins (VS Code, JetBrains)

## Skill Ideas Backlog

We have 80+ skill ideas across categories:

- DevOps & Infrastructure (10)
- Cloud Providers (8)
- Databases (7)
- CI/CD (7)
- Monitoring (6)
- Communication (6)
- Project Management (6)
- And more...

See [ideas.md](ideas.md) for the full list.

## Contributing to the Roadmap

We welcome community input! To propose features:

1. Open a [Discussion](https://github.com/kubiyabot/skill/discussions/categories/ideas)
2. Explain the use case and value
3. Gather community feedback
4. If accepted, it gets added to the roadmap

## How to Help

| Interest         | How to Contribute                                                                         |
| ---------------- | ----------------------------------------------------------------------------------------- |
| Build a skill    | See [Creating Skills](https://www.skill-ai.dev/guides/skill-development)                  |
| Improve docs     | PRs welcome to `docs-site/`                                                               |
| Fix bugs         | Check [good first issues](https://github.com/kubiyabot/skill/labels/good%20first%20issue) |
| Maintain a skill | Comment on skill issues to volunteer                                                      |
```

### 1.5 Fix CODE_OF_CONDUCT.md

**File:** `CODE_OF_CONDUCT.md`

- Replace `[INSERT CONTACT METHOD]` with: `conduct@kubiya.ai or via GitHub Issues`

---

## Phase 2: Enhanced README.md

### 2.1 Add More Badges

**File:** `README.md`

Add after existing badges:

```markdown
[![CI](https://github.com/kubiyabot/skill/actions/workflows/claude-bridge-tests.yml/badge.svg)](https://github.com/kubiyabot/skill/actions)
[![codecov](https://codecov.io/gh/kubiyabot/skill/branch/main/graph/badge.svg)](https://codecov.io/gh/kubiyabot/skill)
[![GitHub Discussions](https://img.shields.io/github/discussions/kubiyabot/skill)](https://github.com/kubiyabot/skill/discussions)
[![Documentation](https://img.shields.io/badge/docs-skill--ai.dev-blue)](https://www.skill-ai.dev/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE)
```

### 2.2 Add Quick Links Section

After badges:

```markdown
<p align="center">
  <a href="https://www.skill-ai.dev/">Documentation</a> •
  <a href="https://github.com/kubiyabot/skill/discussions">Community</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="CONTRIBUTING.md">Contributing</a> •
  <a href="ROADMAP.md">Roadmap</a>
</p>
```

### 2.3 Add "Who Uses Skill" Section

Before Contributing section:

```markdown
## Who Uses Skill

> "Skill transformed how our team interacts with infrastructure. The WASM sandbox gives us confidence to let AI agents execute real commands."
> — DevOps Lead, Enterprise Customer

Used by teams at:

- [Your Company Here] - We'd love to feature you!

[Tell us your story →](https://github.com/kubiyabot/skill/discussions/categories/show-and-tell)
```

---

## Phase 3: Professional Documentation (docs-site/)

### 3.1 New Getting Started Tutorial

**File:** `docs-site/getting-started/first-skill.md` (new)

Content outline:

```markdown
# Your First Skill (10 minutes)

By the end of this tutorial, you'll have created, installed, and run your own skill.

## What You'll Build

A "weather" skill that returns weather information for a city.

## Prerequisites

- Skill CLI installed (`skill --version`)
- Text editor

## Step 1: Create the Skill Directory

...

## Step 2: Write the SKILL.md

...

## Step 3: Install Your Skill

...

## Step 4: Test It

...

## Step 5: Add More Tools

...

## What's Next?

- [Skill Development Guide](/guides/skill-development)
- [SKILL.md Format Reference](/api/skill-md-format)
- [Example Skills](/examples/)
```

### 3.2 New Tutorials Section

**Directory:** `docs-site/tutorials/` (new)

Files to create:

- `index.md` - Tutorials overview
- `first-wasm-skill.md` - Build a WASM skill with JavaScript
- `first-native-skill.md` - Build a native CLI wrapper skill
- `api-integration.md` - Build a skill that calls external APIs
- `testing-skills.md` - How to test your skills

### 3.3 New Troubleshooting Guide

**File:** `docs-site/guides/troubleshooting.md` (new)

Sections:

- Installation Issues
- Skill Not Found
- Permission Denied
- WASM Compilation Errors
- MCP Connection Issues
- Search Not Working
- Performance Problems

### 3.4 New Cookbook/Recipes

**File:** `docs-site/examples/cookbook.md` (new)

Patterns:

- API with Authentication
- File Processing
- Multi-Tool Skills
- Error Handling
- Configuration Validation
- Streaming Responses

### 3.5 FAQ Page

**File:** `docs-site/faq.md` (new)

Common questions:

- What's the difference between WASM and native skills?
- How do I store credentials securely?
- Can I use Skill without Claude Code?
- How do I debug a skill?
- Is Skill production-ready?

### 3.6 CI/CD Documentation Page

**File:** `docs-site/guides/ci-cd.md` (new)

Document existing GitHub Actions:

```markdown
# CI/CD Pipeline

Skill uses GitHub Actions for continuous integration and deployment.

## Workflows

| Workflow                  | Trigger      | Purpose                                           |
| ------------------------- | ------------ | ------------------------------------------------- |
| `claude-bridge-tests.yml` | Manual/PR    | Unit tests, doc tests, integration tests          |
| `security-tests.yml`      | Manual       | Security scanning (cargo-audit, gitleaks, CodeQL) |
| `docs.yml`                | Push to main | Build and deploy documentation site               |
| `release.yml`             | Tag push     | Multi-platform binary release                     |
| `performance-tests.yml`   | Manual       | Benchmark suite                                   |
| `web-ui-tests.yml`        | Manual       | Frontend tests                                    |

## Running Tests Locally

...

## Security Scanning

...

## Release Process

...
```

### 3.7 Testing Guide Page

**File:** `docs-site/guides/testing.md` (enhance existing or create)

Comprehensive testing documentation:

```markdown
# Testing Guide

## Test Types

### Unit Tests

cargo test --workspace

### Integration Tests

./tests/mcp_integration_tests.sh

### Security Tests

./tests/security/

### Performance Tests

cargo bench -p skill-cli

## Writing Tests for Skills

...

## Mocking and Test Utilities

...

## CI Integration

...
```

### 3.8 Contributing: Bug Reports & Feature Requests

**File:** `docs-site/contributing.md` (enhance)

Add clear section:

```markdown
## Reporting Issues

### Bug Reports

1. Search existing issues first
2. Use the bug report template
3. Include: version, OS, steps to reproduce, expected vs actual

[Report a Bug →](https://github.com/kubiyabot/skill/issues/new?template=bug_report.md)

### Feature Requests

1. Check the roadmap first
2. Discuss in GitHub Discussions
3. Use the feature request template

[Request a Feature →](https://github.com/kubiyabot/skill/issues/new?template=feature_request.md)

### Security Issues

DO NOT open public issues for security vulnerabilities.
Email: security@kubiya.ai

See [SECURITY.md](../SECURITY.md) for details.
```

### 3.9 Update VitePress Config

**File:** `docs-site/.vitepress/config.ts`

Add new navigation:

```typescript
// Add to nav
{ text: 'Tutorials', link: '/tutorials/', activeMatch: '/tutorials/' },

// Add new sidebar section
'/tutorials/': [
  {
    text: 'Tutorials',
    items: [
      { text: 'Overview', link: '/tutorials/' },
      { text: 'Your First WASM Skill', link: '/tutorials/first-wasm-skill' },
      { text: 'Your First Native Skill', link: '/tutorials/first-native-skill' },
      { text: 'API Integration', link: '/tutorials/api-integration' },
      { text: 'Testing Skills', link: '/tutorials/testing-skills' }
    ]
  }
],

// Add to guides sidebar
{ text: 'CI/CD Pipeline', link: '/guides/ci-cd' },
{ text: 'Testing', link: '/guides/testing' },
{ text: 'Troubleshooting', link: '/guides/troubleshooting' },

// Add FAQ to footer/nav
{ text: 'FAQ', link: '/faq' },
```

---

## Phase 4: Professional Examples

### 4.1 Enhance Sparse Examples

**File:** `examples/docker-runtime-skills/node-runner/README.md`

Expand to include:

```markdown
# Node.js Runner Skill

Execute JavaScript/TypeScript code in a sandboxed Node.js environment.

## When to Use

- Run arbitrary JavaScript code safely
- Execute npm scripts
- Process JSON data
- Quick scripting tasks

## Features

- Node.js 20 LTS runtime
- Isolated Docker container
- Network access controlled
- File system sandboxed

## Quick Start

...

## Available Tools

### run

Execute JavaScript code...

### npm-run

Execute npm scripts...

## Examples

...

## Configuration

...

## Security Considerations

...
```

**File:** `examples/docker-runtime-skills/python-runner/README.md`
Similar comprehensive expansion.

### 4.2 Create Tutorial Example

**Directory:** `examples/tutorials/weather-skill/` (new)

Files:

- `SKILL.md` - Heavily commented for learning
- `README.md` - Step-by-step walkthrough
- `expected-output.txt` - What users should see

### 4.3 Create Cookbook Examples

**Directory:** `examples/cookbook/` (new)

Subdirectories:

- `api-with-auth/` - OAuth/API key patterns
- `error-handling/` - Proper error responses
- `multi-tool/` - Skills with related tools
- `config-validation/` - Input validation patterns

---

## Phase 5: GitHub Configuration

### 5.1 Enable GitHub Discussions

**Manual step** - Enable in repository settings:

- Categories: Q&A, Ideas, Show and Tell, General

### 5.2 Issue Labels

Create/update labels:

```
good first issue - green - Easy issues for newcomers
help wanted - yellow - Community contributions welcome
skill-request - blue - Request for new skill
documentation - purple - Documentation improvements
security - red - Security related
breaking-change - orange - Contains breaking changes
```

### 5.3 Update Issue Templates

**File:** `.github/ISSUE_TEMPLATE/config.yml`

Add:

```yaml
contact_links:
  - name: Question or Discussion
    url: https://github.com/kubiyabot/skill/discussions
    about: Ask questions in GitHub Discussions instead of opening an issue
  - name: Documentation
    url: https://www.skill-ai.dev/
    about: Check documentation for answers
```

---

## Phase 6: Cleanup

### 6.1 Move Internal Files

Move to `.internal/` or delete:

- `docs/session-progress.md`
- `docs/testing-progress.md`
- `docs/example-skills-findings.md`

### 6.2 Keep Reference Files

Keep `ideas.md` as source for community to explore skill ideas (referenced in ROADMAP.md)

### 6.3 Remove After Incorporating

- `skills_plan.md` (after ROADMAP.md created)

---

## Implementation Order

### Week 1: Foundation

1. Fix SECURITY.md
2. Expand CHANGELOG.md
3. Create SUPPORT.md
4. Create ROADMAP.md
5. Fix CODE_OF_CONDUCT.md
6. Update README.md badges and sections

### Week 2: Documentation

1. Create first-skill tutorial
2. Create troubleshooting guide
3. Create FAQ page
4. Update VitePress navigation
5. Create cookbook page

### Week 3: Examples

1. Enhance node-runner README
2. Enhance python-runner README
3. Create weather-skill tutorial example
4. Create cookbook examples

### Week 4: Polish

1. Enable GitHub Discussions
2. Configure issue labels
3. Update issue templates
4. Move/cleanup internal files
5. Final review and testing

---

## Verification Checklist

### Community Health

- [ ] `SECURITY.md` - No placeholders, correct versions
- [ ] `CHANGELOG.md` - Multiple versions, follows keepachangelog
- [ ] `SUPPORT.md` - Links work, channels documented
- [ ] `ROADMAP.md` - Vision clear, ways to contribute
- [ ] `CODE_OF_CONDUCT.md` - Contact info present

### Documentation Site

```bash
cd docs-site && npm install && npm run dev
# Verify at http://localhost:5173:
```

- [ ] All new pages render correctly
- [ ] Navigation includes new sections
- [ ] Search finds new content
- [ ] No broken links (run link checker)

### Examples

```bash
# Test tutorial example
skill install ./examples/tutorials/weather-skill
skill run weather:get city="New York"

# Test cookbook examples
skill install ./examples/cookbook/api-with-auth
skill info api-with-auth
```

### README

- [ ] All badges load correctly (CI, codecov, discussions, docs, license)
- [ ] Quick links work
- [ ] Roadmap link works

### GitHub Actions

- [ ] CI badge shows passing status
- [ ] All 6 workflows documented in ci-cd.md
- [ ] Manual workflows can be triggered

### Testing

- [ ] All test commands documented
- [ ] `cargo test --workspace` passes
- [ ] `./tests/mcp_integration_tests.sh` passes
- [ ] Testing guide covers all skill types

### License

- [ ] LICENSE (Apache-2.0) present
- [ ] LICENSE-MIT present
- [ ] README badge links to license
- [ ] CONTRIBUTING.md references dual license

---

## Files Summary

### Create New (15 files)

- `SUPPORT.md`
- `ROADMAP.md`
- `docs-site/getting-started/first-skill.md`
- `docs-site/tutorials/index.md`
- `docs-site/tutorials/first-wasm-skill.md`
- `docs-site/tutorials/first-native-skill.md`
- `docs-site/tutorials/api-integration.md`
- `docs-site/guides/troubleshooting.md`
- `docs-site/guides/ci-cd.md` - CI/CD pipeline documentation
- `docs-site/guides/testing.md` - Comprehensive testing guide
- `docs-site/examples/cookbook.md`
- `docs-site/faq.md`
- `examples/tutorials/weather-skill/` (directory)
- `examples/cookbook/` (directory with subdirs)

### Modify (9 files)

- `SECURITY.md` - Complete rewrite
- `CHANGELOG.md` - Add historical entries
- `CODE_OF_CONDUCT.md` - Fix placeholder
- `README.md` - Add badges (CI, coverage, license, docs, discussions), quick links, testimonials
- `docs-site/.vitepress/config.ts` - New navigation (tutorials, CI/CD, testing, FAQ)
- `docs-site/contributing.md` - Add bug report/feature request section
- `examples/docker-runtime-skills/node-runner/README.md` - Expand
- `examples/docker-runtime-skills/python-runner/README.md` - Expand
- `.github/ISSUE_TEMPLATE/config.yml` - Verify contact links

### Cleanup (3 files)

- `docs/session-progress.md` - Move to .internal/
- `docs/testing-progress.md` - Move to .internal/
- `skills_plan.md` - Remove after roadmap

---

## Success Metrics

After implementation, the project should:

- Pass GitHub Community Standards check (100%)
- Have clear paths for users AND contributors
- Provide tutorials from beginner to advanced
- Document all common issues
- Make it easy to ask for help
- Show active roadmap and vision
- Recognize contributors
- Enable community discussions
