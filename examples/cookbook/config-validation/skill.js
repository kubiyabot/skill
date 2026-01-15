/**
 * Config Validation Pattern
 * ==========================
 *
 * This pattern shows how to:
 * 1. Define required configuration
 * 2. Validate config values
 * 3. Provide helpful error messages
 * 4. Use defaults for optional config
 */

// Store config globally
let config = {};

// PATTERN: Define config schema
const CONFIG_SCHEMA = {
  api_key: {
    required: true,
    type: 'string',
    minLength: 10,
    description: 'API key for authentication'
  },
  base_url: {
    required: false,
    type: 'string',
    pattern: /^https?:\/\//,
    default: 'https://api.example.com',
    description: 'Base URL for API requests'
  },
  timeout: {
    required: false,
    type: 'number',
    min: 1000,
    max: 60000,
    default: 30000,
    description: 'Request timeout in milliseconds'
  },
  debug: {
    required: false,
    type: 'boolean',
    default: false,
    description: 'Enable debug logging'
  }
};

export function getMetadata() {
  return {
    name: "config-validation-example",
    version: "1.0.0",
    description: "Demonstrates configuration validation patterns",
    author: "Skill Engine"
  };
}

export function getTools() {
  return [
    {
      name: "show-config",
      description: "Display current configuration (masks sensitive values)",
      parameters: []
    },
    {
      name: "test-connection",
      description: "Test API connection with current configuration",
      parameters: []
    }
  ];
}

export function executeTool(name, argsJson) {
  if (name === "show-config") {
    return showConfig();
  }
  if (name === "test-connection") {
    return testConnection();
  }
  return {
    success: false,
    output: "",
    errorMessage: `Unknown tool: ${name}`
  };
}

function showConfig() {
  // PATTERN: Mask sensitive values
  const masked = {
    ...config,
    api_key: config.api_key ? maskValue(config.api_key) : '(not set)'
  };

  return {
    success: true,
    output: JSON.stringify(masked, null, 2),
    errorMessage: null
  };
}

function testConnection() {
  if (!config.api_key) {
    return {
      success: false,
      output: "",
      errorMessage: "API key not configured. Run: skill config config-validation-example"
    };
  }

  return {
    success: true,
    output: `Configuration valid. Would connect to: ${config.base_url}`,
    errorMessage: null
  };
}

/**
 * PATTERN: Comprehensive config validation
 */
export function validateConfig(configJson) {
  if (!configJson) {
    // PATTERN: Return requirements for empty config
    const required = Object.entries(CONFIG_SCHEMA)
      .filter(([_, schema]) => schema.required)
      .map(([name, schema]) => `- ${name}: ${schema.description}`);

    if (required.length > 0) {
      return {
        ok: null,
        error: `Missing required configuration:\n${required.join('\n')}\n\nRun: skill config config-validation-example`
      };
    }
    return { ok: null };
  }

  let parsed;
  try {
    parsed = JSON.parse(configJson);
  } catch (e) {
    return {
      ok: null,
      error: `Invalid JSON: ${e.message}`
    };
  }

  // PATTERN: Validate each field against schema
  const errors = [];

  for (const [name, schema] of Object.entries(CONFIG_SCHEMA)) {
    const value = parsed[name];

    // Check required
    if (schema.required && (value === undefined || value === null || value === '')) {
      errors.push(`'${name}' is required (${schema.description})`);
      continue;
    }

    // Skip validation for undefined optional values
    if (value === undefined) {
      continue;
    }

    // Check type
    if (schema.type === 'string' && typeof value !== 'string') {
      errors.push(`'${name}' must be a string`);
      continue;
    }
    if (schema.type === 'number' && typeof value !== 'number') {
      errors.push(`'${name}' must be a number`);
      continue;
    }
    if (schema.type === 'boolean' && typeof value !== 'boolean') {
      errors.push(`'${name}' must be a boolean`);
      continue;
    }

    // Check string constraints
    if (schema.type === 'string' && typeof value === 'string') {
      if (schema.minLength && value.length < schema.minLength) {
        errors.push(`'${name}' must be at least ${schema.minLength} characters`);
      }
      if (schema.pattern && !schema.pattern.test(value)) {
        errors.push(`'${name}' has invalid format`);
      }
    }

    // Check number constraints
    if (schema.type === 'number' && typeof value === 'number') {
      if (schema.min !== undefined && value < schema.min) {
        errors.push(`'${name}' must be at least ${schema.min}`);
      }
      if (schema.max !== undefined && value > schema.max) {
        errors.push(`'${name}' must be at most ${schema.max}`);
      }
    }
  }

  // PATTERN: Report all errors at once
  if (errors.length > 0) {
    return {
      ok: null,
      error: `Configuration validation failed:\n- ${errors.join('\n- ')}`
    };
  }

  // PATTERN: Apply defaults and store config
  config = {};
  for (const [name, schema] of Object.entries(CONFIG_SCHEMA)) {
    config[name] = parsed[name] !== undefined ? parsed[name] : schema.default;
  }

  return { ok: null };
}

/**
 * PATTERN: Mask sensitive values for display
 */
function maskValue(value) {
  if (typeof value !== 'string') return value;
  if (value.length <= 8) return '****';
  return value.substring(0, 4) + '****' + value.substring(value.length - 4);
}
