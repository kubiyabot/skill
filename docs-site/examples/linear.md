# Linear Skill

Manage issues, projects, and workflows in Linear through the GraphQL API.

## Overview

The Linear skill provides AI agents with Linear project management capabilities. Create and update issues, manage projects, track cycles, and automate development workflows through Linear's GraphQL API.

**Runtime**: WASM (JavaScript/TypeScript)
**Source**: [examples/wasm-skills/linear-skill](https://github.com/kubiyabot/skill/tree/main/examples/wasm-skills/linear-skill)

## Installation

```bash
# Install the skill
skill install github:kubiyabot/skill:linear

# Or from local directory
skill install ./examples/wasm-skills/linear-skill
```

## Configuration

Configure your Linear API key:

```bash
skill config linear --set api_key=YOUR_API_KEY
```

Or via environment variables:

```bash
export LINEAR_API_KEY=your_api_key
```

### Getting an API Key

1. Go to [Linear Settings](https://linear.app/settings/api)
2. Click "Create Key"
3. Give it a descriptive name
4. Copy the generated key

## Tools Reference

### list-issues

List issues with filtering options.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `team` | string | No | Team key or ID |
| `project` | string | No | Project name or ID |
| `status` | string | No | Status: `backlog`, `todo`, `in_progress`, `done`, `cancelled` |
| `assignee` | string | No | Assignee email or ID |
| `label` | string | No | Label name |
| `limit` | number | No | Maximum results (default: 50) |

**Examples:**

```bash
# List all issues for a team
skill run linear list-issues --team ENG

# Filter by status
skill run linear list-issues --team ENG --status in_progress

# Filter by assignee
skill run linear list-issues --assignee alice@example.com

# Filter by project and label
skill run linear list-issues \
  --project "Q1 Launch" \
  --label bug
```

**Output:**
```json
{
  "issues": [
    {
      "id": "ENG-123",
      "title": "Implement user authentication",
      "status": "In Progress",
      "priority": 2,
      "assignee": {"name": "Alice", "email": "alice@example.com"},
      "project": "Q1 Launch",
      "labels": ["feature", "backend"],
      "estimate": 5,
      "createdAt": "2025-01-10T10:00:00.000Z"
    }
  ]
}
```

### get-issue

Get detailed information about a specific issue.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Issue ID (e.g., ENG-123) |

**Examples:**

```bash
skill run linear get-issue --id ENG-123
```

**Output:**
```json
{
  "id": "ENG-123",
  "title": "Implement user authentication",
  "description": "Add JWT-based authentication...",
  "status": "In Progress",
  "priority": 2,
  "assignee": {"name": "Alice"},
  "project": {"name": "Q1 Launch"},
  "cycle": {"name": "Sprint 5"},
  "labels": ["feature", "backend"],
  "estimate": 5,
  "comments": [
    {"body": "Started implementation", "user": "Alice", "createdAt": "..."}
  ],
  "history": [
    {"field": "status", "from": "Todo", "to": "In Progress", "actor": "Alice"}
  ]
}
```

### create-issue

Create a new issue.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | Yes | Issue title |
| `team` | string | Yes | Team key |
| `description` | string | No | Issue description (Markdown supported) |
| `status` | string | No | Initial status |
| `priority` | number | No | Priority (0=None, 1=Urgent, 2=High, 3=Medium, 4=Low) |
| `assignee` | string | No | Assignee email |
| `project` | string | No | Project name or ID |
| `labels` | string | No | Comma-separated label names |
| `estimate` | number | No | Point estimate |

**Examples:**

```bash
# Create basic issue
skill run linear create-issue \
  --title "Fix login timeout bug" \
  --team ENG

# Create detailed issue
skill run linear create-issue \
  --title "Implement OAuth2 support" \
  --team ENG \
  --description "Add Google and GitHub OAuth2 providers for user authentication" \
  --priority 2 \
  --assignee alice@example.com \
  --project "Q1 Launch" \
  --labels "feature,auth" \
  --estimate 8
```

**Output:**
```json
{
  "id": "ENG-456",
  "title": "Implement OAuth2 support",
  "url": "https://linear.app/myteam/issue/ENG-456"
}
```

### update-issue

Update an existing issue.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Issue ID |
| `title` | string | No | New title |
| `description` | string | No | New description |
| `status` | string | No | New status |
| `priority` | number | No | New priority |
| `assignee` | string | No | New assignee email |
| `labels` | string | No | Comma-separated labels |
| `estimate` | number | No | Point estimate |

**Examples:**

```bash
# Update status
skill run linear update-issue \
  --id ENG-123 \
  --status done

# Reassign and update priority
skill run linear update-issue \
  --id ENG-123 \
  --assignee bob@example.com \
  --priority 1

# Update multiple fields
skill run linear update-issue \
  --id ENG-123 \
  --title "Implement OAuth2 support (Updated)" \
  --labels "feature,auth,urgent" \
  --estimate 13
```

### add-comment

Add a comment to an issue.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `issue_id` | string | Yes | Issue ID |
| `body` | string | Yes | Comment content (Markdown supported) |

**Examples:**

```bash
skill run linear add-comment \
  --issue_id ENG-123 \
  --body "Completed the initial implementation. Ready for review."

# With Markdown
skill run linear add-comment \
  --issue_id ENG-123 \
  --body "## Progress Update\n- [x] Backend API\n- [ ] Frontend integration\n- [ ] Tests"
```

### list-projects

List projects in a team.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `team` | string | No | Team key |
| `status` | string | No | Filter: `planned`, `started`, `paused`, `completed`, `cancelled` |

**Examples:**

```bash
# List all projects
skill run linear list-projects

# Filter by team
skill run linear list-projects --team ENG

# Filter active projects
skill run linear list-projects --status started
```

**Output:**
```json
{
  "projects": [
    {
      "id": "proj_123",
      "name": "Q1 Launch",
      "status": "started",
      "progress": 0.45,
      "targetDate": "2025-03-31",
      "lead": {"name": "Alice"},
      "issueCount": 24
    }
  ]
}
```

### list-cycles

List cycles (sprints) for a team.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `team` | string | Yes | Team key |
| `filter` | string | No | Filter: `current`, `upcoming`, `past` |

**Examples:**

```bash
# List all cycles
skill run linear list-cycles --team ENG

# Get current cycle
skill run linear list-cycles --team ENG --filter current
```

**Output:**
```json
{
  "cycles": [
    {
      "id": "cycle_456",
      "name": "Sprint 5",
      "number": 5,
      "startsAt": "2025-01-13T00:00:00.000Z",
      "endsAt": "2025-01-27T00:00:00.000Z",
      "progress": 0.32,
      "issueCount": 15,
      "completedIssueCount": 5
    }
  ]
}
```

### list-teams

List all teams in the workspace.

**Examples:**

```bash
skill run linear list-teams
```

**Output:**
```json
{
  "teams": [
    {"id": "team_123", "key": "ENG", "name": "Engineering"},
    {"id": "team_456", "key": "DES", "name": "Design"},
    {"id": "team_789", "key": "OPS", "name": "Operations"}
  ]
}
```

## Common Workflows

### Bug Triage

```bash
# 1. List unassigned bugs
skill run linear list-issues \
  --team ENG \
  --label bug \
  --status backlog

# 2. Get issue details
skill run linear get-issue --id ENG-789

# 3. Assign and prioritize
skill run linear update-issue \
  --id ENG-789 \
  --assignee alice@example.com \
  --priority 2 \
  --status todo

# 4. Add context
skill run linear add-comment \
  --issue_id ENG-789 \
  --body "Reproduced. Root cause identified in auth service."
```

### Sprint Planning

```bash
# 1. Check current cycle progress
skill run linear list-cycles --team ENG --filter current

# 2. List issues in backlog
skill run linear list-issues --team ENG --status backlog

# 3. Create new issues
skill run linear create-issue \
  --title "API rate limiting" \
  --team ENG \
  --priority 3 \
  --estimate 5

# 4. Review project progress
skill run linear list-projects --team ENG --status started
```

### Daily Standup

```bash
# Check in-progress issues
skill run linear list-issues --team ENG --status in_progress

# Review completed yesterday
skill run linear list-issues --team ENG --status done --limit 10

# Check blocked items (by label)
skill run linear list-issues --team ENG --label blocked
```

## Security Considerations

- **API Key Scope**: Linear API keys have workspace-wide access
- **Read-Only Keys**: Create separate keys for read-only operations
- **Key Rotation**: Rotate API keys periodically
- **Audit Log**: Linear tracks all API actions in the audit log

## Troubleshooting

### Authentication Failed

```
Error: 401 Unauthorized
```

**Solution**: Verify your API key:

```bash
skill config linear --set api_key=YOUR_CORRECT_KEY
```

### Team Not Found

```
Error: Team not found
```

**Solution**: Use team key (e.g., "ENG") not team name. List teams to find correct key.

### Rate Limited

```
Error: 429 Too Many Requests
```

**Solution**: Linear has rate limits. Add delays between bulk operations.

### Issue Not Found

```
Error: Issue not found
```

**Solution**: Verify issue ID format (e.g., "ENG-123").

## Integration with Claude Code

```bash
# Natural language commands
"Create a bug report for the login timeout issue"
"What issues are assigned to me?"
"Update ENG-123 to in progress"
"Show me the current sprint progress"
```

## Next Steps

- [GitHub Skill](./github.md) - GitHub integration
- [Jira Skill](./jira.md) - Jira project management
- [Slack Skill](./slack.md) - Team notifications
- [Linear Documentation](https://developers.linear.app/)
