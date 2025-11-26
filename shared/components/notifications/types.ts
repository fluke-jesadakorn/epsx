/**
 * Shared Notification Types
 *
 * Centralized type definitions for notifications across frontend and admin applications.
 * Ensures type consistency and reduces duplication.
 */

// ============================================================================
// CORE NOTIFICATION TYPES
// ============================================================================

export type NotificationType =
  | 'system'
  | 'security'
  | 'permission'
  | 'wallet_management'
  | 'user_management'
  | 'wallet'
  | 'payment'
  | 'general'

export type NotificationPriority = 'low' | 'normal' | 'high' | 'critical' | 'urgent'

export type NotificationStatus = 'read' | 'unread' | 'all'

// ============================================================================
// NOTIFICATION INTERFACE
// ============================================================================

export interface Notification {
  id: string
  title: string
  message: string
  type: NotificationType
  priority: NotificationPriority
  timestamp: string
  expires_at?: string
  read_at?: string
  clicked_at?: string
  delivered_at?: string
  action_url?: string
  image_url?: string
  wallet_address?: string
  data?: Record<string, any>
  read: boolean
}

// ============================================================================
// SSE NOTIFICATION
// ============================================================================

export interface SSENotification {
  id: string
  wallet_address: string
  notification_type: string
  title: string
  message: string
  data?: Record<string, any>
  priority: string
  timestamp: string
  expires_at?: string
}

// ============================================================================
// NOTIFICATION FILTERS
// ============================================================================

export interface NotificationFilters {
  page?: number
  limit?: number
  type?: NotificationType
  priority?: NotificationPriority
  status?: NotificationStatus
  start_date?: string
  end_date?: string
  wallet_address?: string
}

// ============================================================================
// NOTIFICATION STATE
// ============================================================================

export interface NotificationState {
  notifications: Notification[]
  count: number
  loading: boolean
  error: string | null
  isConnected: boolean
}

// ============================================================================
// TYPE GUARDS
// ============================================================================

export function isNotificationType(value: string): value is NotificationType {
  return [
    'system',
    'security',
    'permission',
    'wallet_management',
    'user_management',
    'wallet',
    'payment',
    'general',
  ].includes(value)
}

export function isNotificationPriority(value: string): value is NotificationPriority {
  return ['low', 'normal', 'high', 'critical', 'urgent'].includes(value)
}
