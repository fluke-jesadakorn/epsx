/**
 * Granular Permissions Admin API Client
 * Provides admin-level permission management capabilities
 */

import { 
  PermissionStatusResponse,
  GrantPermissionRequest,
  RevokePermissionRequest,
  BulkPermissionRequest,
  ExtendPermissionRequest,
  BulkOperationResult,
  PermissionAuditEntry,
  AdminPermissionDashboard,
  UserPermissionOverview,
  PermissionSearchFilters,
  PermissionTemplate
} from '@/shared/permissions/types'
import { URL, URLContext, Service } from '../../../../shared/utils/url-resolver';

// Base configuration
const API_BASE_URL = URL.get(Service.BACKEND, URLContext.CLIENT);

// Helper function to get auth token
function getAuthToken(): string | null {
  if (typeof window === 'undefined') return null
  
  // Try access_token first (OIDC), fallback to legacy token
  return localStorage.getItem('access_token') || 
         localStorage.getItem('token') ||
         document.cookie.split('; ')
           .find(row => row.startsWith('access_token='))
           ?.split('=')[1] ||
         null
}

// Base fetch function with authentication and error handling
async function adminPermissionsFetch(
  endpoint: string, 
  options: RequestInit = {}
): Promise<any> {
  try {
    const token = getAuthToken()
    
    if (!token) {
      throw new Error('Admin authentication required')
    }

    const headers = {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`,
      ...options.headers,
    }

    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      ...options,
      headers,
    })

    if (!response.ok) {
      const errorBody = await response.text().catch(() => 'Unknown error')
      
      if (response.status === 401) {
        localStorage.removeItem('access_token')
        localStorage.removeItem('token')
        throw new Error('Admin authentication expired. Please log in again.')
      }
      
      if (response.status === 403) {
        throw new Error('Insufficient admin permissions for this operation')
      }

      let errorMessage = `Admin API Error: ${response.status} ${response.statusText}`
      try {
        const errorJson = JSON.parse(errorBody)
        if (errorJson.message) {
          errorMessage = errorJson.message
        }
      } catch {
        // Use default error message
      }

      throw new Error(errorMessage)
    }

    return response.json()
    
  } catch (error) {
    console.error('Admin Permissions API Error:', {
      endpoint,
      error: error instanceof Error ? error.message : error
    })
    throw error
  }
}

// Admin User Permission API
export class AdminUserPermissionAPI {
  static async getUserPermissions(userId: string): Promise<PermissionStatusResponse> {
    return adminPermissionsFetch(`/api/v1/admin/users/${userId}/permissions`)
  }

  static async getAllUsersWithPermissions(filters?: PermissionSearchFilters): Promise<UserPermissionOverview[]> {
    const params = new URLSearchParams()
    if (filters?.platform) params.append('platform', filters.platform)
    if (filters?.permission_pattern) params.append('permission_pattern', filters.permission_pattern)
    if (filters?.expires_before) params.append('expires_before', filters.expires_before.toString())
    if (filters?.expires_after) params.append('expires_after', filters.expires_after.toString())
    
    const query = params.toString() ? `?${params.toString()}` : ''
    return adminPermissionsFetch(`/api/v1/admin/users/permissions${query}`)
  }

  static async searchUsers(query: string): Promise<{ user_id: string; email: string; display_name?: string }[]> {
    return adminPermissionsFetch(`/api/v1/admin/users/search?q=${encodeURIComponent(query)}`)
  }
}

// Admin Permission Management API
export class AdminPermissionManagementAPI {
  static async grantPermission(request: GrantPermissionRequest): Promise<void> {
    return adminPermissionsFetch('/api/v1/admin/permissions/grant', {
      method: 'POST',
      body: JSON.stringify(request)
    })
  }

  static async revokePermission(request: RevokePermissionRequest): Promise<void> {
    return adminPermissionsFetch('/api/v1/admin/permissions/revoke', {
      method: 'POST',
      body: JSON.stringify(request)
    })
  }

  static async extendPermission(request: ExtendPermissionRequest): Promise<void> {
    return adminPermissionsFetch('/api/v1/admin/permissions/extend', {
      method: 'POST',
      body: JSON.stringify(request)
    })
  }

  static async bulkGrantPermissions(request: BulkPermissionRequest): Promise<BulkOperationResult> {
    return adminPermissionsFetch('/api/v1/admin/permissions/bulk/grant', {
      method: 'POST',
      body: JSON.stringify(request)
    })
  }

  static async bulkRevokePermissions(request: Omit<BulkPermissionRequest, 'expires_at' | 'source'>): Promise<BulkOperationResult> {
    return adminPermissionsFetch('/api/v1/admin/permissions/bulk/revoke', {
      method: 'POST',
      body: JSON.stringify(request)
    })
  }

  static async bulkCleanupExpired(userIds?: string[]): Promise<BulkOperationResult> {
    return adminPermissionsFetch('/api/v1/admin/permissions/cleanup/expired', {
      method: 'POST',
      body: JSON.stringify({ user_ids: userIds })
    })
  }
}

// Admin Permission Template API
export class AdminPermissionTemplateAPI {
  static async getPermissionTemplates(): Promise<PermissionTemplate[]> {
    return adminPermissionsFetch('/api/v1/admin/permissions/templates')
  }

  static async createPermissionTemplate(template: Omit<PermissionTemplate, 'id' | 'created_at' | 'created_by'>): Promise<PermissionTemplate> {
    return adminPermissionsFetch('/api/v1/admin/permissions/templates', {
      method: 'POST',
      body: JSON.stringify(template)
    })
  }

  static async deletePermissionTemplate(templateId: string): Promise<void> {
    return adminPermissionsFetch(`/api/v1/admin/permissions/templates/${templateId}`, {
      method: 'DELETE'
    })
  }

  static async applyPermissionTemplate(templateId: string, userIds: string[]): Promise<BulkOperationResult> {
    return adminPermissionsFetch(`/api/v1/admin/permissions/templates/${templateId}/apply`, {
      method: 'POST',
      body: JSON.stringify({ user_ids: userIds })
    })
  }
}

// Admin Permission Analytics API
export class AdminPermissionAnalyticsAPI {
  static async getDashboard(): Promise<AdminPermissionDashboard> {
    return adminPermissionsFetch('/api/v1/admin/permissions/dashboard')
  }

  static async getPermissionAudit(userId?: string, limit?: number): Promise<PermissionAuditEntry[]> {
    const params = new URLSearchParams()
    if (userId) params.append('user_id', userId)
    if (limit) params.append('limit', limit.toString())
    
    const query = params.toString() ? `?${params.toString()}` : ''
    return adminPermissionsFetch(`/api/v1/admin/permissions/audit${query}`)
  }

  static async getSystemHealth(): Promise<{ health_score: number; issues: string[] }> {
    return adminPermissionsFetch('/api/v1/admin/permissions/health')
  }
}

// Admin Permission Cache API
export class AdminPermissionCacheAPI {
  static async invalidateUserPermissionCache(userId: string): Promise<void> {
    return adminPermissionsFetch(`/api/v1/admin/permissions/cache/invalidate/${userId}`, {
      method: 'POST'
    })
  }

  static async refreshPermissionCache(): Promise<{ 
    refreshed_users: number; 
    failed_users: number; 
    duration_ms: number 
  }> {
    return adminPermissionsFetch('/api/v1/admin/permissions/cache/refresh', {
      method: 'POST'
    })
  }

  static async getCacheStatistics(): Promise<{
    total_cached_users: number
    cache_hit_rate: number
    cache_miss_rate: number
    average_cache_age: number
    memory_usage: number
    evictions_last_24h: number
  }> {
    return adminPermissionsFetch('/api/v1/admin/permissions/cache/stats')
  }
}