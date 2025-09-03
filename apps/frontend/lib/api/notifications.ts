import { cookies } from 'next/headers'

// TypeScript interfaces for notifications
export interface Notification {
  id: string
  title: string
  body: string
  type: 'system' | 'admin' | 'data' | 'feature' | 'security'
  priority: 'low' | 'normal' | 'high' | 'urgent'
  sender: 'system' | 'admin' | 'automated'
  imageUrl?: string
  actionUrl?: string
  customData?: Record<string, any>
  createdAt: string
  readAt?: string
  clickedAt?: string
  deliveredAt?: string
  expiresAt?: string
}

export interface NotificationStats {
  totalSent: number
  delivered: number
  failed: number
  pending: number
  successRate: number
  unreadCount: number
}

export interface NotificationPreferences {
  pushEnabled: boolean
  inAppEnabled: boolean
  emailEnabled: boolean
  quietHoursStart?: string
  quietHoursEnd?: string
  blockedCategories: string[]
  soundEnabled: boolean
  vibrationEnabled: boolean
}

// Helper to get server auth token
async function getServerAuthToken(): Promise<string | null> {
  const cookieStore = await cookies()
  const accessToken = cookieStore.get('access_token')
  return accessToken?.value || null
}

// API functions for server components
export async function getUnreadNotifications(): Promise<Notification[]> {
  try {
    const token = await getServerAuthToken()
    if (!token) return []

    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/notifications/unread`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      },
      cache: 'no-store' // Always fresh for notifications
    })

    if (!response.ok) {
      console.error('Failed to fetch unread notifications:', response.statusText)
      return []
    }

    const data = await response.json()
    return data.notifications || []
  } catch (error) {
    console.error('Error fetching unread notifications:', error)
    return []
  }
}

export async function getUserNotifications(limit = 50, offset = 0): Promise<{
  notifications: Notification[]
  totalCount: number
  unreadCount: number
}> {
  try {
    const token = await getServerAuthToken()
    if (!token) return { notifications: [], totalCount: 0, unreadCount: 0 }

    const response = await fetch(
      `${process.env.BACKEND_URL}/api/v1/notifications?limit=${limit}&offset=${offset}`,
      {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        cache: 'no-store' // Always fresh for notifications
      }
    )

    if (!response.ok) {
      console.error('Failed to fetch user notifications:', response.statusText)
      return { notifications: [], totalCount: 0, unreadCount: 0 }
    }

    const data = await response.json()
    
    // Map backend response to frontend format
    return {
      notifications: data.notifications.map((notification: any) => ({
        ...notification,
        type: notification.notification_type,
        createdAt: notification.created_at,
        readAt: notification.read_at,
        clickedAt: notification.clicked_at,
        deliveredAt: notification.delivered_at,
        expiresAt: notification.expires_at,
        imageUrl: notification.image_url,
        actionUrl: notification.action_url,
        customData: notification.data_payload
      })),
      totalCount: data.total_count,
      unreadCount: data.unread_count
    }
  } catch (error) {
    console.error('Error fetching user notifications:', error)
    return { notifications: [], totalCount: 0, unreadCount: 0 }
  }
}

export async function getNotificationStats(): Promise<NotificationStats> {
  try {
    const token = await getServerAuthToken()
    if (!token) return {
      totalSent: 0,
      delivered: 0, 
      failed: 0,
      pending: 0,
      successRate: 0,
      unreadCount: 0
    }

    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/notifications/stats`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      },
      next: { revalidate: 300 } // Cache for 5 minutes
    })

    if (!response.ok) {
      console.error('Failed to fetch notification stats:', response.statusText)
      return {
        totalSent: 0,
        delivered: 0,
        failed: 0, 
        pending: 0,
        successRate: 0,
        unreadCount: 0
      }
    }

    return await response.json()
  } catch (error) {
    console.error('Error fetching notification stats:', error)
    return {
      totalSent: 0,
      delivered: 0,
      failed: 0,
      pending: 0, 
      successRate: 0,
      unreadCount: 0
    }
  }
}

export async function getNotificationPreferences(): Promise<NotificationPreferences> {
  try {
    const token = await getServerAuthToken()
    if (!token) return {
      pushEnabled: true,
      inAppEnabled: true,
      emailEnabled: true,
      blockedCategories: [],
      soundEnabled: true,
      vibrationEnabled: true
    }

    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/notifications/preferences`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      },
      next: { revalidate: 600 } // Cache for 10 minutes
    })

    if (!response.ok) {
      console.error('Failed to fetch notification preferences:', response.statusText)
      return {
        pushEnabled: true,
        inAppEnabled: true,
        emailEnabled: true,
        blockedCategories: [],
        soundEnabled: true,
        vibrationEnabled: true
      }
    }

    return await response.json()
  } catch (error) {
    console.error('Error fetching notification preferences:', error)
    return {
      pushEnabled: true,
      inAppEnabled: true,
      emailEnabled: true,
      blockedCategories: [],
      soundEnabled: true,
      vibrationEnabled: true
    }
  }
}