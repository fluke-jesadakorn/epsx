/**
 * useNotificationBell Hook
 *
 * Centralized business logic for notification bell functionality.
 * Handles fetching, state management, and actions for both frontend and admin.
 *
 * Now uses server actions for all notification operations to eliminate proxy dependency.
 */

import type { Dispatch, MutableRefObject, SetStateAction } from 'react'
import { useCallback, useEffect, useRef, useState } from 'react'
import type { Notification as ApiNotification, SSENotification } from '../api/notifications'
import { MAX_DROPDOWN_NOTIFICATIONS } from '../components/notifications/constants'
import type { Notification } from '../components/notifications/types'
import { isNotificationPriority, isNotificationType } from '../components/notifications/types'
import type { UnifiedApiClient } from '../utils/api-client'
import { logger } from '../utils/logger'
import { useSSENotifications } from './use-sse-notifications'

function getBrowserNotifType(notifType: string): string {
  if (notifType === 'security') { return 'security'; }
  if (notifType === 'permission') { return 'permissions'; }
  if (notifType === 'wallet') { return 'analytics'; }
  return 'system';
}

function mapSSEToNotification(sseNotif: SSENotification): Notification {
  return {
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
}

function mapApiToNotification(n: ApiNotification): Notification {
  return {
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
  }
}

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

interface SSESetupCtx {
  options: UseNotificationBellOptions
  optionsRef: MutableRefObject<UseNotificationBellOptions>
  setNotifications: Dispatch<SetStateAction<Notification[]>>
  setCount: Dispatch<SetStateAction<number>>
  setError: Dispatch<SetStateAction<string | null>>
}

function useSSESetup(ctx: SSESetupCtx): { isSSEConnected: boolean; reconnectSSE: () => void } {
  const { options, optionsRef, setNotifications, setCount, setError } = ctx
  const { apiClient, walletAddress, isAuthenticated, enableSSE = true } = options
  const sseEnabled = enableSSE && Boolean(apiClient)

  const { isConnected: isSSEConnected, reconnect: reconnectSSE } = useSSENotifications({
    apiClient,
    walletAddress,
    refreshSession: optionsRef.current.refreshSession,
    autoConnect: sseEnabled && isAuthenticated && Boolean(walletAddress),
    onNotification: useCallback((sseNotif: SSENotification) => {
      const newNotif = mapSSEToNotification(sseNotif)
      setNotifications((prev) => {
        const deduped = prev.filter((n) => n.id !== newNotif.id)
        return [newNotif, ...deduped].slice(0, 50)
      })
      setCount((prev) => Math.min(prev + 1, 50))
      const bn = optionsRef.current.browserNotifications
      if (bn !== undefined && (sseNotif.priority === 'high' || sseNotif.priority === 'critical')) {
        bn.showNotification(getBrowserNotifType(sseNotif.notification_type), sseNotif.title, sseNotif.message)
      }
      optionsRef.current.onNotificationReceived?.(newNotif)
    }, [setNotifications, setCount, optionsRef]),
    onError: useCallback((sseError: string) => {
      logger.warn('SSE connection error:', sseError)
      setError('Connection lost. Reconnecting...')
    }, [setError]),
    onConnect: useCallback(() => {
      setError(null)
    }, [setError]),
  })

  return { isSSEConnected, reconnectSSE }
}

export function useNotificationBell(
  options: UseNotificationBellOptions
): UseNotificationBellReturn {
  const { isAuthenticated, walletAddress } = options
  const optionsRef = useRef(options)
  optionsRef.current = options

  const [notifications, setNotifications] = useState<Notification[]>([])
  const [count, setCount] = useState(0)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const { isSSEConnected, reconnectSSE } = useSSESetup({
    options, optionsRef, setNotifications, setCount, setError,
  })

  const fetchNotifications = useCallback(async () => {
    if (isAuthenticated === false || walletAddress === undefined || walletAddress === '') {
      setNotifications([]); setCount(0); setLoading(false); setError(null)
      return
    }
    try {
      setLoading(true); setError(null)
      const data = await optionsRef.current.actions.getNotifications({
        page: 1, limit: MAX_DROPDOWN_NOTIFICATIONS, status: 'all',
      })
      if (data.success && data.data?.notifications) {
        setNotifications(data.data.notifications.map(mapApiToNotification))
        setCount(data.data.unread_count ?? 0)
      } else {
        setNotifications([]); setCount(0)
      }
    } catch (err) {
      const apiError = err as { status?: number }
      if (apiError.status !== 401) {
        logger.warn('Failed to fetch notifications:', err)
        setError('Failed to load notifications')
      }
      setNotifications([]); setCount(0)
    } finally {
      setLoading(false)
    }
  }, [isAuthenticated, walletAddress])

  const markAsRead = useCallback(async (notificationId: string) => {
    try {
      const result = await optionsRef.current.actions.markAsRead(notificationId)
      if (result.success) {
        setNotifications((prev) => prev.map((n) => (n.id === notificationId ? { ...n, read: true } : n)))
        setCount((prev) => Math.max(0, prev - 1))
      }
    } catch (err) {
      logger.error('Failed to mark notification as read:', err)
    }
  }, [])

  const markAsUnread = useCallback(async (notificationId: string) => {
    try {
      const result = await optionsRef.current.actions.markAsUnread(notificationId)
      if (result.success) {
        setNotifications((prev) => prev.map((n) => (n.id === notificationId ? { ...n, read: false } : n)))
        setCount((prev) => prev + 1)
      }
    } catch (err) {
      logger.error('Failed to mark notification as unread:', err)
    }
  }, [])

  const markAllAsRead = useCallback(async () => {
    try {
      await optionsRef.current.actions.markAllAsRead()
      setNotifications((prev) => prev.map((n) => ({ ...n, read: true })))
      setCount(0)
    } catch (err) {
      logger.error('Failed to mark all notifications as read:', err)
    }
  }, [])

  useEffect(() => {
    void fetchNotifications()
  }, [fetchNotifications])

  return {
    notifications, count, loading, error, isSSEConnected,
    fetchNotifications, markAsRead, markAsUnread, markAllAsRead, reconnectSSE,
  }
}
