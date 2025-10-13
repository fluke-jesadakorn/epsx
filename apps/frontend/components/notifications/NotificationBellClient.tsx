'use client'

import { Bell, Settings, ExternalLink } from 'lucide-react'
import { useState, useEffect } from 'react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { createNotificationsClient } from '@/shared/api/notifications'
import { createFrontendApiClient } from '@/shared/utils/api-client'
import { useSSENotifications } from '@/shared/hooks/useSSENotifications'
import { useBrowserNotifications } from './BrowserNotifications'
import { useWeb3AuthStore } from '@/lib/auth/store'

interface Notification {
  id: string
  title: string
  message: string
  type: string
  priority: string
  timestamp: string
  actionUrl?: string
  read: boolean
}

export function NotificationBellClient() {
  const router = useRouter()
  const [count, setCount] = useState(0)
  const [notifications, setNotifications] = useState<Notification[]>([])
  const [isOpen, setIsOpen] = useState(false)
  const [loading, setLoading] = useState(true)

  // Get wallet authentication state
  const { isAuthenticated, walletAddress, isConnected } = useWeb3AuthStore()

  // Debug authentication state
  useEffect(() => {
    console.log('🔐 NotificationBell Auth State:', {
      isConnected,
      isAuthenticated,
      walletAddress,
      willConnectSSE: isAuthenticated && !!walletAddress
    })
  }, [isConnected, isAuthenticated, walletAddress])

  // Browser notifications integration
  const { showNotification: showBrowserNotification } = useBrowserNotifications()

  // SSE real-time notifications - only connect if authenticated
  const { isConnected: sseConnected } = useSSENotifications({
    apiClient: createFrontendApiClient(),
    walletAddress: walletAddress,
    autoConnect: isAuthenticated, // Only auto-connect if authenticated
    onNotification: (sseNotif) => {
      // Add to notifications list
      const newNotification: Notification = {
        id: sseNotif.id,
        title: sseNotif.title,
        message: sseNotif.message,
        type: sseNotif.notification_type,
        priority: sseNotif.priority,
        timestamp: sseNotif.timestamp,
        actionUrl: undefined,
        read: false,
      }

      setNotifications(prev => [newNotification, ...prev])
      setCount(prev => prev + 1)

      // Show browser notification for high priority
      if (sseNotif.priority === 'high' || sseNotif.priority === 'critical') {
        const notifType = sseNotif.notification_type === 'security' ? 'security' :
                         sseNotif.notification_type === 'permission' ? 'permissions' :
                         sseNotif.notification_type === 'wallet' ? 'trading' : 'system'

        showBrowserNotification(notifType, sseNotif.title, sseNotif.message)
      }
    },
    onError: (error) => {
      console.warn('SSE connection error:', error)
    },
  })

  // Fetch notifications when authentication state changes
  useEffect(() => {
    if (isAuthenticated && walletAddress) {
      fetchNotifications()
    } else {
      // Clear notifications when not authenticated
      setNotifications([])
      setCount(0)
      setLoading(false)
    }
  }, [isAuthenticated, walletAddress])

  const fetchNotifications = async () => {
    try {
      setLoading(true)
      const client = createNotificationsClient(createFrontendApiClient())
      const data = await client.getNotifications({
        page: 1,
        limit: 5,
        status: 'unread'
      })

      const mappedNotifications = data.data.notifications.map(n => ({
        id: n.id,
        title: n.title,
        message: n.message,
        type: n.notification_type,
        priority: n.priority,
        timestamp: n.timestamp,
        actionUrl: n.action_url,
        read: !!n.read_at
      }))

      setNotifications(mappedNotifications)
      setCount(data.data.unread_count)
    } catch (error) {
      // Silently fail if not authenticated - this is expected behavior
      // The notification bell will simply show 0 notifications
      const apiError = error as any
      if (apiError?.status !== 401) {
        console.warn('Failed to fetch notifications:', error)
      }
      // Set empty state
      setNotifications([])
      setCount(0)
    } finally {
      setLoading(false)
    }
  }

  const markAsRead = async (notificationId: string) => {
    try {
      const client = createNotificationsClient(createFrontendApiClient())
      await client.markAsRead(notificationId)

      // Update local state
      setNotifications(prev => prev.map(n =>
        n.id === notificationId ? { ...n, read: true } : n
      ))
      setCount(prev => Math.max(0, prev - 1))
    } catch (error) {
      console.error('Failed to mark notification as read:', error)
    }
  }

  const markAllAsRead = async () => {
    try {
      const client = createNotificationsClient(createFrontendApiClient())
      await client.markAllAsRead()

      // Update local state
      setNotifications(prev => prev.map(n => ({ ...n, read: true })))
      setCount(0)
    } catch (error) {
      console.error('Failed to mark all as read:', error)
    }
  }

  const getNotificationIcon = (type: string) => {
    switch (type) {
      case 'security': return '🔒'
      case 'permission': return '🔑'
      case 'wallet': return '💼'
      case 'payment': return '💳'
      case 'system': return '⚙️'
      default: return '📬'
    }
  }

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'critical':
      case 'urgent': return 'bg-red-500'
      case 'high': return 'bg-orange-500'
      case 'normal': return 'bg-blue-500'
      case 'low': return 'bg-green-500'
      default: return 'bg-gray-500'
    }
  }

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const minutes = Math.floor(diff / (1000 * 60))
    const hours = Math.floor(diff / (1000 * 60 * 60))
    const days = Math.floor(diff / (1000 * 60 * 60 * 24))

    if (minutes < 1) return 'Just now'
    if (minutes < 60) return `${minutes}m ago`
    if (hours < 24) return `${hours}h ago`
    if (days < 7) return `${days}d ago`
    return date.toLocaleDateString()
  }

  const handleToggleDropdown = (e: React.MouseEvent) => {
    e.stopPropagation()
    setIsOpen(!isOpen)
  }

  const handleCloseDropdown = () => {
    setIsOpen(false)
  }

  const handleNotificationClick = async (notificationId: string) => {
    handleCloseDropdown()
    // Auto-mark as read when clicked
    await markAsRead(notificationId)
    router.push(`/notifications?id=${notificationId}`)
  }

  return (
    <div className="relative">
      <button
        onClick={handleToggleDropdown}
        className="relative flex items-center gap-2 rounded-2xl px-3 py-2 text-sm font-medium text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200"
      >
        <Bell className="h-5 w-5 text-orange-500" />
        {count > 0 && (
          <span className="absolute -top-1 -right-1 flex h-5 w-5 items-center justify-center rounded-full bg-red-500 text-xs text-white">
            {count > 9 ? '9+' : count}
          </span>
        )}
      </button>

      {isOpen && (
        <>
          {/* Backdrop */}
          <div
            className="fixed inset-0 z-40"
            onClick={handleCloseDropdown}
          />

          {/* Dropdown */}
          <div className="absolute top-full right-0 z-50 mt-2 w-96 rounded-2xl border border-orange-100/50 bg-white/95 backdrop-blur-xl shadow-2xl dark:border-slate-700/50 dark:bg-slate-900/95">
            {loading ? (
              <div className="px-4 py-8 text-center">
                <Bell className="h-12 w-12 mx-auto mb-3 text-slate-300 dark:text-slate-600 animate-pulse" />
                <p className="text-sm text-slate-500 dark:text-slate-400">
                  Loading notifications...
                </p>
              </div>
            ) : notifications.length === 0 ? (
              <div className="px-4 py-8 text-center">
                <Bell className="h-12 w-12 mx-auto mb-3 text-slate-300 dark:text-slate-600" />
                <p className="text-sm text-slate-500 dark:text-slate-400">
                  No notifications yet
                </p>
              </div>
            ) : (
              <>
                <div className="px-4 py-3 border-b border-orange-100 dark:border-slate-700">
                  <div className="flex items-center justify-between">
                    <h3 className="text-sm font-semibold text-slate-700 dark:text-slate-300">
                      Notifications
                    </h3>
                    {count > 0 && (
                      <span className="text-xs text-orange-600 dark:text-orange-400">
                        {count} unread
                      </span>
                    )}
                  </div>
                </div>

                <div className="max-h-96 overflow-y-auto">
                  {notifications.map((notification) => (
                    <div
                      key={notification.id}
                      className="flex items-start gap-3 px-4 py-3 border-b border-orange-50 dark:border-slate-800 cursor-pointer hover:bg-slate-50/80 dark:hover:bg-slate-800/40"
                      onClick={() => handleNotificationClick(notification.id)}
                    >
                      <div className={`w-8 h-8 rounded-full ${getPriorityColor(notification.priority)} flex items-center justify-center flex-shrink-0`}>
                        <span className="text-sm">{getNotificationIcon(notification.type)}</span>
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-start justify-between gap-2">
                          <p className="text-sm font-medium text-slate-900 dark:text-slate-100 line-clamp-1">
                            {notification.title}
                          </p>
                          {!notification.read && (
                            <div className="w-2 h-2 rounded-full bg-orange-500 flex-shrink-0 mt-1.5" />
                          )}
                        </div>
                        <p className="text-xs text-slate-600 dark:text-slate-400 line-clamp-2 mt-0.5">
                          {notification.message}
                        </p>
                        <p className="text-xs text-slate-400 dark:text-slate-500 mt-1">
                          {formatTimestamp(notification.timestamp)}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>

                <div className="px-4 py-3 border-t border-orange-100 dark:border-slate-700 space-y-2">
                  <button
                    onClick={markAllAsRead}
                    className="w-full flex items-center justify-center gap-2 rounded-xl px-3 py-2 text-sm font-medium text-orange-600 hover:bg-orange-50 dark:text-orange-400 dark:hover:bg-orange-900/20"
                  >
                    <Bell className="h-4 w-4" />
                    Mark All as Read
                  </button>
                  <Link
                    href="/notifications"
                    onClick={handleCloseDropdown}
                    className="w-full flex items-center justify-center gap-2 rounded-xl px-3 py-2 text-sm font-medium text-slate-600 hover:bg-slate-50 dark:text-slate-400 dark:hover:bg-slate-800/40"
                  >
                    <ExternalLink className="h-4 w-4" />
                    View All Notifications
                  </Link>
                </div>
              </>
            )}
          </div>
        </>
      )}
    </div>
  )
}
