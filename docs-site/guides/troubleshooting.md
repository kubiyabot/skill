# Troubleshooting

This guide helps you solve common issues with Skill Engine.

## Installation Issues

### "Command not found: skill"

**Cause:** Skill is not in your PATH.

**Solution:**
```bash
# Add to PATH (add to your shell profile)
export PATH="$HOME/.skill-engine/bin:$PATH"

# Or reinstall
curl -fsSL https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/install.sh | sh
```

### Installation fails on Apple Silicon (M1/M2)

**Cause:** Architecture mismatch.

**Solution:**
```bash
# Verify architecture
uname -m  # Should show arm64

# Reinstall with correct binary
SKILL_VERSION=latest curl -fsSL https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/install.sh | sh
```

### Permission denied during installation

**Cause:** Cannot write to install directory.

**Solution:**
```bash
# Option 1: Use custom directory
SKILL_INSTALL_DIR="$HOME/.local/bin" curl -fsSL https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/install.sh | sh

# Option 2: Fix permissions
chmod +x ~/.skill-engine/bin/skill
```

## Skill Not Found

### "Skill 'X' not found"

**Cause:** Skill not installed or name mismatch.

**Solution:**
```bash
# List installed skills
skill list

# Check skill name in SKILL.md
cat ./my-skill/SKILL.md | head -5

# Reinstall
skill install ./my-skill
```

### Skill installed but tools not visible

**Cause:** SKILL.md parsing issue.

**Solution:**
```bash
# Check skill info
skill info my-skill

# Verify SKILL.md format
# Tools must start with "### tool-name"
```

## Execution Errors

### "Permission denied" when running tools

**Cause:** Command not in allowed-tools list.

**Solution:**
Edit your SKILL.md frontmatter:
```yaml
---
name: my-skill
allowed-tools: Bash, kubectl, git
---
```

### "Command failed with exit code X"

**Cause:** Underlying command error.

**Solution:**
```bash
# Run the command directly to see full error
kubectl get pods

# Check command is installed
which kubectl
```

### Tool parameters not working

**Cause:** Parameter format issue.

**Solution:**
```bash
# Correct format
skill run my-skill:tool param="value"

# With spaces - use quotes
skill run my-skill:tool message="hello world"

# Boolean values
skill run my-skill:tool verbose=true

# No spaces around =
skill run my-skill:tool count=5  # Correct
skill run my-skill:tool count = 5  # Wrong
```

## WASM Skill Issues

### "Failed to compile WASM"

**Cause:** Invalid JavaScript or missing exports.

**Solution:**
Check your skill.js exports:
```javascript
// All these exports are required
export function getMetadata() { ... }
export function getTools() { ... }
export function executeTool(name, args) { ... }
export function validateConfig() { ... }
```

### "Module not found" in WASM skill

**Cause:** Unsupported import.

**Solution:**
WASM skills can only use:
- Built-in JavaScript (fetch, JSON, etc.)
- Exported functions

Cannot use:
- Node.js modules (fs, path)
- npm packages (unless bundled)

### Async operations fail silently

**Cause:** Not using async/await properly.

**Solution:**
```javascript
// Mark function as async
export async function executeTool(name, args) {
  const response = await fetch(url);
  const data = await response.json();
  return { success: true, output: JSON.stringify(data) };
}
```

## MCP Server Issues

### "Connection refused" to MCP server

**Cause:** Server not running or wrong port.

**Solution:**
```bash
# Start the server
skill serve

# Or with HTTP mode
skill serve --http --port 3000

# Check if running
curl http://localhost:3000/health
```

### MCP tools not appearing in Claude

**Cause:** Configuration issue.

**Solution:**
```bash
# Check Claude Code setup
skill claude status

# Reconfigure
skill claude remove
skill claude setup

# Verify .mcp.json
cat .mcp.json
```

### "Invalid JSON-RPC" errors

**Cause:** Output going to stdout instead of stderr.

**Solution:**
Ensure all logging uses stderr:
```javascript
// Wrong - goes to stdout
console.log("debug info");

// Right - goes to stderr
console.error("debug info");
```

## Search Issues

### Search returns no results

**Cause:** Index not built or empty.

**Solution:**
```bash
# Reinstall skills to rebuild index
skill install ./my-skill

# Check installed skills
skill list

# Try broader search
skill find "kubernetes"
```

### Search results not relevant

**Cause:** Poor tool descriptions.

**Solution:**
Improve your SKILL.md descriptions:
```markdown
### deploy
Deploy applications to Kubernetes cluster. Use when you need to
create new deployments, update existing ones, or roll out changes.
Supports canary and blue-green deployment strategies.
```

## Performance Issues

### Slow skill startup

**Cause:** Large skill or cold start.

**Solution:**
```bash
# Check skill size
du -sh ~/.skill-engine/skills/my-skill

# Pre-warm with a simple call
skill run my-skill:health
```

### High memory usage

**Cause:** Large data processing or memory leak.

**Solution:**
- Process data in chunks
- Clear large variables after use
- Check for infinite loops

### Search is slow

**Cause:** Large index or first query.

**Solution:**
```bash
# First query builds cache - subsequent queries are faster
skill find "test"  # Slow
skill find "test"  # Fast
```

## Configuration Issues

### "Configuration not found"

**Cause:** Skill not configured.

**Solution:**
```bash
# Configure the skill
skill config my-skill

# Follow prompts to enter values
```

### Credentials not persisting

**Cause:** Keyring access issue.

**Solution:**
```bash
# Check keyring service
# On Linux, install secret-service
sudo apt install gnome-keyring

# On macOS, unlock keychain
security unlock-keychain

# Or use environment variables
export MY_SKILL_API_KEY="your-key"
```

## Docker Runtime Issues

### "Docker daemon not running"

**Cause:** Docker not started.

**Solution:**
```bash
# Start Docker
# macOS/Windows: Start Docker Desktop
# Linux:
sudo systemctl start docker
```

### Container fails to start

**Cause:** Image not found or resource limits.

**Solution:**
```bash
# Pull the image manually
docker pull node:20

# Check available resources
docker system info
```

## Getting More Help

If you're still stuck:

1. **Search existing issues**: https://github.com/kubiyabot/skill/issues
2. **Ask in discussions**: https://github.com/kubiyabot/skill/discussions
3. **Include debug info**:
   ```bash
   skill --version
   uname -a
   skill list
   # And the full error message
   ```

## Debug Mode

Enable verbose output for debugging:

```bash
# Set log level
RUST_LOG=debug skill run my-skill:tool

# Or for specific components
RUST_LOG=skill_runtime=debug skill run my-skill:tool
```
