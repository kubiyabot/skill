# Skill Engine Examples

This directory contains example skills demonstrating the three runtime types supported by Skill Engine.

## Skill Types Overview

| Type | Description | Network | Security | Use Case |
|------|-------------|---------|----------|----------|
| **WASM Skills** | WebAssembly components compiled from JS/TS/Python | Configurable | Sandboxed by WASM runtime | Custom logic, API integrations |
| **Native Skills** | CLI wrappers that execute host commands | Full access | Command allowlist | DevOps tools (kubectl, docker) |
| **Docker Runtime Skills** | Run tools inside Docker containers | Isolated (none by default) | Container sandbox + limits | Heavy processing (ffmpeg, imagemagick) |

## Directory Structure

```
examples/
├── wasm-skills/              # WASM-based skills (6 examples)
│   ├── simple-skill/         # Beginner - Hello World
│   ├── aws-skill/            # Cloud - AWS S3/EC2/Lambda
│   ├── github-skill/         # API - GitHub repos/issues/PRs
│   ├── github-oauth-skill/   # OAuth2 - GitHub with device flow
│   ├── slack-skill/          # Messaging - Slack channels/messages
│   └── python-skill/         # Python SDK example
│
├── native-skills/            # Native command wrapper skills (2 examples)
│   ├── docker-skill/         # Docker CLI (30+ tools)
│   └── kubernetes-skill/     # Kubectl CLI (18+ tools)
│
└── docker-runtime-skills/    # Docker container-based skills (6 examples)
    ├── ffmpeg-skill/         # Video/audio processing
    ├── python-runner/        # Python script execution
    ├── node-runner/          # Node.js script execution
    ├── imagemagick-skill/    # Image manipulation
    ├── postgres-skill/       # PostgreSQL CLI client
    └── redis-skill/          # Redis CLI client
```

## Quick Start

### Running Examples

```bash
# WASM skill
skill run simple:hello name=World

# Native skill (Docker CLI)
skill run docker ps

# Docker runtime skill
skill run python-runner -- --version
```

### Adding to Your Project

Copy the relevant manifest configuration from each example's `manifest.toml` into your `.skill-engine.toml`:

```toml
# WASM skill from local path
[skills.simple]
source = "./examples/wasm-skills/simple-skill"

# Native skill
[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"

# Docker runtime skill
[skills.ffmpeg]
source = "docker:linuxserver/ffmpeg:latest"
runtime = "docker"
[skills.ffmpeg.docker]
image = "linuxserver/ffmpeg:latest"
entrypoint = "ffmpeg"
```

## Choosing the Right Skill Type

### Use WASM Skills when:
- You need custom business logic
- Integrating with APIs (REST, GraphQL)
- Processing data transformations
- Building reusable tool libraries

### Use Native Skills when:
- Wrapping existing CLI tools
- Tools need full system access (kubectl, docker, terraform)
- Performance is critical (no container overhead)

### Use Docker Runtime Skills when:
- Running heavy processing (video, image, ML)
- Tool isn't available as WASM
- You need strict isolation
- Reproducible environments are important

## Security Comparison

| Aspect | WASM | Native | Docker Runtime |
|--------|------|--------|----------------|
| Network | Configurable | Full | None by default |
| Filesystem | Sandboxed | Full | Volume mounts only |
| System calls | Limited | Full | Container limits |
| Resource limits | WASM limits | None | Memory/CPU configurable |
| Escape risk | Very low | High | Low |

## More Information

- [WASM Skills Guide](./wasm-skills/README.md)
- [Native Skills Guide](./native-skills/README.md)
- [Docker Runtime Skills Guide](./docker-runtime-skills/README.md)
