/* eslint-disable max-lines-per-function */
/**
 * useNotificationBell Hook
 *
 * Centralized business logic for notification bell functionality.
 * Handles fetching, state management, and actions for both frontend and admin.
 *
 * Now uses server actions for all notification operations to eliminate proxy dependency.
 */

import { useCallback, useEffect, useRef, useState } from 'react'
import type { Notification as ApiNotification } from '../api/notifications'
import { MAX_DROPDOWN_NOTIFICATIONS } from '../components/notifications/constants'
import type { Notification } from '../components/notifications/types'
import { isNotificationPriority, isNotificationType } from '../components/notifications/types'
import type { UnifiedApiClient } from '../utils/api-client'
import { logger } from '../utils/logger'
import { useSSENotifications } from './use-sse-notifications'

interface BrowserNotificationAPI {
  showNotification: (type: string, title: string, message: string) => void
}

// Server action functions for notification operations
interface NotificationActions {
  getNotifications: (filters: { page: number; limit: number; status?: string }) => Promise<{ success: boolean; data?: { notifications: ApiNotification[]; unread_count?: number } }>
  markAsRead: (notificationId: string) => Promise<{ success: boolean; message: string }>
  markAsUnread: (notificationId: string) => Promise<{ success: boolean; message: string }>
  markAllAsRead: () => Promise<{ success: boolean; updated_count: number }>
  deleteNotification?: (notificationId: string) => Promise<{ success: boolean; message: string }>
  clearAll?: () => Promise<{ success: boolean; deleted_count: number }>
}

interface UseNotificationBellOptions {
  actions: NotificationActions
  apiClient?: UnifiedApiClient // Only needed for SSE
  walletAddress?: string
  isAuthenticated: boolean
  enableSSE?: boolean
  browserNotifications?: BrowserNotificationAPI
  onNotificationReceived?: (notification: Notification) => void
  refreshSession?: () => Promise<boolean>
}

interface UseNotificationBellReturn {
  notifications: Notification[]
  count: number
  loading: boolean
  error: string | null
  isSSEConnected: boolean
  fetchNotifications: () => Promise<void>
  markAsRead: (notificationId: string) => Promise<void>
  markAsUnread: (notificationId: string) => Promise<void>
  markAllAsRead: () => Promise<void>
  reconnectSSE: () => void
}

export function useNotificationBell(
  options: UseNotificationBellOptions
): UseNotificationBellReturn {
  const {
    actions,
    apiClient,
    walletAddress,
    isAuthenticated,
    enableSSE = true,
    browserNotifications,
    onNotificationReceived,
  } = options

  // Store options in a ref so mutable callbacks/objects don't cause infinite re-renders
  const optionsRef = useRef(options)
  optionsRef.current = options


  const [notifications, setNotifications] = useState<Notification[]>([])
  const [count, setCount] = useState(0)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // SSE real-time notifications (only if apiClient is provided)
  const sseEnabled = enableSSE && Boolean(apiClient);
  const { isConnected: isSSEConnected, reconnect: reconnectSSE } = useSSENotifications({
    apiClient,
    walletAddress,
    refreshSession: options.refreshSession,
    autoConnect: sseEnabled && isAuthenticated && Boolean(walletAddress),
    onNotification: useCallback(
      (sseNotif: { id: string; title: string; message: string; notification_type: string; priority: string; timestamp: string; expires_at?: string; wallet_address: string; data?: Record<string, unknown> }) => {
        // Map SSE notification to Notification format
        const newNotification: Notification = {
          id: sseNotif.id,
          title: sseNotif.title,
          message: sseNotif.message,
          type: isNotificationType(sseNotif.notification_type) ? sseNotif.notification_type : 'system',
          priority: isNotificationPriority(sseNotif.priority) ? sseNotif.priority : 'normal',
          timestamp: sseNotif.timestamp,
          expires_at: sseNotif.expires_at,
          wallet_address: sseNotif.wallet_address,
          data: sseNotif.data,
          read: false,
        }

        setNotifications((prev) => {
          const newArray = [newNotification, ...prev]
          // Keep only last 50 notifications to prevent memory issues
          return newArray.slice(0, 50)
        })
        setCount((prev) => Math.min(prev + 1, 50)) // Cap count at 50

        // Show browser notification for high priority
        if (
          optionsRef.current.browserNotifications &&
          (sseNotif.priority === 'high' || sseNotif.priority === 'critical')
        ) {
          const notifType =
            sseNotif.notification_type === 'security'
              ? 'security'
              : sseNotif.notification_type === 'permission'
                ? 'permissions'
                : sseNotif.notification_type === 'wallet'
                  ? 'analytics'
                  : 'system'

          optionsRef.current.browserNotifications.showNotification(
            notifType,
            sseNotif.title,
            sseNotif.message
          )
        }

        // Call custom callback
        optionsRef.current.onNotificationReceived?.(newNotification)
      },
      [] // Options accessed via ref
    ),
    onError: useCallback((sseError: string) => {
      logger.warn('SSE connection error:', sseError)
      setError('Connection lost. Reconnecting...')
    }, []),
    onConnect: useCallback(() => {
      logger.info('✅ SSE connected for wallet:', { walletAddress })
      setError(null)
    }, [walletAddress]),
  })

  // Fetch notifications from API using server actions
  const fetchNotifications = useCallback(async () => {
    if (isAuthenticated === false || walletAddress === undefined || walletAddress === '') {
      setNotifications([])
      setCount(0)
      setLoading(false)
      setError(null)
      return
    }

    try {
      setLoading(true)
      setError(null)

      const data = await optionsRef.current.actions.getNotifications({
        page: 1,
        limit: MAX_DROPDOWN_NOTIFICATIONS,
        status: 'all',
      })

      if (data.success && data.data?.notifications) {
        const mappedNotifications: Notification[] = data.data.notifications.map((n: ApiNotification) => ({
          id: n.id,
          title: n.title,
          message: n.message,
          type: isNotificationType(n.notification_type) ? n.notification_type : 'system',
          priority: isNotificationPriority(n.priority) ? n.priority : 'normal',
          timestamp: n.timestamp,
          expires_at: n.expires_at,
          action_url: n.action_url,
          image_url: n.image_url,
          wallet_address: n.wallet_address,
          data: n.data,
          read: Boolean(n.read_at),
        }))

        setNotifications(mappedNotifications)
        setCount(data.data.unread_count ?? 0)
      } else {
        setNotifications([])
        setCount(0)
      }
    } catch (err) {
      // Silently fail if not authenticated - this is expected
      const apiError = err as { status?: number }
      if (apiError.status !== 401) {
        logger.warn('Failed to fetch notifications:', err)
        setError('Failed to load notifications')
      }
      setNotifications([])
      setCount(0)
    } finally {
      setLoading(false)
    }
  }, [isAuthenticated, walletAddress]) // Options accessed via ref


  // Mark notification as read using server action
  const markAsRead = useCallback(
    async (notificationId: string) => {
      try {
        const result = await optionsRef.current.actions.markAsRead(notificationId)

        if (result.success) {
          // Update local state only on success
          setNotifications((prev) =>
            prev.map((n) => (n.id === notificationId ? { ...n, read: true } : n))
          )
          setCount((prev) => Math.max(0, prev - 1))
        }
      } catch (err) {
        logger.error('Failed to mark notification as read:', err)
      }
    },
    [] // Options accessed via ref
  )

  // Mark notification as unread using server action
  const markAsUnread = useCallback(
    async (notificationId: string) => {
      try {
        const result = await optionsRef.current.actions.markAsUnread(notificationId)
        if (result.success) {
          setNotifications((prev) =>
            prev.map((n) => (n.id === notificationId ? { ...n, read: false } : n))
          )
          setCount((prev) => prev + 1)
        }
      } catch (err) {
        logger.error('Failed to mark notification as unread:', err)
      }
    },
    [] // Options accessed via ref
  )

  // Mark all notifications as read using server action
  const markAllAsRead = useCallback(async () => {
    try {
      await optionsRef.current.actions.markAllAsRead()

      // Update local state
      setNotifications((prev) => prev.map((n) => ({ ...n, read: true })))
      setCount(0)
    } catch (err) {
      logger.error('Failed to mark all notifications as read:', err)
    }
  }, []) // Options accessed via ref

  // Fetch notifications when authentication state changes
  useEffect(() => {
    void fetchNotifications()
  }, [fetchNotifications])

  return {
    notifications,
    count,
    loading,
    error,
    isSSEConnected,
    fetchNotifications,
    markAsRead,
    markAsUnread,
    markAllAsRead,
    reconnectSSE,
  }
}
