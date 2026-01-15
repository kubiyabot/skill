/**
 * GitHub OAuth Skill
 *
 * A comprehensive GitHub integration demonstrating:
 * - OAuth2 Device Flow authentication
 * - Type-safe HTTP client with authentication
 * - Parameter validation
 * - Structured error handling
 *
 * Setup:
 *   1. skill auth login github --skill github-oauth-skill
 *   2. skill run ./examples/github-oauth-skill list-repos
 *
 * @example
 * ```bash
 * # List your repositories
 * skill run github-oauth-skill list-repos
 *
 * # Create an issue
 * skill run github-oauth-skill create-issue --repo owner/repo --title "Bug report"
 *
 * # List pull requests
 * skill run github-oauth-skill list-prs --repo owner/repo --state open
 * ```
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

// GitHub API types
interface GitHubRepo {
  id: number;
  name: string;
  full_name: string;
  description: string | null;
  private: boolean;
  html_url: string;
  stargazers_count: number;
  forks_count: number;
  language: string | null;
  updated_at: string;
}

interface GitHubIssue {
  id: number;
  number: number;
  title: string;
  state: string;
  html_url: string;
  user: { login: string };
  created_at: string;
  labels: Array<{ name: string }>;
}

interface GitHubPullRequest {
  id: number;
  number: number;
  title: string;
  state: string;
  html_url: string;
  user: { login: string };
  head: { ref: string };
  base: { ref: string };
  draft: boolean;
  created_at: string;
}

interface GitHubUser {
  login: string;
  name: string | null;
  email: string | null;
  bio: string | null;
  public_repos: number;
  followers: number;
  following: number;
}

// Create authenticated GitHub client
function getGitHubClient() {
  return createAuthenticatedClient({
    baseUrl: 'https://api.github.com',
    authType: 'bearer',
    tokenKey: 'GITHUB_TOKEN',
    headers: {
      'Accept': 'application/vnd.github.v3+json',
      'X-GitHub-Api-Version': '2022-11-28',
    },
  });
}

// Handle GitHub API errors
function handleApiError(response: HttpResponse): ExecutionResult {
  if (response.status === 401) {
    return err('Authentication failed. Run: skill auth login github', errors.auth());
  }
  if (response.status === 403) {
    return err('Rate limit exceeded or permission denied', errors.rateLimit());
  }
  if (response.status === 404) {
    return err('Resource not found', errors.notFound('Resource'));
  }
  return err(`GitHub API error: ${response.status} ${response.statusText}`);
}

// Skill definition
export default defineSkill({
  metadata: {
    name: 'github-oauth-skill',
    version: '1.0.0',
    description: 'GitHub integration with OAuth2 authentication for repos, issues, and PRs',
    author: 'Skill Engine Team',
    tags: ['github', 'oauth', 'api', 'git'],
    repository: 'https://github.com/skill-engine/skill-engine',
  },
  tools: [
    // ========================================
    // User Tools
    // ========================================
    {
      name: 'whoami',
      description: 'Get information about the authenticated GitHub user',
      parameters: [],
      handler: (async (): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const response = await client.get<GitHubUser>('/user');

        if (!response.ok) {
          return handleApiError(response);
        }

        const user = response.data;
        const output = [
          `GitHub User: ${user.login}`,
          user.name ? `Name: ${user.name}` : null,
          user.email ? `Email: ${user.email}` : null,
          user.bio ? `Bio: ${user.bio}` : null,
          `Public Repos: ${user.public_repos}`,
          `Followers: ${user.followers} | Following: ${user.following}`,
        ].filter(Boolean).join('\n');

        return ok(output, { user });
      }) as ToolHandler,
    },

    // ========================================
    // Repository Tools
    // ========================================
    {
      name: 'list-repos',
      description: 'List repositories for the authenticated user or an organization',
      parameters: [
        {
          name: 'org',
          paramType: 'string',
          description: 'Organization name (uses your repos if not specified)',
          required: false,
        },
        {
          name: 'type',
          paramType: 'string',
          description: 'Filter type: all, owner, member, private, public',
          required: false,
          defaultValue: 'all',
          validation: {
            enum: ['all', 'owner', 'member', 'private', 'public'],
          },
        },
        {
          name: 'sort',
          paramType: 'string',
          description: 'Sort by: created, updated, pushed, full_name',
          required: false,
          defaultValue: 'updated',
          validation: {
            enum: ['created', 'updated', 'pushed', 'full_name'],
          },
        },
        {
          name: 'limit',
          paramType: 'number',
          description: 'Maximum number of repos to return',
          required: false,
          defaultValue: '30',
          validation: {
            minimum: 1,
            maximum: 100,
          },
        },
      ],
      handler: (async (args: {
        org?: string;
        type: string;
        sort: string;
        limit: number;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();

        const endpoint = args.org
          ? `/orgs/${args.org}/repos`
          : '/user/repos';

        const params = new URLSearchParams({
          type: args.type,
          sort: args.sort,
          per_page: String(args.limit),
        });

        const response = await client.get<GitHubRepo[]>(`${endpoint}?${params}`);

        if (!response.ok) {
          return handleApiError(response);
        }

        const repos = response.data;

        if (repos.length === 0) {
          return ok('No repositories found.');
        }

        const output = repos.map(repo => {
          const visibility = repo.private ? '🔒' : '🌐';
          const stars = repo.stargazers_count > 0 ? `⭐${repo.stargazers_count}` : '';
          const lang = repo.language ? `[${repo.language}]` : '';
          return `${visibility} ${repo.full_name} ${lang} ${stars}`.trim();
        }).join('\n');

        return ok(
          `Found ${repos.length} repositories:\n\n${output}`,
          { repos: repos.map(r => ({ name: r.full_name, url: r.html_url })) }
        );
      }) as ToolHandler,
    },

    {
      name: 'get-repo',
      description: 'Get detailed information about a repository',
      parameters: [
        {
          name: 'repo',
          paramType: 'string',
          description: 'Repository in format owner/repo',
          required: true,
          validation: {
            pattern: '^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$',
          },
        },
      ],
      handler: (async (args: { repo: string }): Promise<ExecutionResult> => {
        const client = getGitHubClient();
        const response = await client.get<GitHubRepo>(`/repos/${args.repo}`);

        if (!response.ok) {
          return handleApiError(response);
        }

        const repo = response.data;
        const output = [
          `Repository: ${repo.full_name}`,
          `URL: ${repo.html_url}`,
          `Description: ${repo.description || 'No description'}`,
          `Visibility: ${repo.private ? 'Private' : 'Public'}`,
          `Language: ${repo.language || 'None'}`,
          `Stars: ${repo.stargazers_count} | Forks: ${repo.forks_count}`,
          `Last updated: ${new Date(repo.updated_at).toLocaleDateString()}`,
        ].join('\n');

        return ok(output, { repo });
      }) as ToolHandler,
    },

    // ========================================
    // Issue Tools
    // ========================================
    {
      name: 'list-issues',
      description: 'List issues in a repository',
      parameters: [
        {
          name: 'repo',
          paramType: 'string',
          description: 'Repository in format owner/repo',
          required: true,
          validation: {
            pattern: '^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$',
          },
        },
        {
          name: 'state',
          paramType: 'string',
          description: 'Issue state: open, closed, all',
          required: false,
          defaultValue: 'open',
          validation: {
            enum: ['open', 'closed', 'all'],
          },
        },
        {
          name: 'limit',
          paramType: 'number',
          description: 'Maximum number of issues to return',
          required: false,
          defaultValue: '20',
          validation: {
            minimum: 1,
            maximum: 100,
          },
        },
      ],
      handler: (async (args: {
        repo: string;
        state: string;
        limit: number;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();

        const params = new URLSearchParams({
          state: args.state,
          per_page: String(args.limit),
        });

        const response = await client.get<GitHubIssue[]>(
          `/repos/${args.repo}/issues?${params}`
        );

        if (!response.ok) {
          return handleApiError(response);
        }

        const issues = response.data.filter(i => !('pull_request' in i));

        if (issues.length === 0) {
          return ok(`No ${args.state} issues found in ${args.repo}.`);
        }

        const output = issues.map(issue => {
          const state = issue.state === 'open' ? '🟢' : '🔴';
          const labels = issue.labels.length > 0
            ? `[${issue.labels.map(l => l.name).join(', ')}]`
            : '';
          return `${state} #${issue.number}: ${issue.title} ${labels}`;
        }).join('\n');

        return ok(
          `Found ${issues.length} ${args.state} issues:\n\n${output}`,
          { issues: issues.map(i => ({ number: i.number, title: i.title, url: i.html_url })) }
        );
      }) as ToolHandler,
    },

    {
      name: 'create-issue',
      description: 'Create a new issue in a repository',
      parameters: [
        {
          name: 'repo',
          paramType: 'string',
          description: 'Repository in format owner/repo',
          required: true,
          validation: {
            pattern: '^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$',
          },
        },
        {
          name: 'title',
          paramType: 'string',
          description: 'Issue title',
          required: true,
          validation: {
            minLength: 1,
            maxLength: 256,
          },
        },
        {
          name: 'body',
          paramType: 'string',
          description: 'Issue body (markdown supported)',
          required: false,
        },
        {
          name: 'labels',
          paramType: 'array',
          description: 'Comma-separated list of labels',
          required: false,
        },
      ],
      handler: (async (args: {
        repo: string;
        title: string;
        body?: string;
        labels?: string[];
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();

        const response = await client.post<GitHubIssue>(
          `/repos/${args.repo}/issues`,
          {
            title: args.title,
            body: args.body,
            labels: args.labels,
          }
        );

        if (!response.ok) {
          return handleApiError(response);
        }

        const issue = response.data;
        return ok(
          `Created issue #${issue.number}: ${issue.title}\nURL: ${issue.html_url}`,
          { issue: { number: issue.number, url: issue.html_url } }
        );
      }) as ToolHandler,
    },

    // ========================================
    // Pull Request Tools
    // ========================================
    {
      name: 'list-prs',
      description: 'List pull requests in a repository',
      parameters: [
        {
          name: 'repo',
          paramType: 'string',
          description: 'Repository in format owner/repo',
          required: true,
          validation: {
            pattern: '^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$',
          },
        },
        {
          name: 'state',
          paramType: 'string',
          description: 'PR state: open, closed, all',
          required: false,
          defaultValue: 'open',
          validation: {
            enum: ['open', 'closed', 'all'],
          },
        },
        {
          name: 'limit',
          paramType: 'number',
          description: 'Maximum number of PRs to return',
          required: false,
          defaultValue: '20',
          validation: {
            minimum: 1,
            maximum: 100,
          },
        },
      ],
      handler: (async (args: {
        repo: string;
        state: string;
        limit: number;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();

        const params = new URLSearchParams({
          state: args.state,
          per_page: String(args.limit),
        });

        const response = await client.get<GitHubPullRequest[]>(
          `/repos/${args.repo}/pulls?${params}`
        );

        if (!response.ok) {
          return handleApiError(response);
        }

        const prs = response.data;

        if (prs.length === 0) {
          return ok(`No ${args.state} pull requests found in ${args.repo}.`);
        }

        const output = prs.map(pr => {
          const state = pr.state === 'open' ? '🟢' : (pr.draft ? '⚪' : '🟣');
          const draft = pr.draft ? '[DRAFT] ' : '';
          return `${state} #${pr.number}: ${draft}${pr.title} (${pr.head.ref} → ${pr.base.ref})`;
        }).join('\n');

        return ok(
          `Found ${prs.length} ${args.state} pull requests:\n\n${output}`,
          { prs: prs.map(p => ({ number: p.number, title: p.title, url: p.html_url })) }
        );
      }) as ToolHandler,
    },

    {
      name: 'create-pr',
      description: 'Create a new pull request',
      parameters: [
        {
          name: 'repo',
          paramType: 'string',
          description: 'Repository in format owner/repo',
          required: true,
          validation: {
            pattern: '^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$',
          },
        },
        {
          name: 'title',
          paramType: 'string',
          description: 'Pull request title',
          required: true,
          validation: {
            minLength: 1,
            maxLength: 256,
          },
        },
        {
          name: 'head',
          paramType: 'string',
          description: 'Branch containing your changes',
          required: true,
        },
        {
          name: 'base',
          paramType: 'string',
          description: 'Branch to merge into',
          required: false,
          defaultValue: 'main',
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
        draft: boolean;
      }): Promise<ExecutionResult> => {
        const client = getGitHubClient();

        const response = await client.post<GitHubPullRequest>(
          `/repos/${args.repo}/pulls`,
          {
            title: args.title,
            head: args.head,
            base: args.base,
            body: args.body,
            draft: args.draft,
          }
        );

        if (!response.ok) {
          return handleApiError(response);
        }

        const pr = response.data;
        const draftText = pr.draft ? ' (draft)' : '';
        return ok(
          `Created PR #${pr.number}${draftText}: ${pr.title}\nURL: ${pr.html_url}`,
          { pr: { number: pr.number, url: pr.html_url } }
        );
      }) as ToolHandler,
    },
  ],
});
