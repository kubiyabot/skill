# PagerDuty Skill

Manage incidents and on-call schedules with PagerDuty API integration.

## Overview

The PagerDuty skill provides AI agents with incident management capabilities. Create, acknowledge, and resolve incidents, manage on-call schedules, and query service health.

**Runtime**: WASM (JavaScript/TypeScript)
**Source**: [examples/wasm-skills/pagerduty-skill](https://github.com/kubiyabot/skill/tree/main/examples/wasm-skills/pagerduty-skill)

## Installation

```bash
# Install the skill
skill install github:kubiyabot/skill:pagerduty

# Or from local directory
skill install ./examples/wasm-skills/pagerduty-skill
```

## Configuration

Configure your PagerDuty API token:

```bash
skill config pagerduty --set api_key=YOUR_API_KEY
```

Or via environment variables:

```bash
export PAGERDUTY_API_KEY=your_api_key
```

## Tools Reference

### list-incidents

List incidents with optional filtering.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | Filter: `triggered`, `acknowledged`, `resolved` |
| `urgency` | string | No | Filter: `high`, `low` |
| `service` | string | No | Filter by service ID or name |
| `since` | string | No | Start date (ISO 8601 or relative) |
| `until` | string | No | End date |
| `limit` | number | No | Maximum results (default: 25) |

**Examples:**

```bash
# List active incidents
skill run pagerduty list-incidents --status triggered

# List high-urgency incidents
skill run pagerduty list-incidents --urgency high

# Filter by service
skill run pagerduty list-incidents --service "Production API"

# Time range query
skill run pagerduty list-incidents \
  --since "24h ago" \
  --status resolved
```

**Output:**
```json
{
  "incidents": [
    {
      "id": "P1234ABC",
      "title": "High CPU on web-server-01",
      "status": "triggered",
      "urgency": "high",
      "service": "Production API",
      "created_at": "2025-01-13T10:30:00Z",
      "assignments": [
        {"name": "John Doe", "email": "john@example.com"}
      ]
    }
  ]
}
```

### get-incident

Get detailed information about a specific incident.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Incident ID |

**Examples:**

```bash
skill run pagerduty get-incident --id P1234ABC
```

### create-incident

Create a new incident manually.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | Yes | Incident title |
| `service` | string | Yes | Service ID or name |
| `urgency` | string | No | `high` or `low` (default: high) |
| `body` | string | No | Incident description |
| `escalation_policy` | string | No | Escalation policy ID |

**Examples:**

```bash
# Create high-urgency incident
skill run pagerduty create-incident \
  --title "Database connection failures" \
  --service "Production Database" \
  --urgency high \
  --body "Multiple connection timeouts detected"

# Create low-urgency incident
skill run pagerduty create-incident \
  --title "Disk space warning" \
  --service "Monitoring" \
  --urgency low
```

### acknowledge-incident

Acknowledge an incident.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Incident ID |
| `message` | string | No | Acknowledgment message |

**Examples:**

```bash
skill run pagerduty acknowledge-incident \
  --id P1234ABC \
  --message "Investigating the issue"
```

### resolve-incident

Resolve an incident.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Incident ID |
| `resolution` | string | No | Resolution notes |

**Examples:**

```bash
skill run pagerduty resolve-incident \
  --id P1234ABC \
  --resolution "Fixed by scaling up the service"
```

### add-note

Add a note to an incident timeline.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `incident_id` | string | Yes | Incident ID |
| `content` | string | Yes | Note content |

**Examples:**

```bash
skill run pagerduty add-note \
  --incident_id P1234ABC \
  --content "Identified root cause: memory leak in auth service"
```

### list-oncalls

List current on-call assignments.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `schedule` | string | No | Filter by schedule ID or name |
| `escalation_policy` | string | No | Filter by escalation policy |

**Examples:**

```bash
# List all on-call users
skill run pagerduty list-oncalls

# Filter by schedule
skill run pagerduty list-oncalls --schedule "Primary On-Call"
```

**Output:**
```json
{
  "oncalls": [
    {
      "user": {
        "name": "Jane Smith",
        "email": "jane@example.com"
      },
      "schedule": "Primary On-Call",
      "escalation_level": 1,
      "start": "2025-01-13T00:00:00Z",
      "end": "2025-01-14T00:00:00Z"
    }
  ]
}
```

### list-services

List PagerDuty services.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | No | Search query |
| `team` | string | No | Filter by team |

**Examples:**

```bash
# List all services
skill run pagerduty list-services

# Search services
skill run pagerduty list-services --query "production"
```

## Common Workflows

### Incident Response

```bash
# 1. Check active incidents
skill run pagerduty list-incidents --status triggered

# 2. Get incident details
skill run pagerduty get-incident --id P1234ABC

# 3. Acknowledge while investigating
skill run pagerduty acknowledge-incident \
  --id P1234ABC \
  --message "Starting investigation"

# 4. Add investigation notes
skill run pagerduty add-note \
  --incident_id P1234ABC \
  --content "Checked logs - seeing OOM errors"

# 5. Resolve with details
skill run pagerduty resolve-incident \
  --id P1234ABC \
  --resolution "Increased memory limits, deployed fix"
```

### On-Call Handoff

```bash
# Check who's currently on-call
skill run pagerduty list-oncalls

# Review open incidents before handoff
skill run pagerduty list-incidents --status acknowledged
```

### Daily Standup Review

```bash
# Incidents from last 24 hours
skill run pagerduty list-incidents --since "24h ago"

# Currently active incidents
skill run pagerduty list-incidents --status triggered,acknowledged
```

## Security Considerations

- **API Token Scope**: Use tokens with minimal required permissions
- **Read-Only Access**: Create read-only tokens for query-only use cases
- **User Context**: Actions are attributed to the API token owner
- **Sensitive Data**: Incident details may contain sensitive information

## Troubleshooting

### Authentication Failed

```
Error: 401 Unauthorized
```

**Solution**: Verify your API key is valid and has required scopes:

```bash
skill config pagerduty --set api_key=YOUR_NEW_KEY
```

### Service Not Found

```
Error: Service not found
```

**Solution**: Use `list-services` to find the correct service ID or name.

### Rate Limiting

```
Error: 429 Too Many Requests
```

**Solution**: PagerDuty has rate limits. Add delays between bulk operations.

## Integration with Claude Code

```bash
# Natural language commands
"Show me all active PagerDuty incidents"
"Acknowledge incident P1234ABC"
"Who is on-call for the production team?"
"Create an incident for the database service"
```

## Next Steps

- [Datadog Skill](./datadog.md) - Monitoring metrics
- [Grafana Skill](./grafana.md) - Dashboard management
- [Slack Skill](./slack.md) - Team notifications
- [PagerDuty API Docs](https://developer.pagerduty.com/)
