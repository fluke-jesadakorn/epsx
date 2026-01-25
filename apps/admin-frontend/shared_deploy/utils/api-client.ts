/**
 * UNIFIED API CLIENT
 *
 * Consolidates API client logic for both admin-frontend and frontend applications.
 * Replaces duplicate API client implementations with a single, platform-aware solution.
 *
 * Features:
 * - Platform-aware authentication (admin vs frontend)
 * - Generic HTTP methods with type safety
 * - Consistent error handling and response types
 * - Cookie and token-based authentication support
 * - Server-side and client-side request handling
 * - Configurable request options and timeouts
 */

import { COOKIES, setClientCookie } from '../auth/cookies';
import { getBackendUrl } from './url-resolver';

// ============================================================================
// CORE TYPES AND INTERFACES
// ============================================================================

import type { ApiError, ApiResponse, PaginatedResponse } from '../types/api';
export type { ApiError, ApiResponse, PaginatedResponse };

export interface RequestConfig extends RequestInit {
  serverSide?: boolean; // Timeout removed
  platform?: 'admin' | 'frontend';
}

export type Platform = 'admin' | 'frontend';

// ============================================================================
// UNIFIED API CLIENT CLASS
// ============================================================================

export class UnifiedApiClient {
  private baseURL: string;
  private token?: string;
  private platform: Platform;
  private isServerSide: boolean;

  constructor(options: {
    baseURL?: string;
    token?: string;
    platform: Platform;
    serverSide?: boolean;
  }) {
    this.platform = options.platform;
    this.isServerSide = options.serverSide ?? typeof window === 'undefined';
    this.baseURL = options.baseURL || this.getDefaultBaseURL();
    this.token = options.token;
  }

  private getDefaultBaseURL(): string {
    // 1. Client-Side (Browser): Use local proxy path
    if (!this.isServerSide) {
      return '/api/proxy';
    }

    // 2. Server-Side: Use direct backend URL
    return getBackendUrl('server');
  }

  private async getAuthHeaders(): Promise<HeadersInit> {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
    };

    // Use provided token if available
    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`;
      return headers;
    }

    // Server-side: attempt to get token from cookies (Next.js only)
    if (this.isServerSide) {
      try {
        // Dynamic import to avoid bundling server modules in client
        // This is safe because we're inside the isServerSide check
        const { cookies } = await import('next/headers');

        // cookies() is an async function in newer Next.js versions
        const cookieStore = await cookies();

        // Get access token from unified cookies (no context separation)
        const tokenCookie = cookieStore.get(COOKIES.access_token);

        if (tokenCookie?.value) {
          headers['Authorization'] = `Bearer ${tokenCookie.value}`;
        }
      } catch (error) {
        // This is expected during build time or in non-request contexts
        // We log only if we really expected to be in a request context (optional)
        // console.debug(`[UnifiedApiClient] Server-side token retrieval skipped:`, error);
      }
    } else {
      // Client-side:
      // We DO NOT manually attach the Authorization header anymore.
      // The Next.js Middleware intercepts requests to /api/proxy and injects the token.
      // This prevents exposing the token in client-side logs or managing manual headers.
    }

    return headers;
  }

  private async request<T>(
    endpoint: string,
    config: RequestConfig = {}
  ): Promise<ApiResponse<T>> {
    const { serverSide, platform, ...options } = config;
    const url = `${this.baseURL}${endpoint}`;

    try {
      const headers = await this.getAuthHeaders();

      const requestConfig: RequestInit = {
        ...options,
        headers: {
          ...headers,
          ...options.headers,
        },
        credentials: this.isServerSide ? undefined : 'include',
        cache: this.isServerSide ? 'no-store' : 'default',
      };

      // Apply timeout if supported
      // TIMEOUT REMOVED: We now rely on browser/network defaults or backend timeouts.
      // previous logic: if (timeout && !('signal' in requestConfig)) { ... }

      // DIAGNOSTIC and FIX: Ensure server-side logic doesn't use relative paths for fetch
      if (this.isServerSide && url.startsWith('/')) {
        // This should have been caught by getDefaultBaseURL, but if a custom endpoint was passed as full URL...
        // Actually, endpoint is appended to baseURL.
        // If baseURL is relative (e.g. /api/proxy) and we are server side, that's bad.
        if (this.baseURL.startsWith('/')) {
          console.warn(`[UnifiedApiClient] Server-side client has relative baseURL '${this.baseURL}'. This will fail. Resolving to internal backend URL.`);
          this.baseURL = getBackendUrl('server');
          // Reconstruct URL
          // Note: recursive call might be safer but let's fix in place
        }
      }

      const finalUrl = `${this.baseURL}${endpoint}`;

      // DEBUG LOG
      // console.log(`[UnifiedApiClient] Fetching: ${requestConfig.method || 'GET'} ${finalUrl} (Server: ${this.isServerSide})`);

      const response = await fetch(finalUrl, requestConfig);

      // Handle authentication errors
      if (response.status === 401) {
        // Attempt to refresh token IF not already retrying (avoid infinite loop)
        // And IF this is not the refresh endpoint itself
        // And specific check for client-side execution to leverage proxy cookie injection
        if (!this.isServerSide && !endpoint.includes('/api/auth/session/refresh') && !(headers as any)['x-retry']) {
          try {
            console.debug('[UnifiedApiClient] 401 detected, attempting token refresh...');

            // Call refresh endpoint via proxy (which injects the refresh_token cookie)
            const refreshResponse = await fetch(`${this.baseURL}/api/auth/session/refresh`, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({}) // Empty body allowed because proxy injects token
            });

            if (refreshResponse.ok) {
              const refreshData = await refreshResponse.json();

              if (refreshData.access_token) {
                console.debug('[UnifiedApiClient] Token refreshed successfully, retrying request');

                // Update internal token
                this.setAuthToken(refreshData.access_token);

                // CRITICAL: Update cookies so Proxy uses new token
                if (!this.isServerSide) {
                  setClientCookie(COOKIES.access_token, refreshData.access_token, refreshData.expires_in || 3600);
                  if (refreshData.refresh_token) {
                    setClientCookie(COOKIES.refresh_token, refreshData.refresh_token, 2592000);
                  }
                }

                // Retry original request with new token
                // We recursively call request() but need to ensure headers are updated
                return this.request<T>(endpoint, {
                  ...config,
                  // token property removed as it's not in RequestConfig
                  headers: {
                    ...options.headers,
                    'x-retry': 'true' // Flag to prevent infinite recursion
                  }
                });
              }
            } else {
              console.warn('[UnifiedApiClient] Token refresh attempt failed');
            }
          } catch (refreshError) {
            console.error('[UnifiedApiClient] Error during token refresh:', refreshError);
          }
        }

        // Return error if refresh failed or not attempted
        return {
          success: false,
          data: null,
          error: {
            code: 'UNAUTHORIZED',
            message: 'Unauthorized - please log in again',
            requestId: (headers as any)['x-request-id'],
          },
          meta: {
            timestamp: new Date().toISOString(),
          }
        };
      }

      // Parse response data
      let data;
      try {
        data = response.ok ? await response.json() : null;
      } catch {
        data = null;
      }

      if (!response.ok) {
        const errorMessage = data?.message || data?.error || `HTTP ${response.status}: ${response.statusText}`;
        // Don't log 404 errors - they are often expected (e.g., user has no permissions)
        if (response.status !== 404) {
          console.error(`[UnifiedApiClient] Error ${response.status} ${requestConfig.method || 'GET'} ${url}:`, errorMessage);
        }
        throw new APIError(
          errorMessage,
          data?.code || 'HTTP_ERROR',
          response.status,
          data?.details
        );
      }

      return this.normalizeResponse(response, data);

    } catch (error) {
      // Handle AbortError (timeout) - REMOVED custom timeout error logic
      // if (error instanceof Error && error.name === 'AbortError') { ... }

      // Re-throw if it's already a handled response (shouldn't happen with this design, but for safety)
      if (this.isApiSuccess(error as any) || (error as any).success === false) {
        return error as any;
      }

      // Handle network and other unexpected errors
      const errorMessage = error instanceof Error ? error.message : 'Unknown network error';
      return {
        success: false,
        data: null,
        error: {
          code: 'NETWORK_ERROR',
          message: errorMessage,
          details: { originalError: error }
        }
      };
    }
  }

  /**
   * Normalizes the response to ensure it adheres to the ApiResponse<T> contract.
   * Handles legacy responses, double-wrapping, and error conversion.
   */
  private normalizeResponse<T>(response: Response, data: any): ApiResponse<T> {
    const status = response.status;
    const isSuccess = response.ok;

    // 1. If it's a known API Error (status >= 400)
    if (!isSuccess) {
      // Try to extract structured error from data
      let apiError: ApiError = {
        code: 'HTTP_ERROR',
        message: `HTTP ${status}: ${response.statusText}`,
      };

      if (data && typeof data === 'object') {
        if (data.error && typeof data.error === 'object') {
          // Already has strict error structure
          apiError = data.error as ApiError;
        } else if (data.message) {
          // Simple message format
          apiError.message = data.message;
          apiError.code = data.code || `HTTP_${status}`;
          apiError.details = data.details || data;
        } else if (data.error) {
          // String error
          apiError.message = typeof data.error === 'string' ? data.error : JSON.stringify(data.error);
          apiError.code = data.code || `HTTP_${status}`;
        }
      }

      return {
        success: false,
        data: null,
        error: apiError,
        meta: {
          timestamp: new Date().toISOString(),
          trace_id: response.headers.get('x-request-id') || undefined
        }
      };
    }

    // 2. Success Case - Check for Double Wrapping or Legacy Formats
    // If the data itself IS an ApiResponse (has success, data, error keys), return it directly.
    if (this.isApiResponse<T>(data)) {
      return data;
    }

    // 3. Fallback: Wrap raw data in standard success response
    return {
      success: true,
      data: data as T,
      error: null,
      meta: {
        timestamp: new Date().toISOString(),
        trace_id: response.headers.get('x-request-id') || undefined
      }
    };
  }

  private isApiResponse<T>(data: any): data is ApiResponse<T> {
    return data && typeof data === 'object' && 'success' in data && 'data' in data;
  }


  // ============================================================================
  // GENERIC HTTP METHODS
  // ============================================================================

  async get<T = any>(endpoint: string, params?: Record<string, any>, config?: RequestConfig): Promise<ApiResponse<T>> {
    const queryString = params ? '?' + new URLSearchParams(
      Object.entries(params).reduce((acc, [key, value]) => {
        if (value !== undefined && value !== null) {
          acc[key] = String(value);
        }
        return acc;
      }, {} as Record<string, string>)
    ).toString() : '';

    return this.request<T>(`${endpoint}${queryString}`, {
      method: 'GET',
      ...config
    });
  }

  async post<T = any>(endpoint: string, data?: any, config?: RequestConfig): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
      ...config
    });
  }

  async put<T = any>(endpoint: string, data?: any, config?: RequestConfig): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
      ...config
    });
  }

  async delete<T = any>(endpoint: string, config?: RequestConfig): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'DELETE',
      ...config
    });
  }

  async patch<T = any>(endpoint: string, data?: any, config?: RequestConfig): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'PATCH',
      body: data ? JSON.stringify(data) : undefined,
      ...config
    });
  }

  // ============================================================================
  // AUTHENTICATION MANAGEMENT
  // ============================================================================

  setAuthToken(token: string): void {
    this.token = token;
  }

  removeAuthToken(): void {
    this.token = undefined;
  }

  getAuthToken(): string | undefined {
    return this.token;
  }

  // ============================================================================
  // UTILITY METHODS
  // ============================================================================

  isApiError(error: any): error is ApiError {
    return error &&
      typeof error.code === 'string' &&
      typeof error.message === 'string';
  }

  isApiSuccess<T>(response: ApiResponse<T>): response is ApiResponse<T> & { success: true; data: T } {
    return response.success && response.data !== null;
  }

  // Create a new client with different configuration
  clone(overrides: Partial<{
    baseURL: string;
    token: string;
    platform: Platform;
    serverSide: boolean;
  }>): UnifiedApiClient {
    return new UnifiedApiClient({
      baseURL: overrides.baseURL || this.baseURL,
      token: overrides.token || this.token,
      platform: overrides.platform || this.platform,
      serverSide: overrides.serverSide || this.isServerSide
    });
  }

  /**
   * Performs a raw request and returns the full Response object.
   * Useful for proxying, streaming, or handling non-JSON responses.
   */
  async requestRaw(
    endpoint: string,
    config: RequestConfig = {}
  ): Promise<Response> {
    const { serverSide, platform, ...options } = config;
    const url = `${this.baseURL}${endpoint}`;

    const baseHeaders = await this.getAuthHeaders();
    const mergedHeaders = new Headers(baseHeaders);

    if (options.headers) {
      new Headers(options.headers).forEach((value, key) => {
        mergedHeaders.set(key, value);
      });
    }

    const requestConfig: RequestInit = {
      ...options,
      headers: mergedHeaders,
      credentials: this.isServerSide ? undefined : 'include',
      cache: this.isServerSide ? 'no-store' : 'default',
    };

    return fetch(url, requestConfig);
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/**
 * Create an API client for admin-frontend application
 */
export function createAdminApiClient(options: {
  baseURL?: string;
  token?: string;
  serverSide?: boolean;
} = {}): UnifiedApiClient {
  return new UnifiedApiClient({
    platform: 'admin',
    ...options
  });
}

/**
 * Create an API client for frontend application
 */
export function createFrontendApiClient(options: {
  baseURL?: string;
  token?: string;
  serverSide?: boolean;
} = {}): UnifiedApiClient {
  return new UnifiedApiClient({
    platform: 'frontend',
    ...options
  });
}

/**
 * Legacy createApiClient function for backward compatibility
 */
export function createApiClient(baseURL?: string, token?: string): UnifiedApiClient {
  return new UnifiedApiClient({
    platform: 'frontend', // Default to frontend for legacy compatibility
    baseURL,
    token,
    serverSide: false
  });
}

// ============================================================================
// ERROR CLASSES
// ============================================================================

// ============================================================================
// ERROR CLASSES
// ============================================================================

export class APIError extends Error implements ApiError {
  public code: string;
  public details?: any;
  public status?: number; // Optional status for compatibility
  public requestId?: string;

  constructor(message: string, code: string = 'UNKNOWN_ERROR', status?: number, details?: any) {
    super(message);
    this.name = 'APIError';
    this.code = code;
    this.status = status;
    this.details = details;
  }

  static fromResponse(response: { status: number; statusText: string; data?: any }): APIError {
    const message = response.data?.message || response.data?.error?.message || `HTTP ${response.status}: ${response.statusText}`;
    const code = response.data?.code || response.data?.error?.code || 'HTTP_ERROR';
    const details = response.data?.details || response.data?.error?.details;
    return new APIError(message, code, response.status, details);
  }
}

// ============================================================================
// UTILITY FUNCTIONS FOR COMMON API PATTERNS
// ============================================================================

/**
 * Handle paginated API responses with proper error handling
 */
export async function handlePaginatedRequest<T>(
  client: UnifiedApiClient,
  endpoint: string,
  params: Record<string, any> = {}
): Promise<PaginatedResponse<T>> {
  const response = await client.get<PaginatedResponse<T>>(endpoint, params);

  if (!response.success || !response.data) {
    throw new APIError(
      response.error?.message || 'Failed to fetch data',
      response.error?.code || 'FETCH_ERROR',
      0,
      response.error?.details
    );
  }

  return response.data;
}

/**
 * Handle simple API requests with error handling
 */
export async function handleSimpleRequest<T>(
  client: UnifiedApiClient,
  method: 'get' | 'post' | 'put' | 'delete' | 'patch',
  endpoint: string,
  data?: any
): Promise<T> {
  // DIAGNOSTIC LOG: Trace usage of this supposedly unused function
  console.log(`[UnifiedApiClient] handleSimpleRequest called: ${method.toUpperCase()} ${endpoint}`);

  const response = await client[method]<T>(endpoint, data);

  if (!response.success || !response.data) {
    console.error(`[UnifiedApiClient] handleSimpleRequest failed: ${response.error?.message || 'Unknown Error'}`);
    throw new APIError(
      response.error?.message || 'Request failed',
      response.error?.code || 'REQUEST_FAILED',
      0,
      response.error?.details
    );
  }

  return response.data;
}

/**
 * Retry API requests with exponential backoff
 */
export async function retryRequest<T>(
  requestFn: () => Promise<ApiResponse<T>>,
  maxRetries: number = 3,
  baseDelay: number = 1000
): Promise<ApiResponse<T>> {
  let lastResult: ApiResponse<T> | undefined;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      const result = await requestFn();

      if (result.success) {
        return result;
      }

      lastResult = result;

      // Retry on network errors or 5xx (implied by code conventions)
      const errorCode = result.error?.code || '';
      const shouldRetry =
        errorCode === 'NETWORK_ERROR' ||
        errorCode === 'TIMEOUT' ||
        errorCode.includes('HTTP_5') || // e.g. HTTP_500
        errorCode === 'HTTP_429';

      if (!shouldRetry) {
        return result;
      }

      if (attempt === maxRetries) {
        return result;
      }

      const delay = baseDelay * Math.pow(2, attempt);
      await new Promise(resolve => setTimeout(resolve, delay));

    } catch (error) {
      throw error;
    }
  }

  return lastResult!;
}

// ============================================================================
// TYPE GUARDS AND HELPERS
// ============================================================================

export function isApiError(error: any): error is ApiError {
  return error &&
    typeof error.code === 'string' &&
    typeof error.message === 'string';
}

export function isApiResponse<T>(response: any): response is ApiResponse<T> {
  return response &&
    typeof response === 'object' &&
    typeof response.success === 'boolean' &&
    (response.data !== undefined || response.error !== undefined);
}

export function isPaginatedResponse<T>(data: any): data is PaginatedResponse<T> {
  return data && typeof data === 'object' && Array.isArray(data.data) && 'pagination' in data;
}

// ============================================================================
// EXPORTS
// ============================================================================

// Types are already exported as interfaces above
// Removing duplicate type exports to prevent TypeScript conflicts