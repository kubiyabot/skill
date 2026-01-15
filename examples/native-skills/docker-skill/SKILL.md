---
name: docker
description: Docker container and image management with native docker CLI integration. Use when you need to manage containers, images, networks, volumes, or run Docker Compose.
allowed-tools: Bash, skill-run
---

# Docker Skill

Comprehensive Docker container and image management through native Docker CLI execution. This skill provides 30 tools for managing your containers, images, networks, volumes, and Docker Compose stacks.

## When to Use

- Managing Docker containers (run, exec, logs, stop, start, restart)
- Building and managing Docker images (build, pull, push, tag)
- Viewing container status and resource usage
- Managing Docker networks and volumes
- Running Docker Compose stacks
- Cleaning up unused Docker resources

## Requirements

- `docker` must be installed and in PATH
- Docker daemon must be running
- Appropriate permissions to access Docker socket
- For Compose: Docker Compose V2 (included in Docker Desktop)

## Tools Provided

### Container Lifecycle Tools

#### run
Create and start a new container.

**Parameters**:
- `image` (required): Container image name[:tag]
- `name` (optional): Container name
- `detach` (optional): Run in background (true/false)
- `ports` (optional): Port mappings (e.g., "8080:80" or "8080:80,443:443")
- `volumes` (optional): Volume mounts (e.g., "/host:/container" or "/data:/app/data,/logs:/app/logs")
- `env` (optional): Environment variables (e.g., "KEY=value" or "DB_HOST=localhost,DB_PORT=5432")
- `network` (optional): Network to connect to
- `rm` (optional): Remove container when it exits (true/false)
- `command` (optional): Command to run in container

**Example**:
```bash
skill run docker run image=nginx:latest name=web detach=true ports=8080:80
skill run docker run image=postgres:15 name=db detach=true env="POSTGRES_PASSWORD=secret" volumes=/data:/var/lib/postgresql/data
skill run docker run image=alpine command="echo hello world"
```

#### exec
Execute a command in a running container.

**Parameters**:
- `container` (required): Container name or ID
- `command` (required): Command to execute
- `interactive` (optional): Keep STDIN open (true/false)
- `tty` (optional): Allocate pseudo-TTY (true/false)
- `user` (optional): Username or UID
- `workdir` (optional): Working directory inside container

**Example**:
```bash
skill run docker exec container=web command="ls -la /var/log"
skill run docker exec container=db command="psql -U postgres -c 'SELECT 1'"
skill run docker exec container=app user=node workdir=/app command="npm run migrate"
```

#### logs
Fetch container logs.

**Parameters**:
- `container` (required): Container name or ID
- `tail` (optional): Number of lines to show from end
- `follow` (optional): Follow log output (true/false)
- `since` (optional): Show logs since timestamp (e.g., "2h", "2024-01-01")
- `timestamps` (optional): Show timestamps (true/false)

**Example**:
```bash
skill run docker logs container=web tail=100
skill run docker logs container=app since=1h timestamps=true
skill run docker logs container=db follow=true
```

#### ps
List containers.

**Parameters**:
- `all` (optional): Show all containers including stopped (true/false)
- `filter` (optional): Filter output (e.g., "status=running", "name=web")
- `format` (optional): Pretty-print using Go template
- `quiet` (optional): Only display container IDs (true/false)

**Example**:
```bash
skill run docker ps
skill run docker ps all=true
skill run docker ps filter="status=running"
skill run docker ps quiet=true
```

#### start
Start one or more stopped containers.

**Parameters**:
- `container` (required): Container name(s) or ID(s), comma-separated for multiple
- `attach` (optional): Attach STDOUT/STDERR (true/false)

**Example**:
```bash
skill run docker start container=web
skill run docker start container="web,db,cache"
```

#### stop
Stop one or more running containers.

**Parameters**:
- `container` (required): Container name(s) or ID(s), comma-separated for multiple
- `time` (optional): Seconds to wait before killing (default: 10)

**Example**:
```bash
skill run docker stop container=web
skill run docker stop container="web,db" time=30
```

#### rm
Remove one or more containers.

**Parameters**:
- `container` (required): Container name(s) or ID(s), comma-separated for multiple
- `force` (optional): Force removal of running container (true/false)
- `volumes` (optional): Remove associated anonymous volumes (true/false)

**Example**:
```bash
skill run docker rm container=web
skill run docker rm container="old-web,old-db" force=true volumes=true
```

#### restart
Restart one or more containers.

**Parameters**:
- `container` (required): Container name(s) or ID(s), comma-separated for multiple
- `time` (optional): Seconds to wait before killing (default: 10)

**Example**:
```bash
skill run docker restart container=web
skill run docker restart container="web,api" time=5
```

#### inspect
Display detailed information about a container or image.

**Parameters**:
- `target` (required): Container or image name/ID
- `format` (optional): Format output using Go template
- `type` (optional): Type of object (container, image)

**Example**:
```bash
skill run docker inspect target=web
skill run docker inspect target=nginx:latest type=image
skill run docker inspect target=web format="{{.NetworkSettings.IPAddress}}"
```

### Image Management Tools

#### images
List images.

**Parameters**:
- `all` (optional): Show all images including intermediate (true/false)
- `filter` (optional): Filter output (e.g., "dangling=true", "reference=nginx*")
- `format` (optional): Pretty-print using Go template
- `quiet` (optional): Only show image IDs (true/false)

**Example**:
```bash
skill run docker images
skill run docker images filter="dangling=true"
skill run docker images quiet=true
```

#### pull
Pull an image from a registry.

**Parameters**:
- `image` (required): Image name[:tag]
- `platform` (optional): Set platform (e.g., "linux/amd64", "linux/arm64")

**Example**:
```bash
skill run docker pull image=nginx:latest
skill run docker pull image=python:3.11-slim platform=linux/amd64
```

#### push
Push an image to a registry.

**Parameters**:
- `image` (required): Image name[:tag]

**Example**:
```bash
skill run docker push image=myregistry.com/myapp:v1.0
```

#### build
Build an image from a Dockerfile.

**Parameters**:
- `context` (required): Build context path (e.g., "." or "/path/to/project")
- `file` (optional): Path to Dockerfile (default: context/Dockerfile)
- `tag` (optional): Image tag (e.g., "myapp:v1.0")
- `build_arg` (optional): Build-time variables (e.g., "VERSION=1.0" or "VERSION=1.0,ENV=prod")
- `no_cache` (optional): Do not use cache (true/false)
- `platform` (optional): Set target platform (e.g., "linux/amd64")

**Example**:
```bash
skill run docker build context=. tag=myapp:latest
skill run docker build context=/app file=/app/Dockerfile.prod tag=myapp:prod build_arg="VERSION=1.0"
skill run docker build context=. tag=myapp:latest no_cache=true platform=linux/amd64
```

#### tag
Create a tag for an image.

**Parameters**:
- `source` (required): Source image name[:tag]
- `target` (required): Target image name[:tag]

**Example**:
```bash
skill run docker tag source=myapp:latest target=myregistry.com/myapp:v1.0
skill run docker tag source=nginx:latest target=nginx:backup
```

#### rmi
Remove one or more images.

**Parameters**:
- `image` (required): Image name(s) or ID(s), comma-separated for multiple
- `force` (optional): Force removal (true/false)

**Example**:
```bash
skill run docker rmi image=nginx:old
skill run docker rmi image="nginx:old,postgres:13" force=true
```

### Network Tools

#### network-ls
List networks.

**Parameters**:
- `filter` (optional): Filter output (e.g., "driver=bridge", "name=my-net")
- `format` (optional): Pretty-print using Go template

**Example**:
```bash
skill run docker network-ls
skill run docker network-ls filter="driver=bridge"
```

#### network-create
Create a network.

**Parameters**:
- `name` (required): Network name
- `driver` (optional): Network driver (bridge, overlay, host, none)
- `subnet` (optional): Subnet in CIDR format (e.g., "172.20.0.0/16")

**Example**:
```bash
skill run docker network-create name=my-network
skill run docker network-create name=my-network driver=bridge subnet=172.20.0.0/16
```

#### network-connect
Connect a container to a network.

**Parameters**:
- `network` (required): Network name
- `container` (required): Container name or ID
- `ip` (optional): IPv4 address

**Example**:
```bash
skill run docker network-connect network=my-network container=web
skill run docker network-connect network=my-network container=web ip=172.20.0.10
```

#### network-disconnect
Disconnect a container from a network.

**Parameters**:
- `network` (required): Network name
- `container` (required): Container name or ID
- `force` (optional): Force disconnection (true/false)

**Example**:
```bash
skill run docker network-disconnect network=my-network container=web
```

### Volume Tools

#### volume-ls
List volumes.

**Parameters**:
- `filter` (optional): Filter output (e.g., "dangling=true", "name=my-vol")
- `format` (optional): Pretty-print using Go template

**Example**:
```bash
skill run docker volume-ls
skill run docker volume-ls filter="dangling=true"
```

#### volume-create
Create a volume.

**Parameters**:
- `name` (required): Volume name
- `driver` (optional): Volume driver (default: local)

**Example**:
```bash
skill run docker volume-create name=my-data
skill run docker volume-create name=my-data driver=local
```

#### volume-rm
Remove one or more volumes.

**Parameters**:
- `name` (required): Volume name(s), comma-separated for multiple
- `force` (optional): Force removal (true/false)

**Example**:
```bash
skill run docker volume-rm name=old-data
skill run docker volume-rm name="old-data,temp-data" force=true
```

#### volume-inspect
Display detailed information about a volume.

**Parameters**:
- `name` (required): Volume name
- `format` (optional): Format output using Go template

**Example**:
```bash
skill run docker volume-inspect name=my-data
skill run docker volume-inspect name=my-data format="{{.Mountpoint}}"
```

### Docker Compose Tools

#### compose-up
Create and start containers defined in docker-compose.yml.

**Parameters**:
- `file` (optional): Path to compose file (default: docker-compose.yml)
- `detach` (optional): Run in background (true/false)
- `build` (optional): Build images before starting (true/false)
- `services` (optional): Specific services to start, comma-separated

**Example**:
```bash
skill run docker compose-up detach=true
skill run docker compose-up file=docker-compose.prod.yml detach=true build=true
skill run docker compose-up services="web,db"
```

#### compose-down
Stop and remove containers, networks created by compose up.

**Parameters**:
- `file` (optional): Path to compose file
- `volumes` (optional): Remove named volumes (true/false)
- `remove_orphans` (optional): Remove orphan containers (true/false)

**Example**:
```bash
skill run docker compose-down
skill run docker compose-down volumes=true remove_orphans=true
```

#### compose-ps
List containers managed by Compose.

**Parameters**:
- `file` (optional): Path to compose file
- `all` (optional): Show all containers including stopped (true/false)

**Example**:
```bash
skill run docker compose-ps
skill run docker compose-ps all=true
```

#### compose-logs
View output from containers.

**Parameters**:
- `file` (optional): Path to compose file
- `service` (optional): Specific service to show logs for
- `follow` (optional): Follow log output (true/false)
- `tail` (optional): Number of lines to show

**Example**:
```bash
skill run docker compose-logs
skill run docker compose-logs service=web tail=100
skill run docker compose-logs follow=true
```

### System Tools

#### system-info
Display system-wide information.

**Example**:
```bash
skill run docker system-info
```

#### system-prune
Remove unused data (stopped containers, unused networks, dangling images).

**Parameters**:
- `all` (optional): Remove all unused images, not just dangling (true/false)
- `volumes` (optional): Also prune volumes (true/false)
- `force` (optional): Do not prompt for confirmation (true/false)

**Example**:
```bash
skill run docker system-prune force=true
skill run docker system-prune all=true volumes=true force=true
```

#### raw
Execute any docker command directly.

**Parameters**:
- `args` (required): Raw docker arguments

**Example**:
```bash
skill run docker raw args="version"
skill run docker raw args="info --format '{{.ServerVersion}}'"
skill run docker raw args="stats --no-stream"
```

## Configuration

This skill uses your existing Docker configuration. No additional configuration is required.

Configure in `.skill-engine.toml`:
```toml
[skills.docker]
source = "./examples/docker-skill"
description = "Docker container and image management"

[skills.docker.instances.default]
# No config needed - uses system Docker
```

For remote Docker hosts:
```bash
export DOCKER_HOST=tcp://192.168.1.100:2376
export DOCKER_TLS_VERIFY=1
```

## Security Model

### Allowed Commands
This skill only executes `docker` commands through the allowlisted command system.

### Blocked Operations
The following operations are blocked for security:
- `--privileged` flag - Grants full host access
- `-v /var/run/docker.sock:/var/run/docker.sock` - Docker-in-Docker escape vector
- `-v /:/host` or similar root filesystem mounts
- `--pid=host` - Host PID namespace access
- `--network=host` with `--privileged`

### Warning Operations
The following operations will proceed but should be used with caution:
- `--network=host` - Container shares host network stack
- Volume mounts to sensitive paths (`/etc`, `/var`, `/root`)
- `system-prune` with `--all --volumes` - Can delete important data

### Best Practices
1. Use named volumes instead of host path mounts when possible
2. Run containers as non-root users (`--user`)
3. Limit container capabilities (`--cap-drop=ALL --cap-add=...`)
4. Use read-only root filesystem when possible (`--read-only`)
5. Set resource limits (`--memory`, `--cpus`)

## Troubleshooting

### Docker daemon not running
```bash
# macOS/Linux
sudo systemctl start docker
# or for Docker Desktop, start the application
```

### Permission denied
```bash
# Add user to docker group (Linux)
sudo usermod -aG docker $USER
# Then log out and back in
```

### Container networking issues
```bash
# Inspect network settings
skill run docker inspect target=container_name format="{{json .NetworkSettings}}"
```
