/**
 * UNIFIED PERMISSIONS API CLIENT
 *
 * Permission management endpoints for wallet-based permissions.
 * Consolidates permission-related API calls across EPSX applications.
 *
 * Features:
 * - Permission grant/revoke (admin only)
 * - Permission listing and filtering
 * - User permission queries
 * - Permission expiry management
 * - Bulk permission operations
 */

import { UnifiedApiClient, ApiResponse, PaginatedResponse } from '../utils/api-client';
import { API_ROUTES } from '../config/route-constants';

// ============================================================================
// TYPES
// ============================================================================

export interface Permission {
  permission: string;
  granted_at: string;
  granted_by?: string;
  expires_at?: number;
  source: 'direct' | 'group' | 'tier';
  notes?: string;
  metadata?: Record<string, any>;
}

export interface PermissionEntry {
  wallet_address: string;
  permission: string;
  granted_at: string;
  granted_by?: string;
  expires_at?: number;
  source: 'direct' | 'group' | 'tier';
  is_active: boolean;
}

export interface PermissionFilters {
  wallet_address?: string;
  permission?: string;
  source?: 'direct' | 'group' | 'tier';
  is_active?: boolean;
  expires_before?: number;
  expires_after?: number;
  granted_after?: string;
  granted_before?: string;
  limit?: number;
  offset?: number;
}

export interface GrantPermissionRequest {
  wallet_address: string;
  permission: string;
  expires_at?: number;
  notes?: string;
  notify_user?: boolean;
}

export interface RevokePermissionRequest {
  wallet_address: string;
  permission: string;
  reason?: string;
  notify_user?: boolean;
}

export interface BulkGrantRequest {
  wallet_addresses: string[];
  permissions: string[];
  expires_at?: number;
  notes?: string;
}

export interface BulkRevokeRequest {
  wallet_addresses: string[];
  permissions: string[];
  reason?: string;
}

export interface PermissionStats {
  total_permissions: number;
  active_permissions: number;
  expired_permissions: number;
  expiring_soon: number;
  by_source: Record<string, number>;
  by_permission: Record<string, number>;
  recent_grants: number;
  recent_revocations: number;
}

export interface UserPermissionsResponse {
  wallet_address: string;
  permissions: Permission[];
  groups: Array<{
    group_id: string;
    name: string;
    permissions: string[];
  }>;
  tier: string;
  tier_permissions: string[];
  effective_permissions: string[];
  has_access: boolean;
}

// ============================================================================
// PERMISSIONS API CLASS
// ============================================================================

export class PermissionsApi {
  private client: UnifiedApiClient;

  constructor(client: UnifiedApiClient) {
    this.client = client;
  }

  // ============================================================================
  // USER PERMISSIONS (Frontend & Admin)
  // ============================================================================

  /**
   * Get current user's permissions
   * GET /api/auth/web3/permissions
   */
  async getCurrentUserPermissions(): Promise<ApiResponse<UserPermissionsResponse>> {
    return this.client.get<UserPermissionsResponse>('/api/auth/web3/permissions');
  }

  /**
   * Get permissions for specific wallet (Admin only)
   * GET /api/v1/auth/web3/permissions?wallet_address={address}
   */
  async getWalletPermissions(wallet_address: string): Promise<ApiResponse<UserPermissionsResponse>> {
    return this.client.get<UserPermissionsResponse>('/api/v1/auth/web3/permissions', { wallet_address });
  }

  /**
   * List all permissions with filters (Admin only)
   * GET /api/v1/admin/permissions
   */
  async listPermissions(filters?: PermissionFilters): Promise<ApiResponse<PaginatedResponse<PermissionEntry>>> {
    return this.client.get<PaginatedResponse<PermissionEntry>>('/api/v1/admin/permissions', filters);
  }

  // ============================================================================
  // PERMISSION MANAGEMENT (Admin only)
  // ============================================================================

  /**
   * Grant permission to wallet
   * POST /api/v1/admin/permissions/grant
   */
  async grantPermission(data: GrantPermissionRequest): Promise<ApiResponse<{ granted: boolean; permission: Permission }>> {
    return this.client.post<{ granted: boolean; permission: Permission }>('/api/v1/admin/permissions/grant', data);
  }

  /**
   * Revoke permission from wallet
   * POST /api/v1/admin/permissions/revoke
   */
  async revokePermission(data: RevokePermissionRequest): Promise<ApiResponse<{ revoked: boolean }>> {
    return this.client.post<{ revoked: boolean }>('/api/v1/admin/permissions/revoke', data);
  }

  /**
   * Grant permissions in bulk
   * POST /api/admin/users/bulk/permissions/grant
   */
  async bulkGrantPermissions(data: BulkGrantRequest): Promise<ApiResponse<{ granted_count: number; failed: string[] }>> {
    return this.client.post<{ granted_count: number; failed: string[] }>('/api/admin/users/bulk/permissions/grant', data);
  }

  /**
   * Revoke permissions in bulk
   * POST /api/admin/users/bulk/permissions/revoke
   */
  async bulkRevokePermissions(data: BulkRevokeRequest): Promise<ApiResponse<{ revoked_count: number; failed: string[] }>> {
    return this.client.post<{ revoked_count: number; failed: string[] }>('/api/admin/users/bulk/permissions/revoke', data);
  }

  /**
   * Update permission expiry
   * PUT /api/v1/admin/permissions/expiry
   */
  async updateExpiry(wallet_address: string, permission: string, expires_at?: number): Promise<ApiResponse<{ updated: boolean }>> {
    return this.client.put<{ updated: boolean }>('/api/v1/admin/permissions/expiry', {
      wallet_address,
      permission,
      expires_at
    });
  }

  // ============================================================================
  // PERMISSION MANAGEMENT (Admin only)
  // ============================================================================

  /**
   * Get permissions (Admin)
   * GET /api/admin/permissions
   */
  async getPermissions(): Promise<ApiResponse<PermissionEntry[]>> {
    return this.client.get<PermissionEntry[]>('/api/admin/permissions');
  }

  /**
   * Grant permission (Admin)
   * POST /api/admin/permissions/grant
   */
  async grantPermission(data: GrantPermissionRequest): Promise<ApiResponse<{ granted: boolean }>> {
    return this.client.post<{ granted: boolean }>('/api/admin/permissions/grant', data);
  }

  /**
   * @deprecated Use getPermissions instead
   */
  async getWeb3Permissions(): Promise<ApiResponse<PermissionEntry[]>> {
    // Keep using old route for backward compatibility
    return this.client.get<PermissionEntry[]>('/api/admin/web3/permissions');
  }

  /**
   * @deprecated Use grantPermission instead
   */
  async grantWeb3Permission(data: GrantPermissionRequest): Promise<ApiResponse<{ granted: boolean }>> {
    // Keep using old route for backward compatibility
    return this.client.post<{ granted: boolean }>('/api/admin/web3/permissions/grant', data);
  }

  // ============================================================================
  // PERMISSION QUERIES
  // ============================================================================

  /**
   * Check if wallet has permission
   * POST /api/v1/permissions/check
   */
  async checkPermission(wallet_address: string, permission: string): Promise<ApiResponse<{ has_permission: boolean; reason?: string }>> {
    return this.client.post<{ has_permission: boolean; reason?: string }>('/api/v1/permissions/check', {
      wallet_address,
      permission
    });
  }

  /**
   * Get expiring permissions (Admin)
   * GET /api/v1/admin/permissions/expiring
   */
  async getExpiringPermissions(days: number = 7): Promise<ApiResponse<PermissionEntry[]>> {
    return this.client.get<PermissionEntry[]>('/api/v1/admin/permissions/expiring', { days });
  }

  /**
   * Get expired permissions (Admin)
   * GET /api/v1/admin/permissions/expired
   */
  async getExpiredPermissions(): Promise<ApiResponse<PermissionEntry[]>> {
    return this.client.get<PermissionEntry[]>('/api/v1/admin/permissions/expired');
  }

  /**
   * Get permission statistics (Admin)
   * GET /api/v1/admin/permissions/stats
   */
  async getStats(): Promise<ApiResponse<PermissionStats>> {
    return this.client.get<PermissionStats>('/api/v1/admin/permissions/stats');
  }

  // ============================================================================
  // PERMISSION HISTORY
  // ============================================================================

  /**
   * Get permission grant history (Admin)
   * GET /api/v1/admin/permissions/history/grants
   */
  async getGrantHistory(filters?: { wallet_address?: string; limit?: number }): Promise<ApiResponse<PermissionEntry[]>> {
    return this.client.get<PermissionEntry[]>('/api/v1/admin/permissions/history/grants', filters);
  }

  /**
   * Get permission revocation history (Admin)
   * GET /api/v1/admin/permissions/history/revocations
   */
  async getRevocationHistory(filters?: { wallet_address?: string; limit?: number }): Promise<ApiResponse<PermissionEntry[]>> {
    return this.client.get<PermissionEntry[]>('/api/v1/admin/permissions/history/revocations', filters);
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/**
 * Create Permissions API client for frontend
 */
export function createPermissionsClient(client: UnifiedApiClient): PermissionsApi {
  return new PermissionsApi(client);
}

/**
 * Create Permissions API client for admin
 */
export function createAdminPermissionsClient(client: UnifiedApiClient): PermissionsApi {
  return new PermissionsApi(client);
}

// ============================================================================
// EXPORTS
// ============================================================================

export default PermissionsApi;
