# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Professional open source documentation
- Tutorials section with step-by-step guides
- FAQ and troubleshooting documentation
- CI/CD pipeline documentation
- Cookbook with common patterns

### Changed
- Enhanced SECURITY.md with responsible disclosure policy
- Expanded example documentation (node-runner, python-runner)

## [0.3.4] - 2025-12-23

### Added
- Initial open source release of Skill Engine
- Core WASM runtime with Component Model support via Wasmtime
- CLI for skill management (`skill` command)
- Support for JavaScript, TypeScript, and Python skills
- Built-in examples for AWS, GitHub, Kubernetes, and more
- Documentation site at https://www.skill-ai.dev/
- MCP server with stdio and HTTP streaming modes
- Semantic search with FastEmbed for tool discovery
- SKILL.md format for native command execution
- Claude Code integration (`skill claude setup`)
- Web UI with skill browser and execution interface

### Changed
- Refactored internal crate structure for better modularity
- Improved error handling in WASM sandbox

### Security
- Implemented capability-based security model
- Added encrypted credential storage using OS keychain
- WASM sandbox isolation via Wasmtime

## [0.3.0] - 2025-12-01

### Added
- RAG search pipeline with hybrid search (vector + BM25)
- Docker runtime execution mode for containerized skills
- Multi-instance skill configuration
- Cross-encoder reranking for improved search accuracy
- Query understanding with intent classification

### Changed
- Improved installation script with platform detection
- Enhanced error messages with actionable suggestions
- Optimized embedding model loading

### Fixed
- Memory leak in long-running MCP server sessions
- Search index corruption on concurrent updates

## [0.2.0] - 2025-11-01

### Added
- Initial skill-runtime implementation
- Basic CLI commands: install, run, list, remove, info
- SKILL.md format specification for native skills
- Git-based skill installation support
- Manifest configuration (`.skill-engine.toml`)

### Changed
- Migrated to Wasmtime 26.0
- Improved startup time with lazy initialization

### Fixed
- Windows path handling in skill installation
- Unicode handling in skill output

## [0.1.0] - 2025-10-01

### Added
- Project initialization and architecture design
- WIT interface definition for skill components
- Basic WASM executor prototype
- Initial documentation structure

---

[Unreleased]: https://github.com/kubiyabot/skill/compare/v0.3.4...HEAD
[0.3.4]: https://github.com/kubiyabot/skill/compare/v0.3.0...v0.3.4
[0.3.0]: https://github.com/kubiyabot/skill/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/kubiyabot/skill/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/kubiyabot/skill/releases/tag/v0.1.0
