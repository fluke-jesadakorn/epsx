 
'use client'

import { useSharedAuth } from '@/shared/components/auth'
import { formatTimestamp, getNotificationIcon, getPriorityColor } from '@/shared/components/notifications/utils'
import { useNotificationBell } from '@/shared/hooks/use-notification-bell'
import { createFrontendApiClient } from '@/shared/utils/api-client'
import { Bell, ExternalLink, RefreshCw, LogIn } from 'lucide-react'
import Link from 'next/link'
import { useEffect, useRef, useState } from 'react'
import { useBrowserNotifications } from './browser-notifications'
import {
  getInitialNotificationsAction,
  markAsReadAction,
  markAsUnreadAction,
  markAllAsReadAction,
  deleteNotificationAction,
} from '@/app/actions/notifications'
import type { Notification } from '@/shared/types/notifications'

interface NotificationItemProps {
  notification: Notification
  onToggleRead: (id: string, read: boolean) => void
  onDeleteNotification: (e: React.MouseEvent, id: string) => void
}

function NotificationItem({ notification, onToggleRead, onDeleteNotification }: NotificationItemProps) {
  const unread = !notification.read
  return (
    <div className={`flex items-start gap-3 px-4 py-3 border-b border-orange-50 dark:border-slate-800 group transition-colors cursor-pointer ${unread ? 'bg-orange-50/30 dark:bg-orange-950/15 hover:bg-orange-50/60 dark:hover:bg-orange-950/25' : 'opacity-60 hover:opacity-80 hover:bg-slate-50/50 dark:hover:bg-slate-800/30'}`}
      onClick={() => { onToggleRead(notification.id, notification.read) }}
    >
      <div className={`w-8 h-8 rounded-full ${getPriorityColor(notification.priority)} flex items-center justify-center flex-shrink-0`}>
        <span className="text-sm">{getNotificationIcon(notification.type)}</span>
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-start justify-between gap-2">
          <p className={`text-sm line-clamp-1 ${unread ? 'font-semibold text-slate-900 dark:text-slate-100' : 'font-normal text-slate-500 dark:text-slate-400'}`}>
            {notification.title}
          </p>
          <div className="flex-shrink-0 mt-1.5 w-5 h-5 flex items-center justify-center">
            {unread ? (
              <div className="w-2.5 h-2.5 rounded-full bg-orange-500" />
            ) : (
              <div className="w-2.5 h-2.5 rounded-full border border-slate-300 dark:border-slate-600 group-hover:border-orange-400" />
            )}
          </div>
        </div>
        <p className={`text-xs line-clamp-2 mt-0.5 ${unread ? 'text-slate-600 dark:text-slate-300' : 'text-slate-400 dark:text-slate-500'}`}>
          {notification.message}
        </p>
        <p className="text-xs text-slate-400 dark:text-slate-500 mt-1">
          {formatTimestamp(notification.timestamp)}
        </p>
      </div>
      <button
        onClick={(e) => {
          e.stopPropagation()
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
    if (!isOpen) {return}
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
    error,
    fetchNotifications,
    markAsRead,
    markAsUnread,
    markAllAsRead,
  } = useNotificationBell({
    actions: {
      getNotifications: getInitialNotificationsAction,
      markAsRead: markAsReadAction,
      markAsUnread: markAsUnreadAction,
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

  const handleToggleRead = async (notificationId: string, currentlyRead: boolean) => {
    if (currentlyRead) {
      await markAsUnread(notificationId)
    } else {
      await markAsRead(notificationId)
    }
    void fetchNotifications()
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
        className="relative flex items-center gap-2 rounded-2xl px-3 py-2 text-sm font-medium text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-white/10 dark:hover:text-slate-200"
      >
        <Bell className="h-4 w-4 text-orange-500" />
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
            ) : !isAuthenticated ? (
              <div className="px-4 py-8 text-center">
                <LogIn className="h-12 w-12 mx-auto mb-3 text-slate-300 dark:text-slate-600" />
                <p className="text-sm text-slate-500 dark:text-slate-400 mb-3">
                  Sign in to see notifications
                </p>
                <Link
                  href="/auth"
                  onClick={handleCloseDropdown}
                  className="inline-flex items-center gap-2 rounded-xl px-4 py-2 text-sm font-medium bg-orange-500 text-white hover:bg-orange-600 transition-colors"
                >
                  <LogIn className="h-4 w-4" />
                  Connect Wallet
                </Link>
              </div>
            ) : notifications.length === 0 ? (
              <div className="px-4 py-8 text-center">
                <Bell className="h-12 w-12 mx-auto mb-3 text-slate-300 dark:text-slate-600" />
                <p className="text-sm text-slate-500 dark:text-slate-400 mb-3">
                  {error !== null ? 'Could not load notifications' : 'No notifications yet'}
                </p>
                <button
                  onClick={() => { void fetchNotifications() }}
                  className="inline-flex items-center gap-2 rounded-xl px-4 py-2 text-sm font-medium text-orange-600 hover:bg-orange-50 dark:text-orange-400 dark:hover:bg-orange-900/20 transition-colors"
                >
                  <RefreshCw className="h-4 w-4" />
                  {error !== null ? 'Retry' : 'Refresh'}
                </button>
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
                      onToggleRead={(id, read) => {
                        void handleToggleRead(id, read)
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
                    className="w-full flex items-center justify-center gap-2 rounded-xl px-3 py-2 text-sm font-medium text-slate-600 hover:bg-slate-50 dark:text-slate-400 dark:hover:bg-white/10"
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

