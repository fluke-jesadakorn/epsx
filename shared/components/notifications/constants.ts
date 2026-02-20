/**
 * Shared Notification Constants
 *
 * Centralized configuration values for the notification system.
 * Replaces magic numbers throughout the codebase.
 */

// ============================================================================
// NOTIFICATION LIMITS
// ============================================================================

/**
 * Maximum number of notifications to keep in memory
 */
export const MAX_IN_MEMORY_NOTIFICATIONS = 100

/**
 * Maximum number of notifications to fetch per API call
 */
export const MAX_FETCH_LIMIT = 100

/**
 * Default number of notifications to fetch
 */
export const DEFAULT_FETCH_LIMIT = 20

/**
 * Maximum number of notifications to show in dropdown
 */
export const MAX_DROPDOWN_NOTIFICATIONS = 5

// ============================================================================
// SSE CONFIGURATION
// ============================================================================

/**
 * SSE keep-alive interval in milliseconds (15 seconds)
 */
export const SSE_KEEP_ALIVE_INTERVAL = 15000

/**
 * SSE connection timeout in milliseconds (2 minutes)
 */
export const SSE_CONNECTION_TIMEOUT = 120000

/**
 * Maximum SSE reconnection attempts
 */
export const MAX_RECONNECT_ATTEMPTS = 10

/**
 * Base interval between reconnection attempts in milliseconds
 */
export const RECONNECT_BASE_INTERVAL = 5000

/**
 * Maximum interval between reconnection attempts in milliseconds
 */
export const RECONNECT_MAX_INTERVAL = 30000

// ============================================================================
// BACKEND CONFIGURATION
// ============================================================================

/**
 * Maximum offline notifications to fetch from database (backend)
 */
export const MAX_OFFLINE_QUEUE_SIZE = 100

/**
 * Number of days to retain read notifications
 */
export const READ_NOTIFICATION_RETENTION_DAYS = 90

/**
 * Number of days to retain soft-deleted notifications
 */
export const SOFT_DELETE_RETENTION_DAYS = 7

/**
 * Number of days to keep notification history
 */
export const NOTIFICATION_HISTORY_DAYS = 30

// ============================================================================
// UI CONFIGURATION
// ============================================================================

/**
 * Debounce delay for mark-as-read actions in milliseconds
 */
export const MARK_AS_READ_DEBOUNCE = 500

/**
 * Auto-hide notification dropdown after this many milliseconds
 */
export const AUTO_HIDE_DROPDOWN_DELAY = 300

/**
 * Notification badge pulse animation duration in milliseconds
 */
export const BADGE_PULSE_DURATION = 1000

// ============================================================================
// NOTIFICATION TYPES
// ============================================================================

/**
 * All available notification types
 */
export const NOTIFICATION_TYPES = [
  'system',
  'security',
  'permission',
  'wallet_management',
  'wallet',
  'payment',
  'general',
  'announcement',
  'advertisement',
  'chat',
] as const

/**
 * All available notification priorities
 */
export const NOTIFICATION_PRIORITIES = ['low', 'normal', 'high', 'critical', 'urgent'] as const

// ============================================================================
// BROWSER NOTIFICATION TYPES
// ============================================================================

/**
 * Minimum priority level for browser notifications
 */
export const BROWSER_NOTIFICATION_MIN_PRIORITY = 'high'

/**
 * Notification types that should trigger browser notifications
 */
export const BROWSER_NOTIFICATION_TYPES = ['security', 'permission', 'system', 'critical']

// ============================================================================
// API ENDPOINTS
// ============================================================================

/**
 * SSE stream endpoint path
 */
export const SSE_STREAM_ENDPOINT = '/api/notifications/stream'

/**
 * Notifications API endpoint path
 */
export const NOTIFICATIONS_API_ENDPOINT = '/api/notifications'

/**
 * Admin notifications API endpoint path
 */
export const ADMIN_NOTIFICATIONS_API_ENDPOINT = '/api/admin/notifications'
