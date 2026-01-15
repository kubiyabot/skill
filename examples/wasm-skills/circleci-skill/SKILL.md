# CircleCI Skill

CircleCI CI/CD pipeline management and monitoring.

## Overview

This skill provides comprehensive access to the CircleCI API v2 for managing pipelines, workflows, jobs, and project configuration.

## Requirements

- **CircleCI API Token** with appropriate permissions
- Project must be connected to CircleCI

## Configuration

```bash
export SKILL_CIRCLECI_TOKEN=your_api_token
```

Generate a token at: https://app.circleci.com/settings/user/tokens

## Tools (18)

### User Tools

- `me` - Get current authenticated user information

### Project Tools

- `project-get` - Get project information and settings

### Pipeline Tools

- `pipeline-list` - List pipelines for a project with optional branch filter
- `pipeline-get` - Get detailed pipeline information by ID
- `pipeline-trigger` - Trigger a new pipeline on a branch or tag

### Workflow Tools

- `workflow-get` - Get workflow details by ID
- `workflow-cancel` - Cancel a running workflow
- `workflow-rerun` - Rerun a workflow (optionally from failed jobs only)
- `workflow-jobs` - List all jobs in a workflow

### Job Tools

- `job-get` - Get detailed job information
- `job-cancel` - Cancel a running job
- `job-artifacts` - List artifacts produced by a job

### Insights Tools

- `insights-summary` - Get workflow metrics summary (success rate, duration)
- `insights-jobs` - Get job-level insights for a workflow

### Configuration Tools

- `context-list` - List contexts for an organization
- `context-get` - Get context details by ID
- `env-var-list` - List environment variables for a project
- `env-var-create` - Create or update a project environment variable

## Project Slug Format

CircleCI uses project slugs in the format: `<vcs>/<org>/<repo>`

Examples:
- `gh/myorg/myrepo` - GitHub repository
- `bb/myorg/myrepo` - Bitbucket repository
- `circleci/myorg/myrepo` - CircleCI standalone project

## Example Usage

### Trigger a Pipeline

```
Tool: pipeline-trigger
Args: {
  "project_slug": "gh/myorg/myrepo",
  "branch": "main"
}
Result: Pipeline triggered with ID and number
```

### Trigger with Parameters

```
Tool: pipeline-trigger
Args: {
  "project_slug": "gh/myorg/myrepo",
  "branch": "main",
  "parameters": "{\"deploy_env\": \"staging\", \"run_tests\": true}"
}
Result: Pipeline triggered with custom parameters
```

### List Recent Pipelines

```
Tool: pipeline-list
Args: {
  "project_slug": "gh/myorg/myrepo",
  "branch": "main"
}
Result: List of recent pipelines with status and trigger info
```

### Get Workflow Status

```
Tool: workflow-get
Args: { "workflow_id": "abc-123-def-456" }
Result: Workflow status, duration, and job summary
```

### Rerun Failed Jobs

```
Tool: workflow-rerun
Args: {
  "workflow_id": "abc-123-def-456",
  "from_failed": true
}
Result: Workflow rerun triggered from failed jobs
```

### Get Job Artifacts

```
Tool: job-artifacts
Args: {
  "project_slug": "gh/myorg/myrepo",
  "job_number": 123
}
Result: List of artifacts with download URLs
```

### Get Workflow Insights

```
Tool: insights-summary
Args: {
  "project_slug": "gh/myorg/myrepo",
  "workflow_name": "build-and-test",
  "branch": "main"
}
Result: Success rate, mean duration, throughput metrics
```

### Manage Environment Variables

```
Tool: env-var-create
Args: {
  "project_slug": "gh/myorg/myrepo",
  "name": "API_KEY",
  "value": "secret-value"
}
Result: Environment variable created/updated
```

## Response Format

Successful responses return JSON with CircleCI API data:

```json
{
  "id": "pipeline-id",
  "state": "running",
  "number": 123,
  "created_at": "2024-01-01T00:00:00Z",
  ...
}
```

## Error Handling

- **401**: Authentication failed - check your CIRCLECI_TOKEN
- **404**: Resource not found - verify project slug and IDs
- **429**: Rate limited - reduce request frequency

## Security Notes

- API tokens should have minimal required permissions
- Environment variable values are write-only (cannot be read back)
- Context access requires organization-level permissions
- Audit logs track all API operations
