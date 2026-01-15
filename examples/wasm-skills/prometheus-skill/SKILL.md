# Prometheus Skill

Prometheus metrics querying and monitoring operations.

## Overview

This skill provides comprehensive access to the Prometheus HTTP API for querying metrics, managing alerts, and monitoring system status.

## Requirements

- **Prometheus server** accessible via HTTP
- Optional: Basic authentication credentials if enabled

## Configuration

```bash
export SKILL_PROMETHEUS_URL=http://localhost:9090
```

## Tools (15)

### Query Tools

- `query` - Execute instant PromQL query at a single point in time
- `query-range` - Execute PromQL query over a time range with step interval
- `series` - Find time series matching label selectors
- `labels` - Get all label names in the database
- `label-values` - Get all values for a specific label

### Target & Rule Tools

- `targets` - Get current scrape targets and their health status
- `rules` - Get alerting and recording rules
- `alerts` - Get currently firing alerts
- `metadata` - Get metric metadata (type, help, unit)

### Status Tools

- `status-config` - Get current Prometheus configuration
- `status-flags` - Get configured flag values
- `status-runtimeinfo` - Get runtime information (memory, goroutines, etc.)
- `status-buildinfo` - Get build information (version, revision, etc.)
- `status-tsdb` - Get TSDB storage statistics
- `check-health` - Check if Prometheus is healthy and ready

## Example Usage

### Query Current Metrics

```
Tool: query
Args: { "query": "up" }
Result: Shows which targets are up (1) or down (0)
```

### Query CPU Usage Over Time

```
Tool: query-range
Args: {
  "query": "rate(node_cpu_seconds_total{mode=\"user\"}[5m])",
  "start": "2024-01-01T00:00:00Z",
  "end": "2024-01-01T01:00:00Z",
  "step": "1m"
}
Result: CPU usage rate over the specified time range
```

### Find Series by Label

```
Tool: series
Args: { "match": "{job=\"prometheus\"}" }
Result: All time series for the prometheus job
```

### Get Label Values

```
Tool: label-values
Args: { "label": "job" }
Result: ["prometheus", "node-exporter", "alertmanager", ...]
```

### Check Firing Alerts

```
Tool: alerts
Args: {}
Result: List of currently firing alerts with labels and annotations
```

### Get Target Status

```
Tool: targets
Args: { "state": "active" }
Result: All active scrape targets with last scrape time and health
```

## PromQL Query Examples

### Basic Queries
- `up` - Target availability
- `node_memory_MemTotal_bytes` - Total memory
- `process_cpu_seconds_total` - Process CPU time

### Rate Calculations
- `rate(http_requests_total[5m])` - Request rate over 5 minutes
- `irate(http_requests_total[5m])` - Instantaneous request rate

### Aggregations
- `sum(rate(http_requests_total[5m])) by (handler)` - Requests by handler
- `avg(node_cpu_seconds_total) by (instance)` - Average CPU by instance

### Filtering
- `http_requests_total{status="500"}` - Only 500 errors
- `node_disk_io_time_seconds_total{device=~"sd.*"}` - Disk devices matching pattern

## Response Format

All query results follow the Prometheus API response format:

```json
{
  "resultType": "vector|matrix|scalar|string",
  "result": [
    {
      "metric": { "label": "value" },
      "value": [timestamp, "value"]
    }
  ]
}
```

## Error Handling

- Connection errors return descriptive messages
- Invalid PromQL queries return Prometheus error messages
- Timeout errors indicate slow queries (consider adjusting timeout parameter)

## Security Notes

- No authentication is performed by default
- Configure Prometheus with authentication if exposed publicly
- Queries can be resource-intensive; use appropriate time ranges and step intervals
