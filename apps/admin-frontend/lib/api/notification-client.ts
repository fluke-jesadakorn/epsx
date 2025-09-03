/**
 * Server Notification API Client
 * Comprehensive notification management for admin interface
 */

import { getJWTFromCookies } from '@/lib/server/jwt'
import { cookies } from 'next/headers'

// Base configuration
const API_BASE_URL = process.env.BACKEND_URL || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'

// ============================================================================
// TYPES FOR NOTIFICATION MANAGEMENT
// ============================================================================

export interface Notification {
  id: string
  type: string
  title: string
  message: string
  is_read: boolean
  created_at: string
  metadata?: Record<string, any>
}

export interface SystemMessage {
  id: string
  name: string
  enabled: boolean
  priority: 'normal' | 'high'
  template: string
  description: string
  category: string
  triggers: string[]
  delivery_channels: string[]
  schedule?: {
    schedule_type: string
    immediate: boolean
  }
  created_at: string
  updated_at: string
}

export interface NotificationTemplate {
  id: string
  name: string
  title_template: string
  body_template: string
  data_template?: Record<string, string>
  category: string
  priority: 'normal' | 'high'
  enabled: boolean
  usage_count: number
  created_at: string
  updated_at: string
}

export interface PushMessageRequest {
  title: string
  body: string
  data?: Record<string, string>
  image_url?: string
  priority?: 'normal' | 'high'
  ttl?: number
}

export interface TestMessageRequest {
  template_id?: string
  title: string
  body: string
  test_user_ids: string[]
  data?: Record<string, string>
  priority?: 'normal' | 'high'
}

export interface BulkMessageRequest {
  template_id: string
  variables: Record<string, string>
  target: 'all' | 'active_users' | { user_list: string[] }
  priority?: 'normal' | 'high'
}

export interface NotificationStats {
  total_notifications: number
  unread_count: number
  critical_count: number
  today_count: number
  last_notification_at?: string
}

export interface SystemStats {
  total_messages: number
  active_messages: number
  total_templates: number
  messages_sent_today: number
  messages_sent_this_week: number
  most_used_category: string
  most_used_template?: string
}

export interface FCMStats {
  total_tokens: number
  active_tokens: number
  inactive_tokens: number
  platform_breakdown: Record<string, number>
}

// ============================================================================
// API CLIENT HELPER
// ============================================================================

async function makeRequest<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const jwt = await getJWTFromCookies()
  
  const url = `${API_BASE_URL}/api/fcm${endpoint}`
  
  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${jwt}`,
      ...options.headers,
    },
  })

  if (!response.ok) {
    console.error(`API Error: ${response.status} ${response.statusText}`)
    throw new Error(`API request failed: ${response.statusText}`)
  }

  return response.json()
}

// ============================================================================
// SERVER NOTIFICATION API
// ============================================================================

export const ServerNotificationAPI = {
  // Basic notification operations
  async getNotifications(offset: number = 0, limit: number = 50): Promise<Notification[]> {
    try {
      const cookieStore = cookies()
      const token = getJWTFromCookies(cookieStore)
      
      if (!token) {
        console.error('No auth token available')
        return []
      }

      const response = await fetch(`${API_BASE_URL}/api/v1/notifications?page=${Math.floor(offset/limit) + 1}&per_page=${limit}`, {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
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

  async getUnreadCount(): Promise<{ count: number }> {
    try {
      const cookieStore = cookies()
      const token = getJWTFromCookies(cookieStore)
      
      if (!token) {
        console.error('No auth token available')
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
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      const data = await response.json()
      return { count: data.unread_count || 0 }
    } catch (error) {
      console.error('Failed to fetch unread count:', error)
      return { count: 0 }
    }
  },

  async markAsRead(notificationId: string): Promise<void> {
    try {
      const cookieStore = cookies()
      const token = getJWTFromCookies(cookieStore)
      
      if (!token) {
        console.error('No auth token available')
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
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      console.log(`Successfully marked notification ${notificationId} as read`)
    } catch (error) {
      console.error('Failed to mark notification as read:', error)
    }
  },

  async markAllAsRead(): Promise<void> {
    try {
      const cookieStore = cookies()
      const token = getJWTFromCookies(cookieStore)
      
      if (!token) {
        console.error('No auth token available')
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
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      console.log('Successfully marked all notifications as read')
    } catch (error) {
      console.error('Failed to mark all notifications as read:', error)
    }
  },

  // System message management
  async getSystemMessages(page: number = 1, limit: number = 20): Promise<{ messages: SystemMessage[], total: number }> {
    try {
      const response = await makeRequest<{ messages: SystemMessage[], total: number }>(`/admin/system-messages?page=${page}&limit=${limit}`)
      return response
    } catch (error) {
      console.error('Failed to fetch system messages:', error)
      return { messages: [], total: 0 }
    }
  },

  async createSystemMessage(data: {
    name: string
    template: string
    description: string
    category: string
    priority?: 'normal' | 'high'
    triggers: string[]
    delivery_channels: string[]
    enabled: boolean
  }): Promise<{ success: boolean, system_message?: SystemMessage }> {
    try {
      const response = await makeRequest<{ success: boolean, system_message?: SystemMessage }>('/admin/system-messages', {
        method: 'POST',
        body: JSON.stringify(data),
      })
      return response
    } catch (error) {
      console.error('Failed to create system message:', error)
      return { success: false }
    }
  },

  async updateSystemMessage(messageId: string, data: Partial<SystemMessage>): Promise<{ success: boolean }> {
    try {
      const response = await makeRequest<{ success: boolean }>(`/admin/system-messages/${messageId}`, {
        method: 'POST',
        body: JSON.stringify(data),
      })
      return response
    } catch (error) {
      console.error('Failed to update system message:', error)
      return { success: false }
    }
  },

  async deleteSystemMessage(messageId: string): Promise<{ success: boolean }> {
    try {
      const response = await makeRequest<{ success: boolean }>(`/admin/system-messages/${messageId}`, {
        method: 'DELETE',
      })
      return response
    } catch (error) {
      console.error('Failed to delete system message:', error)
      return { success: false }
    }
  },

  // Template management
  async getTemplates(page: number = 1, limit: number = 20, category?: string): Promise<{ templates: NotificationTemplate[], total: number }> {
    try {
      const params = new URLSearchParams({ page: page.toString(), limit: limit.toString() })
      if (category) params.append('category', category)
      
      const response = await makeRequest<{ templates: NotificationTemplate[], total: number }>(`/admin/templates?${params}`)
      return response
    } catch (error) {
      console.error('Failed to fetch templates:', error)
      return { templates: [], total: 0 }
    }
  },

  async getTemplate(templateId: string): Promise<NotificationTemplate | null> {
    try {
      const response = await makeRequest<{ success: boolean, template?: NotificationTemplate }>(`/admin/templates/${templateId}`)
      return response.template || null
    } catch (error) {
      console.error('Failed to fetch template:', error)
      return null
    }
  },

  async createTemplate(data: {
    name: string
    title_template: string
    body_template: string
    data_template?: Record<string, string>
    category: string
    priority?: 'normal' | 'high'
  }): Promise<{ success: boolean, template?: NotificationTemplate }> {
    try {
      const response = await makeRequest<{ success: boolean, template?: NotificationTemplate }>('/admin/templates', {
        method: 'POST',
        body: JSON.stringify(data),
      })
      return response
    } catch (error) {
      console.error('Failed to create template:', error)
      return { success: false }
    }
  },

  async updateTemplate(templateId: string, data: Partial<NotificationTemplate>): Promise<{ success: boolean }> {
    try {
      const response = await makeRequest<{ success: boolean }>(`/admin/templates/${templateId}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      })
      return response
    } catch (error) {
      console.error('Failed to update template:', error)
      return { success: false }
    }
  },

  async deleteTemplate(templateId: string): Promise<{ success: boolean }> {
    try {
      const response = await makeRequest<{ success: boolean }>(`/admin/templates/${templateId}`, {
        method: 'DELETE',
      })
      return response
    } catch (error) {
      console.error('Failed to delete template:', error)
      return { success: false }
    }
  },

  async renderTemplate(templateId: string, variables: Record<string, string>): Promise<{
    success: boolean
    title: string
    body: string
    preview_html: string
  }> {
    try {
      const response = await makeRequest<{
        success: boolean
        title: string
        body: string
        preview_html: string
      }>('/admin/templates/render', {
        method: 'POST',
        body: JSON.stringify({ template_id: templateId, variables }),
      })
      return response
    } catch (error) {
      console.error('Failed to render template:', error)
      return { success: false, title: '', body: '', preview_html: '' }
    }
  },

  // Push messaging
  async sendToUser(userId: string, message: PushMessageRequest): Promise<{
    success: boolean
    message: string
    sent_count: number
    failed_count: number
  }> {
    try {
      const response = await makeRequest<{
        success: boolean
        message: string
        sent_count: number
        failed_count: number
      }>(`/admin/push/user/${userId}`, {
        method: 'POST',
        body: JSON.stringify(message),
      })
      return response
    } catch (error) {
      console.error('Failed to send push to user:', error)
      return { success: false, message: 'Failed to send', sent_count: 0, failed_count: 1 }
    }
  },

  async sendBroadcast(message: PushMessageRequest & { limit?: number }): Promise<{
    success: boolean
    message: string
    sent_count: number
    failed_count: number
  }> {
    try {
      const response = await makeRequest<{
        success: boolean
        message: string
        sent_count: number
        failed_count: number
      }>('/admin/push/broadcast', {
        method: 'POST',
        body: JSON.stringify(message),
      })
      return response
    } catch (error) {
      console.error('Failed to send broadcast:', error)
      return { success: false, message: 'Failed to send', sent_count: 0, failed_count: 1 }
    }
  },

  async testMessage(request: TestMessageRequest): Promise<{
    success: boolean
    message: string
    sent_count: number
    failed_count: number
    results: Array<{
      user_id: string
      success: boolean
      message: string
    }>
  }> {
    try {
      const response = await makeRequest<{
        success: boolean
        message: string
        sent_count: number
        failed_count: number
        results: Array<{
          user_id: string
          success: boolean
          message: string
        }>
      }>('/admin/system-messages/test', {
        method: 'POST',
        body: JSON.stringify(request),
      })
      return response
    } catch (error) {
      console.error('Failed to test message:', error)
      return { 
        success: false, 
        message: 'Failed to test', 
        sent_count: 0, 
        failed_count: 1,
        results: []
      }
    }
  },

  async sendBulkMessage(request: BulkMessageRequest): Promise<{
    success: boolean
    message: string
    sent_count: number
    failed_count: number
  }> {
    try {
      const response = await makeRequest<{
        success: boolean
        message: string
        sent_count: number
        failed_count: number
      }>('/admin/system-messages/bulk', {
        method: 'POST',
        body: JSON.stringify(request),
      })
      return response
    } catch (error) {
      console.error('Failed to send bulk message:', error)
      return { success: false, message: 'Failed to send', sent_count: 0, failed_count: 1 }
    }
  },

  // Statistics
  async getNotificationStats(): Promise<NotificationStats> {
    try {
      // Mock implementation for now
      return {
        total_notifications: 156,
        unread_count: 12,
        critical_count: 3,
        today_count: 24,
        last_notification_at: new Date().toISOString(),
      }
    } catch (error) {
      console.error('Failed to fetch notification stats:', error)
      return {
        total_notifications: 0,
        unread_count: 0,
        critical_count: 0,
        today_count: 0,
      }
    }
  },

  async getSystemStats(): Promise<SystemStats> {
    try {
      const response = await makeRequest<SystemStats>('/admin/system-messages/stats')
      return response
    } catch (error) {
      console.error('Failed to fetch system stats:', error)
      return {
        total_messages: 0,
        active_messages: 0,
        total_templates: 0,
        messages_sent_today: 0,
        messages_sent_this_week: 0,
        most_used_category: 'System',
      }
    }
  },

  async getTemplateStats(): Promise<{
    total_templates: number
    active_templates: number
    most_used?: string
    templates_by_category: Record<string, number>
    usage_stats: Array<{
      template_id: string
      template_name: string
      usage_count: number
      last_used?: string
      success_rate: number
    }>
  }> {
    try {
      const response = await makeRequest<{
        total_templates: number
        active_templates: number
        most_used?: string
        templates_by_category: Record<string, number>
        usage_stats: Array<{
          template_id: string
          template_name: string
          usage_count: number
          last_used?: string
          success_rate: number
        }>
      }>('/admin/templates/stats')
      return response
    } catch (error) {
      console.error('Failed to fetch template stats:', error)
      return {
        total_templates: 0,
        active_templates: 0,
        templates_by_category: {},
        usage_stats: [],
      }
    }
  },

  async getFCMStats(): Promise<FCMStats> {
    try {
      const response = await makeRequest<FCMStats>('/admin/fcm/stats')
      return response
    } catch (error) {
      console.error('Failed to fetch FCM stats:', error)
      return {
        total_tokens: 0,
        active_tokens: 0,
        inactive_tokens: 0,
        platform_breakdown: {},
      }
    }
  },
}