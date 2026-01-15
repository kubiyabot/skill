# Roadmap

This document outlines the future direction of Skill Engine. Dates are estimates, not commitments. We welcome community input on priorities.

## Vision

Make Skill the universal runtime for AI agent tools - secure, portable, and intelligent.

## Current Focus: v0.4.x

### Platform Stability
- [ ] Improved error messages with actionable suggestions
- [ ] Performance optimizations for large skill catalogs
- [ ] Enhanced test coverage (target: 80%)
- [ ] Documentation polish and tutorials

### Developer Experience
- [ ] `skill init` - Project scaffolding command
- [ ] Hot reload during skill development
- [ ] Better VS Code integration
- [ ] Skill template gallery

### Documentation
- [ ] Complete tutorial series
- [ ] Video walkthroughs
- [ ] Cookbook with common patterns
- [ ] Troubleshooting guide

## Next: v0.5.x

### Core Skills

Priority skills based on community feedback:

| Skill | Status | Category |
|-------|--------|----------|
| terraform-skill | :white_check_mark: Available | DevOps |
| kubernetes-skill | :white_check_mark: Available | DevOps |
| github-skill | :white_check_mark: Available | CI/CD |
| git-skill | :white_check_mark: Available | CI/CD |
| postgres-skill | :white_check_mark: Available | Database |
| redis-skill | :white_check_mark: Available | Database |
| slack-skill | :white_check_mark: Available | Communication |
| prometheus-skill | :white_check_mark: Available | Monitoring |
| docker-skill | :white_check_mark: Available | DevOps |
| http-skill | :white_check_mark: Available | Network |
| vault-skill | :construction: Planned | Security |
| openai-skill | :construction: Planned | AI/ML |
| aws-skill | :construction: In Progress | Cloud |

### Platform Features
- [ ] Skill registry and discovery service
- [ ] Skill versioning and automatic updates
- [ ] Homebrew tap for easy installation
- [ ] Official Docker image
- [ ] Pre-built binaries for more platforms

## Future: v1.0 and Beyond

### Enterprise Features
- [ ] Role-based access control (RBAC)
- [ ] Audit logging dashboard
- [ ] Multi-tenant support
- [ ] SSO integration (SAML, OIDC)
- [ ] Air-gapped deployment support

### Ecosystem
- [ ] Public skill marketplace
- [ ] Visual skill builder (no-code)
- [ ] IDE plugins (VS Code, JetBrains)
- [ ] GitHub Action for skill CI/CD
- [ ] Skill dependency management

### Advanced Features
- [ ] Skill composition and chaining
- [ ] Async/background skill execution
- [ ] Webhook triggers
- [ ] Scheduled skill runs

## Skill Ideas Backlog

We have 80+ skill ideas across 16 categories. Here's a summary:

| Category | Count | Examples |
|----------|-------|----------|
| DevOps & Infrastructure | 10 | ansible, helm, argocd, pulumi |
| Cloud Providers | 8 | aws, gcp, azure, cloudflare |
| Databases | 7 | mysql, mongodb, elasticsearch |
| CI/CD & Version Control | 7 | gitlab, jenkins, circleci |
| Monitoring & Observability | 6 | grafana, datadog, sentry |
| Communication | 6 | discord, teams, email, twilio |
| Project Management | 6 | jira, linear, notion, asana |
| Security | 5 | trivy, snyk, 1password, okta |
| AI/ML | 5 | ollama, huggingface, pinecone |
| File & Data Processing | 5 | ffmpeg, imagemagick, pandoc |
| Testing | 5 | playwright, postman, k6 |
| Network & API | 5 | curl, dns, websocket, grpc |
| Payments & Business | 5 | stripe, shopify, hubspot |

See [ideas.md](ideas.md) for the complete list with descriptions.

## Contributing to the Roadmap

We welcome community input! Here's how to participate:

### Propose a Feature
1. Open a [Discussion](https://github.com/kubiyabot/skill/discussions/categories/ideas)
2. Explain the use case and value
3. Gather community feedback
4. If accepted, it gets added to the roadmap

### Build a Skill
1. Check the [Skill Ideas](ideas.md) for inspiration
2. Follow the [Skill Development Guide](https://www.skill-ai.dev/guides/skill-development)
3. Submit a PR to add your skill to examples

### Contribute Code
1. Check [good first issues](https://github.com/kubiyabot/skill/labels/good%20first%20issue)
2. Read the [Contributing Guide](CONTRIBUTING.md)
3. Join the discussion on GitHub

## How to Help

| Interest | How to Contribute |
|----------|-------------------|
| Build a skill | See [Skill Development](https://www.skill-ai.dev/guides/skill-development) |
| Improve docs | PRs welcome to `docs-site/` directory |
| Fix bugs | Check [issues](https://github.com/kubiyabot/skill/issues) |
| Add tests | Improve coverage in `tests/` |
| Review PRs | Help review open pull requests |

## Release Schedule

We aim for regular releases:
- **Patch releases** (0.x.y): As needed for bug fixes
- **Minor releases** (0.x.0): Monthly with new features
- **Major releases** (x.0.0): When ready, with stability guarantees

## Feedback

Have thoughts on the roadmap? We'd love to hear from you:
- [GitHub Discussions](https://github.com/kubiyabot/skill/discussions)
- [Open an Issue](https://github.com/kubiyabot/skill/issues/new)

---

*Last updated: January 2025*
