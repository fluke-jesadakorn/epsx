/**
 * Legacy API Adapter
 * Provides backward compatibility for existing API calls during migration
 */

import { getBearerToken } from '@/lib/actions/server-auth'
import type { UnifiedUserData, UserOperationResult } from '@/lib/types/unified-user'

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080'

/**
 * Legacy API compatibility layer - maps old API calls to new unified endpoints
 * This helps maintain backward compatibility during the migration period
 */

// Legacy IAM API calls
export class LegacyIAMAdapter {
  /**
   * Get user roles (legacy IAM endpoint)
   * Maps to unified user data permissions tab
   */
  static async getUserRoles(userId: string): Promise<UserOperationResult<any[]>> {
    try {
      const token = await getBearerToken()
      if (!token) {
        return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
      }

      // Try new unified endpoint first
      const unifiedResponse = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/unified`, {
        headers: { 'Authorization': `Bearer ${token}` }
      })

      if (unifiedResponse.ok) {
        const userData: UnifiedUserData = await unifiedResponse.json()
        return { 
          success: true, 
          data: userData.roles || []
        }
      }

      // Fallback to legacy endpoint
      const legacyResponse = await fetch(`${BACKEND_URL}/api/v1/iam/users/${userId}/roles`, {
        headers: { 'Authorization': `Bearer ${token}` }
      })

      if (!legacyResponse.ok) {
        return {
          success: false,
          error: { code: 'FETCH_ERROR', message: 'Failed to fetch user roles' }
        }
      }

      const roles = await legacyResponse.json()
      return { success: true, data: roles }

    } catch (error) {
      console.error('Legacy IAM adapter error:', error)
      return {
        success: false,
        error: { code: 'UNKNOWN_ERROR', message: 'An unexpected error occurred' }
      }
    }
  }

  /**
   * Assign role to user (legacy IAM endpoint)
   */
  static async assignUserRole(userId: string, roleId: string): Promise<UserOperationResult> {
    try {
      const token = await getBearerToken()
      if (!token) {
        return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
      }

      // Try new unified endpoint first
      const unifiedResponse = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/roles`, {
        method: 'PUT',
        headers: { 
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ roleIds: [roleId], operation: 'add' })
      })

      if (unifiedResponse.ok) {
        return { success: true }
      }

      // Fallback to legacy endpoint
      const legacyResponse = await fetch(`${BACKEND_URL}/api/v1/iam/users/${userId}/roles/${roleId}`, {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}` }
      })

      if (!legacyResponse.ok) {
        return {
          success: false,
          error: { code: 'UPDATE_ERROR', message: 'Failed to assign role' }
        }
      }

      return { success: true }

    } catch (error) {
      console.error('Legacy IAM adapter error:', error)
      return {
        success: false,
        error: { code: 'UNKNOWN_ERROR', message: 'An unexpected error occurred' }
      }
    }
  }

  /**
   * Get permission profiles (legacy IAM endpoint)
   */
  static async getPermissionProfiles(userId?: string): Promise<UserOperationResult<any[]>> {
    try {
      const token = await getBearerToken()
      if (!token) {
        return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
      }

      if (userId) {
        // Try unified endpoint for user-specific profiles
        const unifiedResponse = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/unified`, {
          headers: { 'Authorization': `Bearer ${token}` }
        })

        if (unifiedResponse.ok) {
          const userData: UnifiedUserData = await unifiedResponse.json()
          return { 
            success: true, 
            data: userData.permissionProfiles || []
          }
        }
      }

      // Fallback to legacy endpoint
      const endpoint = userId 
        ? `${BACKEND_URL}/api/v1/iam/users/${userId}/permission-profiles`
        : `${BACKEND_URL}/api/v1/iam/permission-profiles`

      const legacyResponse = await fetch(endpoint, {
        headers: { 'Authorization': `Bearer ${token}` }
      })

      if (!legacyResponse.ok) {
        return {
          success: false,
          error: { code: 'FETCH_ERROR', message: 'Failed to fetch permission profiles' }
        }
      }

      const profiles = await legacyResponse.json()
      return { success: true, data: profiles }

    } catch (error) {
      console.error('Legacy IAM adapter error:', error)
      return {
        success: false,
        error: { code: 'UNKNOWN_ERROR', message: 'An unexpected error occurred' }
      }
    }
  }
}

// Legacy Module API calls
export class LegacyModuleAdapter {
  /**
   * Get user modules (legacy modules endpoint)
   * Maps to unified user data modules tab
   */
  static async getUserModules(userId: string): Promise<UserOperationResult<any[]>> {
    try {
      const token = await getBearerToken()
      if (!token) {
        return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
      }

      // Try new unified endpoint first
      const unifiedResponse = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/unified`, {
        headers: { 'Authorization': `Bearer ${token}` }
      })

      if (unifiedResponse.ok) {
        const userData: UnifiedUserData = await unifiedResponse.json()
        return { 
          success: true, 
          data: userData.moduleAccess || []
        }
      }

      // Fallback to legacy endpoint
      const legacyResponse = await fetch(`${BACKEND_URL}/api/v1/modules/users/${userId}`, {
        headers: { 'Authorization': `Bearer ${token}` }
      })

      if (!legacyResponse.ok) {
        return {
          success: false,
          error: { code: 'FETCH_ERROR', message: 'Failed to fetch user modules' }
        }
      }

      const modules = await legacyResponse.json()
      return { success: true, data: modules }

    } catch (error) {
      console.error('Legacy Module adapter error:', error)
      return {
        success: false,
        error: { code: 'UNKNOWN_ERROR', message: 'An unexpected error occurred' }
      }
    }
  }

  /**
   * Update module access (legacy modules endpoint)
   */
  static async updateModuleAccess(userId: string, moduleId: string, accessLevel: string): Promise<UserOperationResult> {
    try {
      const token = await getBearerToken()
      if (!token) {
        return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
      }

      // Try new unified endpoint first
      const unifiedResponse = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/modules`, {
        method: 'PUT',
        headers: { 
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          modules: [{ moduleId, accessLevel, operation: 'update' }]
        })
      })

      if (unifiedResponse.ok) {
        return { success: true }
      }

      // Fallback to legacy endpoint
      const legacyResponse = await fetch(`${BACKEND_URL}/api/v1/modules/users/${userId}/access`, {
        method: 'PUT',
        headers: { 
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ moduleId, accessLevel })
      })

      if (!legacyResponse.ok) {
        return {
          success: false,
          error: { code: 'UPDATE_ERROR', message: 'Failed to update module access' }
        }
      }

      return { success: true }

    } catch (error) {
      console.error('Legacy Module adapter error:', error)
      return {
        success: false,
        error: { code: 'UNKNOWN_ERROR', message: 'An unexpected error occurred' }
      }
    }
  }

  /**
   * Get module quotas (legacy modules endpoint)
   */
  static async getModuleQuotas(userId: string): Promise<UserOperationResult<any[]>> {
    try {
      const token = await getBearerToken()
      if (!token) {
        return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
      }

      // Try new unified endpoint first
      const unifiedResponse = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/unified`, {
        headers: { 'Authorization': `Bearer ${token}` }
      })

      if (unifiedResponse.ok) {
        const userData: UnifiedUserData = await unifiedResponse.json()
        return { 
          success: true, 
          data: userData.moduleQuotas || []
        }
      }

      // Fallback to legacy endpoint
      const legacyResponse = await fetch(`${BACKEND_URL}/api/v1/modules/users/${userId}/quotas`, {
        headers: { 'Authorization': `Bearer ${token}` }
      })

      if (!legacyResponse.ok) {
        return {
          success: false,
          error: { code: 'FETCH_ERROR', message: 'Failed to fetch module quotas' }
        }
      }

      const quotas = await legacyResponse.json()
      return { success: true, data: quotas }

    } catch (error) {
      console.error('Legacy Module adapter error:', error)
      return {
        success: false,
        error: { code: 'UNKNOWN_ERROR', message: 'An unexpected error occurred' }
      }
    }
  }
}

// Legacy Billing API calls
export class LegacyBillingAdapter {
  /**
   * Get user billing info (legacy billing endpoint)
   * Maps to unified user data packages tab
   */
  static async getUserBilling(userId: string): Promise<UserOperationResult<any>> {
    try {
      const token = await getBearerToken()
      if (!token) {
        return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
      }

      // Try new unified endpoint first
      const unifiedResponse = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/unified`, {
        headers: { 'Authorization': `Bearer ${token}` }
      })

      if (unifiedResponse.ok) {
        const userData: UnifiedUserData = await unifiedResponse.json()
        return { 
          success: true, 
          data: userData.billing
        }
      }

      // Fallback to legacy endpoint
      const legacyResponse = await fetch(`${BACKEND_URL}/api/v1/billing/users/${userId}`, {
        headers: { 'Authorization': `Bearer ${token}` }
      })

      if (!legacyResponse.ok) {
        return {
          success: false,
          error: { code: 'FETCH_ERROR', message: 'Failed to fetch billing info' }
        }
      }

      const billing = await legacyResponse.json()
      return { success: true, data: billing }

    } catch (error) {
      console.error('Legacy Billing adapter error:', error)
      return {
        success: false,
        error: { code: 'UNKNOWN_ERROR', message: 'An unexpected error occurred' }
      }
    }
  }

  /**
   * Get stock ranking packages (legacy billing endpoint)
   */
  static async getStockRankingPackages(userId: string): Promise<UserOperationResult<any[]>> {
    try {
      const token = await getBearerToken()
      if (!token) {
        return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
      }

      // Try new unified endpoint first
      const unifiedResponse = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/unified`, {
        headers: { 'Authorization': `Bearer ${token}` }
      })

      if (unifiedResponse.ok) {
        const userData: UnifiedUserData = await unifiedResponse.json()
        return { 
          success: true, 
          data: userData.stockRankingPackages || []
        }
      }

      // Fallback to legacy endpoint
      const legacyResponse = await fetch(`${BACKEND_URL}/api/v1/stock-ranking-packages/users/${userId}`, {
        headers: { 'Authorization': `Bearer ${token}` }
      })

      if (!legacyResponse.ok) {
        return {
          success: false,
          error: { code: 'FETCH_ERROR', message: 'Failed to fetch packages' }
        }
      }

      const packages = await legacyResponse.json()
      return { success: true, data: packages }

    } catch (error) {
      console.error('Legacy Billing adapter error:', error)
      return {
        success: false,
        error: { code: 'UNKNOWN_ERROR', message: 'An unexpected error occurred' }
      }
    }
  }

  /**
   * Update user billing (legacy billing endpoint)
   */
  static async updateUserBilling(userId: string, billingData: any): Promise<UserOperationResult> {
    try {
      const token = await getBearerToken()
      if (!token) {
        return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
      }

      // Try new unified endpoint first
      const unifiedResponse = await fetch(`${BACKEND_URL}/api/v1/admin/users/${userId}/billing`, {
        method: 'PUT',
        headers: { 
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(billingData)
      })

      if (unifiedResponse.ok) {
        return { success: true }
      }

      // Fallback to legacy endpoint
      const legacyResponse = await fetch(`${BACKEND_URL}/api/v1/billing/users/${userId}`, {
        method: 'PUT',
        headers: { 
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(billingData)
      })

      if (!legacyResponse.ok) {
        return {
          success: false,
          error: { code: 'UPDATE_ERROR', message: 'Failed to update billing' }
        }
      }

      return { success: true }

    } catch (error) {
      console.error('Legacy Billing adapter error:', error)
      return {
        success: false,
        error: { code: 'UNKNOWN_ERROR', message: 'An unexpected error occurred' }
      }
    }
  }
}

/**
 * Migration utilities for gradual transition
 */
export class MigrationUtils {
  /**
   * Feature flag to determine which API to use
   */
  static shouldUseUnifiedAPI(): boolean {
    return process.env.NEXT_PUBLIC_UNIFIED_USER_MANAGEMENT === 'true'
  }

  /**
   * Log API usage for monitoring migration progress
   */
  static logAPIUsage(endpoint: string, method: 'unified' | 'legacy') {
    if (process.env.NODE_ENV === 'development') {
      console.log(`[API Migration] ${endpoint} called via ${method} method`)
    }
    
    // In production, send to analytics/monitoring service
    if (typeof window !== 'undefined' && window.gtag) {
      window.gtag('event', 'api_usage', {
        endpoint,
        method,
        category: 'migration'
      })
    }
  }

  /**
   * Graceful degradation wrapper
   */
  static async withFallback<T>(
    unifiedCall: () => Promise<UserOperationResult<T>>,
    legacyCall: () => Promise<UserOperationResult<T>>
  ): Promise<UserOperationResult<T>> {
    try {
      if (this.shouldUseUnifiedAPI()) {
        this.logAPIUsage('unified', 'unified')
        return await unifiedCall()
      } else {
        this.logAPIUsage('legacy', 'legacy')
        return await legacyCall()
      }
    } catch (error) {
      console.error('API call failed, trying fallback:', error)
      try {
        this.logAPIUsage('fallback', 'legacy')
        return await legacyCall()
      } catch (fallbackError) {
        console.error('Fallback API call also failed:', fallbackError)
        return {
          success: false,
          error: { code: 'UNKNOWN_ERROR', message: 'All API endpoints failed' }
        }
      }
    }
  }
}