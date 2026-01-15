# Grafana Skill

Manage Grafana dashboards, alerts, and data sources through the API.

## Overview

The Grafana skill enables AI agents to interact with Grafana for dashboard management, alert configuration, and visualization queries. Built as a WASM component for secure, sandboxed execution.

**Runtime**: WASM (JavaScript/TypeScript)
**Source**: [examples/wasm-skills/grafana-skill](https://github.com/kubiyabot/skill/tree/main/examples/wasm-skills/grafana-skill)

## Installation

```bash
# Install the skill
skill install github:kubiyabot/skill:grafana

# Or from local directory
skill install ./examples/wasm-skills/grafana-skill
```

## Configuration

Configure your Grafana connection:

```bash
skill config grafana \
  --set url=https://grafana.example.com \
  --set api_key=YOUR_API_KEY
```

Or via environment variables:

```bash
export GRAFANA_URL=https://grafana.example.com
export GRAFANA_API_KEY=your_api_key
```

## Tools Reference

### list-dashboards

List all dashboards with optional filtering.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `folder` | string | No | Filter by folder name |
| `tag` | string | No | Filter by tag |
| `query` | string | No | Search query |
| `limit` | number | No | Maximum results (default: 100) |

**Examples:**

```bash
# List all dashboards
skill run grafana list-dashboards

# Filter by folder
skill run grafana list-dashboards --folder "Production"

# Search dashboards
skill run grafana list-dashboards --query "kubernetes"

# Filter by tag
skill run grafana list-dashboards --tag "infrastructure"
```

**Output:**
```json
{
  "dashboards": [
    {
      "uid": "abc123",
      "title": "Kubernetes Overview",
      "folder": "Production",
      "tags": ["kubernetes", "infrastructure"],
      "url": "/d/abc123/kubernetes-overview"
    }
  ]
}
```

### get-dashboard

Get dashboard details including panels.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `uid` | string | Yes | Dashboard UID |

**Examples:**

```bash
skill run grafana get-dashboard --uid abc123
```

### create-dashboard

Create a new dashboard from JSON definition.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | Yes | Dashboard title |
| `folder` | string | No | Target folder |
| `panels` | json | No | Panel definitions (JSON) |
| `template` | string | No | Use template: `basic`, `kubernetes`, `node` |

**Examples:**

```bash
# Create from template
skill run grafana create-dashboard \
  --title "My Service Dashboard" \
  --folder "Development" \
  --template kubernetes

# Create empty dashboard
skill run grafana create-dashboard \
  --title "Custom Dashboard" \
  --folder "Team A"
```

### list-alerts

List alert rules.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `state` | string | No | Filter by state: `firing`, `pending`, `inactive` |
| `folder` | string | No | Filter by folder |
| `limit` | number | No | Maximum results (default: 100) |

**Examples:**

```bash
# List all alerts
skill run grafana list-alerts

# List firing alerts
skill run grafana list-alerts --state firing

# List alerts in folder
skill run grafana list-alerts --folder "Production Alerts"
```

### get-alert

Get details of a specific alert rule.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `uid` | string | Yes | Alert rule UID |

**Examples:**

```bash
skill run grafana get-alert --uid alert-xyz
```

### silence-alert

Create a silence for an alert.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `alert_uid` | string | Yes | Alert rule UID |
| `duration` | string | Yes | Silence duration (e.g., `1h`, `30m`, `2d`) |
| `comment` | string | No | Reason for silence |

**Examples:**

```bash
# Silence for 1 hour
skill run grafana silence-alert \
  --alert_uid alert-xyz \
  --duration "1h" \
  --comment "Maintenance window"
```

### query-datasource

Query a data source directly.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `datasource` | string | Yes | Data source name or UID |
| `query` | string | Yes | Query string (PromQL, SQL, etc.) |
| `from` | string | No | Start time (default: 1h ago) |
| `to` | string | No | End time (default: now) |

**Examples:**

```bash
# Query Prometheus
skill run grafana query-datasource \
  --datasource "Prometheus" \
  --query "rate(http_requests_total[5m])"

# Query with time range
skill run grafana query-datasource \
  --datasource "Prometheus" \
  --query "sum(container_memory_usage_bytes) by (pod)" \
  --from "6h ago"
```

### list-datasources

List configured data sources.

**Examples:**

```bash
skill run grafana list-datasources
```

**Output:**
```json
{
  "datasources": [
    {
      "name": "Prometheus",
      "type": "prometheus",
      "url": "http://prometheus:9090",
      "default": true
    },
    {
      "name": "Loki",
      "type": "loki",
      "url": "http://loki:3100"
    }
  ]
}
```

## Common Workflows

### Incident Investigation

```bash
# 1. Check firing alerts
skill run grafana list-alerts --state firing

# 2. Get alert details
skill run grafana get-alert --uid alert-high-cpu

# 3. Query related metrics
skill run grafana query-datasource \
  --datasource "Prometheus" \
  --query "avg(rate(container_cpu_usage_seconds_total[5m])) by (pod)"

# 4. Find relevant dashboards
skill run grafana list-dashboards --query "cpu"
```

### Dashboard Maintenance

```bash
# List all production dashboards
skill run grafana list-dashboards --folder "Production"

# Export dashboard for backup
skill run grafana get-dashboard --uid prod-overview

# Create new dashboard from template
skill run grafana create-dashboard \
  --title "New Service Metrics" \
  --folder "Production" \
  --template basic
```

### Silence During Maintenance

```bash
# Silence all production alerts for 2 hours
skill run grafana list-alerts --folder "Production Alerts" | \
  xargs -I {} skill run grafana silence-alert \
    --alert_uid {} \
    --duration "2h" \
    --comment "Scheduled maintenance"
```

## Security Considerations

- **API Key Scope**: Create API keys with minimal required permissions
- **Network Access**: WASM skill only accesses configured Grafana URL
- **Read-Only Option**: Use Viewer-level API keys for read-only access
- **Audit Trail**: All operations logged in Grafana audit log

## Troubleshooting

### Connection Failed

```
Error: Failed to connect to Grafana
```

**Solution**: Verify URL and network connectivity:

```bash
skill config grafana --set url=https://correct-grafana-url.com
```

### Invalid API Key

```
Error: 401 Unauthorized
```

**Solution**: Generate a new API key with appropriate permissions in Grafana Admin.

### Dashboard Not Found

```
Error: Dashboard not found
```

**Solution**: Verify the UID is correct. Use `list-dashboards` to find valid UIDs.

## Integration with Claude Code

```bash
# Natural language queries work
"Show me all firing alerts in Grafana"
"Create a new dashboard for my Node.js service"
"Query Prometheus for HTTP error rates"
"Silence the high-memory alert for 1 hour"
```

## Next Steps

- [Datadog Skill](./datadog.md) - Metrics and monitoring
- [PagerDuty Skill](./pagerduty.md) - Incident management
- [Prometheus Skill](./prometheus.md) - Direct Prometheus access
- [Grafana Documentation](https://grafana.com/docs/)
