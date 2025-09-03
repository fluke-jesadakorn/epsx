'use client'

import { useState } from 'react'
import Link from 'next/link'
import { Bell, X } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { 
  Sheet, 
  SheetContent, 
  SheetHeader, 
  SheetTitle, 
  SheetTrigger 
} from '@/components/ui/sheet'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover'
import { Notification } from '@/lib/api/notifications'

interface NotificationBellClientProps {
  count: number
  recentNotifications: Notification[]
}

function NotificationCard({ notification }: { notification: Notification }) {
  const priorityColors = {
    urgent: 'border-red-500 bg-red-50 dark:bg-red-950',
    high: 'border-orange-500 bg-orange-50 dark:bg-orange-950', 
    normal: 'border-blue-500 bg-blue-50 dark:bg-blue-950',
    low: 'border-gray-500 bg-gray-50 dark:bg-gray-950'
  }

  const typeIcons = {
    system: '🔧',
    admin: '👨‍💼',
    data: '📊', 
    feature: '✨',
    security: '🔒'
  }

  const timeAgo = (dateString: string) => {
    const date = new Date(dateString)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMs / 3600000)
    
    if (diffMins < 1) return 'Just now'
    if (diffMins < 60) return `${diffMins}m ago`
    if (diffHours < 24) return `${diffHours}h ago`
    return date.toLocaleDateString()
  }

  return (
    <div className={`p-3 rounded-lg border-l-4 ${priorityColors[notification.priority]} transition-colors hover:bg-opacity-80`}>
      <div className="flex items-start gap-2">
        <span className="text-lg">{typeIcons[notification.type]}</span>
        <div className="flex-1 min-w-0">
          <div className="flex items-start justify-between gap-2">
            <h4 className="font-medium text-sm line-clamp-2 text-gray-900 dark:text-gray-100">
              {notification.title}
            </h4>
            <span className="text-xs text-gray-500 dark:text-gray-400 flex-shrink-0">
              {timeAgo(notification.createdAt)}
            </span>
          </div>
          <p className="text-sm text-gray-600 dark:text-gray-300 line-clamp-2 mt-1">
            {notification.body}
          </p>
          {notification.actionUrl && (
            <Link 
              href={notification.actionUrl}
              className="text-xs text-blue-600 dark:text-blue-400 hover:underline mt-2 inline-block"
            >
              View Details →
            </Link>
          )}
        </div>
      </div>
    </div>
  )
}

export function NotificationBellClient({ count, recentNotifications }: NotificationBellClientProps) {
  const [isOpen, setIsOpen] = useState(false)

  // Mobile: Sheet, Desktop: Popover
  const isMobile = typeof window !== 'undefined' && window.innerWidth < 768

  const NotificationContent = () => (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="font-semibold text-lg">Notifications</h3>
        {count > 0 && (
          <Badge variant="secondary" className="text-xs">
            {count} new
          </Badge>
        )}
      </div>

      {recentNotifications.length === 0 ? (
        <div className="text-center py-8 text-gray-500 dark:text-gray-400">
          <Bell className="h-8 w-8 mx-auto mb-2 opacity-50" />
          <p className="text-sm">No new notifications</p>
        </div>
      ) : (
        <div className="space-y-3 max-h-80 overflow-y-auto">
          {recentNotifications.map((notification) => (
            <NotificationCard key={notification.id} notification={notification} />
          ))}
        </div>
      )}

      <div className="border-t pt-3 flex justify-between items-center">
        <Link 
          href="/notifications"
          className="text-sm text-blue-600 dark:text-blue-400 hover:underline"
          onClick={() => setIsOpen(false)}
        >
          View All Notifications
        </Link>
        <Link 
          href="/notifications/preferences"
          className="text-sm text-gray-600 dark:text-gray-400 hover:underline"
          onClick={() => setIsOpen(false)}
        >
          ⚙️ Settings
        </Link>
      </div>
    </div>
  )

  // Mobile view with Sheet
  if (isMobile) {
    return (
      <Sheet open={isOpen} onOpenChange={setIsOpen}>
        <SheetTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className="relative h-10 w-10 rounded-full hover:bg-gray-100 dark:hover:bg-gray-800"
          >
            <Bell className="h-5 w-5" />
            {count > 0 && (
              <Badge 
                className="absolute -top-1 -right-1 h-5 w-5 rounded-full p-0 flex items-center justify-center text-xs min-w-[20px]"
                variant="destructive"
              >
                {count > 99 ? '99+' : count}
              </Badge>
            )}
          </Button>
        </SheetTrigger>
        <SheetContent side="right" className="w-full sm:w-96">
          <SheetHeader>
            <SheetTitle>Notifications</SheetTitle>
          </SheetHeader>
          <div className="mt-4">
            <NotificationContent />
          </div>
        </SheetContent>
      </Sheet>
    )
  }

  // Desktop view with Popover
  return (
    <Popover open={isOpen} onOpenChange={setIsOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="relative h-10 w-10 rounded-full hover:bg-gray-100 dark:hover:bg-gray-800"
        >
          <Bell className="h-5 w-5" />
          {count > 0 && (
            <Badge 
              className="absolute -top-1 -right-1 h-5 w-5 rounded-full p-0 flex items-center justify-center text-xs min-w-[20px]"
              variant="destructive"
            >
              {count > 99 ? '99+' : count}
            </Badge>
          )}
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-96 p-4">
        <NotificationContent />
      </PopoverContent>
    </Popover>
  )
}