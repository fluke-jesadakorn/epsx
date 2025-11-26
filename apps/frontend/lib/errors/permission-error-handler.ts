/**
 * Backend Permission Error Handler
 * Pure error handling - no client-side validation
 * Frontend displays exactly what backend tells us
 */

// Backend permission error structure (from apps/backend/src/web/errors/permission_errors.rs)
export interface BackendPermissionError {
  code: number
  error_type: 'PermissionDenied' | 'InsufficientGroup' | 'UsageLimitExceeded' | 'PermissionExpired' | 'SecurityRestriction' | 'AuthenticationRequired'
  message: string
  permission?: string
  reason?: string
  current_group?: string
  required_group?: string
  upgrade_url?: string
  benefits?: string[]
  suggested_actions?: string[]
  expired_at?: string
  renewal_url?: string
  current_usage?: number
  limit?: number
  reset_at?: string
  risk_level?: 'low' | 'medium' | 'high' | 'critical'
  contact_support?: boolean
}

// Permission error event for global handling
export interface PermissionErrorEvent {
  error: BackendPermissionError
  context?: {
    feature?: string
    action?: string
    timestamp: number
  }
}

// Global permission error listeners
const errorListeners = new Set<(event: PermissionErrorEvent) => void>()

/**
 * Subscribe to permission errors globally
 * @param callback
 */
export function onPermissionError(callback: (event: PermissionErrorEvent) => void): () => void {
  errorListeners.add(callback)
  return () => errorListeners.delete(callback)
}

/**
 * Emit permission error to all listeners
 * @param event
 */
function emitPermissionError(event: PermissionErrorEvent): void {
  errorListeners.forEach(listener => listener(event))
}

/**
 * Handle permission error from backend
 * Displays error and triggers upgrade flow if applicable
 * @param error
 * @param context
 * @param context.feature
 * @param context.action
 */
export function handlePermissionError(
  error: BackendPermissionError,
  context?: { feature?: string; action?: string }
): void {
  const event: PermissionErrorEvent = {
    error,
    context: context ? { ...context, timestamp: Date.now() } : { timestamp: Date.now() }
  }

  // Emit to global listeners
  emitPermissionError(event)

  // Log for debugging
   
  console.error('🔒 Permission Denied:', {
    type: error.error_type,
    message: error.message,
    current: error.current_group,
    required: error.required_group,
    context
  })
}

/**
 * Extract permission error from fetch response
 * @param response
 */
export async function extractPermissionError(response: Response): Promise<BackendPermissionError | null> {
  if (response.status === 403 || response.status === 401) {
    try {
      const data = await response.json()
      return {
        code: response.status,
        error_type: data.error_type ?? (response.status === 401 ? 'AuthenticationRequired' : 'PermissionDenied'),
        message: data.message ?? data.error?.message ?? 'Permission denied',
        permission: data.permission ?? data.error?.permission,
        reason: data.reason ?? data.error?.reason,
        current_group: data.current_group ?? data.error?.current_group,
        required_group: data.required_group ?? data.error?.required_group,
        upgrade_url: data.upgrade_url ?? data.error?.upgrade_url,
        benefits: data.benefits ?? data.error?.benefits,
        suggested_actions: data.suggested_actions ?? data.error?.suggested_actions,
        expired_at: data.expired_at ?? data.error?.expired_at,
        renewal_url: data.renewal_url ?? data.error?.renewal_url,
        current_usage: data.current_usage ?? data.error?.current_usage,
        limit: data.limit ?? data.error?.limit,
        reset_at: data.reset_at ?? data.error?.reset_at,
        risk_level: data.risk_level ?? data.error?.risk_level,
        contact_support: data.contact_support ?? data.error?.contact_support
      }
    } catch {
      return {
        code: response.status,
        error_type: response.status === 401 ? 'AuthenticationRequired' : 'PermissionDenied',
        message: response.statusText ?? 'Permission denied'
      }
    }
  }
  return null
}

/**
 * Wrap fetch with automatic permission error handling
 * @param url
 * @param options
 * @param context
 * @param context.feature
 * @param context.action
 */
export async function fetchWithPermissionHandling(
  url: string,
  options?: RequestInit,
  context?: { feature?: string; action?: string }
): Promise<Response> {
  const response = await fetch(url, options)

  const permError = await extractPermissionError(response)
  if (permError) {
    handlePermissionError(permError, context)
    throw new PermissionDeniedError(permError)
  }

  return response
}

/**
 * Custom error class for permission errors
 */
export class PermissionDeniedError extends Error {
  public readonly permissionError: BackendPermissionError

  /**
   *
   * @param error
   */
  constructor(error: BackendPermissionError) {
    super(error.message)
    this.name = 'PermissionDeniedError'
    this.permissionError = error
  }
}

/**
 * Check if error is a permission error
 * @param error
 */
export function isPermissionError(error: unknown): error is PermissionDeniedError {
  return error instanceof PermissionDeniedError
}

/**
 * Get user-friendly message for error type
 * @param error
 */
export function getErrorMessage(error: BackendPermissionError): string {
  switch (error.error_type) {
    case 'InsufficientGroup':
      return `Upgrade to ${error.required_group} to access this feature`
    case 'PermissionExpired':
      return 'Your permission has expired. Renew to continue'
    case 'UsageLimitExceeded':
      return `Usage limit reached (${error.current_usage}/${error.limit}). Upgrade for more`
    case 'SecurityRestriction':
      return error.contact_support
        ? 'Security restriction. Please contact support'
        : error.message
    case 'AuthenticationRequired':
      return 'Please sign in to continue'
    default:
      return error.message ?? 'Permission denied'
  }
}
