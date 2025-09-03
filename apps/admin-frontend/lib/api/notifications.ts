import { cookies } from 'next/headers'

// Re-export shared types
export interface Notification {
  id: string
  title: string
  body: string
  type: 'system' | 'admin' | 'data' | 'feature' | 'security'
  priority: 'low' | 'normal' | 'high' | 'urgent'
  sender: 'system' | 'admin' | 'automated'
  targetAudience: 'all_users' | 'admin_users' | 'premium_users' | 'specific_user'
  imageUrl?: string
  actionUrl?: string
  customData?: Record<string, any>
  createdAt: string
  readAt?: string
  clickedAt?: string
  deliveredAt?: string
  expiresAt?: string
}

export interface AdminNotificationStats {
  totalSent: number
  delivered: number
  failed: number
  pending: number
  successRate: number
  todaysSent: number
  todaysDelivered: number
  avgDeliveryTime: number
  peakHour: string
}

export interface RecentNotification {
  id: string
  title: string
  body: string
  target: string
  sentAt: string
  recipientCount: number
  deliveryStatus: 'sent' | 'delivering' | 'delivered' | 'failed'
  priority: string
  type: string
}

// Helper to get server auth token
async function getServerAuthToken(): Promise<string | null> {
  const cookieStore = await cookies()
  const accessToken = cookieStore.get('access_token')
  return accessToken?.value || null
}

// Admin API functions for server components
export async function getAdminNotificationStats(): Promise<AdminNotificationStats> {
  try {
    const token = await getServerAuthToken()
    if (!token) return {
      totalSent: 0,
      delivered: 0,
      failed: 0,
      pending: 0,
      successRate: 0,
      todaysSent: 0,
      todaysDelivered: 0,
      avgDeliveryTime: 0,
      peakHour: '14:00-15:00'
    }

    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/admin/notifications/stats`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      },
      next: { revalidate: 60 } // Cache for 1 minute for admin stats
    })

    if (!response.ok) {
      console.error('Failed to fetch admin notification stats:', response.statusText)
      return {
        totalSent: 0,
        delivered: 0,
        failed: 0,
        pending: 0,
        successRate: 0,
        todaysSent: 0,
        todaysDelivered: 0,
        avgDeliveryTime: 0,
        peakHour: '14:00-15:00'
      }
    }

    return await response.json()
  } catch (error) {
    console.error('Error fetching admin notification stats:', error)
    return {
      totalSent: 0,
      delivered: 0,
      failed: 0,
      pending: 0,
      successRate: 0,
      todaysSent: 0,
      todaysDelivered: 0,
      avgDeliveryTime: 0,
      peakHour: '14:00-15:00'
    }
  }
}

export async function getRecentNotifications(limit = 10): Promise<RecentNotification[]> {
  try {
    const token = await getServerAuthToken()
    if (!token) return []

    const response = await fetch(
      `${process.env.BACKEND_URL}/api/v1/admin/notifications/recent?limit=${limit}`,
      {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        next: { revalidate: 30 } // Cache for 30 seconds for recent notifications
      }
    )

    if (!response.ok) {
      console.error('Failed to fetch recent notifications:', response.statusText)
      return []
    }

    const data = await response.json()
    return data.notifications || []
  } catch (error) {
    console.error('Error fetching recent notifications:', error)
    return []
  }
}

export async function getAdminUnreadNotifications(): Promise<Notification[]> {
  try {
    const token = await getServerAuthToken()
    if (!token) return []

    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/admin/notifications/unread`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      },
      cache: 'no-store' // Always fresh for admin notifications
    })

    if (!response.ok) {
      console.error('Failed to fetch admin unread notifications:', response.statusText)
      return []
    }

    const data = await response.json()
    return data.notifications || []
  } catch (error) {
    console.error('Error fetching admin unread notifications:', error)
    return []
  }
}

export async function getNotificationHistory(limit = 50, offset = 0): Promise<{
  notifications: Notification[]
  totalCount: number
}> {
  try {
    const token = await getServerAuthToken()
    if (!token) return { notifications: [], totalCount: 0 }

    const response = await fetch(
      `${process.env.BACKEND_URL}/api/v1/admin/notifications/history?limit=${limit}&offset=${offset}`,
      {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        next: { revalidate: 120 } // Cache for 2 minutes
      }
    )

    if (!response.ok) {
      console.error('Failed to fetch notification history:', response.statusText)
      return { notifications: [], totalCount: 0 }
    }

    return await response.json()
  } catch (error) {
    console.error('Error fetching notification history:', error)
    return { notifications: [], totalCount: 0 }
  }
}