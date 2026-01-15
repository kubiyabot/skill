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

interface DigitalOceanConfig {
  DIGITALOCEAN_TOKEN: string;
}

export default defineSkill({
  metadata: {
    name: 'digitalocean',
    version: '1.0.0',
    description: 'DigitalOcean infrastructure management',
    author: 'Skill Engine',
    tags: ['cloud', 'droplets', 'dns', 'digitalocean'],
    homepage: 'https://digitalocean.com/',
  },

  tools: [
    {
      name: 'droplet-list',
      description: 'List all droplets in your account',
      parameters: [
        { name: 'tag', paramType: 'string', description: 'Filter by tag', required: false },
        { name: 'region', paramType: 'string', description: 'Filter by region', required: false },
      ],
      handler: (async (args: { tag?: string; region?: string }): Promise<ExecutionResult> => {
        try {
          const client = getDoClient();
          let url = '/v2/droplets';
          if (args.tag) url += `?tag_name=${encodeURIComponent(args.tag)}`;

          const response = await client.get<any>(url);
          if (!response.ok) {
            return err(`Failed to list droplets: ${response.status}`, errors.service('DigitalOcean', String(response.status)));
          }

          let droplets = response.data?.droplets || [];
          if (args.region) {
            droplets = droplets.filter((d: any) => d.region?.slug === args.region);
          }

          return ok(JSON.stringify({ droplets }, null, 2), { data: droplets });
        } catch (error: any) {
          return err(`Error listing droplets: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'droplet-create',
      description: 'Create a new droplet',
      parameters: [
        { name: 'name', paramType: 'string', description: 'Droplet name', required: true },
        { name: 'region', paramType: 'string', description: 'Region slug (e.g., nyc1, sfo3)', required: true },
        { name: 'size', paramType: 'string', description: 'Size slug (e.g., s-1vcpu-1gb)', required: true },
        { name: 'image', paramType: 'string', description: 'Image slug or ID', required: true },
        { name: 'ssh_keys', paramType: 'string', description: 'Comma-separated SSH key IDs', required: false },
        { name: 'tags', paramType: 'string', description: 'Comma-separated tags', required: false },
      ],
      handler: (async (args: { name: string; region: string; size: string; image: string; ssh_keys?: string; tags?: string }): Promise<ExecutionResult> => {
        try {
          const client = getDoClient();

          const dropletData: any = {
            name: args.name,
            region: args.region,
            size: args.size,
            image: args.image,
          };

          if (args.ssh_keys) {
            dropletData.ssh_keys = args.ssh_keys.split(',').map(k => k.trim());
          }
          if (args.tags) {
            dropletData.tags = args.tags.split(',').map(t => t.trim());
          }

          const response = await client.post<any>('/v2/droplets', dropletData);
          if (!response.ok) {
            return err(`Failed to create droplet: ${response.status}`, errors.service('DigitalOcean', String(response.status)));
          }

          return ok(`Droplet created: ${response.data?.droplet?.id}`, { data: response.data?.droplet });
        } catch (error: any) {
          return err(`Error creating droplet: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'droplet-action',
      description: 'Perform actions on a droplet (power on/off, reboot, snapshot)',
      parameters: [
        { name: 'id', paramType: 'string', description: 'Droplet ID', required: true },
        { name: 'action', paramType: 'string', description: 'Action: power_on, power_off, reboot, snapshot', required: true },
        { name: 'name', paramType: 'string', description: 'Snapshot name (for snapshot action)', required: false },
      ],
      handler: (async (args: { id: string; action: string; name?: string }): Promise<ExecutionResult> => {
        try {
          const client = getDoClient();

          const actionData: any = { type: args.action };
          if (args.action === 'snapshot' && args.name) {
            actionData.name = args.name;
          }

          const response = await client.post<any>(`/v2/droplets/${args.id}/actions`, actionData);
          if (!response.ok) {
            return err(`Failed to perform action: ${response.status}`, errors.service('DigitalOcean', String(response.status)));
          }

          return ok(`Action ${args.action} initiated on droplet ${args.id}`, { data: response.data?.action });
        } catch (error: any) {
          return err(`Error performing action: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'droplet-delete',
      description: 'Delete a droplet',
      parameters: [
        { name: 'id', paramType: 'string', description: 'Droplet ID', required: true },
      ],
      handler: (async (args: { id: string }): Promise<ExecutionResult> => {
        try {
          const client = getDoClient();
          const response = await client.delete<any>(`/v2/droplets/${args.id}`);
          if (!response.ok && response.status !== 204) {
            return err(`Failed to delete droplet: ${response.status}`, errors.service('DigitalOcean', String(response.status)));
          }

          return ok(`Droplet ${args.id} deleted`);
        } catch (error: any) {
          return err(`Error deleting droplet: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'dns-list',
      description: 'List DNS records for a domain',
      parameters: [
        { name: 'domain', paramType: 'string', description: 'Domain name', required: true },
        { name: 'type', paramType: 'string', description: 'Filter by record type (A, CNAME, etc.)', required: false },
      ],
      handler: (async (args: { domain: string; type?: string }): Promise<ExecutionResult> => {
        try {
          const client = getDoClient();
          let url = `/v2/domains/${args.domain}/records`;
          if (args.type) url += `?type=${args.type}`;

          const response = await client.get<any>(url);
          if (!response.ok) {
            return err(`Failed to list DNS records: ${response.status}`, errors.service('DigitalOcean', String(response.status)));
          }

          return ok(JSON.stringify({ records: response.data?.domain_records }, null, 2), { data: response.data?.domain_records });
        } catch (error: any) {
          return err(`Error listing DNS records: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'dns-create',
      description: 'Create a DNS record',
      parameters: [
        { name: 'domain', paramType: 'string', description: 'Domain name', required: true },
        { name: 'type', paramType: 'string', description: 'Record type (A, AAAA, CNAME, MX, TXT)', required: true },
        { name: 'name', paramType: 'string', description: 'Record name', required: true },
        { name: 'data', paramType: 'string', description: 'Record data', required: true },
        { name: 'ttl', paramType: 'number', description: 'TTL in seconds (default: 3600)', required: false },
        { name: 'priority', paramType: 'number', description: 'Priority (for MX records)', required: false },
      ],
      handler: (async (args: { domain: string; type: string; name: string; data: string; ttl?: number; priority?: number }): Promise<ExecutionResult> => {
        try {
          const client = getDoClient();

          const recordData: any = {
            type: args.type,
            name: args.name,
            data: args.data,
            ttl: args.ttl || 3600,
          };
          if (args.priority !== undefined) {
            recordData.priority = args.priority;
          }

          const response = await client.post<any>(`/v2/domains/${args.domain}/records`, recordData);
          if (!response.ok) {
            return err(`Failed to create DNS record: ${response.status}`, errors.service('DigitalOcean', String(response.status)));
          }

          return ok(`DNS record created: ${response.data?.domain_record?.id}`, { data: response.data?.domain_record });
        } catch (error: any) {
          return err(`Error creating DNS record: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'dns-delete',
      description: 'Delete a DNS record',
      parameters: [
        { name: 'domain', paramType: 'string', description: 'Domain name', required: true },
        { name: 'id', paramType: 'string', description: 'Record ID', required: true },
      ],
      handler: (async (args: { domain: string; id: string }): Promise<ExecutionResult> => {
        try {
          const client = getDoClient();
          const response = await client.delete<any>(`/v2/domains/${args.domain}/records/${args.id}`);
          if (!response.ok && response.status !== 204) {
            return err(`Failed to delete DNS record: ${response.status}`, errors.service('DigitalOcean', String(response.status)));
          }

          return ok(`DNS record ${args.id} deleted`);
        } catch (error: any) {
          return err(`Error deleting DNS record: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'database-list',
      description: 'List managed databases',
      parameters: [],
      handler: (async (): Promise<ExecutionResult> => {
        try {
          const client = getDoClient();
          const response = await client.get<any>('/v2/databases');
          if (!response.ok) {
            return err(`Failed to list databases: ${response.status}`, errors.service('DigitalOcean', String(response.status)));
          }

          return ok(JSON.stringify({ databases: response.data?.databases }, null, 2), { data: response.data?.databases });
        } catch (error: any) {
          return err(`Error listing databases: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
  ],

  validateConfig: (config) => {
    if (!config.DIGITALOCEAN_TOKEN) {
      return { err: 'DIGITALOCEAN_TOKEN is required' };
    }
    return { ok: null };
  },
});

function getDoClient() {
  return createAuthenticatedClient({
    baseUrl: 'https://api.digitalocean.com',
    authType: 'bearer',
    tokenKey: 'DIGITALOCEAN_TOKEN',
    headers: {
      'Content-Type': 'application/json',
    },
  });
}
