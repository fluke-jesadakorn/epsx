// ============================================================================
// ADMIN FRONTEND PERMISSION API CLIENT
// ============================================================================
// Admin-specific API client that uses shared permission types and logic

import {
  PermissionStatusResponse,
  UserPermissionOverview,
  GrantPermissionRequest,
  RevokePermissionRequest,
  BulkPermissionRequest,
  ExtendPermissionRequest,
  BulkOperationResult,
  PermissionAuditEntry,
  AdminPermissionDashboard,
  PermissionSearchFilters,
  PermissionTemplate,
  SystemHealthResponse,
  AdminPermissionApiClient
} from '@/shared/permissions/types'

// ============================================================================
// ADMIN API CLIENT IMPLEMENTATION
// ============================================================================

class AdminPermissionApiClientImpl implements AdminPermissionApiClient {
  private baseUrl: string = '/api/v1/admin'

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

  async getAllUsersWithPermissions(filters?: PermissionSearchFilters): Promise<UserPermissionOverview[]> {
    const query = filters ? `?${new URLSearchParams(filters as any).toString()}` : ''
    return this.apiCall<UserPermissionOverview[]>(`/permissions/users${query}`)
  }

  async searchUsers(query: string): Promise<Array<{ user_id: string; email: string; display_name?: string }>> {
    return this.apiCall<Array<{ user_id: string; email: string; display_name?: string }>>(
      `/users/search?q=${encodeURIComponent(query)}`
    )
  }

  // ============================================================================
  // PERMISSION MANAGEMENT
  // ============================================================================

  async grantPermission(request: GrantPermissionRequest): Promise<void> {
    await this.apiCall('/permissions/grant', {
      method: 'POST',
      body: JSON.stringify(request)
    })
  }

  async revokePermission(request: RevokePermissionRequest): Promise<void> {
    await this.apiCall('/permissions/revoke', {
      method: 'POST',
      body: JSON.stringify(request)
    })
  }

  async extendPermission(request: ExtendPermissionRequest): Promise<void> {
    await this.apiCall('/permissions/extend', {
      method: 'POST',
      body: JSON.stringify(request)
    })
  }

  // ============================================================================
  // BULK OPERATIONS
  // ============================================================================

  async bulkGrantPermissions(request: BulkPermissionRequest): Promise<BulkOperationResult> {
    return this.apiCall<BulkOperationResult>('/permissions/bulk/grant', {
      method: 'POST',
      body: JSON.stringify(request)
    })
  }

  async bulkRevokePermissions(
    request: Omit<BulkPermissionRequest, 'expires_at' | 'source'>
  ): Promise<BulkOperationResult> {
    return this.apiCall<BulkOperationResult>('/permissions/bulk/revoke', {
      method: 'POST',
      body: JSON.stringify(request)
    })
  }

  async bulkCleanupExpired(userIds?: string[]): Promise<BulkOperationResult> {
    return this.apiCall<BulkOperationResult>('/permissions/bulk/cleanup', {
      method: 'POST',
      body: JSON.stringify({ user_ids: userIds })
    })
  }

  // ============================================================================
  // TEMPLATES
  // ============================================================================

  async getPermissionTemplates(): Promise<PermissionTemplate[]> {
    return this.apiCall<PermissionTemplate[]>('/permissions/templates')
  }

  async createPermissionTemplate(
    template: Omit<PermissionTemplate, 'id' | 'created_at' | 'created_by'>
  ): Promise<PermissionTemplate> {
    return this.apiCall<PermissionTemplate>('/permissions/templates', {
      method: 'POST',
      body: JSON.stringify(template)
    })
  }

  async deletePermissionTemplate(templateId: string): Promise<void> {
    await this.apiCall(`/permissions/templates/${templateId}`, {
      method: 'DELETE'
    })
  }

  async applyPermissionTemplate(templateId: string, userIds: string[]): Promise<BulkOperationResult> {
    return this.apiCall<BulkOperationResult>(`/permissions/templates/${templateId}/apply`, {
      method: 'POST',
      body: JSON.stringify({ user_ids: userIds })
    })
  }

  // ============================================================================
  // MONITORING AND AUDIT
  // ============================================================================

  async getDashboard(): Promise<AdminPermissionDashboard> {
    return this.apiCall<AdminPermissionDashboard>('/permissions/dashboard')
  }

  async getPermissionAudit(userId?: string, limit: number = 100): Promise<PermissionAuditEntry[]> {
    const query = new URLSearchParams({ limit: limit.toString() })
    if (userId) query.append('user_id', userId)
    
    return this.apiCall<PermissionAuditEntry[]>(`/permissions/audit?${query.toString()}`)
  }

  async getSystemHealth(): Promise<SystemHealthResponse> {
    return this.apiCall<SystemHealthResponse>('/system/health')
  }

  // ============================================================================
  // CACHE MANAGEMENT
  // ============================================================================

  async invalidateUserPermissionCache(userId: string): Promise<void> {
    await this.apiCall(`/permissions/cache/invalidate/${userId}`, {
      method: 'POST'
    })
  }

  async refreshPermissionCache(): Promise<void> {
    await this.apiCall('/permissions/cache/refresh', {
      method: 'POST'
    })
  }

  // ============================================================================
  // TOKEN OPERATIONS
  // ============================================================================

  async refreshUserToken(): Promise<any> {
    return this.apiCall('/auth/refresh', {
      method: 'POST'
    })
  }

  async validatePermissionHash(hash: string): Promise<any> {
    return this.apiCall('/permissions/validate-hash', {
      method: 'POST',
      body: JSON.stringify({ hash })
    })
  }

  // ============================================================================
  // ADMIN-SPECIFIC OPERATIONS
  // ============================================================================

  async promoteUserToAdmin(userId: string, reason?: string): Promise<void> {
    await this.apiCall('/admin/promote', {
      method: 'POST',
      body: JSON.stringify({ user_id: userId, reason })
    })
  }

  async revokeAdminAccess(userId: string, reason?: string): Promise<void> {
    await this.apiCall('/admin/revoke', {
      method: 'POST',
      body: JSON.stringify({ user_id: userId, reason })
    })
  }

  async getAdminAuditLog(limit: number = 50): Promise<PermissionAuditEntry[]> {
    return this.apiCall<PermissionAuditEntry[]>(`/admin/audit?limit=${limit}`)
  }

  async impersonateUser(userId: string, reason: string): Promise<{ token: string; expires_at: number }> {
    return this.apiCall('/admin/impersonate', {
      method: 'POST',
      body: JSON.stringify({ user_id: userId, reason })
    })
  }

  async endImpersonation(): Promise<void> {
    await this.apiCall('/admin/impersonate/end', {
      method: 'POST'
    })
  }
}

// Export singleton instance
export const adminPermissionApiClient = new AdminPermissionApiClientImpl()

// Export class for testing
export { AdminPermissionApiClientImpl }