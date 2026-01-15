---
name: github-oauth-skill
version: 2.0.0
description: GitHub integration with OAuth2 Device Flow authentication for repositories, issues, pull requests, and more
author: Skill Engine Team
allowed-tools: ["whoami", "list-repos", "get-repo", "list-issues", "create-issue", "list-prs", "create-pr"]
---

# GitHub OAuth Skill

A comprehensive GitHub integration demonstrating OAuth2 Device Flow authentication with the Skill Engine SDK. Provides secure, token-based access to GitHub repositories, issues, and pull requests without exposing credentials in configuration files.

## Overview

This skill showcases modern OAuth2 authentication patterns for CLI tools, using GitHub's Device Flow (RFC 8628) which is ideal for command-line applications. All tokens are stored securely in your system keychain.

## When to Use This Skill

**Use this skill when you need to**:
- Authenticate with GitHub without manually creating personal access tokens
- List and search your repositories or organization repos
- Manage issues (create, list, filter by labels/state)
- Work with pull requests (create, list, filter)
- Get information about your authenticated user
- Store credentials securely in system keychain

## Prerequisites

1. **GitHub Account**: An active GitHub account with repository access
2. **System Keychain**: macOS Keychain, Windows Credential Manager, or Linux Secret Service

### OAuth2 Device Flow Setup

The Device Flow allows you to authenticate without a local HTTP server:

1. Run the login command
2. A browser window opens with GitHub authorization page
3. Enter the provided device code
4. Authorize the application
5. Token is automatically stored in your system keychain

**Required Scopes**:
- `repo` - Full control of private repositories
- `read:user` - Read user profile data

## Authentication

### Initial Setup

```bash
# Authenticate with GitHub (opens browser)
skill auth login github --skill github-oauth-skill

# This will:
# 1. Display a URL and device code
# 2. Open GitHub authorization page in your browser
# 3. Prompt you to enter the code
# 4. Store the token securely in system keychain
```

### Check Authentication Status

```bash
# View current auth status
skill auth status

# Test the connection
skill run github-oauth-skill whoami

# Log out (revokes token)
skill auth logout github
```

## Tools

### User Tools

#### whoami
Get information about the authenticated GitHub user.

**Parameters**: None

**Returns**: User login, name, email, public repos count, followers, following

**Example**:
```bash
skill run github-oauth-skill whoami
```

### Repository Tools

#### list-repos
List repositories for the authenticated user or an organization.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| org | string | No | - | Organization name to list repos from |
| type | string | No | all | Filter: all, owner, member, private, public |
| sort | string | No | updated | Sort by: created, updated, pushed, full_name |
| per_page | number | No | 30 | Results per page (max 100) |

**Example**:
```bash
# List your repositories
skill run github-oauth-skill list-repos

# List organization repos
skill run github-oauth-skill list-repos org=microsoft sort=created per_page=50

# Only show public repos
skill run github-oauth-skill list-repos type=public
```

#### get-repo
Get detailed information about a specific repository.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |

**Example**:
```bash
skill run github-oauth-skill get-repo repo=torvalds/linux
skill run github-oauth-skill get-repo repo=facebook/react
```

### Issue Tools

#### list-issues
List issues in a repository with filtering options.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| repo | string | Yes | - | Repository in "owner/repo" format |
| state | string | No | open | Filter: open, closed, all |
| assignee | string | No | - | Filter by assignee username |
| labels | string | No | - | Filter by labels (comma-separated) |
| per_page | number | No | 30 | Results per page (max 100) |

**Example**:
```bash
# List open issues
skill run github-oauth-skill list-issues repo=owner/repo

# List all closed issues
skill run github-oauth-skill list-issues repo=owner/repo state=closed

# Filter by labels
skill run github-oauth-skill list-issues repo=owner/repo labels=bug,high-priority

# Filter by assignee
skill run github-oauth-skill list-issues repo=owner/repo assignee=username
```

#### create-issue
Create a new issue in a repository.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |
| title | string | Yes | Issue title |
| body | string | No | Issue description (supports markdown) |
| labels | string | No | Labels (comma-separated) |
| assignees | string | No | Assignees (comma-separated usernames) |

**Example**:
```bash
# Basic issue
skill run github-oauth-skill create-issue \
  repo=owner/repo \
  title="Bug: Login fails on mobile"

# Issue with description and labels
skill run github-oauth-skill create-issue \
  repo=owner/repo \
  title="feat: Add dark mode" \
  body="## Description\nUsers have requested dark mode support.\n\n## Requirements\n- [ ] Dark theme\n- [ ] Theme switcher" \
  labels=enhancement,ui

# Assign to multiple people
skill run github-oauth-skill create-issue \
  repo=owner/repo \
  title="Critical: Data loss bug" \
  body="Users reporting data loss in version 2.1.0" \
  labels=bug,critical \
  assignees=dev1,dev2
```

### Pull Request Tools

#### list-prs
List pull requests in a repository.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| repo | string | Yes | - | Repository in "owner/repo" format |
| state | string | No | open | Filter: open, closed, all |
| head | string | No | - | Filter by head branch |
| base | string | No | - | Filter by base branch |
| per_page | number | No | 30 | Results per page (max 100) |

**Example**:
```bash
# List open PRs
skill run github-oauth-skill list-prs repo=owner/repo

# List all PRs (including closed)
skill run github-oauth-skill list-prs repo=owner/repo state=all

# Filter by branch
skill run github-oauth-skill list-prs repo=owner/repo head=feature/auth base=main
```

#### create-pr
Create a new pull request.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| repo | string | Yes | - | Repository in "owner/repo" format |
| title | string | Yes | - | PR title |
| head | string | Yes | - | Branch with changes |
| base | string | Yes | - | Branch to merge into |
| body | string | No | - | PR description (supports markdown) |
| draft | boolean | No | false | Create as draft PR |

**Example**:
```bash
# Basic PR
skill run github-oauth-skill create-pr \
  repo=owner/repo \
  title="feat: Add user authentication" \
  head=feature/auth \
  base=main

# PR with description and draft mode
skill run github-oauth-skill create-pr \
  repo=owner/repo \
  title="WIP: Refactor database layer" \
  head=refactor/db \
  base=develop \
  body="## Changes\n- Extract query builders\n- Add connection pooling\n- Optimize migrations\n\n## TODO\n- [ ] Add tests\n- [ ] Update docs" \
  draft=true

# Cross-repository PR
skill run github-oauth-skill create-pr \
  repo=upstream/repo \
  title="fix: Correct typo in README" \
  head=myuser:patch-1 \
  base=main \
  body="Fixes typo in installation instructions."
```

## Workflows

### Repository Discovery Workflow

```bash
# Find interesting repos
skill run github-oauth-skill list-repos org=microsoft sort=updated per_page=10

# Get details on specific repo
skill run github-oauth-skill get-repo repo=microsoft/vscode

# Check open issues
skill run github-oauth-skill list-issues repo=microsoft/vscode state=open labels=bug
```

### Issue Management Workflow

```bash
# Create a new issue
skill run github-oauth-skill create-issue \
  repo=myorg/myrepo \
  title="Bug: API timeout" \
  body="API endpoint /users times out after 30s" \
  labels=bug,api

# List issues assigned to you
skill run github-oauth-skill list-issues repo=myorg/myrepo assignee=myusername

# List high-priority bugs
skill run github-oauth-skill list-issues repo=myorg/myrepo labels=bug,high-priority
```

### Pull Request Workflow

```bash
# After pushing commits to feature branch, create PR
skill run github-oauth-skill create-pr \
  repo=myorg/myrepo \
  title="feat: Implement caching layer" \
  head=feature/caching \
  base=main \
  body="## Summary\nAdds Redis caching for API responses\n\n## Performance\n- 80% reduction in database queries\n- 200ms average response time improvement"

# List your open PRs
skill run github-oauth-skill list-prs repo=myorg/myrepo state=open

# Check all recent PR activity
skill run github-oauth-skill list-prs repo=myorg/myrepo state=all per_page=20
```

## Configuration

No manual configuration needed! Authentication is handled through OAuth2 Device Flow.

The skill automatically:
- Stores tokens securely in system keychain
- Refreshes tokens when needed (if supported by GitHub)
- Uses environment variables as fallback: `GITHUB_TOKEN`

### Manual Token (Optional)

If you prefer to use a personal access token:

```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

## SDK Features Demonstrated

This skill showcases advanced SDK capabilities:

### 1. OAuth2 Device Flow
```typescript
import { defineSkill, createAuthenticatedClient } from '@skill-engine/sdk';
```

### 2. Type-Safe HTTP Client
```typescript
const client = createAuthenticatedClient({
  baseUrl: 'https://api.github.com',
  authType: 'bearer',
  tokenKey: 'GITHUB_TOKEN',
});
```

### 3. Parameter Validation
```typescript
{
  name: 'repo',
  paramType: 'string',
  validation: {
    pattern: '^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$',
  },
}
```

### 4. Structured Error Handling
```typescript
if (response.status === 401) {
  return err('Auth failed', errors.auth());
}
```

## Security Notes

- **Token Storage**: Tokens are stored in system keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- **Automatic Refresh**: Tokens are refreshed automatically when supported
- **No Config Files**: Credentials never written to configuration files
- **Scope Limitation**: Only requests minimal required scopes (repo, read:user)
- **Revocation**: Use `skill auth logout github` to revoke and remove tokens

## Rate Limits

GitHub API rate limits:
- **Authenticated**: 5,000 requests/hour
- **Search API**: 30 requests/minute
- **GraphQL API**: 5,000 points/hour

Rate limit info is included in API responses. The skill automatically handles rate limit errors.

## Troubleshooting

| Error | Cause | Solution |
|-------|-------|----------|
| 401 Unauthorized | Invalid or expired token | Run `skill auth login github` again |
| 403 Forbidden | Rate limit exceeded or insufficient permissions | Wait for rate limit reset or check token scopes |
| 404 Not Found | Repository doesn't exist or no access | Verify repo name and access permissions |
| 422 Validation Failed | Invalid parameters | Check parameter format (e.g., repo must be "owner/name") |

### Common Issues

**Browser doesn't open during auth**:
```bash
# Manually open the URL displayed in terminal
# Then enter the device code shown
```

**Token not found**:
```bash
# Re-authenticate
skill auth login github --skill github-oauth-skill
```

**Permission denied errors**:
```bash
# Check token scopes - may need to re-authenticate with broader scopes
skill auth logout github
skill auth login github --skill github-oauth-skill
```

## Resources

- [GitHub REST API Documentation](https://docs.github.com/en/rest)
- [OAuth2 Device Flow (RFC 8628)](https://datatracker.ietf.org/doc/html/rfc8628)
- [GitHub Personal Access Tokens](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)
- [Skill Engine SDK Documentation](https://skill-engine.dev/docs/sdk)

## Contributing

This skill serves as a reference implementation for OAuth2 in Skill Engine. Contributions welcome:

- Additional GitHub API endpoints
- Improved error handling
- Support for GitHub Enterprise
- GraphQL API integration
