/**
 * Jira Skill - Project management and issue tracking
 *
 * Provides access to Jira REST API for issues, projects, sprints, and boards.
 *
 * Setup:
 *   export SKILL_JIRA_URL=https://your-domain.atlassian.net
 *   export SKILL_JIRA_EMAIL=your-email@example.com
 *   export SKILL_JIRA_TOKEN=your_api_token
 */

import {
  defineSkill,
  getConfig,
  ok,
  err,
  errors,
  httpRequest,
  type ExecutionResult,
  type ToolHandler,
} from '@skill-engine/sdk';

function getJiraAuth(): string {
  const email = getConfig('JIRA_EMAIL') || '';
  const token = getConfig('JIRA_TOKEN') || '';
  return Buffer.from(`${email}:${token}`).toString('base64');
}

function getJiraUrl(): string {
  return getConfig('JIRA_URL') || '';
}

async function jiraRequest(
  method: string,
  endpoint: string,
  body?: unknown
): Promise<ExecutionResult> {
  try {
    const baseUrl = getJiraUrl();
    if (!baseUrl) return err('JIRA_URL not configured');

    const response = await httpRequest({
      method,
      url: `${baseUrl}/rest/api/3${endpoint}`,
      headers: {
        'Authorization': `Basic ${getJiraAuth()}`,
        'Accept': 'application/json',
        'Content-Type': 'application/json',
      },
      body: body ? JSON.stringify(body) : undefined,
    });

    if (!response.ok) {
      if (response.status === 401) return err('Authentication failed. Check JIRA_EMAIL and JIRA_TOKEN.', errors.auth());
      if (response.status === 404) return err('Resource not found.');
      return err(`Jira API error: ${response.status} ${response.statusText}`);
    }

    const data = response.body ? JSON.parse(response.body) : {};
    return ok(JSON.stringify(data, null, 2), { data });
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : String(e);
    return err(`Request failed: ${message}`);
  }
}

async function agileRequest(
  method: string,
  endpoint: string,
  body?: unknown
): Promise<ExecutionResult> {
  try {
    const baseUrl = getJiraUrl();
    if (!baseUrl) return err('JIRA_URL not configured');

    const response = await httpRequest({
      method,
      url: `${baseUrl}/rest/agile/1.0${endpoint}`,
      headers: {
        'Authorization': `Basic ${getJiraAuth()}`,
        'Accept': 'application/json',
        'Content-Type': 'application/json',
      },
      body: body ? JSON.stringify(body) : undefined,
    });

    if (!response.ok) {
      return err(`Jira Agile API error: ${response.status} ${response.statusText}`);
    }

    const data = response.body ? JSON.parse(response.body) : {};
    return ok(JSON.stringify(data, null, 2), { data });
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : String(e);
    return err(`Request failed: ${message}`);
  }
}

export default defineSkill({
  metadata: {
    name: 'jira-skill',
    version: '1.0.0',
    description: 'Jira project management and issue tracking',
    author: 'Skill Engine Team',
    tags: ['jira', 'project-management', 'issues', 'agile'],
  },
  tools: [
    // User
    { name: 'myself', description: 'Get current user information', parameters: [],
      handler: (async (): Promise<ExecutionResult> => jiraRequest('GET', '/myself')) as ToolHandler },
    { name: 'user-search', description: 'Search for users', parameters: [
      { name: 'query', paramType: 'string', description: 'Search query', required: true },
      { name: 'maxResults', paramType: 'number', description: 'Max results', required: false, defaultValue: '10' },
    ],
      handler: (async (args: { query: string; maxResults?: number }): Promise<ExecutionResult> =>
        jiraRequest('GET', `/user/search?query=${encodeURIComponent(args.query)}&maxResults=${args.maxResults || 10}`)) as ToolHandler },

    // Issues - Search
    { name: 'search', description: 'Search issues using JQL', parameters: [
      { name: 'jql', paramType: 'string', description: 'JQL query', required: true },
      { name: 'fields', paramType: 'string', description: 'Fields to return (comma-separated)', required: false },
      { name: 'maxResults', paramType: 'number', description: 'Max results', required: false, defaultValue: '50' },
      { name: 'startAt', paramType: 'number', description: 'Start at index', required: false, defaultValue: '0' },
    ],
      handler: (async (args: { jql: string; fields?: string; maxResults?: number; startAt?: number }): Promise<ExecutionResult> => {
        const params = new URLSearchParams();
        params.append('jql', args.jql);
        if (args.fields) params.append('fields', args.fields);
        params.append('maxResults', String(args.maxResults || 50));
        params.append('startAt', String(args.startAt || 0));
        return jiraRequest('GET', `/search?${params}`);
      }) as ToolHandler },

    // Issues - CRUD
    { name: 'issue-get', description: 'Get issue details', parameters: [
      { name: 'key', paramType: 'string', description: 'Issue key (e.g., PROJ-123)', required: true },
      { name: 'fields', paramType: 'string', description: 'Fields to return', required: false },
    ],
      handler: (async (args: { key: string; fields?: string }): Promise<ExecutionResult> => {
        const query = args.fields ? `?fields=${args.fields}` : '';
        return jiraRequest('GET', `/issue/${args.key}${query}`);
      }) as ToolHandler },

    { name: 'issue-create', description: 'Create a new issue', parameters: [
      { name: 'project', paramType: 'string', description: 'Project key', required: true },
      { name: 'summary', paramType: 'string', description: 'Issue summary', required: true },
      { name: 'issue_type', paramType: 'string', description: 'Issue type (Bug, Story, Task, etc.)', required: true },
      { name: 'description', paramType: 'string', description: 'Issue description', required: false },
      { name: 'priority', paramType: 'string', description: 'Priority name', required: false },
      { name: 'assignee', paramType: 'string', description: 'Assignee account ID', required: false },
      { name: 'labels', paramType: 'string', description: 'Labels (comma-separated)', required: false },
    ],
      handler: (async (args: { project: string; summary: string; issue_type: string; description?: string; priority?: string; assignee?: string; labels?: string }): Promise<ExecutionResult> => {
        const fields: Record<string, unknown> = {
          project: { key: args.project },
          summary: args.summary,
          issuetype: { name: args.issue_type },
        };
        if (args.description) fields.description = { type: 'doc', version: 1, content: [{ type: 'paragraph', content: [{ type: 'text', text: args.description }] }] };
        if (args.priority) fields.priority = { name: args.priority };
        if (args.assignee) fields.assignee = { accountId: args.assignee };
        if (args.labels) fields.labels = args.labels.split(',').map(l => l.trim());
        return jiraRequest('POST', '/issue', { fields });
      }) as ToolHandler },

    { name: 'issue-update', description: 'Update an issue', parameters: [
      { name: 'key', paramType: 'string', description: 'Issue key', required: true },
      { name: 'summary', paramType: 'string', description: 'New summary', required: false },
      { name: 'description', paramType: 'string', description: 'New description', required: false },
      { name: 'priority', paramType: 'string', description: 'New priority', required: false },
      { name: 'assignee', paramType: 'string', description: 'New assignee account ID', required: false },
    ],
      handler: (async (args: { key: string; summary?: string; description?: string; priority?: string; assignee?: string }): Promise<ExecutionResult> => {
        const fields: Record<string, unknown> = {};
        if (args.summary) fields.summary = args.summary;
        if (args.description) fields.description = { type: 'doc', version: 1, content: [{ type: 'paragraph', content: [{ type: 'text', text: args.description }] }] };
        if (args.priority) fields.priority = { name: args.priority };
        if (args.assignee) fields.assignee = { accountId: args.assignee };
        return jiraRequest('PUT', `/issue/${args.key}`, { fields });
      }) as ToolHandler },

    { name: 'issue-delete', description: 'Delete an issue', parameters: [
      { name: 'key', paramType: 'string', description: 'Issue key', required: true },
    ],
      handler: (async (args: { key: string }): Promise<ExecutionResult> => jiraRequest('DELETE', `/issue/${args.key}`)) as ToolHandler },

    { name: 'issue-assign', description: 'Assign an issue', parameters: [
      { name: 'key', paramType: 'string', description: 'Issue key', required: true },
      { name: 'assignee', paramType: 'string', description: 'Assignee account ID (or -1 for unassigned)', required: true },
    ],
      handler: (async (args: { key: string; assignee: string }): Promise<ExecutionResult> =>
        jiraRequest('PUT', `/issue/${args.key}/assignee`, { accountId: args.assignee === '-1' ? null : args.assignee })) as ToolHandler },

    { name: 'transitions', description: 'Get available transitions for an issue', parameters: [
      { name: 'key', paramType: 'string', description: 'Issue key', required: true },
    ],
      handler: (async (args: { key: string }): Promise<ExecutionResult> => jiraRequest('GET', `/issue/${args.key}/transitions`)) as ToolHandler },

    { name: 'issue-transition', description: 'Transition an issue to a new status', parameters: [
      { name: 'key', paramType: 'string', description: 'Issue key', required: true },
      { name: 'transition_id', paramType: 'string', description: 'Transition ID', required: true },
    ],
      handler: (async (args: { key: string; transition_id: string }): Promise<ExecutionResult> =>
        jiraRequest('POST', `/issue/${args.key}/transitions`, { transition: { id: args.transition_id } })) as ToolHandler },

    // Comments
    { name: 'comment-list', description: 'List comments on an issue', parameters: [
      { name: 'key', paramType: 'string', description: 'Issue key', required: true },
    ],
      handler: (async (args: { key: string }): Promise<ExecutionResult> => jiraRequest('GET', `/issue/${args.key}/comment`)) as ToolHandler },

    { name: 'comment-add', description: 'Add a comment to an issue', parameters: [
      { name: 'key', paramType: 'string', description: 'Issue key', required: true },
      { name: 'body', paramType: 'string', description: 'Comment text', required: true },
    ],
      handler: (async (args: { key: string; body: string }): Promise<ExecutionResult> =>
        jiraRequest('POST', `/issue/${args.key}/comment`, { body: { type: 'doc', version: 1, content: [{ type: 'paragraph', content: [{ type: 'text', text: args.body }] }] } })) as ToolHandler },

    { name: 'comment-update', description: 'Update a comment', parameters: [
      { name: 'key', paramType: 'string', description: 'Issue key', required: true },
      { name: 'comment_id', paramType: 'string', description: 'Comment ID', required: true },
      { name: 'body', paramType: 'string', description: 'New comment text', required: true },
    ],
      handler: (async (args: { key: string; comment_id: string; body: string }): Promise<ExecutionResult> =>
        jiraRequest('PUT', `/issue/${args.key}/comment/${args.comment_id}`, { body: { type: 'doc', version: 1, content: [{ type: 'paragraph', content: [{ type: 'text', text: args.body }] }] } })) as ToolHandler },

    // Worklogs
    { name: 'worklog-list', description: 'List worklogs on an issue', parameters: [
      { name: 'key', paramType: 'string', description: 'Issue key', required: true },
    ],
      handler: (async (args: { key: string }): Promise<ExecutionResult> => jiraRequest('GET', `/issue/${args.key}/worklog`)) as ToolHandler },

    { name: 'worklog-add', description: 'Add a worklog to an issue', parameters: [
      { name: 'key', paramType: 'string', description: 'Issue key', required: true },
      { name: 'time_spent', paramType: 'string', description: 'Time spent (e.g., 2h, 30m)', required: true },
      { name: 'comment', paramType: 'string', description: 'Worklog comment', required: false },
      { name: 'started', paramType: 'string', description: 'Start time (ISO format)', required: false },
    ],
      handler: (async (args: { key: string; time_spent: string; comment?: string; started?: string }): Promise<ExecutionResult> => {
        const body: Record<string, unknown> = { timeSpent: args.time_spent };
        if (args.comment) body.comment = { type: 'doc', version: 1, content: [{ type: 'paragraph', content: [{ type: 'text', text: args.comment }] }] };
        if (args.started) body.started = args.started;
        return jiraRequest('POST', `/issue/${args.key}/worklog`, body);
      }) as ToolHandler },

    // Projects
    { name: 'project-list', description: 'List projects', parameters: [
      { name: 'maxResults', paramType: 'number', description: 'Max results', required: false, defaultValue: '50' },
    ],
      handler: (async (args: { maxResults?: number }): Promise<ExecutionResult> =>
        jiraRequest('GET', `/project/search?maxResults=${args.maxResults || 50}`)) as ToolHandler },

    { name: 'project-get', description: 'Get project details', parameters: [
      { name: 'key', paramType: 'string', description: 'Project key', required: true },
    ],
      handler: (async (args: { key: string }): Promise<ExecutionResult> => jiraRequest('GET', `/project/${args.key}`)) as ToolHandler },

    // Agile - Boards
    { name: 'board-list', description: 'List boards', parameters: [
      { name: 'project', paramType: 'string', description: 'Project key to filter', required: false },
      { name: 'type', paramType: 'string', description: 'Board type (scrum, kanban)', required: false },
    ],
      handler: (async (args: { project?: string; type?: string }): Promise<ExecutionResult> => {
        const params = new URLSearchParams();
        if (args.project) params.append('projectKeyOrId', args.project);
        if (args.type) params.append('type', args.type);
        return agileRequest('GET', `/board?${params}`);
      }) as ToolHandler },

    { name: 'board-get', description: 'Get board details', parameters: [
      { name: 'board_id', paramType: 'number', description: 'Board ID', required: true },
    ],
      handler: (async (args: { board_id: number }): Promise<ExecutionResult> => agileRequest('GET', `/board/${args.board_id}`)) as ToolHandler },

    // Agile - Sprints
    { name: 'sprint-list', description: 'List sprints for a board', parameters: [
      { name: 'board_id', paramType: 'number', description: 'Board ID', required: true },
      { name: 'state', paramType: 'string', description: 'Sprint state (active, closed, future)', required: false },
    ],
      handler: (async (args: { board_id: number; state?: string }): Promise<ExecutionResult> => {
        const query = args.state ? `?state=${args.state}` : '';
        return agileRequest('GET', `/board/${args.board_id}/sprint${query}`);
      }) as ToolHandler },

    { name: 'sprint-get', description: 'Get sprint details', parameters: [
      { name: 'sprint_id', paramType: 'number', description: 'Sprint ID', required: true },
    ],
      handler: (async (args: { sprint_id: number }): Promise<ExecutionResult> => agileRequest('GET', `/sprint/${args.sprint_id}`)) as ToolHandler },

    // Meta
    { name: 'priorities', description: 'Get available priorities', parameters: [],
      handler: (async (): Promise<ExecutionResult> => jiraRequest('GET', '/priority')) as ToolHandler },

    { name: 'issue-types', description: 'Get available issue types', parameters: [
      { name: 'project', paramType: 'string', description: 'Project key', required: false },
    ],
      handler: (async (args: { project?: string }): Promise<ExecutionResult> => {
        if (args.project) return jiraRequest('GET', `/issue/createmeta?projectKeys=${args.project}&expand=projects.issuetypes`);
        return jiraRequest('GET', '/issuetype');
      }) as ToolHandler },
  ],
});
