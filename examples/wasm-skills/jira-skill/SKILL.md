# Jira Skill

Jira project management and issue tracking operations.

## Overview

This skill provides comprehensive access to the Jira REST API and Agile API for managing issues, projects, sprints, and boards.

## Requirements

- **Jira Cloud** or **Jira Data Center** instance
- API token (Cloud) or personal access token (Data Center)
- User email address for authentication

## Configuration

```bash
export SKILL_JIRA_URL=https://your-domain.atlassian.net
export SKILL_JIRA_EMAIL=your-email@example.com
export SKILL_JIRA_TOKEN=your_api_token
```

Generate a token at: https://id.atlassian.com/manage-profile/security/api-tokens

## Tools (25)

### User Tools

- `myself` - Get current authenticated user information
- `user-search` - Search for users by name or email

### Issue Search

- `search` - Search issues using JQL (Jira Query Language)

### Issue CRUD

- `issue-get` - Get detailed issue information
- `issue-create` - Create a new issue
- `issue-update` - Update an existing issue
- `issue-delete` - Delete an issue
- `issue-assign` - Assign or unassign an issue

### Issue Workflow

- `transitions` - Get available transitions for an issue
- `issue-transition` - Move issue to a new status

### Comments

- `comment-list` - List comments on an issue
- `comment-add` - Add a comment to an issue
- `comment-update` - Update an existing comment

### Work Logs

- `worklog-list` - List work logs on an issue
- `worklog-add` - Log time spent on an issue

### Projects

- `project-list` - List all accessible projects
- `project-get` - Get project details

### Agile - Boards

- `board-list` - List agile boards
- `board-get` - Get board details

### Agile - Sprints

- `sprint-list` - List sprints for a board
- `sprint-get` - Get sprint details

### Metadata

- `priorities` - Get available priority levels
- `issue-types` - Get available issue types (optionally per project)

## JQL Query Examples

### Basic Queries
- `project = PROJ` - Issues in project PROJ
- `assignee = currentUser()` - My assigned issues
- `status = "In Progress"` - Issues in progress

### Combined Queries
- `project = PROJ AND status != Done ORDER BY priority DESC`
- `assignee = currentUser() AND sprint in openSprints()`
- `created >= -7d AND type = Bug`

### Text Search
- `text ~ "search term"` - Full text search
- `summary ~ "bug"` - Summary contains "bug"

## Example Usage

### Search Issues

```
Tool: search
Args: {
  "jql": "project = PROJ AND status = 'In Progress'",
  "fields": "summary,status,assignee",
  "maxResults": 20
}
Result: List of matching issues with specified fields
```

### Create an Issue

```
Tool: issue-create
Args: {
  "project": "PROJ",
  "summary": "Implement new feature",
  "issue_type": "Story",
  "description": "As a user, I want to...",
  "priority": "High",
  "labels": "feature,q1"
}
Result: Created issue key (e.g., PROJ-123)
```

### Update an Issue

```
Tool: issue-update
Args: {
  "key": "PROJ-123",
  "summary": "Updated summary",
  "priority": "Critical"
}
Result: Issue updated successfully
```

### Transition an Issue

```
Tool: transitions
Args: { "key": "PROJ-123" }
Result: Available transitions (e.g., Start Progress, Done)

Tool: issue-transition
Args: {
  "key": "PROJ-123",
  "transition_id": "21"
}
Result: Issue moved to new status
```

### Add a Comment

```
Tool: comment-add
Args: {
  "key": "PROJ-123",
  "body": "Working on this now, ETA is end of day."
}
Result: Comment added successfully
```

### Log Work

```
Tool: worklog-add
Args: {
  "key": "PROJ-123",
  "time_spent": "2h 30m",
  "comment": "Implemented core functionality"
}
Result: Work log added
```

### Get Sprint Information

```
Tool: board-list
Args: { "project": "PROJ", "type": "scrum" }
Result: List of scrum boards

Tool: sprint-list
Args: { "board_id": 1, "state": "active" }
Result: Active sprints for the board
```

## Response Format

Successful responses return Jira API JSON:

```json
{
  "id": "10001",
  "key": "PROJ-123",
  "fields": {
    "summary": "Issue title",
    "status": { "name": "In Progress" },
    "assignee": { "displayName": "John Doe" }
  }
}
```

## Error Handling

- **401**: Authentication failed - check email and token
- **403**: Permission denied - verify project access
- **404**: Issue/resource not found
- **400**: Invalid JQL or request format

## Description Format

Jira Cloud uses Atlassian Document Format (ADF) for descriptions.
This skill automatically converts plain text to ADF format.

## Security Notes

- API tokens have full account permissions
- Use project-specific tokens when possible
- Never share tokens or commit them to version control
- Audit logs track all API operations
