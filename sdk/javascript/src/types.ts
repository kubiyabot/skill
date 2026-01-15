/**
 * Type definitions for Skill Engine SDK
 *
 * These types match the WIT interface defined in skill-interface.wit
 * with additional validation and error handling support.
 */

// ============================================================================
// Core Types
// ============================================================================

/**
 * Metadata about a skill
 */
export interface SkillMetadata {
  /** Unique skill identifier (kebab-case) */
  name: string;
  /** Semantic version (e.g., "1.0.0") */
  version: string;
  /** Brief description of what the skill does */
  description: string;
  /** Skill author name or organization */
  author: string;
  /** Tags for categorization */
  tags?: string[];
  /** Project homepage URL */
  homepage?: string;
  /** Project repository URL */
  repository?: string;
}

/**
 * Parameter type enumeration - extended with new types
 */
export type ParameterType =
  | 'string'
  | 'number'
  | 'boolean'
  | 'file'
  | 'json'      // Parsed JSON object
  | 'array'     // Array of strings
  | 'secret';   // Sensitive value (masked in logs)

/**
 * Common string formats for validation
 */
export type StringFormat =
  | 'email'
  | 'url'
  | 'uri'
  | 'date'
  | 'datetime'
  | 'time'
  | 'uuid'
  | 'hostname'
  | 'ipv4'
  | 'ipv6';

/**
 * Parameter validation constraints
 */
export interface ParameterValidation {
  /** Regex pattern for string validation */
  pattern?: string;
  /** Minimum string length */
  minLength?: number;
  /** Maximum string length */
  maxLength?: number;
  /** Minimum numeric value */
  minimum?: number;
  /** Maximum numeric value */
  maximum?: number;
  /** Allowed enum values */
  enum?: string[];
  /** String format (email, url, date, etc.) */
  format?: StringFormat;
}

/**
 * Tool parameter definition with validation support
 */
export interface Parameter {
  /** Parameter name (kebab-case or camelCase) */
  name: string;
  /** Parameter data type */
  paramType: ParameterType;
  /** Description of what this parameter does */
  description: string;
  /** Whether this parameter is required */
  required: boolean;
  /** Default value if parameter is not provided */
  defaultValue?: string;
  /** Validation constraints */
  validation?: ParameterValidation;
}

/**
 * Tool definition
 */
export interface ToolDefinition {
  /** Tool name (kebab-case) */
  name: string;
  /** Description of what this tool does */
  description: string;
  /** List of parameters this tool accepts */
  parameters: Parameter[];
}

// ============================================================================
// Error Handling
// ============================================================================

/**
 * Error codes for structured error handling
 */
export type ErrorCode =
  | 'VALIDATION_ERROR'     // Input validation failed
  | 'AUTH_ERROR'           // Authentication failed
  | 'PERMISSION_ERROR'     // Authorization/permission denied
  | 'RATE_LIMIT'           // Rate limit exceeded
  | 'NOT_FOUND'            // Resource not found
  | 'CONFLICT'             // Resource conflict
  | 'TIMEOUT'              // Operation timed out
  | 'NETWORK_ERROR'        // Network connectivity issue
  | 'SERVICE_ERROR'        // External service error
  | 'INTERNAL_ERROR';      // Internal error

/**
 * Structured error with context
 */
export interface SkillError {
  /** Error code for programmatic handling */
  code: ErrorCode;
  /** Human-readable error message */
  message: string;
  /** Additional error details */
  details?: Record<string, unknown>;
  /** Original error (if wrapping) */
  cause?: Error;
  /** Whether the operation can be retried */
  retryable: boolean;
  /** Seconds to wait before retrying (if retryable) */
  retryAfter?: number;
}

/**
 * Create a structured error
 */
export function createError(
  code: ErrorCode,
  message: string,
  options?: {
    details?: Record<string, unknown>;
    cause?: Error;
    retryable?: boolean;
    retryAfter?: number;
  }
): SkillError {
  return {
    code,
    message,
    details: options?.details,
    cause: options?.cause,
    retryable: options?.retryable ?? false,
    retryAfter: options?.retryAfter,
  };
}

/**
 * Common error factories
 */
export const errors = {
  validation: (message: string, details?: Record<string, unknown>) =>
    createError('VALIDATION_ERROR', message, { details }),

  auth: (message: string = 'Authentication required') =>
    createError('AUTH_ERROR', message),

  permission: (message: string = 'Permission denied') =>
    createError('PERMISSION_ERROR', message),

  rateLimit: (retryAfter?: number) =>
    createError('RATE_LIMIT', 'Rate limit exceeded', { retryable: true, retryAfter }),

  notFound: (resource: string) =>
    createError('NOT_FOUND', `${resource} not found`),

  timeout: (operation: string) =>
    createError('TIMEOUT', `${operation} timed out`, { retryable: true }),

  network: (message: string, cause?: Error) =>
    createError('NETWORK_ERROR', message, { cause, retryable: true }),

  service: (service: string, message: string) =>
    createError('SERVICE_ERROR', `${service}: ${message}`, { retryable: true }),

  internal: (message: string, cause?: Error) =>
    createError('INTERNAL_ERROR', message, { cause }),
};

// ============================================================================
// Execution Results
// ============================================================================

/**
 * Result of tool execution
 */
export interface ExecutionResult {
  /** Whether the tool executed successfully */
  success: boolean;
  /** Output text/data from the tool execution */
  output: string;
  /** Error message if execution failed */
  errorMessage: string | null;
  /** Structured error (if available) */
  error?: SkillError;
  /** Structured data output (for programmatic access) */
  data?: unknown;
}

/**
 * Create a successful execution result
 */
export function ok(output: string, data?: unknown): ExecutionResult {
  return {
    success: true,
    output,
    errorMessage: null,
    data,
  };
}

/**
 * Create a failed execution result
 */
export function err(message: string, error?: SkillError): ExecutionResult {
  return {
    success: false,
    output: '',
    errorMessage: message,
    error,
  };
}

/**
 * Create an execution result from a SkillError
 */
export function fromError(error: SkillError): ExecutionResult {
  return {
    success: false,
    output: '',
    errorMessage: error.message,
    error,
  };
}

// ============================================================================
// Tool Handlers
// ============================================================================

/**
 * Tool handler function signature
 *
 * @param args - Parsed arguments as key-value object
 * @returns Execution result with success status and output
 */
export type ToolHandler<TArgs = Record<string, any>> = (
  args: TArgs
) => Promise<ExecutionResult> | ExecutionResult;

/**
 * Tool definition with handler
 */
export interface Tool<TArgs = Record<string, any>> {
  /** Tool name */
  name: string;
  /** Tool description */
  description: string;
  /** Tool parameters */
  parameters: Parameter[];
  /** Handler function to execute the tool */
  handler: ToolHandler<TArgs>;
}

// ============================================================================
// Configuration
// ============================================================================

/**
 * Configuration validation result
 */
export interface ValidationResult {
  /** Validation succeeded */
  ok: null;
}

/**
 * Configuration validation error
 */
export interface ValidationError {
  /** Error message */
  err: string;
}

/**
 * Configuration validator function
 */
export type ConfigValidator = (
  config: Record<string, string>
) => Promise<ValidationResult | ValidationError> | ValidationResult | ValidationError;

/**
 * Skill configuration
 */
export interface SkillConfig {
  /** Skill metadata */
  metadata: SkillMetadata;
  /** Tools provided by this skill */
  tools: Tool[];
  /** Optional configuration validator */
  validateConfig?: ConfigValidator;
}

/**
 * Environment configuration type helper
 *
 * Used with getConfig<T>() to provide type-safe access to environment variables
 */
export type EnvironmentConfig = Record<string, string>;

// ============================================================================
// HTTP Client Types
// ============================================================================

/**
 * HTTP request method
 */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS';

/**
 * HTTP request options
 */
export interface RequestOptions {
  /** HTTP method (default: GET) */
  method?: HttpMethod;
  /** Request headers */
  headers?: Record<string, string>;
  /** Request body (will be JSON stringified if object) */
  body?: unknown;
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Number of retry attempts */
  retries?: number;
  /** Delay between retries in milliseconds */
  retryDelay?: number;
}

/**
 * HTTP response
 */
export interface HttpResponse<T = unknown> {
  /** Whether the response was successful (2xx status) */
  ok: boolean;
  /** HTTP status code */
  status: number;
  /** HTTP status text */
  statusText: string;
  /** Response headers */
  headers: Record<string, string>;
  /** Parsed response body */
  data: T;
}

/**
 * Authentication type for HTTP client
 */
export type AuthType = 'bearer' | 'basic' | 'api-key';

/**
 * HTTP client options
 */
export interface HttpClientOptions {
  /** Base URL for all requests */
  baseUrl?: string;
  /** Default headers for all requests */
  headers?: Record<string, string>;
  /** Default timeout in milliseconds */
  timeout?: number;
  /** Default retry count */
  retries?: number;
}

/**
 * Options for creating an authenticated HTTP client
 */
export interface AuthenticatedClientOptions extends HttpClientOptions {
  /** Authentication type */
  authType: AuthType;
  /** Config key containing the token/key */
  tokenKey?: string;
  /** Header name for API key auth */
  headerName?: string;
}

// ============================================================================
// JSON Schema Types (for MCP integration)
// ============================================================================

/**
 * JSON Schema for tool parameters
 */
export interface JsonSchema {
  type: string;
  title?: string;
  description?: string;
  properties?: Record<string, JsonSchema>;
  required?: string[];
  additionalProperties?: boolean;
  items?: JsonSchema;
  enum?: string[];
  default?: unknown;
  format?: string;
  pattern?: string;
  minLength?: number;
  maxLength?: number;
  minimum?: number;
  maximum?: number;
}
