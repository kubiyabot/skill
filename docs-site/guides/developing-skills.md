# Developing Skills

Learn how to create your own skills for Skill Engine using native binaries, Docker containers, or WASM components.

## Skill Types

| Type | Best For | Complexity | Performance |
|------|----------|------------|-------------|
| **Native** | CLI tools, system commands | Low | Fastest |
| **Docker** | Complex dependencies, isolation | Medium | Good |
| **WASM** | Portable, sandboxed code | High | Very Fast |

## Skill Structure

Every skill needs:
1. **Manifest** (`SKILL.md`) - Describes tools and parameters
2. **Executables** - Native binaries, Docker images, or WASM modules
3. **Configuration** (optional) - `.skill-engine.toml` for registration

## Native Skills

Native skills wrap existing CLI tools or scripts.

### Example: kubectl Wrapper

**Directory structure:**

```
kubernetes-skill/
├── SKILL.md
└── tools/
    ├── get.sh
    ├── logs.sh
    ├── describe.sh
    └── apply.sh
```

**SKILL.md:**

```markdown
---
name: kubernetes
version: 1.0.0
runtime: native
description: Kubernetes cluster management with kubectl
---

# Kubernetes Skill

Manage Kubernetes clusters using kubectl commands.

## Tools

### get
Get Kubernetes resources

Parameters:
- resource (required): Resource type (pods, services, deployments)
- namespace (optional): Namespace (default: default)
- output (optional): Output format (json, yaml, wide)

### logs
View pod logs

Parameters:
- pod (required): Pod name
- namespace (optional): Namespace
- tail (optional): Number of lines to show
- follow (optional): Follow log output

### describe
Describe a Kubernetes resource

Parameters:
- resource (required): Resource type
- name (required): Resource name
- namespace (optional): Namespace
```

**tools/get.sh:**

```bash
#!/bin/bash
set -euo pipefail

# Parse parameters from Skill Engine
RESOURCE="${resource}"
NAMESPACE="${namespace:-default}"
OUTPUT="${output:-wide}"

# Execute kubectl
kubectl get "$RESOURCE" \
  --namespace="$NAMESPACE" \
  --output="$OUTPUT"
```

**tools/logs.sh:**

```bash
#!/bin/bash
set -euo pipefail

POD="${pod}"
NAMESPACE="${namespace:-default}"
TAIL="${tail:-100}"
FOLLOW="${follow:-false}"

CMD="kubectl logs $POD --namespace=$NAMESPACE --tail=$TAIL"

if [ "$FOLLOW" = "true" ]; then
  CMD="$CMD --follow"
fi

eval "$CMD"
```

### Register Native Skill

`.skill-engine.toml`:

```toml
[skills.kubernetes]
source = "./kubernetes-skill"
runtime = "native"
description = "Kubernetes cluster management"

[skills.kubernetes.instances.default]
capabilities.allowed_commands = ["kubectl", "helm"]
capabilities.timeout_ms = 30000
```

### Test Native Skill

```bash
# Run the skill
skill run kubernetes get --resource pods --namespace default

# Check logs
skill run kubernetes logs --pod nginx-xxx --tail 50
```

## Docker Skills

Docker skills run in isolated containers with all dependencies included.

### Example: FFmpeg Video Processing

**Directory structure:**

```
ffmpeg-skill/
├── SKILL.md
└── Dockerfile (optional)
```

**SKILL.md:**

```markdown
---
name: ffmpeg
version: 1.0.0
runtime: docker
docker_image: linuxserver/ffmpeg:latest
docker_entrypoint: ffmpeg
description: Video and audio processing with FFmpeg
---

# FFmpeg Skill

Process video and audio files using FFmpeg in an isolated container.

## Tools

### convert
Convert video format

Parameters:
- input (required): Input file path
- output (required): Output file path
- format (optional): Output format (mp4, webm, avi)
- quality (optional): Quality preset (high, medium, low)

### extract-audio
Extract audio from video

Parameters:
- input (required): Input video file
- output (required): Output audio file
- codec (optional): Audio codec (mp3, aac, flac)

### create-thumbnail
Create video thumbnail

Parameters:
- input (required): Input video file
- output (required): Output image file
- timestamp (optional): Timestamp (default: 00:00:05)
```

**Register Docker Skill:**

`.skill-engine.toml`:

```toml
[skills.ffmpeg]
source = "docker:linuxserver/ffmpeg:latest"
runtime = "docker"
description = "Video processing with FFmpeg"

[skills.ffmpeg.docker]
image = "linuxserver/ffmpeg:latest"
entrypoint = "ffmpeg"
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"

[skills.ffmpeg.instances.default]
capabilities.filesystem_access = true
capabilities.allowed_paths = ["${PWD}"]
capabilities.timeout_ms = 300000  # 5 minutes for video processing
```

### Use Docker Skill

```bash
# Convert video
skill run ffmpeg convert --input video.mp4 --output video.webm

# Extract audio
skill run ffmpeg extract-audio --input video.mp4 --output audio.mp3

# Create thumbnail
skill run ffmpeg create-thumbnail --input video.mp4 --output thumb.jpg --timestamp 00:00:10
```

### Custom Docker Image

**Dockerfile:**

```dockerfile
FROM python:3.11-slim

# Install dependencies
RUN pip install requests pandas numpy

# Copy skill code
COPY tools/ /app/tools/
WORKDIR /app

# Entrypoint
ENTRYPOINT ["python"]
```

**Build and register:**

```bash
# Build image
docker build -t my-python-skill:latest .

# Register in manifest
cat >> .skill-engine.toml <<EOF
[skills.my-python-skill]
source = "docker:my-python-skill:latest"
runtime = "docker"

[skills.my-python-skill.docker]
image = "my-python-skill:latest"
entrypoint = "python"
volumes = ["\${PWD}:/data"]
working_dir = "/data"
EOF
```

## WASM Skills

WASM skills are portable, sandboxed, and very fast.

::: warning Experimental
The WASM Skill SDK is currently **experimental** and the API may change in future releases. Use in production at your own risk.
:::

### Prerequisites

- Rust 1.75+ ([rustup.rs](https://rustup.rs))
- `wasm32-wasi` target: `rustup target add wasm32-wasi`
- `wasm-pack`: `cargo install wasm-pack`

### Example: GitHub API Skill

**Directory structure:**

```
github-wasm-skill/
├── Cargo.toml
├── SKILL.md
└── src/
    └── lib.rs
```

**Cargo.toml:**

```toml
[package]
name = "github-skill"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
# EXPERIMENTAL: API may change
skill-wasm-sdk = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["blocking", "json"] }

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization
```

**src/lib.rs:**

```rust
use skill_wasm_sdk::{skill_tool, ToolResult};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ListReposParams {
    owner: String,
    #[serde(default)]
    per_page: u32,
}

#[derive(Serialize)]
struct Repository {
    name: String,
    full_name: String,
    description: Option<String>,
    html_url: String,
    stargazers_count: u32,
}

/// List GitHub repositories for a user
#[skill_tool]
fn list_repos(params: ListReposParams) -> ToolResult {
    let url = format!(
        "https://api.github.com/users/{}/repos?per_page={}",
        params.owner,
        if params.per_page == 0 { 30 } else { params.per_page }
    );

    let response = reqwest::blocking::get(&url)?;
    let repos: Vec<Repository> = response.json()?;

    ToolResult::success(serde_json::to_value(repos)?)
}

#[derive(Deserialize)]
struct CreateIssueParams {
    owner: String,
    repo: String,
    title: String,
    body: Option<String>,
}

/// Create a GitHub issue
#[skill_tool]
fn create_issue(params: CreateIssueParams) -> ToolResult {
    let url = format!(
        "https://api.github.com/repos/{}/{}/issues",
        params.owner, params.repo
    );

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "title": params.title,
            "body": params.body.unwrap_or_default()
        }))
        .send()?;

    ToolResult::success(response.json()?)
}
```

**SKILL.md:**

```markdown
---
name: github
version: 1.0.0
runtime: wasm
description: GitHub API integration
---

# GitHub Skill

Interact with GitHub repositories and issues via the GitHub API.

## Tools

### list_repos
List repositories for a user

Parameters:
- owner (required): GitHub username
- per_page (optional): Number of repos per page (default: 30)

### create_issue
Create a new issue

Parameters:
- owner (required): Repository owner
- repo (required): Repository name
- title (required): Issue title
- body (optional): Issue description
```

### Build WASM Skill

```bash
# Build for wasm32-wasi target
cargo build --target wasm32-wasi --release

# WASM module is at:
# target/wasm32-wasi/release/github_skill.wasm
```

### Register WASM Skill

`.skill-engine.toml`:

```toml
[skills.github]
source = "./github-wasm-skill/target/wasm32-wasi/release/github_skill.wasm"
runtime = "wasm"
description = "GitHub API integration"

[skills.github.instances.default]
env.GITHUB_TOKEN = "${GITHUB_TOKEN}"
capabilities.network_access = true
capabilities.allowed_domains = ["api.github.com", "github.com"]
capabilities.max_memory_mb = 128
capabilities.timeout_ms = 30000
```

### Use WASM Skill

```bash
# List repositories
skill run github list_repos --owner torvalds

# Create issue
skill run github create_issue \
  --owner myuser \
  --repo myrepo \
  --title "Bug report" \
  --body "Description of the bug"
```

## Skill Development Best Practices

### 1. Clear Tool Descriptions

```markdown
### tool_name
Short one-line description of what this tool does

Parameters:
- param1 (required): Clear description with examples
- param2 (optional): Description with default value (default: value)
- param3 (optional): Description with allowed values (options: a, b, c)

Example:
```bash
skill run myskill tool_name --param1 value --param2 value
```
```

### 2. Parameter Validation

**Native (bash):**

```bash
#!/bin/bash
set -euo pipefail

# Validate required parameters
if [ -z "${input:-}" ]; then
  echo "Error: 'input' parameter is required" >&2
  exit 1
fi

# Validate file exists
if [ ! -f "$input" ]; then
  echo "Error: File '$input' not found" >&2
  exit 1
fi
```

**WASM (Rust):**

```rust
#[skill_tool]
fn process_file(params: ProcessParams) -> ToolResult {
    if params.input.is_empty() {
        return ToolResult::error("'input' parameter is required");
    }

    if params.count < 1 || params.count > 1000 {
        return ToolResult::error("'count' must be between 1 and 1000");
    }

    // Process...
}
```

### 3. Structured Output

Always return structured JSON:

```bash
# Good - JSON output
echo '{"status": "success", "files_processed": 42}'

# Bad - Plain text
echo "Processed 42 files"
```

```rust
// Good - Structured result
ToolResult::success(serde_json::json!({
    "status": "success",
    "files_processed": 42,
    "duration_ms": 1234
}))
```

### 4. Error Handling

**Native:**

```bash
#!/bin/bash
set -euo pipefail

# Trap errors
trap 'echo "{\"error\": \"Script failed at line $LINENO\"}" >&2; exit 1' ERR

# Your code here
```

**WASM:**

```rust
#[skill_tool]
fn risky_operation(params: Params) -> ToolResult {
    match perform_operation() {
        Ok(result) => ToolResult::success(result),
        Err(e) => ToolResult::error(&format!("Operation failed: {}", e))
    }
}
```

### 5. Logging and Debugging

**Native:**

```bash
# Log to stderr (doesn't pollute JSON output)
echo "Processing file: $input" >&2

# Output JSON to stdout
echo '{"result": "success"}'
```

**WASM:**

```rust
use skill_wasm_sdk::log;

#[skill_tool]
fn my_tool(params: Params) -> ToolResult {
    log::debug(&format!("Processing: {}", params.input));

    // Tool logic...

    log::info("Processing complete");
    ToolResult::success(result)
}
```

## Testing Your Skills

### Manual Testing

```bash
# Test native skill
./tools/get.sh <<EOF
export resource="pods"
export namespace="default"
EOF

# Test with skill run
skill run myskill mytool --param value

# Test with debug logging
LOG_LEVEL=debug skill run myskill mytool
```

### Integration Testing

See [Testing Guide](./testing.md) for comprehensive testing strategies.

## Publishing Skills

### 1. Create GitHub Repository

```bash
git init
git add .
git commit -m "Initial skill implementation"
git remote add origin https://github.com/user/myskill
git push -u origin main
```

### 2. Tag Release

```bash
git tag -a v1.0.0 -m "First release"
git push origin v1.0.0
```

### 3. Install from Git

Others can now install:

```bash
skill install https://github.com/user/myskill
```

### 4. Submit to Catalog

Open a PR to the [Skill Catalog](https://github.com/kubiyabot/skill) to list your skill.

## Next Steps

- **[Manifest Reference](./manifest.md)** - Complete SKILL.md format
- **[Testing](./testing.md)** - Test your skills
- **[Environment Variables](./environment.md)** - Managing configuration
- **[Security Model](./advanced/security.md)** - Capability system
- **[Examples](/examples/)** - Real-world skill examples

## Resources

- **WASM SDK Documentation**: [github.com/kubiyabot/skill-wasm-sdk](https://github.com/kubiyabot/skill-wasm-sdk) (Experimental)
- **Example Skills**: `/examples/native-skills/`, `/examples/docker-skills/`, `/examples/wasm-skills/`
- **Community**: [GitHub Discussions](https://github.com/kubiyabot/skill/discussions)
