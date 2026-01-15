---
name: linear
version: 1.0.0
description: Linear issue tracking and project management
author: Skill Engine
---

# Linear Skill

Issue tracking and project management through the Linear GraphQL API.

## Installation

```bash
skill install ./examples/wasm-skills/linear-skill
```

## Configuration

```bash
skill config linear --set LINEAR_API_KEY=lin_api_xxxxx
```

Get your API key from Linear Settings > API > Personal API keys.

## Tools

### list-issues
List issues with filtering options.

**Parameters:**
- `team` (optional, string): Team key or ID
- `project` (optional, string): Project name or ID
- `status` (optional, string): Status: backlog, todo, in_progress, done, cancelled
- `assignee` (optional, string): Assignee email or ID
- `limit` (optional, number): Maximum results (default: 50)

**Example:**
```
skill run linear list-issues --team ENG --status in_progress
```

### get-issue
Get detailed information about a specific issue.

**Parameters:**
- `id` (required, string): Issue ID (e.g., ENG-123)

**Example:**
```
skill run linear get-issue --id ENG-123
```

### create-issue
Create a new issue.

**Parameters:**
- `title` (required, string): Issue title
- `team` (required, string): Team key
- `description` (optional, string): Issue description
- `priority` (optional, number): Priority (0=None, 1=Urgent, 2=High, 3=Medium, 4=Low)
- `assignee` (optional, string): Assignee email
- `labels` (optional, string): Comma-separated label names
- `estimate` (optional, number): Point estimate

**Example:**
```
skill run linear create-issue --title "Fix login bug" --team ENG --priority 2
```

### update-issue
Update an existing issue.

**Parameters:**
- `id` (required, string): Issue ID
- `title` (optional, string): New title
- `description` (optional, string): New description
- `status` (optional, string): New status
- `priority` (optional, number): New priority
- `estimate` (optional, number): Point estimate

**Example:**
```
skill run linear update-issue --id ENG-123 --priority 1 --status in_progress
```

### add-comment
Add a comment to an issue.

**Parameters:**
- `issue_id` (required, string): Issue ID
- `body` (required, string): Comment content (Markdown supported)

**Example:**
```
skill run linear add-comment --issue_id abc123 --body "Fixed in latest commit"
```

### list-projects
List projects in a team.

**Parameters:**
- `team` (optional, string): Team key
- `status` (optional, string): Filter: planned, started, paused, completed, cancelled

**Example:**
```
skill run linear list-projects --team ENG --status started
```

### list-cycles
List cycles (sprints) for a team.

**Parameters:**
- `team` (required, string): Team key
- `filter` (optional, string): Filter: current, upcoming, past

**Example:**
```
skill run linear list-cycles --team ENG --filter current
```

### list-teams
List all teams in the workspace.

**Parameters:**
None

**Example:**
```
skill run linear list-teams
```

## Use Cases

### Sprint Planning
```bash
# View current cycle
skill run linear list-cycles --team ENG --filter current

# List backlog items
skill run linear list-issues --team ENG --status backlog

# Create new issue for sprint
skill run linear create-issue --title "Implement feature X" --team ENG --estimate 3
```

### Daily Standup
```bash
# Check in-progress issues
skill run linear list-issues --team ENG --status in_progress

# Get details on blocked issue
skill run linear get-issue --id ENG-456

# Add status update
skill run linear add-comment --issue_id abc123 --body "Waiting on API review"
```

### Project Tracking
```bash
# List active projects
skill run linear list-projects --status started

# View project issues
skill run linear list-issues --project "Q1 Release"
```
