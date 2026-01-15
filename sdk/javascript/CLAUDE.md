# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the JavaScript/TypeScript SDK for creating Skill Engine skills. Skills built with this SDK compile to WebAssembly Components and run in a sandboxed WASI Preview 2 environment.

## Build & Development Commands

```bash
# Build SDK (compiles TypeScript to JavaScript)
npm run build

# Run tests
npm test

# Type check without emitting
npx tsc --noEmit
```

## Architecture

### Source Files (`src/`)

```
src/
├── index.ts    # Main entry - defineSkill(), getConfig(), validation
├── types.ts    # Type definitions, error helpers (ok, err, errors)
├── http.ts     # SkillHttpClient, createAuthenticatedClient
├── schema.ts   # JSON Schema generation utilities
└── cli.ts      # skill-sdk CLI (componentize, validate commands)
```

### Core Pattern

Skills are defined using `defineSkill()` which returns an object matching the WIT interface:

```typescript
export default defineSkill({
  metadata: { name, version, description, author },
  tools: [
    {
      name: 'tool-name',
      parameters: [{ name, paramType, description, required }],
      handler: async (args) => ok('output') // or err('message')
    }
  ],
  validateConfig: (config) => { ok: null } // optional
});
```

The returned object exposes:
- `getMetadata()` - Returns skill metadata
- `getTools()` - Returns tool definitions (without handlers)
- `executeTool(toolName, args)` - Executes tool and returns Result
- `validateConfig(configValues)` - Validates configuration

### HTTP Client

`createAuthenticatedClient()` reads tokens from skill config (environment variables with `SKILL_` prefix):

```typescript
const client = createAuthenticatedClient({
  baseUrl: 'https://api.example.com',
  authType: 'bearer',  // or 'basic', 'api-key'
  tokenKey: 'API_KEY'  // reads from SKILL_API_KEY env var
});
```

### Error Handling

Use the provided helpers for consistent error handling:

```typescript
import { ok, err, errors } from '@skill-engine/sdk';

// Success
return ok('Operation completed', { data: result });

// Error with message
return err('Something went wrong');

// Structured errors
return err('Not found', errors.notFound('Resource'));
return err('Auth failed', errors.auth());
return err('Rate limited', errors.rateLimit(60));
```

### Parameter Types

Supported: `string`, `number`, `boolean`, `file`, `json`, `array`, `secret`

Parameters can include validation:
```typescript
{
  name: 'email',
  paramType: 'string',
  validation: { format: 'email', minLength: 5 }
}
```

## CLI Tool

The SDK includes `skill-sdk` CLI:

```bash
# Compile JavaScript to WASM Component
npx skill-sdk componentize dist/skill.js -o skill.wasm

# Validate skill structure
npx skill-sdk validate dist/skill.js
```

## Skill Project Structure

When creating a new skill:
```
my-skill/
├── package.json       # Must have "type": "module"
├── tsconfig.json      # Target ES2022, module ES2022
├── src/skill.ts       # Main skill using defineSkill()
├── dist/              # Compiled output
└── SKILL.md           # Optional documentation with YAML frontmatter
```

## Key Patterns

1. **All exports are async-compatible** - handlers can return Promise or direct value
2. **Arguments arrive as `[key, value]` tuples** - parsed to object before handler
3. **Config uses SKILL_ prefix** - `skill config set API_KEY=x` becomes `SKILL_API_KEY`
4. **WASM runs via jco/componentize-js** - uses StarlingMonkey for JS execution
