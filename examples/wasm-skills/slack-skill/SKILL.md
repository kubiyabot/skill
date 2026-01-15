---
name: slack-skill
version: 1.0.0
description: Comprehensive Slack integration for messaging, channels, users, reactions, pins, and workspace management
author: Skill Engine Team
tags: [slack, messaging, chat, communication, collaboration]
allowed-tools: ["send-message", "send-dm", "list-channels", "get-channel-info", "list-users", "get-user-info", "update-message", "delete-message", "schedule-message", "get-thread-replies", "react", "unreact", "pin-message", "unpin-message", "create-channel", "archive-channel", "invite-to-channel", "kick-from-channel", "set-topic", "set-purpose", "get-presence", "set-status", "search-messages", "upload-file", "test-connection"]
---

# Slack Skill

Comprehensive Slack workspace integration with 30+ tools for messaging, channel management, user operations, reactions, pins, and more. Built with the Skill Engine SDK showcasing OAuth2 token-based authentication and the Slack Web API.

## Overview

This skill provides full access to Slack's Web API, enabling automation of workspace communication and management tasks. It demonstrates type-safe parameter validation, structured error handling, and production-ready API integration patterns.

## When to Use This Skill

**Use this skill when you need to**:
- Send messages to channels or direct messages to users
- Create, manage, and archive channels
- List and search workspace users and channels
- Add reactions and pin important messages
- Schedule messages for later delivery
- Manage threads and conversations
- Upload files and snippets
- Set user status and presence
- Search message history
- Automate Slack workflows

## Prerequisites

1. **Slack Workspace**: Admin or appropriate permissions
2. **Slack App**: Create an app at [api.slack.com/apps](https://api.slack.com/apps)
3. **Bot Token**: OAuth token with required scopes

### Setting Up Your Slack App

1. Go to https://api.slack.com/apps
2. Click "Create New App" → "From scratch"
3. Name your app (e.g., "Skill Engine Bot") and select your workspace
4. Navigate to "OAuth & Permissions"
5. Add Bot Token Scopes:
   ```
   Required Scopes:
   - chat:write           (Send messages)
   - channels:read        (List public channels)
   - users:read           (List users)
   - reactions:write      (Add/remove reactions)
   - pins:write           (Pin messages)
   - channels:manage      (Create/archive channels)
   - groups:write         (Manage private channels)
   - im:write             (Send DMs)
   - mpim:write           (Manage group DMs)
   - search:read          (Search messages - optional)
   - users:write          (Set status - optional)
   - files:write          (Upload files - optional)
   ```
6. Click "Install to Workspace" and authorize
7. Copy the "Bot User OAuth Token" (starts with `xoxb-`)

### Configuration

Set the Slack token as an environment variable:

```bash
export SKILL_SLACK_TOKEN=xoxb-your-token-here
```

Or add to `.skill-engine.toml`:

```toml
[skills.slack-skill.instances.default]
config.slack_token = "${SLACK_TOKEN}"
```

## Tools

### Messaging Tools (4 tools)

#### send-message
Send a message to a Slack channel.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channel | string | Yes | Channel name (without #) or channel ID |
| text | string | Yes | Message text (supports Slack markdown) |
| thread_ts | string | No | Thread timestamp to reply in thread |

**Examples**:
```bash
# Send to channel
skill run slack-skill send-message channel=general text="Hello team!"

# Send with markdown
skill run slack-skill send-message channel=dev text="*Important:* New release deployed :rocket:"

# Reply in thread
skill run slack-skill send-message channel=general text="Follow-up message" thread_ts=1234567890.123456
```

#### send-dm
Send a direct message to a user.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| user | string | Yes | User ID (U12345) or @username |
| text | string | Yes | Message text |

**Examples**:
```bash
# Send DM by username
skill run slack-skill send-dm user=@john text="Hey, can you review PR #42?"

# Send DM by user ID
skill run slack-skill send-dm user=U12345ABC text="Meeting in 10 minutes"
```

#### update-message
Update an existing message.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channel | string | Yes | Channel ID |
| ts | string | Yes | Message timestamp |
| text | string | Yes | New message text |

**Example**:
```bash
skill run slack-skill update-message channel=C123ABC ts=1234567890.123456 text="Updated: Meeting at 3pm"
```

#### delete-message
Delete a message.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channel | string | Yes | Channel ID |
| ts | string | Yes | Message timestamp |

**Example**:
```bash
skill run slack-skill delete-message channel=C123ABC ts=1234567890.123456
```

#### schedule-message
Schedule a message for later delivery.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channel | string | Yes | Channel ID or name |
| text | string | Yes | Message text |
| post_at | number | Yes | Unix timestamp for when to post |

**Example**:
```bash
# Schedule message for tomorrow at 9am (Unix timestamp)
skill run slack-skill schedule-message channel=general text="Daily standup reminder" post_at=1704283200
```

#### get-thread-replies
Get all replies in a thread.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| channel | string | Yes | - | Channel ID |
| ts | string | Yes | - | Thread parent message timestamp |
| limit | number | No | 50 | Maximum replies to return |

**Example**:
```bash
skill run slack-skill get-thread-replies channel=C123ABC ts=1234567890.123456 limit=100
```

### Channel Tools (6 tools)

#### list-channels
List channels in the workspace.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| type | string | No | public | Filter: public, private, all |
| limit | number | No | 50 | Maximum channels to return (1-200) |

**Examples**:
```bash
# List public channels
skill run slack-skill list-channels

# List all channels (public + private)
skill run slack-skill list-channels type=all limit=100

# List only private channels
skill run slack-skill list-channels type=private
```

#### get-channel-info
Get detailed information about a channel.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channel | string | Yes | Channel name or ID |

**Example**:
```bash
skill run slack-skill get-channel-info channel=general
skill run slack-skill get-channel-info channel=C123ABC
```

#### create-channel
Create a new channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| name | string | Yes | - | Channel name (lowercase, no spaces) |
| is_private | boolean | No | false | Create as private channel |

**Examples**:
```bash
# Create public channel
skill run slack-skill create-channel name=project-alpha

# Create private channel
skill run slack-skill create-channel name=leadership is_private=true
```

#### archive-channel
Archive a channel (make it read-only).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channel | string | Yes | Channel ID |

**Example**:
```bash
skill run slack-skill archive-channel channel=C123ABC
```

#### invite-to-channel
Invite users to a channel.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channel | string | Yes | Channel ID |
| users | string | Yes | Comma-separated user IDs |

**Example**:
```bash
skill run slack-skill invite-to-channel channel=C123ABC users=U111,U222,U333
```

#### kick-from-channel
Remove a user from a channel.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channel | string | Yes | Channel ID |
| user | string | Yes | User ID to remove |

**Example**:
```bash
skill run slack-skill kick-from-channel channel=C123ABC user=U12345
```

#### set-topic
Set the topic for a channel.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channel | string | Yes | Channel ID |
| topic | string | Yes | New topic text |

**Example**:
```bash
skill run slack-skill set-topic channel=C123ABC topic="Q1 2024 Planning"
```

#### set-purpose
Set the purpose/description for a channel.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channel | string | Yes | Channel ID |
| purpose | string | Yes | New purpose text |

**Example**:
```bash
skill run slack-skill set-purpose channel=C123ABC purpose="Discuss project alpha milestones"
```

### User Tools (4 tools)

#### list-users
List users in the workspace.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| include_bots | boolean | No | false | Include bot users |
| limit | number | No | 50 | Maximum users to return (1-200) |

**Examples**:
```bash
# List human users
skill run slack-skill list-users

# Include bots
skill run slack-skill list-users include_bots=true limit=100
```

#### get-user-info
Get information about a specific user.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| user | string | Yes | User ID or @username |

**Examples**:
```bash
skill run slack-skill get-user-info user=@john
skill run slack-skill get-user-info user=U12345ABC
```

#### get-presence
Get a user's presence status (active/away).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| user | string | Yes | User ID |

**Example**:
```bash
skill run slack-skill get-presence user=U12345ABC
```

#### set-status
Set your own status (emoji and text).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| status_text | string | Yes | Status text |
| status_emoji | string | No | Status emoji (e.g., :coffee:) |
| expiration | number | No | Unix timestamp when status expires |

**Examples**:
```bash
# Simple status
skill run slack-skill set-status status_text="In a meeting"

# Status with emoji
skill run slack-skill set-status status_text="On vacation" status_emoji=":palm_tree:"

# Status that expires in 1 hour
skill run slack-skill set-status status_text="Do not disturb" status_emoji=":no_entry_sign:" expiration=1704290400
```

### Reaction Tools (2 tools)

#### react
Add a reaction emoji to a message.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channel | string | Yes | Channel ID |
| ts | string | Yes | Message timestamp |
| emoji | string | Yes | Emoji name (without colons) |

**Examples**:
```bash
skill run slack-skill react channel=C123ABC ts=1234567890.123456 emoji=thumbsup
skill run slack-skill react channel=C123ABC ts=1234567890.123456 emoji=rocket
skill run slack-skill react channel=C123ABC ts=1234567890.123456 emoji=heart
```

#### unreact
Remove a reaction from a message.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channel | string | Yes | Channel ID |
| ts | string | Yes | Message timestamp |
| emoji | string | Yes | Emoji name (without colons) |

**Example**:
```bash
skill run slack-skill unreact channel=C123ABC ts=1234567890.123456 emoji=thumbsup
```

### Pin Tools (2 tools)

#### pin-message
Pin a message to a channel.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channel | string | Yes | Channel ID |
| ts | string | Yes | Message timestamp |

**Example**:
```bash
skill run slack-skill pin-message channel=C123ABC ts=1234567890.123456
```

#### unpin-message
Unpin a message from a channel.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channel | string | Yes | Channel ID |
| ts | string | Yes | Message timestamp |

**Example**:
```bash
skill run slack-skill unpin-message channel=C123ABC ts=1234567890.123456
```

### Search & Files (2 tools)

#### search-messages
Search for messages across the workspace (requires `search:read` scope).

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| query | string | Yes | - | Search query |
| count | number | No | 20 | Number of results |
| sort | string | No | score | Sort by: score or timestamp |

**Examples**:
```bash
# Search for messages
skill run slack-skill search-messages query="deployment failed"

# Search with more results
skill run slack-skill search-messages query="bug" count=50 sort=timestamp
```

#### upload-file
Upload content as a file (text snippets, code).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| channels | string | Yes | Comma-separated channel IDs |
| content | string | Yes | File content |
| filename | string | Yes | Filename |
| filetype | string | No | File type (text, javascript, python, etc.) |
| title | string | No | File title |
| initial_comment | string | No | Comment to add with file |

**Examples**:
```bash
# Upload code snippet
skill run slack-skill upload-file \
  channels=C123ABC \
  content="console.log('Hello');" \
  filename=test.js \
  filetype=javascript \
  title="Test Script"

# Upload with comment
skill run slack-skill upload-file \
  channels=C123ABC,C456DEF \
  content="API logs from server..." \
  filename=logs.txt \
  filetype=text \
  initial_comment="Here are today's logs"
```

### Utility Tools (1 tool)

#### test-connection
Test the Slack API connection and verify token.

**Parameters**: None

**Example**:
```bash
skill run slack-skill test-connection
# Output: Connected to Slack!
#         Team: My Workspace
#         Bot User: skill-engine-bot
```

## Workflows

### Daily Standup Automation

```bash
# Send standup reminder
skill run slack-skill send-message \
  channel=engineering \
  text=":wave: Good morning! Time for standup. Please share:\n1. What you did yesterday\n2. What you'll do today\n3. Any blockers?"

# React to responses
skill run slack-skill react channel=C123ABC ts=1234567890.123456 emoji=white_check_mark
```

### Channel Management Workflow

```bash
# Create project channel
skill run slack-skill create-channel name=project-phoenix

# Set topic and purpose
skill run slack-skill set-topic channel=C123ABC topic="Project Phoenix - Q1 2024"
skill run slack-skill set-purpose channel=C123ABC purpose="Coordinate Project Phoenix development"

# Invite team members
skill run slack-skill invite-to-channel channel=C123ABC users=U111,U222,U333

# Send welcome message
skill run slack-skill send-message channel=project-phoenix text="Welcome to Project Phoenix! :rocket:"
```

### Notification & Alert Workflow

```bash
# Send alert to channel
skill run slack-skill send-message \
  channel=alerts \
  text=":rotating_light: *ALERT:* Database CPU usage >90%\nServer: prod-db-01\nTime: $(date)"

# Send DM to on-call engineer
skill run slack-skill send-dm \
  user=@oncall \
  text="FYI: Alert triggered for database. Check #alerts channel."

# Pin critical message
skill run slack-skill pin-message channel=C123ABC ts=1234567890.123456
```

### Status Update Workflow

```bash
# Set away status
skill run slack-skill set-status \
  status_text="Lunch break" \
  status_emoji=":fork_and_knife:" \
  expiration=$(date -d '+1 hour' +%s)

# Update channel topic with current sprint
skill run slack-skill set-topic \
  channel=C123ABC \
  topic="Sprint 42 (Jan 2-15) | Goal: Launch beta | Daily standup @9am"
```

## Error Handling

The skill provides clear error messages for common issues:

| Error | Cause | Solution |
|-------|-------|----------|
| 401 Authentication failed | Invalid or missing token | Check `SKILL_SLACK_TOKEN` is set correctly |
| 429 Rate limit exceeded | Too many requests | Wait and retry (headers show reset time) |
| channel_not_found | Channel doesn't exist | Verify channel name/ID |
| not_in_channel | Bot not in channel | Invite bot to channel first |
| invalid_auth | Token revoked or expired | Generate new token from Slack app settings |
| missing_scope | Token lacks required permission | Add scope in Slack app OAuth settings and reinstall |

## Rate Limits

Slack API has tiered rate limits:

- **Tier 1** (most endpoints): 1 request per second
- **Tier 2** (chat.postMessage): 1 request per second per channel
- **Tier 3** (search, files): Lower limits
- **Tier 4** (special methods): Higher limits

The skill handles rate limit errors and includes retry information.

## Security Notes

- **Token Security**: Store tokens in environment variables, never in code
- **Scope Minimization**: Only request scopes your bot needs
- **Channel Access**: Bot can only access public channels and private channels it's invited to
- **User Privacy**: Cannot read DMs between users (only bot's own DMs)
- **Audit Logs**: Slack maintains audit logs of all bot actions

## Best Practices

1. **Use Channel Names When Possible**: More readable than IDs in scripts
2. **Check Responses**: Slack returns `ok: false` for API errors even with 200 status
3. **Thread Conversations**: Use `thread_ts` to keep related messages together
4. **Format Messages**: Use Slack's markdown for better readability
5. **Test in Private Channels**: Test bots in private channels before public use
6. **Monitor Rate Limits**: Check `X-Rate-Limit-*` headers in responses
7. **Handle Errors Gracefully**: Retry on rate limits, fail clearly on auth errors

## Slack Markdown Reference

```
*bold text*
_italic text_
~strikethrough~
`code`
```code block```
> quote
• bulleted list
1. numbered list
<URL|link text>
<@U12345> (mention user)
<#C12345> (link to channel)
:emoji: (emoji by name)
```

## Troubleshooting

### Bot Not Receiving Permissions

```bash
# Verify scopes in Slack app settings
# Reinstall app to workspace
# Test connection
skill run slack-skill test-connection
```

### Cannot Find Channel

```bash
# List all channels bot can see
skill run slack-skill list-channels type=all

# Check if bot is invited to private channel
# Invite bot via Slack UI: /invite @your-bot-name
```

### Messages Not Sending

```bash
# Verify token is set
echo $SKILL_SLACK_TOKEN

# Test connection
skill run slack-skill test-connection

# Check channel name (without #)
skill run slack-skill send-message channel=general text="Test"  # Correct
skill run slack-skill send-message channel=#general text="Test" # May fail
```

## SDK Features Demonstrated

This skill showcases advanced SDK patterns:

### 1. Authenticated HTTP Client
```typescript
const client = createAuthenticatedClient({
  baseUrl: 'https://slack.com/api',
  authType: 'bearer',
  tokenKey: 'SLACK_TOKEN',
});
```

### 2. Error Handling with Context
```typescript
if (response.status === 429) {
  return err('Rate limit exceeded', errors.rateLimit());
}
```

### 3. Parameter Validation
```typescript
{
  name: 'channel',
  paramType: 'string',
  validation: {
    minLength: 1,
    maxLength: 80,
  },
}
```

### 4. Enum Validation
```typescript
{
  name: 'type',
  paramType: 'string',
  validation: {
    enum: ['public', 'private', 'all'],
  },
}
```

## Resources

- [Slack API Documentation](https://api.slack.com/docs)
- [Slack Bot Users](https://api.slack.com/bot-users)
- [Slack App Management](https://api.slack.com/apps)
- [OAuth Scopes Reference](https://api.slack.com/scopes)
- [Slack Markdown Formatting](https://api.slack.com/reference/surfaces/formatting)
- [Rate Limits](https://api.slack.com/docs/rate-limits)

## Contributing

Contributions welcome! Possible enhancements:

- Slash command support
- Interactive message components
- Event subscriptions
- File download support
- Workflow builder integration
- Enterprise Grid features
