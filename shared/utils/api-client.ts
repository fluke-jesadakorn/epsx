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

import { refreshSessionAction } from '../auth/actions';
import { COOKIES } from '../auth/cookies';
import { isApiError, isApiResponse, isApiSuccess, isPaginatedResponse, type ApiError, type ApiResponse, type PaginatedResponse } from '../types/api';
import { logger } from './logger';
import { getBackendUrl } from './url-resolver';

// ============================================================================
// SESSION EXPIRY HANDLER
// ============================================================================

let sessionExpiredHandler: (() => void) | undefined;
let isRedirecting = false;

// Mutex for token refresh to prevent simultaneous refresh attempts
let refreshPromise: Promise<{ success: boolean; access_token?: string }> | null = null;

// Shared client-side token accessible by all UnifiedApiClient instances.
// Set by auth provider after login/hydration so client-side API calls work
// (access_token cookie is HttpOnly and unreadable by JS).
let sharedClientToken: string | undefined;

export function setSharedClientToken(token: string | undefined): void {
  sharedClientToken = token;
}

export function registerSessionExpiredHandler(fn: () => void): void {
  sessionExpiredHandler = fn;
}

// ============================================================================
// CORE TYPES AND INTERFACES
// ============================================================================

export type { ApiError, ApiResponse, PaginatedResponse };

export interface RequestConfig extends RequestInit {
  timeout?: number;
  serverSide?: boolean;
  platform?: 'admin' | 'frontend';
  token?: string;
  _isRetry?: boolean;
}

export type Platform = 'admin' | 'frontend';

// ============================================================================
// ERROR CLASSES
// ============================================================================

export class APIError extends Error implements ApiError {
  public code: string;
  public details?: unknown;
  public status?: number; // Optional status for compatibility
  public requestId?: string;

  constructor(message: string, code = 'UNKNOWN_ERROR', options: { status?: number; details?: unknown; requestId?: string } = {}) {
    super(message);
    this.name = 'APIError';
    this.code = code;
    this.status = options.status;
    this.details = options.details;
    this.requestId = options.requestId;
  }

  static fromResponse(response: {
    status: number;
    statusText: string;
    data?: {
      message?: string;
      error?: { message?: string; code?: string; details?: unknown };
      code?: string;
      details?: unknown;
    };
  }): APIError {
    const { status, statusText, data } = response;
    let message = `HTTP ${status}: ${statusText}`;
    let code = 'HTTP_ERROR';
    let details: unknown = undefined;

    if (data !== undefined) {
      const error = data.error;
      message = determineErrorMessage({
        dataMessage: data.message,
        errorMessage: error?.message,
        status,
        statusText
      });
      code = determineErrorCode(data.code, error?.code);
      details = data.details ?? error?.details;
    }

    return new APIError(message, code, { status, details });
  }
}

function determineErrorMessage(ctx: {
  dataMessage?: string;
  errorMessage?: string;
  status: number;
  statusText: string;
}): string {
  const { dataMessage, errorMessage, status, statusText } = ctx;
  if (dataMessage !== undefined && dataMessage !== '') {
    return dataMessage;
  }
  if (errorMessage !== undefined && errorMessage !== '') {
    return errorMessage;
  }
  return `HTTP ${status}: ${statusText}`;
}

function determineErrorCode(
  dataCode: string | undefined,
  errorCode: string | undefined
): string {
  if (dataCode !== undefined && dataCode !== '') {
    return dataCode;
  }
  if (errorCode !== undefined && errorCode !== '') {
    return errorCode;
  }
  return 'HTTP_ERROR';
}

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
    this.baseURL = options.baseURL ?? this.getDefaultBaseURL();
    this.token = options.token;
  }

  public getPlatform(): Platform {
    return this.platform;
  }

  public getBaseURL(): string {
    return this.baseURL;
  }

  private getDefaultBaseURL(): string {
    return getBackendUrl(this.isServerSide ? 'server' : 'client');
  }

  protected async getAuthHeaders(config?: RequestConfig): Promise<HeadersInit> {
    const configHeaders = config?.headers ?? {};
    const baseHeaders: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    // Prioritize token from config, then instance token, then fetched token
    const requestToken = config?.token ?? this.token;

    if (requestToken !== undefined && requestToken !== '') {
      baseHeaders['Authorization'] = `Bearer ${requestToken}`;
    } else {
      const fetchedToken = this.isServerSide
        ? await this.getServerSideToken()
        : this.getClientSideToken();

      if (fetchedToken !== undefined && fetchedToken !== '') {
        baseHeaders['Authorization'] = `Bearer ${fetchedToken}`;
      }
    }

    return { ...baseHeaders, ...configHeaders } as HeadersInit;
  }

  private async getServerSideToken(): Promise<string | undefined> {
    try {
      const { cookies } = await import('next/headers');
      const { getServerAuthToken } = await import('../auth/cookies');
      const cookieStore = await cookies();
      return getServerAuthToken(cookieStore) ?? undefined;
    } catch {
      return undefined;
    }
  }

  private getClientSideToken(): string | undefined {
    // Per-instance token first, then shared token from auth provider
    return this.token ?? sharedClientToken;
  }

  private async request<T>(
    endpoint: string,
    config: RequestConfig = {}
  ): Promise<ApiResponse<T>> {
    const { timeout = 0, _isRetry: _, ...options } = config;
    const url = `${this.baseURL}${endpoint}`;

    try {
      const headers = await this.getAuthHeaders(config);
      const requestConfig = this.prepareRequestConfig(options, headers, timeout);

      const response = await fetch(url, requestConfig);

      if (response.status === 401 && !endpoint.includes('/api/auth/session/refresh')) {
        const result = await this.handleUnauthorized(endpoint, config, headers as Record<string, string>);
        if (result !== null) { return result as ApiResponse<T>; }
      }

      const data = await this.parseResponseData(response);

      // 403: Don't throw — return normalized response to preserve status for redirect handling
      if (!response.ok && response.status !== 403) {
        this.handleErrorResponse({
          response,
          data,
          method: requestConfig.method ?? 'GET',
          url
        });
      }

      return this.normalizeResponse(response, data);
    } catch (error) {
      return this.handleRequestError<T>(error, timeout, options.headers);
    }
  }

  private prepareRequestConfig(options: RequestInit, headers: HeadersInit, timeout: number): RequestInit {
    const requestConfig: RequestInit = {
      ...options,
      headers: { ...headers, ...options.headers },
      credentials: this.isServerSide ? undefined : 'include',
      cache: this.isServerSide ? 'no-store' : 'default',
    };
    this.applyTimeout(requestConfig, timeout);
    return requestConfig;
  }

  private async handleUnauthorized(endpoint: string, config: RequestConfig, headers: Record<string, string>): Promise<unknown | null> {
    if (config._isRetry === true) { return null; }
    if (!this.isServerSide && isRedirecting) { return this.createUnauthorizedResponse(headers); }

    const refreshResult = await this.handleTokenRefresh();
    if (refreshResult.success && refreshResult.access_token !== undefined && refreshResult.access_token !== '') {
      return await this.request(endpoint, {
        ...config,
        _isRetry: true
      });
    }

    if (!this.isServerSide) { this.triggerSessionExpiry(); }

    return this.createUnauthorizedResponse(headers);
  }

  private triggerSessionExpiry(): void {
    const onAuthPage = typeof window !== 'undefined' && window.location.pathname.startsWith('/auth');
    // Don't redirect if there was never an active session — avoids loop on unauthenticated page load
    if (isRedirecting || onAuthPage || sharedClientToken === undefined) { return; }
    isRedirecting = true;
    if (sessionExpiredHandler !== undefined) {
      sessionExpiredHandler();
    } else if (typeof window !== 'undefined') {
      window.location.href = '/auth?clear=true';
    }
  }

  private applyTimeout(config: RequestInit, timeout: number): void {
    if (timeout !== 0 && !('signal' in config)) {
      const controller = new AbortController();
      setTimeout(() => controller.abort(), timeout);
      config.signal = controller.signal;
    }
  }

  private handleRequestError<T>(error: unknown, timeout: number, headers?: HeadersInit): ApiResponse<T> {
    // Handle AbortError (timeout)
    if (error instanceof Error && error.name === 'AbortError') {
      return {
        success: false,
        data: null as unknown as T,
        error: {
          code: 'TIMEOUT',
          message: `Request timeout after ${timeout}ms`,
          requestId: (headers as Record<string, string> | undefined)?.['x-request-id']
        }
      };
    }

    // Re-throw if it's already a handled response
    if (this.isApiResponse<T>(error)) {
      return error;
    }

    // Handle APIError (from handleErrorResponse) - preserve parsed error info
    if (error instanceof APIError) {
      return {
        success: false,
        data: null as unknown as T,
        error: {
          code: error.code,
          message: error.message,
          status: error.status,
          details: error.details,
          requestId: error.requestId
        }
      };
    }

    // Handle network and other unexpected errors (keep serializable for RSC)
    const errorMessage = error instanceof Error ? error.message : 'Unknown network error';
    return {
      success: false,
      data: null as unknown as T,
      error: {
        code: 'NETWORK_ERROR',
        message: errorMessage,
      }
    };
  }

  /**
   * Internal helper to handle token refresh logic for 401 errors.
   * Uses a mutex to deduplicate concurrent refresh attempts.
   */
  private async handleTokenRefresh(): Promise<{ success: boolean; access_token?: string }> {
    // Deduplicate: if a refresh is already in progress, reuse it
    if (!this.isServerSide && refreshPromise !== null) {
      return refreshPromise;
    }

    const doRefresh = async (): Promise<{ success: boolean; access_token?: string }> => {
      try {
        logger.debug('[UnifiedApiClient] 401 detected, attempting token refresh...');

        const result = this.isServerSide
          ? await this.refreshServerToken()
          : await this.refreshClientToken();

        if (result.success && result.access_token !== undefined && result.access_token !== '') {
          logger.debug('[UnifiedApiClient] Token refreshed successfully');
          this.setAuthToken(result.access_token);
          return result;
        }

        logger.warn('[UnifiedApiClient] Token refresh attempt failed');
        return { success: false };
      } catch (refreshError) {
        logger.error('[UnifiedApiClient] Error during token refresh:', refreshError);
        return { success: false };
      } finally {
        refreshPromise = null;
      }
    };

    if (!this.isServerSide) {
      refreshPromise = doRefresh();
      return refreshPromise;
    }
    return doRefresh();
  }

  private async refreshServerToken(): Promise<{ success: boolean; access_token?: string }> {
    try {
      const { cookies } = await import('next/headers');
      const cookieStore = await cookies();
      const refreshToken = cookieStore.get(COOKIES.refresh_token)?.value;

      if (refreshToken === undefined || refreshToken === '') { return { success: false }; }

      const clientId = process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID ?? 'epsx-frontend';

      const response = await fetch(`${this.baseURL}/api/auth/session/refresh`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ refresh_token: refreshToken, client_id: clientId }),
      });

      if (!response.ok) { return { success: false }; }

      const data = (await response.json()) as { access_token?: string; refresh_token?: string; expires_in?: number };
      if (data.access_token === undefined || data.access_token === '') { return { success: false }; }

      this.updateServerCookies(cookieStore, data as { access_token: string; refresh_token?: string; expires_in?: number });
      return { success: true, access_token: data.access_token };
    } catch {
      return { success: false };
    }
  }

  private updateServerCookies(cookieStore: { set: (name: string, value: string, options: Record<string, unknown>) => void }, data: { access_token: string; refresh_token?: string; expires_in?: number }): void {
    const isProd = process.env.NODE_ENV === 'production';
    cookieStore.set(COOKIES.access_token, data.access_token, {
      httpOnly: true,
      secure: isProd,
      sameSite: 'lax',
      path: '/',
      maxAge: data.expires_in ?? 3600,
    } as Record<string, unknown>);

    if (data.refresh_token !== undefined && data.refresh_token !== '') {
      cookieStore.set(COOKIES.refresh_token, data.refresh_token, {
        httpOnly: true,
        secure: isProd,
        sameSite: 'lax',
        path: '/',
        maxAge: 2592000,
      } as Record<string, unknown>);
    }
  }

  private async refreshClientToken(): Promise<{ success: boolean; access_token?: string }> {
    try {
      const refreshResult = (await refreshSessionAction()) as { success: boolean; access_token?: string };

      if (refreshResult.success && refreshResult.access_token !== undefined && refreshResult.access_token !== '') {
        this.token = refreshResult.access_token;
        // Share refreshed token with all client instances
        sharedClientToken = refreshResult.access_token;
        return { success: true, access_token: refreshResult.access_token };
      }
      return { success: false };
    } catch {
      return { success: false };
    }
  }

  /**
   * Normalizes the response to ensure it adheres to the ApiResponse<T> contract.
   * Handles legacy responses, double-wrapping, and error conversion.
   */
  private normalizeResponse<T>(response: Response, data: unknown): ApiResponse<T> {
    const isSuccess = response.ok;

    if (!isSuccess) {
      return this.normalizeErrorResponse<T>(response, data);
    }

    if (this.isApiResponse<T>(data)) {
      return data;
    }

    return {
      success: true,
      data: data as T,
      error: null,
      meta: {
        timestamp: new Date().toISOString(),
        trace_id: response.headers.get('x-request-id') ?? undefined
      }
    };
  }

  private normalizeErrorResponse<T>(_response: Response, data: unknown): ApiResponse<T> {
    const status = _response.status;
    const defaultMessage = `HTTP ${status}: ${_response.statusText}`;
    const requestId = _response.headers.get('x-request-id') ?? undefined;

    let apiError: ApiError = {
      code: 'HTTP_ERROR',
      message: defaultMessage,
      status,
      requestId
    };

    if (data !== null && typeof data === 'object') {
      apiError = this.extractApiError({
        data: data as Record<string, unknown>,
        defaultMessage,
        status,
        requestId
      });
    }

    return {
      success: false,
      data: null as unknown as T,
      error: apiError,
      meta: {
        timestamp: new Date().toISOString(),
        trace_id: requestId
      }
    };
  }

  private extractApiError(ctx: {
    data: Record<string, unknown>;
    defaultMessage: string;
    status: number;
    requestId?: string;
  }): ApiError {
    const { data, defaultMessage, status, requestId } = ctx;
    const error = data.error;
    const message = data.message;
    const code = data.code;
    const details = data.details;

    if (error !== null && typeof error === 'object') {
      const nestedError = error as Record<string, unknown>;
      return {
        code: (nestedError.code as string | undefined) ?? (typeof code === 'string' ? code : 'HTTP_ERROR'),
        message: (nestedError.message as string | undefined) ?? (typeof message === 'string' ? message : defaultMessage),
        details: nestedError.details ?? details,
        status,
        requestId: (nestedError.requestId as string | undefined) ?? requestId
      };
    }

    return this.fallbackErrorExtraction(ctx);
  }

  private fallbackErrorExtraction(ctx: {
    data: Record<string, unknown>;
    defaultMessage: string;
    status: number;
    requestId?: string;
  }): ApiError {
    const { data, defaultMessage, status, requestId } = ctx;
    const message = data.message;
    const code = data.code;
    const details = data.details;
    const error = data.error;

    if (typeof message === 'string' && message !== '') {
      return {
        message,
        code: (typeof code === 'string' && code !== '') ? code : `HTTP_${status}`,
        details: details ?? data,
        status,
        requestId
      };
    }

    if (typeof error === 'string' && error !== '') {
      return {
        message: error,
        code: (typeof code === 'string' && code !== '') ? code : `HTTP_${status}`,
        status,
        requestId
      };
    }

    return { code: 'HTTP_ERROR', message: defaultMessage, status, requestId };
  }

  private isApiResponse<T>(data: unknown): data is ApiResponse<T> {
    return Boolean(data !== null && typeof data === 'object' && 'success' in (data as Record<string, unknown>) && 'data' in (data as Record<string, unknown>));
  }

  private createUnauthorizedResponse<T>(headers: HeadersInit): ApiResponse<T> {
    return {
      success: false,
      data: null as unknown as T,
      error: {
        code: 'UNAUTHORIZED',
        message: 'Unauthorized - please log in again',
        requestId: (headers as Record<string, string>)['x-request-id'],
      },
      meta: {
        timestamp: new Date().toISOString(),
      }
    };
  }

  private async parseResponseData(response: Response): Promise<unknown> {
    try {
      const contentType = response.headers.get('content-type');
      if (contentType?.includes('application/json') === true) {
        return await response.json();
      }
      return null; // Or handle other content types if needed
    } catch {
      return null;
    }
  }

  private handleErrorResponse(options: {
    response: Response;
    data: unknown;
    method: string;
    url: string;
  }): never {
    const { response, data, method, url } = options;
    const parsedData = data as { message?: string; error?: string; code?: string; details?: unknown; requestId?: string } | null;
    const message = parsedData?.message;
    const error = parsedData?.error;
    const errorMessage = (typeof message === 'string' && message !== '')
      ? message
      : (typeof error === 'string' && error !== '' ? error : `HTTP ${response.status}: ${response.statusText}`);

    if (response.status !== 404) {
      logger.error(`[UnifiedApiClient] Error ${response.status} ${method} ${url}:`, errorMessage);
    }

    throw new APIError(
      errorMessage,
      parsedData?.code ?? 'HTTP_ERROR',
      { status: response.status, details: parsedData?.details, requestId: parsedData?.requestId }
    );
  }

  // ============================================================================
  // GENERIC HTTP METHODS
  // ============================================================================

  async get<T = unknown>(endpoint: string, params?: Record<string, unknown> | object, config?: RequestConfig): Promise<ApiResponse<T>> {
    const queryString = (params !== undefined) ? `?${new URLSearchParams(
      Object.entries(params).reduce<Record<string, string>>((acc, [key, value]) => {
        if (value !== undefined && value !== null) {
          acc[key] = String(value);
        }
        return acc;
      }, {})
    ).toString()}` : '';

    return await this.request<T>(`${endpoint}${queryString}`, {
      method: 'GET',
      ...config
    });
  }

  async post<T = unknown>(endpoint: string, data?: unknown, config?: RequestConfig): Promise<ApiResponse<T>> {
    return await this.request<T>(endpoint, {
      method: 'POST',
      body: (data !== undefined && data !== null) ? JSON.stringify(data) : undefined,
      ...config
    });
  }

  async put<T = unknown>(endpoint: string, data?: unknown, config?: RequestConfig): Promise<ApiResponse<T>> {
    return await this.request<T>(endpoint, {
      method: 'PUT',
      body: (data !== undefined && data !== null) ? JSON.stringify(data) : undefined,
      ...config
    });
  }

  async delete<T = unknown>(endpoint: string, config?: RequestConfig): Promise<ApiResponse<T>> {
    return await this.request<T>(endpoint, {
      method: 'DELETE',
      ...config
    });
  }

  async patch<T = unknown>(endpoint: string, data?: unknown, config?: RequestConfig): Promise<ApiResponse<T>> {
    return await this.request<T>(endpoint, {
      method: 'PATCH',
      body: (data !== undefined && data !== null) ? JSON.stringify(data) : undefined,
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

  isApiError(error: unknown): error is ApiError {
    return isApiError(error);
  }

  isApiSuccess<T>(response: ApiResponse<T>): response is ApiResponse<T> & { success: true; data: T } {
    return response.success === true && response.data !== null;
  }

  // Create a new client with different configuration
  clone(overrides: Partial<{
    baseURL: string;
    token: string;
    platform: Platform;
    serverSide: boolean;
  }>): UnifiedApiClient {
    return new UnifiedApiClient({
      baseURL: overrides.baseURL ?? this.baseURL,
      token: overrides.token ?? this.token,
      platform: overrides.platform ?? this.platform,
      serverSide: overrides.serverSide ?? this.isServerSide
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
    const { timeout = 0, serverSide: _serverSide, platform: _platform, ...options } = config;
    const url = `${this.baseURL}${endpoint}`;

    const baseHeaders = await this.getAuthHeaders(config);
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

    if (timeout !== 0 && !('signal' in requestConfig)) {
      const controller = new AbortController();
      setTimeout(() => controller.abort(), timeout);
      requestConfig.signal = controller.signal;
    }

    return await fetch(url, requestConfig);
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

// Classes moved to the top

// ============================================================================
// UTILITY FUNCTIONS FOR COMMON API PATTERNS
// ============================================================================

/**
 * Handle paginated API responses with proper error handling
 */
export async function handlePaginatedRequest<T>(
  client: UnifiedApiClient,
  endpoint: string,
  params: Record<string, unknown> = {}
): Promise<PaginatedResponse<T>> {
  const response = await client.get<PaginatedResponse<T>>(endpoint, params);

  if (response.success !== true || response.data === null) {
    const apiError = response.error;
    throw new APIError(
      apiError?.message ?? 'Failed to fetch data',
      apiError?.code ?? 'FETCH_ERROR',
      { status: apiError?.status, details: apiError?.details, requestId: apiError?.requestId }
    );
  }

  return response.data;
}

/**
 * Handle simple API requests with error handling
 */
export async function handleSimpleRequest<T>(
  client: UnifiedApiClient,
  options: {
    method: 'get' | 'post' | 'put' | 'delete' | 'patch';
    endpoint: string;
    data?: unknown;
  }
): Promise<T> {
  const { method, endpoint, data } = options;
  // DIAGNOSTIC LOG: Trace usage of this supposedly unused function
  logger.info(`[UnifiedApiClient] handleSimpleRequest called: ${method.toUpperCase()} ${endpoint}`);

  let response: ApiResponse<T>;
  switch (method) {
    case 'get':
      response = await client.get<T>(endpoint, data as Record<string, unknown>);
      break;
    case 'delete':
      response = await client.delete<T>(endpoint, data as RequestConfig);
      break;
    default:
      response = await client[method]<T>(endpoint, data);
      break;
  }

  if (response.success !== true || response.data === null) {
    const apiError = response.error;
    const message = apiError?.message ?? 'Unknown error';
    logger.error(`[UnifiedApiClient] handleSimpleRequest failed: ${message}`);
    throw new APIError(message, apiError?.code ?? 'HTTP_ERROR', { status: apiError?.status, details: apiError?.details, requestId: apiError?.requestId });
  }

  return response.data;
}

function shouldRetryRequest(error: { code?: string } | null | undefined): boolean {
  if (error === null || error === undefined) { return false; }
  const errorCode = error.code ?? '';
  return (
    errorCode === 'NETWORK_ERROR' ||
    errorCode === 'TIMEOUT' ||
    errorCode.includes('HTTP_5') || // e.g. HTTP_500
    errorCode === 'HTTP_429'
  );
}

/**
 * Retry API requests with exponential backoff
 */
export async function retryRequest<T>(
  requestFn: () => Promise<ApiResponse<T>>,
  maxRetries = 3,
  baseDelay = 1000
): Promise<ApiResponse<T>> {
  let lastResult: ApiResponse<T> | undefined;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    const result = await requestFn();

    if (result.success === true) {
      return result;
    }

    lastResult = result;

    if (!shouldRetryRequest(result.error) || attempt === maxRetries) {
      return result;
    }

    const delay = baseDelay * Math.pow(2, attempt);
    await new Promise(resolve => setTimeout(resolve, delay));
  }

  if (!lastResult) {
    throw new Error('Retry loop exited without result');
  }

  return lastResult;
}

// ============================================================================
// TYPE GUARDS AND HELPERS
// ============================================================================

export { isApiError, isApiResponse, isApiSuccess, isPaginatedResponse };

// ============================================================================
// EXPORTS
// ============================================================================

// Types are already exported as interfaces above
// Removing duplicate type exports to prevent TypeScript conflicts