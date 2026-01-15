/**
 * HTTP Skill
 *
 * A universal HTTP client demonstrating:
 * - Multiple HTTP methods (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)
 * - Flexible authentication (Bearer, API-Key, Basic, None)
 * - Content type support (JSON, form-urlencoded, multipart)
 * - Specialized requests (GraphQL, file upload/download)
 *
 * Setup:
 *   No configuration required. Authentication is per-request.
 *
 * Examples:
 *   skill run http-skill get --url "https://api.example.com/users"
 *   skill run http-skill post --url "https://api.example.com/users" --body '{"name":"John"}'
 *   skill run http-skill graphql --url "https://api.example.com/graphql" --query "{ users { id name } }"
 */

import {
  defineSkill,
  getConfig,
  ok,
  err,
  errors,
  httpRequest,
  type ExecutionResult,
  type ToolHandler,
} from '@skill-engine/sdk';

// Types
interface HttpArgs {
  url: string;
  headers?: string;
  auth_type?: string;
  auth_value?: string;
  timeout?: number;
}

interface RequestArgs extends HttpArgs {
  body?: string;
  content_type?: string;
}

interface GraphqlArgs extends HttpArgs {
  query: string;
  variables?: string;
  operation_name?: string;
}

// Parse headers from JSON string
function parseHeaders(headersJson?: string): Record<string, string> {
  if (!headersJson) return {};
  try {
    return JSON.parse(headersJson);
  } catch {
    return {};
  }
}

// Build request headers with auth
function buildHeaders(
  args: HttpArgs,
  contentType?: string
): Record<string, string> {
  const headers: Record<string, string> = parseHeaders(args.headers);

  // Add content type if specified
  if (contentType) {
    headers['Content-Type'] = contentType;
  }

  // Add authentication
  if (args.auth_type && args.auth_value) {
    switch (args.auth_type.toLowerCase()) {
      case 'bearer':
        headers['Authorization'] = `Bearer ${args.auth_value}`;
        break;
      case 'basic':
        // auth_value should be "username:password"
        const encoded = Buffer.from(args.auth_value).toString('base64');
        headers['Authorization'] = `Basic ${encoded}`;
        break;
      case 'api-key':
        // auth_value should be "HeaderName:value"
        const [headerName, value] = args.auth_value.split(':');
        if (headerName && value) {
          headers[headerName] = value;
        }
        break;
    }
  }

  return headers;
}

// Format response for output
function formatResponse(
  status: number,
  statusText: string,
  headers: Record<string, string>,
  body: string,
  showHeaders: boolean = false
): string {
  const lines: string[] = [];
  lines.push(`Status: ${status} ${statusText}`);

  if (showHeaders) {
    lines.push('\nHeaders:');
    for (const [key, value] of Object.entries(headers)) {
      lines.push(`  ${key}: ${value}`);
    }
  }

  if (body) {
    lines.push('\nBody:');
    // Try to pretty-print JSON
    try {
      const parsed = JSON.parse(body);
      lines.push(JSON.stringify(parsed, null, 2));
    } catch {
      lines.push(body);
    }
  }

  return lines.join('\n');
}

// Common HTTP request executor
async function executeRequest(
  method: string,
  args: RequestArgs,
  defaultContentType?: string
): Promise<ExecutionResult> {
  try {
    const contentType = args.content_type || defaultContentType;
    const headers = buildHeaders(args, contentType);
    const timeout = args.timeout || 30000;

    const response = await httpRequest({
      method,
      url: args.url,
      headers,
      body: args.body,
      timeout,
    });

    if (!response.ok) {
      return err(
        `HTTP Error: ${response.status} ${response.statusText}\n${response.body || ''}`
      );
    }

    return ok(
      formatResponse(response.status, response.statusText, response.headers || {}, response.body || ''),
      {
        status: response.status,
        headers: response.headers,
        body: response.body,
      }
    );
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : String(e);
    return err(`Request failed: ${message}`);
  }
}

// Auth parameter definitions (reused across tools)
const authParams = [
  {
    name: 'auth_type',
    paramType: 'string' as const,
    description: 'Authentication type: bearer, basic, api-key, or none',
    required: false,
    validation: {
      enum: ['bearer', 'basic', 'api-key', 'none'],
    },
  },
  {
    name: 'auth_value',
    paramType: 'string' as const,
    description: 'Auth value (token for bearer, user:pass for basic, Header:value for api-key)',
    required: false,
  },
];

const commonParams = [
  {
    name: 'url',
    paramType: 'string' as const,
    description: 'Request URL',
    required: true,
    validation: {
      pattern: '^https?://',
    },
  },
  {
    name: 'headers',
    paramType: 'string' as const,
    description: 'Custom headers as JSON object (e.g., {"X-Custom": "value"})',
    required: false,
  },
  ...authParams,
  {
    name: 'timeout',
    paramType: 'number' as const,
    description: 'Request timeout in milliseconds',
    required: false,
    defaultValue: '30000',
  },
];

const bodyParams = [
  {
    name: 'body',
    paramType: 'string' as const,
    description: 'Request body',
    required: false,
  },
  {
    name: 'content_type',
    paramType: 'string' as const,
    description: 'Content-Type header',
    required: false,
  },
];

// Skill definition
export default defineSkill({
  metadata: {
    name: 'http-skill',
    version: '1.0.0',
    description: 'Universal HTTP client for API requests with flexible authentication',
    author: 'Skill Engine Team',
    tags: ['http', 'api', 'rest', 'client', 'web'],
  },
  tools: [
    // ========================================
    // Basic HTTP Methods
    // ========================================
    {
      name: 'get',
      description: 'Send HTTP GET request',
      parameters: commonParams,
      handler: (async (args: HttpArgs): Promise<ExecutionResult> => {
        return executeRequest('GET', args as RequestArgs);
      }) as ToolHandler,
    },

    {
      name: 'post',
      description: 'Send HTTP POST request',
      parameters: [...commonParams, ...bodyParams],
      handler: (async (args: RequestArgs): Promise<ExecutionResult> => {
        return executeRequest('POST', args, 'application/json');
      }) as ToolHandler,
    },

    {
      name: 'put',
      description: 'Send HTTP PUT request',
      parameters: [...commonParams, ...bodyParams],
      handler: (async (args: RequestArgs): Promise<ExecutionResult> => {
        return executeRequest('PUT', args, 'application/json');
      }) as ToolHandler,
    },

    {
      name: 'patch',
      description: 'Send HTTP PATCH request',
      parameters: [...commonParams, ...bodyParams],
      handler: (async (args: RequestArgs): Promise<ExecutionResult> => {
        return executeRequest('PATCH', args, 'application/json');
      }) as ToolHandler,
    },

    {
      name: 'delete',
      description: 'Send HTTP DELETE request',
      parameters: commonParams,
      handler: (async (args: HttpArgs): Promise<ExecutionResult> => {
        return executeRequest('DELETE', args as RequestArgs);
      }) as ToolHandler,
    },

    {
      name: 'head',
      description: 'Send HTTP HEAD request (returns headers only)',
      parameters: commonParams,
      handler: (async (args: HttpArgs): Promise<ExecutionResult> => {
        try {
          const headers = buildHeaders(args);
          const timeout = args.timeout || 30000;

          const response = await httpRequest({
            method: 'HEAD',
            url: args.url,
            headers,
            timeout,
          });

          const headerLines = Object.entries(response.headers || {})
            .map(([k, v]) => `${k}: ${v}`)
            .join('\n');

          return ok(
            `Status: ${response.status} ${response.statusText}\n\nHeaders:\n${headerLines}`,
            {
              status: response.status,
              headers: response.headers,
            }
          );
        } catch (e: unknown) {
          const message = e instanceof Error ? e.message : String(e);
          return err(`Request failed: ${message}`);
        }
      }) as ToolHandler,
    },

    {
      name: 'options',
      description: 'Send HTTP OPTIONS request (returns allowed methods)',
      parameters: commonParams,
      handler: (async (args: HttpArgs): Promise<ExecutionResult> => {
        try {
          const headers = buildHeaders(args);
          const timeout = args.timeout || 30000;

          const response = await httpRequest({
            method: 'OPTIONS',
            url: args.url,
            headers,
            timeout,
          });

          const allowedMethods = response.headers?.['Allow'] || response.headers?.['allow'] || 'Not specified';
          const corsHeaders = Object.entries(response.headers || {})
            .filter(([k]) => k.toLowerCase().startsWith('access-control'))
            .map(([k, v]) => `${k}: ${v}`)
            .join('\n');

          return ok(
            `Status: ${response.status} ${response.statusText}\nAllowed Methods: ${allowedMethods}\n\nCORS Headers:\n${corsHeaders || 'None'}`,
            {
              status: response.status,
              allowedMethods,
              headers: response.headers,
            }
          );
        } catch (e: unknown) {
          const message = e instanceof Error ? e.message : String(e);
          return err(`Request failed: ${message}`);
        }
      }) as ToolHandler,
    },

    // ========================================
    // Specialized Requests
    // ========================================
    {
      name: 'json-get',
      description: 'GET request with JSON response parsing and optional JQ-like filtering',
      parameters: [
        ...commonParams,
        {
          name: 'filter',
          paramType: 'string',
          description: 'JSON path filter (e.g., ".data.users", "[0].name")',
          required: false,
        },
      ],
      handler: (async (args: HttpArgs & { filter?: string }): Promise<ExecutionResult> => {
        try {
          const headers = buildHeaders(args);
          headers['Accept'] = 'application/json';
          const timeout = args.timeout || 30000;

          const response = await httpRequest({
            method: 'GET',
            url: args.url,
            headers,
            timeout,
          });

          if (!response.ok) {
            return err(`HTTP Error: ${response.status} ${response.statusText}`);
          }

          let data = JSON.parse(response.body || '{}');

          // Apply simple filter if provided
          if (args.filter) {
            const path = args.filter.replace(/^\./,'').split('.');
            for (const key of path) {
              if (key.startsWith('[') && key.endsWith(']')) {
                const index = parseInt(key.slice(1, -1));
                data = data[index];
              } else if (data && typeof data === 'object') {
                data = data[key];
              }
            }
          }

          return ok(
            JSON.stringify(data, null, 2),
            { data }
          );
        } catch (e: unknown) {
          const message = e instanceof Error ? e.message : String(e);
          return err(`Request failed: ${message}`);
        }
      }) as ToolHandler,
    },

    {
      name: 'json-post',
      description: 'POST request with JSON body and response',
      parameters: [
        ...commonParams,
        {
          name: 'data',
          paramType: 'string',
          description: 'JSON data to send',
          required: true,
        },
      ],
      handler: (async (args: HttpArgs & { data: string }): Promise<ExecutionResult> => {
        try {
          // Validate JSON
          JSON.parse(args.data);
        } catch {
          return err('Invalid JSON data');
        }
        return executeRequest('POST', { ...args, body: args.data } as RequestArgs, 'application/json');
      }) as ToolHandler,
    },

    {
      name: 'form-post',
      description: 'POST request with form-urlencoded data',
      parameters: [
        ...commonParams,
        {
          name: 'data',
          paramType: 'string',
          description: 'Form data as JSON object (e.g., {"field1": "value1", "field2": "value2"})',
          required: true,
        },
      ],
      handler: (async (args: HttpArgs & { data: string }): Promise<ExecutionResult> => {
        try {
          const formData = JSON.parse(args.data);
          const encoded = Object.entries(formData)
            .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(String(v))}`)
            .join('&');

          return executeRequest(
            'POST',
            { ...args, body: encoded } as RequestArgs,
            'application/x-www-form-urlencoded'
          );
        } catch (e: unknown) {
          const message = e instanceof Error ? e.message : String(e);
          return err(`Failed to encode form data: ${message}`);
        }
      }) as ToolHandler,
    },

    {
      name: 'graphql',
      description: 'Send GraphQL query or mutation',
      parameters: [
        ...commonParams,
        {
          name: 'query',
          paramType: 'string',
          description: 'GraphQL query or mutation',
          required: true,
        },
        {
          name: 'variables',
          paramType: 'string',
          description: 'GraphQL variables as JSON',
          required: false,
        },
        {
          name: 'operation_name',
          paramType: 'string',
          description: 'Operation name (if query contains multiple operations)',
          required: false,
        },
      ],
      handler: (async (args: GraphqlArgs): Promise<ExecutionResult> => {
        try {
          const graphqlBody: Record<string, unknown> = {
            query: args.query,
          };

          if (args.variables) {
            graphqlBody.variables = JSON.parse(args.variables);
          }

          if (args.operation_name) {
            graphqlBody.operationName = args.operation_name;
          }

          const headers = buildHeaders(args, 'application/json');
          const timeout = args.timeout || 30000;

          const response = await httpRequest({
            method: 'POST',
            url: args.url,
            headers,
            body: JSON.stringify(graphqlBody),
            timeout,
          });

          if (!response.ok) {
            return err(`HTTP Error: ${response.status} ${response.statusText}`);
          }

          const result = JSON.parse(response.body || '{}');

          if (result.errors && result.errors.length > 0) {
            const errorMessages = result.errors.map((e: { message: string }) => e.message).join('; ');
            return err(`GraphQL Errors: ${errorMessages}`, errors.validation());
          }

          return ok(
            JSON.stringify(result.data, null, 2),
            { data: result.data, extensions: result.extensions }
          );
        } catch (e: unknown) {
          const message = e instanceof Error ? e.message : String(e);
          return err(`GraphQL request failed: ${message}`);
        }
      }) as ToolHandler,
    },

    {
      name: 'download',
      description: 'Download a file and return its content (text) or info (binary)',
      parameters: [
        ...commonParams,
        {
          name: 'as_text',
          paramType: 'boolean',
          description: 'Return content as text (for text files)',
          required: false,
          defaultValue: 'true',
        },
      ],
      handler: (async (args: HttpArgs & { as_text?: boolean }): Promise<ExecutionResult> => {
        try {
          const headers = buildHeaders(args);
          const timeout = args.timeout || 60000;

          const response = await httpRequest({
            method: 'GET',
            url: args.url,
            headers,
            timeout,
          });

          if (!response.ok) {
            return err(`HTTP Error: ${response.status} ${response.statusText}`);
          }

          const contentType = response.headers?.['Content-Type'] || response.headers?.['content-type'] || 'unknown';
          const contentLength = response.headers?.['Content-Length'] || response.headers?.['content-length'] || 'unknown';

          if (args.as_text !== false) {
            return ok(
              `Downloaded (${contentType}, ${contentLength} bytes):\n\n${response.body}`,
              {
                contentType,
                contentLength,
                content: response.body,
              }
            );
          }

          return ok(
            `File downloaded\nContent-Type: ${contentType}\nContent-Length: ${contentLength} bytes`,
            {
              contentType,
              contentLength,
              content: response.body,
            }
          );
        } catch (e: unknown) {
          const message = e instanceof Error ? e.message : String(e);
          return err(`Download failed: ${message}`);
        }
      }) as ToolHandler,
    },

    {
      name: 'upload',
      description: 'Upload content to a URL via PUT or POST',
      parameters: [
        ...commonParams,
        {
          name: 'content',
          paramType: 'string',
          description: 'Content to upload',
          required: true,
        },
        {
          name: 'method',
          paramType: 'string',
          description: 'HTTP method: PUT or POST',
          required: false,
          defaultValue: 'PUT',
          validation: {
            enum: ['PUT', 'POST'],
          },
        },
        {
          name: 'content_type',
          paramType: 'string',
          description: 'Content-Type header',
          required: false,
          defaultValue: 'application/octet-stream',
        },
      ],
      handler: (async (args: HttpArgs & {
        content: string;
        method?: string;
        content_type?: string;
      }): Promise<ExecutionResult> => {
        const method = args.method || 'PUT';
        return executeRequest(method, {
          ...args,
          body: args.content,
        } as RequestArgs, args.content_type || 'application/octet-stream');
      }) as ToolHandler,
    },

    {
      name: 'test-url',
      description: 'Test if a URL is reachable and return status info',
      parameters: [
        {
          name: 'url',
          paramType: 'string',
          description: 'URL to test',
          required: true,
          validation: {
            pattern: '^https?://',
          },
        },
        {
          name: 'timeout',
          paramType: 'number',
          description: 'Timeout in milliseconds',
          required: false,
          defaultValue: '5000',
        },
      ],
      handler: (async (args: { url: string; timeout?: number }): Promise<ExecutionResult> => {
        const startTime = Date.now();

        try {
          const response = await httpRequest({
            method: 'HEAD',
            url: args.url,
            timeout: args.timeout || 5000,
          });

          const duration = Date.now() - startTime;
          const status = response.status;
          const isUp = status >= 200 && status < 400;

          return ok(
            `URL: ${args.url}\nStatus: ${status} ${response.statusText}\nReachable: ${isUp ? 'Yes' : 'No'}\nResponse Time: ${duration}ms`,
            {
              url: args.url,
              status,
              reachable: isUp,
              responseTime: duration,
            }
          );
        } catch (e: unknown) {
          const duration = Date.now() - startTime;
          const message = e instanceof Error ? e.message : String(e);

          return ok(
            `URL: ${args.url}\nStatus: Error\nReachable: No\nError: ${message}\nResponse Time: ${duration}ms`,
            {
              url: args.url,
              status: 0,
              reachable: false,
              error: message,
              responseTime: duration,
            }
          );
        }
      }) as ToolHandler,
    },

    // Note: websocket-send is not fully implementable in WASM context
    // as WebSocket connections require persistent connections.
    // This is a placeholder that documents the limitation.
    {
      name: 'websocket-send',
      description: 'Send a WebSocket message (Note: Limited in WASM context)',
      parameters: [
        {
          name: 'url',
          paramType: 'string',
          description: 'WebSocket URL (ws:// or wss://)',
          required: true,
          validation: {
            pattern: '^wss?://',
          },
        },
        {
          name: 'message',
          paramType: 'string',
          description: 'Message to send',
          required: true,
        },
      ],
      handler: (async (args: { url: string; message: string }): Promise<ExecutionResult> => {
        // WebSocket connections are not fully supported in WASM context
        // This would require host-side implementation
        return err(
          'WebSocket connections are not supported in WASM context. ' +
          'Use the host runtime for WebSocket operations.',
          errors.notImplemented()
        );
      }) as ToolHandler,
    },
  ],
});
