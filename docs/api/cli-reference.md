# CLI Reference

The `skill` command line interface is your primary tool for managing and using skills.

## Global Flags

- `--version`, `-V`: Print version information.
- `--help`: Print help.
- `--verbose`: Enable verbose output (useful for debugging).

## Commands

### `find`

Semantic search for tools across all installed skills.

```bash
skill find <query>
```

**Examples:**

```bash
skill find "manage k8s pods"
skill find "convert image format"
```

### `run`

Execute a tool.

```bash
skill run <skill>:<tool> [args]
skill run ./local-path <tool> [args]
```

**Examples:**

```bash
skill run aws:s3-list bucket=my-bucket
skill run ./my-skill greet name=Alice
```

### `install`

Install a skill from a source.

```bash
skill install <source> [--instance name]
```

**Sources:**

- Local path: `./path/to/skill`
- HTTP URL: `https://example.com/skill.wasm`
- GitHub: `github:user/repo`

### `list`

List installed skills.

```bash
skill list
```

### `remove`

Uninstall a skill.

```bash
skill remove <skill-name> [--instance name]
```

### `config`

Configure a skill (e.g., set API keys).

```bash
skill config <skill-name>
```

### `serve`

Start the MCP server.

```bash
skill serve [--http] [--port 8080]
```

- `--http`: Enable HTTP mode (includes Web UI).
- `--port`: Port to listen on (default: 3000).

### `claude`

Manage Claude Code integration.

```bash
skill claude setup    # Configure Claude to use Skill Engine
skill claude status   # Check integration status
skill claude remove   # Remove integration
```
