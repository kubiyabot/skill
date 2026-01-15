/**
 * API with Authentication Pattern
 * ================================
 *
 * This pattern shows how to:
 * 1. Store API credentials securely via config
 * 2. Validate credentials are present
 * 3. Make authenticated API requests
 * 4. Handle authentication errors
 */

// Store config globally (set during validateConfig)
let config = {};

export function getMetadata() {
  return {
    name: "api-auth-example",
    version: "1.0.0",
    description: "Demonstrates API authentication patterns",
    author: "Skill Engine"
  };
}

export function getTools() {
  return [
    {
      name: "fetch-data",
      description: "Fetch data from authenticated API endpoint",
      parameters: [
        {
          name: "endpoint",
          paramType: "string",
          description: "API endpoint path (e.g., '/users', '/items')",
          required: true
        }
      ]
    },
    {
      name: "check-auth",
      description: "Verify API credentials are valid",
      parameters: []
    }
  ];
}

export async function executeTool(name, argsJson) {
  const args = JSON.parse(argsJson);

  // PATTERN: Always check for credentials first
  if (!config.api_key) {
    return {
      success: false,
      output: "",
      errorMessage: "API key not configured. Run: skill config api-auth-example"
    };
  }

  try {
    if (name === "fetch-data") {
      return await fetchData(args);
    }
    if (name === "check-auth") {
      return await checkAuth();
    }
    return {
      success: false,
      output: "",
      errorMessage: `Unknown tool: ${name}`
    };
  } catch (error) {
    return {
      success: false,
      output: "",
      errorMessage: `API error: ${error.message}`
    };
  }
}

/**
 * PATTERN: Make authenticated request
 */
async function fetchData(args) {
  const { endpoint } = args;
  const baseUrl = config.base_url || "https://api.example.com";

  // PATTERN: Add authentication header
  const response = await fetch(`${baseUrl}${endpoint}`, {
    headers: {
      // Common auth patterns:
      // 1. Bearer token
      "Authorization": `Bearer ${config.api_key}`,
      // 2. API key header (alternative)
      // "X-API-Key": config.api_key,
      // 3. Basic auth (alternative)
      // "Authorization": `Basic ${btoa(config.username + ':' + config.password)}`,
      "Content-Type": "application/json"
    }
  });

  // PATTERN: Handle auth-specific errors
  if (response.status === 401) {
    return {
      success: false,
      output: "",
      errorMessage: "Authentication failed. Check your API key."
    };
  }

  if (response.status === 403) {
    return {
      success: false,
      output: "",
      errorMessage: "Access forbidden. Your API key may not have permission for this resource."
    };
  }

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }

  const data = await response.json();
  return {
    success: true,
    output: JSON.stringify(data, null, 2),
    errorMessage: null
  };
}

/**
 * PATTERN: Validate credentials
 */
async function checkAuth() {
  const baseUrl = config.base_url || "https://api.example.com";

  // Many APIs have a "me" or "validate" endpoint
  const response = await fetch(`${baseUrl}/auth/validate`, {
    headers: {
      "Authorization": `Bearer ${config.api_key}`
    }
  });

  if (response.ok) {
    return {
      success: true,
      output: "Authentication successful. API key is valid.",
      errorMessage: null
    };
  }

  return {
    success: false,
    output: "",
    errorMessage: `Authentication failed: HTTP ${response.status}`
  };
}

/**
 * PATTERN: Validate and store config
 */
export function validateConfig(configJson) {
  if (!configJson) {
    // No config provided - will prompt when needed
    return { ok: null };
  }

  try {
    config = JSON.parse(configJson);

    // PATTERN: Check required fields
    if (!config.api_key) {
      return {
        ok: null,
        error: "Missing 'api_key' in configuration"
      };
    }

    // PATTERN: Validate format if needed
    if (config.api_key.length < 10) {
      return {
        ok: null,
        error: "API key appears to be invalid (too short)"
      };
    }

    return { ok: null };
  } catch (e) {
    return {
      ok: null,
      error: `Invalid configuration: ${e.message}`
    };
  }
}
