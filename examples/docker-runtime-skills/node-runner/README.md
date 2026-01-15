# Node.js Runner Skill

Execute JavaScript/TypeScript code in a sandboxed Node.js Docker environment.

## When to Use

Use this skill when you need to:
- Run arbitrary JavaScript code safely
- Process JSON data with Node.js
- Execute npm scripts in isolation
- Quick scripting tasks
- File processing with JavaScript

**Choose this over WASM skills when:**
- You need access to Node.js APIs (fs, path, etc.)
- Your code requires npm packages
- You're working with files on disk
- You need network access for API calls

## Features

| Feature | Status |
|---------|--------|
| Node.js 20 LTS | ✅ |
| npm/npx included | ✅ |
| File system access | ✅ (current directory) |
| Network access | ⚙️ (configurable) |
| Memory limits | ✅ (512MB default) |
| Automatic cleanup | ✅ |

## Quick Start

```bash
# Install the skill
skill install ./examples/docker-runtime-skills/node-runner

# Run a simple script
skill run node-runner -- -e "console.log('Hello from Node!')"
```

## Usage

### Basic Commands

```bash
# Check Node version
skill run node-runner -- --version

# Run a script file
skill run node-runner -- script.js

# Evaluate expression
skill run node-runner -- -e "console.log(JSON.stringify({hello: 'world'}))"

# Run with arguments
skill run node-runner -- app.js --config config.json

# Interactive REPL (with TTY)
skill run node-runner
```

### Working with Files

```bash
# Process a JSON file
skill run node-runner -- -e "
const data = require('./data.json');
console.log(data.items.length);
"

# Transform data
cat input.json | skill run node-runner -- -e "
let data = '';
process.stdin.on('data', d => data += d);
process.stdin.on('end', () => {
  const obj = JSON.parse(data);
  console.log(JSON.stringify(obj, null, 2));
});
"
```

### Using npm Packages

For npm packages, use the network-enabled variant:

```bash
# Run with network access for npm install
skill run node-runner-net -- -e "
const fetch = (...args) => import('node-fetch').then(({default: fetch}) => fetch(...args));
fetch('https://api.github.com').then(r => r.json()).then(console.log);
"
```

## Configuration

### Basic (No Network)

The default configuration for sandboxed execution:

```toml
[skills.node-runner]
source = "docker:node:20-alpine"
runtime = "docker"
description = "Node.js script execution (sandboxed)"

[skills.node-runner.docker]
image = "node:20-alpine"
entrypoint = "node"
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
memory = "512m"
network = "none"
rm = true
```

### With Network (for npm/APIs)

Enable network access when needed:

```toml
[skills.node-runner-net]
source = "docker:node:20-alpine"
runtime = "docker"
description = "Node.js with network access"

[skills.node-runner-net.docker]
image = "node:20-alpine"
entrypoint = "node"
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
memory = "512m"
network = "bridge"
rm = true
```

### Custom npm Runner

For npm commands specifically:

```toml
[skills.npm-runner]
source = "docker:node:20-alpine"
runtime = "docker"
description = "npm package manager"

[skills.npm-runner.docker]
image = "node:20-alpine"
entrypoint = "npm"
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
memory = "1g"
network = "bridge"
rm = true
```

## Security

| Control | Setting | Description |
|---------|---------|-------------|
| Network | `none` by default | No internet access unless configured |
| Memory | 512MB limit | Prevents memory exhaustion |
| Volumes | Current directory only | No access to host system |
| Image | Alpine-based | Minimal attack surface (~50MB) |
| Cleanup | Auto-remove | Container deleted after execution |

### Security Best Practices

1. **Use `network = "none"` for untrusted code**
2. **Review scripts before execution**
3. **Limit volume mounts** to only what's needed
4. **Set appropriate memory limits** for your use case

## Use Cases

| Scenario | Network | Command |
|----------|---------|---------|
| JSON processing | `none` | `skill run node-runner -- transform.js data.json` |
| File operations | `none` | `skill run node-runner -- process.js` |
| API calls | `bridge` | `skill run node-runner-net -- fetch.js` |
| npm install | `bridge` | `skill run npm-runner -- install` |
| Build scripts | `bridge` | `skill run npm-runner -- run build` |

## Examples

### JSON Processing

```javascript
// transform.js
const fs = require('fs');
const data = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));

const transformed = data.map(item => ({
  ...item,
  processed: true,
  timestamp: new Date().toISOString()
}));

console.log(JSON.stringify(transformed, null, 2));
```

```bash
skill run node-runner -- transform.js input.json > output.json
```

### File Processing

```javascript
// count-lines.js
const fs = require('fs');
const files = fs.readdirSync('.').filter(f => f.endsWith('.js'));
files.forEach(f => {
  const lines = fs.readFileSync(f, 'utf8').split('\n').length;
  console.log(`${f}: ${lines} lines`);
});
```

```bash
skill run node-runner -- count-lines.js
```

## Docker Image

| Property | Value |
|----------|-------|
| Image | `node:20-alpine` |
| Size | ~50MB |
| Node.js | 20.x LTS |
| npm | Included |
| npx | Included |
| OS | Alpine Linux |

## Troubleshooting

### "Cannot find module" Error

Ensure the file exists in the current directory:
```bash
ls -la script.js
skill run node-runner -- script.js
```

### Network Requests Fail

Use the network-enabled variant:
```bash
skill run node-runner-net -- fetch-script.js
```

### Out of Memory

Increase the memory limit:
```toml
[skills.node-runner.docker]
memory = "1g"  # Increase to 1GB
```

### Permission Denied

Ensure the script is readable:
```bash
chmod +r script.js
```

## Related Skills

- [Python Runner](../python-runner) - Python script execution
- [WASM Skills](/guides/developing-skills) - For portable, sandboxed code
