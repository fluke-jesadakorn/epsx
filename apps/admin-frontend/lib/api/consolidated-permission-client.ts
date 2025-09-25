/**
 * Consolidated Permission API Client
 * Merges: granular-permissions-admin-client.ts and lib/permissions/api-client.ts
 * Extends UnifiedAdminClient for consistency with other specialized clients
 */

import { UnifiedAdminClient } from './unified-admin-client';
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
} from '@/shared/permissions/types';

// ============================================================================
// CONSOLIDATED PERMISSION API CLIENT
// ============================================================================

export class ConsolidatedPermissionClient extends UnifiedAdminClient implements AdminPermissionApiClient {
  constructor(baseURL?: string, token?: string, serverSide = false) {
    super(baseURL, token, serverSide);
  }

  // ============================================================================
  // USER PERMISSION OPERATIONS
  // ============================================================================

  async getUserPermissions(userId?: string): Promise<PermissionStatusResponse> {
    const endpoint = userId ? `/admin/users/${userId}/permissions` : '/admin/permissions/status';
    const response = await this.get<PermissionStatusResponse>(endpoint);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch user permissions');
    }
    return response.data;
  }

  async getAllUsersWithPermissions(filters?: PermissionSearchFilters): Promise<UserPermissionOverview[]> {
    const params: Record<string, string> = {};
    
    if (filters?.platform) params.platform = filters.platform;
    if (filters?.permission_pattern) params.permission_pattern = filters.permission_pattern;
    if (filters?.expires_before) params.expires_before = filters.expires_before.toString();
    if (filters?.expires_after) params.expires_after = filters.expires_after.toString();
    
    const response = await this.get<UserPermissionOverview[]>('/admin/users/permissions', params);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch users with permissions');
    }
    return response.data;
  }

  async searchUsers(query: string): Promise<Array<{ user_id: string; email: string; display_name?: string }>> {
    const response = await this.get<Array<{ user_id: string; email: string; display_name?: string }>>(
      '/admin/users/search',
      { q: query }
    );
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to search users');
    }
    return response.data;
  }

  // ============================================================================
  // PERMISSION MANAGEMENT
  // ============================================================================

  async grantPermission(request: GrantPermissionRequest): Promise<void> {
    const response = await this.post('/admin/permissions/grant', request);
    
    if (!response.success) {
      throw new Error(response.error || 'Failed to grant permission');
    }
  }

  async revokePermission(request: RevokePermissionRequest): Promise<void> {
    const response = await this.post('/admin/permissions/revoke', request);
    
    if (!response.success) {
      throw new Error(response.error || 'Failed to revoke permission');
    }
  }

  async extendPermission(request: ExtendPermissionRequest): Promise<void> {
    const response = await this.post('/admin/permissions/extend', request);
    
    if (!response.success) {
      throw new Error(response.error || 'Failed to extend permission');
    }
  }

  // ============================================================================
  // BULK OPERATIONS
  // ============================================================================

  async bulkGrantPermissions(request: BulkPermissionRequest): Promise<BulkOperationResult> {
    const response = await this.post<BulkOperationResult>('/admin/permissions/bulk/grant', request);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to bulk grant permissions');
    }
    return response.data;
  }

  async bulkRevokePermissions(
    request: Omit<BulkPermissionRequest, 'expires_at' | 'source'>
  ): Promise<BulkOperationResult> {
    const response = await this.post<BulkOperationResult>('/admin/permissions/bulk/revoke', request);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to bulk revoke permissions');
    }
    return response.data;
  }

  async bulkCleanupExpired(userIds?: string[]): Promise<BulkOperationResult> {
    const response = await this.post<BulkOperationResult>('/admin/permissions/cleanup/expired', {
      user_ids: userIds
    });
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to cleanup expired permissions');
    }
    return response.data;
  }

  // ============================================================================
  // TEMPLATES
  // ============================================================================

  async getPermissionTemplates(): Promise<PermissionTemplate[]> {
    const response = await this.get<PermissionTemplate[]>('/admin/permissions/templates');
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch permission templates');
    }
    return response.data;
  }

  async createPermissionTemplate(
    template: Omit<PermissionTemplate, 'id' | 'created_at' | 'created_by'>
  ): Promise<PermissionTemplate> {
    const response = await this.post<PermissionTemplate>('/admin/permissions/templates', template);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to create permission template');
    }
    return response.data;
  }

  async deletePermissionTemplate(templateId: string): Promise<void> {
    const response = await this.delete(`/admin/permissions/templates/${templateId}`);
    
    if (!response.success) {
      throw new Error(response.error || 'Failed to delete permission template');
    }
  }

  async applyPermissionTemplate(templateId: string, userIds: string[]): Promise<BulkOperationResult> {
    const response = await this.post<BulkOperationResult>(
      `/admin/permissions/templates/${templateId}/apply`,
      { user_ids: userIds }
    );
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to apply permission template');
    }
    return response.data;
  }

  // ============================================================================
  // MONITORING AND AUDIT
  // ============================================================================

  async getDashboard(): Promise<AdminPermissionDashboard> {
    const response = await this.get<AdminPermissionDashboard>('/admin/permissions/dashboard');
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch permission dashboard');
    }
    return response.data;
  }

  async getPermissionAudit(userId?: string, limit: number = 100): Promise<PermissionAuditEntry[]> {
    const params: Record<string, string> = { limit: limit.toString() };
    if (userId) params.user_id = userId;
    
    const response = await this.get<PermissionAuditEntry[]>('/admin/permissions/audit', params);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch permission audit');
    }
    return response.data;
  }

  async getSystemHealth(): Promise<SystemHealthResponse> {
    const response = await this.get<SystemHealthResponse>('/admin/permissions/health');
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch system health');
    }
    return response.data;
  }

  // ============================================================================
  // CACHE MANAGEMENT
  // ============================================================================

  async invalidateUserPermissionCache(userId: string): Promise<void> {
    const response = await this.post(`/admin/permissions/cache/invalidate/${userId}`);
    
    if (!response.success) {
      throw new Error(response.error || 'Failed to invalidate user permission cache');
    }
  }

  async refreshPermissionCache(): Promise<{
    refreshed_users: number;
    failed_users: number;
    duration_ms: number;
  }> {
    const response = await this.post<{
      refreshed_users: number;
      failed_users: number;
      duration_ms: number;
    }>('/admin/permissions/cache/refresh');
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to refresh permission cache');
    }
    return response.data;
  }

  async getCacheStatistics(): Promise<{
    total_cached_users: number;
    cache_hit_rate: number;
    cache_miss_rate: number;
    average_cache_age: number;
    memory_usage: number;
    evictions_last_24h: number;
  }> {
    const response = await this.get<{
      total_cached_users: number;
      cache_hit_rate: number;
      cache_miss_rate: number;
      average_cache_age: number;
      memory_usage: number;
      evictions_last_24h: number;
    }>('/admin/permissions/cache/stats');
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch cache statistics');
    }
    return response.data;
  }

  // ============================================================================
  // ADMIN-SPECIFIC OPERATIONS (from original api-client.ts)
  // ============================================================================

  async refreshUserToken(): Promise<any> {
    const response = await this.post('/admin/auth/refresh');
    
    if (!response.success) {
      throw new Error(response.error || 'Failed to refresh user token');
    }
    return response.data;
  }

  async validatePermissionHash(hash: string): Promise<any> {
    const response = await this.post('/admin/permissions/validate-hash', { hash });
    
    if (!response.success) {
      throw new Error(response.error || 'Failed to validate permission hash');
    }
    return response.data;
  }

  async promoteUserToAdmin(userId: string, reason?: string): Promise<void> {
    const response = await this.post('/admin/admin/promote', {
      user_id: userId,
      reason
    });
    
    if (!response.success) {
      throw new Error(response.error || 'Failed to promote user to admin');
    }
  }

  async revokeAdminAccess(userId: string, reason?: string): Promise<void> {
    const response = await this.post('/admin/admin/revoke', {
      user_id: userId,
      reason
    });
    
    if (!response.success) {
      throw new Error(response.error || 'Failed to revoke admin access');
    }
  }

  async getAdminAuditLog(limit: number = 50): Promise<PermissionAuditEntry[]> {
    const response = await this.get<PermissionAuditEntry[]>('/admin/admin/audit', {
      limit: limit.toString()
    });
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch admin audit log');
    }
    return response.data;
  }

  async impersonateUser(userId: string, reason: string): Promise<{ token: string; expires_at: number }> {
    const response = await this.post<{ token: string; expires_at: number }>('/admin/admin/impersonate', {
      user_id: userId,
      reason
    });
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to impersonate user');
    }
    return response.data;
  }

  async endImpersonation(): Promise<void> {
    const response = await this.post('/admin/admin/impersonate/end');
    
    if (!response.success) {
      throw new Error(response.error || 'Failed to end impersonation');
    }
  }

  // ============================================================================
  // CONVENIENCE METHODS
  // ============================================================================

  /**
   * Get permission expiry status for a user
   */
  async getPermissionExpiryStatus(userId: string): Promise<{
    expiringPermissions: Array<{
      permission: string;
      expiresAt: string;
      daysRemaining: number;
    }>;
    expiredPermissions: string[];
    healthScore: number;
  }> {
    const response = await this.get<{
      expiringPermissions: Array<{
        permission: string;
        expiresAt: string;
        daysRemaining: number;
      }>;
      expiredPermissions: string[];
      healthScore: number;
    }>(`/admin/users/${userId}/permissions/expiry-status`);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch permission expiry status');
    }
    return response.data;
  }

  /**
   * Get system-wide permission health check
   */
  async getPermissionHealthSummary(): Promise<{
    health_score: number;
    issues: string[];
    total_users: number;
    users_with_expiring_permissions: number;
    users_with_expired_permissions: number;
  }> {
    const response = await this.get<{
      health_score: number;
      issues: string[];
      total_users: number;
      users_with_expiring_permissions: number;
      users_with_expired_permissions: number;
    }>('/admin/permissions/health');
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch permission health summary');
    }
    return response.data;
  }
}

// ============================================================================
// FACTORY FUNCTIONS AND EXPORTS
// ============================================================================

export function createPermissionClient(baseURL?: string, token?: string): ConsolidatedPermissionClient {
  return new ConsolidatedPermissionClient(baseURL, token, false);
}

export function createServerPermissionClient(baseURL?: string, token?: string): ConsolidatedPermissionClient {
  return new ConsolidatedPermissionClient(baseURL, token, true);
}

// Default instances
export const permissionClient = createPermissionClient();
export const serverPermissionClient = createServerPermissionClient();

// Legacy compatibility exports
export const adminPermissionApiClient = permissionClient;

// Re-export for convenience
export { ConsolidatedPermissionClient as PermissionApiClient };