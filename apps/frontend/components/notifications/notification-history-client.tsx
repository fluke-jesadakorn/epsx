'use client'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import type { Notification } from '@/lib/api'
import { Filter, RefreshCw, Search, Settings } from 'lucide-react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { useMemo, useState, useTransition } from 'react'
import { toast } from 'sonner'
// TODO: Implement these functions when backend is ready
const markAllNotificationsRead = async () => {
};

const markNotificationRead = async (notificationId: string) => {
};

interface NotificationHistoryClientProps {
  initialNotifications: Notification[]
  totalCount: number
  unreadCount: number
}

function NotificationCard({ notification, onMarkAsRead, onCardClick }: {
  notification: Notification
  onMarkAsRead?: (id: string) => void
  onCardClick?: (id: string) => void
}) {
  const priorityColors: Record<string, string> = {
    critical: 'border-red-500 bg-red-50 dark:bg-red-950 text-red-900 dark:text-red-100',
    urgent: 'border-red-500 bg-red-50 dark:bg-red-950 text-red-900 dark:text-red-100',
    high: 'border-orange-500 bg-orange-50 dark:bg-orange-950 text-orange-900 dark:text-orange-100',
    normal: 'border-blue-500 bg-blue-50 dark:bg-blue-950 text-blue-900 dark:text-blue-100',
    low: 'border-gray-500 bg-gray-50 dark:bg-gray-950 text-gray-900 dark:text-gray-100'
  }

  const typeIcons: Record<string, string> = {
    system: '🔧',
    admin: '👨‍💼',
    data: '📊',
    feature: '✨',
    security: '🔒'
  }

  const priorityLabels: Record<string, string> = {
    critical: '🔴 CRITICAL',
    urgent: '🔴 URGENT',
    high: '🟡 HIGH',
    normal: '🔵 NORMAL',
    low: '⚪ LOW'
  }

  const timeAgo = (dateString: string) => {
    const date = new Date(dateString)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMs / 3600000)
    const diffDays = Math.floor(diffMs / 86400000)

    if (diffMins < 1) {return 'Just now'}
    if (diffMins < 60) {return `${diffMins}m ago`}
    if (diffHours < 24) {return `${diffHours}h ago`}
    if (diffDays < 7) {return `${diffDays}d ago`}
    return date.toLocaleDateString()
  }

  const handleCardClick = () => {
    if (!notification.readAt && onCardClick) {
      onCardClick(notification.id)
    }
  }

  return (
    <div
      className={`p-4 rounded-lg border-l-4 ${priorityColors[notification.priority]} hover:shadow-md cursor-pointer`}
      onClick={handleCardClick}
    >
      <div className="flex items-start gap-3">
        <span className="text-2xl flex-shrink-0">{typeIcons[notification.type]}</span>
        
        <div className="flex-1 min-w-0">
          <div className="flex items-start justify-between gap-3 mb-2">
            <div className="flex items-center gap-2 flex-wrap">
              <Badge variant="outline" className="text-xs">
                {priorityLabels[notification.priority]}
              </Badge>
              <span className="text-xs text-gray-500 dark:text-gray-400">
                {timeAgo(notification.createdAt)}
              </span>
              {!notification.readAt && (
                <Badge variant="destructive" className="text-xs">
                  NEW
                </Badge>
              )}
            </div>
          </div>

          <h3 className="font-semibold text-lg mb-2 leading-tight">
            {notification.title}
          </h3>
          
          <p className="text-gray-700 dark:text-gray-300 text-sm leading-relaxed mb-3">
            {notification.body}
          </p>

          <div className="flex items-center gap-3">
            {notification.actionUrl && (
              <Link
                href={notification.actionUrl}
                onClick={(e) => e.stopPropagation()}
                className="text-sm bg-blue-600 text-white px-3 py-1 rounded-md hover:bg-blue-700"
              >
                View Details
              </Link>
            )}

            {!notification.readAt && onMarkAsRead && (
              <button
                onClick={(e) => {
                  e.stopPropagation()
                  onMarkAsRead(notification.id)
                }}
                className="text-sm text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400"
              >
                Mark as Read
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export function NotificationHistoryClient({
  initialNotifications,
  totalCount,
  unreadCount
}: NotificationHistoryClientProps) {
  const [notifications] = useState<Notification[]>(initialNotifications)
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedType, setSelectedType] = useState<string>('all')
  const [selectedPriority, setSelectedPriority] = useState<string>('all')
  const [showUnreadOnly, setShowUnreadOnly] = useState(false)
  const [isPending, startTransition] = useTransition()
  const router = useRouter()

  const handleMarkAllAsRead = async () => {
    startTransition(async () => {
      try {
        await markAllNotificationsRead()
        toast.success('All notifications marked as read')
        router.refresh()
      } catch (error) {
        toast.error('Failed to mark all notifications as read')
      }
    })
  }

  const handleClearHistory = async () => {
    if (!confirm('Are you sure you want to clear all notification history? This cannot be undone.')) {
      return
    }
    
    startTransition(async () => {
      // Clear functionality - using deleteNotification for individual notifications
      // For bulk clear, we'd need to call deleteNotification for each notification
      toast.info('Bulk clear not implemented yet')
    })
  }

  const handleMarkAsRead = async (notificationId: string) => {
    startTransition(async () => {
      try {
        await markNotificationRead(notificationId)
        toast.success('Notification marked as read')
        router.refresh()
      } catch (error) {
        toast.error('Failed to mark notification as read')
      }
    })
  }

  // Group notifications by date
  const groupedNotifications = useMemo(() => {
    const filtered = notifications.filter(notification => {
      const matchesSearch = notification.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
                           notification.body.toLowerCase().includes(searchQuery.toLowerCase())
      const matchesType = selectedType === 'all' || notification.type === selectedType
      const matchesPriority = selectedPriority === 'all' || notification.priority === selectedPriority
      const matchesReadStatus = !showUnreadOnly || !notification.readAt

      return matchesSearch && matchesType && matchesPriority && matchesReadStatus
    })

    const groups: { [key: string]: Notification[] } = {}
    filtered.forEach(notification => {
      const date = new Date(notification.createdAt)
      const today = new Date()
      const yesterday = new Date(today)
      yesterday.setDate(yesterday.getDate() - 1)

      let groupKey: string
      if (date.toDateString() === today.toDateString()) {
        groupKey = 'Today'
      } else if (date.toDateString() === yesterday.toDateString()) {
        groupKey = 'Yesterday'
      } else {
        groupKey = date.toLocaleDateString()
      }

      if (!groups[groupKey]) {
        groups[groupKey] = []
      }
      groups[groupKey].push(notification)
    })

    // Sort groups by date
    const sortedGroups = Object.entries(groups).sort(([a], [b]) => {
      if (a === 'Today') {return -1}
      if (b === 'Today') {return 1}
      if (a === 'Yesterday') {return -1}
      if (b === 'Yesterday') {return 1}
      return new Date(b).getTime() - new Date(a).getTime()
    })

    return sortedGroups
  }, [notifications, searchQuery, selectedType, selectedPriority, showUnreadOnly])

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      {/* Stats Header */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-blue-50 dark:bg-blue-950 p-4 rounded-lg">
          <h3 className="font-semibold text-blue-900 dark:text-blue-100">📊 Total</h3>
          <p className="text-2xl font-bold text-blue-600 dark:text-blue-400">{totalCount}</p>
        </div>
        <div className="bg-red-50 dark:bg-red-950 p-4 rounded-lg">
          <h3 className="font-semibold text-red-900 dark:text-red-100">🔔 Unread</h3>
          <p className="text-2xl font-bold text-red-600 dark:text-red-400">{unreadCount}</p>
        </div>
        <div className="bg-green-50 dark:bg-green-950 p-4 rounded-lg">
          <h3 className="font-semibold text-green-900 dark:text-green-100">✓ Read</h3>
          <p className="text-2xl font-bold text-green-600 dark:text-green-400">{totalCount - unreadCount}</p>
        </div>
      </div>

      {/* Search and Filters */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm border border-gray-200 dark:border-gray-700">
        <div className="flex flex-col lg:flex-row gap-4">
          <div className="flex-1">
            <div className="relative">
              <Search className="absolute left-3 top-3 h-4 w-4 text-gray-400" />
              <Input
                placeholder="Search notifications..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-10"
              />
            </div>
          </div>
          
          <div className="flex flex-col sm:flex-row gap-3">
            <Select value={selectedType} onValueChange={setSelectedType}>
              <SelectTrigger className="w-40">
                <SelectValue placeholder="All Types" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Types</SelectItem>
                <SelectItem value="system">🔧 System</SelectItem>
                <SelectItem value="admin">👨‍💼 Admin</SelectItem>
                <SelectItem value="data">📊 Data</SelectItem>
                <SelectItem value="feature">✨ Feature</SelectItem>
                <SelectItem value="security">🔒 Security</SelectItem>
              </SelectContent>
            </Select>

            <Select value={selectedPriority} onValueChange={setSelectedPriority}>
              <SelectTrigger className="w-40">
                <SelectValue placeholder="All Priorities" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Priorities</SelectItem>
                <SelectItem value="critical">🔴 Critical</SelectItem>
                <SelectItem value="high">🟡 High</SelectItem>
                <SelectItem value="normal">🔵 Normal</SelectItem>
                <SelectItem value="low">⚪ Low</SelectItem>
              </SelectContent>
            </Select>

            <Button
              variant={showUnreadOnly ? "default" : "outline"}
              onClick={() => setShowUnreadOnly(!showUnreadOnly)}
              className="whitespace-nowrap"
            >
              <Filter className="h-4 w-4 mr-2" />
              Unread Only
            </Button>
          </div>
        </div>

        <div className="flex justify-between items-center mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
          <div className="flex gap-3">
            <Button 
              variant="outline" 
              size="sm"
              onClick={handleMarkAllAsRead}
              disabled={isPending || unreadCount === 0}
            >
              📝 Mark All as Read
            </Button>
            <Button 
              variant="outline" 
              size="sm"
              onClick={handleClearHistory}
              disabled={isPending || totalCount === 0}
            >
              🗑️ Clear History
            </Button>
          </div>
          <Link href="/notifications/preferences">
            <Button variant="outline" size="sm">
              <Settings className="h-4 w-4 mr-2" />
              Preferences
            </Button>
          </Link>
        </div>
      </div>

      {/* Notifications List */}
      <div className="space-y-6">
        {groupedNotifications.length === 0 ? (
          <div className="text-center py-12 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
            <div className="text-6xl mb-4">📭</div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
              No notifications found
            </h3>
            <p className="text-gray-600 dark:text-gray-400">
              Try adjusting your filters or search terms
            </p>
          </div>
        ) : (
          groupedNotifications.map(([date, notifications]) => (
            <div key={date} className="space-y-3">
              <div className="flex items-center gap-3">
                <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  {date}
                </h2>
                <div className="flex-1 h-px bg-gray-200 dark:bg-gray-700" />
                <Badge variant="outline">
                  {notifications.length} {notifications.length === 1 ? 'notification' : 'notifications'}
                </Badge>
              </div>
              
              <div className="space-y-3">
                {notifications.map((notification) => (
                  <NotificationCard
                    key={notification.id}
                    notification={notification}
                    onMarkAsRead={handleMarkAsRead}
                    onCardClick={handleMarkAsRead}
                  />
                ))}
              </div>
            </div>
          ))
        )}
      </div>

      {/* Load More */}
      {groupedNotifications.length > 0 && (
        <div className="text-center py-6">
          <Button variant="outline">
            <RefreshCw className="h-4 w-4 mr-2" />
            Load More Notifications
          </Button>
        </div>
      )}
    </div>
  )
}