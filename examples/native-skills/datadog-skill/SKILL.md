---
name: datadog
description: Datadog monitoring and observability with native dogshell CLI integration
allowed-tools: Bash
---

# Datadog Skill

Comprehensive Datadog monitoring using the native dogshell CLI.

## Overview

This skill provides AI agents with access to Datadog monitoring capabilities through the dogshell CLI. Query metrics, manage monitors, search logs, and create events.

## Requirements

- Datadog Python CLI installed (`pip install datadog`)
- API and Application keys configured

## Configuration

Set your Datadog credentials:

```bash
skill config datadog --set api_key=YOUR_API_KEY --set app_key=YOUR_APP_KEY
```

Or use environment variables:

```bash
export DD_API_KEY=your_api_key
export DD_APP_KEY=your_app_key
```

## Tools

### metric-query

Query time-series metrics from Datadog.

**Parameters:**
- `query` (required, string): Datadog metrics query (e.g., avg:system.cpu.user{*})
- `from` (optional, string): Start time in seconds or relative (e.g., -1h)
- `to` (optional, string): End time in seconds or relative (default: now)

**Examples:**
```bash
skill run datadog metric-query --query "avg:system.cpu.user{*}"
skill run datadog metric-query --query "avg:kubernetes.cpu.usage{cluster:prod}" --from "-1h"
skill run datadog metric-query --query "sum:trace.http.request.errors{env:production}.as_rate()" --from "-24h"
```

### monitor-list

List Datadog monitors.

**Parameters:**
- `name` (optional, string): Filter by monitor name (substring match)
- `tags` (optional, string): Filter by tags (comma-separated)
- `status` (optional, string): Filter by status: Alert, Warn, OK, No Data

**Examples:**
```bash
skill run datadog monitor-list
skill run datadog monitor-list --status Alert
skill run datadog monitor-list --tags "env:production,team:platform"
skill run datadog monitor-list --name "CPU"
```

### monitor-get

Get details of a specific monitor.

**Parameters:**
- `id` (required, string): Monitor ID

**Examples:**
```bash
skill run datadog monitor-get --id 12345678
```

### monitor-mute

Mute a monitor to silence alerts.

**Parameters:**
- `id` (required, string): Monitor ID
- `scope` (optional, string): Scope to mute (e.g., host:web-01)
- `end` (optional, string): End time in seconds or duration (e.g., 3600)

**Examples:**
```bash
skill run datadog monitor-mute --id 12345678
skill run datadog monitor-mute --id 12345678 --scope "host:web-01" --end 3600
```

### monitor-unmute

Unmute a muted monitor.

**Parameters:**
- `id` (required, string): Monitor ID
- `scope` (optional, string): Scope to unmute

**Examples:**
```bash
skill run datadog monitor-unmute --id 12345678
```

### event-post

Post a custom event to Datadog.

**Parameters:**
- `title` (required, string): Event title
- `text` (required, string): Event description
- `alert_type` (optional, string): Type: info, warning, error, success (default: info)
- `tags` (optional, string): Comma-separated tags

**Examples:**
```bash
skill run datadog event-post --title "Deployment: api-v2.3.0" --text "Deployed new version to production" --alert_type success --tags "env:production,service:api"
```

### host-list

List hosts reporting to Datadog.

**Parameters:**
- `filter` (optional, string): Filter expression
- `count` (optional, number): Maximum hosts to return

**Examples:**
```bash
skill run datadog host-list
skill run datadog host-list --filter "env:production"
skill run datadog host-list --count 50
```

### host-mute

Mute a host.

**Parameters:**
- `hostname` (required, string): Host name
- `message` (optional, string): Mute reason
- `end` (optional, string): End time in seconds or duration

**Examples:**
```bash
skill run datadog host-mute --hostname web-01 --message "Maintenance window" --end 7200
```

### host-unmute

Unmute a muted host.

**Parameters:**
- `hostname` (required, string): Host name

**Examples:**
```bash
skill run datadog host-unmute --hostname web-01
```

### downtime-schedule

Schedule a downtime.

**Parameters:**
- `scope` (required, string): Scope for downtime (e.g., host:web-01 or env:production)
- `start` (optional, string): Start time in seconds (default: now)
- `end` (optional, string): End time in seconds or duration
- `message` (optional, string): Downtime message

**Examples:**
```bash
skill run datadog downtime-schedule --scope "host:web-01" --end 7200 --message "Scheduled maintenance"
skill run datadog downtime-schedule --scope "env:staging" --end 3600
```

### downtime-list

List scheduled downtimes.

**Parameters:**
- `current_only` (optional, boolean): Show only current downtimes

**Examples:**
```bash
skill run datadog downtime-list
skill run datadog downtime-list --current_only
```

### dashboard-list

List Datadog dashboards.

**Examples:**
```bash
skill run datadog dashboard-list
```

### service-check

Post a service check.

**Parameters:**
- `check` (required, string): Check name
- `host` (required, string): Host name
- `status` (required, string): Status: ok, warning, critical, unknown
- `message` (optional, string): Check message
- `tags` (optional, string): Comma-separated tags

**Examples:**
```bash
skill run datadog service-check --check "app.health" --host web-01 --status ok --message "All systems operational"
```

## Security Considerations

- Store API and App keys securely
- Use scoped API keys with minimal permissions
- Enable audit logging for compliance
- Consider using Datadog's RBAC for team access

## Troubleshooting

### Authentication Failed
```
ERROR: 403 Forbidden - Invalid API key
```
**Solution:** Verify your API key is correct and has appropriate permissions

### No Data
```
WARNING: Query returned no data
```
**Solution:** Check your query syntax and time range. Use Datadog UI to validate queries first.

### Rate Limiting
```
ERROR: 429 Too Many Requests
```
**Solution:** Add delays between bulk operations. Datadog has rate limits per API key.
