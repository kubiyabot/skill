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

interface PagerDutyConfig {
  PAGERDUTY_API_KEY: string;
}

export default defineSkill({
  metadata: {
    name: 'pagerduty',
    version: '1.0.0',
    description: 'PagerDuty incident management and on-call scheduling',
    author: 'Skill Engine',
    tags: ['incidents', 'on-call', 'alerting', 'pagerduty'],
    homepage: 'https://pagerduty.com/',
  },

  tools: [
    {
      name: 'list-incidents',
      description: 'List incidents with optional filtering',
      parameters: [
        { name: 'status', paramType: 'string', description: 'Filter: triggered, acknowledged, resolved', required: false },
        { name: 'urgency', paramType: 'string', description: 'Filter: high, low', required: false },
        { name: 'service', paramType: 'string', description: 'Filter by service ID', required: false },
        { name: 'since', paramType: 'string', description: 'Start date (ISO 8601)', required: false },
        { name: 'until', paramType: 'string', description: 'End date (ISO 8601)', required: false },
        { name: 'limit', paramType: 'number', description: 'Maximum results (default: 25)', required: false },
      ],
      handler: (async (args: { status?: string; urgency?: string; service?: string; since?: string; until?: string; limit?: number }): Promise<ExecutionResult> => {
        try {
          const client = getPagerDutyClient();
          const params = new URLSearchParams();

          if (args.status) {
            args.status.split(',').forEach(s => params.append('statuses[]', s.trim()));
          }
          if (args.urgency) params.append('urgencies[]', args.urgency);
          if (args.service) params.append('service_ids[]', args.service);
          if (args.since) params.append('since', args.since);
          if (args.until) params.append('until', args.until);
          params.append('limit', String(args.limit || 25));

          const response = await client.get<any>(`/incidents?${params.toString()}`);
          if (!response.ok) {
            return err(`Failed to list incidents: ${response.status}`, errors.service('PagerDuty', String(response.status)));
          }

          return ok(JSON.stringify({ incidents: response.data?.incidents }, null, 2), { data: response.data?.incidents });
        } catch (error: any) {
          return err(`Error listing incidents: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'get-incident',
      description: 'Get detailed information about a specific incident',
      parameters: [
        { name: 'id', paramType: 'string', description: 'Incident ID', required: true },
      ],
      handler: (async (args: { id: string }): Promise<ExecutionResult> => {
        try {
          const client = getPagerDutyClient();
          const response = await client.get<any>(`/incidents/${args.id}`);
          if (!response.ok) {
            return err(`Incident not found: ${args.id}`, errors.notFound(`Incident ${args.id}`));
          }

          return ok(JSON.stringify(response.data?.incident, null, 2), { data: response.data?.incident });
        } catch (error: any) {
          return err(`Error getting incident: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'create-incident',
      description: 'Create a new incident manually',
      parameters: [
        { name: 'title', paramType: 'string', description: 'Incident title', required: true },
        { name: 'service', paramType: 'string', description: 'Service ID', required: true },
        { name: 'urgency', paramType: 'string', description: 'high or low (default: high)', required: false },
        { name: 'body', paramType: 'string', description: 'Incident description', required: false },
        { name: 'from', paramType: 'string', description: 'Email of the user creating the incident', required: true },
      ],
      handler: (async (args: { title: string; service: string; urgency?: string; body?: string; from: string }): Promise<ExecutionResult> => {
        try {
          const client = getPagerDutyClient();

          const incident = {
            incident: {
              type: 'incident',
              title: args.title,
              service: { id: args.service, type: 'service_reference' },
              urgency: args.urgency || 'high',
              body: args.body ? { type: 'incident_body', details: args.body } : undefined,
            },
          };

          const response = await client.post<any>('/incidents', incident, {
            headers: { 'From': args.from },
          });

          if (!response.ok) {
            return err(`Failed to create incident: ${response.status}`, errors.service('PagerDuty', String(response.status)));
          }

          return ok(`Incident created: ${response.data?.incident?.id}`, { data: response.data?.incident });
        } catch (error: any) {
          return err(`Error creating incident: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'acknowledge-incident',
      description: 'Acknowledge an incident',
      parameters: [
        { name: 'id', paramType: 'string', description: 'Incident ID', required: true },
        { name: 'from', paramType: 'string', description: 'Email of the user acknowledging', required: true },
      ],
      handler: (async (args: { id: string; from: string }): Promise<ExecutionResult> => {
        try {
          const client = getPagerDutyClient();

          const response = await client.put<any>(`/incidents/${args.id}`, {
            incident: { type: 'incident_reference', status: 'acknowledged' },
          }, {
            headers: { 'From': args.from },
          });

          if (!response.ok) {
            return err(`Failed to acknowledge incident: ${response.status}`, errors.service('PagerDuty', String(response.status)));
          }

          return ok(`Incident ${args.id} acknowledged`, { data: response.data?.incident });
        } catch (error: any) {
          return err(`Error acknowledging incident: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'resolve-incident',
      description: 'Resolve an incident',
      parameters: [
        { name: 'id', paramType: 'string', description: 'Incident ID', required: true },
        { name: 'from', paramType: 'string', description: 'Email of the user resolving', required: true },
        { name: 'resolution', paramType: 'string', description: 'Resolution notes', required: false },
      ],
      handler: (async (args: { id: string; from: string; resolution?: string }): Promise<ExecutionResult> => {
        try {
          const client = getPagerDutyClient();

          const response = await client.put<any>(`/incidents/${args.id}`, {
            incident: {
              type: 'incident_reference',
              status: 'resolved',
              resolution: args.resolution,
            },
          }, {
            headers: { 'From': args.from },
          });

          if (!response.ok) {
            return err(`Failed to resolve incident: ${response.status}`, errors.service('PagerDuty', String(response.status)));
          }

          return ok(`Incident ${args.id} resolved`, { data: response.data?.incident });
        } catch (error: any) {
          return err(`Error resolving incident: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'add-note',
      description: 'Add a note to an incident timeline',
      parameters: [
        { name: 'incident_id', paramType: 'string', description: 'Incident ID', required: true },
        { name: 'content', paramType: 'string', description: 'Note content', required: true },
        { name: 'from', paramType: 'string', description: 'Email of the user adding the note', required: true },
      ],
      handler: (async (args: { incident_id: string; content: string; from: string }): Promise<ExecutionResult> => {
        try {
          const client = getPagerDutyClient();

          const response = await client.post<any>(`/incidents/${args.incident_id}/notes`, {
            note: { content: args.content },
          }, {
            headers: { 'From': args.from },
          });

          if (!response.ok) {
            return err(`Failed to add note: ${response.status}`, errors.service('PagerDuty', String(response.status)));
          }

          return ok(`Note added to incident ${args.incident_id}`, { data: response.data?.note });
        } catch (error: any) {
          return err(`Error adding note: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'list-oncalls',
      description: 'List current on-call assignments',
      parameters: [
        { name: 'schedule', paramType: 'string', description: 'Filter by schedule ID', required: false },
        { name: 'escalation_policy', paramType: 'string', description: 'Filter by escalation policy ID', required: false },
      ],
      handler: (async (args: { schedule?: string; escalation_policy?: string }): Promise<ExecutionResult> => {
        try {
          const client = getPagerDutyClient();
          const params = new URLSearchParams();

          if (args.schedule) params.append('schedule_ids[]', args.schedule);
          if (args.escalation_policy) params.append('escalation_policy_ids[]', args.escalation_policy);

          const response = await client.get<any>(`/oncalls?${params.toString()}`);
          if (!response.ok) {
            return err(`Failed to list on-calls: ${response.status}`, errors.service('PagerDuty', String(response.status)));
          }

          return ok(JSON.stringify({ oncalls: response.data?.oncalls }, null, 2), { data: response.data?.oncalls });
        } catch (error: any) {
          return err(`Error listing on-calls: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'list-services',
      description: 'List PagerDuty services',
      parameters: [
        { name: 'query', paramType: 'string', description: 'Search query', required: false },
        { name: 'team', paramType: 'string', description: 'Filter by team ID', required: false },
      ],
      handler: (async (args: { query?: string; team?: string }): Promise<ExecutionResult> => {
        try {
          const client = getPagerDutyClient();
          const params = new URLSearchParams();

          if (args.query) params.append('query', args.query);
          if (args.team) params.append('team_ids[]', args.team);

          const response = await client.get<any>(`/services?${params.toString()}`);
          if (!response.ok) {
            return err(`Failed to list services: ${response.status}`, errors.service('PagerDuty', String(response.status)));
          }

          return ok(JSON.stringify({ services: response.data?.services }, null, 2), { data: response.data?.services });
        } catch (error: any) {
          return err(`Error listing services: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
  ],

  validateConfig: (config) => {
    if (!config.PAGERDUTY_API_KEY) {
      return { err: 'PAGERDUTY_API_KEY is required' };
    }
    return { ok: null };
  },
});

function getPagerDutyClient() {
  return createAuthenticatedClient({
    baseUrl: 'https://api.pagerduty.com',
    authType: 'bearer',
    tokenKey: 'PAGERDUTY_API_KEY',
    headers: {
      'Content-Type': 'application/json',
    },
  });
}
