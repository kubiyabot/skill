/**
 * Prometheus Skill - Metrics querying and monitoring
 *
 * Provides access to Prometheus API for querying metrics, alerts, and system status.
 *
 * Setup:
 *   export SKILL_PROMETHEUS_URL=http://localhost:9090
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

// Get Prometheus URL from config
function getPrometheusUrl(): string {
  return getConfig('PROMETHEUS_URL') || 'http://localhost:9090';
}

// Make Prometheus API request
async function prometheusRequest(
  endpoint: string,
  params?: Record<string, string>
): Promise<ExecutionResult> {
  try {
    const baseUrl = getPrometheusUrl();
    const queryString = params
      ? '?' + Object.entries(params).map(([k, v]) => `${k}=${encodeURIComponent(v)}`).join('&')
      : '';

    const response = await httpRequest({
      method: 'GET',
      url: `${baseUrl}/api/v1${endpoint}${queryString}`,
      headers: { 'Accept': 'application/json' },
    });

    if (!response.ok) {
      return err(`Prometheus API error: ${response.status} ${response.statusText}`);
    }

    const data = JSON.parse(response.body || '{}');
    if (data.status !== 'success') {
      return err(`Prometheus error: ${data.error || 'Unknown error'}`);
    }

    return ok(JSON.stringify(data.data, null, 2), { data: data.data });
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : String(e);
    return err(`Request failed: ${message}`);
  }
}

export default defineSkill({
  metadata: {
    name: 'prometheus-skill',
    version: '1.0.0',
    description: 'Prometheus metrics querying and monitoring',
    author: 'Skill Engine Team',
    tags: ['prometheus', 'metrics', 'monitoring', 'observability'],
  },
  tools: [
    // Query Tools
    {
      name: 'query',
      description: 'Execute instant PromQL query',
      parameters: [
        { name: 'query', paramType: 'string', description: 'PromQL query expression', required: true },
        { name: 'time', paramType: 'string', description: 'Evaluation timestamp (RFC3339 or Unix)', required: false },
        { name: 'timeout', paramType: 'string', description: 'Evaluation timeout', required: false },
      ],
      handler: (async (args: { query: string; time?: string; timeout?: string }): Promise<ExecutionResult> => {
        const params: Record<string, string> = { query: args.query };
        if (args.time) params.time = args.time;
        if (args.timeout) params.timeout = args.timeout;
        return prometheusRequest('/query', params);
      }) as ToolHandler,
    },
    {
      name: 'query-range',
      description: 'Execute range PromQL query',
      parameters: [
        { name: 'query', paramType: 'string', description: 'PromQL query expression', required: true },
        { name: 'start', paramType: 'string', description: 'Start timestamp', required: true },
        { name: 'end', paramType: 'string', description: 'End timestamp', required: true },
        { name: 'step', paramType: 'string', description: 'Query resolution step (e.g., 15s, 1m)', required: true },
        { name: 'timeout', paramType: 'string', description: 'Evaluation timeout', required: false },
      ],
      handler: (async (args: { query: string; start: string; end: string; step: string; timeout?: string }): Promise<ExecutionResult> => {
        const params: Record<string, string> = {
          query: args.query,
          start: args.start,
          end: args.end,
          step: args.step,
        };
        if (args.timeout) params.timeout = args.timeout;
        return prometheusRequest('/query_range', params);
      }) as ToolHandler,
    },
    {
      name: 'series',
      description: 'Find time series matching label selectors',
      parameters: [
        { name: 'match', paramType: 'string', description: 'Series selector (e.g., up, {job="prometheus"})', required: true },
        { name: 'start', paramType: 'string', description: 'Start timestamp', required: false },
        { name: 'end', paramType: 'string', description: 'End timestamp', required: false },
      ],
      handler: (async (args: { match: string; start?: string; end?: string }): Promise<ExecutionResult> => {
        const params: Record<string, string> = { 'match[]': args.match };
        if (args.start) params.start = args.start;
        if (args.end) params.end = args.end;
        return prometheusRequest('/series', params);
      }) as ToolHandler,
    },
    {
      name: 'labels',
      description: 'Get all label names',
      parameters: [
        { name: 'start', paramType: 'string', description: 'Start timestamp', required: false },
        { name: 'end', paramType: 'string', description: 'End timestamp', required: false },
        { name: 'match', paramType: 'string', description: 'Series selector to filter', required: false },
      ],
      handler: (async (args: { start?: string; end?: string; match?: string }): Promise<ExecutionResult> => {
        const params: Record<string, string> = {};
        if (args.start) params.start = args.start;
        if (args.end) params.end = args.end;
        if (args.match) params['match[]'] = args.match;
        return prometheusRequest('/labels', params);
      }) as ToolHandler,
    },
    {
      name: 'label-values',
      description: 'Get values for a specific label',
      parameters: [
        { name: 'label', paramType: 'string', description: 'Label name', required: true },
        { name: 'start', paramType: 'string', description: 'Start timestamp', required: false },
        { name: 'end', paramType: 'string', description: 'End timestamp', required: false },
        { name: 'match', paramType: 'string', description: 'Series selector to filter', required: false },
      ],
      handler: (async (args: { label: string; start?: string; end?: string; match?: string }): Promise<ExecutionResult> => {
        const params: Record<string, string> = {};
        if (args.start) params.start = args.start;
        if (args.end) params.end = args.end;
        if (args.match) params['match[]'] = args.match;
        return prometheusRequest(`/label/${args.label}/values`, params);
      }) as ToolHandler,
    },
    // Target and Rule Tools
    {
      name: 'targets',
      description: 'Get current scrape targets and their status',
      parameters: [
        { name: 'state', paramType: 'string', description: 'Filter by state: active, dropped, any', required: false },
      ],
      handler: (async (args: { state?: string }): Promise<ExecutionResult> => {
        const params: Record<string, string> = {};
        if (args.state) params.state = args.state;
        return prometheusRequest('/targets', params);
      }) as ToolHandler,
    },
    {
      name: 'rules',
      description: 'Get alerting and recording rules',
      parameters: [
        { name: 'type', paramType: 'string', description: 'Filter by type: alert, record', required: false },
      ],
      handler: (async (args: { type?: string }): Promise<ExecutionResult> => {
        const params: Record<string, string> = {};
        if (args.type) params.type = args.type;
        return prometheusRequest('/rules', params);
      }) as ToolHandler,
    },
    {
      name: 'alerts',
      description: 'Get active alerts',
      parameters: [],
      handler: (async (): Promise<ExecutionResult> => {
        return prometheusRequest('/alerts');
      }) as ToolHandler,
    },
    {
      name: 'metadata',
      description: 'Get metric metadata',
      parameters: [
        { name: 'metric', paramType: 'string', description: 'Metric name to filter', required: false },
        { name: 'limit', paramType: 'number', description: 'Maximum number of metrics', required: false },
      ],
      handler: (async (args: { metric?: string; limit?: number }): Promise<ExecutionResult> => {
        const params: Record<string, string> = {};
        if (args.metric) params.metric = args.metric;
        if (args.limit) params.limit = String(args.limit);
        return prometheusRequest('/metadata', params);
      }) as ToolHandler,
    },
    // Status Tools
    {
      name: 'status-config',
      description: 'Get current Prometheus configuration',
      parameters: [],
      handler: (async (): Promise<ExecutionResult> => {
        return prometheusRequest('/status/config');
      }) as ToolHandler,
    },
    {
      name: 'status-flags',
      description: 'Get Prometheus flag values',
      parameters: [],
      handler: (async (): Promise<ExecutionResult> => {
        return prometheusRequest('/status/flags');
      }) as ToolHandler,
    },
    {
      name: 'status-runtimeinfo',
      description: 'Get runtime information',
      parameters: [],
      handler: (async (): Promise<ExecutionResult> => {
        return prometheusRequest('/status/runtimeinfo');
      }) as ToolHandler,
    },
    {
      name: 'status-buildinfo',
      description: 'Get build information',
      parameters: [],
      handler: (async (): Promise<ExecutionResult> => {
        return prometheusRequest('/status/buildinfo');
      }) as ToolHandler,
    },
    {
      name: 'status-tsdb',
      description: 'Get TSDB statistics',
      parameters: [],
      handler: (async (): Promise<ExecutionResult> => {
        return prometheusRequest('/status/tsdb');
      }) as ToolHandler,
    },
    {
      name: 'check-health',
      description: 'Check Prometheus health status',
      parameters: [],
      handler: (async (): Promise<ExecutionResult> => {
        try {
          const baseUrl = getPrometheusUrl();
          const response = await httpRequest({
            method: 'GET',
            url: `${baseUrl}/-/healthy`,
          });
          if (response.ok) {
            return ok('Prometheus is healthy', { healthy: true });
          }
          return ok('Prometheus is not healthy', { healthy: false });
        } catch (e: unknown) {
          const message = e instanceof Error ? e.message : String(e);
          return err(`Health check failed: ${message}`);
        }
      }) as ToolHandler,
    },
  ],
});
