/**
 * CircleCI Skill - CI/CD pipeline management
 *
 * Provides access to CircleCI API for pipelines, workflows, and jobs.
 *
 * Setup:
 *   export SKILL_CIRCLECI_TOKEN=your_api_token
 */

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

function getCircleCIClient() {
  return createAuthenticatedClient({
    baseUrl: 'https://circleci.com/api/v2',
    authType: 'bearer',
    tokenKey: 'CIRCLECI_TOKEN',
    headers: { 'Content-Type': 'application/json' },
  });
}

function handleError(response: { ok: boolean; status: number; statusText: string }): ExecutionResult {
  if (response.status === 401) return err('Authentication failed. Check your CIRCLECI_TOKEN.', errors.auth());
  if (response.status === 404) return err('Resource not found.');
  return err(`CircleCI API error: ${response.status} ${response.statusText}`);
}

export default defineSkill({
  metadata: {
    name: 'circleci-skill',
    version: '1.0.0',
    description: 'CircleCI CI/CD pipeline management',
    author: 'Skill Engine Team',
    tags: ['circleci', 'ci-cd', 'pipelines', 'devops'],
  },
  tools: [
    // User
    {
      name: 'me',
      description: 'Get current user information',
      parameters: [],
      handler: (async (): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const response = await client.get<{ name: string; login: string; id: string }>('/me');
        if (!response.ok) return handleError(response);
        return ok(`User: ${response.data.login}\nName: ${response.data.name}\nID: ${response.data.id}`, { user: response.data });
      }) as ToolHandler,
    },
    // Project
    {
      name: 'project-get',
      description: 'Get project information',
      parameters: [
        { name: 'project_slug', paramType: 'string', description: 'Project slug (e.g., gh/org/repo or bb/org/repo)', required: true },
      ],
      handler: (async (args: { project_slug: string }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const response = await client.get(`/project/${args.project_slug}`);
        if (!response.ok) return handleError(response);
        return ok(JSON.stringify(response.data, null, 2), { project: response.data });
      }) as ToolHandler,
    },
    // Pipelines
    {
      name: 'pipeline-list',
      description: 'List pipelines for a project',
      parameters: [
        { name: 'project_slug', paramType: 'string', description: 'Project slug', required: true },
        { name: 'branch', paramType: 'string', description: 'Filter by branch', required: false },
        { name: 'page_token', paramType: 'string', description: 'Page token for pagination', required: false },
      ],
      handler: (async (args: { project_slug: string; branch?: string; page_token?: string }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const params = new URLSearchParams();
        if (args.branch) params.append('branch', args.branch);
        if (args.page_token) params.append('page-token', args.page_token);
        const query = params.toString() ? `?${params}` : '';
        const response = await client.get(`/project/${args.project_slug}/pipeline${query}`);
        if (!response.ok) return handleError(response);
        return ok(JSON.stringify(response.data, null, 2), response.data);
      }) as ToolHandler,
    },
    {
      name: 'pipeline-get',
      description: 'Get pipeline by ID',
      parameters: [
        { name: 'pipeline_id', paramType: 'string', description: 'Pipeline ID', required: true },
      ],
      handler: (async (args: { pipeline_id: string }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const response = await client.get(`/pipeline/${args.pipeline_id}`);
        if (!response.ok) return handleError(response);
        return ok(JSON.stringify(response.data, null, 2), { pipeline: response.data });
      }) as ToolHandler,
    },
    {
      name: 'pipeline-trigger',
      description: 'Trigger a new pipeline',
      parameters: [
        { name: 'project_slug', paramType: 'string', description: 'Project slug', required: true },
        { name: 'branch', paramType: 'string', description: 'Branch to build', required: false },
        { name: 'tag', paramType: 'string', description: 'Tag to build', required: false },
        { name: 'parameters', paramType: 'string', description: 'Pipeline parameters as JSON', required: false },
      ],
      handler: (async (args: { project_slug: string; branch?: string; tag?: string; parameters?: string }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const body: Record<string, unknown> = {};
        if (args.branch) body.branch = args.branch;
        if (args.tag) body.tag = args.tag;
        if (args.parameters) body.parameters = JSON.parse(args.parameters);
        const response = await client.post(`/project/${args.project_slug}/pipeline`, body);
        if (!response.ok) return handleError(response);
        return ok(`Pipeline triggered!\nID: ${(response.data as { id: string }).id}`, { pipeline: response.data });
      }) as ToolHandler,
    },
    // Workflows
    {
      name: 'workflow-get',
      description: 'Get workflow by ID',
      parameters: [
        { name: 'workflow_id', paramType: 'string', description: 'Workflow ID', required: true },
      ],
      handler: (async (args: { workflow_id: string }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const response = await client.get(`/workflow/${args.workflow_id}`);
        if (!response.ok) return handleError(response);
        return ok(JSON.stringify(response.data, null, 2), { workflow: response.data });
      }) as ToolHandler,
    },
    {
      name: 'workflow-cancel',
      description: 'Cancel a running workflow',
      parameters: [
        { name: 'workflow_id', paramType: 'string', description: 'Workflow ID', required: true },
      ],
      handler: (async (args: { workflow_id: string }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const response = await client.post(`/workflow/${args.workflow_id}/cancel`, {});
        if (!response.ok) return handleError(response);
        return ok(`Workflow ${args.workflow_id} cancelled.`);
      }) as ToolHandler,
    },
    {
      name: 'workflow-rerun',
      description: 'Rerun a workflow',
      parameters: [
        { name: 'workflow_id', paramType: 'string', description: 'Workflow ID', required: true },
        { name: 'from_failed', paramType: 'boolean', description: 'Rerun from failed jobs only', required: false, defaultValue: 'false' },
      ],
      handler: (async (args: { workflow_id: string; from_failed?: boolean }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const body = args.from_failed ? { from_failed: true } : {};
        const response = await client.post(`/workflow/${args.workflow_id}/rerun`, body);
        if (!response.ok) return handleError(response);
        return ok(`Workflow ${args.workflow_id} rerun triggered.`, response.data);
      }) as ToolHandler,
    },
    {
      name: 'workflow-jobs',
      description: 'List jobs in a workflow',
      parameters: [
        { name: 'workflow_id', paramType: 'string', description: 'Workflow ID', required: true },
      ],
      handler: (async (args: { workflow_id: string }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const response = await client.get(`/workflow/${args.workflow_id}/job`);
        if (!response.ok) return handleError(response);
        return ok(JSON.stringify(response.data, null, 2), response.data);
      }) as ToolHandler,
    },
    // Jobs
    {
      name: 'job-get',
      description: 'Get job details',
      parameters: [
        { name: 'project_slug', paramType: 'string', description: 'Project slug', required: true },
        { name: 'job_number', paramType: 'number', description: 'Job number', required: true },
      ],
      handler: (async (args: { project_slug: string; job_number: number }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const response = await client.get(`/project/${args.project_slug}/job/${args.job_number}`);
        if (!response.ok) return handleError(response);
        return ok(JSON.stringify(response.data, null, 2), { job: response.data });
      }) as ToolHandler,
    },
    {
      name: 'job-cancel',
      description: 'Cancel a running job',
      parameters: [
        { name: 'project_slug', paramType: 'string', description: 'Project slug', required: true },
        { name: 'job_number', paramType: 'number', description: 'Job number', required: true },
      ],
      handler: (async (args: { project_slug: string; job_number: number }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const response = await client.post(`/project/${args.project_slug}/job/${args.job_number}/cancel`, {});
        if (!response.ok) return handleError(response);
        return ok(`Job ${args.job_number} cancelled.`);
      }) as ToolHandler,
    },
    {
      name: 'job-artifacts',
      description: 'List artifacts from a job',
      parameters: [
        { name: 'project_slug', paramType: 'string', description: 'Project slug', required: true },
        { name: 'job_number', paramType: 'number', description: 'Job number', required: true },
      ],
      handler: (async (args: { project_slug: string; job_number: number }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const response = await client.get(`/project/${args.project_slug}/${args.job_number}/artifacts`);
        if (!response.ok) return handleError(response);
        return ok(JSON.stringify(response.data, null, 2), response.data);
      }) as ToolHandler,
    },
    // Insights
    {
      name: 'insights-summary',
      description: 'Get workflow insights summary',
      parameters: [
        { name: 'project_slug', paramType: 'string', description: 'Project slug', required: true },
        { name: 'workflow_name', paramType: 'string', description: 'Workflow name', required: true },
        { name: 'branch', paramType: 'string', description: 'Branch name', required: false },
      ],
      handler: (async (args: { project_slug: string; workflow_name: string; branch?: string }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const params = args.branch ? `?branch=${args.branch}` : '';
        const response = await client.get(`/insights/${args.project_slug}/workflows/${args.workflow_name}/summary${params}`);
        if (!response.ok) return handleError(response);
        return ok(JSON.stringify(response.data, null, 2), response.data);
      }) as ToolHandler,
    },
    {
      name: 'insights-jobs',
      description: 'Get job insights for a workflow',
      parameters: [
        { name: 'project_slug', paramType: 'string', description: 'Project slug', required: true },
        { name: 'workflow_name', paramType: 'string', description: 'Workflow name', required: true },
      ],
      handler: (async (args: { project_slug: string; workflow_name: string }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const response = await client.get(`/insights/${args.project_slug}/workflows/${args.workflow_name}/jobs`);
        if (!response.ok) return handleError(response);
        return ok(JSON.stringify(response.data, null, 2), response.data);
      }) as ToolHandler,
    },
    // Context/Config
    {
      name: 'context-list',
      description: 'List contexts for an organization',
      parameters: [
        { name: 'owner_slug', paramType: 'string', description: 'Organization slug (e.g., gh/org)', required: true },
      ],
      handler: (async (args: { owner_slug: string }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const response = await client.get(`/context?owner-slug=${args.owner_slug}`);
        if (!response.ok) return handleError(response);
        return ok(JSON.stringify(response.data, null, 2), response.data);
      }) as ToolHandler,
    },
    {
      name: 'context-get',
      description: 'Get context by ID',
      parameters: [
        { name: 'context_id', paramType: 'string', description: 'Context ID', required: true },
      ],
      handler: (async (args: { context_id: string }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const response = await client.get(`/context/${args.context_id}`);
        if (!response.ok) return handleError(response);
        return ok(JSON.stringify(response.data, null, 2), { context: response.data });
      }) as ToolHandler,
    },
    {
      name: 'env-var-list',
      description: 'List environment variables for a project',
      parameters: [
        { name: 'project_slug', paramType: 'string', description: 'Project slug', required: true },
      ],
      handler: (async (args: { project_slug: string }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const response = await client.get(`/project/${args.project_slug}/envvar`);
        if (!response.ok) return handleError(response);
        return ok(JSON.stringify(response.data, null, 2), response.data);
      }) as ToolHandler,
    },
    {
      name: 'env-var-create',
      description: 'Create or update an environment variable',
      parameters: [
        { name: 'project_slug', paramType: 'string', description: 'Project slug', required: true },
        { name: 'name', paramType: 'string', description: 'Variable name', required: true },
        { name: 'value', paramType: 'string', description: 'Variable value', required: true },
      ],
      handler: (async (args: { project_slug: string; name: string; value: string }): Promise<ExecutionResult> => {
        const client = getCircleCIClient();
        const response = await client.post(`/project/${args.project_slug}/envvar`, { name: args.name, value: args.value });
        if (!response.ok) return handleError(response);
        return ok(`Environment variable ${args.name} created/updated.`, { envvar: response.data });
      }) as ToolHandler,
    },
  ],
});
