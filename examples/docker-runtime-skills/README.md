# Docker Runtime Skills

Docker runtime skills execute tools inside isolated Docker containers. They provide strong security guarantees while leveraging the entire Docker ecosystem.

## Examples

| Skill | Docker Image | Purpose | Network |
|-------|-------------|---------|---------|
| [ffmpeg-skill](./ffmpeg-skill/) | `linuxserver/ffmpeg:latest` | Video/audio processing | none |
| [python-runner](./python-runner/) | `python:3.12-slim` | Python script execution | none |
| [node-runner](./node-runner/) | `node:20-alpine` | Node.js script execution | none |
| [imagemagick-skill](./imagemagick-skill/) | `dpokidov/imagemagick:latest` | Image manipulation | none |
| [postgres-skill](./postgres-skill/) | `postgres:16-alpine` | PostgreSQL CLI client | bridge |
| [redis-skill](./redis-skill/) | `redis:7-alpine` | Redis CLI client | bridge |

## How Docker Runtime Works

```
skill run ffmpeg -- -i input.mp4 output.webm
        ↓
    Manifest: runtime = "docker"
        ↓
    Docker Runtime Engine
        ↓
    docker run --rm --network none \
      -v ${PWD}:/workdir \
      --memory 2g \
      linuxserver/ffmpeg:latest \
      -i input.mp4 output.webm
        ↓
    Container Output Captured
```

## Security Features

### Network Isolation (Default: none)
```toml
[skills.ffmpeg.docker]
network = "none"  # No network access - most secure
```

### Resource Limits
```toml
[skills.ffmpeg.docker]
memory = "2g"     # Max 2GB RAM
cpus = "4"        # Max 4 CPU cores
```

### Blocked Operations
The runtime automatically blocks:
- `--privileged` flag
- `/var/run/docker.sock` mounts
- Host network mode
- Mounting sensitive paths (`/etc/passwd`, `/root`, etc.)

### Auto-Cleanup
```toml
[skills.ffmpeg.docker]
rm = true  # Container removed after execution
```

## Creating a Docker Runtime Skill

### 1. Choose a Docker Image

Pick an image with the tool you need:
```bash
docker search ffmpeg
docker pull linuxserver/ffmpeg:latest
```

### 2. Create Manifest Configuration

```toml
[skills.my-tool]
source = "docker:my-image:tag"
runtime = "docker"
description = "My containerized tool"

[skills.my-tool.docker]
image = "my-image:tag"
entrypoint = "my-command"
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
memory = "512m"
network = "none"
rm = true
```

### 3. Test It

```bash
skill run my-tool -- --version
skill run my-tool -- --help
```

## Configuration Reference

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `image` | string | required | Docker image (e.g., `python:3.12-slim`) |
| `entrypoint` | string | image default | Override container entrypoint |
| `command` | string[] | none | Default command arguments |
| `volumes` | string[] | `[]` | Volume mounts (`host:container`) |
| `working_dir` | string | none | Working directory inside container |
| `environment` | string[] | `[]` | Environment variables (`KEY=value`) |
| `memory` | string | none | Memory limit (e.g., `512m`, `2g`) |
| `cpus` | string | none | CPU limit (e.g., `0.5`, `2`) |
| `network` | string | `"none"` | Network mode (`none`, `bridge`, `host`) |
| `rm` | bool | `true` | Remove container after execution |
| `user` | string | none | Run as user (`uid:gid` or username) |
| `gpus` | string | none | GPU access (`all` or device IDs) |
| `read_only` | bool | `false` | Read-only root filesystem |
| `platform` | string | none | Target platform (`linux/amd64`, `linux/arm64`) |
| `extra_args` | string[] | `[]` | Additional docker run arguments |

## Environment Variable Expansion

Use `${VAR}` syntax for dynamic configuration:

```toml
[skills.my-tool.docker]
volumes = ["${PWD}:/workdir", "${HOME}/.config:/config:ro"]
environment = ["API_KEY=${MY_API_KEY}"]
```

Supported formats:
- `${VAR}` - Required, error if not set
- `${VAR:-default}` - Use default if not set
- `${VAR:?error message}` - Custom error message

## Network Modes

### `none` (Most Secure)
```toml
network = "none"
```
- No network access
- Best for processing local files
- Use for: ffmpeg, imagemagick, file processing

### `bridge` (Selective Access)
```toml
network = "bridge"
```
- Can connect to other containers
- Can access internet
- Use for: database clients, API tools

### `host` (BLOCKED by default)
```toml
network = "host"  # Will be rejected by security policy
```
- Shares host network
- Security risk - blocked by default

## GPU Support

For ML/AI workloads with NVIDIA GPUs:

```toml
[skills.ml-inference.docker]
image = "nvidia/cuda:12.0-runtime"
gpus = "all"  # Or specific: "0,1"
memory = "8g"
```

Requires nvidia-container-runtime.

## Best Practices

1. **Use `network = "none"`** for file processing tools
2. **Set memory limits** to prevent resource exhaustion
3. **Use read-only volumes** when possible (`:ro` suffix)
4. **Pin image versions** for reproducibility (`python:3.12.1` not `python:latest`)
5. **Use Alpine images** when available (smaller, faster)
6. **Clean up** with `rm = true` (default)

## Comparison with Other Runtime Types

| Feature | WASM | Native | Docker Runtime |
|---------|------|--------|----------------|
| Startup time | Fast (~100ms) | Instant | Slower (~1s) |
| Image size | Small (11MB) | N/A | Varies (50MB-4GB) |
| Ecosystem | Limited | System tools | Docker Hub |
| Isolation | WASM sandbox | None | Container |
| GPU support | No | Yes | Yes |
| Custom tools | Write code | Install on host | Any Docker image |
