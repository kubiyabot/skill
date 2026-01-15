---
name: grafana
version: 1.0.0
description: Grafana dashboard and alert management
author: Skill Engine
---

# Grafana Skill

Manage Grafana dashboards, alerts, and data sources through the API.

## Installation

```bash
skill install ./examples/wasm-skills/grafana-skill
```

## Configuration

```bash
skill config grafana \
  --set GRAFANA_URL=https://grafana.example.com \
  --set GRAFANA_API_KEY=your_api_key
```

## Tools

### list-dashboards
List all dashboards with optional filtering.

**Parameters:**
- `folder` (optional, string): Filter by folder name
- `tag` (optional, string): Filter by tag
- `query` (optional, string): Search query
- `limit` (optional, number): Maximum results (default: 100)

**Example:**
```
skill run grafana list-dashboards
skill run grafana list-dashboards --folder "Production"
skill run grafana list-dashboards --query "kubernetes"
```

### get-dashboard
Get dashboard details including panels.

**Parameters:**
- `uid` (required, string): Dashboard UID

**Example:**
```
skill run grafana get-dashboard --uid abc123
```

### list-alerts
List alert rules.

**Parameters:**
- `state` (optional, string): Filter by state: firing, pending, inactive
- `folder` (optional, string): Filter by folder
- `limit` (optional, number): Maximum results (default: 100)

**Example:**
```
skill run grafana list-alerts
skill run grafana list-alerts --state firing
```

### get-alert
Get details of a specific alert rule.

**Parameters:**
- `uid` (required, string): Alert rule UID

**Example:**
```
skill run grafana get-alert --uid alert-xyz
```

### silence-alert
Create a silence for an alert.

**Parameters:**
- `matchers` (required, string): Label matchers (e.g., alertname=High CPU)
- `duration` (required, string): Silence duration (e.g., 1h, 30m, 2d)
- `comment` (optional, string): Reason for silence
- `createdBy` (optional, string): Creator name

**Example:**
```
skill run grafana silence-alert --matchers "alertname=HighCPU" --duration "1h" --comment "Maintenance"
```

### query-datasource
Query a data source directly.

**Parameters:**
- `datasource` (required, string): Data source name or UID
- `query` (required, string): Query string (PromQL, SQL, etc.)
- `from` (optional, string): Start time (default: 1h ago)
- `to` (optional, string): End time (default: now)

**Example:**
```
skill run grafana query-datasource --datasource "Prometheus" --query "rate(http_requests_total[5m])"
```

### list-datasources
List configured data sources.

**Example:**
```
skill run grafana list-datasources
```

### list-folders
List dashboard folders.

**Example:**
```
skill run grafana list-folders
```
