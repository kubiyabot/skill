/**
 * GitHub Skill - GitHub API Integration
 *
 * Provides access to GitHub repositories, issues, and pull requests.
 * Run directly without build steps:
 *   skill run ./examples/github-skill repo-list
 */

// Note: In a real implementation, you would import Octokit:
// import { Octokit } from '@octokit/rest';

// For this example, we'll simulate GitHub API calls

/**
 * Get skill metadata
 */
export function getMetadata() {
  return {
    name: "github-skill",
    version: "1.0.0",
    description: "GitHub integration for repositories, issues, and pull requests",
    author: "Skill Engine Team"
  };
}

/**
 * Define available tools
 */
export function getTools() {
  return [
    {
      name: "repo-list",
      description: "List repositories for user or organization",
      parameters: [
        {
          name: "org",
          paramType: "string",
          description: "Organization name (uses authenticated user if not specified)",
          required: false,
          defaultValue: ""
        },
        {
          name: "type",
          paramType: "string",
          description: "Filter by type (all, owner, member, private, public)",
          required: false,
          defaultValue: "all"
        },
        {
          name: "sort",
          paramType: "string",
          description: "Sort by (created, updated, pushed, full_name)",
          required: false,
          defaultValue: "updated"
        }
      ]
    },
    {
      name: "repo-create",
      description: "Create a new repository",
      parameters: [
        {
          name: "name",
          paramType: "string",
          description: "Repository name",
          required: true
        },
        {
          name: "description",
          paramType: "string",
          description: "Repository description",
          required: false,
          defaultValue: ""
        },
        {
          name: "private",
          paramType: "boolean",
          description: "Make repository private",
          required: false,
          defaultValue: "false"
        },
        {
          name: "org",
          paramType: "string",
          description: "Organization to create repo in",
          required: false,
          defaultValue: ""
        }
      ]
    },
    {
      name: "issue-list",
      description: "List issues in a repository",
      parameters: [
        {
          name: "repo",
          paramType: "string",
          description: "Repository in format 'owner/repo'",
          required: true
        },
        {
          name: "state",
          paramType: "string",
          description: "Filter by state (open, closed, all)",
          required: false,
          defaultValue: "open"
        },
        {
          name: "assignee",
          paramType: "string",
          description: "Filter by assignee username",
          required: false,
          defaultValue: ""
        },
        {
          name: "labels",
          paramType: "string",
          description: "Filter by labels (comma-separated)",
          required: false,
          defaultValue: ""
        }
      ]
    },
    {
      name: "issue-create",
      description: "Create a new issue",
      parameters: [
        {
          name: "repo",
          paramType: "string",
          description: "Repository in format 'owner/repo'",
          required: true
        },
        {
          name: "title",
          paramType: "string",
          description: "Issue title",
          required: true
        },
        {
          name: "body",
          paramType: "string",
          description: "Issue description/body",
          required: false,
          defaultValue: ""
        },
        {
          name: "labels",
          paramType: "string",
          description: "Labels to add (comma-separated)",
          required: false,
          defaultValue: ""
        },
        {
          name: "assignees",
          paramType: "string",
          description: "Users to assign (comma-separated)",
          required: false,
          defaultValue: ""
        }
      ]
    },
    {
      name: "pr-list",
      description: "List pull requests in a repository",
      parameters: [
        {
          name: "repo",
          paramType: "string",
          description: "Repository in format 'owner/repo'",
          required: true
        },
        {
          name: "state",
          paramType: "string",
          description: "Filter by state (open, closed, all)",
          required: false,
          defaultValue: "open"
        },
        {
          name: "head",
          paramType: "string",
          description: "Filter by head branch",
          required: false,
          defaultValue: ""
        },
        {
          name: "base",
          paramType: "string",
          description: "Filter by base branch",
          required: false,
          defaultValue: ""
        }
      ]
    },
    {
      name: "pr-create",
      description: "Create a new pull request",
      parameters: [
        {
          name: "repo",
          paramType: "string",
          description: "Repository in format 'owner/repo'",
          required: true
        },
        {
          name: "title",
          paramType: "string",
          description: "Pull request title",
          required: true
        },
        {
          name: "head",
          paramType: "string",
          description: "Branch with changes",
          required: true
        },
        {
          name: "base",
          paramType: "string",
          description: "Branch to merge into",
          required: true
        },
        {
          name: "body",
          paramType: "string",
          description: "Pull request description",
          required: false,
          defaultValue: ""
        },
        {
          name: "draft",
          paramType: "boolean",
          description: "Create as draft PR",
          required: false,
          defaultValue: "false"
        }
      ]
    }
  ];
}

/**
 * Execute a tool
 */
export async function executeTool(toolName, argsJson) {
  try {
    const args = JSON.parse(argsJson);

    // Get GitHub configuration from environment
    const githubConfig = getGitHubConfig();

    switch (toolName) {
      case "repo-list":
        return await handleRepoList(args, githubConfig);
      case "repo-create":
        return await handleRepoCreate(args, githubConfig);
      case "issue-list":
        return await handleIssueList(args, githubConfig);
      case "issue-create":
        return await handleIssueCreate(args, githubConfig);
      case "pr-list":
        return await handlePRList(args, githubConfig);
      case "pr-create":
        return await handlePRCreate(args, githubConfig);
      default:
        return {
          success: false,
          output: "",
          errorMessage: `Unknown tool: ${toolName}`
        };
    }
  } catch (error) {
    return {
      success: false,
      output: "",
      errorMessage: `Error executing tool: ${error.message}`
    };
  }
}

/**
 * Validate configuration
 */
export async function validateConfig() {
  const token = process.env.SKILL_GITHUB_TOKEN;

  if (!token) {
    return {
      err: "GitHub token not configured. Run: skill config github-skill"
    };
  }

  return { ok: null };
}

// Tool Handlers

/**
 * Get GitHub configuration from environment
 */
function getGitHubConfig() {
  return {
    token: process.env.SKILL_GITHUB_TOKEN || "",
    username: process.env.SKILL_GITHUB_USERNAME || "",
    defaultOrg: process.env.SKILL_DEFAULT_ORG || ""
  };
}

/**
 * Parse repository string (owner/repo)
 */
function parseRepo(repoStr) {
  const parts = repoStr.split("/");
  if (parts.length !== 2) {
    throw new Error(`Invalid repository format: '${repoStr}'. Use 'owner/repo' format.`);
  }
  return { owner: parts[0], repo: parts[1] };
}

/**
 * Handle repository listing
 */
async function handleRepoList(args, githubConfig) {
  const { org = "", type = "all", sort = "updated" } = args;

  // In real implementation:
  // const octokit = new Octokit({ auth: githubConfig.token });
  // const response = org
  //   ? await octokit.repos.listForOrg({ org, type, sort })
  //   : await octokit.repos.listForAuthenticatedUser({ type, sort });

  // Simulated response
  const targetUser = org || githubConfig.username || "user";
  const simulatedRepos = [
    {
      name: "awesome-project",
      full_name: `${targetUser}/awesome-project`,
      description: "An awesome project",
      private: false,
      html_url: `https://github.com/${targetUser}/awesome-project`,
      language: "JavaScript",
      stargazers_count: 142,
      forks_count: 28,
      updated_at: new Date(Date.now() - 86400000).toISOString()
    },
    {
      name: "data-pipeline",
      full_name: `${targetUser}/data-pipeline`,
      description: "ETL data processing pipeline",
      private: true,
      html_url: `https://github.com/${targetUser}/data-pipeline`,
      language: "Python",
      stargazers_count: 87,
      forks_count: 12,
      updated_at: new Date(Date.now() - 172800000).toISOString()
    },
    {
      name: "ml-experiments",
      full_name: `${targetUser}/ml-experiments`,
      description: "Machine learning experiments",
      private: false,
      html_url: `https://github.com/${targetUser}/ml-experiments`,
      language: "Python",
      stargazers_count: 203,
      forks_count: 45,
      updated_at: new Date(Date.now() - 259200000).toISOString()
    }
  ];

  let output = `\n📚 Repositories${org ? ` for ${org}` : ""}\n`;
  output += `🔍 Filter: ${type} | Sort: ${sort}\n\n`;

  for (const repo of simulatedRepos) {
    const privacy = repo.private ? "🔒 Private" : "🌐 Public";
    const stars = repo.stargazers_count > 0 ? `⭐ ${repo.stargazers_count}` : "";
    const forks = repo.forks_count > 0 ? `🔱 ${repo.forks_count}` : "";
    const updated = new Date(repo.updated_at).toLocaleDateString();

    output += `${repo.full_name} ${privacy}\n`;
    output += `  ${repo.description || "No description"}\n`;
    output += `  ${repo.language || "No language"} | ${stars} | ${forks} | Updated: ${updated}\n`;
    output += `  ${repo.html_url}\n\n`;
  }

  output += `✓ Found ${simulatedRepos.length} repositories\n`;

  return {
    success: true,
    output,
    errorMessage: null
  };
}

/**
 * Handle repository creation
 */
async function handleRepoCreate(args, githubConfig) {
  const { name, description = "", private: isPrivate = false, org = "" } = args;

  if (!name) {
    return {
      success: false,
      output: "",
      errorMessage: "Parameter 'name' is required"
    };
  }

  // In real implementation:
  // const octokit = new Octokit({ auth: githubConfig.token });
  // const response = org
  //   ? await octokit.repos.createInOrg({ org, name, description, private: isPrivate })
  //   : await octokit.repos.createForAuthenticatedUser({ name, description, private: isPrivate });

  const owner = org || githubConfig.username || "user";
  const fullName = `${owner}/${name}`;
  const url = `https://github.com/${fullName}`;

  const output = `
✓ Repository created successfully

Name: ${fullName}
Description: ${description || "No description"}
Visibility: ${isPrivate ? "🔒 Private" : "🌐 Public"}
URL: ${url}

Next steps:
  git remote add origin git@github.com:${fullName}.git
  git push -u origin main
`;

  return {
    success: true,
    output,
    errorMessage: null
  };
}

/**
 * Handle issue listing
 */
async function handleIssueList(args, githubConfig) {
  const { repo, state = "open", assignee = "", labels = "" } = args;

  if (!repo) {
    return {
      success: false,
      output: "",
      errorMessage: "Parameter 'repo' is required (format: owner/repo)"
    };
  }

  const { owner, repo: repoName } = parseRepo(repo);

  // In real implementation:
  // const octokit = new Octokit({ auth: githubConfig.token });
  // const response = await octokit.issues.listForRepo({
  //   owner, repo: repoName, state, assignee, labels
  // });

  // Simulated response
  const simulatedIssues = [
    {
      number: 123,
      title: "Fix login bug on mobile devices",
      state: "open",
      user: { login: "developer1" },
      assignee: { login: "developer2" },
      labels: [{ name: "bug" }, { name: "priority-high" }],
      created_at: new Date(Date.now() - 86400000).toISOString(),
      comments: 5,
      html_url: `https://github.com/${repo}/issues/123`
    },
    {
      number: 118,
      title: "Add dark mode support",
      state: "open",
      user: { login: "designer1" },
      assignee: null,
      labels: [{ name: "feature" }, { name: "ui" }],
      created_at: new Date(Date.now() - 259200000).toISOString(),
      comments: 12,
      html_url: `https://github.com/${repo}/issues/118`
    },
    {
      number: 115,
      title: "Update dependencies to latest versions",
      state: "open",
      user: { login: "maintainer" },
      assignee: { login: "developer1" },
      labels: [{ name: "maintenance" }],
      created_at: new Date(Date.now() - 432000000).toISOString(),
      comments: 3,
      html_url: `https://github.com/${repo}/issues/115`
    }
  ];

  let output = `\n🐛 Issues in ${repo}\n`;
  output += `📊 State: ${state}`;
  if (assignee) output += ` | Assignee: ${assignee}`;
  if (labels) output += ` | Labels: ${labels}`;
  output += `\n\n`;

  for (const issue of simulatedIssues) {
    const assigneeStr = issue.assignee ? `👤 ${issue.assignee.login}` : "👤 Unassigned";
    const labelsStr = issue.labels.map(l => `🏷️  ${l.name}`).join(" ");
    const age = Math.floor((Date.now() - new Date(issue.created_at).getTime()) / 86400000);

    output += `#${issue.number}: ${issue.title}\n`;
    output += `  By ${issue.user.login} | ${assigneeStr} | ${age} days old | 💬 ${issue.comments} comments\n`;
    if (labelsStr) output += `  ${labelsStr}\n`;
    output += `  ${issue.html_url}\n\n`;
  }

  output += `✓ Found ${simulatedIssues.length} issues\n`;

  return {
    success: true,
    output,
    errorMessage: null
  };
}

/**
 * Handle issue creation
 */
async function handleIssueCreate(args, githubConfig) {
  const { repo, title, body = "", labels = "", assignees = "" } = args;

  if (!repo) {
    return {
      success: false,
      output: "",
      errorMessage: "Parameter 'repo' is required (format: owner/repo)"
    };
  }

  if (!title) {
    return {
      success: false,
      output: "",
      errorMessage: "Parameter 'title' is required"
    };
  }

  const { owner, repo: repoName } = parseRepo(repo);

  // In real implementation:
  // const octokit = new Octokit({ auth: githubConfig.token });
  // const response = await octokit.issues.create({
  //   owner, repo: repoName, title, body,
  //   labels: labels ? labels.split(',') : [],
  //   assignees: assignees ? assignees.split(',') : []
  // });

  const issueNumber = 124; // Simulated
  const url = `https://github.com/${repo}/issues/${issueNumber}`;

  let output = `\n✓ Issue created successfully\n\n`;
  output += `Repository: ${repo}\n`;
  output += `Issue #${issueNumber}: ${title}\n`;
  if (body) output += `\nDescription:\n${body}\n`;
  if (labels) output += `\n🏷️  Labels: ${labels}\n`;
  if (assignees) output += `👤 Assignees: ${assignees}\n`;
  output += `\n${url}\n`;

  return {
    success: true,
    output,
    errorMessage: null
  };
}

/**
 * Handle pull request listing
 */
async function handlePRList(args, githubConfig) {
  const { repo, state = "open", head = "", base = "" } = args;

  if (!repo) {
    return {
      success: false,
      output: "",
      errorMessage: "Parameter 'repo' is required (format: owner/repo)"
    };
  }

  const { owner, repo: repoName } = parseRepo(repo);

  // In real implementation:
  // const octokit = new Octokit({ auth: githubConfig.token });
  // const response = await octokit.pulls.list({
  //   owner, repo: repoName, state, head, base
  // });

  // Simulated response
  const simulatedPRs = [
    {
      number: 45,
      title: "Implement user authentication system",
      state: "open",
      user: { login: "developer1" },
      head: { ref: "feature/auth" },
      base: { ref: "main" },
      draft: false,
      created_at: new Date(Date.now() - 172800000).toISOString(),
      comments: 8,
      html_url: `https://github.com/${repo}/pull/45`
    },
    {
      number: 42,
      title: "Refactor database queries",
      state: "open",
      user: { login: "developer2" },
      head: { ref: "refactor/db" },
      base: { ref: "main" },
      draft: true,
      created_at: new Date(Date.now() - 432000000).toISOString(),
      comments: 15,
      html_url: `https://github.com/${repo}/pull/42`
    }
  ];

  let output = `\n🔀 Pull Requests in ${repo}\n`;
  output += `📊 State: ${state}`;
  if (head) output += ` | Head: ${head}`;
  if (base) output += ` | Base: ${base}`;
  output += `\n\n`;

  for (const pr of simulatedPRs) {
    const draftStr = pr.draft ? "📝 Draft" : "✅ Ready";
    const age = Math.floor((Date.now() - new Date(pr.created_at).getTime()) / 86400000);

    output += `#${pr.number}: ${pr.title} ${draftStr}\n`;
    output += `  By ${pr.user.login} | ${pr.head.ref} → ${pr.base.ref} | ${age} days old | 💬 ${pr.comments} comments\n`;
    output += `  ${pr.html_url}\n\n`;
  }

  output += `✓ Found ${simulatedPRs.length} pull requests\n`;

  return {
    success: true,
    output,
    errorMessage: null
  };
}

/**
 * Handle pull request creation
 */
async function handlePRCreate(args, githubConfig) {
  const { repo, title, head, base, body = "", draft = false } = args;

  if (!repo) {
    return {
      success: false,
      output: "",
      errorMessage: "Parameter 'repo' is required (format: owner/repo)"
    };
  }

  if (!title || !head || !base) {
    return {
      success: false,
      output: "",
      errorMessage: "Parameters 'title', 'head', and 'base' are required"
    };
  }

  const { owner, repo: repoName } = parseRepo(repo);

  // In real implementation:
  // const octokit = new Octokit({ auth: githubConfig.token });
  // const response = await octokit.pulls.create({
  //   owner, repo: repoName, title, head, base, body, draft
  // });

  const prNumber = 46; // Simulated
  const url = `https://github.com/${repo}/pull/${prNumber}`;

  let output = `\n✓ Pull request created successfully\n\n`;
  output += `Repository: ${repo}\n`;
  output += `PR #${prNumber}: ${title}\n`;
  output += `🔀 ${head} → ${base}\n`;
  output += `Status: ${draft ? "📝 Draft" : "✅ Ready for review"}\n`;
  if (body) output += `\nDescription:\n${body}\n`;
  output += `\n${url}\n`;

  return {
    success: true,
    output,
    errorMessage: null
  };
}
