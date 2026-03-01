'use client'

import { Bell, ExternalLink, RefreshCw } from 'lucide-react'
import { usePathname, useRouter } from 'next/navigation'
import { useCallback, useEffect, useRef, useState } from 'react'

import {
  deleteAdminNotificationAction,
  getAdminNotificationsAction,
  markAllAsReadAction,
  markAsReadAction,
  markAsUnreadAction,
} from '@/app/actions/notifications'
import { toast } from '@/hooks/use-toast'
import type { SSENotification } from '@/shared/api/notifications'
import { useSharedAuth } from '@/shared/components/auth'
import { MAX_DROPDOWN_NOTIFICATIONS } from '@/shared/components/notifications/constants'
import type { Notification } from '@/shared/components/notifications/types'
import {
  formatTimestamp,
  formatWalletAddress,
  getNotificationIcon,
  getPriorityColor,
} from '@/shared/components/notifications/utils'
import { useSSENotifications } from '@/shared/hooks/use-sse-notifications'
import { createAdminApiClient } from '@/shared/utils/api-client'

interface NotificationItemProps {
  notification: Notification
  onToggleRead: (id: string, read: boolean) => void
  onDeleteNotification: (e: React.MouseEvent, id: string) => void
}

function NotificationItem({ notification, onToggleRead, onDeleteNotification }: NotificationItemProps) {
  const unread = !notification.read
  return (
    <div className={`flex items-start gap-3 px-4 py-3 border-b border-orange-50 dark:border-slate-800 group transition-colors ${unread ? 'bg-orange-50/30 dark:bg-orange-950/15' : 'opacity-60 hover:opacity-80 hover:bg-slate-50/50 dark:hover:bg-slate-800/30'}`}>
      <div className={`w-8 h-8 rounded-full ${getPriorityColor(notification.priority)} flex items-center justify-center flex-shrink-0`}>
        <span className="text-sm">{getNotificationIcon(notification.type)}</span>
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-start justify-between gap-2">
          <p className={`text-sm line-clamp-1 ${unread ? 'font-semibold text-slate-900 dark:text-foreground' : 'font-normal text-muted-foreground'}`}>
            {notification.title}
          </p>
          <button
            onClick={(e) => {
              e.stopPropagation()
              onToggleRead(notification.id, notification.read)
            }}
            className="flex-shrink-0 mt-0.5 w-7 h-7 flex items-center justify-center rounded-full hover:bg-orange-100/60 dark:hover:bg-orange-900/30 transition-colors"
            title={unread ? 'Mark as read' : 'Mark as unread'}
          >
            {unread ? (
              <div className="w-2.5 h-2.5 rounded-full bg-orange-500 hover:scale-125 transition-transform" />
            ) : (
              <div className="w-2.5 h-2.5 rounded-full border-2 border-slate-300 dark:border-border/40 hover:border-orange-400 transition-colors" />
            )}
          </button>
        </div>
        <p className={`text-xs line-clamp-2 mt-0.5 ${unread ? 'text-slate-600 dark:text-slate-300' : 'text-muted-foreground'}`}>
          {notification.message}
        </p>
        <div className="flex items-center gap-2 mt-1">
          <p className="text-xs text-slate-400 dark:text-muted-foreground">
            {formatTimestamp(notification.timestamp)}
          </p>
          {notification.wallet_address !== undefined && notification.wallet_address !== 'all' && (
            <span className="text-xs font-mono text-slate-400 dark:text-muted-foreground">
              {formatWalletAddress(notification.wallet_address)}
            </span>
          )}
        </div>
      </div>
      <button
        onClick={(e) => {
          e.stopPropagation()
          onDeleteNotification(e, notification.id)
        }}
        className="opacity-0 group-hover:opacity-100 text-slate-400 hover:text-red-500 dark:text-muted-foreground dark:hover:text-red-400 flex-shrink-0"
        title="Delete notification"
      >
        ✕
      </button>
    </div>
  )
}

interface DropdownProps {
  loading: boolean
  error: string | null
  notifications: Notification[]
  count: number
  onRefresh: () => void
  onToggleRead: (id: string, read: boolean) => void
  onDelete: (e: React.MouseEvent, id: string) => void
  onMarkAllRead: () => void
  onViewAll: () => void
}

function NotificationDropdown({ loading, error, notifications, count, onRefresh, onToggleRead, onDelete, onMarkAllRead, onViewAll }: DropdownProps) {
  return (
    <div className="absolute top-full right-0 z-50 mt-2 w-96 rounded-2xl border border-orange-100/50 bg-white/95 shadow-2xl dark:border-border/40 dark:bg-card/95">
      {loading ? (
        <div className="px-4 py-8 text-center">
          <Bell className="h-12 w-12 mx-auto mb-3 text-muted-foreground animate-pulse" />
          <p className="text-sm text-muted-foreground">Loading notifications...</p>
        </div>
      ) : notifications.length === 0 ? (
        <div className="px-4 py-8 text-center">
          <Bell className="h-12 w-12 mx-auto mb-3 text-muted-foreground" />
          <p className="text-sm text-muted-foreground mb-3">{error !== null ? 'Could not load notifications' : 'No notifications yet'}</p>
          <button onClick={onRefresh} className="inline-flex items-center gap-2 rounded-xl px-4 py-2 text-sm font-medium text-orange-600 hover:bg-orange-50 dark:text-orange-400 dark:hover:bg-orange-900/20 transition-colors">
            <RefreshCw className="h-4 w-4" />
            {error !== null ? 'Retry' : 'Refresh'}
          </button>
        </div>
      ) : (
        <>
          <div className="px-4 py-3 border-b border-orange-100 dark:border-slate-700">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold text-slate-700 dark:text-muted-foreground">Notifications</h3>
              {count > 0 && <span className="text-xs text-orange-600 dark:text-orange-400">{count} unread</span>}
            </div>
          </div>
          <div className="max-h-96 overflow-y-auto">
            {notifications.map((notification) => (
              <NotificationItem key={notification.id} notification={notification} onToggleRead={onToggleRead} onDeleteNotification={onDelete} />
            ))}
          </div>
          <div className="px-4 py-3 border-t border-orange-100 dark:border-slate-700 space-y-2">
            {count > 0 && (
              <button onClick={onMarkAllRead} className="w-full flex items-center justify-center gap-2 rounded-xl px-3 py-2 text-sm font-medium text-orange-600 hover:bg-orange-50 dark:text-orange-400 dark:hover:bg-orange-900/20">
                <Bell className="h-4 w-4" />
                Mark All as Read
              </button>
            )}
            <button onClick={onViewAll} className="w-full flex items-center justify-center gap-2 rounded-xl px-3 py-2 text-sm font-medium text-slate-600 hover:bg-slate-50 dark:text-muted-foreground dark:hover:bg-white/10">
              <ExternalLink className="h-4 w-4" />
              View All Notifications
            </button>
          </div>
        </>
      )}
    </div>
  )
}

async function execMarkAllRead(
  setNotifications: React.Dispatch<React.SetStateAction<Notification[]>>,
  setCount: React.Dispatch<React.SetStateAction<number>>,
) {
  const result = await markAllAsReadAction()
  if (result.success === true) {
    setNotifications(prev => prev.map(n => ({ ...n, read: true })))
    setCount(0)
    toast({ title: 'All notifications marked as read' })
  } else {
    toast({ title: 'Failed to mark all as read', variant: 'destructive' })
  }
}

function useNotificationBellLogic(isOnAuthPage: boolean) {
  const [count, setCount] = useState(0)
  const [notifications, setNotifications] = useState<Notification[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const { refreshSession } = useSharedAuth()

  const fetchNotifications = useCallback(async () => {
    try {
      setLoading(true)
      const data = await getAdminNotificationsAction({
        page: 1,
        limit: MAX_DROPDOWN_NOTIFICATIONS,
        status: 'all',
      })

      if (data.success === true && data.data?.notifications !== undefined) {
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
        setCount(data.data.unread_count)
        setError(null)
      } else {
        setNotifications([])
        setCount(0)
      }
    } catch {
      setNotifications([])
      setCount(0)
      setError('Failed to load notifications')
    } finally {
      setLoading(false)
    }
  }, [])

  const { isConnected: _sseConnected } = useSSENotifications({
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
  }, [isOnAuthPage, fetchNotifications])

  const handleToggleRead = useCallback(async (notificationId: string, currentlyRead: boolean) => {
    // Optimistic update
    const newRead = !currentlyRead
    setNotifications(prev => prev.map(n => n.id === notificationId ? { ...n, read: newRead } : n))
    setCount(prev => newRead ? Math.max(0, prev - 1) : prev + 1)

    try {
      const result = currentlyRead
        ? await markAsUnreadAction(notificationId)
        : await markAsReadAction(notificationId)

      if (result.success !== true) {
        // Revert on failure — reverse the optimistic change
        setNotifications(prev => prev.map(n => n.id === notificationId ? { ...n, read: currentlyRead } : n))
        setCount(prev => currentlyRead ? prev + 1 : Math.max(0, prev - 1))
        toast({ title: 'Failed to update notification', variant: 'destructive' })
      }
    } catch {
      // Revert on error
      setNotifications(prev => prev.map(n => n.id === notificationId ? { ...n, read: currentlyRead } : n))
      setCount(prev => currentlyRead ? prev + 1 : Math.max(0, prev - 1))
      toast({ title: 'Failed to update notification', variant: 'destructive' })
    }
  }, [])

  const handleDeleteNotification = useCallback(async (e: React.MouseEvent, notificationId: string) => {
    e.stopPropagation()
    const notification = notifications.find(n => n.id === notificationId)
    try {
      const result = await deleteAdminNotificationAction(notificationId)
      if (result.success === true) {
        setNotifications(prev => prev.filter(n => n.id !== notificationId))
        if (notification !== undefined && !notification.read) {
          setCount(prev => Math.max(0, prev - 1))
        }
        toast({ title: 'Notification deleted' })
      } else {
        toast({ title: 'Failed to delete notification', variant: 'destructive' })
      }
    } catch {
      toast({ title: 'Failed to delete notification', variant: 'destructive' })
    }
  }, [notifications])

  const handleMarkAllAsRead = useCallback(async () => {
    try {
      await execMarkAllRead(setNotifications, setCount)
    } catch {
      toast({ title: 'Failed to mark all as read', variant: 'destructive' })
    }
  }, [])

  return { count, notifications, loading, error, fetchNotifications, handleToggleRead, handleDeleteNotification, handleMarkAllAsRead }
}

export function AdminNotificationBell() {
  const pathname = usePathname()
  const router = useRouter()
  const [isOpen, setIsOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)
  const isOnAuthPage = pathname === '/auth' || pathname.startsWith('/auth')

  const { count, notifications, loading, error, fetchNotifications, handleToggleRead, handleDeleteNotification, handleMarkAllAsRead } =
    useNotificationBellLogic(isOnAuthPage)

  useEffect(() => {
    if (!isOpen) { return; }
    const handler = (e: MouseEvent) => {
      if (containerRef.current !== null && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false)
      }
    }
    document.addEventListener('mousedown', handler)
    return () => { document.removeEventListener('mousedown', handler) }
  }, [isOpen])

  return (
    <div className="relative" ref={containerRef}>
      <button
        onClick={() => setIsOpen(prev => !prev)}
        className="relative flex items-center gap-2 rounded-2xl px-3 py-2 text-sm font-medium text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-muted-foreground dark:hover:bg-white/10 dark:hover:text-slate-200"
      >
        <Bell className="h-4 w-4 text-orange-500" />
        {count > 0 && (
          <span className="absolute -top-1 -right-1 flex h-5 w-5 items-center justify-center rounded-full bg-red-500 text-xs text-white">
            {count > 9 ? '9+' : count}
          </span>
        )}
      </button>

      {isOpen && (
        <NotificationDropdown
          loading={loading}
          error={error}
          notifications={notifications}
          count={count}
          onRefresh={() => { void fetchNotifications() }}
          onToggleRead={(id, read) => { void handleToggleRead(id, read) }}
          onDelete={(e, id) => { void handleDeleteNotification(e, id) }}
          onMarkAllRead={() => { void handleMarkAllAsRead() }}
          onViewAll={() => { router.push('/notifications'); setIsOpen(false) }}
        />
      )}
    </div>
  )
}
