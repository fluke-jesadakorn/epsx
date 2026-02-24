 
'use client'

import { useSharedAuth } from '@/shared/components/auth'
import { formatTimestamp, getNotificationIcon, getPriorityColor } from '@/shared/components/notifications/utils'
import { useNotificationBell } from '@/shared/hooks/use-notification-bell'
import { createFrontendApiClient } from '@/shared/utils/api-client'
import { Bell, ExternalLink } from 'lucide-react'
import Link from 'next/link'
import { useEffect, useRef, useState } from 'react'
import { useBrowserNotifications } from './browser-notifications'
import {
  getInitialNotificationsAction,
  markAsReadAction,
  markAllAsReadAction,
  deleteNotificationAction,
} from '@/app/actions/notifications'
import type { Notification } from '@/shared/types/notifications'

interface NotificationItemProps {
  notification: Notification
  onNotificationClick: (id: string) => void
  onDeleteNotification: (e: React.MouseEvent, id: string) => void
}

function NotificationItem({ notification, onNotificationClick, onDeleteNotification }: NotificationItemProps) {
  return (
    <div className="flex items-start gap-3 px-4 py-3 border-b border-orange-50 dark:border-slate-800 group hover:bg-orange-50/50 dark:hover:bg-slate-800/50 transition-colors cursor-pointer">
      <div className={`w-8 h-8 rounded-full ${getPriorityColor(notification.priority)} flex items-center justify-center flex-shrink-0`}>
        <span className="text-sm">{getNotificationIcon(notification.type)}</span>
      </div>
      <div
        className="flex-1 min-w-0 cursor-pointer"
        onClick={() => {
          onNotificationClick(notification.id)
        }}
      >
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
      <button
        onClick={(e) => {
          onDeleteNotification(e, notification.id)
        }}
        className="opacity-0 group-hover:opacity-100 text-slate-400 hover:text-red-500 dark:text-slate-500 dark:hover:text-red-400 flex-shrink-0"
        title="Delete notification"
      >
        ✕
      </button>
    </div>
  )
}

export function NotificationBellClient() {
  const [isOpen, setIsOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!isOpen) return
    const handler = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false)
      }
    }
    document.addEventListener('mousedown', handler)
    return () => { document.removeEventListener('mousedown', handler) }
  }, [isOpen])

  // Get authentication state
  const { isAuthenticated, user, refreshSession } = useSharedAuth()

  // Browser notifications integration
  const { showNotification: showBrowserNotification } = useBrowserNotifications()

  // Use shared notification bell hook with server actions
  const {
    notifications,
    count,
    loading,
    fetchNotifications,
    markAsRead,
    markAllAsRead,
  } = useNotificationBell({
    actions: {
      getNotifications: getInitialNotificationsAction,
      markAsRead: markAsReadAction,
      markAllAsRead: markAllAsReadAction,
      deleteNotification: deleteNotificationAction,
    },
    apiClient: createFrontendApiClient(), // Only needed for SSE
    walletAddress: user?.wallet_address,
    isAuthenticated,
    refreshSession,
    enableSSE: true,
    browserNotifications: {
      showNotification: (type, title, message) => {
        const notifType = type === 'security' ? 'security' :
          type === 'permission' ? 'permissions' :
            type === 'wallet' ? 'analytics' : 'system'
        showBrowserNotification(notifType, title, message)
      }
    },
  })

  const handleToggleDropdown = () => {
    setIsOpen(!isOpen)
  }

  const handleCloseDropdown = () => {
    setIsOpen(false)
  }

  const handleNotificationClick = async (notificationId: string) => {
    await markAsRead(notificationId)
  }

  // Delete notification functionality using server action
  const handleDeleteNotification = async (e: React.MouseEvent, notificationId: string) => {
    e.stopPropagation()
    try {
      await deleteNotificationAction(notificationId)
      void fetchNotifications() // Refresh the list
    } catch (_err) {
      // Error logged silently
    }
  }

  return (
    <div className="relative" ref={containerRef}>
      <button
        onClick={handleToggleDropdown}
        className="relative flex items-center gap-2 rounded-2xl px-3 py-2 text-sm font-medium text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-white/10 dark:bg-slate-800/40 dark:hover:text-slate-200"
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
                    <NotificationItem
                      key={notification.id}
                      notification={notification}
                      onNotificationClick={(id) => {
                        void handleNotificationClick(id)
                      }}
                      onDeleteNotification={(e, id) => {
                        void handleDeleteNotification(e, id)
                      }}
                    />
                  ))}
                </div>

                <div className="px-4 py-3 border-t border-orange-100 dark:border-slate-700 space-y-2">
                  <button
                    onClick={() => {
                      void markAllAsRead()
                    }}
                    className="w-full flex items-center justify-center gap-2 rounded-xl px-3 py-2 text-sm font-medium text-orange-600 hover:bg-orange-50 dark:text-orange-400 dark:hover:bg-orange-900/20"
                  >
                    <Bell className="h-4 w-4" />
                    Mark All as Read
                  </button>
                  <Link
                    href="/notifications"
                    onClick={handleCloseDropdown}
                    className="w-full flex items-center justify-center gap-2 rounded-xl px-3 py-2 text-sm font-medium text-slate-600 hover:bg-slate-50 dark:text-slate-400 dark:hover:bg-white/10 dark:bg-slate-800/40"
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

