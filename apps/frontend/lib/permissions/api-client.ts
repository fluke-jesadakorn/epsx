// ============================================================================
// FRONTEND PERMISSION API CLIENT
// ============================================================================
// Frontend-specific API client that uses shared permission types and logic

import {
  PermissionStatusResponse,
  PermissionValidationRequest,
  PermissionValidationResponse,
  TokenValidationResult,
  HashValidationResult,
  PermissionApiClient
} from '@/shared/permissions/types'

// ============================================================================
// FRONTEND API CLIENT IMPLEMENTATION
// ============================================================================

class FrontendPermissionApiClientImpl implements PermissionApiClient {
  private baseUrl: string = '/api/v1'

  private async apiCall<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const token = this.getAuthToken()
    
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
        ...options.headers
      }
    })

    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: 'Unknown error' }))
      throw new Error(error.message || `API call failed: ${response.status}`)
    }

    return response.json()
  }

  private getAuthToken(): string {
    // Get token from localStorage, cookies, or auth context
    return localStorage.getItem('access_token') || ''
  }

  // ============================================================================
  // USER PERMISSION OPERATIONS
  // ============================================================================

  async getUserPermissions(userId?: string): Promise<PermissionStatusResponse> {
    const endpoint = userId ? `/permissions/users/${userId}` : '/permissions/status'
    return this.apiCall<PermissionStatusResponse>(endpoint)
  }

  async refreshUserToken(): Promise<TokenValidationResult> {
    return this.apiCall<TokenValidationResult>('/auth/refresh', {
      method: 'POST'
    })
  }

  async validatePermissionHash(hash: string): Promise<HashValidationResult> {
    return this.apiCall<HashValidationResult>('/permissions/validate-hash', {
      method: 'POST',
      body: JSON.stringify({ hash })
    })
  }

  // ============================================================================
  // PERMISSION VALIDATION
  // ============================================================================

  async validatePermission(request: PermissionValidationRequest): Promise<PermissionValidationResponse> {
    return this.apiCall<PermissionValidationResponse>('/permissions/validate', {
      method: 'POST',
      body: JSON.stringify(request)
    })
  }

  // ============================================================================
  // BASIC PERMISSION OPERATIONS (require appropriate permissions)
  // ============================================================================

  async grantPermission(): Promise<void> {
    throw new Error('Permission granting not available in frontend context')
  }

  async revokePermission(): Promise<void> {
    throw new Error('Permission revoking not available in frontend context')
  }

  async extendPermission(): Promise<void> {
    throw new Error('Permission extending not available in frontend context')
  }

  // ============================================================================
  // FRONTEND-SPECIFIC OPERATIONS
  // ============================================================================

  async checkFeatureAccess(feature: string): Promise<{ hasAccess: boolean; reason?: string }> {
    return this.apiCall<{ hasAccess: boolean; reason?: string }>(`/features/${feature}/check`)
  }

  async getRankingAccess(): Promise<{ limit: number; tier: string }> {
    return this.apiCall<{ limit: number; tier: string }>('/analytics/ranking-access')
  }

  async getAnalyticsPermissions(): Promise<{
    canView: boolean
    canExport: boolean
    canAccessRealtime: boolean
    canUseAdvancedFilters: boolean
  }> {
    return this.apiCall('/analytics/permissions')
  }

  async requestPermissionUpgrade(permission: string, reason?: string): Promise<{ requestId: string }> {
    return this.apiCall<{ requestId: string }>('/permissions/request-upgrade', {
      method: 'POST',
      body: JSON.stringify({ permission, reason })
    })
  }

  async trackPermissionUsage(permission: string, context?: Record<string, any>): Promise<void> {
    await this.apiCall('/analytics/permission-usage', {
      method: 'POST',
      body: JSON.stringify({ permission, context, timestamp: Date.now() })
    })
  }

  // ============================================================================
  // LEGACY SUPPORT
  // ============================================================================

  async getLegacyUserTier(): Promise<{ tier: string; permissions: string[] }> {
    return this.apiCall<{ tier: string; permissions: string[] }>('/user/tier')
  }

  async convertLegacyPermissions(): Promise<{ 
    converted: string[]
    failed: string[]
    newPermissions: string[]
  }> {
    return this.apiCall('/permissions/convert-legacy', {
      method: 'POST'
    })
  }
}

// Export singleton instance
export const frontendPermissionApiClient = new FrontendPermissionApiClientImpl()

// Export class for testing
export { FrontendPermissionApiClientImpl }