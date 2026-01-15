/**
 * Skill Engine JavaScript/TypeScript SDK
 *
 * This SDK provides utilities for creating skills that can be compiled
 * to WebAssembly Components and executed by the Skill Engine runtime.
 *
 * @example
 * ```typescript
 * import { defineSkill, getConfig, ok, err, errors } from '@skill-engine/sdk';
 * import { SkillHttpClient, createAuthenticatedClient } from '@skill-engine/sdk/http';
 *
 * export default defineSkill({
 *   metadata: {
 *     name: 'my-skill',
 *     version: '1.0.0',
 *     description: 'My awesome skill',
 *     author: 'Your Name'
 *   },
 *   tools: [
 *     {
 *       name: 'greet',
 *       description: 'Greet someone',
 *       parameters: [
 *         {
 *           name: 'name',
 *           paramType: 'string',
 *           description: 'Name to greet',
 *           required: true,
 *           validation: { minLength: 1, maxLength: 100 }
 *         },
 *         {
 *           name: 'email',
 *           paramType: 'string',
 *           description: 'Email address',
 *           required: false,
 *           validation: { format: 'email' }
 *         }
 *       ],
 *       handler: async (args) => ok(`Hello, ${args.name}!`)
 *     }
 *   ]
 * });
 * ```
 */

// Re-export all types
export * from './types.js';

// Re-export HTTP client utilities
export * from './http.js';

// Re-export schema generation utilities
export * from './schema.js';

import type {
  SkillConfig,
  SkillMetadata,
  ToolDefinition,
  ExecutionResult,
  EnvironmentConfig,
  Parameter,
  ParameterType,
  StringFormat,
} from './types.js';

/**
 * Define a skill with metadata, tools, and optional configuration validation.
 *
 * This function returns an object with exports that match the WIT interface
 * expected by the Skill Engine runtime.
 *
 * @param config - Skill configuration object
 * @returns Skill exports object for Component Model
 *
 * @example
 * ```typescript
 * export default defineSkill({
 *   metadata: { name: 'example', version: '1.0.0', ... },
 *   tools: [{ name: 'my-tool', ... }]
 * });
 * ```
 */
export function defineSkill(config: SkillConfig) {
  // Validate configuration
  validateSkillConfig(config);

  return {
    /**
     * Get skill metadata
     * Maps to WIT: get-metadata: func() -> metadata
     */
    getMetadata(): SkillMetadata {
      return config.metadata;
    },

    /**
     * Get list of tools provided by this skill
     * Maps to WIT: get-tools: func() -> list<tool>
     */
    getTools(): ToolDefinition[] {
      return config.tools.map((tool) => ({
        name: tool.name,
        description: tool.description,
        parameters: tool.parameters,
      }));
    },

    /**
     * Execute a tool with provided arguments
     * Maps to WIT: execute-tool: func(tool-name: string, args: list<tuple<string, string>>) -> result<execution-result, string>
     *
     * @param toolName - Name of the tool to execute
     * @param args - Array of [key, value] tuples
     * @returns Result with execution-result on success or error string on failure
     */
    async executeTool(
      toolName: string,
      args: Array<[string, string]>
    ): Promise<{ ok?: ExecutionResult; err?: string }> {
      try {
        // Find the tool
        const tool = config.tools.find((t) => t.name === toolName);
        if (!tool) {
          return { err: `Unknown tool: ${toolName}` };
        }

        // Convert tuple array to object
        const argsObject: Record<string, any> = {};
        for (const [key, value] of args) {
          argsObject[key] = value;
        }

        // Validate required parameters
        for (const param of tool.parameters) {
          if (param.required && !(param.name in argsObject)) {
            return { err: `Missing required parameter: ${param.name}` };
          }
        }

        // Apply default values
        for (const param of tool.parameters) {
          if (!(param.name in argsObject) && param.defaultValue !== undefined) {
            argsObject[param.name] = param.defaultValue;
          }
        }

        // Parse and validate parameter values
        for (const param of tool.parameters) {
          if (param.name in argsObject) {
            const rawValue = argsObject[param.name];

            // Parse value based on type
            argsObject[param.name] = parseParameterValue(rawValue, param.paramType);

            // Validate against constraints if defined
            if (param.validation) {
              const validationErrors = validateParameter(
                argsObject[param.name],
                param
              );
              if (validationErrors.length > 0) {
                return {
                  err: `Validation error for '${param.name}': ${validationErrors.join(', ')}`
                };
              }
            }
          }
        }

        // Execute the tool handler
        const result = await Promise.resolve(tool.handler(argsObject));

        // Ensure result has correct structure
        return {
          ok: {
            success: result.success,
            output: result.output || '',
            errorMessage: result.errorMessage ?? null,
          }
        };
      } catch (error) {
        return {
          err: `Tool execution error: ${
            error instanceof Error ? error.message : String(error)
          }`
        };
      }
    },

    /**
     * Validate configuration
     * Maps to WIT: validate-config: func(config: list<config-value>) -> result<_, string>
     *
     * @param configValues - Configuration values from host
     * @returns Validation result (ok) or error string
     */
    async validateConfig(
      configValues: Array<{ key: string; value: string; secret: boolean }>
    ): Promise<{ ok?: null; err?: string }> {
      if (!config.validateConfig) {
        return { ok: null };
      }

      try {
        // Convert config values to environment variables format
        const configEnv: Record<string, string> = {};
        for (const { key, value } of configValues) {
          configEnv[key] = value;
        }

        // Call user's validator with config
        const result = await Promise.resolve(config.validateConfig(configEnv));
        return result;
      } catch (error) {
        return {
          err: `Configuration validation error: ${
            error instanceof Error ? error.message : String(error)
          }`,
        };
      }
    },
  };
}

/**
 * Get configuration values from environment variables.
 *
 * The Skill Engine runtime passes configuration as environment variables
 * with the SKILL_ prefix. This function provides type-safe access to them.
 *
 * @template T - Type of the configuration object
 * @returns Configuration object with all SKILL_* environment variables
 *
 * @example
 * ```typescript
 * interface MyConfig {
 *   API_KEY: string;
 *   REGION: string;
 * }
 *
 * const config = getConfig<MyConfig>();
 * console.log(config.API_KEY); // Type-safe access
 * ```
 */
export function getConfig<T extends EnvironmentConfig = EnvironmentConfig>(): T {
  const config: Record<string, string> = {};

  // Extract all SKILL_* environment variables
  for (const [key, value] of Object.entries(process.env)) {
    if (key.startsWith('SKILL_')) {
      // Remove SKILL_ prefix
      const configKey = key.slice(6);
      config[configKey] = value || '';
    }
  }

  return config as T;
}

/**
 * Validate skill configuration
 *
 * @param config - Skill configuration to validate
 * @throws Error if configuration is invalid
 */
function validateSkillConfig(config: SkillConfig): void {
  // Validate metadata
  if (!config.metadata) {
    throw new Error('Skill metadata is required');
  }
  if (!config.metadata.name || typeof config.metadata.name !== 'string') {
    throw new Error('Skill metadata.name must be a non-empty string');
  }
  if (!config.metadata.version || typeof config.metadata.version !== 'string') {
    throw new Error('Skill metadata.version must be a non-empty string');
  }
  if (!config.metadata.description || typeof config.metadata.description !== 'string') {
    throw new Error('Skill metadata.description must be a non-empty string');
  }
  if (!config.metadata.author || typeof config.metadata.author !== 'string') {
    throw new Error('Skill metadata.author must be a non-empty string');
  }

  // Validate tools
  if (!Array.isArray(config.tools)) {
    throw new Error('Skill tools must be an array');
  }
  if (config.tools.length === 0) {
    throw new Error('Skill must define at least one tool');
  }

  // Validate each tool
  const toolNames = new Set<string>();
  for (const tool of config.tools) {
    if (!tool.name || typeof tool.name !== 'string') {
      throw new Error('Tool name must be a non-empty string');
    }
    if (toolNames.has(tool.name)) {
      throw new Error(`Duplicate tool name: ${tool.name}`);
    }
    toolNames.add(tool.name);

    if (!tool.description || typeof tool.description !== 'string') {
      throw new Error(`Tool ${tool.name} must have a description`);
    }
    if (!Array.isArray(tool.parameters)) {
      throw new Error(`Tool ${tool.name} parameters must be an array`);
    }
    if (typeof tool.handler !== 'function') {
      throw new Error(`Tool ${tool.name} must have a handler function`);
    }

    // Validate parameters
    const paramNames = new Set<string>();
    for (const param of tool.parameters) {
      if (!param.name || typeof param.name !== 'string') {
        throw new Error(`Tool ${tool.name} has parameter with invalid name`);
      }
      if (paramNames.has(param.name)) {
        throw new Error(`Tool ${tool.name} has duplicate parameter: ${param.name}`);
      }
      paramNames.add(param.name);

      const validTypes: ParameterType[] = ['string', 'number', 'boolean', 'file', 'json', 'array', 'secret'];
      if (!param.paramType || !validTypes.includes(param.paramType)) {
        throw new Error(
          `Tool ${tool.name} parameter ${param.name} has invalid paramType: ${param.paramType}. Valid types: ${validTypes.join(', ')}`
        );
      }
      if (typeof param.required !== 'boolean') {
        throw new Error(
          `Tool ${tool.name} parameter ${param.name} required must be a boolean`
        );
      }
    }
  }
}

/**
 * Parse a raw parameter value into the appropriate type.
 *
 * @param value - Raw string value from arguments
 * @param paramType - Expected parameter type
 * @returns Parsed value
 */
function parseParameterValue(value: string, paramType: ParameterType): unknown {
  switch (paramType) {
    case 'number':
      const num = parseFloat(value);
      return isNaN(num) ? value : num;

    case 'boolean':
      return value.toLowerCase() === 'true';

    case 'json':
      try {
        return JSON.parse(value);
      } catch {
        return value;
      }

    case 'array':
      // Handle both JSON arrays and comma-separated values
      if (value.startsWith('[')) {
        try {
          return JSON.parse(value);
        } catch {
          // Fall through to comma-separated
        }
      }
      return value.split(',').map(s => s.trim());

    case 'string':
    case 'file':
    case 'secret':
    default:
      return value;
  }
}

/**
 * Validate a parameter value against its constraints.
 *
 * @param value - Parsed parameter value
 * @param param - Parameter definition with validation constraints
 * @returns Array of validation error messages (empty if valid)
 */
function validateParameter(value: unknown, param: Parameter): string[] {
  const errors: string[] = [];
  const v = param.validation;

  if (!v) return errors;

  // String validations
  if (typeof value === 'string') {
    if (v.minLength !== undefined && value.length < v.minLength) {
      errors.push(`must be at least ${v.minLength} characters`);
    }
    if (v.maxLength !== undefined && value.length > v.maxLength) {
      errors.push(`must be at most ${v.maxLength} characters`);
    }
    if (v.pattern) {
      const regex = new RegExp(v.pattern);
      if (!regex.test(value)) {
        errors.push(`must match pattern: ${v.pattern}`);
      }
    }
    if (v.enum && !v.enum.includes(value)) {
      errors.push(`must be one of: ${v.enum.join(', ')}`);
    }
    if (v.format) {
      const formatErrors = validateFormat(value, v.format);
      errors.push(...formatErrors);
    }
  }

  // Number validations
  if (typeof value === 'number') {
    if (v.minimum !== undefined && value < v.minimum) {
      errors.push(`must be at least ${v.minimum}`);
    }
    if (v.maximum !== undefined && value > v.maximum) {
      errors.push(`must be at most ${v.maximum}`);
    }
  }

  return errors;
}

/**
 * Validate a string against a format constraint.
 */
function validateFormat(value: string, format: StringFormat): string[] {
  const formatValidators: Record<string, RegExp> = {
    email: /^[^\s@]+@[^\s@]+\.[^\s@]+$/,
    url: /^https?:\/\/.+/,
    uri: /^[a-z][a-z0-9+.-]*:/i,
    uuid: /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
    date: /^\d{4}-\d{2}-\d{2}$/,
    datetime: /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/,
    time: /^\d{2}:\d{2}:\d{2}/,
    ipv4: /^(\d{1,3}\.){3}\d{1,3}$/,
    ipv6: /^([0-9a-f]{1,4}:){7}[0-9a-f]{1,4}$/i,
    hostname: /^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)*$/i,
  };

  const validator = formatValidators[format];
  if (validator && !validator.test(value)) {
    return [`invalid ${format} format`];
  }

  return [];
}
