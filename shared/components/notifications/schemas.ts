/**
 * Notification Zod Schemas
 *
 * Runtime validation schemas for notifications using Zod.
 * Provides type-safe validation for API responses and user input.
 */

import { z } from 'zod'
import {
  MAX_FETCH_LIMIT
} from './constants'

// ============================================================================
// BASE SCHEMAS
// ============================================================================

export const NotificationTypeSchema = z.enum([
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
])

export const NotificationPrioritySchema = z.enum([
  'low',
  'normal',
  'high',
  'critical',
  'urgent',
])

export const NotificationStatusSchema = z.enum(['read', 'unread', 'all'])

// ============================================================================
// NOTIFICATION SCHEMA
// ============================================================================

export const NotificationSchema = z.object({
  id: z.string().uuid({ message: 'Invalid notification ID format' }),
  title: z.string().min(1, 'Title is required').max(200, 'Title too long'),
  message: z.string().min(1, 'Message is required').max(1000, 'Message too long'),
  type: NotificationTypeSchema,
  priority: NotificationPrioritySchema,
  timestamp: z.string().datetime({ message: 'Invalid timestamp format' }),
  expires_at: z.string().datetime().optional(),
  read_at: z.string().datetime().optional(),
  clicked_at: z.string().datetime().optional(),
  delivered_at: z.string().datetime().optional(),
  action_url: z.string().url({ message: 'Invalid action URL' }).optional(),
  image_url: z.string().url({ message: 'Invalid image URL' }).optional(),
  wallet_address: z.string().optional(),
  data: z.record(z.string(), z.any()).optional(),
  read: z.boolean(),
})

export type NotificationValidated = z.infer<typeof NotificationSchema>

// ============================================================================
// SSE NOTIFICATION SCHEMA
// ============================================================================

export const SSENotificationSchema = z.object({
  id: z.string().uuid(),
  wallet_address: z.string(),
  notification_type: z.string(),
  title: z.string().min(1).max(200),
  message: z.string().min(1).max(1000),
  data: z.record(z.string(), z.any()).nullish(),
  priority: z.string(),
  timestamp: z.string().datetime(),
  expires_at: z.string().datetime().nullish(),
})

export type SSENotificationValidated = z.infer<typeof SSENotificationSchema>

// ============================================================================
// FILTER SCHEMAS
// ============================================================================

export const NotificationFiltersSchema = z.object({
  page: z.number().multipleOf(1).positive().optional(),
  limit: z
    .number()
    .multipleOf(1)
    .positive()
    .max(MAX_FETCH_LIMIT, `Limit cannot exceed ${MAX_FETCH_LIMIT}`)
    .optional(),
  type: NotificationTypeSchema.optional(),
  priority: NotificationPrioritySchema.optional(),
  status: NotificationStatusSchema.optional(),
  start_date: z.string().datetime().optional(),
  end_date: z.string().datetime().optional(),
  wallet_address: z.string().optional(),
})

export type NotificationFiltersValidated = z.infer<typeof NotificationFiltersSchema>

// ============================================================================
// RESPONSE SCHEMAS
// ============================================================================

export const NotificationsResponseSchema = z.object({
  success: z.boolean(),
  data: z.object({
    notifications: z.array(NotificationSchema),
    total_count: z.number().multipleOf(1).nonnegative(),
    unread_count: z.number().multipleOf(1).nonnegative(),
    page: z.number().multipleOf(1).positive(),
    limit: z.number().multipleOf(1).positive(),
    total_pages: z.number().multipleOf(1).nonnegative(),
  }),
  api_version: z.string().optional(),
  access_level: z.string().optional(),
})

export type NotificationsResponseValidated = z.infer<typeof NotificationsResponseSchema>

// ============================================================================
// ADMIN SCHEMAS
// ============================================================================

export const SendNotificationRequestSchema = z.object({
  recipient_wallet_address: z.string().optional(),
  recipient_group: z.string().optional(),
  broadcast: z.boolean().optional(),
  notification_type: NotificationTypeSchema,
  priority: NotificationPrioritySchema,
  title: z.string().min(1, 'Title is required').max(200, 'Title too long'),
  message: z.string().min(1, 'Message is required').max(1000, 'Message too long'),
  data: z.record(z.string(), z.any()).optional(),
  action_url: z.string().url({ message: 'Invalid action URL' }).optional(),
  image_url: z.string().url({ message: 'Invalid image URL' }).optional(),
  expires_at: z.string().datetime().optional(),
  schedule_at: z.string().datetime().optional(),
}).refine(
  (data) => {
    // Must specify either recipient_wallet_address, recipient_group, or broadcast
    return Boolean(data.recipient_wallet_address ?? data.recipient_group ?? data.broadcast)
  },
  {
    message: 'Must specify recipient_wallet_address, recipient_group, or broadcast',
    path: ['recipient_wallet_address'],
  }
)

export type SendNotificationRequestValidated = z.infer<typeof SendNotificationRequestSchema>

export const SendNotificationResponseSchema = z.object({
  success: z.boolean(),
  data: z.object({
    notification_id: z.string().uuid(),
    recipients_count: z.number().multipleOf(1).nonnegative(),
    scheduled: z.boolean(),
    delivery_status: z.enum(['sent', 'scheduled', 'failed']),
  }),
  message: z.string(),
  api_version: z.string().optional(),
})

export type SendNotificationResponseValidated = z.infer<typeof SendNotificationResponseSchema>

// ============================================================================
// PREFERENCE SCHEMAS
// ============================================================================

export const NotificationPreferencesSchema = z.object({
  email_enabled: z.boolean(),
  push_enabled: z.boolean(),
  sms_enabled: z.boolean().optional(),
  types: z.object({
    system: z.boolean(),
    security: z.boolean(),
    permission: z.boolean(),
    wallet_management: z.boolean(),
    wallet: z.boolean(),
    payment: z.boolean(),
    general: z.boolean(),
    announcement: z.boolean(),
    advertisement: z.boolean(),
    chat: z.boolean(),
  }),
  priority_filter: NotificationPrioritySchema,
  quiet_hours: z
    .object({
      enabled: z.boolean(),
      start_time: z.string().regex(/^([0-1][0-9]|2[0-3]):[0-5][0-9]$/, 'Invalid time format (HH:MM)'),
      end_time: z.string().regex(/^([0-1][0-9]|2[0-3]):[0-5][0-9]$/, 'Invalid time format (HH:MM)'),
      timezone: z.string(),
    })
    .optional(),
})

export type NotificationPreferencesValidated = z.infer<typeof NotificationPreferencesSchema>

// ============================================================================
// VALIDATION HELPERS
// ============================================================================

/**
 * Safely parse and validate notification data
 */
export function validateNotification(data: unknown): NotificationValidated | null {
  try {
    return NotificationSchema.parse(data)
  } catch (_error) {
    // console.error('Notification validation failed:', _error)
    return null
  }
}

/**
 * Safely parse and validate SSE notification
 */
export function validateSSENotification(data: unknown): SSENotificationValidated | null {
  try {
    return SSENotificationSchema.parse(data)
  } catch (_error) {
    // console.error('SSE notification validation failed:', _error)
    return null
  }
}

/**
 * Validate notification filters with detailed error reporting
 */
export function validateNotificationFilters(
  data: unknown
): { success: true; data: NotificationFiltersValidated } | { success: false; errors: z.ZodError } {
  const result = NotificationFiltersSchema.safeParse(data)

  if (result.success) {
    return { success: true, data: result.data }
  }

  return { success: false, errors: result.error }
}

/**
 * Validate send notification request
 */
export function validateSendNotificationRequest(
  data: unknown
):
  | { success: true; data: SendNotificationRequestValidated }
  | { success: false; errors: z.ZodError } {
  const result = SendNotificationRequestSchema.safeParse(data)

  if (result.success) {
    return { success: true, data: result.data }
  }

  return { success: false, errors: result.error }
}

/**
 * Batch validate notifications array
 */
export function validateNotifications(data: unknown[]): NotificationValidated[] {
  return data
    .map(validateNotification)
    .filter((n): n is NotificationValidated => n !== null)
}
