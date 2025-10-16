/**
 * useNotificationBell Hook
 *
 * Centralized business logic for notification bell functionality.
 * Handles fetching, state management, and actions for both frontend and admin.
 */

import { useState, useEffect, useCallback } from 'react'
import type { UnifiedApiClient } from '../utils/api-client'
import { createNotificationsClient } from '../api/notifications'
import { useSSENotifications } from './useSSENotifications'
import type { Notification, NotificationFilters } from '../components/notifications/types'
import { MAX_DROPDOWN_NOTIFICATIONS } from '../components/notifications/constants'

interface BrowserNotificationAPI {
  showNotification: (type: string, title: string, message: string) => void
}

interface UseNotificationBellOptions {
  apiClient: UnifiedApiClient
  walletAddress?: string
  isAuthenticated: boolean
  enableSSE?: boolean
  browserNotifications?: BrowserNotificationAPI
  onNotificationReceived?: (notification: Notification) => void
}

interface UseNotificationBellReturn {
  notifications: Notification[]
  count: number
  loading: boolean
  error: string | null
  isSSEConnected: boolean
  fetchNotifications: () => Promise<void>
  markAsRead: (notificationId: string) => Promise<void>
  markAllAsRead: () => Promise<void>
  deleteNotification: (notificationId: string) => Promise<void>
  reconnectSSE: () => void
}

export function useNotificationBell(
  options: UseNotificationBellOptions
): UseNotificationBellReturn {
  const {
    apiClient,
    walletAddress,
    isAuthenticated,
    enableSSE = true,
    browserNotifications,
    onNotificationReceived,
  } = options

  const [notifications, setNotifications] = useState<Notification[]>([])
  const [count, setCount] = useState(0)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // SSE real-time notifications
  const { isConnected: isSSEConnected, reconnect: reconnectSSE } = useSSENotifications({
    apiClient,
    walletAddress,
    autoConnect: false, // We'll control it manually
    onNotification: useCallback(
      (sseNotif) => {
        // Map SSE notification to Notification format
        const newNotification: Notification = {
          id: sseNotif.id,
          title: sseNotif.title,
          message: sseNotif.message,
          type: sseNotif.notification_type as any,
          priority: sseNotif.priority as any,
          timestamp: sseNotif.timestamp,
          expires_at: sseNotif.expires_at,
          wallet_address: sseNotif.wallet_address,
          data: sseNotif.data,
          read: false,
        }

        // Add notification and deduplicate by ID
        setNotifications((prev) => {
          const exists = prev.some((n) => n.id === newNotification.id)
          if (exists) return prev
          return [newNotification, ...prev]
        })
        setCount((prev) => prev + 1)

        // Show browser notification for high priority
        if (
          browserNotifications &&
          (sseNotif.priority === 'high' || sseNotif.priority === 'critical')
        ) {
          const notifType =
            sseNotif.notification_type === 'security'
              ? 'security'
              : sseNotif.notification_type === 'permission'
              ? 'permissions'
              : sseNotif.notification_type === 'wallet'
              ? 'trading'
              : 'system'

          browserNotifications.showNotification(
            notifType as any,
            sseNotif.title,
            sseNotif.message
          )
        }

        // Call custom callback
        onNotificationReceived?.(newNotification)
      },
      [browserNotifications, onNotificationReceived]
    ),
    onError: useCallback((error) => {
      console.warn('SSE connection error:', error)
      setError('Connection lost. Reconnecting...')
    }, []),
    onConnect: useCallback(() => {
      console.log('✅ SSE connected for wallet:', walletAddress)
      setError(null)
    }, [walletAddress]),
  })

  // Fetch notifications from API
  const fetchNotifications = useCallback(async () => {
    if (!isAuthenticated || !walletAddress) {
      setNotifications([])
      setCount(0)
      setLoading(false)
      return
    }

    try {
      setLoading(true)
      setError(null)

      const client = createNotificationsClient(apiClient)
      const data = await client.getNotifications({
        page: 1,
        limit: MAX_DROPDOWN_NOTIFICATIONS,
        // Show all notifications (both read and unread)
      })

      const mappedNotifications: Notification[] = data.data.notifications.map((n) => ({
        id: n.id,
        title: n.title,
        message: n.message,
        type: n.notification_type as any,
        priority: n.priority as any,
        timestamp: n.timestamp,
        expires_at: n.expires_at,
        action_url: n.action_url,
        wallet_address: n.wallet_address,
        data: n.data,
        read: !!n.read_at,
      }))

      // Deduplicate notifications by ID
      setNotifications((prev) => {
        const newIds = new Set(mappedNotifications.map((n) => n.id))
        const filtered = prev.filter((n) => !newIds.has(n.id))
        return [...mappedNotifications, ...filtered]
      })
      setCount(data.data.unread_count)
    } catch (err) {
      // Silently fail if not authenticated - this is expected
      const apiError = err as any
      if (apiError?.status !== 401) {
        console.warn('Failed to fetch notifications:', err)
        setError('Failed to load notifications')
      }
      setNotifications([])
      setCount(0)
    } finally {
      setLoading(false)
    }
  }, [apiClient, isAuthenticated, walletAddress])

  // Mark notification as read
  const markAsRead = useCallback(
    async (notificationId: string) => {
      if (!isAuthenticated || !walletAddress) {
        console.warn('Cannot mark notification as read: not authenticated')
        return
      }

      try {
        const client = createNotificationsClient(apiClient)
        await client.markAsRead(notificationId)

        // Update notification as read in local state
        setNotifications((prev) =>
          prev.map((n) => (n.id === notificationId ? { ...n, read: true } : n))
        )
        setCount((prev) => Math.max(0, prev - 1))
      } catch (err) {
        console.error('Failed to mark notification as read:', err)
      }
    },
    [apiClient, isAuthenticated, walletAddress]
  )

  // Mark all notifications as read
  const markAllAsRead = useCallback(async () => {
    if (!isAuthenticated || !walletAddress) {
      console.warn('Cannot mark all notifications as read: not authenticated')
      return
    }

    try {
      const client = createNotificationsClient(apiClient)
      await client.markAllAsRead()

      // Mark all notifications as read in local state
      setNotifications((prev) => prev.map((n) => ({ ...n, read: true })))
      setCount(0)
    } catch (err) {
      console.error('Failed to mark all notifications as read:', err)
    }
  }, [apiClient, isAuthenticated, walletAddress])

  // Delete notification
  const deleteNotification = useCallback(
    async (notificationId: string) => {
      if (!isAuthenticated || !walletAddress) {
        console.warn('Cannot delete notification: not authenticated')
        return
      }

      try {
        const client = createNotificationsClient(apiClient)
        await client.deleteNotification(notificationId)

        // Remove from local state
        setNotifications((prev) => prev.filter((n) => n.id !== notificationId))
        setCount((prev) => Math.max(0, prev - 1))
      } catch (err) {
        console.error('Failed to delete notification:', err)
      }
    },
    [apiClient, isAuthenticated, walletAddress]
  )

  // Fetch notifications when authentication state changes
  useEffect(() => {
    fetchNotifications()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isAuthenticated, walletAddress])

  // Connect SSE when authenticated and enabled
  useEffect(() => {
    if (enableSSE && isAuthenticated && walletAddress && !isSSEConnected) {
      console.log('🔌 Connecting SSE for authenticated user:', walletAddress)
      reconnectSSE()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [enableSSE, isAuthenticated, walletAddress, isSSEConnected])

  return {
    notifications,
    count,
    loading,
    error,
    isSSEConnected,
    fetchNotifications,
    markAsRead,
    markAllAsRead,
    deleteNotification,
    reconnectSSE,
  }
}
