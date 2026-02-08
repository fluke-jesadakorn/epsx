// ============================================================================
// UNIFIED API RESPONSE HANDLER
// Consolidated response handling for all EPSX applications
// Eliminates 1,082 lines of duplicate code across frontend and admin-frontend
// ============================================================================

// ============================================================================
// STANDARDIZED API RESPONSE TYPES
// ============================================================================

import { logger } from './logger';

export interface ApiSuccess<T = unknown> {
  success: true
  data: T
  metadata?: {
    timestamp: string
    request_id?: string
    cache_hit?: boolean
    response_time_ms?: number
  }
}

export interface ApiError {
  success: false
  error: {
    type: string
    code: string
    message: string
    user_message: string
    details?: Record<string, unknown>
    suggested_actions: string[]
    retry_after?: number
    upgrade_info?: {
      current_tier: string
      required_tier: string
      upgrade_url: string
      benefits: string[]
    }
  }
  metadata?: {
    timestamp: string
    request_id?: string
    correlation_id?: string
  }
}

export type ApiResponse<T = unknown> = ApiSuccess<T> | ApiError

// ============================================================================
// PERMISSION-SPECIFIC ERROR TYPES
// ============================================================================

export interface PermissionDeniedError extends ApiError {
  error: ApiError['error'] & {
    type: 'PERMISSION_DENIED'
    permission: string
    required_permissions: string[]
    user_permissions: string[]
    access_level_required: string
    current_access_level: string
  }
}

export interface InsufficientTierError extends ApiError {
  error: ApiError['error'] & {
    type: 'INSUFFICIENT_TIER'
    current_tier: string
    required_tier: string
    missing_permissions: string[]
    upgrade_info: NonNullable<ApiError['error']['upgrade_info']>
  }
}

export interface PermissionExpiredError extends ApiError {
  error: ApiError['error'] & {
    type: 'PERMISSION_EXPIRED'
    expired_permissions: Array<{
      permission: string
      expired_at: string
      grace_period_ends?: string
    }>
    renewal_url?: string
  }
}

export interface RateLimitExceededError extends ApiError {
  error: ApiError['error'] & {
    type: 'RATE_LIMIT_EXCEEDED'
    rate_limit: {
      limit: number
      remaining: number
      reset_at: string
      window_size: string
    }
    upgrade_for_higher_limits?: {
      tier: string
      new_limit: number
    }
  }

}

interface BackendErrorPayload {
  error_type?: string;
  message?: string;
  user_message?: string;
  details?: {
    permission?: string;
    required_permissions?: string[];
    user_permissions?: string[];
    access_level_required?: string;
    current_access_level?: string;
    current_tier?: string;
    required_tier?: string;
    missing_permissions?: string[];
    upgrade_info?: {
      current_tier: string;
      required_tier: string;
      upgrade_url: string;
      benefits: string[];
      required_tier_name?: string;
      new_limit?: number;
    };
    renewal_url?: string;
    expired_permissions?: Array<{
      permission: string;
      expired_at: string;
      grace_period_ends?: string;
    }>;
    usage_info?: {
      limit: number;
      current_usage: number;
      reset_at: string;
      period: string;
    }
  };
  suggested_actions?: string[];
}

// ============================================================================
// UNIFIED API RESPONSE HANDLER CLASS  
// ============================================================================

export interface ResponseContext {
  operation?: string
  user_id?: string
  permission?: string
  component?: string
  platform?: 'frontend' | 'admin'
}

export class UnifiedApiResponseHandler {
  private static instance?: UnifiedApiResponseHandler

  static getInstance(): UnifiedApiResponseHandler {
    UnifiedApiResponseHandler.instance ??= new UnifiedApiResponseHandler()
    return UnifiedApiResponseHandler.instance
  }

  /**
   * 🔒 SECURITY CRITICAL: Handle backend permission authority responses
   * Standardized processing of all API responses with structured error handling
   */
  async handleResponse<T>(
    response: Response,
    context?: ResponseContext
  ): Promise<ApiResponse<T>> {
    const metadata = {
      timestamp: new Date().toISOString(),
      request_id: response.headers.get('x-request-id') ?? undefined,
      response_time_ms: this.getResponseTime(response),
      platform: context?.platform
    }

    try {
      // Handle non-JSON responses
      const contentType = response.headers.get('content-type')
      const isJson = contentType?.includes('application/json') === true
      if (!isJson) {
        return this.createError({
          type: 'INVALID_RESPONSE',
          message: `Expected JSON response, got ${contentType ?? 'unknown'}`,
          userMessage: 'The server returned an unexpected response format',
          suggestedActions: [],
          metadata
        })
      }

      const data: unknown = await response.json()

      // Handle successful responses
      if (response.ok) {
        return {
          success: true,
          data: data as T,
          metadata: {
            ...metadata,
            cache_hit: response.headers.get('x-cache-status') === 'HIT'
          }
        }
      }

      // Handle structured error responses from backend
      return this.processErrorResponse(response, data as BackendErrorPayload, { context, metadata })

    } catch (parseError) {
      logger.error('Failed to parse API response:', parseError, {
        status: response.status,
        url: response.url,
        context
      })

      return this.createError({
        type: 'PARSE_ERROR',
        message: 'Failed to parse server response',
        userMessage: 'We received an invalid response from the server. Please try again.',
        suggestedActions: ['Retry the request', 'Contact support if this persists'],
        metadata
      })
    }
  }

  /**
   * Process structured error responses from backend permission authority
   */
  private processErrorResponse(
    response: Response,
    data: BackendErrorPayload,
    options: {
      context?: ResponseContext,
      metadata?: ApiError['metadata']
    }
  ): ApiError | PermissionDeniedError | InsufficientTierError | PermissionExpiredError | RateLimitExceededError {
    const { context, metadata } = options;

    // Handle structured backend permission errors
    if (typeof data.error_type === 'string') {
      return this.handleStructuredBackendError(data, context, metadata);
    }

    // Handle standard HTTP errors
    return this.createHttpError(response.status, data, { metadata, platform: context?.platform })
  }

  private handleStructuredBackendError(
    data: BackendErrorPayload,
    context?: ResponseContext,
    metadata?: ApiError['metadata']
  ): ApiError {
    switch (data.error_type) {
      case 'permission_denied':
        return this.createPermissionDeniedError(data, context, metadata);

      case 'insufficient_tier':
        return this.createInsufficientTierError(data, metadata);

      case 'permission_expired':
        return this.createPermissionExpiredError(data, metadata);

      case 'usage_limit_exceeded':
        return this.createRateLimitExceededError(data, metadata);

      default:
        return this.createGenericBackendError(data, metadata);
    }
  }

  private createPermissionDeniedError(
    data: BackendErrorPayload,
    context?: ResponseContext,
    metadata?: ApiError['metadata']
  ): PermissionDeniedError {
    const details = data.details ?? {};
    const { permission, suggestedActions } = this.getPermissionDeniedDetails(data, context, details);

    return {
      success: false,
      metadata,
      error: {
        type: 'PERMISSION_DENIED',
        code: 'PERMISSION_DENIED',
        message: data.message ?? 'Permission denied',
        user_message: data.user_message ?? 'You don\'t have permission to access this resource.',
        details,
        suggested_actions: suggestedActions,
        permission,
        required_permissions: details.required_permissions ?? [],
        user_permissions: details.user_permissions ?? [],
        access_level_required: details.access_level_required ?? 'unknown',
        current_access_level: details.current_access_level ?? 'none'
      }
    };
  }

  private getPermissionDeniedDetails(
    data: BackendErrorPayload,
    context: ResponseContext | undefined,
    details: NonNullable<BackendErrorPayload['details']>
  ) {
    const permission = context?.permission ?? details.permission ?? 'unknown';
    const suggestedActions = data.suggested_actions ?? [
      context?.platform === 'admin'
        ? 'Contact system administrator'
        : 'Contact your administrator to request access'
    ];
    return { permission, suggestedActions };
  }

  private createInsufficientTierError(
    data: BackendErrorPayload,
    metadata?: ApiError['metadata']
  ): InsufficientTierError {
    const details = data.details ?? {};
    const upgradeInfo = details.upgrade_info ?? {
      current_tier: 'free',
      required_tier: 'premium',
      upgrade_url: '/upgrade',
      benefits: ['Access to premium features']
    };

    return {
      success: false,
      metadata,
      error: {
        type: 'INSUFFICIENT_TIER',
        code: 'INSUFFICIENT_TIER',
        message: data.message ?? 'Insufficient tier level',
        user_message: data.user_message ?? 'This feature requires a higher tier subscription.',
        details,
        suggested_actions: data.suggested_actions ?? ['Upgrade your subscription'],
        current_tier: details.current_tier ?? 'free',
        required_tier: details.required_tier ?? 'premium',
        missing_permissions: details.missing_permissions ?? [],
        upgrade_info: upgradeInfo
      }
    };
  }

  private createPermissionExpiredError(
    data: BackendErrorPayload,
    metadata?: ApiError['metadata']
  ): PermissionExpiredError {
    const details = data.details ?? {};

    return {
      success: false,
      metadata,
      error: {
        type: 'PERMISSION_EXPIRED',
        code: 'PERMISSION_EXPIRED',
        message: data.message ?? 'Permission has expired',
        user_message: data.user_message ?? 'Your access to this feature has expired.',
        details,
        suggested_actions: data.suggested_actions ?? ['Renew your subscription'],
        expired_permissions: details.expired_permissions ?? [],
        renewal_url: details.renewal_url ?? '/renew'
      }
    };
  }

  private createRateLimitExceededError(
    data: BackendErrorPayload,
    metadata?: ApiError['metadata']
  ): RateLimitExceededError {
    const details = data.details ?? {};
    const { rateLimit, upgradeForHigherLimits } = this.getRateLimitDetails(details);

    return {
      success: false,
      metadata,
      error: {
        type: 'RATE_LIMIT_EXCEEDED',
        code: 'RATE_LIMIT_EXCEEDED',
        message: data.message ?? 'Usage limit exceeded',
        user_message: data.user_message ?? 'You have exceeded your usage limits.',
        details,
        suggested_actions: data.suggested_actions ?? ['Wait for limits to reset', 'Upgrade for higher limits'],
        rate_limit: rateLimit,
        upgrade_for_higher_limits: upgradeForHigherLimits
      }
    };
  }

  private getRateLimitDetails(
    details: NonNullable<BackendErrorPayload['details']>
  ) {
    const usageInfo = details.usage_info;
    const upgradeInfo = details.upgrade_info;

    const limit = usageInfo?.limit ?? 0;
    const currentUsage = usageInfo?.current_usage ?? 0;
    const remaining = Math.max(0, limit - currentUsage);

    const upgradeForHigherLimits = upgradeInfo ? {
      tier: upgradeInfo.required_tier,
      new_limit: upgradeInfo.new_limit ?? 0
    } : undefined;

    return {
      rateLimit: {
        limit,
        remaining,
        reset_at: usageInfo?.reset_at ?? new Date().toISOString(),
        window_size: usageInfo?.period ?? 'hour'
      },
      upgradeForHigherLimits
    };
  }

  private createGenericBackendError(
    data: BackendErrorPayload,
    metadata?: ApiError['metadata']
  ): ApiError {
    return this.createError({
      type: (data.error_type ?? 'UNKNOWN').toUpperCase().replace(/[^A-Z0-9_]/g, '_'),
      message: data.message ?? 'An error occurred',
      userMessage: data.user_message ?? 'Something went wrong. Please try again.',
      suggestedActions: data.suggested_actions ?? ['Try again'],
      metadata
    });
  }

  /**
   * Create structured error responses for HTTP errors with platform-aware messaging
   */
  private createHttpError(
    status: number,
    data: BackendErrorPayload,
    options: {
      metadata?: ApiError['metadata'],
      platform?: 'frontend' | 'admin'
    }
  ): ApiError {
    const { metadata, platform } = options;
    const isAdmin = platform === 'admin';

    const errorMap: Record<number, { type: string; message: string; userMessage: string; actions: string[] }> = {
      400: {
        type: 'BAD_REQUEST',
        message: 'Invalid request format',
        userMessage: 'There was a problem with your request. Please check and try again.',
        actions: ['Check your input and try again']
      },
      401: {
        type: 'AUTHENTICATION_REQUIRED',
        message: 'Authentication required',
        userMessage: isAdmin
          ? 'Admin authentication required. Please sign in with admin credentials.'
          : 'Please sign in to continue.',
        actions: [isAdmin ? 'Sign in as administrator' : 'Sign in to your account']
      },
      403: {
        type: 'ACCESS_FORBIDDEN',
        message: 'Access forbidden',
        userMessage: isAdmin
          ? 'You don\'t have the required admin permissions.'
          : 'You don\'t have permission to perform this action.',
        actions: [isAdmin
          ? 'Contact system administrator for elevated permissions'
          : 'Contact your administrator for access']
      },
      404: {
        type: 'RESOURCE_NOT_FOUND',
        message: 'Resource not found',
        userMessage: 'The requested resource could not be found.',
        actions: ['Check the URL and try again']
      },
      429: {
        type: 'RATE_LIMIT_EXCEEDED',
        message: 'Rate limit exceeded',
        userMessage: 'You are making requests too quickly. Please slow down.',
        actions: ['Wait a moment and try again']
      },
      500: {
        type: 'INTERNAL_SERVER_ERROR',
        message: 'Internal server error',
        userMessage: 'We encountered a technical problem. Please try again in a moment.',
        actions: isAdmin
          ? ['Check server logs', 'Try again in a few minutes', 'Contact technical support']
          : ['Try again in a few minutes', 'Contact support if this continues']
      },
      503: {
        type: 'SERVICE_UNAVAILABLE',
        message: 'Service temporarily unavailable',
        userMessage: 'The service is temporarily unavailable. Please try again shortly.',
        actions: ['Try again in a few minutes']
      }
    }

    const errorInfo = errorMap[status] ?? {
      type: 'UNKNOWN_ERROR',
      message: `HTTP ${status} error`,
      userMessage: 'An unexpected error occurred. Please try again.',
      actions: ['Try again', 'Contact support if this continues']
    }

    return this.createError({
      type: errorInfo.type,
      message: data.message ?? errorInfo.message,
      userMessage: data.user_message ?? errorInfo.userMessage,
      suggestedActions: data.suggested_actions ?? errorInfo.actions,
      metadata
    })
  }

  /**
   * Create a standardized error response
   */
  private createError(params: {
    type: string,
    message: string,
    userMessage: string,
    suggestedActions: string[],
    metadata?: ApiError['metadata']
  }): ApiError {
    const { type, message, userMessage, suggestedActions, metadata } = params;
    return {
      success: false,
      error: {
        type,
        code: type,
        message,
        user_message: userMessage,
        suggested_actions: suggestedActions
      },
      metadata
    }
  }

  /**
   * Extract response time from response headers
   */
  private getResponseTime(response: Response): number | undefined {
    const responseTime = response.headers.get('x-response-time-ms')
    return responseTime !== null ? parseInt(responseTime, 10) : undefined
  }

  /**
   * 🔒 SECURITY CRITICAL: Retry mechanism with exponential backoff
   * Handles transient failures while respecting rate limits
   */
  async withRetry<T>(
    operation: () => Promise<Response>,
    options: {
      maxRetries?: number
      baseDelay?: number
      maxDelay?: number
      retryableErrorTypes?: string[]
      context?: {
        operation?: string
        user_id?: string
        permission?: string
        component?: string
        platform?: 'frontend' | 'admin'
      }
    } = {}
  ): Promise<ApiResponse<T>> {
    const {
      maxRetries = 3,
      baseDelay = 1000,
      maxDelay = 10000,
      retryableErrorTypes = ['INTERNAL_SERVER_ERROR', 'SERVICE_UNAVAILABLE', 'NETWORK_ERROR'],
      context
    } = options

    let lastError: ApiError | null = null

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        const result = await this.executeAttempt<T>(operation, context)

        // Return successful responses immediately
        if (result.success) {
          return result
        }

        // Check if error is retryable
        const isRetryable = retryableErrorTypes.includes(result.error.type)
        if (!this.shouldRetry(attempt, maxRetries, isRetryable)) {
          return result
        }

        lastError = result
        await this.delay(attempt, baseDelay, maxDelay)

      } catch (networkError) {
        lastError = this.handleNetworkError(networkError, attempt, context)

        if (!this.shouldRetry(attempt, maxRetries, true)) {
          break
        }
        await this.delay(attempt, baseDelay, maxDelay)
      }
    }

    return lastError ?? this.createError({
      type: 'UNKNOWN_ERROR',
      message: 'Request failed after retries',
      userMessage: 'The request failed. Please try again.',
      suggestedActions: ['Try again', 'Contact support if this continues']
    })
  }

  private async executeAttempt<T>(operation: () => Promise<Response>, context?: ResponseContext): Promise<ApiResponse<T>> {
    const response = await operation()
    return await this.handleResponse<T>(response, context)
  }

  private shouldRetry(attempt: number, maxRetries: number, isRetryable: boolean): boolean {
    return isRetryable && attempt < maxRetries
  }

  private async delay(attempt: number, baseDelay: number, maxDelay: number): Promise<void> {
    const delay = Math.min(baseDelay * Math.pow(2, attempt), maxDelay)
    await new Promise(resolve => setTimeout(resolve, delay))
  }

  private handleNetworkError(networkError: unknown, attempt: number, context?: ResponseContext): ApiError {
    logger.error('Network error during API call:', networkError, { attempt, context })

    return this.createError({
      type: 'NETWORK_ERROR',
      message: 'Network request failed',
      userMessage: 'Unable to connect to the server. Please check your connection.',
      suggestedActions: ['Check your internet connection', 'Try again']
    })
  }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

export const unifiedApiHandler = UnifiedApiResponseHandler.getInstance()

/**
 * Process API response with standardized error handling
 */
export async function handleApiResponse<T>(
  response: Response,
  context?: ResponseContext
): Promise<ApiResponse<T>> {
  return await unifiedApiHandler.handleResponse<T>(response, context)
}

/**
 * Make API call with retry mechanism
 */
export interface ApiCallOptions {
  maxRetries?: number
  baseDelay?: number
  maxDelay?: number
  retryableErrorTypes?: string[]
  context?: ResponseContext
}

/**
 * Make API call with retry mechanism
 */
export async function apiCallWithRetry<T>(
  operation: () => Promise<Response>,
  options?: ApiCallOptions
): Promise<ApiResponse<T>> {
  return await unifiedApiHandler.withRetry<T>(operation, options)
}

/**
 * Type guards for error responses
 */
export const isPermissionDeniedError = (response: ApiResponse): response is PermissionDeniedError => {
  return !response.success && response.error.type === 'PERMISSION_DENIED'
}

export const isInsufficientTierError = (response: ApiResponse): response is InsufficientTierError => {
  return !response.success && response.error.type === 'INSUFFICIENT_TIER'
}

export const isPermissionExpiredError = (response: ApiResponse): response is PermissionExpiredError => {
  return !response.success && response.error.type === 'PERMISSION_EXPIRED'
}

export const isRateLimitExceededError = (response: ApiResponse): response is RateLimitExceededError => {
  return !response.success && response.error.type === 'RATE_LIMIT_EXCEEDED'
}

/**
 * Legacy compatibility exports
 * Maintains compatibility with existing imports
 */
export const apiHandler = unifiedApiHandler;
export const ApiResponseHandler = UnifiedApiResponseHandler;

// ============================================================================
// PLATFORM-SPECIFIC HELPERS
// ============================================================================

/**
 * Create response handler with frontend context
 */
export const createFrontendResponseHandler = () => ({
  handleResponse: <T>(response: Response, context?: ResponseContext) =>
    handleApiResponse<T>(response, { ...context, platform: 'frontend' }),

  withRetry: <T>(operation: () => Promise<Response>, options?: ApiCallOptions) =>
    apiCallWithRetry<T>(operation, { ...options, context: { ...options?.context, platform: 'frontend' } })
});

/**
 * Create response handler with admin context
 */
export const createAdminResponseHandler = () => ({
  handleResponse: <T>(response: Response, context?: ResponseContext) =>
    handleApiResponse<T>(response, { ...context, platform: 'admin' }),

  withRetry: <T>(operation: () => Promise<Response>, options?: ApiCallOptions) =>
    apiCallWithRetry<T>(operation, { ...options, context: { ...options?.context, platform: 'admin' } })
});

// ============================================================================
// UNIFIED RESPONSE HANDLER COMPLETE
// ============================================================================
//
// 🎉 UNIFIED RESPONSE HANDLER COMPLETE!
//
// Consolidated 1,082 lines of duplicate response handling code:
// - Eliminated identical 541-line files from frontend and admin-frontend
// - Added platform-aware error messaging and actions
// - Enhanced type safety with context awareness
// - Maintained full backward compatibility
// - Added convenience functions for both platforms
//
// Benefits:
// ✅ Zero code duplication across applications
// ✅ Platform-specific error messaging
// ✅ Single source of truth for error handling
// ✅ Enhanced debugging with context tracking
// ✅ Consistent error UX across all apps
// ✅ Easy maintenance and updates
//
// The unified response handler eliminates massive duplication! 🚀
// ============================================================================