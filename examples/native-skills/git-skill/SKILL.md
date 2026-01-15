# Git Skill

Git version control operations with native git CLI integration.

## Overview

This skill provides comprehensive Git operations through the native git CLI. It wraps git commands and returns them for host execution, ensuring security while providing full git functionality.

## Requirements

- **git** must be installed and in PATH
- Appropriate repository access and credentials configured

## Tools

### Status & Information

#### status
Show the working tree status.

**Parameters:**
- `short` (boolean, optional): Give output in short format
- `branch` (boolean, optional): Show branch info in short format
- `porcelain` (boolean, optional): Machine-readable output

**Example:**
```json
{"short": true, "branch": true}
```

#### log
Show commit logs.

**Parameters:**
- `count` (number, optional): Number of commits to show (default: 10)
- `oneline` (boolean, optional): One line per commit
- `author` (string, optional): Filter by author
- `since` (string, optional): Show commits since date (e.g., '2 weeks ago', '2024-01-01')
- `until` (string, optional): Show commits until date
- `grep` (string, optional): Filter by commit message
- `path` (string, optional): Filter by file path
- `format` (string, optional): Pretty format (short, medium, full, fuller, oneline)

**Example:**
```json
{"count": 5, "oneline": true, "author": "john"}
```

#### diff
Show changes between commits, working tree, etc.

**Parameters:**
- `staged` (boolean, optional): Show staged changes (--cached)
- `name_only` (boolean, optional): Show only names of changed files
- `stat` (boolean, optional): Show diffstat
- `path` (string, optional): Limit diff to path
- `commit` (string, optional): Compare with specific commit
- `commit2` (string, optional): Second commit for comparison

**Example:**
```json
{"staged": true, "stat": true}
```

#### show
Show various types of objects.

**Parameters:**
- `ref` (string, optional): Commit, tag, or tree to show (default: HEAD)
- `stat` (boolean, optional): Show diffstat
- `name_only` (boolean, optional): Show only file names
- `format` (string, optional): Pretty format

**Example:**
```json
{"ref": "HEAD~1", "stat": true}
```

#### blame
Show what revision and author last modified each line of a file.

**Parameters:**
- `file` (string, required): File to annotate
- `line_range` (string, optional): Line range (e.g., '10,20')

**Example:**
```json
{"file": "src/main.js", "line_range": "1,50"}
```

### Branch Management

#### branch
List, create, or delete branches.

**Parameters:**
- `name` (string, optional): Branch name to create
- `delete` (boolean, optional): Delete branch
- `force_delete` (boolean, optional): Force delete branch (-D)
- `all` (boolean, optional): List all branches (local and remote)
- `remote` (boolean, optional): List remote branches only
- `verbose` (boolean, optional): Show more info
- `move` (string, optional): Rename branch to this name

**Examples:**
```json
// List all branches
{"all": true, "verbose": true}

// Create new branch
{"name": "feature/new-feature"}

// Delete branch
{"name": "old-branch", "delete": true}

// Rename branch
{"name": "old-name", "move": "new-name"}
```

#### checkout
Switch branches or restore working tree files.

**Parameters:**
- `branch` (string, optional): Branch name to checkout
- `create` (boolean, optional): Create new branch (-b)
- `path` (string, optional): File path to restore
- `force` (boolean, optional): Force checkout (discard local changes)

**Examples:**
```json
// Switch to branch
{"branch": "main"}

// Create and switch to new branch
{"branch": "feature/new", "create": true}

// Restore file from HEAD
{"path": "src/file.js"}
```

### Staging & Commits

#### add
Add file contents to the staging area.

**Parameters:**
- `path` (string, required): File or directory path (use '.' for all)
- `all` (boolean, optional): Add all changes (-A)
- `update` (boolean, optional): Update tracked files only (-u)
- `dry_run` (boolean, optional): Dry run

**Examples:**
```json
// Add specific file
{"path": "src/main.js"}

// Add all changes
{"path": ".", "all": true}
```

#### commit
Record changes to the repository.

**Parameters:**
- `message` (string, required): Commit message
- `all` (boolean, optional): Stage all modified files (-a)
- `amend` (boolean, optional): Amend previous commit
- `no_verify` (boolean, optional): Skip pre-commit hooks
- `author` (string, optional): Override author (format: 'Name <email>')

**Examples:**
```json
// Simple commit
{"message": "Fix bug in login flow"}

// Commit all modified files
{"message": "Update dependencies", "all": true}

// Amend last commit
{"message": "Fixed typo in message", "amend": true}
```

#### reset
Reset current HEAD to specified state.

**Parameters:**
- `commit` (string, optional): Commit to reset to (default: HEAD)
- `soft` (boolean, optional): Keep changes staged (--soft)
- `hard` (boolean, optional): Discard all changes (--hard)
- `path` (string, optional): Unstage specific file

**Examples:**
```json
// Unstage file
{"path": "src/file.js"}

// Soft reset to previous commit
{"commit": "HEAD~1", "soft": true}

// Hard reset (destructive!)
{"commit": "HEAD~3", "hard": true}
```

#### stash
Stash the changes in a dirty working directory.

**Parameters:**
- `action` (string, optional): Action: push, pop, list, apply, drop, clear, show (default: push)
- `message` (string, optional): Stash message (for push)
- `index` (number, optional): Stash index (for pop, apply, drop, show)
- `include_untracked` (boolean, optional): Include untracked files

**Examples:**
```json
// Stash current changes
{"action": "push", "message": "WIP: feature work"}

// List stashes
{"action": "list"}

// Apply specific stash
{"action": "apply", "index": 2}

// Pop latest stash
{"action": "pop"}
```

### Merge & Rebase

#### merge
Join two or more development histories together.

**Parameters:**
- `branch` (string, required): Branch to merge
- `no_ff` (boolean, optional): Create merge commit even if fast-forward possible
- `squash` (boolean, optional): Squash commits
- `abort` (boolean, optional): Abort current merge
- `message` (string, optional): Merge commit message

**Examples:**
```json
// Merge branch
{"branch": "feature/auth"}

// Merge with no fast-forward
{"branch": "develop", "no_ff": true}

// Abort merge
{"abort": true}
```

#### rebase
Reapply commits on top of another base.

**Parameters:**
- `branch` (string, optional): Branch to rebase onto
- `abort` (boolean, optional): Abort current rebase
- `continue` (boolean, optional): Continue after resolving conflicts
- `skip` (boolean, optional): Skip current patch

**Examples:**
```json
// Rebase onto main
{"branch": "main"}

// Continue after conflict resolution
{"continue": true}

// Abort rebase
{"abort": true}
```

### Remote Operations

#### pull
Fetch from and integrate with another repository or branch.

**Parameters:**
- `remote` (string, optional): Remote name (default: origin)
- `branch` (string, optional): Branch name
- `rebase` (boolean, optional): Rebase instead of merge
- `no_commit` (boolean, optional): Don't auto-commit after merge

**Examples:**
```json
// Pull from origin
{}

// Pull with rebase
{"rebase": true}

// Pull specific branch
{"remote": "upstream", "branch": "main"}
```

#### push
Update remote refs along with associated objects.

**Parameters:**
- `remote` (string, optional): Remote name (default: origin)
- `branch` (string, optional): Branch name
- `force` (boolean, optional): Force push (blocked for main/master/production)
- `force_with_lease` (boolean, optional): Safer force push
- `set_upstream` (boolean, optional): Set upstream for the branch (-u)
- `tags` (boolean, optional): Push all tags
- `delete` (boolean, optional): Delete remote branch

**Examples:**
```json
// Push current branch
{}

// Push and set upstream
{"branch": "feature/new", "set_upstream": true}

// Force push with lease (safer)
{"branch": "feature/wip", "force_with_lease": true}

// Delete remote branch
{"branch": "old-feature", "delete": true}
```

#### fetch
Download objects and refs from another repository.

**Parameters:**
- `remote` (string, optional): Remote name (default: origin)
- `all` (boolean, optional): Fetch all remotes
- `prune` (boolean, optional): Prune deleted remote branches
- `tags` (boolean, optional): Fetch all tags

**Examples:**
```json
// Fetch from origin
{}

// Fetch all remotes with pruning
{"all": true, "prune": true}
```

#### remote
Manage set of tracked repositories.

**Parameters:**
- `action` (string, optional): Action: list, add, remove, show, rename, get-url, set-url (default: list)
- `name` (string, optional): Remote name
- `url` (string, optional): Remote URL (for add, set-url)
- `new_name` (string, optional): New name (for rename)
- `verbose` (boolean, optional): Show URLs

**Examples:**
```json
// List remotes with URLs
{"action": "list", "verbose": true}

// Add new remote
{"action": "add", "name": "upstream", "url": "https://github.com/org/repo.git"}

// Remove remote
{"action": "remove", "name": "old-origin"}
```

### Tags

#### tag
Create, list, delete or verify tags.

**Parameters:**
- `name` (string, optional): Tag name
- `message` (string, optional): Tag message (creates annotated tag)
- `delete` (boolean, optional): Delete tag
- `list` (boolean, optional): List tags
- `pattern` (string, optional): Pattern to filter tags
- `commit` (string, optional): Commit to tag

**Examples:**
```json
// List all tags
{"list": true}

// List tags matching pattern
{"list": true, "pattern": "v1.*"}

// Create lightweight tag
{"name": "v1.0.0"}

// Create annotated tag
{"name": "v1.0.0", "message": "Release version 1.0.0"}

// Delete tag
{"name": "v0.9.0", "delete": true}
```

### Repository Setup

#### clone
Clone a repository into a new directory.

**Parameters:**
- `url` (string, required): Repository URL
- `directory` (string, optional): Target directory name
- `branch` (string, optional): Branch to clone
- `depth` (number, optional): Shallow clone depth
- `single_branch` (boolean, optional): Clone only one branch
- `recursive` (boolean, optional): Initialize submodules

**Examples:**
```json
// Clone repository
{"url": "https://github.com/org/repo.git"}

// Shallow clone of specific branch
{"url": "https://github.com/org/repo.git", "branch": "main", "depth": 1}

// Clone with submodules
{"url": "https://github.com/org/repo.git", "recursive": true}
```

#### init
Create an empty Git repository or reinitialize an existing one.

**Parameters:**
- `directory` (string, optional): Directory to initialize
- `bare` (boolean, optional): Create a bare repository
- `initial_branch` (string, optional): Initial branch name

**Examples:**
```json
// Initialize in current directory
{}

// Initialize with custom branch name
{"initial_branch": "main"}

// Create bare repository
{"directory": "/path/to/repo.git", "bare": true}
```

## Security

This skill includes security validations:

1. **Commit Message Validation**: Blocks command injection patterns in commit messages (backticks, $(), dangerous shell commands)

2. **Rebase Protection**: Blocks `--exec` flag in rebase to prevent arbitrary command execution

3. **Force Push Protection**: Blocks force push to protected branches (main, master, production). Use `force_with_lease` for safer force pushes.

## Configuration

No configuration required. Git uses:
- System git configuration (~/.gitconfig)
- Repository configuration (.git/config)
- Environment variables (GIT_AUTHOR_NAME, GIT_AUTHOR_EMAIL, etc.)

Optional environment variable:
- `GIT_CMD`: Path to git executable (default: git)
