---
name: discord
version: 1.0.0
description: Discord server and bot management
author: Skill Engine
---

# Discord Skill

Send messages, manage channels, and interact with Discord servers through the API.

## Installation

```bash
skill install ./examples/wasm-skills/discord-skill
```

## Configuration

```bash
skill config discord --set DISCORD_BOT_TOKEN=your_bot_token
```

## Tools

### send-message
Send a message to a channel.

**Parameters:**
- `channel_id` (required, string): Channel ID
- `content` (required, string): Message content
- `embed` (optional, json): Rich embed object

**Example:**
```
skill run discord send-message --channel_id 123456789012345678 --content "Hello!"
```

### get-channel
Get information about a channel.

**Parameters:**
- `channel_id` (required, string): Channel ID

**Example:**
```
skill run discord get-channel --channel_id 123456789012345678
```

### list-channels
List channels in a server.

**Parameters:**
- `guild_id` (required, string): Server (guild) ID
- `type` (optional, string): Filter by type: text, voice, category

**Example:**
```
skill run discord list-channels --guild_id 987654321098765432
```

### create-thread
Create a thread from a message or in a channel.

**Parameters:**
- `channel_id` (required, string): Parent channel ID
- `name` (required, string): Thread name
- `message_id` (optional, string): Message to create thread from
- `auto_archive_duration` (optional, number): Minutes until auto-archive

**Example:**
```
skill run discord create-thread --channel_id 123456789 --name "Discussion"
```

### get-messages
Get recent messages from a channel.

**Parameters:**
- `channel_id` (required, string): Channel ID
- `limit` (optional, number): Number of messages (default: 50)
- `before` (optional, string): Get messages before this ID
- `after` (optional, string): Get messages after this ID

**Example:**
```
skill run discord get-messages --channel_id 123456789 --limit 20
```

### list-members
List members in a server.

**Parameters:**
- `guild_id` (required, string): Server ID
- `limit` (optional, number): Number of members (default: 100)

**Example:**
```
skill run discord list-members --guild_id 987654321098765432
```

### add-role
Add a role to a member.

**Parameters:**
- `guild_id` (required, string): Server ID
- `user_id` (required, string): User ID
- `role_id` (required, string): Role ID

**Example:**
```
skill run discord add-role --guild_id 987654321 --user_id 111222333 --role_id 444555666
```

### remove-role
Remove a role from a member.

**Parameters:**
- `guild_id` (required, string): Server ID
- `user_id` (required, string): User ID
- `role_id` (required, string): Role ID

**Example:**
```
skill run discord remove-role --guild_id 987654321 --user_id 111222333 --role_id 444555666
```

### add-reaction
Add a reaction to a message.

**Parameters:**
- `channel_id` (required, string): Channel ID
- `message_id` (required, string): Message ID
- `emoji` (required, string): Emoji (unicode or custom format)

**Example:**
```
skill run discord add-reaction --channel_id 123456789 --message_id 111222333 --emoji "👍"
```
