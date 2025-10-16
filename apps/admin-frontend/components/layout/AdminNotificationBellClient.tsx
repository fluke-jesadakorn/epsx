'use client'

import { useState, useEffect } from 'react'
import { createNotificationsClient } from '@/shared/api/notifications'
import { createAdminApiClient } from '@/shared/utils/api-client'
import { useSSENotifications } from '@/shared/hooks/useSSENotifications'
import {
  getNotificationIcon,
  formatTimestamp,
  formatWalletAddress,
  getPriorityBgGradient,
  getPriorityBorderColor,
  getPriorityTextColor,
  getPrioritySubTextColor
} from '@/shared/components/notifications/utils'
import type { Notification } from '@/shared/components/notifications/types'
import { MAX_DROPDOWN_NOTIFICATIONS } from '@/shared/components/notifications/constants'
import toast from 'react-hot-toast'

export function AdminNotificationBell() {
  const [count, setCount] = useState(0)
  const [notifications, setNotifications] = useState<Notification[]>([])
  const [showNotifications, setShowNotifications] = useState(false)
  const [loading, setLoading] = useState(true)

  // SSE real-time notifications for admin
  const { isConnected: sseConnected, reconnect: reconnectSSE } = useSSENotifications({
    apiClient: createAdminApiClient(),
    autoConnect: true, // Admin always connects to receive all notifications
    onNotification: (sseNotif) => {
      // Add to notifications list
      const newNotification: Notification = {
        id: sseNotif.id,
        title: sseNotif.title,
        message: sseNotif.message,
        type: sseNotif.notification_type as any,
        priority: sseNotif.priority as any,
        timestamp: sseNotif.timestamp,
        wallet_address: sseNotif.wallet_address,
        read: false,
      }

      setNotifications(prev => [newNotification, ...prev])
      setCount(prev => prev + 1)
    },
    onError: (error) => {
      console.warn('Admin SSE connection error:', error)
    },
    onConnect: () => {
      console.log('✅ Admin SSE connected')
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
        limit: MAX_DROPDOWN_NOTIFICATIONS,
        status: 'unread'
      })

      const mappedNotifications: Notification[] = data.data.notifications.map(n => ({
        id: n.id,
        title: n.title,
        message: n.message,
        type: n.notification_type as any,
        priority: n.priority as any,
        timestamp: n.timestamp,
        wallet_address: n.wallet_address,
        read: !!n.read_at
      }))

      setNotifications(mappedNotifications)
      setCount(data.data.unread_count)
    } catch (error) {
      const apiError = error as any
      if (apiError?.status !== 401) {
        console.warn('Failed to fetch notifications:', error)
      }
      setNotifications([])
      setCount(0)
    } finally {
      setLoading(false)
    }
  }

  const handleToggleDropdown = (e: React.MouseEvent) => {
    e.stopPropagation()
    setShowNotifications(!showNotifications)
  }

  const handleCloseDropdown = () => {
    setShowNotifications(false)
  }

  const handleDeleteNotification = async (e: React.MouseEvent, notificationId: string) => {
    e.stopPropagation()
    try {
      const client = createNotificationsClient(createAdminApiClient())
      await client.deleteAdminNotification(notificationId)

      // Remove from local state
      setNotifications(prev => prev.filter(n => n.id !== notificationId))
      setCount(prev => Math.max(0, prev - 1))

      toast.success('Notification deleted')
    } catch (error) {
      console.error('Failed to delete notification:', error)
      toast.error('Failed to delete notification')
    }
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
                      className={`rounded-2xl border bg-gradient-to-r p-4 group ${getPriorityBorderColor(notification.priority)} ${getPriorityBgGradient(notification.priority)}`}
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
                            {notification.wallet_address && notification.wallet_address !== 'all' && (
                              <span className="font-mono">{formatWalletAddress(notification.wallet_address)}</span>
                            )}
                          </div>
                        </div>
                        <button
                          onClick={(e) => handleDeleteNotification(e, notification.id)}
                          className="opacity-0 group-hover:opacity-100 text-gray-400 hover:text-red-500 dark:text-gray-500 dark:hover:text-red-400 flex-shrink-0"
                          title="Delete notification"
                        >
                          ✕
                        </button>
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
