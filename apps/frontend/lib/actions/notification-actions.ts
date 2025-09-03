'use server'

import { revalidatePath } from 'next/cache'
import { cookies } from 'next/headers'

// Helper to get server auth token
async function getServerAuthToken(): Promise<string | null> {
  const cookieStore = await cookies()
  const accessToken = cookieStore.get('access_token')
  return accessToken?.value || null
}

// Mark notification as read
export async function markNotificationAsRead(notificationId: string) {
  try {
    const token = await getServerAuthToken()
    if (!token) {
      throw new Error('Authentication required')
    }

    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/notifications/${notificationId}/read`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      }
    })

    if (!response.ok) {
      throw new Error('Failed to mark notification as read')
    }

    // Revalidate paths to update UI
    revalidatePath('/notifications')
    revalidatePath('/') // For notification bell count
    
    return { success: true }
  } catch (error) {
    console.error('Error marking notification as read:', error)
    return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
  }
}

// Mark all notifications as read
export async function markAllNotificationsAsRead() {
  try {
    const token = await getServerAuthToken()
    if (!token) {
      throw new Error('Authentication required')
    }

    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/notifications/read-all`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      }
    })

    if (!response.ok) {
      throw new Error('Failed to mark all notifications as read')
    }

    // Revalidate paths to update UI
    revalidatePath('/notifications')
    revalidatePath('/') // For notification bell count
    
    return { success: true }
  } catch (error) {
    console.error('Error marking all notifications as read:', error)
    return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
  }
}

// Archive notification
export async function archiveNotification(notificationId: string) {
  try {
    const token = await getServerAuthToken()
    if (!token) {
      throw new Error('Authentication required')
    }

    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/notifications/${notificationId}/archive`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      }
    })

    if (!response.ok) {
      throw new Error('Failed to archive notification')
    }

    // Revalidate paths to update UI
    revalidatePath('/notifications')
    revalidatePath('/') // For notification bell count
    
    return { success: true }
  } catch (error) {
    console.error('Error archiving notification:', error)
    return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
  }
}

// Update notification preferences
export async function updateNotificationPreferences(preferences: {
  pushEnabled?: boolean
  inAppEnabled?: boolean
  emailEnabled?: boolean
  quietHoursStart?: string
  quietHoursEnd?: string
  blockedCategories?: string[]
  soundEnabled?: boolean
  vibrationEnabled?: boolean
}) {
  try {
    const token = await getServerAuthToken()
    if (!token) {
      throw new Error('Authentication required')
    }

    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/notifications/preferences`, {
      method: 'PUT',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(preferences)
    })

    if (!response.ok) {
      throw new Error('Failed to update notification preferences')
    }

    // Revalidate preferences page
    revalidatePath('/notifications/preferences')
    
    return { success: true }
  } catch (error) {
    console.error('Error updating notification preferences:', error)
    return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
  }
}

// Clear notification history
export async function clearNotificationHistory() {
  try {
    const token = await getServerAuthToken()
    if (!token) {
      throw new Error('Authentication required')
    }

    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/notifications/clear-history`, {
      method: 'DELETE',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      }
    })

    if (!response.ok) {
      throw new Error('Failed to clear notification history')
    }

    // Revalidate notifications page
    revalidatePath('/notifications')
    revalidatePath('/') // For notification bell count
    
    return { success: true }
  } catch (error) {
    console.error('Error clearing notification history:', error)
    return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
  }
}