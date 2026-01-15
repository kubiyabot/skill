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

interface DiscordConfig {
  DISCORD_BOT_TOKEN: string;
}

export default defineSkill({
  metadata: {
    name: 'discord',
    version: '1.0.0',
    description: 'Discord server and bot management',
    author: 'Skill Engine',
    tags: ['discord', 'messaging', 'bot', 'collaboration'],
    homepage: 'https://discord.com/',
  },

  tools: [
    {
      name: 'send-message',
      description: 'Send a message to a channel',
      parameters: [
        { name: 'channel_id', paramType: 'string', description: 'Channel ID', required: true },
        { name: 'content', paramType: 'string', description: 'Message content', required: true },
        { name: 'embed', paramType: 'json', description: 'Rich embed object', required: false },
      ],
      handler: (async (args: { channel_id: string; content: string; embed?: any }): Promise<ExecutionResult> => {
        try {
          const client = getDiscordClient();

          const messageData: any = { content: args.content };
          if (args.embed) {
            messageData.embeds = [args.embed];
          }

          const response = await client.post<any>(`/channels/${args.channel_id}/messages`, messageData);
          if (!response.ok) {
            return err(`Failed to send message: ${response.status}`, errors.service('Discord', String(response.status)));
          }

          return ok(`Message sent: ${response.data?.id}`, { data: response.data });
        } catch (error: any) {
          return err(`Error sending message: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'get-channel',
      description: 'Get information about a channel',
      parameters: [
        { name: 'channel_id', paramType: 'string', description: 'Channel ID', required: true },
      ],
      handler: (async (args: { channel_id: string }): Promise<ExecutionResult> => {
        try {
          const client = getDiscordClient();
          const response = await client.get<any>(`/channels/${args.channel_id}`);
          if (!response.ok) {
            return err(`Channel not found: ${args.channel_id}`, errors.notFound(`Channel ${args.channel_id}`));
          }

          return ok(JSON.stringify(response.data, null, 2), { data: response.data });
        } catch (error: any) {
          return err(`Error getting channel: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'list-channels',
      description: 'List channels in a server',
      parameters: [
        { name: 'guild_id', paramType: 'string', description: 'Server (guild) ID', required: true },
        { name: 'type', paramType: 'string', description: 'Filter by type: text, voice, category', required: false },
      ],
      handler: (async (args: { guild_id: string; type?: string }): Promise<ExecutionResult> => {
        try {
          const client = getDiscordClient();
          const response = await client.get<any[]>(`/guilds/${args.guild_id}/channels`);
          if (!response.ok) {
            return err(`Failed to list channels: ${response.status}`, errors.service('Discord', String(response.status)));
          }

          let channels = response.data || [];
          if (args.type) {
            const typeMap: { [key: string]: number } = { text: 0, voice: 2, category: 4 };
            const typeNum = typeMap[args.type];
            if (typeNum !== undefined) {
              channels = channels.filter((c: any) => c.type === typeNum);
            }
          }

          return ok(JSON.stringify({ channels }, null, 2), { data: channels });
        } catch (error: any) {
          return err(`Error listing channels: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'create-thread',
      description: 'Create a thread from a message or in a channel',
      parameters: [
        { name: 'channel_id', paramType: 'string', description: 'Parent channel ID', required: true },
        { name: 'name', paramType: 'string', description: 'Thread name', required: true },
        { name: 'message_id', paramType: 'string', description: 'Message to create thread from', required: false },
        { name: 'auto_archive_duration', paramType: 'number', description: 'Minutes until auto-archive (60, 1440, 4320, 10080)', required: false },
      ],
      handler: (async (args: { channel_id: string; name: string; message_id?: string; auto_archive_duration?: number }): Promise<ExecutionResult> => {
        try {
          const client = getDiscordClient();

          let url: string;
          let threadData: any;

          if (args.message_id) {
            url = `/channels/${args.channel_id}/messages/${args.message_id}/threads`;
            threadData = {
              name: args.name,
              auto_archive_duration: args.auto_archive_duration || 1440,
            };
          } else {
            url = `/channels/${args.channel_id}/threads`;
            threadData = {
              name: args.name,
              auto_archive_duration: args.auto_archive_duration || 1440,
              type: 11, // Public thread
            };
          }

          const response = await client.post<any>(url, threadData);
          if (!response.ok) {
            return err(`Failed to create thread: ${response.status}`, errors.service('Discord', String(response.status)));
          }

          return ok(`Thread created: ${response.data?.id}`, { data: response.data });
        } catch (error: any) {
          return err(`Error creating thread: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'get-messages',
      description: 'Get recent messages from a channel',
      parameters: [
        { name: 'channel_id', paramType: 'string', description: 'Channel ID', required: true },
        { name: 'limit', paramType: 'number', description: 'Number of messages (default: 50, max: 100)', required: false },
        { name: 'before', paramType: 'string', description: 'Get messages before this ID', required: false },
        { name: 'after', paramType: 'string', description: 'Get messages after this ID', required: false },
      ],
      handler: (async (args: { channel_id: string; limit?: number; before?: string; after?: string }): Promise<ExecutionResult> => {
        try {
          const client = getDiscordClient();
          const params = new URLSearchParams();
          params.append('limit', String(args.limit || 50));
          if (args.before) params.append('before', args.before);
          if (args.after) params.append('after', args.after);

          const response = await client.get<any[]>(`/channels/${args.channel_id}/messages?${params.toString()}`);
          if (!response.ok) {
            return err(`Failed to get messages: ${response.status}`, errors.service('Discord', String(response.status)));
          }

          return ok(JSON.stringify({ messages: response.data }, null, 2), { data: response.data });
        } catch (error: any) {
          return err(`Error getting messages: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'list-members',
      description: 'List members in a server',
      parameters: [
        { name: 'guild_id', paramType: 'string', description: 'Server ID', required: true },
        { name: 'limit', paramType: 'number', description: 'Number of members (default: 100)', required: false },
      ],
      handler: (async (args: { guild_id: string; limit?: number }): Promise<ExecutionResult> => {
        try {
          const client = getDiscordClient();
          const limit = args.limit || 100;
          const response = await client.get<any[]>(`/guilds/${args.guild_id}/members?limit=${limit}`);
          if (!response.ok) {
            return err(`Failed to list members: ${response.status}`, errors.service('Discord', String(response.status)));
          }

          return ok(JSON.stringify({ members: response.data }, null, 2), { data: response.data });
        } catch (error: any) {
          return err(`Error listing members: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'add-role',
      description: 'Add a role to a member',
      parameters: [
        { name: 'guild_id', paramType: 'string', description: 'Server ID', required: true },
        { name: 'user_id', paramType: 'string', description: 'User ID', required: true },
        { name: 'role_id', paramType: 'string', description: 'Role ID', required: true },
      ],
      handler: (async (args: { guild_id: string; user_id: string; role_id: string }): Promise<ExecutionResult> => {
        try {
          const client = getDiscordClient();
          const response = await client.put<any>(`/guilds/${args.guild_id}/members/${args.user_id}/roles/${args.role_id}`, {});
          if (!response.ok && response.status !== 204) {
            return err(`Failed to add role: ${response.status}`, errors.service('Discord', String(response.status)));
          }

          return ok(`Role ${args.role_id} added to user ${args.user_id}`);
        } catch (error: any) {
          return err(`Error adding role: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'remove-role',
      description: 'Remove a role from a member',
      parameters: [
        { name: 'guild_id', paramType: 'string', description: 'Server ID', required: true },
        { name: 'user_id', paramType: 'string', description: 'User ID', required: true },
        { name: 'role_id', paramType: 'string', description: 'Role ID', required: true },
      ],
      handler: (async (args: { guild_id: string; user_id: string; role_id: string }): Promise<ExecutionResult> => {
        try {
          const client = getDiscordClient();
          const response = await client.delete<any>(`/guilds/${args.guild_id}/members/${args.user_id}/roles/${args.role_id}`);
          if (!response.ok && response.status !== 204) {
            return err(`Failed to remove role: ${response.status}`, errors.service('Discord', String(response.status)));
          }

          return ok(`Role ${args.role_id} removed from user ${args.user_id}`);
        } catch (error: any) {
          return err(`Error removing role: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
    {
      name: 'add-reaction',
      description: 'Add a reaction to a message',
      parameters: [
        { name: 'channel_id', paramType: 'string', description: 'Channel ID', required: true },
        { name: 'message_id', paramType: 'string', description: 'Message ID', required: true },
        { name: 'emoji', paramType: 'string', description: 'Emoji (unicode or custom format)', required: true },
      ],
      handler: (async (args: { channel_id: string; message_id: string; emoji: string }): Promise<ExecutionResult> => {
        try {
          const client = getDiscordClient();
          const encodedEmoji = encodeURIComponent(args.emoji);
          const response = await client.put<any>(`/channels/${args.channel_id}/messages/${args.message_id}/reactions/${encodedEmoji}/@me`, {});
          if (!response.ok && response.status !== 204) {
            return err(`Failed to add reaction: ${response.status}`, errors.service('Discord', String(response.status)));
          }

          return ok(`Reaction added`);
        } catch (error: any) {
          return err(`Error adding reaction: ${error.message}`, errors.internal(error.message));
        }
      }) as ToolHandler,
    },
  ],

  validateConfig: (config) => {
    if (!config.DISCORD_BOT_TOKEN) {
      return { err: 'DISCORD_BOT_TOKEN is required' };
    }
    return { ok: null };
  },
});

function getDiscordClient() {
  const config = getConfig<DiscordConfig>();
  return createAuthenticatedClient({
    baseUrl: 'https://discord.com/api/v10',
    authType: 'bearer',
    tokenKey: 'DISCORD_BOT_TOKEN',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bot ${config.DISCORD_BOT_TOKEN}`,
    },
  });
}
