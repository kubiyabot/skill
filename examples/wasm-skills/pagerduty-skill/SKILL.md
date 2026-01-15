---
name: pagerduty
version: 1.0.0
description: PagerDuty incident management and on-call scheduling
author: Skill Engine
---

# PagerDuty Skill

Manage incidents and on-call schedules with PagerDuty API integration.

## Installation

```bash
skill install ./examples/wasm-skills/pagerduty-skill
```

## Configuration

```bash
skill config pagerduty --set PAGERDUTY_API_KEY=your_api_key
```

## Tools

### list-incidents
List incidents with optional filtering.

**Parameters:**
- `status` (optional, string): Filter: triggered, acknowledged, resolved
- `urgency` (optional, string): Filter: high, low
- `service` (optional, string): Filter by service ID
- `since` (optional, string): Start date (ISO 8601)
- `until` (optional, string): End date
- `limit` (optional, number): Maximum results (default: 25)

**Example:**
```
skill run pagerduty list-incidents --status triggered
skill run pagerduty list-incidents --urgency high
```

### get-incident
Get detailed information about a specific incident.

**Parameters:**
- `id` (required, string): Incident ID

**Example:**
```
skill run pagerduty get-incident --id P1234ABC
```

### create-incident
Create a new incident manually.

**Parameters:**
- `title` (required, string): Incident title
- `service` (required, string): Service ID
- `from` (required, string): Email of the user creating the incident
- `urgency` (optional, string): high or low (default: high)
- `body` (optional, string): Incident description

**Example:**
```
skill run pagerduty create-incident --title "Database connection failures" --service PXXXXX --from "user@example.com"
```

### acknowledge-incident
Acknowledge an incident.

**Parameters:**
- `id` (required, string): Incident ID
- `from` (required, string): Email of the user acknowledging

**Example:**
```
skill run pagerduty acknowledge-incident --id P1234ABC --from "user@example.com"
```

### resolve-incident
Resolve an incident.

**Parameters:**
- `id` (required, string): Incident ID
- `from` (required, string): Email of the user resolving
- `resolution` (optional, string): Resolution notes

**Example:**
```
skill run pagerduty resolve-incident --id P1234ABC --from "user@example.com" --resolution "Fixed by scaling up"
```

### add-note
Add a note to an incident timeline.

**Parameters:**
- `incident_id` (required, string): Incident ID
- `content` (required, string): Note content
- `from` (required, string): Email of the user adding the note

**Example:**
```
skill run pagerduty add-note --incident_id P1234ABC --content "Identified root cause" --from "user@example.com"
```

### list-oncalls
List current on-call assignments.

**Parameters:**
- `schedule` (optional, string): Filter by schedule ID
- `escalation_policy` (optional, string): Filter by escalation policy ID

**Example:**
```
skill run pagerduty list-oncalls
```

### list-services
List PagerDuty services.

**Parameters:**
- `query` (optional, string): Search query
- `team` (optional, string): Filter by team ID

**Example:**
```
skill run pagerduty list-services --query "production"
```
