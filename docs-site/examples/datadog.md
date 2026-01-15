# Datadog Skill

Monitor infrastructure and applications with Datadog CLI integration.

## Overview

The Datadog skill provides AI agents with access to Datadog monitoring capabilities. Query metrics, manage monitors, search logs, and handle incidents through the Datadog CLI.

**Runtime**: Native (wraps `datadog-ci` and `dogshell` CLI)
**Source**: [examples/native-skills/datadog-skill](https://github.com/kubiyabot/skill/tree/main/examples/native-skills/datadog-skill)

## Installation

```bash
# Install Datadog CLI (prerequisite)
pip install datadog

# Or via npm for datadog-ci
npm install -g @datadog/datadog-ci

# Install the skill
skill install ./examples/native-skills/datadog-skill
```

## Configuration

Set up your Datadog credentials:

```bash
skill config datadog --set api_key=YOUR_API_KEY --set app_key=YOUR_APP_KEY
```

Or via environment variables:

```bash
export DD_API_KEY=your_api_key
export DD_APP_KEY=your_app_key
```

## Tools Reference

### query-metrics

Query time-series metrics from Datadog.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | Datadog metrics query (e.g., `avg:system.cpu.user{*}`) |
| `from` | string | No | Start time (default: 1 hour ago) |
| `to` | string | No | End time (default: now) |
| `format` | string | No | Output format: `json`, `table` (default: json) |

**Examples:**

```bash
# Query average CPU usage
skill run datadog query-metrics --query "avg:system.cpu.user{*}"

# Query with time range
skill run datadog query-metrics \
  --query "avg:kubernetes.cpu.usage{cluster:prod}" \
  --from "1h ago" \
  --to "now"

# Query memory by host
skill run datadog query-metrics \
  --query "avg:system.mem.used{*} by {host}" \
  --format table
```

**Output:**
```json
{
  "series": [
    {
      "metric": "system.cpu.user",
      "points": [
        [1705234800, 45.2],
        [1705234860, 47.8]
      ],
      "scope": "host:web-01"
    }
  ]
}
```

### list-monitors

List and filter Datadog monitors.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | No | Filter by monitor name |
| `tags` | string | No | Filter by tags (comma-separated) |
| `status` | string | No | Filter by status: `Alert`, `Warn`, `OK`, `No Data` |
| `limit` | number | No | Maximum results (default: 100) |

**Examples:**

```bash
# List all monitors
skill run datadog list-monitors

# Filter by status
skill run datadog list-monitors --status Alert

# Filter by tags
skill run datadog list-monitors --tags "env:production,team:platform"

# Search by name
skill run datadog list-monitors --name "CPU"
```

### get-monitor

Get details of a specific monitor.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Monitor ID |

**Examples:**

```bash
skill run datadog get-monitor --id 12345678
```

### mute-monitor

Mute a monitor to silence alerts.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Monitor ID |
| `scope` | string | No | Scope to mute (e.g., `host:web-01`) |
| `end` | string | No | End time for mute (default: indefinite) |

**Examples:**

```bash
# Mute monitor indefinitely
skill run datadog mute-monitor --id 12345678

# Mute for specific host for 1 hour
skill run datadog mute-monitor \
  --id 12345678 \
  --scope "host:web-01" \
  --end "1h"
```

### search-logs

Search and retrieve logs from Datadog.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | Log search query |
| `from` | string | No | Start time (default: 15 minutes ago) |
| `to` | string | No | End time (default: now) |
| `limit` | number | No | Maximum logs to return (default: 50) |
| `sort` | string | No | Sort order: `asc`, `desc` (default: desc) |

**Examples:**

```bash
# Search for error logs
skill run datadog search-logs --query "status:error"

# Search in specific service
skill run datadog search-logs \
  --query "service:api-gateway status:error" \
  --from "1h ago"

# Search with limit
skill run datadog search-logs \
  --query "@http.status_code:>=500" \
  --limit 100
```

### create-event

Post a custom event to Datadog.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | Yes | Event title |
| `text` | string | Yes | Event description |
| `alert_type` | string | No | Type: `info`, `warning`, `error`, `success` |
| `tags` | string | No | Comma-separated tags |

**Examples:**

```bash
# Post deployment event
skill run datadog create-event \
  --title "Deployment: api-v2.3.0" \
  --text "Deployed new version to production" \
  --alert_type success \
  --tags "env:production,service:api"
```

## Common Workflows

### Investigate High CPU Alert

```bash
# 1. Check current monitors in alert
skill run datadog list-monitors --status Alert

# 2. Query CPU metrics for affected hosts
skill run datadog query-metrics \
  --query "avg:system.cpu.user{host:web-*} by {host}" \
  --from "30m ago"

# 3. Search for related error logs
skill run datadog search-logs \
  --query "host:web-* status:error" \
  --from "30m ago"
```

### Deploy with Monitoring

```bash
# 1. Post deployment start event
skill run datadog create-event \
  --title "Deployment Started: myapp-v1.2.0" \
  --text "Starting rolling deployment" \
  --alert_type info

# 2. Mute deployment-sensitive monitors
skill run datadog mute-monitor --id 12345 --end "15m"

# ... deploy your application ...

# 3. Post deployment complete event
skill run datadog create-event \
  --title "Deployment Complete: myapp-v1.2.0" \
  --text "Deployment successful" \
  --alert_type success
```

### Daily Health Check

```bash
# Check for any alerts
skill run datadog list-monitors --status Alert

# Review error rates
skill run datadog query-metrics \
  --query "sum:trace.http.request.errors{env:production}.as_rate()" \
  --from "24h ago"
```

## Security Considerations

- **API Keys**: Store API and App keys securely using Skill Engine's secret management
- **Scope Limits**: Consider creating Datadog API keys with limited scope
- **Read-Only Access**: Use read-only App keys for query-only use cases
- **Audit Logging**: All operations are logged for compliance

## Troubleshooting

### Authentication Errors

```
Error: 403 Forbidden - Invalid API key
```

**Solution**: Verify your API key is correct and has appropriate permissions:

```bash
skill config datadog --set api_key=YOUR_CORRECT_KEY
```

### Query Syntax Errors

```
Error: Invalid query syntax
```

**Solution**: Check Datadog query syntax. Use the Datadog UI to validate queries first.

### Rate Limiting

```
Error: 429 Too Many Requests
```

**Solution**: Datadog has rate limits. Add delays between bulk operations or use pagination.

## Integration with Claude Code

```bash
# In Claude Code, use natural language
"Check if there are any Datadog monitors currently alerting"
"Show me CPU metrics for the production cluster"
"Search for error logs from the API service in the last hour"
```

## Next Steps

- [Grafana Skill](./grafana.md) - Dashboard visualization
- [PagerDuty Skill](./pagerduty.md) - Incident management
- [Kubernetes Skill](./kubernetes.md) - Cluster management
- [Datadog Documentation](https://docs.datadoghq.com/)
