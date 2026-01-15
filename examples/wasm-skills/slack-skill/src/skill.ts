/**
 * Slack Skill
 *
 * A Slack integration demonstrating:
 * - OAuth2 token-based authentication
 * - Sending messages to channels
 * - Listing channels and users
 * - Posting to threads
 *
 * Setup:
 *   1. Create a Slack app at https://api.slack.com/apps
 *   2. Add Bot Token Scopes: chat:write, channels:read, users:read
 *   3. Install to workspace and copy Bot User OAuth Token
 *   4. Set token: export SKILL_SLACK_TOKEN=xoxb-your-token
 *   5. Run: skill run slack-skill send-message --channel general --text "Hello!"
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

// Slack API types
interface SlackChannel {
  id: string;
  name: string;
  is_private: boolean;
  is_archived: boolean;
  num_members?: number;
  topic?: { value: string };
  purpose?: { value: string };
}

interface SlackUser {
  id: string;
  name: string;
  real_name: string;
  is_bot: boolean;
  is_admin: boolean;
  profile: {
    display_name: string;
    email?: string;
    image_48: string;
  };
}

interface SlackMessage {
  ok: boolean;
  channel: string;
  ts: string;
  message: {
    text: string;
    user: string;
    ts: string;
  };
}

interface SlackResponse<T> {
  ok: boolean;
  error?: string;
  channels?: T[];
  members?: T[];
  channel?: T;
  user?: T;
}

// Create authenticated Slack client
function getSlackClient() {
  return createAuthenticatedClient({
    baseUrl: 'https://slack.com/api',
    authType: 'bearer',
    tokenKey: 'SLACK_TOKEN',
    headers: {
      'Content-Type': 'application/json; charset=utf-8',
    },
  });
}

// Handle Slack API errors
function handleSlackError(response: HttpResponse<SlackResponse<unknown>>): ExecutionResult {
  if (!response.ok) {
    if (response.status === 401) {
      return err('Authentication failed. Check your SLACK_TOKEN.', errors.auth());
    }
    if (response.status === 429) {
      return err('Rate limit exceeded. Try again later.', errors.rateLimit());
    }
    return err(`Slack API error: ${response.status} ${response.statusText}`);
  }

  // Slack returns 200 with ok: false for API errors
  const data = response.data;
  if (!data.ok && data.error) {
    const errorMessages: Record<string, string> = {
      'channel_not_found': 'Channel not found. Check the channel name or ID.',
      'not_in_channel': 'Bot is not in this channel. Invite the bot first.',
      'invalid_auth': 'Invalid authentication token.',
      'token_revoked': 'Token has been revoked. Generate a new one.',
      'no_permission': 'Bot lacks permission for this action.',
      'user_not_found': 'User not found.',
      'missing_scope': 'Missing required OAuth scope.',
    };
    const message = errorMessages[data.error] || `Slack error: ${data.error}`;
    return err(message);
  }

  return ok(''); // Success placeholder
}

// Skill definition
export default defineSkill({
  metadata: {
    name: 'slack-skill',
    version: '1.0.0',
    description: 'Slack integration for messaging, channels, and user management',
    author: 'Skill Engine Team',
    tags: ['slack', 'messaging', 'chat', 'communication'],
  },
  tools: [
    // ========================================
    // Messaging Tools
    // ========================================
    {
      name: 'send-message',
      description: 'Send a message to a Slack channel',
      parameters: [
        {
          name: 'channel',
          paramType: 'string',
          description: 'Channel name (without #) or channel ID',
          required: true,
          validation: {
            minLength: 1,
            maxLength: 80,
          },
        },
        {
          name: 'text',
          paramType: 'string',
          description: 'Message text (supports Slack markdown)',
          required: true,
          validation: {
            minLength: 1,
            maxLength: 40000,
          },
        },
        {
          name: 'thread_ts',
          paramType: 'string',
          description: 'Thread timestamp to reply in thread',
          required: false,
        },
      ],
      handler: (async (args: {
        channel: string;
        text: string;
        thread_ts?: string;
      }): Promise<ExecutionResult> => {
        const client = getSlackClient();

        // Resolve channel name to ID if needed
        let channelId = args.channel;
        if (!args.channel.startsWith('C') && !args.channel.startsWith('D')) {
          // It's a channel name, resolve it
          const channelsResponse = await client.get<SlackResponse<SlackChannel>>(
            '/conversations.list?types=public_channel,private_channel&limit=200'
          );

          if (channelsResponse.ok && channelsResponse.data.ok) {
            const channel = channelsResponse.data.channels?.find(
              c => c.name === args.channel.replace(/^#/, '')
            );
            if (channel) {
              channelId = channel.id;
            }
          }
        }

        const body: Record<string, string> = {
          channel: channelId,
          text: args.text,
        };

        if (args.thread_ts) {
          body.thread_ts = args.thread_ts;
        }

        const response = await client.post<SlackMessage>('/chat.postMessage', body);

        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }

        const threadInfo = args.thread_ts ? ' (in thread)' : '';
        return ok(
          `Message sent to #${args.channel}${threadInfo}\nTimestamp: ${response.data.ts}`,
          {
            channel: response.data.channel,
            ts: response.data.ts,
            thread_ts: args.thread_ts
          }
        );
      }) as ToolHandler,
    },

    {
      name: 'send-dm',
      description: 'Send a direct message to a user',
      parameters: [
        {
          name: 'user',
          paramType: 'string',
          description: 'User ID (like U12345) or @username',
          required: true,
        },
        {
          name: 'text',
          paramType: 'string',
          description: 'Message text',
          required: true,
          validation: {
            minLength: 1,
            maxLength: 40000,
          },
        },
      ],
      handler: (async (args: {
        user: string;
        text: string;
      }): Promise<ExecutionResult> => {
        const client = getSlackClient();

        // Resolve username to ID if needed
        let userId = args.user;
        if (args.user.startsWith('@')) {
          const usersResponse = await client.get<SlackResponse<SlackUser>>('/users.list?limit=200');
          if (usersResponse.ok && usersResponse.data.ok) {
            const user = usersResponse.data.members?.find(
              u => u.name === args.user.slice(1) || u.profile.display_name === args.user.slice(1)
            );
            if (user) {
              userId = user.id;
            } else {
              return err(`User ${args.user} not found`);
            }
          }
        }

        // Open DM channel
        const openResponse = await client.post<{ ok: boolean; channel: { id: string } }>(
          '/conversations.open',
          { users: userId }
        );

        if (!openResponse.ok || !openResponse.data.ok) {
          return err('Failed to open DM channel');
        }

        const dmChannelId = openResponse.data.channel.id;

        // Send message
        const response = await client.post<SlackMessage>('/chat.postMessage', {
          channel: dmChannelId,
          text: args.text,
        });

        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }

        return ok(
          `DM sent to ${args.user}\nTimestamp: ${response.data.ts}`,
          { channel: dmChannelId, ts: response.data.ts }
        );
      }) as ToolHandler,
    },

    // ========================================
    // Channel Tools
    // ========================================
    {
      name: 'list-channels',
      description: 'List Slack channels in the workspace',
      parameters: [
        {
          name: 'type',
          paramType: 'string',
          description: 'Channel type: public, private, or all',
          required: false,
          defaultValue: 'public',
          validation: {
            enum: ['public', 'private', 'all'],
          },
        },
        {
          name: 'limit',
          paramType: 'number',
          description: 'Maximum number of channels to return',
          required: false,
          defaultValue: '50',
          validation: {
            minimum: 1,
            maximum: 200,
          },
        },
      ],
      handler: (async (args: {
        type: string;
        limit: number;
      }): Promise<ExecutionResult> => {
        const client = getSlackClient();

        const types = args.type === 'all'
          ? 'public_channel,private_channel'
          : args.type === 'private'
            ? 'private_channel'
            : 'public_channel';

        const response = await client.get<SlackResponse<SlackChannel>>(
          `/conversations.list?types=${types}&limit=${args.limit}&exclude_archived=true`
        );

        if (!response.ok || !response.data.ok) {
          return handleSlackError(response);
        }

        const channels = response.data.channels || [];

        if (channels.length === 0) {
          return ok('No channels found.');
        }

        const output = channels.map(ch => {
          const icon = ch.is_private ? '🔒' : '#';
          const members = ch.num_members ? ` (${ch.num_members} members)` : '';
          return `${icon} ${ch.name}${members}`;
        }).join('\n');

        return ok(
          `Found ${channels.length} channels:\n\n${output}`,
          { channels: channels.map(c => ({ id: c.id, name: c.name })) }
        );
      }) as ToolHandler,
    },

    {
      name: 'get-channel-info',
      description: 'Get detailed information about a channel',
      parameters: [
        {
          name: 'channel',
          paramType: 'string',
          description: 'Channel name or ID',
          required: true,
        },
      ],
      handler: (async (args: { channel: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();

        // Resolve channel name to ID if needed
        let channelId = args.channel;
        if (!args.channel.startsWith('C')) {
          const channelsResponse = await client.get<SlackResponse<SlackChannel>>(
            '/conversations.list?types=public_channel,private_channel&limit=200'
          );
          if (channelsResponse.ok && channelsResponse.data.ok) {
            const channel = channelsResponse.data.channels?.find(
              c => c.name === args.channel.replace(/^#/, '')
            );
            if (channel) {
              channelId = channel.id;
            } else {
              return err(`Channel ${args.channel} not found`);
            }
          }
        }

        const response = await client.get<{ ok: boolean; channel: SlackChannel }>(
          `/conversations.info?channel=${channelId}`
        );

        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }

        const ch = response.data.channel;
        const output = [
          `Channel: #${ch.name}`,
          `ID: ${ch.id}`,
          `Type: ${ch.is_private ? 'Private' : 'Public'}`,
          ch.num_members ? `Members: ${ch.num_members}` : null,
          ch.topic?.value ? `Topic: ${ch.topic.value}` : null,
          ch.purpose?.value ? `Purpose: ${ch.purpose.value}` : null,
          `Archived: ${ch.is_archived ? 'Yes' : 'No'}`,
        ].filter(Boolean).join('\n');

        return ok(output, { channel: ch });
      }) as ToolHandler,
    },

    // ========================================
    // User Tools
    // ========================================
    {
      name: 'list-users',
      description: 'List users in the workspace',
      parameters: [
        {
          name: 'include_bots',
          paramType: 'boolean',
          description: 'Include bot users',
          required: false,
          defaultValue: 'false',
        },
        {
          name: 'limit',
          paramType: 'number',
          description: 'Maximum number of users',
          required: false,
          defaultValue: '50',
          validation: {
            minimum: 1,
            maximum: 200,
          },
        },
      ],
      handler: (async (args: {
        include_bots: boolean;
        limit: number;
      }): Promise<ExecutionResult> => {
        const client = getSlackClient();

        const response = await client.get<SlackResponse<SlackUser>>(
          `/users.list?limit=${args.limit}`
        );

        if (!response.ok || !response.data.ok) {
          return handleSlackError(response);
        }

        let users = response.data.members || [];

        if (!args.include_bots) {
          users = users.filter(u => !u.is_bot);
        }

        if (users.length === 0) {
          return ok('No users found.');
        }

        const output = users.map(u => {
          const admin = u.is_admin ? ' 👑' : '';
          const bot = u.is_bot ? ' 🤖' : '';
          const name = u.real_name || u.profile.display_name || u.name;
          return `@${u.name} - ${name}${admin}${bot}`;
        }).join('\n');

        return ok(
          `Found ${users.length} users:\n\n${output}`,
          { users: users.map(u => ({ id: u.id, name: u.name })) }
        );
      }) as ToolHandler,
    },

    {
      name: 'get-user-info',
      description: 'Get information about a user',
      parameters: [
        {
          name: 'user',
          paramType: 'string',
          description: 'User ID or @username',
          required: true,
        },
      ],
      handler: (async (args: { user: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();

        // Resolve username to ID if needed
        let userId = args.user;
        if (args.user.startsWith('@')) {
          const usersResponse = await client.get<SlackResponse<SlackUser>>('/users.list?limit=200');
          if (usersResponse.ok && usersResponse.data.ok) {
            const user = usersResponse.data.members?.find(
              u => u.name === args.user.slice(1)
            );
            if (user) {
              userId = user.id;
            } else {
              return err(`User ${args.user} not found`);
            }
          }
        }

        const response = await client.get<{ ok: boolean; user: SlackUser }>(
          `/users.info?user=${userId}`
        );

        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }

        const u = response.data.user;
        const output = [
          `User: @${u.name}`,
          `ID: ${u.id}`,
          `Name: ${u.real_name || 'N/A'}`,
          `Display Name: ${u.profile.display_name || 'N/A'}`,
          u.profile.email ? `Email: ${u.profile.email}` : null,
          `Admin: ${u.is_admin ? 'Yes' : 'No'}`,
          `Bot: ${u.is_bot ? 'Yes' : 'No'}`,
        ].filter(Boolean).join('\n');

        return ok(output, { user: u });
      }) as ToolHandler,
    },

    // ========================================
    // Message Management Tools
    // ========================================
    {
      name: 'update-message',
      description: 'Update an existing message',
      parameters: [
        { name: 'channel', paramType: 'string', description: 'Channel ID', required: true },
        { name: 'ts', paramType: 'string', description: 'Message timestamp', required: true },
        { name: 'text', paramType: 'string', description: 'New message text', required: true },
      ],
      handler: (async (args: { channel: string; ts: string; text: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.post<SlackMessage>('/chat.update', {
          channel: args.channel,
          ts: args.ts,
          text: args.text,
        });
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok(`Message updated in ${args.channel}`, { ts: args.ts });
      }) as ToolHandler,
    },

    {
      name: 'delete-message',
      description: 'Delete a message',
      parameters: [
        { name: 'channel', paramType: 'string', description: 'Channel ID', required: true },
        { name: 'ts', paramType: 'string', description: 'Message timestamp', required: true },
      ],
      handler: (async (args: { channel: string; ts: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.post<{ ok: boolean }>('/chat.delete', {
          channel: args.channel,
          ts: args.ts,
        });
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok('Message deleted');
      }) as ToolHandler,
    },

    {
      name: 'schedule-message',
      description: 'Schedule a message for later',
      parameters: [
        { name: 'channel', paramType: 'string', description: 'Channel ID or name', required: true },
        { name: 'text', paramType: 'string', description: 'Message text', required: true },
        { name: 'post_at', paramType: 'number', description: 'Unix timestamp for when to post', required: true },
      ],
      handler: (async (args: { channel: string; text: string; post_at: number }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.post<{ ok: boolean; scheduled_message_id: string }>('/chat.scheduleMessage', {
          channel: args.channel,
          text: args.text,
          post_at: args.post_at,
        });
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok(`Message scheduled for ${new Date(args.post_at * 1000).toISOString()}`, { scheduled_message_id: response.data.scheduled_message_id });
      }) as ToolHandler,
    },

    {
      name: 'get-thread-replies',
      description: 'Get replies in a thread',
      parameters: [
        { name: 'channel', paramType: 'string', description: 'Channel ID', required: true },
        { name: 'ts', paramType: 'string', description: 'Thread parent message timestamp', required: true },
        { name: 'limit', paramType: 'number', description: 'Maximum replies to return', required: false, defaultValue: '50' },
      ],
      handler: (async (args: { channel: string; ts: string; limit?: number }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.get<{ ok: boolean; messages: Array<{ user: string; text: string; ts: string }> }>(
          `/conversations.replies?channel=${args.channel}&ts=${args.ts}&limit=${args.limit || 50}`
        );
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        const messages = response.data.messages || [];
        return ok(`Found ${messages.length} messages in thread`, { messages });
      }) as ToolHandler,
    },

    // ========================================
    // Reaction Tools
    // ========================================
    {
      name: 'react',
      description: 'Add a reaction to a message',
      parameters: [
        { name: 'channel', paramType: 'string', description: 'Channel ID', required: true },
        { name: 'ts', paramType: 'string', description: 'Message timestamp', required: true },
        { name: 'emoji', paramType: 'string', description: 'Emoji name (without colons)', required: true },
      ],
      handler: (async (args: { channel: string; ts: string; emoji: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.post<{ ok: boolean }>('/reactions.add', {
          channel: args.channel,
          timestamp: args.ts,
          name: args.emoji.replace(/:/g, ''),
        });
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok(`Added :${args.emoji}: reaction`);
      }) as ToolHandler,
    },

    {
      name: 'unreact',
      description: 'Remove a reaction from a message',
      parameters: [
        { name: 'channel', paramType: 'string', description: 'Channel ID', required: true },
        { name: 'ts', paramType: 'string', description: 'Message timestamp', required: true },
        { name: 'emoji', paramType: 'string', description: 'Emoji name (without colons)', required: true },
      ],
      handler: (async (args: { channel: string; ts: string; emoji: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.post<{ ok: boolean }>('/reactions.remove', {
          channel: args.channel,
          timestamp: args.ts,
          name: args.emoji.replace(/:/g, ''),
        });
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok(`Removed :${args.emoji}: reaction`);
      }) as ToolHandler,
    },

    // ========================================
    // Pin Tools
    // ========================================
    {
      name: 'pin-message',
      description: 'Pin a message to a channel',
      parameters: [
        { name: 'channel', paramType: 'string', description: 'Channel ID', required: true },
        { name: 'ts', paramType: 'string', description: 'Message timestamp', required: true },
      ],
      handler: (async (args: { channel: string; ts: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.post<{ ok: boolean }>('/pins.add', {
          channel: args.channel,
          timestamp: args.ts,
        });
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok('Message pinned');
      }) as ToolHandler,
    },

    {
      name: 'unpin-message',
      description: 'Unpin a message from a channel',
      parameters: [
        { name: 'channel', paramType: 'string', description: 'Channel ID', required: true },
        { name: 'ts', paramType: 'string', description: 'Message timestamp', required: true },
      ],
      handler: (async (args: { channel: string; ts: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.post<{ ok: boolean }>('/pins.remove', {
          channel: args.channel,
          timestamp: args.ts,
        });
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok('Message unpinned');
      }) as ToolHandler,
    },

    // ========================================
    // Channel Management Tools
    // ========================================
    {
      name: 'create-channel',
      description: 'Create a new channel',
      parameters: [
        { name: 'name', paramType: 'string', description: 'Channel name (lowercase, no spaces)', required: true },
        { name: 'is_private', paramType: 'boolean', description: 'Create as private channel', required: false, defaultValue: 'false' },
      ],
      handler: (async (args: { name: string; is_private?: boolean }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.post<{ ok: boolean; channel: SlackChannel }>('/conversations.create', {
          name: args.name.toLowerCase().replace(/\s+/g, '-'),
          is_private: args.is_private || false,
        });
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok(`Channel #${response.data.channel.name} created`, { channel: response.data.channel });
      }) as ToolHandler,
    },

    {
      name: 'archive-channel',
      description: 'Archive a channel',
      parameters: [
        { name: 'channel', paramType: 'string', description: 'Channel ID', required: true },
      ],
      handler: (async (args: { channel: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.post<{ ok: boolean }>('/conversations.archive', {
          channel: args.channel,
        });
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok('Channel archived');
      }) as ToolHandler,
    },

    {
      name: 'invite-to-channel',
      description: 'Invite users to a channel',
      parameters: [
        { name: 'channel', paramType: 'string', description: 'Channel ID', required: true },
        { name: 'users', paramType: 'string', description: 'Comma-separated user IDs', required: true },
      ],
      handler: (async (args: { channel: string; users: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.post<{ ok: boolean; channel: SlackChannel }>('/conversations.invite', {
          channel: args.channel,
          users: args.users,
        });
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok('Users invited to channel');
      }) as ToolHandler,
    },

    {
      name: 'kick-from-channel',
      description: 'Remove a user from a channel',
      parameters: [
        { name: 'channel', paramType: 'string', description: 'Channel ID', required: true },
        { name: 'user', paramType: 'string', description: 'User ID to remove', required: true },
      ],
      handler: (async (args: { channel: string; user: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.post<{ ok: boolean }>('/conversations.kick', {
          channel: args.channel,
          user: args.user,
        });
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok('User removed from channel');
      }) as ToolHandler,
    },

    {
      name: 'set-topic',
      description: 'Set the topic for a channel',
      parameters: [
        { name: 'channel', paramType: 'string', description: 'Channel ID', required: true },
        { name: 'topic', paramType: 'string', description: 'New topic text', required: true },
      ],
      handler: (async (args: { channel: string; topic: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.post<{ ok: boolean; topic: string }>('/conversations.setTopic', {
          channel: args.channel,
          topic: args.topic,
        });
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok(`Topic set: ${args.topic}`);
      }) as ToolHandler,
    },

    {
      name: 'set-purpose',
      description: 'Set the purpose for a channel',
      parameters: [
        { name: 'channel', paramType: 'string', description: 'Channel ID', required: true },
        { name: 'purpose', paramType: 'string', description: 'New purpose text', required: true },
      ],
      handler: (async (args: { channel: string; purpose: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.post<{ ok: boolean; purpose: string }>('/conversations.setPurpose', {
          channel: args.channel,
          purpose: args.purpose,
        });
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok(`Purpose set: ${args.purpose}`);
      }) as ToolHandler,
    },

    // ========================================
    // User Status Tools
    // ========================================
    {
      name: 'get-presence',
      description: 'Get a user\'s presence status',
      parameters: [
        { name: 'user', paramType: 'string', description: 'User ID', required: true },
      ],
      handler: (async (args: { user: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.get<{ ok: boolean; presence: string; online: boolean; auto_away: boolean }>(
          `/users.getPresence?user=${args.user}`
        );
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        const p = response.data;
        return ok(`Presence: ${p.presence}${p.auto_away ? ' (auto away)' : ''}`, { presence: p.presence, online: p.online });
      }) as ToolHandler,
    },

    {
      name: 'set-status',
      description: 'Set your status (requires users:write scope)',
      parameters: [
        { name: 'status_text', paramType: 'string', description: 'Status text', required: true },
        { name: 'status_emoji', paramType: 'string', description: 'Status emoji (e.g., :coffee:)', required: false },
        { name: 'expiration', paramType: 'number', description: 'Unix timestamp when status expires', required: false },
      ],
      handler: (async (args: { status_text: string; status_emoji?: string; expiration?: number }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const profile: Record<string, unknown> = {
          status_text: args.status_text,
        };
        if (args.status_emoji) profile.status_emoji = args.status_emoji;
        if (args.expiration) profile.status_expiration = args.expiration;

        const response = await client.post<{ ok: boolean }>('/users.profile.set', {
          profile: JSON.stringify(profile),
        });
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok(`Status set: ${args.status_emoji || ''} ${args.status_text}`);
      }) as ToolHandler,
    },

    // ========================================
    // Search Tools
    // ========================================
    {
      name: 'search-messages',
      description: 'Search for messages (requires search:read scope)',
      parameters: [
        { name: 'query', paramType: 'string', description: 'Search query', required: true },
        { name: 'count', paramType: 'number', description: 'Number of results', required: false, defaultValue: '20' },
        { name: 'sort', paramType: 'string', description: 'Sort by: score or timestamp', required: false, defaultValue: 'score' },
      ],
      handler: (async (args: { query: string; count?: number; sort?: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const response = await client.get<{ ok: boolean; messages: { matches: Array<{ text: string; channel: { name: string }; ts: string }> } }>(
          `/search.messages?query=${encodeURIComponent(args.query)}&count=${args.count || 20}&sort=${args.sort || 'score'}`
        );
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        const matches = response.data.messages?.matches || [];
        const output = matches.map(m => `#${m.channel.name}: ${m.text.substring(0, 100)}...`).join('\n');
        return ok(`Found ${matches.length} messages:\n\n${output}`, { matches });
      }) as ToolHandler,
    },

    // ========================================
    // File Tools
    // ========================================
    {
      name: 'upload-file',
      description: 'Upload content as a file (text/code snippets)',
      parameters: [
        { name: 'channels', paramType: 'string', description: 'Comma-separated channel IDs', required: true },
        { name: 'content', paramType: 'string', description: 'File content', required: true },
        { name: 'filename', paramType: 'string', description: 'Filename', required: true },
        { name: 'filetype', paramType: 'string', description: 'File type (e.g., text, javascript, python)', required: false },
        { name: 'title', paramType: 'string', description: 'File title', required: false },
        { name: 'initial_comment', paramType: 'string', description: 'Comment to add with file', required: false },
      ],
      handler: (async (args: { channels: string; content: string; filename: string; filetype?: string; title?: string; initial_comment?: string }): Promise<ExecutionResult> => {
        const client = getSlackClient();
        const body: Record<string, string> = {
          channels: args.channels,
          content: args.content,
          filename: args.filename,
        };
        if (args.filetype) body.filetype = args.filetype;
        if (args.title) body.title = args.title;
        if (args.initial_comment) body.initial_comment = args.initial_comment;

        const response = await client.post<{ ok: boolean; file: { id: string; name: string } }>('/files.upload', body);
        if (!response.ok || !response.data.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }
        return ok(`File uploaded: ${response.data.file.name}`, { file_id: response.data.file.id });
      }) as ToolHandler,
    },

    // ========================================
    // Status Tools
    // ========================================
    {
      name: 'test-connection',
      description: 'Test the Slack API connection and token',
      parameters: [],
      handler: (async (): Promise<ExecutionResult> => {
        const client = getSlackClient();

        const response = await client.get<{ ok: boolean; team: string; user: string }>(
          '/auth.test'
        );

        if (!response.ok) {
          return handleSlackError(response as HttpResponse<SlackResponse<unknown>>);
        }

        if (!response.data.ok) {
          return err('Authentication failed. Check your SLACK_TOKEN.');
        }

        return ok(
          `Connected to Slack!\nTeam: ${response.data.team}\nBot User: ${response.data.user}`,
          { team: response.data.team, user: response.data.user }
        );
      }) as ToolHandler,
    },
  ],
});
