/**
 * HTTP Client for Skill Engine SDK
 *
 * Provides a fetch-based HTTP client with:
 * - Automatic retry with exponential backoff
 * - Auth-aware request helpers
 * - JSON request/response handling
 * - Error handling integration
 *
 * @example
 * ```typescript
 * // Basic usage
 * const client = new SkillHttpClient({ baseUrl: 'https://api.example.com' });
 * const response = await client.get('/users');
 *
 * // With authentication
 * const authClient = createAuthenticatedClient({
 *   baseUrl: 'https://api.github.com',
 *   authType: 'bearer',
 *   tokenKey: 'GITHUB_TOKEN'
 * });
 * const repos = await authClient.get('/user/repos');
 * ```
 */

import { getConfig } from './index.js';
import type {
  RequestOptions,
  HttpResponse,
  HttpClientOptions,
  AuthenticatedClientOptions,
  AuthType,
} from './types.js';
import { errors } from './types.js';

/**
 * Default request timeout in milliseconds
 */
const DEFAULT_TIMEOUT = 30000;

/**
 * Default retry count
 */
const DEFAULT_RETRIES = 3;

/**
 * Default retry delay in milliseconds
 */
const DEFAULT_RETRY_DELAY = 1000;

/**
 * HTTP client for making requests from skills.
 *
 * Uses the fetch API (available in WASI via StarlingMonkey/jco).
 *
 * @example
 * ```typescript
 * const client = new SkillHttpClient({
 *   baseUrl: 'https://api.example.com',
 *   headers: { 'X-Custom-Header': 'value' }
 * });
 *
 * // GET request
 * const users = await client.get<User[]>('/users');
 *
 * // POST request
 * const newUser = await client.post<User>('/users', { name: 'John' });
 *
 * // With error handling
 * const response = await client.get('/data');
 * if (!response.ok) {
 *   console.error(`Request failed: ${response.status}`);
 * }
 * ```
 */
export class SkillHttpClient {
  private baseUrl: string;
  private defaultHeaders: Record<string, string>;
  private defaultTimeout: number;
  private defaultRetries: number;

  constructor(options: HttpClientOptions = {}) {
    this.baseUrl = options.baseUrl?.replace(/\/+$/, '') ?? '';
    this.defaultHeaders = options.headers ?? {};
    this.defaultTimeout = options.timeout ?? DEFAULT_TIMEOUT;
    this.defaultRetries = options.retries ?? DEFAULT_RETRIES;
  }

  /**
   * Make an HTTP request with automatic retry and error handling.
   *
   * @param url - URL path (appended to baseUrl)
   * @param options - Request options
   * @returns HTTP response with parsed body
   */
  async request<T = unknown>(
    url: string,
    options: RequestOptions = {}
  ): Promise<HttpResponse<T>> {
    const fullUrl = this.baseUrl ? `${this.baseUrl}${url}` : url;
    const method = options.method ?? 'GET';
    const retries = options.retries ?? this.defaultRetries;
    const retryDelay = options.retryDelay ?? DEFAULT_RETRY_DELAY;

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
      ...this.defaultHeaders,
      ...options.headers,
    };

    let lastError: Error | undefined;

    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        const controller = new AbortController();
        const timeout = options.timeout ?? this.defaultTimeout;
        const timeoutId = setTimeout(() => controller.abort(), timeout);

        try {
          const response = await fetch(fullUrl, {
            method,
            headers,
            body: options.body ? JSON.stringify(options.body) : undefined,
            signal: controller.signal,
          });

          clearTimeout(timeoutId);

          // Parse response
          let data: T;
          const contentType = response.headers.get('content-type') ?? '';

          if (contentType.includes('application/json')) {
            data = await response.json() as T;
          } else {
            data = await response.text() as unknown as T;
          }

          // Convert headers to plain object
          const responseHeaders: Record<string, string> = {};
          response.headers.forEach((value, key) => {
            responseHeaders[key] = value;
          });

          return {
            ok: response.ok,
            status: response.status,
            statusText: response.statusText,
            headers: responseHeaders,
            data,
          };
        } finally {
          clearTimeout(timeoutId);
        }
      } catch (error) {
        lastError = error as Error;

        // Don't retry on abort (timeout)
        if (lastError.name === 'AbortError') {
          throw errors.timeout(`Request to ${fullUrl}`);
        }

        // Check if we should retry
        if (attempt < retries) {
          const delay = retryDelay * Math.pow(2, attempt);
          await this.delay(delay);
          continue;
        }
      }
    }

    // All retries failed
    throw errors.network(
      `Request to ${fullUrl} failed after ${retries + 1} attempts`,
      lastError
    );
  }

  /**
   * Make a GET request.
   */
  async get<T = unknown>(
    url: string,
    options?: Omit<RequestOptions, 'method' | 'body'>
  ): Promise<HttpResponse<T>> {
    return this.request<T>(url, { ...options, method: 'GET' });
  }

  /**
   * Make a POST request.
   */
  async post<T = unknown>(
    url: string,
    body?: unknown,
    options?: Omit<RequestOptions, 'method'>
  ): Promise<HttpResponse<T>> {
    return this.request<T>(url, { ...options, method: 'POST', body });
  }

  /**
   * Make a PUT request.
   */
  async put<T = unknown>(
    url: string,
    body?: unknown,
    options?: Omit<RequestOptions, 'method'>
  ): Promise<HttpResponse<T>> {
    return this.request<T>(url, { ...options, method: 'PUT', body });
  }

  /**
   * Make a PATCH request.
   */
  async patch<T = unknown>(
    url: string,
    body?: unknown,
    options?: Omit<RequestOptions, 'method'>
  ): Promise<HttpResponse<T>> {
    return this.request<T>(url, { ...options, method: 'PATCH', body });
  }

  /**
   * Make a DELETE request.
   */
  async delete<T = unknown>(
    url: string,
    options?: Omit<RequestOptions, 'method' | 'body'>
  ): Promise<HttpResponse<T>> {
    return this.request<T>(url, { ...options, method: 'DELETE' });
  }

  /**
   * Delay for a specified number of milliseconds.
   */
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

/**
 * Create an HTTP client with authentication configured from skill config.
 *
 * The token/key is read from the skill's configuration (environment variables)
 * which are set by the CLI after `skill auth login`.
 *
 * @example
 * ```typescript
 * // Bearer token auth (OAuth2, JWT)
 * const github = createAuthenticatedClient({
 *   baseUrl: 'https://api.github.com',
 *   authType: 'bearer',
 *   tokenKey: 'GITHUB_TOKEN'  // Set via `skill auth login github`
 * });
 *
 * // API key auth
 * const openai = createAuthenticatedClient({
 *   baseUrl: 'https://api.openai.com/v1',
 *   authType: 'api-key',
 *   tokenKey: 'OPENAI_API_KEY',
 *   headerName: 'Authorization',
 *   headers: { 'OpenAI-Beta': 'assistants=v2' }
 * });
 *
 * // Basic auth
 * const api = createAuthenticatedClient({
 *   baseUrl: 'https://api.example.com',
 *   authType: 'basic',
 *   tokenKey: 'API_CREDENTIALS'  // Format: "username:password"
 * });
 * ```
 */
export function createAuthenticatedClient(
  options: AuthenticatedClientOptions
): SkillHttpClient {
  const config = getConfig();
  const headers: Record<string, string> = { ...options.headers };

  // Get token from config
  const tokenKey = options.tokenKey ?? getDefaultTokenKey(options.authType);
  const token = config[tokenKey];

  if (!token) {
    console.warn(
      `Warning: Auth token not found in config key '${tokenKey}'. ` +
      `Run 'skill auth login' to configure authentication.`
    );
  }

  // Set auth header based on type
  switch (options.authType) {
    case 'bearer':
      if (token) {
        headers['Authorization'] = `Bearer ${token}`;
      }
      break;

    case 'basic':
      if (token) {
        // Token should be in format "username:password"
        const encoded = typeof btoa !== 'undefined'
          ? btoa(token)
          : Buffer.from(token).toString('base64');
        headers['Authorization'] = `Basic ${encoded}`;
      }
      break;

    case 'api-key':
      if (token) {
        const headerName = options.headerName ?? 'X-API-Key';
        headers[headerName] = token;
      }
      break;
  }

  return new SkillHttpClient({
    baseUrl: options.baseUrl,
    headers,
    timeout: options.timeout,
    retries: options.retries,
  });
}

/**
 * Get the default token key for an auth type.
 */
function getDefaultTokenKey(authType: AuthType): string {
  switch (authType) {
    case 'bearer':
      return 'ACCESS_TOKEN';
    case 'api-key':
      return 'API_KEY';
    case 'basic':
      return 'CREDENTIALS';
    default:
      return 'TOKEN';
  }
}

/**
 * Simple fetch wrapper with JSON handling.
 *
 * For quick one-off requests without creating a client.
 *
 * @example
 * ```typescript
 * const data = await fetchJson<User>('https://api.example.com/user/1');
 * ```
 */
export async function fetchJson<T = unknown>(
  url: string,
  options: RequestOptions = {}
): Promise<T> {
  const client = new SkillHttpClient();
  const response = await client.request<T>(url, options);

  if (!response.ok) {
    throw errors.service(
      'HTTP',
      `Request failed with status ${response.status}: ${response.statusText}`
    );
  }

  return response.data;
}

/**
 * Check if a response indicates a rate limit error.
 */
export function isRateLimited(response: HttpResponse): boolean {
  return response.status === 429;
}

/**
 * Get retry-after value from response headers (in seconds).
 */
export function getRetryAfter(response: HttpResponse): number | undefined {
  const retryAfter = response.headers['retry-after'];
  if (!retryAfter) return undefined;

  // Could be a number of seconds or a date
  const seconds = parseInt(retryAfter, 10);
  if (!isNaN(seconds)) return seconds;

  // Try parsing as date
  const date = new Date(retryAfter);
  if (!isNaN(date.getTime())) {
    return Math.max(0, Math.ceil((date.getTime() - Date.now()) / 1000));
  }

  return undefined;
}
