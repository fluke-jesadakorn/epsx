'use client'

import { usePathname } from 'next/navigation'
import { useEffect, useState } from 'react'

import { toast } from '@/hooks/use-toast'
import { useSharedAuth } from '@/shared/components/auth'
import { MAX_DROPDOWN_NOTIFICATIONS } from '@/shared/components/notifications/constants'
import type { Notification } from '@/shared/components/notifications/types'
import {
  formatTimestamp,
  formatWalletAddress,
  getNotificationIcon
} from '@/shared/components/notifications/utils'
import { useSSENotifications } from '@/shared/hooks/use-sse-notifications'
import { createAdminApiClient } from '@/shared/utils/api-client'
import {
  deleteAdminNotificationAction,
  getAdminNotificationsAction,
} from '@/app/actions/notifications'
import type { SSENotification } from '@/shared/api/notifications'

// eslint-disable-next-line max-lines-per-function
export function AdminNotificationBell() {
  const pathname = usePathname()
  const [count, setCount] = useState(0)
  const [notifications, setNotifications] = useState<Notification[]>([])
  const [showNotifications, setShowNotifications] = useState(false)
  const [loading, setLoading] = useState(true)

  const isOnAuthPage = pathname === '/auth' || pathname.startsWith('/auth')
  const { refreshSession } = useSharedAuth()

  const fetchNotifications = async () => {
    try {
      setLoading(true)
      const data = await getAdminNotificationsAction({
        page: 1,
        limit: MAX_DROPDOWN_NOTIFICATIONS,
        status: 'unread',
      })

      if (data.success && data.data?.notifications) {
        const mapped: Notification[] = data.data.notifications.map(n => ({
          id: n.id,
          title: n.title,
          message: n.message,
          type: n.notification_type,
          priority: n.priority,
          timestamp: n.timestamp,
          wallet_address: n.wallet_address,
          read: Boolean(n.read_at),
        }))
        setNotifications(mapped)
        setCount(data.data.unread_count ?? 0)
      } else {
        setNotifications([])
        setCount(0)
      }
    } catch {
      setNotifications([])
      setCount(0)
    } finally {
      setLoading(false)
    }
  }

  const { isConnected: sseConnected, reconnect: _reconnectSSE } = useSSENotifications({
    apiClient: createAdminApiClient(),
    autoConnect: !isOnAuthPage,
    refreshSession,
    onNotification: (sseNotif: SSENotification) => {
      const newNotification: Notification = {
        id: sseNotif.id,
        title: sseNotif.title,
        message: sseNotif.message,
        type: sseNotif.notification_type as Notification['type'],
        priority: sseNotif.priority,
        timestamp: sseNotif.timestamp,
        wallet_address: sseNotif.wallet_address,
        read: false,
      }

      setNotifications(prev => {
        if (prev.some(n => n.id === newNotification.id)) { return prev }
        setCount(c => c + 1)
        return [newNotification, ...prev]
      })
    },
    onError: () => { /* silently fail */ },
    onConnect: () => { /* no-op */ },
  })

  useEffect(() => {
    if (isOnAuthPage) {
      setLoading(false)
      return
    }
    void fetchNotifications()
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isOnAuthPage])

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
      await deleteAdminNotificationAction(notificationId)
      setNotifications(prev => prev.filter(n => n.id !== notificationId))
      setCount(prev => Math.max(0, prev - 1))
      toast({ title: 'Notification deleted' })
    } catch {
      toast({ title: 'Failed to delete notification', variant: 'destructive' })
    }
  }

  return (
    <>
      <div className="relative">
        <button
          onClick={handleToggleDropdown}
          className="relative p-2 rounded-lg text-primary hover:bg-accent transition-colors"
        >
          <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
          </svg>
          {sseConnected && (
            <div className="absolute top-1 right-1 h-2 w-2 rounded-full bg-success" title="Real-time connected" />
          )}
          {count > 0 && (
            <div className="absolute -top-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full bg-destructive text-[10px] font-bold text-destructive-foreground">
              {count > 9 ? '9+' : count}
            </div>
          )}
        </button>

        {showNotifications && (
          <div className="absolute top-12 right-0 z-50 w-80 rounded-xl border border-gray-200 dark:border-slate-700 bg-white dark:bg-[#1a1d2e] p-4 shadow-2xl" onClick={e => e.stopPropagation()}>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-semibold text-foreground">
                  Notifications
                </h3>
                {count > 0 && (
                  <span className="rounded-full bg-primary/20 px-2 py-0.5 text-xs font-medium text-primary">
                    {count} new
                  </span>
                )}
              </div>

              {loading ? (
                <div className="py-12 text-center">
                  <svg className="mx-auto h-12 w-12 text-muted-foreground/30 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                  </svg>
                  <div className="mt-3 flex items-center justify-center gap-2">
                    <svg className="h-4 w-4 animate-spin text-muted-foreground" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                    </svg>
                    <span className="text-sm text-muted-foreground">Loading notifications...</span>
                  </div>
                </div>
              ) : notifications.length === 0 ? (
                <div className="py-12 text-center">
                  <svg className="mx-auto h-12 w-12 text-muted-foreground/30" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                  </svg>
                  <p className="mt-3 text-sm text-muted-foreground">No notifications yet</p>
                </div>
              ) : (
                <div className="space-y-2 max-h-80 overflow-y-auto">
                  {notifications.map((notification) => (
                    <div
                      key={notification.id}
                      className="group rounded-lg border border-border bg-muted/30 p-3 hover:bg-accent hover:border-border/80 transition-colors cursor-pointer"
                    >
                      <div className="flex items-start gap-3">
                        <span className="text-lg">{getNotificationIcon(notification.type)}</span>
                        <div className="flex-1 min-w-0">
                          <div className="font-medium text-sm text-foreground">
                            {notification.title}
                          </div>
                          <div className="text-xs text-muted-foreground mt-0.5 line-clamp-2">
                            {notification.message}
                          </div>
                          <div className="mt-1.5 flex items-center gap-2 text-xs text-muted-foreground/70">
                            <span>{formatTimestamp(notification.timestamp)}</span>
                            {notification.wallet_address !== undefined && notification.wallet_address !== 'all' && (
                              <span className="font-mono">{formatWalletAddress(notification.wallet_address)}</span>
                            )}
                          </div>
                        </div>
                        <button
                          onClick={(e) => void handleDeleteNotification(e, notification.id)}
                          className="opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive transition-opacity"
                          title="Delete"
                        >
                          <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                          </svg>
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}

              <div className="border-t border-border pt-3">
                <button
                  onClick={handleCloseDropdown}
                  className="w-full rounded-lg bg-muted py-2 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
                >
                  View All Notifications
                </button>
              </div>
            </div>
          </div>
        )}
      </div>

      {showNotifications && (
        <div
          className="fixed inset-0 z-[39]"
          onClick={handleCloseDropdown}
        />
      )}
    </>
  )
}
