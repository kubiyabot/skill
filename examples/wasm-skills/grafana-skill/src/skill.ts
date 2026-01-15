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

interface GrafanaConfig {
  GRAFANA_URL: string;
  GRAFANA_API_KEY: string;
}

export default defineSkill({
  metadata: {
    name: 'grafana',
    version: '1.0.0',
    description: 'Grafana dashboard and alert management',
    author: 'Skill Engine',
    tags: ['monitoring', 'dashboards', 'alerts', 'grafana'],
    homepage: 'https://grafana.com/',
  },

  tools: [
    {
      name: 'list-dashboards',
      description: 'List all dashboards with optional filtering',
      parameters: [
        { name: 'folder', paramType: 'string', description: 'Filter by folder name', required: false },
        { name: 'tag', paramType: 'string', description: 'Filter by tag', required: false },
        { name: 'query', paramType: 'string', description: 'Search query', required: false },
        { name: 'limit', paramType: 'number', description: 'Maximum results (default: 100)', required: false },
      ],
      handler: (async (args: { folder?: string; tag?: string; query?: string; limit?: number }): Promise<ExecutionResult> => {
        try {
          const client = getGrafanaClient();
          const params = new URLSearchParams();
          params.append('type', 'dash-db');
          if (args.query) params.append('query', args.query);
          if (args.tag) params.append('tag', args.tag);
          if (args.folder) params.append('folderIds', args.folder);
          params.append('limit', String(args.limit || 100));

          const response = await client.get<any[]>(`/api/search?${params.toString()}`);
          if (!response.ok) {
            return err(`Failed to list dashboards: ${response.status}`, errors.service('Grafana', String(response.status)));
          }

          return ok(JSON.stringify({ dashboards: response.data }, null, 2), { data: response.data });
        } catch (error: any) {
          return err(`Error listing dashboards: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'get-dashboard',
      description: 'Get dashboard details including panels',
      parameters: [
        { name: 'uid', paramType: 'string', description: 'Dashboard UID', required: true },
      ],
      handler: (async (args: { uid: string }): Promise<ExecutionResult> => {
        try {
          const client = getGrafanaClient();
          const response = await client.get<any>(`/api/dashboards/uid/${args.uid}`);
          if (!response.ok) {
            return err(`Dashboard not found: ${args.uid}`, errors.notFound(`Dashboard ${args.uid}`));
          }

          return ok(JSON.stringify(response.data, null, 2), { data: response.data });
        } catch (error: any) {
          return err(`Error getting dashboard: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'list-alerts',
      description: 'List alert rules',
      parameters: [
        { name: 'state', paramType: 'string', description: 'Filter by state: firing, pending, inactive', required: false },
        { name: 'folder', paramType: 'string', description: 'Filter by folder', required: false },
        { name: 'limit', paramType: 'number', description: 'Maximum results (default: 100)', required: false },
      ],
      handler: (async (args: { state?: string; folder?: string; limit?: number }): Promise<ExecutionResult> => {
        try {
          const client = getGrafanaClient();
          const params = new URLSearchParams();
          if (args.state) params.append('state', args.state);
          if (args.limit) params.append('limit', String(args.limit));

          const response = await client.get<any>(`/api/v1/provisioning/alert-rules`);
          if (!response.ok) {
            return err(`Failed to list alerts: ${response.status}`, errors.service('Grafana', String(response.status)));
          }

          let alerts = response.data || [];
          if (args.folder) {
            alerts = alerts.filter((a: any) => a.folderTitle === args.folder);
          }

          return ok(JSON.stringify({ alerts }, null, 2), { data: alerts });
        } catch (error: any) {
          return err(`Error listing alerts: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'get-alert',
      description: 'Get details of a specific alert rule',
      parameters: [
        { name: 'uid', paramType: 'string', description: 'Alert rule UID', required: true },
      ],
      handler: (async (args: { uid: string }): Promise<ExecutionResult> => {
        try {
          const client = getGrafanaClient();
          const response = await client.get<any>(`/api/v1/provisioning/alert-rules/${args.uid}`);
          if (!response.ok) {
            return err(`Alert not found: ${args.uid}`, errors.notFound(`Alert ${args.uid}`));
          }

          return ok(JSON.stringify(response.data, null, 2), { data: response.data });
        } catch (error: any) {
          return err(`Error getting alert: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'silence-alert',
      description: 'Create a silence for an alert',
      parameters: [
        { name: 'matchers', paramType: 'string', description: 'Label matchers (e.g., alertname=High CPU)', required: true },
        { name: 'duration', paramType: 'string', description: 'Silence duration (e.g., 1h, 30m, 2d)', required: true },
        { name: 'comment', paramType: 'string', description: 'Reason for silence', required: false },
        { name: 'createdBy', paramType: 'string', description: 'Creator name', required: false },
      ],
      handler: (async (args: { matchers: string; duration: string; comment?: string; createdBy?: string }): Promise<ExecutionResult> => {
        try {
          const client = getGrafanaClient();

          // Parse duration to end time
          const now = new Date();
          const durationMs = parseDuration(args.duration);
          const endsAt = new Date(now.getTime() + durationMs);

          // Parse matchers
          const matcherPairs = args.matchers.split(',').map(m => {
            const [name, value] = m.trim().split('=');
            return { name: name.trim(), value: value?.trim() || '', isRegex: false, isEqual: true };
          });

          const silence = {
            matchers: matcherPairs,
            startsAt: now.toISOString(),
            endsAt: endsAt.toISOString(),
            comment: args.comment || 'Silenced via Skill Engine',
            createdBy: args.createdBy || 'skill-engine',
          };

          const response = await client.post<any>('/api/alertmanager/grafana/api/v2/silences', silence);
          if (!response.ok) {
            return err(`Failed to create silence: ${response.status}`, errors.service('Grafana', String(response.status)));
          }

          return ok(`Silence created successfully. ID: ${response.data?.silenceID}`, { data: response.data });
        } catch (error: any) {
          return err(`Error creating silence: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'query-datasource',
      description: 'Query a data source directly',
      parameters: [
        { name: 'datasource', paramType: 'string', description: 'Data source name or UID', required: true },
        { name: 'query', paramType: 'string', description: 'Query string (PromQL, SQL, etc.)', required: true },
        { name: 'from', paramType: 'string', description: 'Start time (default: 1h ago)', required: false },
        { name: 'to', paramType: 'string', description: 'End time (default: now)', required: false },
      ],
      handler: (async (args: { datasource: string; query: string; from?: string; to?: string }): Promise<ExecutionResult> => {
        try {
          const client = getGrafanaClient();

          // Get datasource info
          const dsResponse = await client.get<any>(`/api/datasources/name/${encodeURIComponent(args.datasource)}`);
          if (!dsResponse.ok) {
            return err(`Datasource not found: ${args.datasource}`, errors.notFound(`Datasource ${args.datasource}`));
          }

          const datasource = dsResponse.data;
          const now = Date.now();
          const fromTime = args.from ? parseRelativeTime(args.from, now) : now - 3600000;
          const toTime = args.to ? parseRelativeTime(args.to, now) : now;

          const queryPayload = {
            queries: [{
              refId: 'A',
              datasource: { uid: datasource.uid, type: datasource.type },
              expr: args.query,
              instant: false,
              range: true,
            }],
            from: String(fromTime),
            to: String(toTime),
          };

          const response = await client.post<any>('/api/ds/query', queryPayload);
          if (!response.ok) {
            return err(`Query failed: ${response.status}`, errors.service('Grafana', String(response.status)));
          }

          return ok(JSON.stringify(response.data, null, 2), { data: response.data });
        } catch (error: any) {
          return err(`Error querying datasource: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'list-datasources',
      description: 'List configured data sources',
      parameters: [],
      handler: (async (): Promise<ExecutionResult> => {
        try {
          const client = getGrafanaClient();
          const response = await client.get<any[]>('/api/datasources');
          if (!response.ok) {
            return err(`Failed to list datasources: ${response.status}`, errors.service('Grafana', String(response.status)));
          }

          const datasources = (response.data || []).map((ds: any) => ({
            name: ds.name,
            type: ds.type,
            url: ds.url,
            default: ds.isDefault,
            uid: ds.uid,
          }));

          return ok(JSON.stringify({ datasources }, null, 2), { data: datasources });
        } catch (error: any) {
          return err(`Error listing datasources: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'list-folders',
      description: 'List dashboard folders',
      parameters: [],
      handler: (async (): Promise<ExecutionResult> => {
        try {
          const client = getGrafanaClient();
          const response = await client.get<any[]>('/api/folders');
          if (!response.ok) {
            return err(`Failed to list folders: ${response.status}`, errors.service('Grafana', String(response.status)));
          }

          return ok(JSON.stringify({ folders: response.data }, null, 2), { data: response.data });
        } catch (error: any) {
          return err(`Error listing folders: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
  ],

  validateConfig: (config) => {
    if (!config.GRAFANA_URL) {
      return { err: 'GRAFANA_URL is required' };
    }
    if (!config.GRAFANA_API_KEY) {
      return { err: 'GRAFANA_API_KEY is required' };
    }
    return { ok: null };
  },
});

function getGrafanaClient() {
  const config = getConfig<GrafanaConfig>();
  return createAuthenticatedClient({
    baseUrl: config.GRAFANA_URL,
    authType: 'bearer',
    tokenKey: 'GRAFANA_API_KEY',
    headers: {
      'Content-Type': 'application/json',
    },
  });
}

function parseDuration(duration: string): number {
  const match = duration.match(/^(\d+)([smhd])$/);
  if (!match) throw new Error(`Invalid duration format: ${duration}`);

  const value = parseInt(match[1]);
  const unit = match[2];

  switch (unit) {
    case 's': return value * 1000;
    case 'm': return value * 60 * 1000;
    case 'h': return value * 60 * 60 * 1000;
    case 'd': return value * 24 * 60 * 60 * 1000;
    default: throw new Error(`Unknown duration unit: ${unit}`);
  }
}

function parseRelativeTime(time: string, now: number): number {
  if (time === 'now') return now;

  const match = time.match(/^(\d+)([smhd])\s*ago$/);
  if (match) {
    const value = parseInt(match[1]);
    const unit = match[2];
    const ms = parseDuration(`${value}${unit}`);
    return now - ms;
  }

  // Try parsing as timestamp
  const ts = parseInt(time);
  if (!isNaN(ts)) return ts;

  // Try parsing as date
  const date = new Date(time);
  if (!isNaN(date.getTime())) return date.getTime();

  return now - 3600000; // Default to 1 hour ago
}
