/**
 * Client-side Notification API
 * For use in client components that can't access server-side functions
 */

// Base configuration
const API_BASE_URL = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'

// Types for client-side notifications
export interface Notification {
  id: string
  type: string
  title: string
  message: string
  is_read: boolean
  created_at: string
  metadata?: Record<string, any>
}

// Client-side notification API that uses cookies directly
export const ClientNotificationAPI = {
  // Get auth token from cookies for client-side use
  getAuthToken(): string | null {
    if (typeof document === 'undefined') return null
    
    const cookies = document.cookie.split(';')
    const authCookie = cookies.find(cookie => 
      cookie.trim().startsWith('epsx_admin_jwt=') || 
      cookie.trim().startsWith('access_token=')
    )
    
    if (!authCookie) return null
    
    return authCookie.split('=')[1]
  },

  async getUnreadCount(): Promise<{ count: number }> {
    try {
      const token = this.getAuthToken()
      if (!token) {
        console.error('No auth token available for unread count')
        return { count: 0 }
      }

      const response = await fetch(`${API_BASE_URL}/api/v1/notifications/unread-count`, {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) {
        console.error(`Failed to fetch unread count: ${response.status}`)
        return { count: 0 }
      }

      const data = await response.json()
      return { count: data.unread_count || 0 }
    } catch (error) {
      console.error('Failed to fetch unread count:', error)
      return { count: 0 }
    }
  },

  async getNotifications(offset: number = 0, limit: number = 20): Promise<Notification[]> {
    try {
      const token = this.getAuthToken()
      if (!token) {
        console.error('No auth token available for notifications')
        return []
      }

      const page = Math.floor(offset / limit) + 1
      const response = await fetch(`${API_BASE_URL}/api/v1/notifications?page=${page}&per_page=${limit}`, {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) {
        console.error(`Failed to fetch notifications: ${response.status}`)
        return []
      }

      const data = await response.json()
      
      // Convert backend format to our client format
      return data.notifications?.map((notification: any) => ({
        id: notification.id,
        type: notification.notification_type,
        title: notification.title,
        message: notification.message,
        is_read: notification.status === 'read',
        created_at: notification.created_at,
        metadata: notification.metadata || {}
      })) || []
    } catch (error) {
      console.error('Failed to fetch notifications:', error)
      return []
    }
  },

  async markAsRead(notificationId: string): Promise<void> {
    try {
      const token = this.getAuthToken()
      if (!token) {
        console.error('No auth token available for mark as read')
        return
      }

      const response = await fetch(`${API_BASE_URL}/api/v1/notifications/read/${notificationId}`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) {
        throw new Error(`Failed to mark notification as read: ${response.status}`)
      }

      console.log(`Successfully marked notification ${notificationId} as read`)
    } catch (error) {
      console.error('Failed to mark notification as read:', error)
      throw error
    }
  },

  async markAllAsRead(): Promise<void> {
    try {
      const token = this.getAuthToken()
      if (!token) {
        console.error('No auth token available for mark all as read')
        return
      }

      const response = await fetch(`${API_BASE_URL}/api/v1/notifications/read-all`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          mark_all: true,
          notification_ids: []
        })
      })

      if (!response.ok) {
        throw new Error(`Failed to mark all notifications as read: ${response.status}`)
      }

      console.log('Successfully marked all notifications as read')
    } catch (error) {
      console.error('Failed to mark all notifications as read:', error)
      throw error
    }
  }
}