---
name: redis
description: Redis CLI client for cache and data store operations running inside a Docker container
runtime: docker
---

# Redis Skill

Access Redis databases using the `redis-cli` client running inside a Docker container. Set/get keys, manage data structures, monitor performance, and perform cache operations without installing Redis locally.

## Overview

This skill provides the `redis-cli` command-line client through a containerized runtime. Connect to any Redis instance (local or remote), execute commands, manipulate data structures, and manage your cache - all while keeping your host system clean.

## When to Use This Skill

**Use this skill when you need to**:
- Connect to Redis instances without local installation
- Set and get cached values
- Manage Redis data structures (strings, lists, sets, hashes, sorted sets)
- Monitor Redis performance and commands
- Test cache connections
- Debug caching issues
- Flush databases and manage keys
- Pub/Sub messaging operations
- Execute administrative commands

## Prerequisites

### Docker

The Redis client runs inside a Docker container:

```bash
# macOS (using Homebrew)
brew install --cask docker

# Linux (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install docker.io
sudo systemctl start docker
```

### Redis Server

You need access to a Redis server:
- Local Redis instance
- Remote Redis server
- Cloud-hosted Redis (AWS ElastiCache, Redis Cloud, Azure Cache, etc.)
- Docker Redis container

## Configuration

Add to your `.skill-engine.toml`:

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

### Configuration Explained

- **image**: `redis:7-alpine` - Redis 7.x Alpine image (~30MB)
- **entrypoint**: `redis-cli` - Redis command-line interface
- **network**: `bridge` - Allows connections to Redis servers (local and remote)
- **memory**: `128m` - Limits container to 128MB RAM
- **rm**: `true` - Automatically removes container after execution

## Usage

Redis skill uses pass-through syntax - all arguments after `--` are passed directly to `redis-cli`:

```bash
skill run redis -- [redis-cli arguments] [COMMAND] [args...]
```

## Connection Methods

### Using Command-Line Arguments

```bash
# Connect to local Redis
skill run redis -- -h localhost

# Connect with specific port
skill run redis -- -h localhost -p 6379

# Connect to remote Redis
skill run redis -- -h redis.example.com -p 6379

# Connect with authentication
skill run redis -- -h localhost -a password

# Connect to specific database number
skill run redis -- -h localhost -n 1
```

### Using Environment Variables

```bash
# Set Redis password
export REDISCLI_AUTH=your_password

# Connect with authentication
skill run redis -- -h localhost
```

## Common Operations

### String Operations

```bash
# Set a key
skill run redis -- -h localhost SET mykey "myvalue"

# Get a key
skill run redis -- -h localhost GET mykey

# Set with expiry (EX = seconds)
skill run redis -- -h localhost SET session:123 "data" EX 3600

# Set multiple keys
skill run redis -- -h localhost MSET key1 "value1" key2 "value2"

# Get multiple keys
skill run redis -- -h localhost MGET key1 key2

# Increment counter
skill run redis -- -h localhost INCR counter

# Decrement counter
skill run redis -- -h localhost DECR counter

# Append to string
skill run redis -- -h localhost APPEND mykey " more data"

# Get string length
skill run redis -- -h localhost STRLEN mykey
```

### Key Management

```bash
# List all keys
skill run redis -- -h localhost KEYS "*"

# List keys with pattern
skill run redis -- -h localhost KEYS "user:*"

# Check if key exists
skill run redis -- -h localhost EXISTS mykey

# Delete key
skill run redis -- -h localhost DEL mykey

# Delete multiple keys
skill run redis -- -h localhost DEL key1 key2 key3

# Rename key
skill run redis -- -h localhost RENAME oldkey newkey

# Get key type
skill run redis -- -h localhost TYPE mykey

# Set expiration (seconds)
skill run redis -- -h localhost EXPIRE mykey 60

# Set expiration (milliseconds)
skill run redis -- -h localhost PEXPIRE mykey 60000

# Remove expiration
skill run redis -- -h localhost PERSIST mykey

# Get time to live
skill run redis -- -h localhost TTL mykey

# Get TTL in milliseconds
skill run redis -- -h localhost PTTL mykey
```

### Hash Operations

```bash
# Set hash field
skill run redis -- -h localhost HSET user:1 name "John"

# Set multiple hash fields
skill run redis -- -h localhost HMSET user:1 name "John" email "john@example.com" age 30

# Get hash field
skill run redis -- -h localhost HGET user:1 name

# Get multiple hash fields
skill run redis -- -h localhost HMGET user:1 name email

# Get all hash fields and values
skill run redis -- -h localhost HGETALL user:1

# Get all hash keys
skill run redis -- -h localhost HKEYS user:1

# Get all hash values
skill run redis -- -h localhost HVALS user:1

# Check if hash field exists
skill run redis -- -h localhost HEXISTS user:1 name

# Delete hash field
skill run redis -- -h localhost HDEL user:1 age

# Increment hash field
skill run redis -- -h localhost HINCRBY user:1 visits 1
```

### List Operations

```bash
# Push to left (beginning)
skill run redis -- -h localhost LPUSH mylist "item1"

# Push to right (end)
skill run redis -- -h localhost RPUSH mylist "item2"

# Push multiple items
skill run redis -- -h localhost RPUSH mylist "item3" "item4" "item5"

# Get list length
skill run redis -- -h localhost LLEN mylist

# Get list range
skill run redis -- -h localhost LRANGE mylist 0 -1  # All items

# Get first N items
skill run redis -- -h localhost LRANGE mylist 0 9

# Get specific range
skill run redis -- -h localhost LRANGE mylist 5 10

# Pop from left
skill run redis -- -h localhost LPOP mylist

# Pop from right
skill run redis -- -h localhost RPOP mylist

# Get item by index
skill run redis -- -h localhost LINDEX mylist 0

# Set item by index
skill run redis -- -h localhost LSET mylist 0 "new value"

# Trim list
skill run redis -- -h localhost LTRIM mylist 0 99
```

### Set Operations

```bash
# Add members to set
skill run redis -- -h localhost SADD myset "member1"

# Add multiple members
skill run redis -- -h localhost SADD myset "member2" "member3"

# Get all members
skill run redis -- -h localhost SMEMBERS myset

# Check if member exists
skill run redis -- -h localhost SISMEMBER myset "member1"

# Get set size
skill run redis -- -h localhost SCARD myset

# Remove member
skill run redis -- -h localhost SREM myset "member1"

# Pop random member
skill run redis -- -h localhost SPOP myset

# Get random member (without removing)
skill run redis -- -h localhost SRANDMEMBER myset

# Set intersection
skill run redis -- -h localhost SINTER set1 set2

# Set union
skill run redis -- -h localhost SUNION set1 set2

# Set difference
skill run redis -- -h localhost SDIFF set1 set2
```

### Sorted Set Operations

```bash
# Add member with score
skill run redis -- -h localhost ZADD leaderboard 100 "player1"

# Add multiple members
skill run redis -- -h localhost ZADD leaderboard 85 "player2" 92 "player3"

# Get range by rank
skill run redis -- -h localhost ZRANGE leaderboard 0 -1

# Get range with scores
skill run redis -- -h localhost ZRANGE leaderboard 0 -1 WITHSCORES

# Get reverse range (highest first)
skill run redis -- -h localhost ZREVRANGE leaderboard 0 -1 WITHSCORES

# Get range by score
skill run redis -- -h localhost ZRANGEBYSCORE leaderboard 80 100

# Get member score
skill run redis -- -h localhost ZSCORE leaderboard "player1"

# Get member rank
skill run redis -- -h localhost ZRANK leaderboard "player1"

# Increment score
skill run redis -- -h localhost ZINCRBY leaderboard 5 "player1"

# Get sorted set size
skill run redis -- -h localhost ZCARD leaderboard

# Remove member
skill run redis -- -h localhost ZREM leaderboard "player1"

# Remove by rank range
skill run redis -- -h localhost ZREMRANGEBYRANK leaderboard 0 2

# Remove by score range
skill run redis -- -h localhost ZREMRANGEBYSCORE leaderboard 0 50
```

### Pub/Sub Operations

```bash
# Subscribe to channel
skill run redis -- -h localhost SUBSCRIBE mychannel

# Subscribe to multiple channels
skill run redis -- -h localhost SUBSCRIBE channel1 channel2

# Subscribe to pattern
skill run redis -- -h localhost PSUBSCRIBE "news:*"

# Publish message
skill run redis -- -h localhost PUBLISH mychannel "Hello World"

# List active channels
skill run redis -- -h localhost PUBSUB CHANNELS

# Count subscribers
skill run redis -- -h localhost PUBSUB NUMSUB mychannel
```

## Database Management

### Select Database

```bash
# Switch to database 0 (default)
skill run redis -- -h localhost -n 0

# Switch to database 1
skill run redis -- -h localhost -n 1

# Select within session
skill run redis -- -h localhost SELECT 2
```

### Flush Operations

```bash
# Flush current database
skill run redis -- -h localhost FLUSHDB

# Flush all databases
skill run redis -- -h localhost FLUSHALL

# Async flush (non-blocking)
skill run redis -- -h localhost FLUSHDB ASYNC
```

## Monitoring & Administration

### Server Information

```bash
# Get all server info
skill run redis -- -h localhost INFO

# Get specific section
skill run redis -- -h localhost INFO server
skill run redis -- -h localhost INFO memory
skill run redis -- -h localhost INFO stats
skill run redis -- -h localhost INFO replication
skill run redis -- -h localhost INFO cpu

# Get server config
skill run redis -- -h localhost CONFIG GET "*"

# Get specific config
skill run redis -- -h localhost CONFIG GET maxmemory

# Set config value
skill run redis -- -h localhost CONFIG SET maxmemory 100mb
```

### Monitoring Commands

```bash
# Monitor all commands in real-time
skill run redis -- -h localhost MONITOR

# Get slow log
skill run redis -- -h localhost SLOWLOG GET 10

# Reset slow log
skill run redis -- -h localhost SLOWLOG RESET

# Get client list
skill run redis -- -h localhost CLIENT LIST

# Get current client name
skill run redis -- -h localhost CLIENT GETNAME

# Set client name
skill run redis -- -h localhost CLIENT SETNAME myapp

# Kill client
skill run redis -- -h localhost CLIENT KILL addr 127.0.0.1:6379
```

### Performance & Statistics

```bash
# Get database size
skill run redis -- -h localhost DBSIZE

# Get last save time
skill run redis -- -h localhost LASTSAVE

# Measure command latency
skill run redis -- -h localhost --latency

# Continuous latency monitoring
skill run redis -- -h localhost --latency-history

# Benchmark
skill run redis -- -h localhost --stat

# Ping server
skill run redis -- -h localhost PING
```

## Advanced Examples

### Cache Pattern

```bash
# Check cache
skill run redis -- -h localhost GET cache:user:123

# If miss, set cache with TTL
skill run redis -- -h localhost SET cache:user:123 '{"name":"John"}' EX 300

# Delete cache
skill run redis -- -h localhost DEL cache:user:123

# Clear all cache keys
skill run redis -- -h localhost EVAL "return redis.call('del', unpack(redis.call('keys', 'cache:*')))" 0
```

### Session Management

```bash
# Create session
skill run redis -- -h localhost SET session:abc123 '{"user_id":1,"email":"user@example.com"}' EX 3600

# Get session
skill run redis -- -h localhost GET session:abc123

# Extend session TTL
skill run redis -- -h localhost EXPIRE session:abc123 3600

# Delete session
skill run redis -- -h localhost DEL session:abc123
```

### Rate Limiting

```bash
# Simple rate limit (5 requests per minute)
skill run redis -- -h localhost INCR rate:user:123:minute
skill run redis -- -h localhost EXPIRE rate:user:123:minute 60

# Check rate limit
skill run redis -- -h localhost GET rate:user:123:minute
```

### Distributed Locks

```bash
# Acquire lock
skill run redis -- -h localhost SET lock:resource:1 "token123" NX EX 30

# Release lock
skill run redis -- -h localhost DEL lock:resource:1

# Check lock
skill run redis -- -h localhost GET lock:resource:1
```

## Batch Operations

### Execute Multiple Commands

```bash
# Pipe commands to redis-cli
cat << EOF | skill run redis -- -h localhost
SET key1 "value1"
SET key2 "value2"
GET key1
EOF

# From file
skill run redis -- -h localhost < commands.txt
```

### Scan Keys

```bash
# Scan keys (safer than KEYS for production)
skill run redis -- -h localhost SCAN 0 MATCH "user:*" COUNT 100

# Scan with cursor
skill run redis -- -h localhost SCAN 0
skill run redis -- -h localhost SCAN cursor MATCH "pattern*"
```

## Security

### Authentication

```bash
# Connect with password
skill run redis -- -h localhost -a your_password

# Or use environment variable
export REDISCLI_AUTH=your_password
skill run redis -- -h localhost

# Authenticate in session
skill run redis -- -h localhost AUTH your_password
```

### ACL (Redis 6+)

```bash
# List users
skill run redis -- -h localhost ACL LIST

# Get current user
skill run redis -- -h localhost ACL WHOAMI

# Create user
skill run redis -- -h localhost ACL SETUSER alice on \>password ~cached:* +get

# Delete user
skill run redis -- -h localhost ACL DELUSER alice
```

## Troubleshooting

### Connection Issues

```bash
# Test connection
skill run redis -- -h localhost PING

# Check server is running
docker ps | grep redis

# Verify port
telnet localhost 6379
```

### Authentication Failed

```bash
# Check password
export REDISCLI_AUTH=correct_password
skill run redis -- -h localhost PING

# Or use -a flag
skill run redis -- -h localhost -a correct_password PING
```

### Cannot Connect to Docker Network

```bash
# Connect to host Redis from container
# macOS/Windows: use host.docker.internal
skill run redis -- -h host.docker.internal

# Linux: use host IP
skill run redis -- -h 172.17.0.1
```

### Performance Issues

```bash
# Check slow queries
skill run redis -- -h localhost SLOWLOG GET 10

# Check memory usage
skill run redis -- -h localhost INFO memory

# Check connected clients
skill run redis -- -h localhost CLIENT LIST
```

## Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "Connection refused" | Redis not running or wrong port | Check server status and port |
| "NOAUTH Authentication required" | Password needed | Use `-a password` or set REDISCLI_AUTH |
| "WRONGPASS invalid password" | Incorrect password | Verify password |
| "MOVED" or "ASK" | Redis Cluster redirect | Connect to correct node |
| "OOM command not allowed" | Out of memory | Check maxmemory setting |

## Redis CLI Options

### Useful Flags

```bash
# Raw output (no formatting)
skill run redis -- -h localhost --raw GET mykey

# CSV output
skill run redis -- -h localhost --csv LRANGE mylist 0 -1

# Execute and exit
skill run redis -- -h localhost -c "SET key value"

# Repeat command N times
skill run redis -- -h localhost --repeat 5 PING

# Sleep between commands
skill run redis -- -h localhost --interval 1 --repeat 10 INFO stats

# Enable cluster mode
skill run redis -- -h localhost -c --cluster

# SSL/TLS connection
skill run redis -- -h localhost --tls --cacert ca.crt
```

## Docker Image Details

- **Image**: `redis:7-alpine`
- **Base**: Alpine Linux
- **Size**: ~30MB
- **Redis Version**: 7.x
- **Included Tools**: redis-cli, redis-benchmark
- **Platforms**: linux/amd64, linux/arm64

## Resources

- [Redis Documentation](https://redis.io/documentation)
- [Redis Commands Reference](https://redis.io/commands)
- [Redis CLI Documentation](https://redis.io/docs/ui/cli/)
- [Redis Data Types](https://redis.io/docs/data-types/)
- [Redis Pub/Sub](https://redis.io/docs/interact/pubsub/)
- [Redis Patterns](https://redis.io/docs/manual/patterns/)

## Quick Reference

### Essential Commands Table

| Operation | Command |
|-----------|---------|
| Connect | `-h host -p port` |
| Authenticate | `-a password` |
| Set key | `SET key value` |
| Get key | `GET key` |
| Set with TTL | `SET key value EX seconds` |
| Delete key | `DEL key` |
| List keys | `KEYS pattern` |
| Check exists | `EXISTS key` |
| Set expiry | `EXPIRE key seconds` |
| Get TTL | `TTL key` |
| Hash set | `HSET hash field value` |
| Hash get | `HGET hash field` |
| List push | `RPUSH list value` |
| List range | `LRANGE list 0 -1` |
| Set add | `SADD set member` |
| Sorted set add | `ZADD zset score member` |
| Publish | `PUBLISH channel message` |
| Subscribe | `SUBSCRIBE channel` |
| Server info | `INFO` |
| Monitor | `MONITOR` |
| Flush DB | `FLUSHDB` |
| Ping | `PING` |
