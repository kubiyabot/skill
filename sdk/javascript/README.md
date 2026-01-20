# Skill Engine JavaScript/TypeScript SDK

Build portable, secure skills for the Skill Engine using JavaScript or TypeScript. Skills compile to WebAssembly Components and run in a sandboxed environment with WASI Preview 2.

## Features

- 🚀 **Zero Config** - Just write JavaScript/TypeScript, no build setup needed
- 🔒 **Secure** - Sandboxed execution with capability-based security
- 📦 **Portable** - Compile to WASM Components that run anywhere
- 🎯 **Type Safe** - Full TypeScript support with type definitions
- 🔧 **Simple API** - Intuitive skill definition with `defineSkill()`
- 🌐 **Multi-Language** - Interoperate with skills written in Rust, Go, Python

## Installation

```bash
npm install @kubiya/skill-engine-sdk
```

## Quick Start

### 1. Create a Skill

Create `skill.ts`:

```typescript
import { defineSkill, getConfig } from '@skill-engine/sdk';

export default defineSkill({
  metadata: {
    name: 'my-skill',
    version: '1.0.0',
    description: 'My awesome skill',
    author: 'Your Name'
  },
  tools: [
    {
      name: 'greet',
      description: 'Greet someone by name',
      parameters: [
        {
          name: 'name',
          paramType: 'string',
          description: 'Name to greet',
          required: true
        }
      ],
      handler: async (args) => {
        return {
          success: true,
          output: `Hello, ${args.name}!`,
          errorMessage: null
        };
      }
    }
  ]
});
```

### 2. Build and Compile

```bash
# Install dependencies
npm install

# Build TypeScript to JavaScript
npm run build

# Compile to WASM Component
npx skill-sdk componentize dist/skill.js -o skill.wasm
```

### 3. Run with Skill Engine

```bash
# Install the skill
skill install ./skill.wasm --instance my-instance

# Run a tool
skill run my-skill greet name=World
# Output: Hello, World!
```

## API Reference

### `defineSkill(config)`

Define a skill with metadata, tools, and optional configuration validation.

```typescript
import { defineSkill, type SkillConfig } from '@skill-engine/sdk';

const skill = defineSkill({
  metadata: {
    name: 'example-skill',
    version: '1.0.0',
    description: 'Example skill',
    author: 'Your Name'
  },
  tools: [
    // Tool definitions...
  ],
  validateConfig: (config) => {
    // Optional: validate configuration
    if (!config.API_KEY) {
      return { err: 'API_KEY is required' };
    }
    return { ok: null };
  }
});
```

#### Parameters

- **metadata** (required): Skill metadata
  - `name`: Unique identifier (kebab-case)
  - `version`: Semantic version (e.g., "1.0.0")
  - `description`: Brief description
  - `author`: Author name or organization

- **tools** (required): Array of tool definitions
  - `name`: Tool identifier (kebab-case)
  - `description`: What the tool does
  - `parameters`: Array of parameter definitions
  - `handler`: Async function that executes the tool

- **validateConfig** (optional): Configuration validator function
  - Receives `config` object with environment variables
  - Returns `{ ok: null }` on success or `{ err: string }` on failure

### `getConfig<T>()`

Get type-safe access to configuration from environment variables. The Skill Engine passes configuration as `SKILL_*` environment variables.

```typescript
import { getConfig, type EnvironmentConfig } from '@skill-engine/sdk';

interface MyConfig extends EnvironmentConfig {
  API_KEY: string;
  REGION: string;
  DEBUG: string;
}

const config = getConfig<MyConfig>();
console.log(config.API_KEY);    // Type-safe access
console.log(config.REGION);     // Type-safe access
console.log(config.DEBUG);      // Type-safe access
```

## Tool Handlers

Tool handlers receive parsed arguments and return execution results:

```typescript
{
  name: 'process-data',
  description: 'Process data with options',
  parameters: [
    {
      name: 'input',
      paramType: 'string',
      description: 'Input data',
      required: true
    },
    {
      name: 'format',
      paramType: 'string',
      description: 'Output format',
      required: false,
      defaultValue: 'json'
    }
  ],
  handler: async (args) => {
    try {
      // Access arguments
      const input = args.input;
      const format = args.format; // Uses default if not provided

      // Do work...
      const result = processData(input, format);

      // Return success
      return {
        success: true,
        output: JSON.stringify(result),
        errorMessage: null
      };
    } catch (error) {
      // Return error
      return {
        success: false,
        output: '',
        errorMessage: error.message
      };
    }
  }
}
```

### Parameter Types

- `string`: Text value
- `number`: Numeric value
- `boolean`: True/false value
- `file`: File path (pre-opened by host)

## Configuration

Skills can access configuration through environment variables prefixed with `SKILL_`:

```typescript
import { getConfig } from '@skill-engine/sdk';

// User configures: skill config my-skill --set API_KEY=abc123
// Skill receives: SKILL_API_KEY=abc123

const config = getConfig();
const apiKey = config.API_KEY; // "abc123"
```

### Configuration Validation

Validate configuration before execution:

```typescript
defineSkill({
  // ...
  validateConfig: (config) => {
    // Check required values
    if (!config.API_KEY) {
      return { err: 'API_KEY is required' };
    }

    // Validate format
    if (!config.API_KEY.startsWith('sk-')) {
      return { err: 'Invalid API_KEY format' };
    }

    // Check dependencies
    if (config.ENABLE_FEATURE && !config.FEATURE_KEY) {
      return { err: 'FEATURE_KEY required when ENABLE_FEATURE is true' };
    }

    return { ok: null };
  }
});
```

## Advanced Examples

### External API Integration

```typescript
import { defineSkill, getConfig } from '@skill-engine/sdk';
import fetch from 'node-fetch';

interface Config {
  API_KEY: string;
  BASE_URL: string;
}

export default defineSkill({
  metadata: {
    name: 'api-skill',
    version: '1.0.0',
    description: 'Call external API',
    author: 'Your Name'
  },
  tools: [
    {
      name: 'fetch-data',
      description: 'Fetch data from API',
      parameters: [
        {
          name: 'endpoint',
          paramType: 'string',
          description: 'API endpoint path',
          required: true
        }
      ],
      handler: async (args) => {
        const config = getConfig<Config>();

        try {
          const response = await fetch(
            `${config.BASE_URL}${args.endpoint}`,
            {
              headers: {
                'Authorization': `Bearer ${config.API_KEY}`
              }
            }
          );

          const data = await response.json();

          return {
            success: true,
            output: JSON.stringify(data, null, 2),
            errorMessage: null
          };
        } catch (error) {
          return {
            success: false,
            output: '',
            errorMessage: error.message
          };
        }
      }
    }
  ],
  validateConfig: (config) => {
    if (!config.API_KEY) {
      return { err: 'API_KEY is required' };
    }
    if (!config.BASE_URL) {
      return { err: 'BASE_URL is required' };
    }
    return { ok: null };
  }
});
```

### File Operations

```typescript
import { defineSkill } from '@skill-engine/sdk';
import { readFile, writeFile } from 'fs/promises';

export default defineSkill({
  metadata: {
    name: 'file-skill',
    version: '1.0.0',
    description: 'File operations skill',
    author: 'Your Name'
  },
  tools: [
    {
      name: 'read-file',
      description: 'Read file contents',
      parameters: [
        {
          name: 'path',
          paramType: 'file',
          description: 'File path to read',
          required: true
        }
      ],
      handler: async (args) => {
        try {
          const content = await readFile(args.path, 'utf-8');
          return {
            success: true,
            output: content,
            errorMessage: null
          };
        } catch (error) {
          return {
            success: false,
            output: '',
            errorMessage: `Failed to read file: ${error.message}`
          };
        }
      }
    },
    {
      name: 'write-file',
      description: 'Write content to file',
      parameters: [
        {
          name: 'path',
          paramType: 'file',
          description: 'File path to write',
          required: true
        },
        {
          name: 'content',
          paramType: 'string',
          description: 'Content to write',
          required: true
        }
      ],
      handler: async (args) => {
        try {
          await writeFile(args.path, args.content, 'utf-8');
          return {
            success: true,
            output: `Wrote ${args.content.length} bytes to ${args.path}`,
            errorMessage: null
          };
        } catch (error) {
          return {
            success: false,
            output: '',
            errorMessage: `Failed to write file: ${error.message}`
          };
        }
      }
    }
  ]
});
```

## CLI Tool: `skill-sdk`

The SDK includes a CLI tool for development tasks:

### Componentize

Compile JavaScript skill to WASM Component:

```bash
npx skill-sdk componentize dist/skill.js -o skill.wasm
```

Options:
- `-o, --output <file>`: Output WASM file (required)
- `--wit <dir>`: WIT directory path (default: ../../../wit)
- `--world <name>`: WIT world name (default: skill)
- `--debug`: Enable debug output

### Validate

Validate skill structure and exports:

```bash
npx skill-sdk validate dist/skill.js
```

This checks:
- Required exports exist (getMetadata, getTools, executeTool, validateConfig)
- Metadata is complete
- Tools are properly defined
- Parameters are valid

## TypeScript Configuration

Recommended `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ES2022",
    "moduleResolution": "bundler",
    "declaration": true,
    "strict": true,
    "outDir": "./dist"
  },
  "include": ["src/**/*"]
}
```

## Package Structure

```
my-skill/
├── package.json
├── tsconfig.json
├── src/
│   └── skill.ts          # Main skill definition
├── dist/                 # Compiled JavaScript (generated)
│   └── skill.js
└── skill.wasm           # WASM Component (generated)
```

## Best Practices

1. **Use TypeScript** for type safety and better IDE support
2. **Validate configuration** to fail fast on missing/invalid config
3. **Handle errors gracefully** and return descriptive error messages
4. **Keep tools focused** - one tool should do one thing well
5. **Use semantic versioning** for your skill versions
6. **Document parameters** with clear descriptions
7. **Test locally** before deploying to production

## Troubleshooting

### "Module not found" errors

Make sure to install all dependencies:
```bash
npm install @skill-engine/sdk
npm install  # Install other dependencies from package.json
```

### Componentization fails

Check that:
- Your skill exports `default` from `defineSkill()`
- All required exports are present (getMetadata, getTools, executeTool, validateConfig)
- The WIT directory exists at `wit/skill-interface.wit`
- You have `@bytecodealliance/jco` installed

### Configuration not available

Configuration is passed as environment variables with `SKILL_` prefix:
- User sets: `skill config my-skill --set API_KEY=value`
- Skill receives: `process.env.SKILL_API_KEY`
- Access via: `getConfig().API_KEY`

## Examples

See the `examples/` directory for complete working examples:
- **simple-skill**: Basic "Hello World" skill
- **aws-skill**: AWS integration with S3, EC2, Lambda
- **github-skill**: GitHub API integration

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR on GitHub.

## Links

- [Skill Engine Documentation](https://github.com/skill-engine/skill-engine)
- [WASM Component Model](https://component-model.bytecodealliance.org/)
- [WASI Preview 2](https://github.com/WebAssembly/WASI/tree/main/preview2)
