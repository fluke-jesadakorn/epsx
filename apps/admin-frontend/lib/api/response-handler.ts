// ============================================================================
// API RESPONSE HANDLER (Admin Frontend - Backend-Only Permission Architecture)
// Standardized handling for all backend permission authority responses
// ============================================================================

// Basic permission types for API responses

// ============================================================================
// STANDARDIZED API RESPONSE TYPES
// ============================================================================

export interface ApiSuccess<T = any> {
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
    details?: Record<string, any>
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

export type ApiResponse<T = any> = ApiSuccess<T> | ApiError

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

// ============================================================================
// API RESPONSE HANDLER CLASS  
// ============================================================================

export class ApiResponseHandler {
  private static instance: ApiResponseHandler
  
  static getInstance(): ApiResponseHandler {
    if (!ApiResponseHandler.instance) {
      ApiResponseHandler.instance = new ApiResponseHandler()
    }
    return ApiResponseHandler.instance
  }

  /**
   * 🔒 SECURITY CRITICAL: Handle backend permission authority responses
   * Standardized processing of all API responses with structured error handling
   */
  async handleResponse<T>(
    response: Response,
    context?: {
      operation?: string
      user_id?: string
      permission?: string
      component?: string
    }
  ): Promise<ApiResponse<T>> {
    const metadata = {
      timestamp: new Date().toISOString(),
      request_id: response.headers.get('x-request-id') || undefined,
      response_time_ms: this.getResponseTime(response)
    }

    try {
      // Handle non-JSON responses
      const contentType = response.headers.get('content-type')
      if (!contentType?.includes('application/json')) {
        return this.createError('INVALID_RESPONSE', 
          `Expected JSON response, got ${contentType}`,
          'The server returned an unexpected response format',
          [], metadata
        )
      }

      const data = await response.json()

      // Handle successful responses
      if (response.ok) {
        return {
          success: true,
          data,
          metadata: {
            ...metadata,
            cache_hit: response.headers.get('x-cache-status') === 'HIT'
          }
        }
      }

      // Handle structured error responses from backend
      return this.processErrorResponse(response, data, context, metadata)

    } catch (parseError) {
      console.error('Failed to parse API response:', parseError, {
        status: response.status,
        url: response.url,
        context
      })

      return this.createError('PARSE_ERROR',
        'Failed to parse server response',
        'We received an invalid response from the server. Please try again.',
        ['Retry the request', 'Contact support if this persists'],
        metadata
      )
    }
  }

  /**
   * Process structured error responses from backend permission authority
   */
  private processErrorResponse(
    response: Response,
    data: any,
    context?: { operation?: string; user_id?: string; permission?: string; component?: string },
    metadata?: any
  ): ApiError | PermissionDeniedError | InsufficientTierError | PermissionExpiredError | RateLimitExceededError {
    const baseError = {
      success: false as const,
      metadata
    }

    // Handle structured backend permission errors
    if (data.error_type) {
      switch (data.error_type) {
        case 'permission_denied':
          return {
            ...baseError,
            error: {
              type: 'PERMISSION_DENIED',
              code: 'PERMISSION_DENIED',
              message: data.message || 'Permission denied',
              user_message: data.user_message || 'You don\'t have permission to access this resource.',
              details: data.details || {},
              suggested_actions: data.suggested_actions || ['Contact your administrator to request access'],
              permission: context?.permission || data.details?.permission || 'unknown',
              required_permissions: data.details?.required_permissions || [],
              user_permissions: data.details?.user_permissions || [],
              access_level_required: data.details?.access_level_required || 'unknown',
              current_access_level: data.details?.current_access_level || 'none'
            }
          } as PermissionDeniedError

        case 'insufficient_tier':
          return {
            ...baseError,
            error: {
              type: 'INSUFFICIENT_TIER',
              code: 'INSUFFICIENT_TIER', 
              message: data.message || 'Insufficient tier level',
              user_message: data.user_message || 'This feature requires a higher tier subscription.',
              details: data.details || {},
              suggested_actions: data.suggested_actions || ['Upgrade your subscription'],
              current_tier: data.details?.current_tier || 'free',
              required_tier: data.details?.required_tier || 'premium',
              missing_permissions: data.details?.missing_permissions || [],
              upgrade_info: data.details?.upgrade_info || {
                current_tier: 'free',
                required_tier: 'premium',
                upgrade_url: '/upgrade',
                benefits: ['Access to premium features']
              }
            }
          } as InsufficientTierError

        case 'permission_expired':
          return {
            ...baseError,
            error: {
              type: 'PERMISSION_EXPIRED',
              code: 'PERMISSION_EXPIRED',
              message: data.message || 'Permission has expired',
              user_message: data.user_message || 'Your access to this feature has expired.',
              details: data.details || {},
              suggested_actions: data.suggested_actions || ['Renew your subscription'],
              expired_permissions: data.details?.expired_permissions || [],
              renewal_url: data.details?.renewal_url || '/renew'
            }
          } as PermissionExpiredError

        case 'usage_limit_exceeded':
          return {
            ...baseError,
            error: {
              type: 'RATE_LIMIT_EXCEEDED',
              code: 'RATE_LIMIT_EXCEEDED',
              message: data.message || 'Usage limit exceeded',
              user_message: data.user_message || 'You have exceeded your usage limits.',
              details: data.details || {},
              suggested_actions: data.suggested_actions || ['Wait for limits to reset', 'Upgrade for higher limits'],
              rate_limit: {
                limit: data.details?.usage_info?.limit || 0,
                remaining: Math.max(0, (data.details?.usage_info?.limit || 0) - (data.details?.usage_info?.current_usage || 0)),
                reset_at: data.details?.usage_info?.reset_at || new Date().toISOString(),
                window_size: data.details?.usage_info?.period || 'hour'
              },
              upgrade_for_higher_limits: data.details?.upgrade_info ? {
                tier: data.details.upgrade_info.required_tier,
                new_limit: data.details.upgrade_info.new_limit || 0
              } : undefined
            }
          } as RateLimitExceededError

        default:
          return this.createError(
            data.error_type.toUpperCase().replace(/[^A-Z0-9_]/g, '_'),
            data.message || 'An error occurred',
            data.user_message || 'Something went wrong. Please try again.',
            data.suggested_actions || ['Try again'],
            metadata
          )
      }
    }

    // Handle standard HTTP errors
    return this.createHttpError(response.status, data, metadata)
  }

  /**
   * Create structured error responses for HTTP errors
   */
  private createHttpError(status: number, data: any, metadata?: any): ApiError {
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
        userMessage: 'Please sign in to continue.',
        actions: ['Sign in to your account']
      },
      403: {
        type: 'ACCESS_FORBIDDEN',
        message: 'Access forbidden',
        userMessage: 'You don\'t have permission to perform this action.',
        actions: ['Contact your administrator for access']
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
        actions: ['Try again in a few minutes', 'Contact support if this continues']
      },
      503: {
        type: 'SERVICE_UNAVAILABLE',
        message: 'Service temporarily unavailable',
        userMessage: 'The service is temporarily unavailable. Please try again shortly.',
        actions: ['Try again in a few minutes']
      }
    }

    const errorInfo = errorMap[status] || {
      type: 'UNKNOWN_ERROR',
      message: `HTTP ${status} error`,
      userMessage: 'An unexpected error occurred. Please try again.',
      actions: ['Try again', 'Contact support if this continues']
    }

    return this.createError(
      errorInfo.type,
      data?.message || errorInfo.message,
      data?.user_message || errorInfo.userMessage,
      data?.suggested_actions || errorInfo.actions,
      metadata
    )
  }

  /**
   * Create a standardized error response
   */
  private createError(
    type: string,
    message: string,
    userMessage: string,
    suggestedActions: string[],
    metadata?: any
  ): ApiError {
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
    return responseTime ? parseInt(responseTime, 10) : undefined
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
        const response = await operation()
        const result = await this.handleResponse<T>(response, context)

        // Return successful responses immediately
        if (result.success) {
          return result
        }

        // Check if error is retryable
        const isRetryable = retryableErrorTypes.includes(result.error.type)
        const hasRetriesLeft = attempt < maxRetries

        if (!isRetryable || !hasRetriesLeft) {
          return result
        }

        lastError = result
        
        // Wait before retrying with exponential backoff
        const delay = Math.min(baseDelay * Math.pow(2, attempt), maxDelay)
        await new Promise(resolve => setTimeout(resolve, delay))

      } catch (networkError) {
        console.error('Network error during API call:', networkError, { attempt, context })
        
        lastError = this.createError(
          'NETWORK_ERROR',
          'Network request failed',
          'Unable to connect to the server. Please check your connection.',
          ['Check your internet connection', 'Try again']
        )

        const hasRetriesLeft = attempt < maxRetries
        if (!hasRetriesLeft) {
          break
        }

        // Wait before retrying
        const delay = Math.min(baseDelay * Math.pow(2, attempt), maxDelay)
        await new Promise(resolve => setTimeout(resolve, delay))
      }
    }

    return lastError || this.createError(
      'UNKNOWN_ERROR',
      'Request failed after retries',
      'The request failed. Please try again.',
      ['Try again', 'Contact support if this continues']
    )
  }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

export const apiHandler = ApiResponseHandler.getInstance()

/**
 * Process API response with standardized error handling
 */
export async function handleApiResponse<T>(
  response: Response,
  context?: { operation?: string; user_id?: string; permission?: string; component?: string }
): Promise<ApiResponse<T>> {
  return await apiHandler.handleResponse<T>(response, context)
}

/**
 * Make API call with retry mechanism
 */
export async function apiCallWithRetry<T>(
  operation: () => Promise<Response>,
  options?: {
    maxRetries?: number
    baseDelay?: number
    maxDelay?: number
    retryableErrorTypes?: string[]
    context?: { operation?: string; user_id?: string; permission?: string; component?: string }
  }
): Promise<ApiResponse<T>> {
  return await apiHandler.withRetry<T>(operation, options)
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

// ============================================================================
// ADMIN FRONTEND API RESPONSE HANDLER COMPLETE
// ============================================================================
//
// 🎉 ADMIN FRONTEND API RESPONSE HANDLER COMPLETE!
//
// Created backend-only permission validation system for admin frontend:
// - Standardized response processing for all admin API calls
// - Structured error types matching backend permission authority
// - Intelligent retry mechanisms with exponential backoff  
// - Type-safe error handling with TypeScript guards
// - Context-aware error reporting and logging
// - User-friendly error messages with suggested actions
//
// Key Features:
// ✅ Handles all backend permission authority error types
// ✅ Retry logic for transient failures
// ✅ Rate limit respecting mechanisms
// ✅ Structured metadata and request tracking
// ✅ User-friendly error messages
// ✅ TypeScript type safety throughout
//
// The admin frontend now follows frontend's backend-only pattern! 🎯
// ============================================================================