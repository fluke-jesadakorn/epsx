'use client'

import { useState, useEffect } from 'react'
import Link from 'next/link'
import { Bell } from 'lucide-react'
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

interface SimpleNotification {
  id: string
  title: string
  body: string
  type: string
  priority: string
  createdAt: string
  readAt?: string
  actionUrl?: string
}

interface NotificationBellSimpleProps {
  className?: string
  showBadge?: boolean
}

function NotificationCard({ notification }: { notification: SimpleNotification }) {
  const typeIcons = {
    system: '🔧',
    admin: '👨‍💼',
    data: '📊', 
    feature: '✨',
    security: '🔒'
  }

  const priorityColors = {
    urgent: 'border-red-500 bg-red-50 dark:bg-red-950',
    high: 'border-orange-500 bg-orange-50 dark:bg-orange-950', 
    normal: 'border-blue-500 bg-blue-50 dark:bg-blue-950',
    low: 'border-gray-500 bg-gray-50 dark:bg-gray-950'
  }

  const timeAgo = (dateString: string) => {
    const date = new Date(dateString)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    
    if (diffMins < 1) return 'Just now'
    if (diffMins < 60) return `${diffMins}m ago`
    const diffHours = Math.floor(diffMs / 3600000)
    if (diffHours < 24) return `${diffHours}h ago`
    return date.toLocaleDateString()
  }

  return (
    <div className={`p-3 rounded-lg border-l-4 ${priorityColors[notification.priority as keyof typeof priorityColors]} transition-colors hover:bg-opacity-80`}>
      <div className="flex items-start gap-2">
        <span className="text-lg">{typeIcons[notification.type as keyof typeof typeIcons] || '📄'}</span>
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
          {!notification.readAt && (
            <Badge variant="destructive" className="text-xs mt-2">
              NEW
            </Badge>
          )}
        </div>
      </div>
    </div>
  )
}

export function NotificationBellSimple({ className = "", showBadge = true }: NotificationBellSimpleProps) {
  const [notifications, setNotifications] = useState<SimpleNotification[]>([])
  const [unreadCount, setUnreadCount] = useState(0)
  const [isOpen, setIsOpen] = useState(false)
  const [loading, setLoading] = useState(true)
  const [isMobile, setIsMobile] = useState(false)
  const [isMounted, setIsMounted] = useState(false)

  console.log('🔔 NotificationBellSimple component mounted')

  // Prevent hydration mismatch by only checking mobile after mount
  useEffect(() => {
    setIsMounted(true)
    const checkMobile = () => setIsMobile(window.innerWidth < 768)
    checkMobile()
    window.addEventListener('resize', checkMobile)
    return () => window.removeEventListener('resize', checkMobile)
  }, [])

  // Fetch notifications
  useEffect(() => {
    const fetchNotifications = async () => {
      try {
        // Get access token from cookies (same way as server components)
        const getCookie = (name: string) => {
          const value = `; ${document.cookie}`;
          const parts = value.split(`; ${name}=`);
          if (parts.length === 2) return parts.pop()?.split(';').shift();
        }
        
        const accessToken = getCookie('access_token')
        console.log('🔔 Notification bell: Fetching notifications, token:', accessToken ? 'Found' : 'Missing')
        if (!accessToken) {
          console.log('No access token available for notifications')
          setLoading(false)
          return
        }

        const response = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'}/api/v1/notifications/unread`, {
          headers: {
            'Authorization': `Bearer ${accessToken}`,
            'Content-Type': 'application/json'
          }
        })
        
        if (response.ok) {
          const data = await response.json()
          // Map backend format to frontend format
          const mappedNotifications = data.notifications.map((notification: any) => ({
            ...notification,
            type: notification.notification_type,
            createdAt: notification.created_at,
            readAt: notification.read_at,
            actionUrl: notification.action_url
          }))
          setNotifications(mappedNotifications || [])
          setUnreadCount(mappedNotifications?.length || 0)
        } else {
          console.error('Failed to fetch notifications:', response.statusText)
        }
      } catch (error) {
        console.error('Failed to fetch notifications:', error)
      } finally {
        setLoading(false)
      }
    }

    fetchNotifications()
    
    // Poll for new notifications every 30 seconds
    const interval = setInterval(fetchNotifications, 30000)
    return () => clearInterval(interval)
  }, [])

  const NotificationContent = () => (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="font-semibold text-lg">Notifications</h3>
        {unreadCount > 0 && (
          <Badge variant="secondary" className="text-xs">
            {unreadCount} new
          </Badge>
        )}
      </div>

      {loading ? (
        <div className="text-center py-8 text-gray-500 dark:text-gray-400">
          <div className="animate-spin rounded-full h-6 w-6 border-2 border-gray-300 border-t-blue-600 mx-auto mb-2"></div>
          <p className="text-sm">Loading...</p>
        </div>
      ) : notifications.length === 0 ? (
        <div className="text-center py-8 text-gray-500 dark:text-gray-400">
          <Bell className="h-8 w-8 mx-auto mb-2 opacity-50" />
          <p className="text-sm">No new notifications</p>
        </div>
      ) : (
        <div className="space-y-3 max-h-80 overflow-y-auto">
          {notifications.slice(0, 5).map((notification) => (
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

  const bellButton = (
    <Button
      variant="ghost"
      size="icon"
      className={`relative h-10 w-10 rounded-full hover:bg-primary/20 hover:text-primary transition-all duration-200 text-gray-700 dark:text-gray-300 hover:scale-105 active:scale-95 ${className}`}
    >
      <Bell className="h-5 w-5 stroke-2" />
      {showBadge && unreadCount > 0 && (
        <Badge 
          className="absolute -top-1 -right-1 h-5 w-5 rounded-full p-0 flex items-center justify-center text-xs min-w-[20px] font-bold shadow-lg animate-pulse"
          variant="destructive"
        >
          {unreadCount > 99 ? '99+' : unreadCount}
        </Badge>
      )}
    </Button>
  )

  // Return consistent markup during hydration, then switch to responsive after mount
  if (!isMounted) {
    // Default to desktop view during hydration to prevent mismatch
    return (
      <Popover open={isOpen} onOpenChange={setIsOpen}>
        <PopoverTrigger asChild>
          {bellButton}
        </PopoverTrigger>
        <PopoverContent align="end" className="w-96 p-4">
          <NotificationContent />
        </PopoverContent>
      </Popover>
    )
  }

  // Mobile view with Sheet (after mount)
  if (isMobile) {
    return (
      <Sheet open={isOpen} onOpenChange={setIsOpen}>
        <SheetTrigger asChild>
          {bellButton}
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
        {bellButton}
      </PopoverTrigger>
      <PopoverContent align="end" className="w-96 p-4">
        <NotificationContent />
      </PopoverContent>
    </Popover>
  )
}