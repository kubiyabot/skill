# Discord Skill

Send messages, manage channels, and interact with Discord servers through the API.

## Overview

The Discord skill provides AI agents with Discord integration capabilities. Send messages to channels, manage server members, create threads, and automate Discord workflows through the Discord API.

**Runtime**: WASM (JavaScript/TypeScript)
**Source**: [examples/wasm-skills/discord-skill](https://github.com/kubiyabot/skill/tree/main/examples/wasm-skills/discord-skill)

## Installation

```bash
# Install the skill
skill install github:kubiyabot/skill:discord

# Or from local directory
skill install ./examples/wasm-skills/discord-skill
```

## Configuration

Configure your Discord bot token:

```bash
skill config discord --set bot_token=YOUR_BOT_TOKEN
```

Or via environment variables:

```bash
export DISCORD_BOT_TOKEN=your_bot_token
```

### Getting a Bot Token

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application
3. Navigate to "Bot" section
4. Click "Add Bot" and copy the token
5. Enable required intents (Message Content, Server Members, etc.)

## Tools Reference

### send-message

Send a message to a channel.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `channel_id` | string | Yes | Channel ID |
| `content` | string | Yes | Message content |
| `embed` | json | No | Rich embed object |

**Examples:**

```bash
# Send simple message
skill run discord send-message \
  --channel_id 123456789012345678 \
  --content "Hello from the skill engine!"

# Send with embed
skill run discord send-message \
  --channel_id 123456789012345678 \
  --content "Check out this update:" \
  --embed '{
    "title": "Deployment Complete",
    "description": "Version 2.3.0 deployed to production",
    "color": 5763719,
    "fields": [
      {"name": "Environment", "value": "Production", "inline": true},
      {"name": "Status", "value": "Success", "inline": true}
    ]
  }'
```

**Output:**
```json
{
  "id": "1234567890123456789",
  "channel_id": "123456789012345678",
  "content": "Hello from the skill engine!",
  "timestamp": "2025-01-14T10:30:00.000Z"
}
```

### get-channel

Get information about a channel.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `channel_id` | string | Yes | Channel ID |

**Examples:**

```bash
skill run discord get-channel --channel_id 123456789012345678
```

**Output:**
```json
{
  "id": "123456789012345678",
  "name": "general",
  "type": 0,
  "guild_id": "987654321098765432",
  "topic": "General discussion",
  "position": 0
}
```

### list-channels

List channels in a server.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `guild_id` | string | Yes | Server (guild) ID |
| `type` | string | No | Filter by type: `text`, `voice`, `category` |

**Examples:**

```bash
# List all channels
skill run discord list-channels --guild_id 987654321098765432

# List only text channels
skill run discord list-channels \
  --guild_id 987654321098765432 \
  --type text
```

**Output:**
```json
{
  "channels": [
    {"id": "123...", "name": "general", "type": "text", "position": 0},
    {"id": "124...", "name": "announcements", "type": "text", "position": 1},
    {"id": "125...", "name": "Voice Chat", "type": "voice", "position": 2}
  ]
}
```

### create-thread

Create a thread from a message or in a channel.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `channel_id` | string | Yes | Parent channel ID |
| `name` | string | Yes | Thread name |
| `message_id` | string | No | Message to create thread from |
| `auto_archive_duration` | number | No | Minutes until auto-archive (60, 1440, 4320, 10080) |

**Examples:**

```bash
# Create thread from message
skill run discord create-thread \
  --channel_id 123456789012345678 \
  --name "Discussion: New Feature" \
  --message_id 111222333444555666

# Create standalone thread
skill run discord create-thread \
  --channel_id 123456789012345678 \
  --name "Weekly Sync" \
  --auto_archive_duration 1440
```

### get-messages

Get recent messages from a channel.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `channel_id` | string | Yes | Channel ID |
| `limit` | number | No | Number of messages (default: 50, max: 100) |
| `before` | string | No | Get messages before this ID |
| `after` | string | No | Get messages after this ID |

**Examples:**

```bash
# Get recent messages
skill run discord get-messages \
  --channel_id 123456789012345678 \
  --limit 20

# Get messages after specific message
skill run discord get-messages \
  --channel_id 123456789012345678 \
  --after 111222333444555666
```

### list-members

List members in a server.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `guild_id` | string | Yes | Server ID |
| `limit` | number | No | Number of members (default: 100) |
| `after` | string | No | Get members after this user ID |

**Examples:**

```bash
skill run discord list-members \
  --guild_id 987654321098765432 \
  --limit 50
```

**Output:**
```json
{
  "members": [
    {
      "user": {"id": "111...", "username": "alice", "discriminator": "1234"},
      "nick": "Alice",
      "roles": ["role_id_1", "role_id_2"],
      "joined_at": "2024-01-15T10:00:00.000Z"
    }
  ]
}
```

### add-role

Add a role to a member.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `guild_id` | string | Yes | Server ID |
| `user_id` | string | Yes | User ID |
| `role_id` | string | Yes | Role ID |

**Examples:**

```bash
skill run discord add-role \
  --guild_id 987654321098765432 \
  --user_id 111222333444555666 \
  --role_id 999888777666555444
```

### remove-role

Remove a role from a member.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `guild_id` | string | Yes | Server ID |
| `user_id` | string | Yes | User ID |
| `role_id` | string | Yes | Role ID |

**Examples:**

```bash
skill run discord remove-role \
  --guild_id 987654321098765432 \
  --user_id 111222333444555666 \
  --role_id 999888777666555444
```

### add-reaction

Add a reaction to a message.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `channel_id` | string | Yes | Channel ID |
| `message_id` | string | Yes | Message ID |
| `emoji` | string | Yes | Emoji (unicode or custom format) |

**Examples:**

```bash
# Unicode emoji
skill run discord add-reaction \
  --channel_id 123456789012345678 \
  --message_id 111222333444555666 \
  --emoji "👍"

# Custom emoji
skill run discord add-reaction \
  --channel_id 123456789012345678 \
  --message_id 111222333444555666 \
  --emoji "custom_emoji:999888777"
```

## Common Workflows

### Deployment Notification

```bash
# 1. Send deployment notification to channel
skill run discord send-message \
  --channel_id 123456789012345678 \
  --content "@here Deployment starting" \
  --embed '{
    "title": "🚀 Deployment Started",
    "description": "Deploying version 2.3.0 to production",
    "color": 16776960,
    "fields": [
      {"name": "Service", "value": "api-gateway", "inline": true},
      {"name": "Initiated by", "value": "CI/CD Pipeline", "inline": true}
    ],
    "timestamp": "2025-01-14T10:30:00.000Z"
  }'

# 2. Create thread for deployment discussion
skill run discord create-thread \
  --channel_id 123456789012345678 \
  --name "Deployment 2.3.0 Discussion" \
  --message_id <returned_message_id>
```

### Incident Response

```bash
# 1. Alert the team
skill run discord send-message \
  --channel_id 123456789012345678 \
  --content "@oncall Critical alert triggered!" \
  --embed '{
    "title": "🚨 INCIDENT: High Error Rate",
    "description": "Error rate exceeded 5% threshold",
    "color": 15158332,
    "fields": [
      {"name": "Service", "value": "payment-api"},
      {"name": "Error Rate", "value": "7.2%"},
      {"name": "Started", "value": "5 minutes ago"}
    ]
  }'

# 2. Create incident thread
skill run discord create-thread \
  --channel_id 123456789012345678 \
  --name "INC-2025-001: Payment API Errors"
```

### Onboarding Automation

```bash
# 1. Add welcome role to new member
skill run discord add-role \
  --guild_id 987654321098765432 \
  --user_id 111222333444555666 \
  --role_id 999888777666555444

# 2. Send welcome message
skill run discord send-message \
  --channel_id 123456789012345678 \
  --content "Welcome <@111222333444555666>! Check out #getting-started"
```

## Security Considerations

- **Bot Token**: Keep your bot token secret; never commit to version control
- **Permissions**: Only grant necessary bot permissions in Discord
- **Rate Limits**: Discord has strict rate limits; avoid bulk operations
- **Intents**: Enable only required Gateway Intents
- **Message Content**: Message Content intent requires verification for bots in 100+ servers

## Troubleshooting

### Authentication Failed

```
Error: 401 Unauthorized
```

**Solution**: Verify your bot token is correct:

```bash
skill config discord --set bot_token=YOUR_CORRECT_TOKEN
```

### Missing Permissions

```
Error: Missing Access
```

**Solution**: Ensure the bot has required permissions in the server and channel.

### Rate Limited

```
Error: 429 Too Many Requests
```

**Solution**: Discord rate limits are strict. Add delays between bulk operations.

### Invalid Channel

```
Error: Unknown Channel
```

**Solution**: Verify channel ID and ensure bot has access to the channel.

## Integration with Claude Code

```bash
# Natural language commands
"Send a message to the deployments channel"
"Create a thread for the incident discussion"
"List all text channels in the server"
"Add the developer role to the new team member"
```

## Next Steps

- [Slack Skill](./slack.md) - Slack messaging integration
- [Linear Skill](./linear.md) - Issue tracking
- [PagerDuty Skill](./pagerduty.md) - Incident management
- [Discord Developer Portal](https://discord.com/developers/docs)
