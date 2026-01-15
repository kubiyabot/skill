# Changelog

All notable changes to Skill Engine will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2024-01-15

### Added

#### Claude Bridge
- **NEW**: Claude Bridge for generating Claude Agent Skills from Skill Markdown
- Automatic conversion of `.skill.md` files to Claude Agent SDK format
- Validation and error handling for skill manifests
- Support for native CLI, Docker, and WASM runtimes
- Comprehensive test suite for edge cases

#### Documentation
- Complete VitePress documentation site
- Interactive API explorer
- MCP protocol reference
- Claude Code integration guide
- 15+ example skill guides

#### Core Features
- MCP server mode with stdio transport
- REST API with OpenAPI specification
- Semantic search for skill discovery
- Execution history tracking
- Multi-instance skill support

### Changed
- Improved error messages across all commands
- Enhanced logging with structured output
- Better performance for WASM runtime initialization

### Fixed
- WASM skill loading on Windows
- Docker skill networking issues
- Race condition in concurrent executions

## [0.2.0] - 2024-12-01

### Added

#### Runtimes
- WASM runtime support via WASI
- Docker runtime with resource limits
- Native CLI runtime with allowlisting

#### Discovery
- `skill find` command for semantic search
- RAG-based tool discovery
- Embedding generation for all tools

#### Management
- `skill config` for credential management
- `skill history` for execution tracking
- Multi-instance configuration support

### Changed
- Migrated from custom protocol to MCP
- Refactored skill loading for better performance
- Improved CLI output formatting

### Fixed
- Memory leaks in long-running server mode
- Path resolution on Windows
- Permission errors with Docker skills

## [0.1.0] - 2024-10-15

### Added
- Initial release
- Basic skill execution with `skill run`
- Skill installation from local paths
- Native CLI runtime support
- Simple skill manifest format
- `skill list` command

### Known Issues
- No semantic search
- Limited error handling
- Manual configuration only

## [Unreleased]

### Planned for 0.4.0
- HTTP/SSE transport for MCP
- WebSocket support for streaming
- GraphQL API
- Skill marketplace integration
- Remote skill execution
- Skill versioning and updates
- Enhanced WASM capabilities
- Plugin system for custom runtimes

### Under Consideration
- GUI for skill management
- Skill testing framework
- Performance profiling tools
- Skill templates and scaffolding
- Team collaboration features
- Skill analytics dashboard

## Migration Guides

### Upgrading from 0.2.x to 1.0.0

**Breaking Changes**: None

**New Features**:
1. **Claude Bridge**: Use `skill claude-bridge` to generate Claude Agent Skills
2. **Enhanced Documentation**: Visit the new docs site for comprehensive guides
3. **OpenAPI Spec**: REST API now includes full OpenAPI specification

**Recommended Actions**:
```bash
# Update to latest version
curl -fsSL https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/install.sh | sh

# Verify installation
skill --version  # Should show 1.0.0

# Try new Claude Bridge feature
skill claude-bridge generate examples/native-skills/kubernetes-skill/SKILL.md
```

### Upgrading from 0.1.x to 0.2.0

**Breaking Changes**:
- Skill manifest format changed (automatic migration on first run)
- Custom protocol replaced with MCP (update client integrations)

**New Features**:
1. **Semantic Search**: Use `skill find "query"` instead of browsing
2. **WASM Runtime**: Install WASM skills for better security
3. **MCP Integration**: Connect to Claude Code and other AI agents

**Migration Steps**:
```bash
# Backup existing skills
cp -r ~/.skill ~/.skill.backup

# Update to 0.2.0
curl -fsSL https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/install.sh | SKILL_VERSION=0.2.0 sh

# Migrate skills (automatic)
skill list  # Triggers migration

# Update MCP configuration
cat >> ~/.config/claude/mcp.json << 'EOF'
{
  "mcpServers": {
    "skill-engine": {
      "command": "skill",
      "args": ["serve"]
    }
  }
}
EOF
```

## Support

- **Bug Reports**: [GitHub Issues](https://github.com/kubiyabot/skill/issues)
- **Feature Requests**: [GitHub Discussions](https://github.com/kubiyabot/skill/discussions)
- **Security Issues**: security@kubiya.ai

## License

Skill Engine is released under the [MIT License](https://github.com/kubiyabot/skill/blob/main/LICENSE).
