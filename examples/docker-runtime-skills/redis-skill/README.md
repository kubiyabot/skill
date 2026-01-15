# Redis Skill

Redis CLI client for cache and data store operations.

## Usage

```bash
# Connect to Redis
skill run redis -- -h localhost

# Set a key
skill run redis -- -h localhost SET mykey "myvalue"

# Get a key
skill run redis -- -h localhost GET mykey

# List all keys
skill run redis -- -h localhost KEYS "*"

# Monitor commands
skill run redis -- -h localhost MONITOR

# Get server info
skill run redis -- -h localhost INFO
```

## Configuration

```toml
[skills.redis]
source = "docker:redis:7-alpine"
runtime = "docker"
description = "Redis CLI client"

[skills.redis.docker]
image = "redis:7-alpine"
entrypoint = "redis-cli"
network = "bridge"
memory = "128m"
rm = true
```

## Security

- **Network**: `bridge` - Required for Redis connections
- **Memory**: 128MB limit
- **Authentication**: Use `-a` flag or `REDISCLI_AUTH` env var

## Environment Variables

| Variable | Description |
|----------|-------------|
| `REDISCLI_AUTH` | Redis password |

## Common Operations

| Operation | Command |
|-----------|---------|
| Connect | `-h host -p port` |
| Authenticate | `-a password` |
| Set key | `SET key value` |
| Get key | `GET key` |
| Delete key | `DEL key` |
| List keys | `KEYS pattern` |
| Set expiry | `EXPIRE key seconds` |
| TTL | `TTL key` |
| Hash set | `HSET hash field value` |
| Hash get | `HGET hash field` |
| List push | `LPUSH list value` |
| List range | `LRANGE list 0 -1` |
| Pub/Sub | `PUBLISH channel message` |
| Monitor | `MONITOR` |
| Info | `INFO` |
| Flush DB | `FLUSHDB` |

## Docker Image

- **Image**: `redis:7-alpine`
- **Size**: ~30MB
- **Redis**: 7.x
- **Includes**: redis-cli, redis-benchmark
