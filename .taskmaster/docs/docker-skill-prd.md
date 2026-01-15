# Docker Skill PRD for Skill-Engine

## Executive Summary

Create a comprehensive Docker integration for skill-engine with three major components:
1. **Docker Skill (Container Management)** - Full Docker CLI wrapper skill following kubernetes-skill pattern
2. **Docker as Execution Runtime** - Use Docker containers as skill execution backends
3. **container2wasm Integration** - Experimental container-to-WASM conversion support

This enables AI assistants to manage Docker containers, use existing container images as skill backends, and optionally convert containers to portable WASM modules.

## Background

### Problem Statement
Currently, skill-engine only supports WASM-based skills. While WASM provides excellent sandboxing and portability, it has limitations:
- Not all tools can be compiled to WASM
- Some workloads require specific system binaries or GPU access
- The container ecosystem has millions of pre-built images

### Solution Overview
1. A Docker skill for container management (similar to kubernetes-skill)
2. Docker as an alternative execution runtime for skills
3. Optional container-to-WASM conversion via container2wasm

### Research Findings

**container2wasm** (https://github.com/container2wasm/container2wasm):
- Converts container images to WASM via CPU emulation (Bochs for x86_64, TinyEMU for RISC-V)
- Experimental, not production-ready
- 10-100x performance overhead
- Output files typically 100MB-1GB
- Use cases: legacy tool preservation, browser execution, sandboxed prototyping

**Docker Desktop WASM Support** (via runwasi):
- Beta feature, being deprecated
- Uses containerd shims for WASM runtimes
- Not recommended as dependency

**Docker SDK/API**:
- Full programmatic control: containers, images, networks, volumes
- Container lifecycle: create, start, stop, remove, restart
- Execution: exec, attach, logs
- Images: pull, push, build, tag

---

## Part 1: Docker Skill (Container Management)

### Overview
A SKILL.md-based skill that wraps Docker CLI commands, enabling AI assistants to manage containers through a structured interface. Follows the established kubernetes-skill pattern.

### SKILL.md Format
```yaml
---
name: docker
description: Docker container and image management with native docker CLI integration. Use for managing containers, images, networks, and volumes.
allowed-tools: Bash, skill-run
---
```

### Tools Specification (30 tools)

#### Container Lifecycle Tools (Priority: HIGH)

1. **run** - Create and start a container
   - Parameters:
     - `image` (required, string): Container image name[:tag]
     - `name` (optional, string): Container name
     - `detach` (optional, boolean): Run in background
     - `ports` (optional, string): Port mappings (e.g., "8080:80,443:443")
     - `volumes` (optional, string): Volume mounts (e.g., "/host:/container")
     - `env` (optional, string): Environment variables (e.g., "KEY=value,FOO=bar")
     - `network` (optional, string): Network to connect to
     - `rm` (optional, boolean): Remove container when it exits
     - `command` (optional, string): Command to run in container

2. **exec** - Execute command in running container
   - Parameters:
     - `container` (required, string): Container name or ID
     - `command` (required, string): Command to execute
     - `interactive` (optional, boolean): Keep STDIN open
     - `tty` (optional, boolean): Allocate a pseudo-TTY
     - `user` (optional, string): Username or UID
     - `workdir` (optional, string): Working directory inside container

3. **logs** - Fetch container logs
   - Parameters:
     - `container` (required, string): Container name or ID
     - `tail` (optional, number): Number of lines to show
     - `follow` (optional, boolean): Follow log output
     - `since` (optional, string): Show logs since timestamp
     - `timestamps` (optional, boolean): Show timestamps

4. **ps** - List containers
   - Parameters:
     - `all` (optional, boolean): Show all containers
     - `filter` (optional, string): Filter output
     - `format` (optional, string): Go template format
     - `quiet` (optional, boolean): Only display container IDs

5. **start** - Start stopped containers
   - Parameters:
     - `container` (required, string): Container name(s) or ID(s)
     - `attach` (optional, boolean): Attach STDOUT/STDERR

6. **stop** - Stop running containers
   - Parameters:
     - `container` (required, string): Container name(s) or ID(s)
     - `time` (optional, number): Seconds to wait before killing

7. **rm** - Remove containers
   - Parameters:
     - `container` (required, string): Container name(s) or ID(s)
     - `force` (optional, boolean): Force removal
     - `volumes` (optional, boolean): Remove associated volumes

8. **restart** - Restart containers
   - Parameters:
     - `container` (required, string): Container name(s) or ID(s)
     - `time` (optional, number): Seconds to wait before killing

9. **inspect** - Display detailed information
   - Parameters:
     - `target` (required, string): Container or image name/ID
     - `format` (optional, string): Go template format
     - `type` (optional, string): container or image

#### Image Management Tools (Priority: MEDIUM)

10. **images** - List images
    - Parameters:
      - `all` (optional, boolean): Show all images
      - `filter` (optional, string): Filter output
      - `format` (optional, string): Go template format
      - `quiet` (optional, boolean): Only show image IDs

11. **pull** - Pull image from registry
    - Parameters:
      - `image` (required, string): Image name[:tag]
      - `platform` (optional, string): Target platform

12. **push** - Push image to registry
    - Parameters:
      - `image` (required, string): Image name[:tag]

13. **build** - Build image from Dockerfile
    - Parameters:
      - `context` (required, string): Build context path
      - `file` (optional, string): Dockerfile path
      - `tag` (optional, string): Image tag
      - `build_arg` (optional, string): Build-time variables
      - `no_cache` (optional, boolean): Do not use cache
      - `platform` (optional, string): Target platform

14. **tag** - Create tag for image
    - Parameters:
      - `source` (required, string): Source image
      - `target` (required, string): Target tag

15. **rmi** - Remove images
    - Parameters:
      - `image` (required, string): Image name(s) or ID(s)
      - `force` (optional, boolean): Force removal

#### Network Tools (Priority: MEDIUM)

16. **network-ls** - List networks
    - Parameters:
      - `filter` (optional, string): Filter output
      - `format` (optional, string): Go template format

17. **network-create** - Create network
    - Parameters:
      - `name` (required, string): Network name
      - `driver` (optional, string): Network driver
      - `subnet` (optional, string): Subnet in CIDR format

18. **network-connect** - Connect container to network
    - Parameters:
      - `network` (required, string): Network name
      - `container` (required, string): Container name or ID
      - `ip` (optional, string): IPv4 address

19. **network-disconnect** - Disconnect from network
    - Parameters:
      - `network` (required, string): Network name
      - `container` (required, string): Container name or ID
      - `force` (optional, boolean): Force disconnection

#### Volume Tools (Priority: MEDIUM)

20. **volume-ls** - List volumes
    - Parameters:
      - `filter` (optional, string): Filter output
      - `format` (optional, string): Go template format

21. **volume-create** - Create volume
    - Parameters:
      - `name` (required, string): Volume name
      - `driver` (optional, string): Volume driver

22. **volume-rm** - Remove volumes
    - Parameters:
      - `name` (required, string): Volume name(s)
      - `force` (optional, boolean): Force removal

23. **volume-inspect** - Display volume information
    - Parameters:
      - `name` (required, string): Volume name
      - `format` (optional, string): Go template format

#### Docker Compose Tools (Priority: MEDIUM)

24. **compose-up** - Create and start containers
    - Parameters:
      - `file` (optional, string): Compose file path
      - `detach` (optional, boolean): Run in background
      - `build` (optional, boolean): Build images
      - `services` (optional, string): Specific services

25. **compose-down** - Stop and remove containers
    - Parameters:
      - `file` (optional, string): Compose file path
      - `volumes` (optional, boolean): Remove volumes
      - `remove_orphans` (optional, boolean): Remove orphans

26. **compose-ps** - List containers
    - Parameters:
      - `file` (optional, string): Compose file path
      - `all` (optional, boolean): Show all containers

27. **compose-logs** - View container output
    - Parameters:
      - `file` (optional, string): Compose file path
      - `service` (optional, string): Specific service
      - `follow` (optional, boolean): Follow output
      - `tail` (optional, number): Number of lines

#### System Tools (Priority: LOW)

28. **system-info** - Display system information
    - Parameters: none

29. **system-prune** - Remove unused data
    - Parameters:
      - `all` (optional, boolean): Remove all unused images
      - `volumes` (optional, boolean): Prune volumes
      - `force` (optional, boolean): No confirmation

30. **raw** - Execute any docker command
    - Parameters:
      - `args` (required, string): Raw docker arguments

### Security Model

1. **Command Allowlist**: Only `docker` commands allowed (already in allowlist)
2. **Blocked Flags**:
   - `--privileged` - Full host access
   - `-v /var/run/docker.sock:/var/run/docker.sock` - Docker-in-Docker escape
   - `-v /:/host` - Full host filesystem access
3. **Warning Flags** (require confirmation):
   - `--network=host` - Host network access
   - Root path mounts (/, /etc, /var)
4. **Audit Logging**: All Docker commands logged

### Implementation Pattern

Following kubernetes-skill pattern:
1. Create `examples/docker-skill/SKILL.md` with tool definitions
2. Create `examples/docker-skill/skill.js` implementing all tools
3. Each tool builds command array and returns `"Command: docker ..."`
4. CLI/MCP intercepts and executes via native command execution
5. Add tests for each tool category

---

## Part 2: Docker as Execution Runtime

### Overview
Use Docker containers as skill execution backends, providing an alternative to WASM for skills requiring specific binaries, libraries, or runtime environments.

### Use Cases
1. Skills requiring native binaries (FFmpeg, ImageMagick, ML models)
2. Skills needing specific Linux distributions or package managers
3. Legacy tools that cannot be compiled to WASM
4. Skills requiring GPU access (via nvidia-container-runtime)

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  skill run <skill>:<tool> args                                   │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
                ┌───────────────────────┐
                │  Skill Engine CLI     │
                │  Detect runtime type  │
                └───────────┬───────────┘
                            │
           ┌────────────────┼────────────────┐
           │                │                │
           ▼                ▼                ▼
    ┌──────────┐    ┌──────────────┐  ┌─────────────┐
    │   WASM   │    │    Docker    │  │  SKILL.md   │
    │ Runtime  │    │   Runtime    │  │   (Native)  │
    └──────────┘    └──────┬───────┘  └─────────────┘
                           │
                           ▼
                ┌──────────────────────┐
                │ docker run/exec      │
                │ <container>          │
                │ <entrypoint> <args>  │
                └──────────────────────┘
```

### Manifest Configuration

Extend `.skill-engine.toml` to support Docker runtime:

```toml
version = "1"

[skills.ffmpeg-skill]
source = "docker:jrottenberg/ffmpeg:5-alpine"
runtime = "docker"
description = "FFmpeg video processing skill"

[skills.ffmpeg-skill.docker]
image = "jrottenberg/ffmpeg:5-alpine"
entrypoint = "/usr/local/bin/ffmpeg"
volumes = ["${SKILL_WORKDIR}:/workdir"]
working_dir = "/workdir"
environment = ["FFMPEG_OPTS=-v warning"]
memory = "512m"
cpus = "2"
network = "none"
rm = true

[skills.ffmpeg-skill.instances.default]
config.output_format = "mp4"
```

### DockerRuntimeConfig Schema

```rust
pub struct DockerRuntimeConfig {
    /// Docker image to use
    pub image: String,

    /// Container entrypoint (overrides image default)
    pub entrypoint: Option<String>,

    /// Volume mounts (host:container format)
    pub volumes: Vec<String>,

    /// Working directory inside container
    pub working_dir: Option<String>,

    /// Environment variables
    pub environment: Vec<String>,

    /// Memory limit (e.g., "512m", "1g")
    pub memory: Option<String>,

    /// CPU limit (e.g., "0.5", "2")
    pub cpus: Option<String>,

    /// Network mode (none, bridge, host)
    pub network: String,  // default: "none"

    /// Remove container after execution
    pub rm: bool,  // default: true

    /// User to run as (uid:gid)
    pub user: Option<String>,

    /// GPU access (requires nvidia-container-runtime)
    pub gpus: Option<String>,
}
```

### Security Constraints

1. **Network Isolation**: Default to `network=none`
2. **Read-only Root**: Consider `--read-only` flag
3. **No Privileged Mode**: Never allow `--privileged`
4. **Resource Limits**: Enforce memory and CPU limits
5. **User Namespaces**: Run as non-root when possible
6. **Seccomp Profiles**: Apply restrictive profiles

### Execution Flow

1. CLI detects `runtime = "docker"` in manifest
2. Ensures Docker image is available (pull if needed)
3. Constructs `docker run` command with security constraints
4. Passes tool arguments to container entrypoint
5. Captures output and returns result

---

## Part 3: container2wasm Integration (EXPERIMENTAL)

### Overview
Support converting existing Docker containers to WASM skills using container2wasm. This is EXPERIMENTAL and has significant limitations.

### CLI Interface

```bash
# Convert container to WASM skill
skill convert <image> --output <skill-name>.wasm

# Options
skill convert <image> \
  --output <path>           # Output WASM file path
  --arch x86_64|riscv64     # Target architecture (default: riscv64)
  --entrypoint <cmd>        # Override container entrypoint
  --optimize                # Run wasm-opt for smaller output
  --bundle                  # Bundle as skill with metadata

# Example
skill convert python:3.11-slim --output python-skill.wasm --optimize
```

### Caveats and Limitations

**CRITICAL**: container2wasm is experimental with significant limitations:

1. **Performance**: 10-100x slower than native (CPU emulation)
2. **Size**: Output files typically 100MB-1GB
3. **Compatibility**: Complex syscalls may fail
4. **Memory**: High memory usage during execution
5. **Threading**: Single-threaded only
6. **Networking**: Requires special proxy configuration

**Recommended Uses**:
- Quick prototyping before WASM port
- Simple CLI tools with minimal dependencies
- Educational/demo purposes

**NOT Recommended**:
- Production workloads
- Performance-sensitive applications
- Multi-process applications

### Implementation

```bash
# Installation check
c2w --version  # container2wasm CLI

# Conversion workflow
1. Pull container image if needed
2. Run c2w to convert to WASM
3. Optionally optimize with wasm-opt
4. Generate SKILL.md metadata
5. Bundle as skill package
```

---

## Implementation Phases

### Phase 1: Docker Skill (HIGH PRIORITY)

Create the Docker container management skill.

**Tasks**:
1. Create `examples/docker-skill/` directory structure
2. Write `SKILL.md` with all 30 tool definitions and documentation
3. Implement `skill.js` with metadata, tools, and execution functions
4. Implement container lifecycle tools (run, exec, logs, ps, start, stop, rm, restart, inspect)
5. Implement image management tools (images, pull, push, build, tag, rmi)
6. Implement network tools (network-ls, network-create, network-connect, network-disconnect)
7. Implement volume tools (volume-ls, volume-create, volume-rm, volume-inspect)
8. Implement Docker Compose tools (compose-up, compose-down, compose-ps, compose-logs)
9. Implement system tools (system-info, system-prune, raw)
10. Add security validation for blocked flags
11. Add Docker skill to `.skill-engine.toml` manifest
12. Write integration tests with Docker daemon
13. Create build configuration (package.json, skill.wit)

### Phase 2: Docker Runtime (MEDIUM PRIORITY)

Enable Docker containers as skill execution backends.

**Tasks**:
1. Extend manifest.rs with DockerRuntimeConfig schema
2. Create `crates/skill-runtime/src/docker_runtime.rs`
3. Implement Docker runtime detection in CLI
4. Implement container lifecycle management (pull, run, cleanup)
5. Implement volume mounting with environment variable expansion
6. Implement resource limits (memory, CPU, network)
7. Add security constraints (no privileged, network isolation)
8. Update MCP server to support Docker runtime skills
9. Write integration tests with sample Docker-based skill
10. Document Docker runtime configuration

### Phase 3: container2wasm Integration (EXPERIMENTAL)

Add container-to-WASM conversion support.

**Tasks**:
1. Create `crates/skill-cli/src/commands/convert.rs`
2. Implement container2wasm binary detection and installation
3. Implement basic conversion workflow (image -> wasm)
4. Add wasm-opt optimization step
5. Generate SKILL.md metadata from container
6. Add experimental warning flags
7. Document caveats and limitations prominently
8. Write tests with simple container conversions

---

## Success Criteria

### Phase 1 (Docker Skill)
- [ ] Docker skill discoverable via `skill list` and MCP
- [ ] All 30 tools execute correctly
- [ ] Security restrictions block dangerous flags
- [ ] Integration tests pass with live Docker daemon
- [ ] Documentation complete in SKILL.md

### Phase 2 (Docker Runtime)
- [ ] Skills can declare `runtime = "docker"`
- [ ] Container execution respects resource limits
- [ ] Network isolation by default
- [ ] GPU access works with nvidia-runtime
- [ ] Volume mounts with env expansion work

### Phase 3 (container2wasm)
- [ ] `skill convert` produces valid WASM
- [ ] Experimental warning displayed
- [ ] Caveats documented prominently
- [ ] Basic containers convert successfully

---

## Technical Dependencies

### Critical Files to Modify

| File | Purpose |
|------|---------|
| `examples/docker-skill/SKILL.md` | Tool definitions (NEW) |
| `examples/docker-skill/skill.js` | Tool implementations (NEW) |
| `crates/skill-runtime/src/manifest.rs` | Add DockerRuntimeConfig |
| `crates/skill-runtime/src/docker_runtime.rs` | Docker runtime (NEW) |
| `crates/skill-cli/src/commands/run.rs` | Runtime detection |
| `crates/skill-cli/src/commands/convert.rs` | container2wasm (NEW) |
| `crates/skill-mcp/src/server.rs` | Docker runtime support |
| `.skill-engine.toml` | Add docker skill |

### Reference Implementations

| File | Purpose |
|------|---------|
| `examples/kubernetes-skill/skill.js` | Tool implementation pattern |
| `examples/kubernetes-skill/SKILL.md` | SKILL.md format |
| `crates/skill-cli/src/commands/run.rs:380` | Native command allowlist |

---

## Risks and Mitigations

### Technical Risks

1. **Docker Socket Access**
   - Risk: Skill may request docker.sock mount for container escape
   - Mitigation: Block docker.sock mounts, require explicit confirmation

2. **Resource Exhaustion**
   - Risk: Containers may consume excessive resources
   - Mitigation: Enforce mandatory resource limits

3. **container2wasm Instability**
   - Risk: Experimental tool may produce broken WASM
   - Mitigation: Mark as experimental, extensive testing, fallback to Docker runtime

4. **Network Security**
   - Risk: Containers may exfiltrate data
   - Mitigation: Default to network=none, require explicit permissions

### Implementation Risks

1. **Scope Creep**
   - Risk: Docker has vast API surface
   - Mitigation: Focus on essential tools, defer advanced features

2. **Platform Compatibility**
   - Risk: Docker behaves differently on Linux/Mac/Windows
   - Mitigation: Test on all platforms, document differences
