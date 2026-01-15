# GitHub Skill

A comprehensive GitHub integration skill for Skill Engine, providing access to repositories, issues, and pull requests through the GitHub API.

## Quick Start

```bash
# Run directly from directory (no build needed!)
skill run ./examples/github-skill repo-list

# List issues in a repository
skill run ./examples/github-skill issue-list repo=owner/repo

# Create a new issue
skill run ./examples/github-skill issue-create \
  repo=owner/repo \
  title="Bug report" \
  body="Description of the bug"

# List pull requests
skill run ./examples/github-skill pr-list repo=owner/repo

# Create a pull request
skill run ./examples/github-skill pr-create \
  repo=owner/repo \
  title="New feature" \
  head=feature-branch \
  base=main
```

## Features

- **Zero Configuration**: Just write JavaScript and run
- **Auto-Compilation**: Runtime compiles to WASM on first use (~3 seconds)
- **Cached Execution**: Subsequent runs use cached WASM (<100ms startup)
- **Secure Tokens**: GitHub tokens stored in system keychain
- **Multi-Account**: Support for multiple GitHub accounts via instances

## Available Tools

### Repository Operations
- `repo-list` - List repositories for user or organization
- `repo-create` - Create new repositories

### Issue Operations
- `issue-list` - List and filter issues
- `issue-create` - Create new issues with labels and assignees

### Pull Request Operations
- `pr-list` - List and filter pull requests
- `pr-create` - Create new pull requests

## Configuration

### Method 1: Config File

Create `skill.config.toml`:

```toml
[config]
github_token = "ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
github_username = "your-username"
default_org = "your-org"  # Optional
```

### Method 2: Interactive Wizard

```bash
# Install skill first
skill install ./examples/github-skill --instance personal

# Configure interactively
skill config github-skill --instance personal
# Follow prompts to enter your token
```

### Method 3: Environment Variables

```bash
export SKILL_GITHUB_TOKEN="ghp_..."
export SKILL_GITHUB_USERNAME="your-username"

skill run ./examples/github-skill repo-list
```

## Getting a GitHub Token

1. Go to https://github.com/settings/tokens
2. Click "Generate new token" → "Generate new token (classic)"
3. Give it a name (e.g., "Skill Engine CLI")
4. Select scopes:
   - ✅ `repo` - Full repository access
   - ✅ `read:org` - Read organization data
5. Click "Generate token"
6. **Copy the token immediately** - you won't see it again!

## Multi-Account Usage

Support for multiple GitHub accounts (personal, work, etc.):

```bash
# Personal account
skill install ./examples/github-skill --instance personal
skill config github-skill --instance personal

# Work account
skill install ./examples/github-skill --instance work
skill config github-skill --instance work

# Use specific account
skill run github-skill --instance personal repo-list
skill run github-skill --instance work issue-list repo=company/project
```

## Examples

### Repository Management

```bash
# List your repositories
skill run ./examples/github-skill repo-list

# List organization repositories
skill run ./examples/github-skill repo-list org=microsoft

# Filter by type and sort
skill run ./examples/github-skill repo-list type=private sort=updated

# Create a new repository
skill run ./examples/github-skill repo-create \
  name=my-new-project \
  description="A cool new project" \
  private=false
```

### Issue Management

```bash
# List open issues
skill run ./examples/github-skill issue-list repo=owner/repo

# Filter by state
skill run ./examples/github-skill issue-list repo=owner/repo state=closed

# Filter by labels
skill run ./examples/github-skill issue-list \
  repo=owner/repo \
  labels=bug,priority-high

# Create a new issue
skill run ./examples/github-skill issue-create \
  repo=owner/repo \
  title="Login bug on mobile" \
  body="Users cannot log in from mobile devices" \
  labels=bug,mobile \
  assignees=developer1,developer2
```

### Pull Request Management

```bash
# List open pull requests
skill run ./examples/github-skill pr-list repo=owner/repo

# Filter by branch
skill run ./examples/github-skill pr-list \
  repo=owner/repo \
  head=feature-branch

# Create a pull request
skill run ./examples/github-skill pr-create \
  repo=owner/repo \
  title="Add new authentication" \
  head=feature/auth \
  base=main \
  body="This PR implements JWT-based authentication"

# Create a draft pull request
skill run ./examples/github-skill pr-create \
  repo=owner/repo \
  title="Work in progress" \
  head=wip-branch \
  base=main \
  draft=true
```

## Security

### Token Security
- Tokens are encrypted in your system keychain
- Never log or print tokens to console
- Use tokens with minimal required permissions
- Rotate tokens regularly (every 90 days recommended)
- Revoke tokens you're no longer using

### Required Token Permissions

**Minimal scopes**:
- `repo` - Full repository access (for private repos)
- `public_repo` - Public repository access only (if you only work with public repos)
- `read:org` - Read organization data

**Fine-grained tokens** (recommended):
- Select specific repositories
- Choose exact permissions needed
- Set expiration dates

### Rate Limiting

GitHub API rate limits:
- **Authenticated**: 5,000 requests/hour
- **Unauthenticated**: 60 requests/hour

This skill automatically authenticates all requests for the higher limit.

## Repository Format

Always use `owner/repo` format:
- ✅ `microsoft/vscode`
- ✅ `facebook/react`
- ✅ `your-username/your-repo`
- ❌ `vscode` (missing owner)
- ❌ `https://github.com/microsoft/vscode` (use owner/repo only)

## Development

This skill is written in pure JavaScript and can be modified directly:

```bash
# Edit the skill
vim examples/github-skill/skill.js

# Run immediately - automatically recompiles if changed
skill run ./examples/github-skill repo-list
```

### Adding New GitHub Operations

To add support for additional GitHub API endpoints:

1. Add tool definition to `getTools()`
2. Implement handler function (e.g., `handleNewOperation`)
3. Add case to switch statement in `executeTool()`
4. Update SKILL.md with documentation

### Real GitHub API Implementation

This example uses simulated responses for demonstration. To connect to real GitHub:

1. Uncomment the Octokit import at the top of `skill.js`
2. Ensure Octokit is available during compilation:
   ```bash
   npm install @octokit/rest
   ```
3. Replace simulated responses with real API calls

The skill structure is already set up for real GitHub integration.

## Workflow Examples

### Bug Triage

```bash
# List all open bugs
skill run ./examples/github-skill issue-list \
  repo=owner/repo \
  state=open \
  labels=bug

# Create new bug report
skill run ./examples/github-skill issue-create \
  repo=owner/repo \
  title="App crashes on startup" \
  body="Steps to reproduce: ..." \
  labels=bug,priority-high \
  assignees=dev-lead
```

### Feature Development

```bash
# Create feature branch
git checkout -b feature/new-feature

# Make changes and commit
git add .
git commit -m "Implement new feature"
git push -u origin feature/new-feature

# Create PR with skill
skill run ./examples/github-skill pr-create \
  repo=owner/repo \
  title="Add new feature" \
  head=feature/new-feature \
  base=main \
  body="Implements feature requested in #123"
```

### Project Initialization

```bash
# Create repository
skill run ./examples/github-skill repo-create \
  name=my-project \
  description="My awesome project" \
  private=false

# Clone locally
git clone git@github.com:username/my-project.git

# Create initial issue
skill run ./examples/github-skill issue-create \
  repo=username/my-project \
  title="Project setup" \
  body="Initialize project structure" \
  labels=setup
```

## Troubleshooting

### "Bad credentials" error
- Check your GitHub token is correct
- Verify token hasn't expired
- Ensure token has required permissions

### "Not Found" error
- Verify repository exists: `owner/repo` format
- Check you have access to the repository
- For private repos, ensure token has `repo` scope

### "Resource not accessible" error
- Token lacks required permissions
- Generate new token with appropriate scopes
- For org resources, check organization permissions

### Rate limit errors
- You've exceeded 5,000 requests/hour
- Wait for reset time (shown in error)
- Consider caching responses

## Documentation

See [SKILL.md](./SKILL.md) for comprehensive documentation including:
- What is GitHub and when to use this skill
- Detailed tool reference with all parameters
- Security best practices and token management
- GitHub API concepts and patterns
- Troubleshooting guide
- Links to GitHub documentation

## Resources

- [GitHub REST API](https://docs.github.com/en/rest)
- [Personal Access Tokens](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)
- [Octokit.js](https://github.com/octokit/rest.js) - Official GitHub API client
- [GitHub CLI](https://cli.github.com/) - Official GitHub CLI

## License

MIT License - Part of Skill Engine project
