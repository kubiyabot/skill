---
name: hello
description: Simple hello world skill for testing basic functionality
---

# Hello Skill

Simple hello world skill for testing basic functionality.

## When to Use This Skill

Use this skill when you need to:
- Test basic skill generation
- Validate minimal skill structure
- Demonstrate simple parameter handling
- Verify greeting functionality

## Quick Reference

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| `greet` | Greet a user by name | `name` (string, required) |

## Tools by Category

### General Operations

- **greet**: Greet a user by name

## Available Tools

### greet

Greet a user by name

**Parameters:**
- `name` (string, required): Name of the person to greet

**Usage:**
```bash
# Using MCP
mcp__skill-engine__execute(
  skill='hello',
  tool='greet',
  args={'name': 'World'}
)

# Using script
./scripts/greet.sh name=World
```

## Context Engineering

This skill supports context engineering features:
- **grep**: Filter output by pattern
- **max_output**: Limit response size
- **head/tail**: Get first/last N lines

Example:
```javascript
mcp__skill-engine__execute(
  skill='hello',
  tool='greet',
  args={'name': 'World'},
  max_output=1000
)
```
