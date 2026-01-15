# DigitalOcean Skill

Manage DigitalOcean droplets, databases, and DNS with API integration.

## Overview

The DigitalOcean skill provides AI agents with access to DigitalOcean infrastructure management. Create and manage droplets, configure DNS records, manage databases, and more through the DigitalOcean API.

**Runtime**: WASM (JavaScript/TypeScript)
**Source**: [examples/wasm-skills/digitalocean-skill](https://github.com/kubiyabot/skill/tree/main/examples/wasm-skills/digitalocean-skill)

## Installation

```bash
# Install the skill
skill install github:kubiyabot/skill:digitalocean

# Or from local directory
skill install ./examples/wasm-skills/digitalocean-skill
```

## Configuration

Configure your DigitalOcean API token:

```bash
skill config digitalocean --set api_token=YOUR_API_TOKEN
```

Or via environment variables:

```bash
export DIGITALOCEAN_TOKEN=your_api_token
```

## Tools Reference

### droplet-list

List all droplets in your account.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tag` | string | No | Filter by tag |
| `region` | string | No | Filter by region |

**Examples:**

```bash
# List all droplets
skill run digitalocean droplet-list

# Filter by tag
skill run digitalocean droplet-list --tag production

# Filter by region
skill run digitalocean droplet-list --region nyc1
```

**Output:**
```json
{
  "droplets": [
    {
      "id": 123456789,
      "name": "web-01",
      "status": "active",
      "size": "s-2vcpu-4gb",
      "region": "nyc1",
      "ip_address": "167.99.123.45",
      "tags": ["production", "web"]
    }
  ]
}
```

### droplet-create

Create a new droplet.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Droplet name |
| `region` | string | Yes | Region slug (e.g., `nyc1`, `sfo3`) |
| `size` | string | Yes | Size slug (e.g., `s-1vcpu-1gb`) |
| `image` | string | Yes | Image slug or ID |
| `ssh_keys` | string | No | Comma-separated SSH key IDs |
| `tags` | string | No | Comma-separated tags |

**Examples:**

```bash
# Create Ubuntu droplet
skill run digitalocean droplet-create \
  --name web-02 \
  --region nyc1 \
  --size s-2vcpu-4gb \
  --image ubuntu-22-04-x64 \
  --tags "production,web"

# Create with SSH keys
skill run digitalocean droplet-create \
  --name api-server \
  --region sfo3 \
  --size s-4vcpu-8gb \
  --image docker-20-04 \
  --ssh_keys "12345,67890"
```

### droplet-action

Perform actions on a droplet (power on/off, reboot, snapshot).

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Droplet ID |
| `action` | string | Yes | Action: `power_on`, `power_off`, `reboot`, `snapshot` |
| `name` | string | No | Snapshot name (for snapshot action) |

**Examples:**

```bash
# Reboot a droplet
skill run digitalocean droplet-action \
  --id 123456789 \
  --action reboot

# Create snapshot
skill run digitalocean droplet-action \
  --id 123456789 \
  --action snapshot \
  --name "pre-upgrade-backup"
```

### droplet-delete

Delete a droplet.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Droplet ID |

**Examples:**

```bash
skill run digitalocean droplet-delete --id 123456789
```

### dns-list

List DNS records for a domain.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `domain` | string | Yes | Domain name |
| `type` | string | No | Filter by record type (A, CNAME, etc.) |

**Examples:**

```bash
# List all records
skill run digitalocean dns-list --domain example.com

# Filter by type
skill run digitalocean dns-list --domain example.com --type A
```

**Output:**
```json
{
  "records": [
    {
      "id": 12345,
      "type": "A",
      "name": "www",
      "data": "167.99.123.45",
      "ttl": 3600
    }
  ]
}
```

### dns-create

Create a DNS record.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `domain` | string | Yes | Domain name |
| `type` | string | Yes | Record type (A, AAAA, CNAME, MX, TXT) |
| `name` | string | Yes | Record name |
| `data` | string | Yes | Record data |
| `ttl` | number | No | TTL in seconds (default: 3600) |
| `priority` | number | No | Priority (for MX records) |

**Examples:**

```bash
# Create A record
skill run digitalocean dns-create \
  --domain example.com \
  --type A \
  --name api \
  --data 167.99.123.45

# Create CNAME record
skill run digitalocean dns-create \
  --domain example.com \
  --type CNAME \
  --name www \
  --data "@"
```

### dns-delete

Delete a DNS record.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `domain` | string | Yes | Domain name |
| `id` | string | Yes | Record ID |

**Examples:**

```bash
skill run digitalocean dns-delete --domain example.com --id 12345
```

### database-list

List managed databases.

**Examples:**

```bash
skill run digitalocean database-list
```

**Output:**
```json
{
  "databases": [
    {
      "id": "9cc10173-e9ea-4176-9dbc-a4cee4c4ff30",
      "name": "production-db",
      "engine": "pg",
      "version": "15",
      "status": "online",
      "size": "db-s-2vcpu-4gb",
      "region": "nyc1",
      "connection": {
        "host": "production-db-do-user-123-0.db.ondigitalocean.com",
        "port": 25060
      }
    }
  ]
}
```

### database-connection

Get database connection details.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Database cluster ID |

**Examples:**

```bash
skill run digitalocean database-connection \
  --id 9cc10173-e9ea-4176-9dbc-a4cee4c4ff30
```

## Common Workflows

### Deploy New Application Server

```bash
# 1. Create droplet
skill run digitalocean droplet-create \
  --name app-server-01 \
  --region nyc1 \
  --size s-2vcpu-4gb \
  --image docker-20-04 \
  --tags "production,app"

# 2. Wait for droplet to be active (check status)
skill run digitalocean droplet-list --tag production

# 3. Create DNS record
skill run digitalocean dns-create \
  --domain example.com \
  --type A \
  --name app \
  --data <droplet_ip>
```

### Disaster Recovery Snapshot

```bash
# 1. List production droplets
skill run digitalocean droplet-list --tag production

# 2. Create snapshot for each
skill run digitalocean droplet-action \
  --id 123456789 \
  --action snapshot \
  --name "dr-backup-2025-01-14"

# 3. Verify snapshot creation
skill run digitalocean snapshot-list
```

### DNS Migration

```bash
# 1. List current records
skill run digitalocean dns-list --domain example.com

# 2. Update A record to new IP
skill run digitalocean dns-delete --domain example.com --id 12345
skill run digitalocean dns-create \
  --domain example.com \
  --type A \
  --name www \
  --data 192.168.1.100
```

## Security Considerations

- **API Token Scope**: Create tokens with minimal required permissions
- **Read-Only Tokens**: Use read-only tokens for monitoring/query use cases
- **Token Rotation**: Rotate API tokens regularly
- **Firewall Rules**: Configure DigitalOcean Cloud Firewalls for droplets

## Troubleshooting

### Authentication Failed

```
Error: 401 Unauthorized
```

**Solution**: Verify your API token is valid:

```bash
skill config digitalocean --set api_token=YOUR_NEW_TOKEN
```

### Rate Limiting

```
Error: 429 Too Many Requests
```

**Solution**: DigitalOcean has rate limits. Add delays between bulk operations.

### Droplet Not Found

```
Error: Droplet not found
```

**Solution**: Verify the droplet ID with `droplet-list`.

## Integration with Claude Code

```bash
# Natural language commands
"List all my DigitalOcean droplets"
"Create a new Ubuntu droplet in NYC"
"Add a DNS A record for api.example.com"
"Reboot the web-01 droplet"
```

## Next Steps

- [AWS Skill](./aws.md) - Amazon Web Services
- [GCP Skill](./gcp.md) - Google Cloud Platform
- [Kubernetes Skill](./kubernetes.md) - Container orchestration
- [DigitalOcean API Docs](https://docs.digitalocean.com/reference/api/)
