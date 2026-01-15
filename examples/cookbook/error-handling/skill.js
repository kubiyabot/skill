/**
 * Error Handling Pattern
 * ======================
 *
 * This pattern shows how to:
 * 1. Handle different types of errors
 * 2. Return meaningful error messages
 * 3. Implement retry logic
 * 4. Log errors appropriately
 */

export function getMetadata() {
  return {
    name: "error-handling-example",
    version: "1.0.0",
    description: "Demonstrates error handling patterns",
    author: "Skill Engine"
  };
}

export function getTools() {
  return [
    {
      name: "fetch-with-retry",
      description: "Fetch data with automatic retry on failure",
      parameters: [
        {
          name: "url",
          paramType: "string",
          description: "URL to fetch",
          required: true
        },
        {
          name: "retries",
          paramType: "integer",
          description: "Number of retry attempts (default: 3)",
          required: false
        }
      ]
    },
    {
      name: "validate-input",
      description: "Demonstrates input validation patterns",
      parameters: [
        {
          name: "email",
          paramType: "string",
          description: "Email address to validate",
          required: true
        },
        {
          name: "age",
          paramType: "integer",
          description: "Age (must be positive)",
          required: false
        }
      ]
    }
  ];
}

export async function executeTool(name, argsJson) {
  // PATTERN: Parse args in try-catch
  let args;
  try {
    args = JSON.parse(argsJson);
  } catch (e) {
    return {
      success: false,
      output: "",
      errorMessage: "Invalid JSON input"
    };
  }

  try {
    if (name === "fetch-with-retry") {
      return await fetchWithRetry(args);
    }
    if (name === "validate-input") {
      return validateInput(args);
    }
    return {
      success: false,
      output: "",
      errorMessage: `Unknown tool: ${name}`
    };
  } catch (error) {
    // PATTERN: Top-level error handler
    return handleError(error);
  }
}

/**
 * PATTERN: Retry logic with exponential backoff
 */
async function fetchWithRetry(args) {
  const { url, retries = 3 } = args;

  // PATTERN: Validate required parameters
  if (!url) {
    return {
      success: false,
      output: "",
      errorMessage: "URL is required"
    };
  }

  // PATTERN: Validate URL format
  try {
    new URL(url);
  } catch (e) {
    return {
      success: false,
      output: "",
      errorMessage: `Invalid URL format: ${url}`
    };
  }

  let lastError;

  for (let attempt = 1; attempt <= retries; attempt++) {
    try {
      const response = await fetch(url);

      // PATTERN: Handle HTTP errors by status
      if (!response.ok) {
        const error = await handleHttpError(response);
        if (error.retryable) {
          lastError = error;
          // PATTERN: Exponential backoff
          const delay = Math.pow(2, attempt) * 100;
          await sleep(delay);
          continue;
        }
        return {
          success: false,
          output: "",
          errorMessage: error.message
        };
      }

      const data = await response.text();
      return {
        success: true,
        output: data,
        errorMessage: null
      };

    } catch (error) {
      // PATTERN: Handle network errors (retryable)
      if (isNetworkError(error)) {
        lastError = error;
        const delay = Math.pow(2, attempt) * 100;
        await sleep(delay);
        continue;
      }
      throw error;
    }
  }

  return {
    success: false,
    output: "",
    errorMessage: `Failed after ${retries} attempts: ${lastError?.message || 'Unknown error'}`
  };
}

/**
 * PATTERN: Input validation with clear error messages
 */
function validateInput(args) {
  const errors = [];

  // PATTERN: Collect all validation errors
  if (!args.email) {
    errors.push("Email is required");
  } else if (!isValidEmail(args.email)) {
    errors.push(`Invalid email format: ${args.email}`);
  }

  if (args.age !== undefined) {
    if (typeof args.age !== 'number') {
      errors.push("Age must be a number");
    } else if (args.age < 0) {
      errors.push("Age cannot be negative");
    } else if (args.age > 150) {
      errors.push("Age seems unrealistic (>150)");
    }
  }

  // PATTERN: Return all errors at once
  if (errors.length > 0) {
    return {
      success: false,
      output: "",
      errorMessage: `Validation failed:\n- ${errors.join('\n- ')}`
    };
  }

  return {
    success: true,
    output: "Validation passed",
    errorMessage: null
  };
}

/**
 * PATTERN: Categorize HTTP errors
 */
async function handleHttpError(response) {
  const status = response.status;

  // Client errors (4xx) - usually not retryable
  if (status === 400) {
    const body = await response.text().catch(() => '');
    return { message: `Bad request: ${body}`, retryable: false };
  }
  if (status === 401) {
    return { message: "Authentication required", retryable: false };
  }
  if (status === 403) {
    return { message: "Access forbidden", retryable: false };
  }
  if (status === 404) {
    return { message: "Resource not found", retryable: false };
  }
  if (status === 429) {
    // Rate limited - should retry
    return { message: "Rate limit exceeded", retryable: true };
  }

  // Server errors (5xx) - usually retryable
  if (status >= 500) {
    return { message: `Server error: HTTP ${status}`, retryable: true };
  }

  return { message: `HTTP ${status}`, retryable: false };
}

/**
 * PATTERN: Top-level error handler
 */
function handleError(error) {
  // Log to stderr for debugging (doesn't interfere with MCP)
  console.error("Error:", error);

  // PATTERN: Return user-friendly message
  return {
    success: false,
    output: "",
    errorMessage: error.message || "An unexpected error occurred"
  };
}

// Helper functions
function isNetworkError(error) {
  return error.name === 'TypeError' && error.message.includes('fetch');
}

function isValidEmail(email) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function validateConfig(configJson) {
  return { ok: null };
}
