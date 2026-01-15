# Frequently Asked Questions

## General

### What is Skill?

Skill is a universal runtime for AI agent tools. It allows AI agents like Claude to execute real-world tasks (query databases, manage infrastructure, call APIs) in a secure, sandboxed environment.

### How is Skill different from MCP?

Skill complements MCP rather than replacing it:

| Aspect | MCP Servers | Skill |
|--------|-------------|-------|
| Architecture | Always-on servers | On-demand execution |
| Deployment | Per-service process | Single binary |
| Tool Discovery | Static config | Semantic search |
| Security | Varies | WASM sandbox |

Skill can also act as an MCP server, providing the best of both worlds.

### Is Skill production-ready?

Yes, for many use cases. Skill is used in production by teams at Kubiya and others. However:
- Review security implications for your use case
- Test thoroughly before deploying
- Check the [roadmap](ROADMAP.md) for planned features

### What languages can I write skills in?

**WASM Skills:**
- JavaScript/TypeScript
- Python (via SDK)
- Rust
- Any language that compiles to WASM

**Native Skills:**
- SKILL.md format wraps any CLI tool

## Skills & Tools

### What's the difference between WASM and native skills?

| Aspect | WASM Skills | Native Skills |
|--------|-------------|---------------|
| Execution | Sandboxed WASM | Shell commands |
| Security | Full isolation | Command allowlist |
| Capabilities | Custom logic | CLI wrappers |
| Use Case | API integration | kubectl, git, etc. |

### Can I use npm packages in WASM skills?

Not directly. WASM skills run in an isolated environment without Node.js. You can:
- Bundle dependencies at build time
- Use web-compatible APIs (fetch, crypto)
- Keep skills focused and lightweight

### How do I store API keys securely?

Use the `skill config` command:

```bash
skill config my-skill
# Enter API key when prompted
```

Keys are stored in your system keyring, not in files.

### Can skills call other skills?

Yes, using the `skill-run` allowed tool:

```yaml
allowed-tools: Bash, skill-run
```

Then in your skill:
```bash
skill run other-skill:tool param="value"
```

## Security

### Is it safe to run skills from untrusted sources?

Exercise caution:
- Review SKILL.md before installing
- Check the allowed-tools list
- WASM skills are sandboxed, but native skills run real commands
- Only install from trusted sources

### Can WASM skills access my filesystem?

No, unless explicitly granted. WASM skills run in a sandbox that:
- Cannot read/write arbitrary files
- Cannot make network requests (unless allowed)
- Cannot access other processes

### How does the command allowlist work?

Native skills declare allowed commands:

```yaml
allowed-tools: kubectl, helm
```

Only these commands can be executed. The Bash exception allows shell commands but is restricted to the allowlist.

## MCP Integration

### How do I use Skill with Claude Code?

```bash
# Automatic setup
skill claude setup

# Verify
skill claude status
```

This configures the MCP server in your project.

### Can I use Skill without Claude?

Yes! Skill works as a standalone CLI:

```bash
skill install ./my-skill
skill run my-skill:tool param="value"
```

No Claude or MCP required.

### Why aren't my skills showing in Claude?

1. Check setup: `skill claude status`
2. Verify MCP config: `cat .mcp.json`
3. Restart Claude Code
4. Check for errors in MCP logs

## Performance

### How fast is Skill?

Typical performance:
- Cold start: ~100ms
- Warm start: <10ms
- Semantic search: <50ms
- MCP tool call: <100ms

### Why is the first search slow?

The first search loads the embedding model and builds the index. Subsequent searches use cached data and are much faster.

### How can I improve performance?

1. Keep skills small and focused
2. Use native skills for CLI operations
3. Pre-warm with a simple query
4. Enable caching where available

## Installation

### Which platforms are supported?

- **macOS**: x86_64 and Apple Silicon (arm64)
- **Linux**: x86_64 (glibc-based distributions)
- **Windows**: Via WSL2

### Can I install without curl?

Yes, build from source:

```bash
git clone https://github.com/kubiyabot/skill
cd skill
cargo install --path crates/skill-cli
```

### How do I update Skill?

```bash
# Re-run the installer
curl -fsSL https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/install.sh | sh

# Or from source
git pull
cargo install --path crates/skill-cli
```

## Development

### How do I debug a skill?

```bash
# Enable debug logging
RUST_LOG=debug skill run my-skill:tool

# Check skill info
skill info my-skill

# Verify SKILL.md parsing
cat ./my-skill/SKILL.md
```

### Can I hot-reload during development?

Not yet, but it's on the roadmap. For now:

```bash
# Reinstall after changes
skill install ./my-skill
```

### How do I test my skill?

See the [Testing Tutorial](/tutorials/testing-skills) for comprehensive testing strategies.

## Contributing

### How do I contribute a skill?

1. Create your skill following the [development guide](/guides/skill-development)
2. Add to `examples/` directory
3. Submit a pull request
4. See [CONTRIBUTING.md](https://github.com/kubiyabot/skill/blob/main/CONTRIBUTING.md)

### How do I report a bug?

Open an issue at: https://github.com/kubiyabot/skill/issues/new

Include:
- Skill version (`skill --version`)
- Operating system
- Steps to reproduce
- Error messages

### How do I request a feature?

1. Check the [roadmap](ROADMAP.md)
2. Search existing issues
3. Open a discussion: https://github.com/kubiyabot/skill/discussions

## Still Have Questions?

- **Documentation**: https://www.skill-ai.dev/
- **Discussions**: https://github.com/kubiyabot/skill/discussions
- **Issues**: https://github.com/kubiyabot/skill/issues
