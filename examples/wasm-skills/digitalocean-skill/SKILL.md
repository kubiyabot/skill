---
name: digitalocean
version: 1.0.0
description: DigitalOcean infrastructure management
author: Skill Engine
---

# DigitalOcean Skill

Manage DigitalOcean droplets, databases, and DNS with API integration.

## Installation

```bash
skill install ./examples/wasm-skills/digitalocean-skill
```

## Configuration

```bash
skill config digitalocean --set DIGITALOCEAN_TOKEN=your_api_token
```

## Tools

### droplet-list
List all droplets in your account.

**Parameters:**
- `tag` (optional, string): Filter by tag
- `region` (optional, string): Filter by region

**Example:**
```
skill run digitalocean droplet-list
skill run digitalocean droplet-list --tag production
```

### droplet-create
Create a new droplet.

**Parameters:**
- `name` (required, string): Droplet name
- `region` (required, string): Region slug (e.g., nyc1, sfo3)
- `size` (required, string): Size slug (e.g., s-1vcpu-1gb)
- `image` (required, string): Image slug or ID
- `ssh_keys` (optional, string): Comma-separated SSH key IDs
- `tags` (optional, string): Comma-separated tags

**Example:**
```
skill run digitalocean droplet-create --name web-02 --region nyc1 --size s-2vcpu-4gb --image ubuntu-22-04-x64
```

### droplet-action
Perform actions on a droplet.

**Parameters:**
- `id` (required, string): Droplet ID
- `action` (required, string): Action: power_on, power_off, reboot, snapshot
- `name` (optional, string): Snapshot name (for snapshot action)

**Example:**
```
skill run digitalocean droplet-action --id 123456789 --action reboot
```

### droplet-delete
Delete a droplet.

**Parameters:**
- `id` (required, string): Droplet ID

**Example:**
```
skill run digitalocean droplet-delete --id 123456789
```

### dns-list
List DNS records for a domain.

**Parameters:**
- `domain` (required, string): Domain name
- `type` (optional, string): Filter by record type

**Example:**
```
skill run digitalocean dns-list --domain example.com
```

### dns-create
Create a DNS record.

**Parameters:**
- `domain` (required, string): Domain name
- `type` (required, string): Record type (A, AAAA, CNAME, MX, TXT)
- `name` (required, string): Record name
- `data` (required, string): Record data
- `ttl` (optional, number): TTL in seconds (default: 3600)
- `priority` (optional, number): Priority (for MX records)

**Example:**
```
skill run digitalocean dns-create --domain example.com --type A --name api --data 167.99.123.45
```

### dns-delete
Delete a DNS record.

**Parameters:**
- `domain` (required, string): Domain name
- `id` (required, string): Record ID

**Example:**
```
skill run digitalocean dns-delete --domain example.com --id 12345
```

### database-list
List managed databases.

**Example:**
```
skill run digitalocean database-list
```
