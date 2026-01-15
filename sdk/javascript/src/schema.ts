/**
 * JSON Schema generation for Skill Engine SDK
 *
 * Generates JSON Schema from tool definitions for:
 * - MCP (Model Context Protocol) integration
 * - Documentation generation
 * - Input validation
 *
 * @example
 * ```typescript
 * import { generateToolSchema, generateSkillSchema } from '@skill-engine/sdk';
 *
 * const tool: ToolDefinition = {
 *   name: 'create-user',
 *   description: 'Create a new user',
 *   parameters: [
 *     { name: 'email', paramType: 'string', validation: { format: 'email' } },
 *     { name: 'age', paramType: 'number', validation: { minimum: 0, maximum: 150 } }
 *   ]
 * };
 *
 * const schema = generateToolSchema(tool);
 * // Returns JSON Schema object for the tool's parameters
 * ```
 */

import type {
  ToolDefinition,
  Parameter,
  SkillConfig,
  JsonSchema,
  ParameterType,
} from './types.js';

/**
 * Generate JSON Schema for a tool's parameters.
 *
 * @param tool - Tool definition
 * @returns JSON Schema object
 *
 * @example
 * ```typescript
 * const schema = generateToolSchema(myTool);
 * // {
 * //   type: 'object',
 * //   title: 'create-user',
 * //   description: 'Create a new user',
 * //   properties: { ... },
 * //   required: ['email']
 * // }
 * ```
 */
export function generateToolSchema(tool: ToolDefinition): JsonSchema {
  const properties: Record<string, JsonSchema> = {};
  const required: string[] = [];

  for (const param of tool.parameters) {
    properties[param.name] = parameterToSchema(param);
    if (param.required) {
      required.push(param.name);
    }
  }

  return {
    type: 'object',
    title: tool.name,
    description: tool.description,
    properties,
    required: required.length > 0 ? required : undefined,
    additionalProperties: false,
  };
}

/**
 * Convert a parameter definition to JSON Schema.
 */
function parameterToSchema(param: Parameter): JsonSchema {
  const schema: JsonSchema = {
    type: paramTypeToJsonType(param.paramType),
    description: param.description,
  };

  // Add default value
  if (param.defaultValue !== undefined) {
    schema.default = parseDefaultValue(param.defaultValue, param.paramType);
  }

  // Add validation constraints
  if (param.validation) {
    const v = param.validation;

    if (v.pattern) schema.pattern = v.pattern;
    if (v.minLength !== undefined) schema.minLength = v.minLength;
    if (v.maxLength !== undefined) schema.maxLength = v.maxLength;
    if (v.minimum !== undefined) schema.minimum = v.minimum;
    if (v.maximum !== undefined) schema.maximum = v.maximum;
    if (v.enum) schema.enum = v.enum;
    if (v.format) schema.format = v.format;
  }

  // Special handling for array type
  if (param.paramType === 'array') {
    schema.items = { type: 'string' };
  }

  // Special handling for json type
  if (param.paramType === 'json') {
    schema.additionalProperties = true;
  }

  return schema;
}

/**
 * Convert parameter type to JSON Schema type.
 */
function paramTypeToJsonType(paramType: ParameterType): string {
  switch (paramType) {
    case 'string':
    case 'file':
    case 'secret':
      return 'string';
    case 'number':
      return 'number';
    case 'boolean':
      return 'boolean';
    case 'json':
      return 'object';
    case 'array':
      return 'array';
    default:
      return 'string';
  }
}

/**
 * Parse default value to appropriate type.
 */
function parseDefaultValue(value: string, paramType: ParameterType): unknown {
  switch (paramType) {
    case 'number':
      return parseFloat(value);
    case 'boolean':
      return value.toLowerCase() === 'true';
    case 'json':
      try {
        return JSON.parse(value);
      } catch {
        return value;
      }
    case 'array':
      return value.split(',').map(s => s.trim());
    default:
      return value;
  }
}

/**
 * Generate complete skill schema with all tools.
 *
 * @param config - Skill configuration
 * @returns Complete schema object
 *
 * @example
 * ```typescript
 * const skillSchema = generateSkillSchema(mySkillConfig);
 * // {
 * //   $schema: 'http://json-schema.org/draft-07/schema#',
 * //   title: 'my-skill',
 * //   description: 'My awesome skill',
 * //   version: '1.0.0',
 * //   tools: [ ... ]
 * // }
 * ```
 */
export function generateSkillSchema(config: SkillConfig): object {
  return {
    $schema: 'http://json-schema.org/draft-07/schema#',
    title: config.metadata.name,
    description: config.metadata.description,
    version: config.metadata.version,
    author: config.metadata.author,
    tools: config.tools.map(tool => ({
      name: tool.name,
      description: tool.description,
      inputSchema: generateToolSchema(tool),
    })),
  };
}

/**
 * Generate MCP-compatible tool definitions.
 *
 * MCP (Model Context Protocol) uses a specific format for tool definitions.
 * This function generates that format from skill tools.
 *
 * @param config - Skill configuration
 * @returns Array of MCP tool definitions
 *
 * @example
 * ```typescript
 * const mcpTools = generateMcpTools(mySkillConfig);
 * // [
 * //   {
 * //     name: 'my-skill:create-user',
 * //     description: 'Create a new user',
 * //     inputSchema: { ... }
 * //   }
 * // ]
 * ```
 */
export function generateMcpTools(config: SkillConfig): object[] {
  return config.tools.map(tool => ({
    name: `${config.metadata.name}:${tool.name}`,
    description: tool.description,
    inputSchema: generateToolSchema(tool),
  }));
}

/**
 * Validate a value against a JSON Schema.
 *
 * Simple validation for common cases. For complex validation,
 * use a dedicated library like Ajv.
 *
 * @param value - Value to validate
 * @param schema - JSON Schema to validate against
 * @returns Validation errors (empty array if valid)
 */
export function validateAgainstSchema(
  value: unknown,
  schema: JsonSchema
): string[] {
  const errors: string[] = [];

  if (schema.type === 'object' && typeof value === 'object' && value !== null) {
    const obj = value as Record<string, unknown>;

    // Check required properties
    if (schema.required) {
      for (const prop of schema.required) {
        if (!(prop in obj)) {
          errors.push(`Missing required property: ${prop}`);
        }
      }
    }

    // Validate each property
    if (schema.properties) {
      for (const [key, propSchema] of Object.entries(schema.properties)) {
        if (key in obj) {
          const propErrors = validateAgainstSchema(obj[key], propSchema);
          errors.push(...propErrors.map(e => `${key}: ${e}`));
        }
      }
    }
  } else if (schema.type === 'string' && typeof value === 'string') {
    validateString(value, schema, errors);
  } else if (schema.type === 'number' && typeof value === 'number') {
    validateNumber(value, schema, errors);
  } else if (schema.type === 'array' && Array.isArray(value)) {
    // Validate array items
    if (schema.items) {
      value.forEach((item, index) => {
        const itemErrors = validateAgainstSchema(item, schema.items!);
        errors.push(...itemErrors.map(e => `[${index}]: ${e}`));
      });
    }
  } else if (value !== undefined && value !== null) {
    // Type mismatch
    const actualType = Array.isArray(value) ? 'array' : typeof value;
    if (actualType !== schema.type) {
      errors.push(`Expected ${schema.type}, got ${actualType}`);
    }
  }

  return errors;
}

/**
 * Validate a string value.
 */
function validateString(value: string, schema: JsonSchema, errors: string[]): void {
  if (schema.minLength !== undefined && value.length < schema.minLength) {
    errors.push(`String too short (min: ${schema.minLength})`);
  }

  if (schema.maxLength !== undefined && value.length > schema.maxLength) {
    errors.push(`String too long (max: ${schema.maxLength})`);
  }

  if (schema.pattern) {
    const regex = new RegExp(schema.pattern);
    if (!regex.test(value)) {
      errors.push(`String does not match pattern: ${schema.pattern}`);
    }
  }

  if (schema.enum && !schema.enum.includes(value)) {
    errors.push(`Value must be one of: ${schema.enum.join(', ')}`);
  }

  if (schema.format) {
    validateFormat(value, schema.format, errors);
  }
}

/**
 * Validate a number value.
 */
function validateNumber(value: number, schema: JsonSchema, errors: string[]): void {
  if (schema.minimum !== undefined && value < schema.minimum) {
    errors.push(`Value too small (min: ${schema.minimum})`);
  }

  if (schema.maximum !== undefined && value > schema.maximum) {
    errors.push(`Value too large (max: ${schema.maximum})`);
  }
}

/**
 * Validate a string format.
 */
function validateFormat(value: string, format: string, errors: string[]): void {
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
    errors.push(`Invalid ${format} format`);
  }
}
