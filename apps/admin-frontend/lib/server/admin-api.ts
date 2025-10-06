/**
 * Unified Admin Server API Service
 * Consolidates all data fetching patterns for server components and server actions
 * Replaces both /lib/actions/users.ts and /lib/api/admin-client.ts patterns
 */

import { revalidatePath } from 'next/cache'
import { cookies } from 'next/headers'
import { redirect } from 'next/navigation'

import { env } from '@/config/env'
import { getSession } from '@/lib/auth/session'

const BACKEND_URL = env.BACKEND_URL

// Types for API responses
export interface User {
  id: string
  email: string
  displayName?: string
  permissions: string[]
  subscription_tier: string
  is_active: boolean
  created_at: string
  updated_at: string
  lastLogin?: string
  twoFactorEnabled?: boolean
  billing?: {
    tier: string
    status: string
  }
  usageMetrics?: {
    apiCallsThisMonth: number
  }
  roles?: Array<{
    isActive: boolean
    name: string
  }>
  moduleAccess?: Array<{
    isActive: boolean
    name: string
  }>
  stockRankingPackages?: Array<{
    isActive: boolean
    name: string
  }>
  status: 'active' | 'inactive' | 'suspended'
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

export interface UserListResult {
  users: User[]
  total: number
  page: number
  totalPages: number
  limit: number
  offset?: number
}

export interface ActivityLogEntry {
  id: string
  timestamp: Date
  action: string
  result: string
  resource_type: string
  resource_id: string
  session_id?: string
  client_ip?: string
  user_agent?: string
  metadata: {
    error_message?: string
    duration_ms?: number
    [key: string]: any
  }
}

export interface ActivityLogResult {
  activities: ActivityLogEntry[]
  statistics: {
    total_activities: number
    login_activities: number
    failed_activities: number
    recent_activities: number
    activity_breakdown: Record<string, number>
  }
  pagination: {
    limit: number
    offset: number
    total: number
  }
}

export interface OperationResult<T = any> {
  success: boolean
  data?: T
  error?: {
    code: string
    message: string
  }
}

/**
 * Unified Admin Server API Service
 * Works for both server components and server actions
 */
export class AdminServerAPI {
  /**
   * Get authentication token from session or cookies
   * Handles both server action and server component contexts
   */
  private static async getAuthToken(): Promise<string | null> {
    try {
      // Try session token first (for server actions)
      const session = await getSession()
      if (session?.accessToken) {
        return session.accessToken
      }
    } catch (_error) {
      // Silently continue to next fallback method
    }

    try {
      // OIDC Migration: Fallback to OIDC access token from cookies (for server components)
      const cookieStore = await cookies()
      const token = cookieStore.get('access_token')?.value
      if (token) {
        return token
      }
    } catch (_error) {
      // Silently continue - no token available
    }

    return null
  }

  /**
   * Unified fetch method with error handling and authentication
   * @param endpoint
   * @param options
   */
  private static async fetch(endpoint: string, options: RequestInit = {}): Promise<any> {
    const token = await this.getAuthToken()
    
    if (!token) {
      // eslint-disable-next-line no-console
      console.warn('❌ No authentication token available')
      redirect('/unauthorized')
    }

    const response = await fetch(`${BACKEND_URL}${endpoint}`, {
      ...options,
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
        ...options.headers
      },
      cache: options.cache || 'no-store' // Default to no-store for admin data
    })

    if (!response.ok) {
      const errorBody = await response.text().catch(() => 'Unknown error')
      // eslint-disable-next-line no-console
      console.error(`❌ API Error: ${response.status} ${response.statusText}`, {
        endpoint,
        body: errorBody
      })

      if (response.status === 401) {
        redirect('/unauthorized')
      }

      if (response.status === 403) {
        redirect('/access-denied')
      }

      throw new Error(`API Error: ${response.status} ${response.statusText}`)
    }

    return await response.json()
  }

  // ===== USER MANAGEMENT METHODS =====

  /**
   * Get single user data by ID
   * @param userId
   * @param options
   */
  static async getUserData(userId: string, options: RequestInit = {}): Promise<User> {
    return this.fetch(`/api/admin/users/${userId}`, options)
  }

  /**
   * Get paginated list of users with filtering
   * @param params
   * @param params.page
   * @param params.limit
   * @param params.search
   * @param params.status
   * @param params.permissions
   * @param params.sortBy
   * @param params.sortOrder
   */
  static async getUsersList(params: {
    page?: number
    limit?: number
    search?: string
    status?: string
    permissions?: string
    sortBy?: string
    sortOrder?: 'asc' | 'desc'
  } = {}): Promise<UserListResult> {
    const searchParams = new URLSearchParams()
    
    if (params.page) {searchParams.set('offset', ((params.page - 1) * (params.limit || 50)).toString())}
    if (params.limit) {searchParams.set('limit', params.limit.toString())}
    if (params.permissions && params.permissions !== 'all') {searchParams.set('permissions_filter', params.permissions)}
    if (params.search) {searchParams.set('search', params.search)}
    if (params.status && params.status !== 'all') {searchParams.set('status', params.status)}

    const url = `/api/admin/users${searchParams.toString() ? '?' + searchParams.toString() : ''}`
    const result = await this.fetch(url)

    return {
      users: result.users || [],
      total: result.total || 0,
      page: Math.floor((result.offset || 0) / (result.limit || 50)) + 1,
      totalPages: Math.ceil((result.total || 0) / (result.limit || 50)),
      limit: result.limit || 50
    }
  }

  /**
   * Search users with query
   * @param params
   * @param params.search
   * @param params.page
   * @param params.per_page
   * @param params.status
   * @param params.package_tier
   */
  static async searchUsers(params: {
    search?: string
    page?: number
    per_page?: number
    status?: string
    package_tier?: string
  }): Promise<UserListResult> {
    const searchParams = new URLSearchParams()
    
    if (params.search) {searchParams.set('search', params.search)}
    if (params.page) {searchParams.set('page', params.page.toString())}
    if (params.per_page) {searchParams.set('per_page', params.per_page.toString())}
    if (params.status) {searchParams.set('status', params.status)}
    if (params.package_tier) {searchParams.set('package_tier', params.package_tier)}

    const url = `/api/admin/users/search?${searchParams.toString()}`
    const result = await this.fetch(url)

    return {
      users: result.users || [],
      total: result.total || 0,
      page: result.page || 1,
      totalPages: result.total_pages || 1,
      limit: result.per_page || 20
    }
  }

  /**
   * Get user statistics
   */
  static async getUserStats(): Promise<UserStats> {
    return this.fetch('/api/admin/analytics/user-statistics')
  }

  /**
   * Get user activity logs
   * @param userId
   * @param params
   * @param params.limit
   * @param params.offset
   * @param params.action_filter
   * @param params.start_date
   * @param params.end_date
   */
  static async getUserActivity(
    userId: string, 
    params: {
      limit?: number
      offset?: number
      action_filter?: string
      start_date?: string
      end_date?: string
    } = {}
  ): Promise<ActivityLogResult> {
    const searchParams = new URLSearchParams()
    
    if (params.limit) {searchParams.set('limit', params.limit.toString())}
    if (params.offset) {searchParams.set('offset', params.offset.toString())}
    if (params.action_filter) {searchParams.set('action_filter', params.action_filter)}
    if (params.start_date) {searchParams.set('start_date', params.start_date)}
    if (params.end_date) {searchParams.set('end_date', params.end_date)}

    const url = `/api/admin/users/${userId}/activity${searchParams.toString() ? '?' + searchParams.toString() : ''}`
    return this.fetch(url)
  }

  /**
   * Get multiple users by IDs
   * @param userIds
   */
  static async getUsersByIds(userIds: string[]): Promise<User[]> {
    if (userIds.length === 0) {return []}
    
    const users = await Promise.all(
      userIds.map(id => this.getUserData(id))
    )
    
    return users.filter(user => user !== null)
  }

  // ===== USER MUTATION METHODS (for server actions) =====

  /**
   * Create new user with permissions
   * @param userData
   * @param userData.email
   * @param userData.displayName
   * @param userData.permissions
   * @param userData.password
   */
  static async createUser(userData: {
    email: string
    displayName?: string
    permissions: string[]
    password: string
  }): Promise<User> {
    const result = await this.fetch('/api/admin/users', {
      method: 'POST',
      body: JSON.stringify({
        email: userData.email,
        display_name: userData.displayName,
        permissions: userData.permissions,
        password: userData.password
      })
    })
    
    revalidatePath('/users')
    return result
  }

  /**
   * Update user data
   * @param userId
   * @param userData
   */
  static async updateUser(userId: string, userData: Partial<User>): Promise<User> {
    const result = await this.fetch(`/api/admin/users/${userId}`, {
      method: 'PUT',
      body: JSON.stringify(userData)
    })
    
    revalidatePath(`/users/${userId}`)
    revalidatePath('/users')
    return result
  }

  /**
   * Delete user (soft delete)
   * @param userId
   */
  static async deleteUser(userId: string): Promise<void> {
    await this.fetch(`/api/admin/users/${userId}`, {
      method: 'DELETE'
    })
    
    revalidatePath('/users')
  }

  // ===== BULK OPERATIONS METHODS =====

  /**
   * Bulk grant permissions
   * @param userIds
   * @param permissions
   */
  static async bulkGrantPermissions(userIds: string[], permissions: string[]): Promise<void> {
    await this.fetch('/api/admin/users/bulk/permissions/grant', {
      method: 'POST',
      body: JSON.stringify({
        user_ids: userIds,
        permissions
      })
    })
    
    revalidatePath('/users')
    userIds.forEach(id => revalidatePath(`/users/${id}`))
  }

  /**
   * Bulk revoke permissions
   * @param userIds
   * @param permissions
   */
  static async bulkRevokePermissions(userIds: string[], permissions: string[]): Promise<void> {
    await this.fetch('/api/admin/users/bulk/permissions/revoke', {
      method: 'POST',
      body: JSON.stringify({
        user_ids: userIds,
        permissions
      })
    })
    
    revalidatePath('/users')
    userIds.forEach(id => revalidatePath(`/users/${id}`))
  }

  /**
   * Bulk assign roles
   * @param userIds
   * @param roles
   */
  static async bulkAssignRoles(userIds: string[], roles: string[]): Promise<void> {
    await this.fetch('/api/admin/users/bulk/roles/assign', {
      method: 'POST',
      body: JSON.stringify({
        user_ids: userIds,
        roles
      })
    })
    
    revalidatePath('/users')
    userIds.forEach(id => revalidatePath(`/users/${id}`))
  }
}

// ===== WRAPPER FUNCTIONS FOR BACKWARD COMPATIBILITY =====

/**
 * Wrapper function that returns OperationResult format
 * Used for backward compatibility with existing server actions
 * @param userId
 */
export async function getUnifiedUserData(userId: string): Promise<OperationResult<User>> {
  try {
    const user = await AdminServerAPI.getUserData(userId)
    return { success: true, data: user }
  } catch (_error) {
    return { 
      success: false, 
      error: { 
        code: 'FETCH_ERROR', 
        message: _error instanceof Error ? _error.message : 'Unknown error' 
      } 
    }
  }
}

/**
 * Wrapper function for user activity logs
 * @param userId
 * @param params
 */
export async function getUserActivityLogs(
  userId: string, 
  params: any = {}
): Promise<OperationResult<ActivityLogResult>> {
  try {
    const result = await AdminServerAPI.getUserActivity(userId, params)
    return { success: true, data: result }
  } catch (_error) {
    return { 
      success: false, 
      error: { 
        code: 'FETCH_ERROR', 
        message: _error instanceof Error ? _error.message : 'Unknown error' 
      } 
    }
  }
}