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

import { COOKIES } from '../auth/cookies';
import { getBackendUrl } from './url-resolver';

// ============================================================================
// CORE TYPES AND INTERFACES
// ============================================================================

import type { AdminMetadata } from '../types/api';

export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
  status: number;
  timestamp?: string;
  /** Admin-specific metadata for operations */
  admin_meta?: AdminMetadata;
}

export interface ApiError {
  status: number;
  message: string;
  code?: string;
  details?: any;
}

export interface RequestConfig extends RequestInit {
  timeout?: number;
  serverSide?: boolean;
  platform?: 'admin' | 'frontend';
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    page: number;
    per_page: number;
    total_pages: number;
    total_items: number;
  };
  metadata?: {
    query_time: number;
    cached: boolean;
    last_updated: string;
  };
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
    this.baseURL = options.baseURL || this.getDefaultBaseURL();
    this.token = options.token;
    this.isServerSide = options.serverSide || typeof window === 'undefined';
  }

  private getDefaultBaseURL(): string {
    // Use environment-specific URL resolution
    return getBackendUrl('client');
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
        // Check if we're in a Next.js environment and headers module is available
        if (typeof process !== 'undefined' && process.env['NEXT_RUNTIME']) {
          try {
            const { cookies } = await import('next/headers');
            const cookieStore = await cookies();

            // Get access token from unified cookies (no context separation)
            const tokenCookie = cookieStore.get(COOKIES.access);

            if (tokenCookie?.value) {
              headers['Authorization'] = `Bearer ${tokenCookie.value}`;
            }
          } catch (nextError) {
            // Next.js headers module might not be available in non-Next.js environments
            console.warn(`Next.js headers not available in ${this.platform} context:`, nextError);
          }
        }
      } catch (error) {
        // Cookie access might fail in some server contexts or non-Next.js environments
        console.warn(`Failed to get server-side ${this.platform} auth token:`, error);
      }
    }

    return headers;
  }

  private async request<T>(
    endpoint: string,
    config: RequestConfig = {}
  ): Promise<ApiResponse<T>> {
    const { timeout = 30000, serverSide, platform, ...options } = config;
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
      if (timeout && 'signal' in requestConfig === false) {
        const controller = new AbortController();
        setTimeout(() => controller.abort(), timeout);
        requestConfig.signal = controller.signal;
      }

      const response = await fetch(url, requestConfig);

      // Handle authentication errors
      if (response.status === 401) {
        // Don't redirect - let the application handle authentication state
        // The auth provider will show wallet connection UI when needed
        throw new APIError(401, 'Unauthorized - please log in again', 'UNAUTHORIZED');
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
        throw new APIError(
          response.status,
          errorMessage,
          data?.code || 'HTTP_ERROR',
          data?.details
        );
      }

      return {
        success: true,
        data,
        status: response.status,
        timestamp: new Date().toISOString()
      };

    } catch (error) {
      // Handle AbortError (timeout)
      if (error instanceof Error && error.name === 'AbortError') {
        throw new APIError(408, `Request timeout after ${timeout}ms`, 'TIMEOUT');
      }

      // Re-throw API errors (already proper Error instances)
      if (error instanceof APIError) {
        throw error;
      }

      // Handle network and other errors
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      throw new APIError(0, errorMessage, 'NETWORK_ERROR');
    }
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
      typeof error.status === 'number' &&
      typeof error.message === 'string';
  }

  isApiSuccess<T>(response: ApiResponse<T>): response is ApiResponse<T> & { success: true; data: T } {
    return response.success && response.data !== undefined;
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

export class APIError extends Error implements ApiError {
  public status: number;
  public code?: string;
  public details?: any;

  constructor(status: number, message: string, code?: string, details?: any) {
    super(message);
    this.name = 'APIError';
    this.status = status;
    this.code = code;
    this.details = details;
  }

  static fromResponse(response: { status: number; statusText: string; data?: any }): APIError {
    const message = response.data?.message || response.data?.error || `HTTP ${response.status}: ${response.statusText}`;
    const code = response.data?.code || 'HTTP_ERROR';
    return new APIError(response.status, message, code, response.data?.details);
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

  if (!client.isApiSuccess(response)) {
    throw new APIError(response.status, response.error || 'Failed to fetch data');
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
  const response = await client[method]<T>(endpoint, data);

  if (!client.isApiSuccess(response)) {
    throw new APIError(response.status, response.error || 'Request failed');
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
  let lastError: ApiError;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await requestFn();
    } catch (error) {
      lastError = error as ApiError;

      // Don't retry client errors (4xx) except 429 (rate limit)
      if (lastError.status >= 400 && lastError.status < 500 && lastError.status !== 429) {
        throw lastError;
      }

      // Don't retry on last attempt
      if (attempt === maxRetries) {
        throw lastError;
      }

      // Wait with exponential backoff
      const delay = baseDelay * Math.pow(2, attempt);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }

  throw lastError!;
}

// ============================================================================
// TYPE GUARDS AND HELPERS
// ============================================================================

export function isApiError(error: any): error is ApiError {
  return error &&
    typeof error.status === 'number' &&
    typeof error.message === 'string';
}

export function isApiResponse<T>(response: any): response is ApiResponse<T> {
  return response &&
    typeof response.success === 'boolean' &&
    typeof response.status === 'number';
}

export function isPaginatedResponse<T>(response: any): response is PaginatedResponse<T> {
  return response &&
    Array.isArray(response.data) &&
    response.pagination &&
    typeof response.pagination.page === 'number' &&
    typeof response.pagination.total_items === 'number';
}

// ============================================================================
// EXPORTS
// ============================================================================

// Types are already exported as interfaces above
// Removing duplicate type exports to prevent TypeScript conflicts