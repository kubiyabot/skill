/**
 * Git Skill - Version control operations via git CLI
 *
 * This skill executes real git commands for version control management.
 * It uses the git CLI from the environment or default location.
 *
 * Requirements:
 * - git must be installed and in PATH
 * - Appropriate repository access and credentials
 */

// Helper functions for result formatting
function success(output) {
  return JSON.stringify({
    ok: { success: true, output, errorMessage: null }
  });
}

function error(message) {
  return JSON.stringify({ err: message });
}

// Define skill metadata
export function getMetadata() {
  return JSON.stringify({
    name: "git",
    version: "1.0.0",
    description: "Git version control operations with native git CLI integration",
    author: "Skill Engine"
  });
}

// Define available tools (20 tools)
export function getTools() {
  return JSON.stringify([
    // Status & Info Tools
    {
      name: "status",
      description: "Show the working tree status",
      parameters: [
        { name: "short", paramType: "boolean", description: "Give output in short format", required: false, defaultValue: "false" },
        { name: "branch", paramType: "boolean", description: "Show branch info in short format", required: false, defaultValue: "false" },
        { name: "porcelain", paramType: "boolean", description: "Machine-readable output", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "log",
      description: "Show commit logs",
      parameters: [
        { name: "count", paramType: "number", description: "Number of commits to show", required: false, defaultValue: "10" },
        { name: "oneline", paramType: "boolean", description: "One line per commit", required: false, defaultValue: "false" },
        { name: "author", paramType: "string", description: "Filter by author", required: false },
        { name: "since", paramType: "string", description: "Show commits since date (e.g., '2 weeks ago', '2024-01-01')", required: false },
        { name: "until", paramType: "string", description: "Show commits until date", required: false },
        { name: "grep", paramType: "string", description: "Filter by commit message", required: false },
        { name: "path", paramType: "string", description: "Filter by file path", required: false },
        { name: "format", paramType: "string", description: "Pretty format (short, medium, full, fuller, oneline)", required: false }
      ]
    },
    {
      name: "diff",
      description: "Show changes between commits, working tree, etc.",
      parameters: [
        { name: "staged", paramType: "boolean", description: "Show staged changes (--cached)", required: false, defaultValue: "false" },
        { name: "name_only", paramType: "boolean", description: "Show only names of changed files", required: false, defaultValue: "false" },
        { name: "stat", paramType: "boolean", description: "Show diffstat", required: false, defaultValue: "false" },
        { name: "path", paramType: "string", description: "Limit diff to path", required: false },
        { name: "commit", paramType: "string", description: "Compare with specific commit", required: false },
        { name: "commit2", paramType: "string", description: "Second commit for comparison", required: false }
      ]
    },
    {
      name: "show",
      description: "Show various types of objects",
      parameters: [
        { name: "ref", paramType: "string", description: "Commit, tag, or tree to show", required: false, defaultValue: "HEAD" },
        { name: "stat", paramType: "boolean", description: "Show diffstat", required: false, defaultValue: "false" },
        { name: "name_only", paramType: "boolean", description: "Show only file names", required: false, defaultValue: "false" },
        { name: "format", paramType: "string", description: "Pretty format", required: false }
      ]
    },
    {
      name: "blame",
      description: "Show what revision and author last modified each line of a file",
      parameters: [
        { name: "file", paramType: "string", description: "File to annotate", required: true },
        { name: "line_range", paramType: "string", description: "Line range (e.g., '10,20')", required: false }
      ]
    },
    // Branch Tools
    {
      name: "branch",
      description: "List, create, or delete branches",
      parameters: [
        { name: "name", paramType: "string", description: "Branch name to create", required: false },
        { name: "delete", paramType: "boolean", description: "Delete branch", required: false, defaultValue: "false" },
        { name: "force_delete", paramType: "boolean", description: "Force delete branch (-D)", required: false, defaultValue: "false" },
        { name: "all", paramType: "boolean", description: "List all branches (local and remote)", required: false, defaultValue: "false" },
        { name: "remote", paramType: "boolean", description: "List remote branches only", required: false, defaultValue: "false" },
        { name: "verbose", paramType: "boolean", description: "Show more info", required: false, defaultValue: "false" },
        { name: "move", paramType: "string", description: "Rename branch to this name", required: false }
      ]
    },
    {
      name: "checkout",
      description: "Switch branches or restore working tree files",
      parameters: [
        { name: "branch", paramType: "string", description: "Branch name to checkout", required: false },
        { name: "create", paramType: "boolean", description: "Create new branch (-b)", required: false, defaultValue: "false" },
        { name: "path", paramType: "string", description: "File path to restore", required: false },
        { name: "force", paramType: "boolean", description: "Force checkout (discard local changes)", required: false, defaultValue: "false" }
      ]
    },
    // Staging & Commit Tools
    {
      name: "add",
      description: "Add file contents to the staging area",
      parameters: [
        { name: "path", paramType: "string", description: "File or directory path (use '.' for all)", required: true },
        { name: "all", paramType: "boolean", description: "Add all changes (-A)", required: false, defaultValue: "false" },
        { name: "update", paramType: "boolean", description: "Update tracked files only (-u)", required: false, defaultValue: "false" },
        { name: "dry_run", paramType: "boolean", description: "Dry run", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "commit",
      description: "Record changes to the repository",
      parameters: [
        { name: "message", paramType: "string", description: "Commit message", required: true },
        { name: "all", paramType: "boolean", description: "Stage all modified files (-a)", required: false, defaultValue: "false" },
        { name: "amend", paramType: "boolean", description: "Amend previous commit", required: false, defaultValue: "false" },
        { name: "no_verify", paramType: "boolean", description: "Skip pre-commit hooks", required: false, defaultValue: "false" },
        { name: "author", paramType: "string", description: "Override author (format: 'Name <email>')", required: false }
      ]
    },
    {
      name: "reset",
      description: "Reset current HEAD to specified state",
      parameters: [
        { name: "commit", paramType: "string", description: "Commit to reset to", required: false, defaultValue: "HEAD" },
        { name: "soft", paramType: "boolean", description: "Keep changes staged (--soft)", required: false, defaultValue: "false" },
        { name: "hard", paramType: "boolean", description: "Discard all changes (--hard)", required: false, defaultValue: "false" },
        { name: "path", paramType: "string", description: "Unstage specific file", required: false }
      ]
    },
    {
      name: "stash",
      description: "Stash the changes in a dirty working directory",
      parameters: [
        { name: "action", paramType: "string", description: "Action: push, pop, list, apply, drop, clear, show", required: false, defaultValue: "push" },
        { name: "message", paramType: "string", description: "Stash message (for push)", required: false },
        { name: "index", paramType: "number", description: "Stash index (for pop, apply, drop, show)", required: false },
        { name: "include_untracked", paramType: "boolean", description: "Include untracked files", required: false, defaultValue: "false" }
      ]
    },
    // Merge & Rebase Tools
    {
      name: "merge",
      description: "Join two or more development histories together",
      parameters: [
        { name: "branch", paramType: "string", description: "Branch to merge", required: true },
        { name: "no_ff", paramType: "boolean", description: "Create merge commit even if fast-forward possible", required: false, defaultValue: "false" },
        { name: "squash", paramType: "boolean", description: "Squash commits", required: false, defaultValue: "false" },
        { name: "abort", paramType: "boolean", description: "Abort current merge", required: false, defaultValue: "false" },
        { name: "message", paramType: "string", description: "Merge commit message", required: false }
      ]
    },
    {
      name: "rebase",
      description: "Reapply commits on top of another base",
      parameters: [
        { name: "branch", paramType: "string", description: "Branch to rebase onto", required: false },
        { name: "abort", paramType: "boolean", description: "Abort current rebase", required: false, defaultValue: "false" },
        { name: "continue", paramType: "boolean", description: "Continue after resolving conflicts", required: false, defaultValue: "false" },
        { name: "skip", paramType: "boolean", description: "Skip current patch", required: false, defaultValue: "false" }
      ]
    },
    // Remote Tools
    {
      name: "pull",
      description: "Fetch from and integrate with another repository or branch",
      parameters: [
        { name: "remote", paramType: "string", description: "Remote name", required: false, defaultValue: "origin" },
        { name: "branch", paramType: "string", description: "Branch name", required: false },
        { name: "rebase", paramType: "boolean", description: "Rebase instead of merge", required: false, defaultValue: "false" },
        { name: "no_commit", paramType: "boolean", description: "Don't auto-commit after merge", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "push",
      description: "Update remote refs along with associated objects",
      parameters: [
        { name: "remote", paramType: "string", description: "Remote name", required: false, defaultValue: "origin" },
        { name: "branch", paramType: "string", description: "Branch name", required: false },
        { name: "force", paramType: "boolean", description: "Force push (use with caution)", required: false, defaultValue: "false" },
        { name: "force_with_lease", paramType: "boolean", description: "Safer force push", required: false, defaultValue: "false" },
        { name: "set_upstream", paramType: "boolean", description: "Set upstream for the branch (-u)", required: false, defaultValue: "false" },
        { name: "tags", paramType: "boolean", description: "Push all tags", required: false, defaultValue: "false" },
        { name: "delete", paramType: "boolean", description: "Delete remote branch", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "fetch",
      description: "Download objects and refs from another repository",
      parameters: [
        { name: "remote", paramType: "string", description: "Remote name", required: false, defaultValue: "origin" },
        { name: "all", paramType: "boolean", description: "Fetch all remotes", required: false, defaultValue: "false" },
        { name: "prune", paramType: "boolean", description: "Prune deleted remote branches", required: false, defaultValue: "false" },
        { name: "tags", paramType: "boolean", description: "Fetch all tags", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "remote",
      description: "Manage set of tracked repositories",
      parameters: [
        { name: "action", paramType: "string", description: "Action: list, add, remove, show, rename, get-url, set-url", required: false, defaultValue: "list" },
        { name: "name", paramType: "string", description: "Remote name", required: false },
        { name: "url", paramType: "string", description: "Remote URL (for add, set-url)", required: false },
        { name: "new_name", paramType: "string", description: "New name (for rename)", required: false },
        { name: "verbose", paramType: "boolean", description: "Show URLs", required: false, defaultValue: "false" }
      ]
    },
    // Tag Tools
    {
      name: "tag",
      description: "Create, list, delete or verify tags",
      parameters: [
        { name: "name", paramType: "string", description: "Tag name", required: false },
        { name: "message", paramType: "string", description: "Tag message (creates annotated tag)", required: false },
        { name: "delete", paramType: "boolean", description: "Delete tag", required: false, defaultValue: "false" },
        { name: "list", paramType: "boolean", description: "List tags", required: false, defaultValue: "false" },
        { name: "pattern", paramType: "string", description: "Pattern to filter tags", required: false },
        { name: "commit", paramType: "string", description: "Commit to tag", required: false }
      ]
    },
    // Repository Tools
    {
      name: "clone",
      description: "Clone a repository into a new directory",
      parameters: [
        { name: "url", paramType: "string", description: "Repository URL", required: true },
        { name: "directory", paramType: "string", description: "Target directory name", required: false },
        { name: "branch", paramType: "string", description: "Branch to clone", required: false },
        { name: "depth", paramType: "number", description: "Shallow clone depth", required: false },
        { name: "single_branch", paramType: "boolean", description: "Clone only one branch", required: false, defaultValue: "false" },
        { name: "recursive", paramType: "boolean", description: "Initialize submodules", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "init",
      description: "Create an empty Git repository or reinitialize an existing one",
      parameters: [
        { name: "directory", paramType: "string", description: "Directory to initialize", required: false },
        { name: "bare", paramType: "boolean", description: "Create a bare repository", required: false, defaultValue: "false" },
        { name: "initial_branch", paramType: "string", description: "Initial branch name", required: false }
      ]
    }
  ]);
}

// Execute a tool
export async function executeTool(toolName, argsJson) {
  try {
    const args = JSON.parse(argsJson);
    let gitArgs = [];

    // Validate security (block dangerous patterns)
    const securityError = validateSecurity(toolName, args);
    if (securityError) {
      return error(securityError);
    }

    switch (toolName) {
      case "status":
        gitArgs = buildStatusArgs(args);
        break;
      case "log":
        gitArgs = buildLogArgs(args);
        break;
      case "diff":
        gitArgs = buildDiffArgs(args);
        break;
      case "show":
        gitArgs = buildShowArgs(args);
        break;
      case "blame":
        gitArgs = buildBlameArgs(args);
        break;
      case "branch":
        gitArgs = buildBranchArgs(args);
        break;
      case "checkout":
        gitArgs = buildCheckoutArgs(args);
        break;
      case "add":
        gitArgs = buildAddArgs(args);
        break;
      case "commit":
        gitArgs = buildCommitArgs(args);
        break;
      case "reset":
        gitArgs = buildResetArgs(args);
        break;
      case "stash":
        gitArgs = buildStashArgs(args);
        break;
      case "merge":
        gitArgs = buildMergeArgs(args);
        break;
      case "rebase":
        gitArgs = buildRebaseArgs(args);
        break;
      case "pull":
        gitArgs = buildPullArgs(args);
        break;
      case "push":
        gitArgs = buildPushArgs(args);
        break;
      case "fetch":
        gitArgs = buildFetchArgs(args);
        break;
      case "remote":
        gitArgs = buildRemoteArgs(args);
        break;
      case "tag":
        gitArgs = buildTagArgs(args);
        break;
      case "clone":
        gitArgs = buildCloneArgs(args);
        break;
      case "init":
        gitArgs = buildInitArgs(args);
        break;
      default:
        return error(`Unknown tool: ${toolName}`);
    }

    // Run git command
    const result = runGit(gitArgs);
    return result;
  } catch (e) {
    return error(`Error executing tool: ${e.message || e}`);
  }
}

// Validate config (none required for git)
export function validateConfig() {
  return JSON.stringify({ ok: null });
}

// Security validation
function validateSecurity(toolName, args) {
  // Block dangerous exec patterns in commit messages
  if (toolName === "commit" && args.message) {
    // Check for command injection attempts
    const dangerousPatterns = [/`.*`/, /\$\(.*\)/, /;\s*rm\s/, /;\s*sudo\s/];
    for (const pattern of dangerousPatterns) {
      if (pattern.test(args.message)) {
        return "Security: Dangerous pattern detected in commit message";
      }
    }
  }

  // Block --exec in rebase (can execute arbitrary commands)
  if (toolName === "rebase" && args.branch && args.branch.includes("--exec")) {
    return "Security: --exec is not allowed in rebase";
  }

  // Block force push to protected branch patterns
  if (toolName === "push" && args.force === true) {
    const branch = args.branch || "";
    if (branch === "main" || branch === "master" || branch === "production") {
      return `Security: Force push to '${branch}' is blocked. Use --force_with_lease for safer force push or push to a different branch.`;
    }
  }

  return null;
}

// Run git command and return formatted result
function runGit(args) {
  const git = globalThis.process?.env?.GIT_CMD || "git";
  const fullCmd = [git, ...args].join(" ");

  // Return the command for host execution
  return success(`Command: ${fullCmd}`);
}

// === Argument Builders ===

function buildStatusArgs(args) {
  const gitArgs = ["status"];
  if (args.short === "true" || args.short === true) gitArgs.push("-s");
  if (args.branch === "true" || args.branch === true) gitArgs.push("-b");
  if (args.porcelain === "true" || args.porcelain === true) gitArgs.push("--porcelain");
  return gitArgs;
}

function buildLogArgs(args) {
  const gitArgs = ["log"];
  const count = args.count || 10;
  gitArgs.push(`-n${count}`);
  if (args.oneline === "true" || args.oneline === true) gitArgs.push("--oneline");
  if (args.author) gitArgs.push(`--author=${args.author}`);
  if (args.since) gitArgs.push(`--since="${args.since}"`);
  if (args.until) gitArgs.push(`--until="${args.until}"`);
  if (args.grep) gitArgs.push(`--grep="${args.grep}"`);
  if (args.format) gitArgs.push(`--pretty=${args.format}`);
  if (args.path) gitArgs.push("--", args.path);
  return gitArgs;
}

function buildDiffArgs(args) {
  const gitArgs = ["diff"];
  if (args.staged === "true" || args.staged === true) gitArgs.push("--cached");
  if (args.name_only === "true" || args.name_only === true) gitArgs.push("--name-only");
  if (args.stat === "true" || args.stat === true) gitArgs.push("--stat");
  if (args.commit) gitArgs.push(args.commit);
  if (args.commit2) gitArgs.push(args.commit2);
  if (args.path) gitArgs.push("--", args.path);
  return gitArgs;
}

function buildShowArgs(args) {
  const gitArgs = ["show"];
  const ref = args.ref || "HEAD";
  gitArgs.push(ref);
  if (args.stat === "true" || args.stat === true) gitArgs.push("--stat");
  if (args.name_only === "true" || args.name_only === true) gitArgs.push("--name-only");
  if (args.format) gitArgs.push(`--pretty=${args.format}`);
  return gitArgs;
}

function buildBlameArgs(args) {
  const gitArgs = ["blame"];
  if (args.line_range) {
    const [start, end] = args.line_range.split(",");
    gitArgs.push(`-L${start},${end}`);
  }
  gitArgs.push(args.file);
  return gitArgs;
}

function buildBranchArgs(args) {
  const gitArgs = ["branch"];
  if (args.all === "true" || args.all === true) gitArgs.push("-a");
  if (args.remote === "true" || args.remote === true) gitArgs.push("-r");
  if (args.verbose === "true" || args.verbose === true) gitArgs.push("-v");
  if (args.delete === "true" || args.delete === true) {
    gitArgs.push("-d", args.name);
  } else if (args.force_delete === "true" || args.force_delete === true) {
    gitArgs.push("-D", args.name);
  } else if (args.move) {
    gitArgs.push("-m", args.name, args.move);
  } else if (args.name) {
    gitArgs.push(args.name);
  }
  return gitArgs;
}

function buildCheckoutArgs(args) {
  const gitArgs = ["checkout"];
  if (args.force === "true" || args.force === true) gitArgs.push("-f");
  if (args.create === "true" || args.create === true) gitArgs.push("-b");
  if (args.branch) gitArgs.push(args.branch);
  if (args.path) gitArgs.push("--", args.path);
  return gitArgs;
}

function buildAddArgs(args) {
  const gitArgs = ["add"];
  if (args.all === "true" || args.all === true) {
    gitArgs.push("-A");
  } else if (args.update === "true" || args.update === true) {
    gitArgs.push("-u");
  }
  if (args.dry_run === "true" || args.dry_run === true) gitArgs.push("-n");
  gitArgs.push(args.path);
  return gitArgs;
}

function buildCommitArgs(args) {
  const gitArgs = ["commit"];
  if (args.all === "true" || args.all === true) gitArgs.push("-a");
  if (args.amend === "true" || args.amend === true) gitArgs.push("--amend");
  if (args.no_verify === "true" || args.no_verify === true) gitArgs.push("--no-verify");
  if (args.author) gitArgs.push(`--author="${args.author}"`);
  // Escape quotes in message
  const message = args.message.replace(/"/g, '\\"');
  gitArgs.push("-m", `"${message}"`);
  return gitArgs;
}

function buildResetArgs(args) {
  const gitArgs = ["reset"];
  if (args.soft === "true" || args.soft === true) {
    gitArgs.push("--soft");
  } else if (args.hard === "true" || args.hard === true) {
    gitArgs.push("--hard");
  }
  if (args.path) {
    gitArgs.push("--", args.path);
  } else {
    gitArgs.push(args.commit || "HEAD");
  }
  return gitArgs;
}

function buildStashArgs(args) {
  const gitArgs = ["stash"];
  const action = args.action || "push";

  switch (action) {
    case "push":
      gitArgs.push("push");
      if (args.include_untracked === "true" || args.include_untracked === true) gitArgs.push("-u");
      if (args.message) gitArgs.push("-m", `"${args.message}"`);
      break;
    case "pop":
      gitArgs.push("pop");
      if (args.index !== undefined) gitArgs.push(`stash@{${args.index}}`);
      break;
    case "apply":
      gitArgs.push("apply");
      if (args.index !== undefined) gitArgs.push(`stash@{${args.index}}`);
      break;
    case "drop":
      gitArgs.push("drop");
      if (args.index !== undefined) gitArgs.push(`stash@{${args.index}}`);
      break;
    case "list":
      gitArgs.push("list");
      break;
    case "show":
      gitArgs.push("show");
      if (args.index !== undefined) gitArgs.push(`stash@{${args.index}}`);
      break;
    case "clear":
      gitArgs.push("clear");
      break;
    default:
      gitArgs.push(action);
  }
  return gitArgs;
}

function buildMergeArgs(args) {
  const gitArgs = ["merge"];
  if (args.abort === "true" || args.abort === true) {
    gitArgs.push("--abort");
    return gitArgs;
  }
  if (args.no_ff === "true" || args.no_ff === true) gitArgs.push("--no-ff");
  if (args.squash === "true" || args.squash === true) gitArgs.push("--squash");
  if (args.message) gitArgs.push("-m", `"${args.message}"`);
  gitArgs.push(args.branch);
  return gitArgs;
}

function buildRebaseArgs(args) {
  const gitArgs = ["rebase"];
  if (args.abort === "true" || args.abort === true) {
    gitArgs.push("--abort");
    return gitArgs;
  }
  if (args.continue === "true" || args.continue === true) {
    gitArgs.push("--continue");
    return gitArgs;
  }
  if (args.skip === "true" || args.skip === true) {
    gitArgs.push("--skip");
    return gitArgs;
  }
  if (args.branch) gitArgs.push(args.branch);
  return gitArgs;
}

function buildPullArgs(args) {
  const gitArgs = ["pull"];
  if (args.rebase === "true" || args.rebase === true) gitArgs.push("--rebase");
  if (args.no_commit === "true" || args.no_commit === true) gitArgs.push("--no-commit");
  gitArgs.push(args.remote || "origin");
  if (args.branch) gitArgs.push(args.branch);
  return gitArgs;
}

function buildPushArgs(args) {
  const gitArgs = ["push"];
  if (args.force === "true" || args.force === true) gitArgs.push("--force");
  if (args.force_with_lease === "true" || args.force_with_lease === true) gitArgs.push("--force-with-lease");
  if (args.set_upstream === "true" || args.set_upstream === true) gitArgs.push("-u");
  if (args.tags === "true" || args.tags === true) gitArgs.push("--tags");
  if (args.delete === "true" || args.delete === true) gitArgs.push("--delete");
  gitArgs.push(args.remote || "origin");
  if (args.branch) gitArgs.push(args.branch);
  return gitArgs;
}

function buildFetchArgs(args) {
  const gitArgs = ["fetch"];
  if (args.all === "true" || args.all === true) {
    gitArgs.push("--all");
  } else {
    gitArgs.push(args.remote || "origin");
  }
  if (args.prune === "true" || args.prune === true) gitArgs.push("--prune");
  if (args.tags === "true" || args.tags === true) gitArgs.push("--tags");
  return gitArgs;
}

function buildRemoteArgs(args) {
  const gitArgs = ["remote"];
  const action = args.action || "list";

  switch (action) {
    case "list":
      if (args.verbose === "true" || args.verbose === true) gitArgs.push("-v");
      break;
    case "add":
      gitArgs.push("add", args.name, args.url);
      break;
    case "remove":
      gitArgs.push("remove", args.name);
      break;
    case "show":
      gitArgs.push("show", args.name);
      break;
    case "rename":
      gitArgs.push("rename", args.name, args.new_name);
      break;
    case "get-url":
      gitArgs.push("get-url", args.name);
      break;
    case "set-url":
      gitArgs.push("set-url", args.name, args.url);
      break;
    default:
      gitArgs.push(action);
  }
  return gitArgs;
}

function buildTagArgs(args) {
  const gitArgs = ["tag"];
  if (args.list === "true" || args.list === true || !args.name) {
    if (args.pattern) gitArgs.push("-l", args.pattern);
    return gitArgs;
  }
  if (args.delete === "true" || args.delete === true) {
    gitArgs.push("-d", args.name);
    return gitArgs;
  }
  if (args.message) {
    gitArgs.push("-a", args.name, "-m", `"${args.message}"`);
  } else {
    gitArgs.push(args.name);
  }
  if (args.commit) gitArgs.push(args.commit);
  return gitArgs;
}

function buildCloneArgs(args) {
  const gitArgs = ["clone"];
  if (args.branch) gitArgs.push("-b", args.branch);
  if (args.depth) gitArgs.push("--depth", String(args.depth));
  if (args.single_branch === "true" || args.single_branch === true) gitArgs.push("--single-branch");
  if (args.recursive === "true" || args.recursive === true) gitArgs.push("--recursive");
  gitArgs.push(args.url);
  if (args.directory) gitArgs.push(args.directory);
  return gitArgs;
}

function buildInitArgs(args) {
  const gitArgs = ["init"];
  if (args.bare === "true" || args.bare === true) gitArgs.push("--bare");
  if (args.initial_branch) gitArgs.push("-b", args.initial_branch);
  if (args.directory) gitArgs.push(args.directory);
  return gitArgs;
}
