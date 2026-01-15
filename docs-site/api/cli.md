# CLI Reference

Complete command-line interface reference for Skill Engine.

## Installation Commands

### skill --version
Display the current version.

```bash
skill --version
# Output: skill 0.3.0
```

### skill --help
Show help information.

```bash
skill --help
```

## Discovery Commands

### skill find
Search for tools using natural language.

```bash
skill find <query>
```

**Examples:**
```bash
skill find "kubernetes pods"
skill find "deploy infrastructure"
skill find "query database"
```

### skill list
List all installed skills.

```bash
skill list [skill_name]
```

**Examples:**
```bash
# List all skills
skill list

# List tools in specific skill
skill list kubernetes
```

## Execution Commands

### skill run
Execute a skill tool.

```bash
skill run <skill>:<tool> [args...]
```

**Examples:**
```bash
skill run kubernetes:get --resource pods
skill run github:create-issue --title "Bug" --body "Description"
```

**With named instances:**
```bash
skill run aws@prod:s3-list
skill run db@staging:query --sql "SELECT * FROM users"
```

## Management Commands

### skill install
Install a skill.

```bash
skill install <source>
```

**Sources:**
- Local path: `skill install ./my-skill`
- HTTP URL: `skill install https://example.com/skill.wasm`
- GitHub: `skill install github:user/repo`

**Examples:**
```bash
skill install ./examples/native-skills/kubernetes-skill
skill install https://cdn.example.com/aws-skill.wasm
skill install github:kubiyabot/skill-catalog
```

### skill remove
Uninstall a skill.

```bash
skill remove <skill_name>
```

**Example:**
```bash
skill remove kubernetes
```

### skill config
Configure skill credentials or settings.

```bash
skill config <skill_name> [--instance <name>]
```

**Examples:**
```bash
# Configure default instance
skill config aws

# Configure named instance
skill config aws --instance production
```

## Server Commands

### skill serve
Start the MCP server.

```bash
skill serve [options]
```

**Options:**
- `--port <port>`: HTTP server port (default: 3000)
- `--http`: Enable HTTP mode
- `--skills-dir <path>`: Custom skills directory
- `--debug`: Enable debug logging

**Examples:**
```bash
# Start MCP stdio server
skill serve

# Start HTTP server
skill serve --http --port 8080

# Debug mode
skill serve --debug
```

## History Commands

### skill history
View execution history.

```bash
skill history [options]
```

**Options:**
- `--limit <n>`: Limit results
- `--skill <name>`: Filter by skill
- `--format <format>`: Output format (table, json)

**Examples:**
```bash
skill history
skill history --limit 10
skill history --skill kubernetes
skill history --format json
```

## Utility Commands

### skill info
Show system information.

```bash
skill info
```

Displays:
- Skill Engine version
- Installed skills count
- Skills directory location
- Configuration file location

### skill completions
Generate shell completions.

```bash
skill completions <shell>
```

**Shells:** bash, zsh, fish, powershell

**Examples:**
```bash
# Bash
skill completions bash > ~/.local/share/bash-completion/completions/skill

# Zsh
skill completions zsh > ~/.zfunc/_skill

# Fish
skill completions fish > ~/.config/fish/completions/skill.fish
```

## Global Options

Available for all commands:

- `--help, -h`: Show help
- `--version, -V`: Show version
- `--verbose, -v`: Verbose output
- `--quiet, -q`: Suppress output

## Environment Variables

- `SKILL_ENGINE_CONFIG`: Config file path
- `SKILL_ENGINE_DIR`: Skills directory
- `SKILL_LOG_LEVEL`: Log level (debug, info, warn, error)
- `RUST_LOG`: Rust logging configuration

## Exit Codes

- `0`: Success
- `1`: General error
- `2`: Usage error (invalid arguments)
- `127`: Command not found

## Examples

### Complete Workflow

```bash
# Install skill
skill install ./kubernetes-skill

# Configure credentials
skill config kubernetes

# Search for tools
skill find "list pods"

# Execute tool
skill run kubernetes:get --resource pods --namespace default

# View history
skill history --skill kubernetes
```

### CI/CD Usage

```bash
#!/bin/bash
set -e

# Install skill
skill install ./infrastructure-skill.wasm

# Deploy
skill run infrastructure:deploy \
  --env production \
  --version v1.2.3

# Verify
skill run infrastructure:health-check
```

## See Also

- [Quick Start Guide](../getting-started/quick-start.md)
- [MCP Server Mode](../guides/mcp.md)
- [REST API Reference](./rest.md)
