import {
  defineSkill,
  getConfig,
  ok,
  err,
  errors,
  createAuthenticatedClient,
  type ExecutionResult,
  type ToolHandler,
} from '@skill-engine/sdk';

interface LinearConfig {
  LINEAR_API_KEY: string;
}

export default defineSkill({
  metadata: {
    name: 'linear',
    version: '1.0.0',
    description: 'Linear issue tracking and project management',
    author: 'Skill Engine',
    tags: ['linear', 'issues', 'project-management', 'agile'],
    homepage: 'https://linear.app/',
  },

  tools: [
    {
      name: 'list-issues',
      description: 'List issues with filtering options',
      parameters: [
        { name: 'team', paramType: 'string', description: 'Team key or ID', required: false },
        { name: 'project', paramType: 'string', description: 'Project name or ID', required: false },
        { name: 'status', paramType: 'string', description: 'Status: backlog, todo, in_progress, done, cancelled', required: false },
        { name: 'assignee', paramType: 'string', description: 'Assignee email or ID', required: false },
        { name: 'limit', paramType: 'number', description: 'Maximum results (default: 50)', required: false },
      ],
      handler: (async (args: { team?: string; project?: string; status?: string; assignee?: string; limit?: number }): Promise<ExecutionResult> => {
        try {
          const client = getLinearClient();

          let filter = '';
          const filters: string[] = [];
          if (args.team) filters.push(`team: { key: { eq: "${args.team}" } }`);
          if (args.status) filters.push(`state: { name: { containsIgnoreCase: "${args.status}" } }`);
          if (args.assignee) filters.push(`assignee: { email: { eq: "${args.assignee}" } }`);
          if (filters.length > 0) filter = `filter: { ${filters.join(', ')} }`;

          const query = `
            query {
              issues(first: ${args.limit || 50}, ${filter}) {
                nodes {
                  id
                  identifier
                  title
                  description
                  priority
                  state { name }
                  assignee { name email }
                  project { name }
                  team { key name }
                  createdAt
                  updatedAt
                }
              }
            }
          `;

          const response = await client.post<any>('/graphql', { query });
          if (!response.ok) {
            return err(`Failed to list issues: ${response.status}`, errors.service('Linear', String(response.status)));
          }

          const issues = response.data?.data?.issues?.nodes || [];
          return ok(JSON.stringify({ issues }, null, 2), { data: issues });
        } catch (error: any) {
          return err(`Error listing issues: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'get-issue',
      description: 'Get detailed information about a specific issue',
      parameters: [
        { name: 'id', paramType: 'string', description: 'Issue ID (e.g., ENG-123)', required: true },
      ],
      handler: (async (args: { id: string }): Promise<ExecutionResult> => {
        try {
          const client = getLinearClient();

          const query = `
            query {
              issue(id: "${args.id}") {
                id
                identifier
                title
                description
                priority
                estimate
                state { name }
                assignee { name email }
                project { name }
                team { key name }
                cycle { name number }
                labels { nodes { name } }
                comments { nodes { body user { name } createdAt } }
                createdAt
                updatedAt
              }
            }
          `;

          const response = await client.post<any>('/graphql', { query });
          if (!response.ok || response.data?.errors) {
            return err(`Issue not found: ${args.id}`, errors.notFound(`Issue ${args.id}`));
          }

          const issue = response.data?.data?.issue;
          return ok(JSON.stringify(issue, null, 2), { data: issue });
        } catch (error: any) {
          return err(`Error getting issue: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'create-issue',
      description: 'Create a new issue',
      parameters: [
        { name: 'title', paramType: 'string', description: 'Issue title', required: true },
        { name: 'team', paramType: 'string', description: 'Team key', required: true },
        { name: 'description', paramType: 'string', description: 'Issue description', required: false },
        { name: 'priority', paramType: 'number', description: 'Priority (0=None, 1=Urgent, 2=High, 3=Medium, 4=Low)', required: false },
        { name: 'assignee', paramType: 'string', description: 'Assignee email', required: false },
        { name: 'labels', paramType: 'string', description: 'Comma-separated label names', required: false },
        { name: 'estimate', paramType: 'number', description: 'Point estimate', required: false },
      ],
      handler: (async (args: { title: string; team: string; description?: string; priority?: number; assignee?: string; labels?: string; estimate?: number }): Promise<ExecutionResult> => {
        try {
          const client = getLinearClient();

          // First get team ID
          const teamQuery = `query { teams(filter: { key: { eq: "${args.team}" } }) { nodes { id } } }`;
          const teamResponse = await client.post<any>('/graphql', { query: teamQuery });
          const teamId = teamResponse.data?.data?.teams?.nodes?.[0]?.id;
          if (!teamId) {
            return err(`Team not found: ${args.team}`, errors.notFound(`Team ${args.team}`));
          }

          const mutation = `
            mutation {
              issueCreate(input: {
                title: "${args.title}"
                teamId: "${teamId}"
                ${args.description ? `description: "${args.description.replace(/"/g, '\\"')}"` : ''}
                ${args.priority !== undefined ? `priority: ${args.priority}` : ''}
                ${args.estimate !== undefined ? `estimate: ${args.estimate}` : ''}
              }) {
                success
                issue {
                  id
                  identifier
                  title
                  url
                }
              }
            }
          `;

          const response = await client.post<any>('/graphql', { query: mutation });
          if (!response.ok || response.data?.errors) {
            return err(`Failed to create issue: ${JSON.stringify(response.data?.errors)}`, errors.service('Linear', 'Creation failed'));
          }

          const issue = response.data?.data?.issueCreate?.issue;
          return ok(`Issue created: ${issue?.identifier}`, { data: issue });
        } catch (error: any) {
          return err(`Error creating issue: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'update-issue',
      description: 'Update an existing issue',
      parameters: [
        { name: 'id', paramType: 'string', description: 'Issue ID', required: true },
        { name: 'title', paramType: 'string', description: 'New title', required: false },
        { name: 'description', paramType: 'string', description: 'New description', required: false },
        { name: 'status', paramType: 'string', description: 'New status', required: false },
        { name: 'priority', paramType: 'number', description: 'New priority', required: false },
        { name: 'estimate', paramType: 'number', description: 'Point estimate', required: false },
      ],
      handler: (async (args: { id: string; title?: string; description?: string; status?: string; priority?: number; estimate?: number }): Promise<ExecutionResult> => {
        try {
          const client = getLinearClient();

          const updates: string[] = [];
          if (args.title) updates.push(`title: "${args.title}"`);
          if (args.description) updates.push(`description: "${args.description.replace(/"/g, '\\"')}"`);
          if (args.priority !== undefined) updates.push(`priority: ${args.priority}`);
          if (args.estimate !== undefined) updates.push(`estimate: ${args.estimate}`);

          if (updates.length === 0) {
            return err('No updates provided');
          }

          const mutation = `
            mutation {
              issueUpdate(id: "${args.id}", input: { ${updates.join(', ')} }) {
                success
                issue {
                  id
                  identifier
                  title
                }
              }
            }
          `;

          const response = await client.post<any>('/graphql', { query: mutation });
          if (!response.ok || response.data?.errors) {
            return err(`Failed to update issue: ${JSON.stringify(response.data?.errors)}`, errors.service('Linear', 'Update failed'));
          }

          const issue = response.data?.data?.issueUpdate?.issue;
          return ok(`Issue ${issue?.identifier} updated`, { data: issue });
        } catch (error: any) {
          return err(`Error updating issue: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'add-comment',
      description: 'Add a comment to an issue',
      parameters: [
        { name: 'issue_id', paramType: 'string', description: 'Issue ID', required: true },
        { name: 'body', paramType: 'string', description: 'Comment content (Markdown supported)', required: true },
      ],
      handler: (async (args: { issue_id: string; body: string }): Promise<ExecutionResult> => {
        try {
          const client = getLinearClient();

          const mutation = `
            mutation {
              commentCreate(input: {
                issueId: "${args.issue_id}"
                body: "${args.body.replace(/"/g, '\\"').replace(/\n/g, '\\n')}"
              }) {
                success
                comment {
                  id
                  body
                }
              }
            }
          `;

          const response = await client.post<any>('/graphql', { query: mutation });
          if (!response.ok || response.data?.errors) {
            return err(`Failed to add comment: ${JSON.stringify(response.data?.errors)}`, errors.service('Linear', 'Comment failed'));
          }

          return ok(`Comment added to issue ${args.issue_id}`, { data: response.data?.data?.commentCreate?.comment });
        } catch (error: any) {
          return err(`Error adding comment: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'list-projects',
      description: 'List projects in a team',
      parameters: [
        { name: 'team', paramType: 'string', description: 'Team key', required: false },
        { name: 'status', paramType: 'string', description: 'Filter: planned, started, paused, completed, cancelled', required: false },
      ],
      handler: (async (args: { team?: string; status?: string }): Promise<ExecutionResult> => {
        try {
          const client = getLinearClient();

          const filters: string[] = [];
          if (args.status) filters.push(`state: { eq: "${args.status}" }`);

          const query = `
            query {
              projects(${filters.length > 0 ? `filter: { ${filters.join(', ')} }` : ''}) {
                nodes {
                  id
                  name
                  state
                  progress
                  targetDate
                  lead { name }
                  teams { nodes { key name } }
                }
              }
            }
          `;

          const response = await client.post<any>('/graphql', { query });
          if (!response.ok) {
            return err(`Failed to list projects: ${response.status}`, errors.service('Linear', String(response.status)));
          }

          let projects = response.data?.data?.projects?.nodes || [];
          if (args.team) {
            projects = projects.filter((p: any) => p.teams?.nodes?.some((t: any) => t.key === args.team));
          }

          return ok(JSON.stringify({ projects }, null, 2), { data: projects });
        } catch (error: any) {
          return err(`Error listing projects: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'list-cycles',
      description: 'List cycles (sprints) for a team',
      parameters: [
        { name: 'team', paramType: 'string', description: 'Team key', required: true },
        { name: 'filter', paramType: 'string', description: 'Filter: current, upcoming, past', required: false },
      ],
      handler: (async (args: { team: string; filter?: string }): Promise<ExecutionResult> => {
        try {
          const client = getLinearClient();

          const query = `
            query {
              cycles(filter: { team: { key: { eq: "${args.team}" } } }) {
                nodes {
                  id
                  name
                  number
                  startsAt
                  endsAt
                  progress
                  completedIssueCountHistory
                  issueCountHistory
                }
              }
            }
          `;

          const response = await client.post<any>('/graphql', { query });
          if (!response.ok) {
            return err(`Failed to list cycles: ${response.status}`, errors.service('Linear', String(response.status)));
          }

          let cycles = response.data?.data?.cycles?.nodes || [];

          // Filter by current/upcoming/past
          if (args.filter) {
            const now = new Date();
            cycles = cycles.filter((c: any) => {
              const start = new Date(c.startsAt);
              const end = new Date(c.endsAt);
              switch (args.filter) {
                case 'current': return start <= now && end >= now;
                case 'upcoming': return start > now;
                case 'past': return end < now;
                default: return true;
              }
            });
          }

          return ok(JSON.stringify({ cycles }, null, 2), { data: cycles });
        } catch (error: any) {
          return err(`Error listing cycles: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'list-teams',
      description: 'List all teams in the workspace',
      parameters: [],
      handler: (async (): Promise<ExecutionResult> => {
        try {
          const client = getLinearClient();

          const query = `
            query {
              teams {
                nodes {
                  id
                  key
                  name
                  description
                }
              }
            }
          `;

          const response = await client.post<any>('/graphql', { query });
          if (!response.ok) {
            return err(`Failed to list teams: ${response.status}`, errors.service('Linear', String(response.status)));
          }

          const teams = response.data?.data?.teams?.nodes || [];
          return ok(JSON.stringify({ teams }, null, 2), { data: teams });
        } catch (error: any) {
          return err(`Error listing teams: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
  ],

  validateConfig: (config) => {
    if (!config.LINEAR_API_KEY) {
      return { err: 'LINEAR_API_KEY is required' };
    }
    return { ok: null };
  },
});

function getLinearClient() {
  return createAuthenticatedClient({
    baseUrl: 'https://api.linear.app',
    authType: 'bearer',
    tokenKey: 'LINEAR_API_KEY',
    headers: {
      'Content-Type': 'application/json',
    },
  });
}
