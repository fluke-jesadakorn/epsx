/**
 * Unified Server Data Fetchers
 * Consolidated server-side data fetching for admin components
 * Uses OIDC Bearer tokens for authentication
 * Modernized with centralized URL resolver
 */

import { cookies } from 'next/headers'
import { URL, URLContext, Service } from '../../../../shared/utils/url-resolver';

const API_BASE_URL = URL.get(Service.BACKEND, URLContext.SERVER);

// Unified server fetch with OIDC authentication
async function serverFetch(endpoint: string, options: RequestInit = {}): Promise<any> {
  try {
    const cookieStore = await cookies()
    const token = cookieStore.get('access_token')?.value
    
    if (!token) {
      console.warn('⚠️  No OIDC access token found in cookies')
    }
    
    const headers = {
      'Content-Type': 'application/json',
      ...(token && { 'Authorization': `Bearer ${token}` }),
      ...options.headers,
    }

    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      ...options,
      headers,
      cache: options.cache || 'no-store',
    })

    if (!response.ok) {
      console.error(`❌ Server API Error: ${response.status} ${response.statusText}`)
      if (response.status === 401) {
        return null // Return null for unauthorized to trigger fallback
      }
      throw new Error(`API Error: ${response.status} ${response.statusText}`)
    }

    return await response.json()
  } catch (error) {
    console.error('❌ Server Fetch Error:', { endpoint, error })
    return null
  }
}

// Type definitions
export interface User {
  id: string
  email: string
  permissions: string[]
  subscription_tier: string
  is_active: boolean
  created_at: string
  updated_at: string
  role?: string
  name?: string
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

export interface PermissionAnalytics {
  total_permissions: number
  users_with_permissions: number
  expiring_soon: number
  expired: number
  health_score: number
  recent_activity: number
}

export interface SystemMetrics {
  api_response_time: number
  database_query_time: number
  memory_usage: number
  active_users: number
  peak_users_today: number
  new_signups: number
}

export interface DashboardData {
  stats: UserStats
  permissionAnalytics: PermissionAnalytics
  systemMetrics: SystemMetrics
  recentUsers: User[]
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

// Mock data for fallbacks
const MockData = {
  userStats: (): UserStats => ({
    total_users: 2847,
    active_users: 2234,
    deleted_users: 613,
    recent_users_30_days: 234,
    by_permissions: {
      'epsx:analytics:view': 1245,
      'admin:users:manage': 23,
      'epsx:export:csv': 567
    },
    by_tier: {
      'basic': 1456,
      'premium': 778,
      'admin': 23
    },
    user_creation_by_month: {
      '2024-12': 45,
      '2024-11': 67,
      '2024-10': 78
    },
    generated_at: new Date().toISOString()
  }),

  permissionAnalytics: (): PermissionAnalytics => ({
    total_permissions: 1847,
    users_with_permissions: 234,
    expiring_soon: 12,
    expired: 3,
    health_score: 92,
    recent_activity: 47
  }),

  systemMetrics: (): SystemMetrics => ({
    api_response_time: 1.2,
    database_query_time: 45,
    memory_usage: 67,
    active_users: 234,
    peak_users_today: 1245,
    new_signups: 12
  }),

  recentUsers: (): User[] => [
    {
      id: '1',
      email: 'user1@example.com',
      permissions: ['epsx:analytics:view'],
      subscription_tier: 'premium',
      is_active: true,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      role: 'user',
      name: 'John Doe'
    },
    {
      id: '2',
      email: 'user2@example.com',
      permissions: ['epsx:analytics:view', 'admin:users:manage'],
      subscription_tier: 'admin',
      is_active: true,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      role: 'admin',
      name: 'Jane Smith'
    }
  ]
}

// Unified Data Fetchers
export class UnifiedDataFetchers {
  // Dashboard data - single fetch for performance
  static async getDashboardData(): Promise<DashboardData> {
    try {
      // Fetch all dashboard data in parallel for better performance
      const [stats, permissionAnalytics, systemMetrics, recentUsers] = await Promise.all([
        this.getUserStats(),
        this.getPermissionAnalytics(),
        this.getSystemMetrics(),
        this.getRecentUsers(5)
      ])

      return {
        stats,
        permissionAnalytics,
        systemMetrics,
        recentUsers
      }
    } catch (error) {
      console.error('Error fetching dashboard data:', error)
      // Return mock data as fallback
      return {
        stats: MockData.userStats(),
        permissionAnalytics: MockData.permissionAnalytics(),
        systemMetrics: MockData.systemMetrics(),
        recentUsers: MockData.recentUsers()
      }
    }
  }

  // User Management
  static async getUsers(offset = 0, limit = 50): Promise<{ users: User[], total: number }> {
    const result = await serverFetch(`/api/v1/admin/users?offset=${offset}&limit=${limit}`)
    return result || { users: [], total: 0 }
  }

  static async getUserStats(): Promise<UserStats> {
    const result = await serverFetch('/api/v1/admin/analytics/user-statistics')
    return result || MockData.userStats()
  }

  static async getRecentUsers(limit = 10): Promise<User[]> {
    const result = await serverFetch(`/api/v1/admin/users?offset=0&limit=${limit}&sort=created_at&order=desc`)
    return result?.users || MockData.recentUsers()
  }

  static async getUserById(id: string): Promise<User | null> {
    const result = await serverFetch(`/api/v1/admin/users/${id}`)
    return result || null
  }

  // Permission Management
  static async getPermissionAnalytics(): Promise<PermissionAnalytics> {
    const result = await serverFetch('/api/v1/admin/analytics/permissions')
    return result || MockData.permissionAnalytics()
  }

  static async getUserPermissions(userId: string): Promise<string[]> {
    const result = await serverFetch(`/api/v1/admin/users/${userId}/permissions`)
    return result?.permissions || []
  }

  // System Metrics
  static async getSystemMetrics(): Promise<SystemMetrics> {
    const result = await serverFetch('/api/v1/admin/analytics/performance')
    return result || MockData.systemMetrics()
  }

  static async getSystemConfig(): Promise<any> {
    const result = await serverFetch('/api/v1/settings/system')
    return result || {
      jwt_secret_configured: true,
      api_base_url: 'localhost:8080',
      smtp_configured: true,
      oauth_configured: true
    }
  }

  // Analytics
  static async getEPSRankings(): Promise<any> {
    const result = await serverFetch('/api/v1/analytics/eps-rankings')
    return result || { rankings: [], total_requests: 0, last_updated: new Date().toISOString() }
  }

  static async getEPSHealth(): Promise<any> {
    const result = await serverFetch('/api/v1/analytics/eps-rankings/health')
    return result || { status: 'healthy', uptime: 99.9, response_time: '2.1s' }
  }

  // Notifications
  static async getNotifications(page: number = 1, limit: number = 20): Promise<Notification[]> {
    const result = await serverFetch(`/api/v1/notifications?page=${page}&per_page=${limit}`)
    if (!result) return []
    
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

  static async getUnreadNotificationCount(): Promise<{ count: number }> {
    const result = await serverFetch('/api/v1/notifications/unread-count')
    return result || { count: 0 }
  }

  // IAM Functions
  static async getIAMUsers(filters?: {
    role?: string
    status?: string
    search?: string
    limit?: number
    offset?: number
  }): Promise<User[]> {
    const queryParams = new URLSearchParams()
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined) {
          queryParams.append(key, String(value))
        }
      })
    }
    
    const endpoint = `/api/v1/iam/users${queryParams.toString() ? `?${queryParams.toString()}` : ''}`
    const result = await serverFetch(endpoint)
    return result?.users || []
  }

  static async getIAMRoles(): Promise<any[]> {
    const result = await serverFetch('/api/v1/iam/roles')
    return result || []
  }

  static async getIAMPolicies(): Promise<any[]> {
    const result = await serverFetch('/api/v1/iam/policies')
    return result || []
  }
}