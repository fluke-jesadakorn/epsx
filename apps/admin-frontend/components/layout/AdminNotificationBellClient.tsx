'use client'

import { useState, useEffect } from 'react'
import { createNotificationsClient } from '@/shared/api/notifications'
import { createAdminApiClient } from '@/shared/utils/api-client'
import { useSSENotifications } from '@/shared/hooks/useSSENotifications'

interface Notification {
  id: string
  title: string
  message: string
  type: string
  priority: string
  timestamp: string
  walletAddress: string
  read: boolean
}

export function AdminNotificationBell() {
  const [count, setCount] = useState(0)
  const [notifications, setNotifications] = useState<Notification[]>([])
  const [showNotifications, setShowNotifications] = useState(false)
  const [loading, setLoading] = useState(true)

  // SSE real-time notifications for admin
  const { isConnected: sseConnected } = useSSENotifications({
    apiClient: createAdminApiClient(),
    autoConnect: true,
    onNotification: (sseNotif) => {
      // Add to notifications list
      const newNotification: Notification = {
        id: sseNotif.id,
        title: sseNotif.title,
        message: sseNotif.message,
        type: sseNotif.notification_type,
        priority: sseNotif.priority,
        timestamp: sseNotif.timestamp,
        walletAddress: sseNotif.wallet_address,
        read: false,
      }

      setNotifications(prev => [newNotification, ...prev])
      setCount(prev => prev + 1)
    },
    onError: (error) => {
      console.warn('Admin SSE connection error:', error)
    },
  })

  useEffect(() => {
    fetchNotifications()
  }, [])

  const fetchNotifications = async () => {
    try {
      setLoading(true)
      const client = createNotificationsClient(createAdminApiClient())
      const data = await client.getAllNotifications({
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
        walletAddress: n.wallet_address,
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

  const getNotificationIcon = (type: string) => {
    switch (type) {
      case 'security': return '🔒'
      case 'permission': return '🔑'
      case 'user_management': return '👥'
      case 'wallet': return '💼'
      case 'payment': return '💳'
      case 'system': return '⚙️'
      default: return '📬'
    }
  }

  const getPriorityBgColor = (priority: string) => {
    switch (priority) {
      case 'critical':
      case 'urgent': return 'from-red-50 to-pink-50 dark:from-red-900/20 dark:to-pink-900/20'
      case 'high': return 'from-orange-50 to-red-50 dark:from-orange-900/20 dark:to-red-900/20'
      case 'normal': return 'from-blue-50 to-purple-50 dark:from-blue-900/20 dark:to-purple-900/20'
      case 'low': return 'from-green-50 to-teal-50 dark:from-green-900/20 dark:to-teal-900/20'
      default: return 'from-gray-50 to-gray-100 dark:from-gray-900/20 dark:to-gray-800/20'
    }
  }

  const getPriorityBorderColor = (priority: string) => {
    switch (priority) {
      case 'critical':
      case 'urgent': return 'border-red-200 dark:border-red-500/30'
      case 'high': return 'border-orange-200 dark:border-orange-500/30'
      case 'normal': return 'border-blue-200 dark:border-blue-500/30'
      case 'low': return 'border-green-200 dark:border-green-500/30'
      default: return 'border-gray-200 dark:border-gray-500/30'
    }
  }

  const getPriorityTextColor = (priority: string) => {
    switch (priority) {
      case 'critical':
      case 'urgent': return 'text-red-800 dark:text-red-300'
      case 'high': return 'text-orange-800 dark:text-orange-300'
      case 'normal': return 'text-blue-800 dark:text-blue-300'
      case 'low': return 'text-green-800 dark:text-green-300'
      default: return 'text-gray-800 dark:text-gray-300'
    }
  }

  const getPrioritySubTextColor = (priority: string) => {
    switch (priority) {
      case 'critical':
      case 'urgent': return 'text-red-600 dark:text-red-400'
      case 'high': return 'text-orange-600 dark:text-orange-400'
      case 'normal': return 'text-blue-600 dark:text-blue-400'
      case 'low': return 'text-green-600 dark:text-green-400'
      default: return 'text-gray-600 dark:text-gray-400'
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
    if (minutes < 60) return `${minutes} minutes ago`
    if (hours < 24) return `${hours} hours ago`
    if (days < 7) return `${days} days ago`
    return date.toLocaleDateString()
  }

  const handleToggleDropdown = (e: React.MouseEvent) => {
    e.stopPropagation()
    setShowNotifications(!showNotifications)
  }

  const handleCloseDropdown = () => {
    setShowNotifications(false)
  }

  return (
    <>
      {/* Notification Bell Button */}
      <div className="relative">
        <button
          onClick={handleToggleDropdown}
          className="relative h-12 w-12 rounded-2xl bg-gradient-to-r from-orange-400 to-red-500 font-semibold text-white shadow-lg"
        >
          <span className="text-xl">🔔</span>
          {sseConnected && (
            <div className="absolute top-1 right-1 h-2.5 w-2.5 rounded-full bg-green-400 ring-2 ring-white" title="Real-time connected" />
          )}
          {count > 0 && (
            <div className="absolute -top-2 -right-2 flex h-6 w-6 items-center justify-center rounded-full bg-gradient-to-r from-pink-500 to-red-500 text-xs text-white shadow-lg">
              {count > 9 ? '9+' : count}
            </div>
          )}
        </button>

        {/* Notification Dropdown */}
        {showNotifications && (
          <div className="absolute top-14 right-0 z-50 w-80 rounded-3xl border border-yellow-200 bg-white p-6 shadow-2xl dark:border-slate-700/50 dark:bg-slate-800">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h3 className="bg-gradient-to-r from-orange-600 to-red-600 bg-clip-text text-lg font-bold text-transparent">
                  🔥 Recent Notifications
                </h3>
                {count > 0 && (
                  <span className="rounded-full bg-gradient-to-r from-orange-500 to-red-500 px-3 py-1 text-xs font-semibold text-white">
                    {count} new
                  </span>
                )}
              </div>

              {loading ? (
                <div className="py-8 text-center text-gray-500 dark:text-gray-400">
                  <span className="text-4xl">⏳</span>
                  <p className="mt-2 text-sm">Loading notifications...</p>
                </div>
              ) : notifications.length === 0 ? (
                <div className="py-8 text-center text-gray-500 dark:text-gray-400">
                  <span className="text-4xl">📭</span>
                  <p className="mt-2 text-sm">No notifications yet</p>
                </div>
              ) : (
                <div className="space-y-3 max-h-96 overflow-y-auto">
                  {notifications.map((notification) => (
                    <div
                      key={notification.id}
                      className={`rounded-2xl border bg-gradient-to-r p-4 ${getPriorityBorderColor(notification.priority)} ${getPriorityBgColor(notification.priority)}`}
                    >
                      <div className="flex items-start gap-3">
                        <span className="text-xl">{getNotificationIcon(notification.type)}</span>
                        <div className="flex-1 min-w-0">
                          <div className={`font-semibold ${getPriorityTextColor(notification.priority)}`}>
                            {notification.title}
                          </div>
                          <div className={`text-sm ${getPrioritySubTextColor(notification.priority)}`}>
                            {notification.message}
                          </div>
                          <div className="mt-2 flex items-center justify-between text-xs text-gray-500">
                            <span>{formatTimestamp(notification.timestamp)}</span>
                            {notification.walletAddress && notification.walletAddress !== 'all' && (
                              <span className="font-mono">{notification.walletAddress.slice(0, 6)}...{notification.walletAddress.slice(-4)}</span>
                            )}
                          </div>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}

              <div className="space-y-2 border-t border-gray-200 pt-3 dark:border-gray-700">
                <button
                  onClick={handleCloseDropdown}
                  className="w-full rounded-2xl bg-gradient-to-r from-blue-400 to-purple-500 py-2 font-semibold text-white"
                >
                  ➕ Send Notification
                </button>
                <button
                  onClick={() => {
                    handleCloseDropdown()
                    // TODO: Navigate to notifications page
                  }}
                  className="w-full rounded-2xl bg-gradient-to-r from-orange-400 to-red-500 py-2 font-semibold text-white"
                >
                  View All Notifications
                </button>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Backdrop - close dropdown when clicking outside */}
      {showNotifications && (
        <div
          className="fixed inset-0 z-40"
          onClick={handleCloseDropdown}
        />
      )}
    </>
  )
}
