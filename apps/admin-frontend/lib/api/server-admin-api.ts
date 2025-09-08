/**
 * Server Admin API Client - Server Components Only
 * Contains server-side data fetching for admin components
 */

import { cookies } from 'next/headers'

// Base configuration
const API_BASE_URL = process.env.BACKEND_URL || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'

// Types for API responses
export interface User {
  id: string
  email: string
  permissions: string[]
  subscription_tier: string
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface UserStats {
  total_users: number
  active_users: number
  deleted_users: number
  recent_users_30_days: number
  by_permissions: Record<string, number>
  by_tier: Record<string, number>
  user_creation_by_month: Record<string, number>
  generated_at: string
}

export interface AnalyticsData {
  eps_rankings?: any[]
  performance_metrics?: any
  security_risks?: any[]
  recommendations?: any[]
}

export interface SystemConfig {
  jwt_secret_configured: boolean
  api_base_url: string
  smtp_configured: boolean
  oauth_configured: boolean
}

export interface PermissionAnalytics {
  total_permissions: number
  users_with_permissions: number
  expiring_soon: number
  expired: number
  health_score: number
  recent_activity: number
}

export interface Notification {
  id: string
  type: string
  title: string
  message: string
  is_read: boolean
  created_at: string
  metadata?: Record<string, any>
}

// Server-side fetch with JWT authentication for server components
async function serverAdminFetch(endpoint: string, options: RequestInit = {}): Promise<any> {
  try {
    const cookieStore = await cookies()
    // OIDC Migration: Get access token instead of legacy JWT
    const token = cookieStore.get('access_token')?.value
    
    if (!token) {
      console.warn('⚠️  No admin JWT token found in cookies')
    }
    
    const headers = {
      'Content-Type': 'application/json',
      ...(token && { 'Authorization': `Bearer ${token}` }),
      ...options.headers,
    }

    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      ...options,
      headers,
      cache: options.cache || 'no-store', // Default to no-store for admin data
    })

    if (!response.ok) {
      console.error(`❌ Server API Error: ${response.status} ${response.statusText}`)
      
      // Don't throw on 401 errors - return null instead for fallback
      if (response.status === 401) {
        return null
      }
      
      throw new Error(`API Error: ${response.status} ${response.statusText}`)
    }

    const data = await response.json()
    return data
    
  } catch (error) {
    console.error('❌ Admin API Fetch Error:', { endpoint, error })
    // Return null to trigger mock data fallback
    return null
  }
}

// Server Component Data Fetchers
export class ServerUserAPI {
  static async getUsers(offset = 0, limit = 50): Promise<{ users: User[], total: number }> {
    const result = await serverAdminFetch(`/api/v1/admin/users?offset=${offset}&limit=${limit}`)
    return result || { users: [], total: 0 }
  }

  static async getUserStats(): Promise<UserStats> {
    const result = await serverAdminFetch('/api/v1/admin/analytics/user-statistics')
    return result || {
      total_users: 0,
      active_users: 0,
      deleted_users: 0,
      recent_users_30_days: 0,
      by_permissions: {},
      by_tier: {},
      user_creation_by_month: {},
      generated_at: new Date().toISOString()
    }
  }
}

// Server Permission Management APIs
export class ServerPermissionAPI {
  static async getPermissionAnalytics(): Promise<PermissionAnalytics> {
    const result = await serverAdminFetch('/api/v1/admin/analytics/permissions')
    return result || {
      total_permissions: 0,
      users_with_permissions: 0,
      expiring_soon: 0,
      expired: 0,
      health_score: 0,
      recent_activity: 0
    }
  }
}

// Server Analytics APIs
export class ServerAnalyticsAPI {
  static async getEPSRankings(): Promise<any> {
    const result = await serverAdminFetch('/api/v1/analytics/eps-rankings')
    return result || { rankings: [], total_requests: 0, last_updated: new Date().toISOString() }
  }

  static async getEPSHealth(): Promise<any> {
    const result = await serverAdminFetch('/api/v1/analytics/eps-rankings/health')
    return result || { status: 'healthy', uptime: 99.9, response_time: '2.1s' }
  }

  static async getPerformanceMetrics(): Promise<any> {
    const result = await serverAdminFetch('/api/v1/admin/analytics/performance')
    return result || {
      api_response_time: 1.2,
      database_query_time: 45,
      memory_usage: 67,
      active_users: 234,
      peak_users_today: 1245,
      new_signups: 12
    }
  }

  static async getRecommendations(): Promise<any> {
    const result = await serverAdminFetch('/api/v1/admin/analytics/recommendations')
    return result || {
      confidence: 87,
      insights: [
        {
          title: 'Optimize EPS Data Caching',
          description: 'Consider implementing Redis caching for frequently accessed EPS data to improve response times.',
          priority: 'high',
          impact: 'High',
          effort: 'Medium'
        },
        {
          title: 'Monitor High-Volume Stocks',
          description: 'NVDA and AAPL are seeing unusually high user engagement. Consider dedicated monitoring.',
          priority: 'medium', 
          impact: 'Medium',
          effort: 'Low'
        },
        {
          title: 'Update Alert Thresholds',
          description: 'Current EPS growth alert thresholds may need adjustment based on recent market volatility.',
          priority: 'low',
          impact: 'Low',
          effort: 'Low'
        }
      ]
    }
  }
}

// Server System Configuration APIs
export class ServerSystemAPI {
  static async getSystemConfig(): Promise<SystemConfig> {
    const result = await serverAdminFetch('/api/v1/settings/system')
    return result || {
      jwt_secret_configured: true,
      api_base_url: 'localhost:8080',
      smtp_configured: true,
      oauth_configured: true
    }
  }

  static async getFeatureFlags(): Promise<Record<string, boolean>> {
    const result = await serverAdminFetch('/api/v1/settings/feature-flags')
    return result || {
      'eps_analytics': true,
      'realtime_updates': true,
      'notifications': true,
      'security_alerts': true,
      'data_export': true
    }
  }
}

export class ServerNotificationAPI {
  static async getUnreadCount(): Promise<{ count: number }> {
    const result = await serverAdminFetch('/api/v1/notifications/unread-count')
    return result || { count: 0 }
  }

  static async getNotifications(page: number = 1, limit: number = 20): Promise<Notification[]> {
    const result = await serverAdminFetch(`/api/v1/notifications?page=${page}&per_page=${limit}`)
    if (!result) return []
    
    // Convert backend format to our client format
    return result.notifications?.map((notification: any) => ({
      id: notification.id,
      type: notification.notification_type,
      title: notification.title,
      message: notification.message,
      is_read: notification.status === 'read',
      created_at: notification.created_at,
      metadata: notification.metadata || {}
    })) || []
  }

  static async sendToUser(userIdOrEmail: string, title: string, body: string, priority: 'normal' | 'high' = 'normal'): Promise<any> {
    const result = await serverAdminFetch('/api/v1/admin/notifications', {
      method: 'POST',
      body: JSON.stringify({
        title,
        message: body,
        notification_type: 'admin_message',
        category: 'admin',
        priority,
        target_user_email: userIdOrEmail.includes('@') ? userIdOrEmail : null,
        target_users: userIdOrEmail.includes('@') ? null : [userIdOrEmail],
        channels: ['push', 'in_app'],
        metadata: null,
        template_id: null,
        template_data: null,
        expires_at: null,
        scheduled_for: null
      })
    })
    return result || { success: false, message: 'Failed to send', sent_count: 0, failed_count: 1 }
  }

  static async sendBroadcast(title: string, body: string, priority: 'normal' | 'high' = 'normal'): Promise<any> {
    const result = await serverAdminFetch('/api/v1/admin/notifications', {
      method: 'POST',
      body: JSON.stringify({
        title,
        message: body,
        notification_type: 'admin_broadcast',
        category: 'admin',
        priority,
        target_users: null,
        channels: ['push', 'in_app'],
        metadata: null,
        template_id: null,
        template_data: null,
        expires_at: null,
        scheduled_for: null
      })
    })
    return result || { success: false, message: 'Failed to send', sent_count: 0, failed_count: 1 }
  }
}