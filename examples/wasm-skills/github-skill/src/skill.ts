/**
 * GitHub Skill - GitHub API Integration
 *
 * Comprehensive GitHub API integration with 25 tools covering:
 * - Repositories (list, get, create, delete, fork)
 * - Issues (list, get, create, update, close, comment)
 * - Pull Requests (list, get, create, merge, close, review, request-reviewers)
 * - Branches (list, create, delete)
 * - Releases (list, create)
 * - Actions (workflow-list, workflow-run)
 *
 * Setup:
 *   1. Create a GitHub Personal Access Token at https://github.com/settings/tokens
 *   2. Required scopes: repo, workflow, read:org
 *   3. Set token: export SKILL_GITHUB_TOKEN=ghp_your_token
 */

import {
  defineSkill,
  getConfig,
  ok,
  err,
  errors,
  createAuthenticatedClient,
  type ExecutionResult,
  type HttpResponse,
  type ToolHandler,
} from '@skill-engine/sdk';

// Types
interface GitHubRepo {
  id: number;
  name: string;
  full_name: string;
  description: string | null;
  private: boolean;
  html_url: string;
  language: string | null;
  stargazers_count: number;
  forks_count: number;
  updated_at: string;
  default_branch: string;
}

interface GitHubIssue {
  number: number;
  title: string;
  state: string;
  body: string | null;
  user: { login: string };
  assignee: { login: string } | null;
  labels: Array<{ name: string }>;
  created_at: string;
  updated_at: string;
  comments: number;
  html_url: string;
}

interface GitHubPR {
  number: number;
  title: string;
  state: string;
  body: string | null;
  user: { login: string };
  head: { ref: string; sha: string };
  base: { ref: string };
  draft: boolean;
  mergeable: boolean | null;
  created_at: string;
  updated_at: string;
  comments: number;
  html_url: string;
}

interface GitHubBranch {
  name: string;
  commit: { sha: string };
  protected: boolean;
}

interface GitHubRelease {
  id: number;
  tag_name: string;
  name: string;
  body: string | null;
  draft: boolean;
  prerelease: boolean;
  created_at: string;
  published_at: string;
  html_url: string;
}

interface GitHubWorkflow {
  id: number;
  name: string;
  path: string;
  state: string;
  created_at: string;
  updated_at: string;
}

interface GitHubWorkflowRun {
  id: number;
  name: string;
  status: string;
  conclusion: string | null;
  html_url: string;
}

// Create authenticated GitHub client
function getGitHubClient() {
  return createAuthenticatedClient({
    baseUrl: 'https://api.github.com',
    authType: 'bearer',
    tokenKey: 'GITHUB_TOKEN',
    headers: {
      'Accept': 'application/vnd.github+json',
      'X-GitHub-Api-Version': '2022-11-28',
    },
  });
}

// Parse owner/repo format
function parseRepo(repoStr: string): { owner: string; repo: string } {
  const parts = repoStr.split('/');
  if (parts.length !== 2) {
    throw new Error(`Invalid repository format: '${repoStr}'. Use 'owner/repo' format.`);
  }
  return { owner: parts[0], repo: parts[1] };
}

// Handle GitHub API errors
function handleGitHubError(response: HttpResponse<unknown>): ExecutionResult {
  if (response.status === 401) {
    return err('Authentication failed. Check your GITHUB_TOKEN.', errors.auth());
  }
  if (response.status === 403) {
    const data = response.data as { message?: string };
    if (data?.message?.includes('rate limit')) {
      return err('GitHub API rate limit exceeded. Try again later.', errors.rateLimit());
    }
    return err('Access denied. Check your token permissions.');
  }
  if (response.status === 404) {
    return err('Resource not found. Check the repository/issue/PR exists.');
  }
  if (response.status === 422) {
    const data = response.data as { message?: string; errors?: Array<{ message: string }> };
    const msg = data.errors?.map(e => e.message).join('; ') || data.message || 'Validation failed';
    return err(`GitHub API error: ${msg}`, errors.validation());
  }
  return err(`GitHub API error: ${response.status} ${response.statusText}`);
}

// Common parameters
const repoParam = {
  name: 'repo',
  paramType: 'string' as const,
  description: 'Repository in format "owner/repo"',
  required: true,
};

// Skill definition
export default defineSkill({
  metadata: {
    name: 'github-skill',
    version: '2.0.0',
    description: 'GitHub integration for repositories, issues, pull requests, branches, releases, and actions',
    author: 'Skill Engine Team',
    tags: ['github', 'git', 'repository', 'issues', 'pr', 'ci-cd'],
  },
  tools: [
    // ========================================
    // Repository Tools
    // ========================================
    {
      name: 'repo-list',
      description: 'List repositories for user or organization',
      parameters: [
        {
          name: 'org',
          paramType: 'string',
          description: 'Organization name (uses authenticated user if not specified)',
          required: false,
        },
        {
          name: 'type',
          paramType: 'string',
          description: 'Filter by type: all, owner, member, private, public',
          required: false,
          defaultValue: 'all',
          validation: { enum: ['all', 'owner', 'member', 'private', 'public'] },
        },
        {
          name: 'sort',
          paramType: 'string',
          description: 'Sort by: created, updated, pushed, full_name',
          required: false,
          defaultValue: 'updated',
          validation: { enum: ['created', 'updated', 'pushed', 'full_name'] },
        },
        {
          name: 'per_page',
          paramType: 'number',
          description: 'Results per page (max 100)',
          required: false,
          defaultValue: '30',
          validation: { minimum: 1, maximum: 100 },
        },
      ],
      handler: (async (args: {
        org?: string;
        type?: string;
        sort?: string;
        per_page?: number;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const params = new URLSearchParams();
        if (args.type) params.append('type', args.type);
        if (args.sort) params.append('sort', args.sort);
        params.append('per_page', String(args.per_page || 30));

        const endpoint = args.org
          ? `/orgs/${args.org}/repos?${params}`
          : `/user/repos?${params}`;

        const response = await client.get<GitHubRepo[]>(endpoint);

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const repos = response.data;
        if (repos.length === 0) {
          return ok('No repositories found.');
        }

        const output = repos.map(r => {
          const visibility = r.private ? '[Private]' : '[Public]';
          return `${r.full_name} ${visibility}\n  ${r.description || 'No description'}\n  ${r.language || 'N/A'} | Stars: ${r.stargazers_count} | Forks: ${r.forks_count}\n  ${r.html_url}`;
        }).join('\n\n');

        return ok(`Found ${repos.length} repositories:\n\n${output}`, { repos });
      }) as ToolHandler,
    },

    {
      name: 'repo-get',
      description: 'Get detailed information about a repository',
      parameters: [repoParam],
      handler: (async (args: { repo: string }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const response = await client.get<GitHubRepo>(`/repos/${owner}/${repo}`);

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const r = response.data;
        const output = [
          `Repository: ${r.full_name}`,
          `Description: ${r.description || 'N/A'}`,
          `Visibility: ${r.private ? 'Private' : 'Public'}`,
          `Language: ${r.language || 'N/A'}`,
          `Default Branch: ${r.default_branch}`,
          `Stars: ${r.stargazers_count}`,
          `Forks: ${r.forks_count}`,
          `URL: ${r.html_url}`,
        ].join('\n');

        return ok(output, { repo: r });
      }) as ToolHandler,
    },

    {
      name: 'repo-create',
      description: 'Create a new repository',
      parameters: [
        {
          name: 'name',
          paramType: 'string',
          description: 'Repository name',
          required: true,
        },
        {
          name: 'description',
          paramType: 'string',
          description: 'Repository description',
          required: false,
        },
        {
          name: 'private',
          paramType: 'boolean',
          description: 'Make repository private',
          required: false,
          defaultValue: 'false',
        },
        {
          name: 'org',
          paramType: 'string',
          description: 'Organization to create repo in',
          required: false,
        },
        {
          name: 'auto_init',
          paramType: 'boolean',
          description: 'Initialize with README',
          required: false,
          defaultValue: 'false',
        },
      ],
      handler: (async (args: {
        name: string;
        description?: string;
        private?: boolean;
        org?: string;
        auto_init?: boolean;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();

        const body = {
          name: args.name,
          description: args.description,
          private: args.private,
          auto_init: args.auto_init,
        };

        const endpoint = args.org ? `/orgs/${args.org}/repos` : '/user/repos';
        const response = await client.post<GitHubRepo>(endpoint, body);

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const r = response.data;
        return ok(
          `Repository created successfully!\n\nName: ${r.full_name}\nURL: ${r.html_url}\n\nClone with:\n  git clone ${r.html_url}.git`,
          { repo: r }
        );
      }) as ToolHandler,
    },

    {
      name: 'repo-delete',
      description: 'Delete a repository (requires admin access)',
      parameters: [repoParam],
      handler: (async (args: { repo: string }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const response = await client.delete(`/repos/${owner}/${repo}`);

        if (!response.ok) {
          return handleGitHubError(response);
        }

        return ok(`Repository ${args.repo} deleted successfully.`);
      }) as ToolHandler,
    },

    {
      name: 'repo-fork',
      description: 'Fork a repository',
      parameters: [
        repoParam,
        {
          name: 'organization',
          paramType: 'string',
          description: 'Organization to fork to (forks to user if not specified)',
          required: false,
        },
        {
          name: 'name',
          paramType: 'string',
          description: 'Name for the forked repository',
          required: false,
        },
      ],
      handler: (async (args: {
        repo: string;
        organization?: string;
        name?: string;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const body: Record<string, string> = {};
        if (args.organization) body.organization = args.organization;
        if (args.name) body.name = args.name;

        const response = await client.post<GitHubRepo>(`/repos/${owner}/${repo}/forks`, body);

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const r = response.data;
        return ok(
          `Repository forked successfully!\n\nForked to: ${r.full_name}\nURL: ${r.html_url}`,
          { repo: r }
        );
      }) as ToolHandler,
    },

    // ========================================
    // Issue Tools
    // ========================================
    {
      name: 'issue-list',
      description: 'List issues in a repository',
      parameters: [
        repoParam,
        {
          name: 'state',
          paramType: 'string',
          description: 'Filter by state: open, closed, all',
          required: false,
          defaultValue: 'open',
          validation: { enum: ['open', 'closed', 'all'] },
        },
        {
          name: 'assignee',
          paramType: 'string',
          description: 'Filter by assignee username',
          required: false,
        },
        {
          name: 'labels',
          paramType: 'string',
          description: 'Filter by labels (comma-separated)',
          required: false,
        },
        {
          name: 'per_page',
          paramType: 'number',
          description: 'Results per page (max 100)',
          required: false,
          defaultValue: '30',
        },
      ],
      handler: (async (args: {
        repo: string;
        state?: string;
        assignee?: string;
        labels?: string;
        per_page?: number;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const params = new URLSearchParams();
        params.append('state', args.state || 'open');
        if (args.assignee) params.append('assignee', args.assignee);
        if (args.labels) params.append('labels', args.labels);
        params.append('per_page', String(args.per_page || 30));

        const response = await client.get<GitHubIssue[]>(`/repos/${owner}/${repo}/issues?${params}`);

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const issues = response.data.filter(i => !('pull_request' in i)); // Exclude PRs
        if (issues.length === 0) {
          return ok('No issues found.');
        }

        const output = issues.map(i => {
          const labels = i.labels.map(l => l.name).join(', ');
          const assignee = i.assignee?.login || 'Unassigned';
          return `#${i.number}: ${i.title} [${i.state}]\n  Assignee: ${assignee} | Labels: ${labels || 'None'}\n  ${i.html_url}`;
        }).join('\n\n');

        return ok(`Found ${issues.length} issues:\n\n${output}`, { issues });
      }) as ToolHandler,
    },

    {
      name: 'issue-get',
      description: 'Get detailed information about an issue',
      parameters: [
        repoParam,
        {
          name: 'issue_number',
          paramType: 'number',
          description: 'Issue number',
          required: true,
        },
      ],
      handler: (async (args: { repo: string; issue_number: number }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const response = await client.get<GitHubIssue>(`/repos/${owner}/${repo}/issues/${args.issue_number}`);

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const i = response.data;
        const labels = i.labels.map(l => l.name).join(', ');
        const output = [
          `Issue #${i.number}: ${i.title}`,
          `State: ${i.state}`,
          `Author: ${i.user.login}`,
          `Assignee: ${i.assignee?.login || 'None'}`,
          `Labels: ${labels || 'None'}`,
          `Comments: ${i.comments}`,
          `Created: ${i.created_at}`,
          `Updated: ${i.updated_at}`,
          `URL: ${i.html_url}`,
          '',
          'Body:',
          i.body || '(No description)',
        ].join('\n');

        return ok(output, { issue: i });
      }) as ToolHandler,
    },

    {
      name: 'issue-create',
      description: 'Create a new issue',
      parameters: [
        repoParam,
        {
          name: 'title',
          paramType: 'string',
          description: 'Issue title',
          required: true,
        },
        {
          name: 'body',
          paramType: 'string',
          description: 'Issue description/body',
          required: false,
        },
        {
          name: 'labels',
          paramType: 'string',
          description: 'Labels to add (comma-separated)',
          required: false,
        },
        {
          name: 'assignees',
          paramType: 'string',
          description: 'Users to assign (comma-separated)',
          required: false,
        },
      ],
      handler: (async (args: {
        repo: string;
        title: string;
        body?: string;
        labels?: string;
        assignees?: string;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const body: Record<string, unknown> = {
          title: args.title,
        };
        if (args.body) body.body = args.body;
        if (args.labels) body.labels = args.labels.split(',').map(l => l.trim());
        if (args.assignees) body.assignees = args.assignees.split(',').map(a => a.trim());

        const response = await client.post<GitHubIssue>(`/repos/${owner}/${repo}/issues`, body);

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const i = response.data;
        return ok(
          `Issue created successfully!\n\nIssue #${i.number}: ${i.title}\nURL: ${i.html_url}`,
          { issue: i }
        );
      }) as ToolHandler,
    },

    {
      name: 'issue-update',
      description: 'Update an existing issue',
      parameters: [
        repoParam,
        {
          name: 'issue_number',
          paramType: 'number',
          description: 'Issue number',
          required: true,
        },
        {
          name: 'title',
          paramType: 'string',
          description: 'New issue title',
          required: false,
        },
        {
          name: 'body',
          paramType: 'string',
          description: 'New issue body',
          required: false,
        },
        {
          name: 'state',
          paramType: 'string',
          description: 'Issue state: open or closed',
          required: false,
          validation: { enum: ['open', 'closed'] },
        },
        {
          name: 'labels',
          paramType: 'string',
          description: 'Labels (comma-separated, replaces existing)',
          required: false,
        },
        {
          name: 'assignees',
          paramType: 'string',
          description: 'Assignees (comma-separated, replaces existing)',
          required: false,
        },
      ],
      handler: (async (args: {
        repo: string;
        issue_number: number;
        title?: string;
        body?: string;
        state?: string;
        labels?: string;
        assignees?: string;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const body: Record<string, unknown> = {};
        if (args.title) body.title = args.title;
        if (args.body) body.body = args.body;
        if (args.state) body.state = args.state;
        if (args.labels) body.labels = args.labels.split(',').map(l => l.trim());
        if (args.assignees) body.assignees = args.assignees.split(',').map(a => a.trim());

        const response = await client.patch<GitHubIssue>(`/repos/${owner}/${repo}/issues/${args.issue_number}`, body);

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const i = response.data;
        return ok(
          `Issue #${i.number} updated successfully!\nURL: ${i.html_url}`,
          { issue: i }
        );
      }) as ToolHandler,
    },

    {
      name: 'issue-close',
      description: 'Close an issue',
      parameters: [
        repoParam,
        {
          name: 'issue_number',
          paramType: 'number',
          description: 'Issue number',
          required: true,
        },
        {
          name: 'reason',
          paramType: 'string',
          description: 'Close reason: completed or not_planned',
          required: false,
          defaultValue: 'completed',
          validation: { enum: ['completed', 'not_planned'] },
        },
      ],
      handler: (async (args: {
        repo: string;
        issue_number: number;
        reason?: string;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const response = await client.patch<GitHubIssue>(`/repos/${owner}/${repo}/issues/${args.issue_number}`, {
          state: 'closed',
          state_reason: args.reason || 'completed',
        });

        if (!response.ok) {
          return handleGitHubError(response);
        }

        return ok(`Issue #${args.issue_number} closed successfully.`);
      }) as ToolHandler,
    },

    {
      name: 'issue-comment',
      description: 'Add a comment to an issue',
      parameters: [
        repoParam,
        {
          name: 'issue_number',
          paramType: 'number',
          description: 'Issue number',
          required: true,
        },
        {
          name: 'body',
          paramType: 'string',
          description: 'Comment text',
          required: true,
        },
      ],
      handler: (async (args: {
        repo: string;
        issue_number: number;
        body: string;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const response = await client.post<{ id: number; html_url: string }>(
          `/repos/${owner}/${repo}/issues/${args.issue_number}/comments`,
          { body: args.body }
        );

        if (!response.ok) {
          return handleGitHubError(response);
        }

        return ok(`Comment added to issue #${args.issue_number}.\nURL: ${response.data.html_url}`);
      }) as ToolHandler,
    },

    // ========================================
    // Pull Request Tools
    // ========================================
    {
      name: 'pr-list',
      description: 'List pull requests in a repository',
      parameters: [
        repoParam,
        {
          name: 'state',
          paramType: 'string',
          description: 'Filter by state: open, closed, all',
          required: false,
          defaultValue: 'open',
          validation: { enum: ['open', 'closed', 'all'] },
        },
        {
          name: 'head',
          paramType: 'string',
          description: 'Filter by head branch (user:branch format)',
          required: false,
        },
        {
          name: 'base',
          paramType: 'string',
          description: 'Filter by base branch',
          required: false,
        },
        {
          name: 'per_page',
          paramType: 'number',
          description: 'Results per page (max 100)',
          required: false,
          defaultValue: '30',
        },
      ],
      handler: (async (args: {
        repo: string;
        state?: string;
        head?: string;
        base?: string;
        per_page?: number;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const params = new URLSearchParams();
        params.append('state', args.state || 'open');
        if (args.head) params.append('head', args.head);
        if (args.base) params.append('base', args.base);
        params.append('per_page', String(args.per_page || 30));

        const response = await client.get<GitHubPR[]>(`/repos/${owner}/${repo}/pulls?${params}`);

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const prs = response.data;
        if (prs.length === 0) {
          return ok('No pull requests found.');
        }

        const output = prs.map(pr => {
          const draft = pr.draft ? '[Draft]' : '';
          return `#${pr.number}: ${pr.title} ${draft}\n  ${pr.head.ref} -> ${pr.base.ref}\n  By: ${pr.user.login} | Comments: ${pr.comments}\n  ${pr.html_url}`;
        }).join('\n\n');

        return ok(`Found ${prs.length} pull requests:\n\n${output}`, { prs });
      }) as ToolHandler,
    },

    {
      name: 'pr-get',
      description: 'Get detailed information about a pull request',
      parameters: [
        repoParam,
        {
          name: 'pr_number',
          paramType: 'number',
          description: 'Pull request number',
          required: true,
        },
      ],
      handler: (async (args: { repo: string; pr_number: number }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const response = await client.get<GitHubPR>(`/repos/${owner}/${repo}/pulls/${args.pr_number}`);

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const pr = response.data;
        const output = [
          `PR #${pr.number}: ${pr.title}`,
          `State: ${pr.state}${pr.draft ? ' (Draft)' : ''}`,
          `Author: ${pr.user.login}`,
          `Branch: ${pr.head.ref} -> ${pr.base.ref}`,
          `Mergeable: ${pr.mergeable === null ? 'Checking...' : pr.mergeable ? 'Yes' : 'No'}`,
          `Comments: ${pr.comments}`,
          `Created: ${pr.created_at}`,
          `Updated: ${pr.updated_at}`,
          `URL: ${pr.html_url}`,
          '',
          'Description:',
          pr.body || '(No description)',
        ].join('\n');

        return ok(output, { pr });
      }) as ToolHandler,
    },

    {
      name: 'pr-create',
      description: 'Create a new pull request',
      parameters: [
        repoParam,
        {
          name: 'title',
          paramType: 'string',
          description: 'Pull request title',
          required: true,
        },
        {
          name: 'head',
          paramType: 'string',
          description: 'Branch with changes',
          required: true,
        },
        {
          name: 'base',
          paramType: 'string',
          description: 'Branch to merge into',
          required: true,
        },
        {
          name: 'body',
          paramType: 'string',
          description: 'Pull request description',
          required: false,
        },
        {
          name: 'draft',
          paramType: 'boolean',
          description: 'Create as draft PR',
          required: false,
          defaultValue: 'false',
        },
      ],
      handler: (async (args: {
        repo: string;
        title: string;
        head: string;
        base: string;
        body?: string;
        draft?: boolean;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const response = await client.post<GitHubPR>(`/repos/${owner}/${repo}/pulls`, {
          title: args.title,
          head: args.head,
          base: args.base,
          body: args.body,
          draft: args.draft,
        });

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const pr = response.data;
        return ok(
          `Pull request created successfully!\n\nPR #${pr.number}: ${pr.title}\nURL: ${pr.html_url}`,
          { pr }
        );
      }) as ToolHandler,
    },

    {
      name: 'pr-merge',
      description: 'Merge a pull request',
      parameters: [
        repoParam,
        {
          name: 'pr_number',
          paramType: 'number',
          description: 'Pull request number',
          required: true,
        },
        {
          name: 'merge_method',
          paramType: 'string',
          description: 'Merge method: merge, squash, rebase',
          required: false,
          defaultValue: 'merge',
          validation: { enum: ['merge', 'squash', 'rebase'] },
        },
        {
          name: 'commit_title',
          paramType: 'string',
          description: 'Commit title (for squash/merge)',
          required: false,
        },
        {
          name: 'commit_message',
          paramType: 'string',
          description: 'Commit message (for squash/merge)',
          required: false,
        },
      ],
      handler: (async (args: {
        repo: string;
        pr_number: number;
        merge_method?: string;
        commit_title?: string;
        commit_message?: string;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const body: Record<string, string> = {};
        body.merge_method = args.merge_method || 'merge';
        if (args.commit_title) body.commit_title = args.commit_title;
        if (args.commit_message) body.commit_message = args.commit_message;

        const response = await client.put<{ sha: string; merged: boolean; message: string }>(
          `/repos/${owner}/${repo}/pulls/${args.pr_number}/merge`,
          body
        );

        if (!response.ok) {
          return handleGitHubError(response);
        }

        return ok(`PR #${args.pr_number} merged successfully!\nCommit: ${response.data.sha}`);
      }) as ToolHandler,
    },

    {
      name: 'pr-close',
      description: 'Close a pull request without merging',
      parameters: [
        repoParam,
        {
          name: 'pr_number',
          paramType: 'number',
          description: 'Pull request number',
          required: true,
        },
      ],
      handler: (async (args: { repo: string; pr_number: number }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const response = await client.patch<GitHubPR>(`/repos/${owner}/${repo}/pulls/${args.pr_number}`, {
          state: 'closed',
        });

        if (!response.ok) {
          return handleGitHubError(response);
        }

        return ok(`PR #${args.pr_number} closed.`);
      }) as ToolHandler,
    },

    {
      name: 'pr-review',
      description: 'Submit a review for a pull request',
      parameters: [
        repoParam,
        {
          name: 'pr_number',
          paramType: 'number',
          description: 'Pull request number',
          required: true,
        },
        {
          name: 'event',
          paramType: 'string',
          description: 'Review action: APPROVE, REQUEST_CHANGES, COMMENT',
          required: true,
          validation: { enum: ['APPROVE', 'REQUEST_CHANGES', 'COMMENT'] },
        },
        {
          name: 'body',
          paramType: 'string',
          description: 'Review comment',
          required: false,
        },
      ],
      handler: (async (args: {
        repo: string;
        pr_number: number;
        event: string;
        body?: string;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const response = await client.post<{ id: number; html_url: string }>(
          `/repos/${owner}/${repo}/pulls/${args.pr_number}/reviews`,
          {
            event: args.event,
            body: args.body,
          }
        );

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const eventText = args.event === 'APPROVE' ? 'Approved'
          : args.event === 'REQUEST_CHANGES' ? 'Requested changes on'
          : 'Commented on';

        return ok(`${eventText} PR #${args.pr_number}.\nURL: ${response.data.html_url}`);
      }) as ToolHandler,
    },

    {
      name: 'pr-request-reviewers',
      description: 'Request reviewers for a pull request',
      parameters: [
        repoParam,
        {
          name: 'pr_number',
          paramType: 'number',
          description: 'Pull request number',
          required: true,
        },
        {
          name: 'reviewers',
          paramType: 'string',
          description: 'Usernames to request (comma-separated)',
          required: false,
        },
        {
          name: 'team_reviewers',
          paramType: 'string',
          description: 'Team slugs to request (comma-separated)',
          required: false,
        },
      ],
      handler: (async (args: {
        repo: string;
        pr_number: number;
        reviewers?: string;
        team_reviewers?: string;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const body: Record<string, string[]> = {};
        if (args.reviewers) body.reviewers = args.reviewers.split(',').map(r => r.trim());
        if (args.team_reviewers) body.team_reviewers = args.team_reviewers.split(',').map(t => t.trim());

        const response = await client.post(
          `/repos/${owner}/${repo}/pulls/${args.pr_number}/requested_reviewers`,
          body
        );

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const requested = [args.reviewers, args.team_reviewers].filter(Boolean).join(', ');
        return ok(`Reviewers requested for PR #${args.pr_number}: ${requested}`);
      }) as ToolHandler,
    },

    // ========================================
    // Branch Tools
    // ========================================
    {
      name: 'branch-list',
      description: 'List branches in a repository',
      parameters: [
        repoParam,
        {
          name: 'protected',
          paramType: 'boolean',
          description: 'Only show protected branches',
          required: false,
          defaultValue: 'false',
        },
        {
          name: 'per_page',
          paramType: 'number',
          description: 'Results per page (max 100)',
          required: false,
          defaultValue: '30',
        },
      ],
      handler: (async (args: {
        repo: string;
        protected?: boolean;
        per_page?: number;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const params = new URLSearchParams();
        if (args.protected) params.append('protected', 'true');
        params.append('per_page', String(args.per_page || 30));

        const response = await client.get<GitHubBranch[]>(`/repos/${owner}/${repo}/branches?${params}`);

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const branches = response.data;
        if (branches.length === 0) {
          return ok('No branches found.');
        }

        const output = branches.map(b => {
          const protection = b.protected ? ' [Protected]' : '';
          return `${b.name}${protection}\n  SHA: ${b.commit.sha.substring(0, 7)}`;
        }).join('\n\n');

        return ok(`Found ${branches.length} branches:\n\n${output}`, { branches });
      }) as ToolHandler,
    },

    {
      name: 'branch-create',
      description: 'Create a new branch',
      parameters: [
        repoParam,
        {
          name: 'branch',
          paramType: 'string',
          description: 'New branch name',
          required: true,
        },
        {
          name: 'from',
          paramType: 'string',
          description: 'Source branch or commit SHA (defaults to default branch)',
          required: false,
        },
      ],
      handler: (async (args: {
        repo: string;
        branch: string;
        from?: string;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        // Get source SHA
        let sha: string;
        if (args.from && args.from.match(/^[a-f0-9]{40}$/i)) {
          sha = args.from;
        } else {
          const sourceBranch = args.from || 'HEAD';
          const refResponse = await client.get<{ object: { sha: string } }>(
            `/repos/${owner}/${repo}/git/ref/heads/${sourceBranch === 'HEAD' ? '' : sourceBranch}`
          );

          if (sourceBranch === 'HEAD' || !refResponse.ok) {
            // Get default branch
            const repoResponse = await client.get<GitHubRepo>(`/repos/${owner}/${repo}`);
            if (!repoResponse.ok) {
              return handleGitHubError(repoResponse);
            }
            const defaultBranch = repoResponse.data.default_branch;
            const defaultRefResponse = await client.get<{ object: { sha: string } }>(
              `/repos/${owner}/${repo}/git/ref/heads/${defaultBranch}`
            );
            if (!defaultRefResponse.ok) {
              return handleGitHubError(defaultRefResponse);
            }
            sha = defaultRefResponse.data.object.sha;
          } else {
            sha = refResponse.data.object.sha;
          }
        }

        // Create branch
        const response = await client.post<{ ref: string; object: { sha: string } }>(
          `/repos/${owner}/${repo}/git/refs`,
          {
            ref: `refs/heads/${args.branch}`,
            sha,
          }
        );

        if (!response.ok) {
          return handleGitHubError(response);
        }

        return ok(`Branch '${args.branch}' created from SHA ${sha.substring(0, 7)}`);
      }) as ToolHandler,
    },

    {
      name: 'branch-delete',
      description: 'Delete a branch',
      parameters: [
        repoParam,
        {
          name: 'branch',
          paramType: 'string',
          description: 'Branch name to delete',
          required: true,
        },
      ],
      handler: (async (args: { repo: string; branch: string }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const response = await client.delete(`/repos/${owner}/${repo}/git/refs/heads/${args.branch}`);

        if (!response.ok) {
          return handleGitHubError(response);
        }

        return ok(`Branch '${args.branch}' deleted.`);
      }) as ToolHandler,
    },

    // ========================================
    // Release Tools
    // ========================================
    {
      name: 'release-list',
      description: 'List releases in a repository',
      parameters: [
        repoParam,
        {
          name: 'per_page',
          paramType: 'number',
          description: 'Results per page (max 100)',
          required: false,
          defaultValue: '30',
        },
      ],
      handler: (async (args: { repo: string; per_page?: number }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const response = await client.get<GitHubRelease[]>(
          `/repos/${owner}/${repo}/releases?per_page=${args.per_page || 30}`
        );

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const releases = response.data;
        if (releases.length === 0) {
          return ok('No releases found.');
        }

        const output = releases.map(r => {
          const status = r.draft ? '[Draft]' : r.prerelease ? '[Pre-release]' : '[Latest]';
          return `${r.tag_name} ${status}\n  Name: ${r.name || 'N/A'}\n  Published: ${r.published_at || 'Not published'}\n  ${r.html_url}`;
        }).join('\n\n');

        return ok(`Found ${releases.length} releases:\n\n${output}`, { releases });
      }) as ToolHandler,
    },

    {
      name: 'release-create',
      description: 'Create a new release',
      parameters: [
        repoParam,
        {
          name: 'tag_name',
          paramType: 'string',
          description: 'Tag name for the release',
          required: true,
        },
        {
          name: 'name',
          paramType: 'string',
          description: 'Release title',
          required: false,
        },
        {
          name: 'body',
          paramType: 'string',
          description: 'Release notes/description',
          required: false,
        },
        {
          name: 'target_commitish',
          paramType: 'string',
          description: 'Branch or commit to create tag from (defaults to default branch)',
          required: false,
        },
        {
          name: 'draft',
          paramType: 'boolean',
          description: 'Create as draft release',
          required: false,
          defaultValue: 'false',
        },
        {
          name: 'prerelease',
          paramType: 'boolean',
          description: 'Mark as pre-release',
          required: false,
          defaultValue: 'false',
        },
        {
          name: 'generate_release_notes',
          paramType: 'boolean',
          description: 'Auto-generate release notes',
          required: false,
          defaultValue: 'false',
        },
      ],
      handler: (async (args: {
        repo: string;
        tag_name: string;
        name?: string;
        body?: string;
        target_commitish?: string;
        draft?: boolean;
        prerelease?: boolean;
        generate_release_notes?: boolean;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const response = await client.post<GitHubRelease>(`/repos/${owner}/${repo}/releases`, {
          tag_name: args.tag_name,
          name: args.name || args.tag_name,
          body: args.body,
          target_commitish: args.target_commitish,
          draft: args.draft,
          prerelease: args.prerelease,
          generate_release_notes: args.generate_release_notes,
        });

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const r = response.data;
        return ok(
          `Release created successfully!\n\nTag: ${r.tag_name}\nName: ${r.name || 'N/A'}\nURL: ${r.html_url}`,
          { release: r }
        );
      }) as ToolHandler,
    },

    // ========================================
    // Actions/Workflow Tools
    // ========================================
    {
      name: 'workflow-list',
      description: 'List workflows in a repository',
      parameters: [
        repoParam,
        {
          name: 'per_page',
          paramType: 'number',
          description: 'Results per page (max 100)',
          required: false,
          defaultValue: '30',
        },
      ],
      handler: (async (args: { repo: string; per_page?: number }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const response = await client.get<{ workflows: GitHubWorkflow[] }>(
          `/repos/${owner}/${repo}/actions/workflows?per_page=${args.per_page || 30}`
        );

        if (!response.ok) {
          return handleGitHubError(response);
        }

        const workflows = response.data.workflows;
        if (workflows.length === 0) {
          return ok('No workflows found.');
        }

        const output = workflows.map(w => {
          return `${w.name} [${w.state}]\n  ID: ${w.id}\n  Path: ${w.path}`;
        }).join('\n\n');

        return ok(`Found ${workflows.length} workflows:\n\n${output}`, { workflows });
      }) as ToolHandler,
    },

    {
      name: 'workflow-run',
      description: 'Trigger a workflow run',
      parameters: [
        repoParam,
        {
          name: 'workflow_id',
          paramType: 'string',
          description: 'Workflow ID or filename (e.g., "ci.yml")',
          required: true,
        },
        {
          name: 'ref',
          paramType: 'string',
          description: 'Branch or tag to run workflow on',
          required: true,
        },
        {
          name: 'inputs',
          paramType: 'string',
          description: 'Workflow inputs as JSON (e.g., {"key": "value"})',
          required: false,
        },
      ],
      handler: (async (args: {
        repo: string;
        workflow_id: string;
        ref: string;
        inputs?: string;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const { owner, repo } = parseRepo(args.repo);

        const body: Record<string, unknown> = { ref: args.ref };
        if (args.inputs) {
          try {
            body.inputs = JSON.parse(args.inputs);
          } catch {
            return err('Invalid JSON in inputs parameter');
          }
        }

        const response = await client.post(
          `/repos/${owner}/${repo}/actions/workflows/${args.workflow_id}/dispatches`,
          body
        );

        if (!response.ok) {
          return handleGitHubError(response);
        }

        return ok(`Workflow '${args.workflow_id}' triggered on '${args.ref}'.\n\nCheck status at: https://github.com/${args.repo}/actions`);
      }) as ToolHandler,
    },
  ],
});
