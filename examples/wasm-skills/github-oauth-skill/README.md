# GitHub OAuth Skill

A comprehensive GitHub integration demonstrating OAuth2 Device Flow authentication with the Skill Engine SDK.

## Features

- **OAuth2 Device Flow**: Secure authentication without exposing tokens in config files
- **Repository Management**: List, search, and inspect repositories
- **Issue Tracking**: List and create issues with labels
- **Pull Requests**: List and create PRs with draft support
- **Type-Safe**: Full TypeScript with parameter validation

## Setup

### 1. Authenticate with GitHub

```bash
skill auth login github --skill github-oauth-skill
```

This will:
1. Display a URL and code
2. Open GitHub in your browser (or copy the URL)
3. Enter the code to authorize
4. Store the token securely in your system keychain

### 2. Build the Skill

```bash
cd examples/github-oauth-skill
npm install
npm run build
```

### 3. Run Tools

```bash
# See who you're logged in as
skill run github-oauth-skill whoami

# List your repositories
skill run github-oauth-skill list-repos

# List repos for an organization
skill run github-oauth-skill list-repos --org microsoft

# Get repo details
skill run github-oauth-skill get-repo --repo owner/repo

# List open issues
skill run github-oauth-skill list-issues --repo owner/repo

# Create an issue
skill run github-oauth-skill create-issue \
  --repo owner/repo \
  --title "Bug: Something broke" \
  --body "## Description\nDetails here..." \
  --labels bug,high-priority

# List pull requests
skill run github-oauth-skill list-prs --repo owner/repo --state all

# Create a pull request
skill run github-oauth-skill create-pr \
  --repo owner/repo \
  --title "feat: Add new feature" \
  --head feature-branch \
  --base main \
  --body "## Changes\n- Added X\n- Fixed Y"
```

## Available Tools

| Tool | Description |
|------|-------------|
| `whoami` | Get authenticated user info |
| `list-repos` | List repositories |
| `get-repo` | Get repository details |
| `list-issues` | List repository issues |
| `create-issue` | Create a new issue |
| `list-prs` | List pull requests |
| `create-pr` | Create a pull request |

## Authentication

This skill uses GitHub OAuth2 Device Flow (RFC 8628), which is ideal for CLI applications because:

- No need to manually create/copy tokens
- No local HTTP server required
- Tokens are stored securely in system keychain
- Automatic token refresh (when supported)

### Required Scopes

The default scopes requested are:
- `repo` - Full control of private repositories
- `read:user` - Read user profile data

### Managing Authentication

```bash
# Check authentication status
skill auth status

# Log out (revoke token)
skill auth logout github
```

## SDK Features Demonstrated

This example showcases:

1. **Enhanced SDK Imports**:
   ```typescript
   import {
     defineSkill,
     ok, err, errors,
     createAuthenticatedClient,
   } from '@skill-engine/sdk';
   ```

2. **Type-Safe HTTP Client**:
   ```typescript
   const client = createAuthenticatedClient({
     baseUrl: 'https://api.github.com',
     authType: 'bearer',
     tokenKey: 'GITHUB_TOKEN',
   });
   ```

3. **Parameter Validation**:
   ```typescript
   {
     name: 'repo',
     paramType: 'string',
     validation: {
       pattern: '^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$',
     },
   }
   ```

4. **Structured Error Handling**:
   ```typescript
   if (response.status === 401) {
     return err('Auth failed', errors.auth());
   }
   ```

## Development

```bash
# Type check
npx tsc --noEmit

# Build
npm run build

# Build WASM component
npm run build:wasm
```
