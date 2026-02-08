/**
 * SHARED Permission Error Handler
 * Pure error handling - no client-side validation
 * Frontend displays exactly what backend tells us
 * 
 * Single source of truth for both admin-frontend and frontend apps
 */
import { logger } from './logger';
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

// Platform type for logging context
export type Platform = 'admin' | 'frontend'

export interface PermissionContext {
    feature?: string
    action?: string
}

/**
 * Custom error class for permission errors
 */
export class PermissionDeniedError extends Error {
    public readonly permissionError: BackendPermissionError

    /**
     * Create a new PermissionDeniedError
     * @param error - Backend permission error
     */
    constructor(error: BackendPermissionError) {
        super(error.message)
        this.name = 'PermissionDeniedError'
        this.permissionError = error
    }
}

// Global permission error listeners
const errorListeners = new Set<(event: PermissionErrorEvent) => void>()

/**
 * Subscribe to permission errors globally
 * @param callback - Function to call when permission error occurs
 * @returns Unsubscribe function
 */
export function onPermissionError(callback: (event: PermissionErrorEvent) => void): () => void {
    errorListeners.add(callback)
    return () => errorListeners.delete(callback)
}

/**
 * Emit permission error to all listeners
 * @param event - Permission error event
 */
function emitPermissionError(event: PermissionErrorEvent): void {
    errorListeners.forEach(listener => listener(event))
}

/**
 * Handle permission error from backend
 * Displays error and triggers upgrade flow if applicable
 * @param error - Backend permission error
 * @param context - Optional context about the error
 * @param platform - Platform context for logging (admin or frontend)
 */
export function handlePermissionError(
    error: BackendPermissionError,
    context?: PermissionContext,
    platform: Platform = 'frontend'
): void {
    const event: PermissionErrorEvent = {
        error,
        context: context ? { ...context, timestamp: Date.now() } : { timestamp: Date.now() }
    }

    // Emit to global listeners
    emitPermissionError(event)

    // Log for debugging
    const prefix = platform === 'admin' ? '🔒 Admin Permission Denied:' : '🔒 Permission Denied:'

    logger.error(prefix, {
        type: error.error_type,
        message: error.message,
        current: error.current_group,
        required: error.required_group,
        context
    })
}

/**
 * Extract permission error from fetch response
 * @param response - Fetch response object
 * @returns BackendPermissionError if response is 401/403, null otherwise
 */
export async function extractPermissionError(response: Response): Promise<BackendPermissionError | null> {
    if (response.status !== 403 && response.status !== 401) {
        return null;
    }

    try {
        const data = (await response.json()) as Partial<BackendPermissionError> & { error?: Partial<BackendPermissionError> };
        return normalizeBackendError(data, response.status);
    } catch {
        return {
            code: response.status,
            error_type: response.status === 401 ? 'AuthenticationRequired' : 'PermissionDenied',
            message: response.statusText || 'Permission denied'
        }
    }
}

function normalizeBackendError(
    data: Partial<BackendPermissionError> & { error?: Partial<BackendPermissionError> },
    status: number
): BackendPermissionError {
    const err = data.error ?? {};
    // Merge data and nested error object, with data taking precedence
    const merged = { ...err, ...data };

    const fallbackType = status === 401 ? 'AuthenticationRequired' : 'PermissionDenied';

    return {
        code: status,
        error_type: merged.error_type ?? fallbackType,
        message: merged.message ?? 'Permission denied',
        permission: merged.permission,
        reason: merged.reason,
        current_group: merged.current_group,
        required_group: merged.required_group,
        upgrade_url: merged.upgrade_url,
        benefits: merged.benefits,
        suggested_actions: merged.suggested_actions,
        expired_at: merged.expired_at,
        renewal_url: merged.renewal_url,
        current_usage: merged.current_usage,
        limit: merged.limit,
        reset_at: merged.reset_at,
        risk_level: merged.risk_level,
        contact_support: merged.contact_support
    }
}

/**
 * Wrap fetch with automatic permission error handling
 * @param url - URL to fetch
 * @param options - Fetch options
 * @param config - Context and platform config
 * @returns Fetch response
 * @throws PermissionDeniedError if response is 401/403
 */
export async function fetchWithPermissionHandling(
    url: string,
    options?: RequestInit,
    config: {
        context?: PermissionContext,
        platform?: Platform
    } = {}
): Promise<Response> {
    const { context, platform = 'frontend' } = config;
    const response = await fetch(url, options)

    const permError = await extractPermissionError(response)
    if (permError) {
        handlePermissionError(permError, context, platform)
        throw new PermissionDeniedError(permError)
    }

    return response
}

/**
 * Check if error is a permission error
 * @param error - Error to check
 * @returns True if error is PermissionDeniedError
 */
export function isPermissionError(error: unknown): error is PermissionDeniedError {
    return error instanceof PermissionDeniedError
}

/**
 * Get user-friendly message for error type
 * @param error - Backend permission error
 * @returns User-friendly error message
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
            return error.contact_support === true
                ? 'Security restriction. Please contact support'
                : error.message
        case 'AuthenticationRequired':
            return 'Please sign in to continue'
        default:
            return error.message || 'Permission denied'
    }
}
