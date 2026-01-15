---
name: github-skill
version: 2.0.0
description: GitHub integration for repositories, issues, pull requests, branches, releases, and actions
author: Skill Engine Team
allowed-tools: ["repo-list", "repo-get", "repo-create", "repo-delete", "repo-fork", "issue-list", "issue-get", "issue-create", "issue-update", "issue-close", "issue-comment", "pr-list", "pr-get", "pr-create", "pr-merge", "pr-close", "pr-review", "pr-request-reviewers", "branch-list", "branch-create", "branch-delete", "release-list", "release-create", "workflow-list", "workflow-run"]
---

# GitHub Skill

Comprehensive GitHub integration with 25 tools for repositories, issues, pull requests, branches, releases, and GitHub Actions.

## Overview

This skill provides full access to the GitHub API, enabling automation of development workflows. It uses the GitHub REST API v3 with Bearer token authentication.

## When to Use This Skill

**Use this skill when you need to**:
- Create, manage, and fork repositories
- List, create, update, close issues and add comments
- Work with pull requests (create, merge, review, request reviewers)
- Manage branches (create, delete, list)
- Create and list releases
- Trigger GitHub Actions workflows

## Prerequisites

1. **GitHub Account**: An active GitHub account
2. **Personal Access Token**: A token with appropriate permissions

### Getting a Personal Access Token

1. Go to https://github.com/settings/tokens
2. Click "Generate new token" > "Generate new token (classic)"
3. Select scopes:
   - `repo` - Full repository access
   - `workflow` - Actions workflow access
   - `read:org` - Read organization info
4. Generate and copy the token

## Configuration

Set the token as an environment variable:

```bash
export SKILL_GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

## Tools

### Repository Tools (5)

#### repo-list
List repositories for user or organization.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| org | string | No | - | Organization name |
| type | string | No | all | Filter: all, owner, member, private, public |
| sort | string | No | updated | Sort: created, updated, pushed, full_name |
| per_page | number | No | 30 | Results per page (max 100) |

#### repo-get
Get detailed repository information.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |

#### repo-create
Create a new repository.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| name | string | Yes | - | Repository name |
| description | string | No | - | Repository description |
| private | boolean | No | false | Make private |
| org | string | No | - | Organization to create in |
| auto_init | boolean | No | false | Initialize with README |

#### repo-delete
Delete a repository (requires admin access).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |

#### repo-fork
Fork a repository.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository to fork |
| organization | string | No | Org to fork to |
| name | string | No | Name for forked repo |

### Issue Tools (6)

#### issue-list
List issues in a repository.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| repo | string | Yes | - | Repository in "owner/repo" format |
| state | string | No | open | Filter: open, closed, all |
| assignee | string | No | - | Filter by assignee |
| labels | string | No | - | Filter by labels (comma-separated) |
| per_page | number | No | 30 | Results per page |

#### issue-get
Get detailed issue information.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |
| issue_number | number | Yes | Issue number |

#### issue-create
Create a new issue.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |
| title | string | Yes | Issue title |
| body | string | No | Issue description |
| labels | string | No | Labels (comma-separated) |
| assignees | string | No | Assignees (comma-separated) |

#### issue-update
Update an existing issue.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |
| issue_number | number | Yes | Issue number |
| title | string | No | New title |
| body | string | No | New body |
| state | string | No | State: open or closed |
| labels | string | No | Labels (replaces existing) |
| assignees | string | No | Assignees (replaces existing) |

#### issue-close
Close an issue.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| repo | string | Yes | - | Repository in "owner/repo" format |
| issue_number | number | Yes | - | Issue number |
| reason | string | No | completed | Reason: completed or not_planned |

#### issue-comment
Add a comment to an issue.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |
| issue_number | number | Yes | Issue number |
| body | string | Yes | Comment text |

### Pull Request Tools (7)

#### pr-list
List pull requests.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| repo | string | Yes | - | Repository in "owner/repo" format |
| state | string | No | open | Filter: open, closed, all |
| head | string | No | - | Filter by head branch |
| base | string | No | - | Filter by base branch |
| per_page | number | No | 30 | Results per page |

#### pr-get
Get detailed PR information.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |
| pr_number | number | Yes | PR number |

#### pr-create
Create a new pull request.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| repo | string | Yes | - | Repository in "owner/repo" format |
| title | string | Yes | - | PR title |
| head | string | Yes | - | Branch with changes |
| base | string | Yes | - | Branch to merge into |
| body | string | No | - | PR description |
| draft | boolean | No | false | Create as draft |

#### pr-merge
Merge a pull request.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| repo | string | Yes | - | Repository in "owner/repo" format |
| pr_number | number | Yes | - | PR number |
| merge_method | string | No | merge | Method: merge, squash, rebase |
| commit_title | string | No | - | Commit title |
| commit_message | string | No | - | Commit message |

#### pr-close
Close a PR without merging.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |
| pr_number | number | Yes | PR number |

#### pr-review
Submit a PR review.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |
| pr_number | number | Yes | PR number |
| event | string | Yes | Action: APPROVE, REQUEST_CHANGES, COMMENT |
| body | string | No | Review comment |

#### pr-request-reviewers
Request reviewers for a PR.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |
| pr_number | number | Yes | PR number |
| reviewers | string | No | Usernames (comma-separated) |
| team_reviewers | string | No | Team slugs (comma-separated) |

### Branch Tools (3)

#### branch-list
List branches.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| repo | string | Yes | - | Repository in "owner/repo" format |
| protected | boolean | No | false | Only show protected branches |
| per_page | number | No | 30 | Results per page |

#### branch-create
Create a new branch.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |
| branch | string | Yes | New branch name |
| from | string | No | Source branch or SHA (defaults to default branch) |

#### branch-delete
Delete a branch.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |
| branch | string | Yes | Branch name to delete |

### Release Tools (2)

#### release-list
List releases.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| repo | string | Yes | - | Repository in "owner/repo" format |
| per_page | number | No | 30 | Results per page |

#### release-create
Create a new release.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| repo | string | Yes | - | Repository in "owner/repo" format |
| tag_name | string | Yes | - | Tag name for release |
| name | string | No | - | Release title |
| body | string | No | - | Release notes |
| target_commitish | string | No | - | Branch or commit to tag |
| draft | boolean | No | false | Create as draft |
| prerelease | boolean | No | false | Mark as pre-release |
| generate_release_notes | boolean | No | false | Auto-generate notes |

### Actions/Workflow Tools (2)

#### workflow-list
List workflows.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| repo | string | Yes | - | Repository in "owner/repo" format |
| per_page | number | No | 30 | Results per page |

#### workflow-run
Trigger a workflow run.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| repo | string | Yes | Repository in "owner/repo" format |
| workflow_id | string | Yes | Workflow ID or filename (e.g., "ci.yml") |
| ref | string | Yes | Branch or tag to run on |
| inputs | string | No | Workflow inputs as JSON |

## Examples

### Create Repository and Initialize

```bash
# Create repo
skill run github-skill repo-create name=my-project description="New project" private=true

# Create initial issue
skill run github-skill issue-create repo=myuser/my-project title="Setup project" labels=setup
```

### PR Workflow

```bash
# Create feature branch
skill run github-skill branch-create repo=owner/repo branch=feature/auth from=main

# After pushing commits, create PR
skill run github-skill pr-create repo=owner/repo title="Add auth" head=feature/auth base=main

# Request reviewers
skill run github-skill pr-request-reviewers repo=owner/repo pr_number=42 reviewers=reviewer1,reviewer2

# Merge PR
skill run github-skill pr-merge repo=owner/repo pr_number=42 merge_method=squash
```

### Release Workflow

```bash
# Create release
skill run github-skill release-create repo=owner/repo tag_name=v1.0.0 name="Version 1.0.0" generate_release_notes=true
```

## Rate Limits

- **Authenticated**: 5,000 requests/hour
- **Search API**: 30 requests/minute

## Troubleshooting

| Error | Solution |
|-------|----------|
| 401 Unauthorized | Check GITHUB_TOKEN is set correctly |
| 403 Forbidden | Token lacks permissions or rate limited |
| 404 Not Found | Check repo exists and you have access |
| 422 Validation | Invalid parameters (e.g., branch exists) |

## Resources

- [GitHub REST API](https://docs.github.com/en/rest)
- [Personal Access Tokens](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)
- [GitHub CLI](https://cli.github.com/)
