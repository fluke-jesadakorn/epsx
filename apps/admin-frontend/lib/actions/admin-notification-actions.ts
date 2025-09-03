'use server'

import { revalidatePath } from 'next/cache'
import { cookies } from 'next/headers'

// Helper to get server auth token
async function getServerAuthToken(): Promise<string | null> {
  const cookieStore = await cookies()
  const accessToken = cookieStore.get('access_token')
  return accessToken?.value || null
}

// Interface for sending notifications
interface SendNotificationPayload {
  title: string
  body: string
  target: 'all_users' | 'admin_users' | 'premium_users' | 'specific_user'
  targetUserId?: string
  priority: 'low' | 'normal' | 'high' | 'urgent'
  type: 'system' | 'admin' | 'data' | 'feature' | 'security'
  imageUrl?: string
  actionUrl?: string
  customData?: Record<string, any>
  scheduledAt?: string
}

// Send notification to users
export async function sendNotification(payload: SendNotificationPayload) {
  try {
    const token = await getServerAuthToken()
    if (!token) {
      throw new Error('Authentication required')
    }

    // Validate required fields
    if (!payload.title.trim() || !payload.body.trim()) {
      throw new Error('Title and body are required')
    }

    // Prepare the request payload
    const requestPayload = {
      title: payload.title.trim(),
      body: payload.body.trim(),
      notification_type: payload.type,
      priority: payload.priority,
      channels: ['fcm_push', 'in_app'],
      ...(payload.imageUrl && { image_url: payload.imageUrl }),
      ...(payload.actionUrl && { action_url: payload.actionUrl }),
      ...(payload.customData && { data_payload: payload.customData }),
      ...(payload.scheduledAt && { scheduled_at: payload.scheduledAt }),
      ...(payload.target === 'specific_user' && payload.targetUserId
        ? { recipient_user_id: payload.targetUserId }
        : { fcm_topic_id: payload.target }
      )
    }

    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/admin/notifications/send`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(requestPayload)
    })

    if (!response.ok) {
      const errorData = await response.text()
      throw new Error(`Failed to send notification: ${response.statusText} - ${errorData}`)
    }

    const result = await response.json()

    // Revalidate admin pages to show updated stats
    revalidatePath('/admin/notifications')
    revalidatePath('/notifications') // In case admin is viewing user notifications
    
    return { 
      success: true, 
      data: result,
      recipientCount: result.recipient_count || 1
    }
  } catch (error) {
    console.error('Error sending notification:', error)
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Unknown error' 
    }
  }
}

// Broadcast notification to topic
export async function broadcastNotification(payload: {
  topic: string
  title: string
  body: string
  priority: 'low' | 'normal' | 'high' | 'urgent'
  data?: Record<string, any>
}) {
  try {
    const token = await getServerAuthToken()
    if (!token) {
      throw new Error('Authentication required')
    }

    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/admin/notifications/broadcast`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(payload)
    })

    if (!response.ok) {
      throw new Error('Failed to broadcast notification')
    }

    const result = await response.json()

    // Revalidate admin pages
    revalidatePath('/admin/notifications')
    
    return { success: true, data: result }
  } catch (error) {
    console.error('Error broadcasting notification:', error)
    return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
  }
}

// Schedule notification for future delivery
export async function scheduleNotification(payload: SendNotificationPayload) {
  try {
    const token = await getServerAuthToken()
    if (!token) {
      throw new Error('Authentication required')
    }

    if (!payload.scheduledAt) {
      throw new Error('Scheduled time is required')
    }

    // Use the same sendNotification function but with scheduledAt
    return await sendNotification(payload)
  } catch (error) {
    console.error('Error scheduling notification:', error)
    return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
  }
}

// Cancel scheduled notification
export async function cancelScheduledNotification(notificationId: string) {
  try {
    const token = await getServerAuthToken()
    if (!token) {
      throw new Error('Authentication required')
    }

    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/admin/notifications/${notificationId}/cancel`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      }
    })

    if (!response.ok) {
      throw new Error('Failed to cancel scheduled notification')
    }

    // Revalidate admin pages
    revalidatePath('/admin/notifications')
    
    return { success: true }
  } catch (error) {
    console.error('Error canceling scheduled notification:', error)
    return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
  }
}

// Get notification analytics
export async function getNotificationAnalytics(timeRange: '24h' | '7d' | '30d' = '7d') {
  try {
    const token = await getServerAuthToken()
    if (!token) {
      throw new Error('Authentication required')
    }

    const response = await fetch(
      `${process.env.BACKEND_URL}/api/v1/admin/notifications/analytics?range=${timeRange}`,
      {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        next: { revalidate: 300 } // Cache for 5 minutes
      }
    )

    if (!response.ok) {
      throw new Error('Failed to fetch notification analytics')
    }

    const analytics = await response.json()
    return { success: true, data: analytics }
  } catch (error) {
    console.error('Error fetching notification analytics:', error)
    return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
  }
}

// Update system notification settings
export async function updateSystemNotificationSettings(settings: {
  maxNotificationsPerHour?: number
  enableQuietHours?: boolean
  quietHoursStart?: string
  quietHoursEnd?: string
  enableRateLimiting?: boolean
  defaultExpiryHours?: number
}) {
  try {
    const token = await getServerAuthToken()
    if (!token) {
      throw new Error('Authentication required')
    }

    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/admin/notifications/settings`, {
      method: 'PUT',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(settings)
    })

    if (!response.ok) {
      throw new Error('Failed to update system notification settings')
    }

    // Revalidate admin settings page
    revalidatePath('/admin/notifications/settings')
    
    return { success: true }
  } catch (error) {
    console.error('Error updating system notification settings:', error)
    return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
  }
}

// Export notification data
export async function exportNotificationData(format: 'csv' | 'json', dateRange: {
  start: string
  end: string
}) {
  try {
    const token = await getServerAuthToken()
    if (!token) {
      throw new Error('Authentication required')
    }

    const response = await fetch(
      `${process.env.BACKEND_URL}/api/v1/admin/notifications/export?format=${format}&start=${dateRange.start}&end=${dateRange.end}`,
      {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        }
      }
    )

    if (!response.ok) {
      throw new Error('Failed to export notification data')
    }

    const data = await response.blob()
    return { success: true, data }
  } catch (error) {
    console.error('Error exporting notification data:', error)
    return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
  }
}