/**
 * User List Server Actions - Focused on user listing and search
 * Extracted from user-actions.ts for better maintainability
 */

'use server'

import { getBearerToken } from '@/lib/actions/server-auth'
import { logger } from '@/lib/logger'
import type { 
  UnifiedUserData,
  UserOperationResult 
} from '@/lib/types/unified-user'

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080'

/**
 * User filters for enhanced user list
 */
export interface UserListFilters {
  search: string
  status: string
  role: string
  page: number
  limit: number
  sortBy: string
  sortOrder: 'asc' | 'desc'
}

export interface UserListResult {
  users: UnifiedUserData[]
  total: number
  page: number
  totalPages: number
  limit: number
}

/**
 * Get paginated list of users with filtering and search
 */
export async function getUsersList(filters?: Partial<UserListFilters>): Promise<UserOperationResult<UserListResult>> {
  try {
    logger.action.start('getUsersList', { filters })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    const searchParams = new URLSearchParams()
    
    if (filters?.page) searchParams.set('offset', ((filters.page - 1) * (filters.limit || 50)).toString())
    if (filters?.limit) searchParams.set('limit', filters.limit.toString())
    if (filters?.role && filters.role !== 'all') searchParams.set('role_filter', filters.role)
    if (filters?.search) searchParams.set('search', filters.search)
    if (filters?.status && filters.status !== 'all') searchParams.set('status', filters.status)
    if (filters?.sortBy) searchParams.set('sort_by', filters.sortBy)
    if (filters?.sortOrder) searchParams.set('sort_order', filters.sortOrder)
    
    const url = `${BACKEND_URL}/api/v1/admin/users${searchParams.toString() ? '?' + searchParams.toString() : ''}`
    
    const response = await fetch(url, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      // Add caching for better performance
      next: { revalidate: 30 }
    })
    
    if (!response.ok) {
      logger.action.error('getUsersList', `Failed to fetch users list: ${response.statusText}`, { filters })
      return { 
        success: false, 
        error: { 
          code: 'FETCH_ERROR', 
          message: `Failed to fetch users list: ${response.statusText}` 
        } 
      }
    }
    
    const result = await response.json()
    
    const usersList: UserListResult = {
      users: result.users || [],
      total: result.total || 0,
      page: Math.floor((result.offset || 0) / (result.limit || 50)) + 1,
      totalPages: Math.ceil((result.total || 0) / (result.limit || 50)),
      limit: result.limit || 50,
    }
    
    logger.action.success('getUsersList', { 
      totalUsers: usersList.total, 
      page: usersList.page,
      filters
    })
    
    return { success: true, data: usersList }
    
  } catch (error) {
    logger.action.error('getUsersList', error, { filters })
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Search users by query with advanced filtering
 */
export async function searchUsers(
  query: string, 
  filters?: Partial<Omit<UserListFilters, 'search'>>
): Promise<UserOperationResult<UserListResult>> {
  try {
    logger.action.start('searchUsers', { query, filters })
    
    return await getUsersList({
      ...filters,
      search: query
    })
    
  } catch (error) {
    logger.action.error('searchUsers', error, { query, filters })
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Get users by status (active, inactive, suspended)
 */
export async function getUsersByStatus(
  status: 'active' | 'inactive' | 'suspended',
  page = 1,
  limit = 50
): Promise<UserOperationResult<UserListResult>> {
  try {
    logger.action.start('getUsersByStatus', { status, page, limit })
    
    return await getUsersList({
      status,
      page,
      limit
    })
    
  } catch (error) {
    logger.action.error('getUsersByStatus', error, { status, page, limit })
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Get users by role
 */
export async function getUsersByRole(
  role: string,
  page = 1,
  limit = 50
): Promise<UserOperationResult<UserListResult>> {
  try {
    logger.action.start('getUsersByRole', { role, page, limit })
    
    return await getUsersList({
      role,
      page,
      limit
    })
    
  } catch (error) {
    logger.action.error('getUsersByRole', error, { role, page, limit })
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Get recently active users
 */
export async function getRecentlyActiveUsers(
  days = 7,
  limit = 20
): Promise<UserOperationResult<UserListResult>> {
  try {
    logger.action.start('getRecentlyActiveUsers', { days, limit })
    
    return await getUsersList({
      sortBy: 'last_activity',
      sortOrder: 'desc',
      limit
    })
    
  } catch (error) {
    logger.action.error('getRecentlyActiveUsers', error, { days, limit })
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}