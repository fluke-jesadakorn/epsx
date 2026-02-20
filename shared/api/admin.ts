/**
 * UNIFIED ADMIN API CLIENT
 * 
 * Consolidates all admin-related API calls across EPSX applications.
 * Eliminates proxy routes by providing direct backend communication.
 * 
 * Features:
 * - User management (list, search, update)
 * - Permission management (grant, revoke, list)
 * - Group assignments and management
 * - Wallet search and management
 * - Admin analytics and metrics
 * - Performance monitoring
 * - Type-safe responses with proper error handling
 */

import type { UnifiedApiClient } from '../utils/api-client';
import { createAdminApiClient } from '../utils/api-client';

// ============================================================================
// ADMIN TYPES
// ============================================================================

export interface AdminUser {
  id: string;
  wallet_address: string;
  email?: string;
  permissions: string[];
  groups?: string[];
  tier: string;
  created_at: string;
  last_login?: string;
  status: 'active' | 'inactive' | 'suspended';
  metadata?: Record<string, unknown>;
}

export interface AdminUserFilters {
  page?: number;
  limit?: number;
  search?: string;
  wallet_address?: string;
  permission?: string;
  group?: string;
  tier?: string;
  status?: 'active' | 'inactive' | 'suspended';
  sort_by?: 'created_at' | 'last_login' | 'wallet_address' | 'tier';
  sort_order?: 'asc' | 'desc';
}

export interface AdminUsersResponse {
  success: boolean;
  data: {
    users: AdminUser[];
    total_count: number;
    page: number;
    limit: number;
    total_pages: number;
  };
  api_version?: string;
  access_level?: string;
}

export interface WalletSearchResult {
  wallet_address: string;
  user_id?: string;
  permissions: string[];
  groups?: string[];
  tier: string;
  status: string;
  created_at: string;
  last_activity?: string;
  transaction_count?: number;
  balance?: string;
}

export interface WalletSearchFilters {
  query?: string;
  address?: string;
  min_balance?: number;
  max_balance?: number;
  tier?: string;
  has_permission?: string;
  in_group?: string;
  status?: string;
  limit?: number;
  offset?: number;
}

export interface WalletSearchResponse {
  success: boolean;
  data: {
    wallets: WalletSearchResult[];
    total_count: number;
    search_metadata: {
      query_time: number;
      total_scanned: number;
      filters_applied: string[];
    };
  };
  api_version?: string;
  access_level?: string;
}

export interface Permission {
  id: string;
  wallet_address: string;
  permission: string;
  source: 'direct' | 'group' | 'tier';
  granted_by?: string;
  granted_at: string;
  expires_at?: string;
  metadata?: Record<string, unknown>;
}

export interface PermissionFilters {
  wallet_address?: string;
  permission?: string;
  source?: 'direct' | 'group' | 'tier';
  limit?: number;
  offset?: number;
  expires_before?: string;
  expires_after?: string;
}

export interface PermissionsResponse {
  success: boolean;
  data: {
    permissions: Permission[];
    total_count: number;
    summary: {
      total_permissions: number;
      direct_permissions: number;
      group_permissions: number;
      tier_permissions: number;
      expired_permissions: number;
    };
  };
  api_version?: string;
  access_level?: string;
}

export interface PermissionGrantRequest {
  wallet_address: string;
  permission: string;
  expires_at?: string;
  metadata?: Record<string, unknown>;
}

export interface PermissionRevokeRequest {
  wallet_address: string;
  permission: string;
  reason?: string;
}

export interface PermissionOperationResponse {
  success: boolean;
  data: {
    operation: 'granted' | 'revoked';
    wallet_address: string;
    permission: string;
    effective_at: string;
    expires_at?: string;
  };
  message: string;
  api_version?: string;
}

export interface Group {
  id: string;
  name: string;
  description?: string;
  permissions: string[];
  member_count: number;
  created_at: string;
  updated_at: string;
  metadata?: Record<string, unknown>;
}

export interface GroupAssignRequest {
  wallet_address: string;
  group_name: string;
  expires_at?: string;
}

export interface GroupAssignResponse {
  success: boolean;
  data: {
    wallet_address: string;
    group_name: string;
    assigned_at: string;
    expires_at?: string;
    inherited_permissions: string[];
  };
  message: string;
  api_version?: string;
}

export interface AdminAnalytics {
  user_metrics: {
    total_users: number;
    active_users: number;
    new_users_today: number;
    new_users_week: number;
  };
  permission_metrics: {
    total_permissions: number;
    permissions_by_type: Record<string, number>;
    recent_grants: number;
    recent_revokes: number;
  };
  performance_metrics: {
    avg_response_time: number;
    total_requests: number;
    error_rate: number;
    cache_hit_rate: number;
  };
  timestamp: string;
}

export interface AdminAnalyticsResponse {
  success: boolean;
  data: AdminAnalytics;
  api_version?: string;
  access_level?: string;
}

export interface PerformanceMetrics {
  api_response_times: Record<string, number>;
  cache_hit_rates: Record<string, number>;
  error_rates: Record<string, number>;
  database_metrics: {
    connection_pool_usage: number;
    query_times: Record<string, number>;
    slow_queries: number;
  };
  system_metrics: {
    memory_usage: number;
    cpu_usage: number;
    disk_usage: number;
  };
}

export interface PerformanceMetricsResponse {
  success: boolean;
  data: PerformanceMetrics;
  api_version?: string;
  access_level?: string;
}

// ============================================================================
// ADMIN API CLIENT CLASS
// ============================================================================

export class AdminAPIClient {
  constructor(private client: UnifiedApiClient) { }

  // ============================================================================
  // USER MANAGEMENT
  // ============================================================================

  /**
   * Get all users with filtering and pagination
   * Route: GET /api/admin/users
   */
  async getUsers(filters: AdminUserFilters = {}): Promise<AdminUsersResponse> {
    const response = await this.client.get<AdminUsersResponse>(
      '/api/admin/users',
      filters,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch users: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data;
  }

  /**
   * Search users by various criteria
   * Route: GET /api/admin/users/search
   */
  async searchUsers(query: string, filters: Omit<AdminUserFilters, 'search'> = {}): Promise<AdminUsersResponse> {
    const searchFilters = { ...filters, search: query };

    const response = await this.client.get<AdminUsersResponse>(
      '/api/admin/users/search',
      searchFilters,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to search users: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data;
  }

  /**
   * Get user statistics
   * Route: GET /api/admin/users/stats
   */
  async getUserStats(): Promise<{
    total_users: number;
    active_users: number;
    new_users_today: number;
    users_by_tier: Record<string, number>;
  }> {
    const response = await this.client.get<{
      data: {
        total_users: number;
        active_users: number;
        new_users_today: number;
        users_by_tier: Record<string, number>;
      };
    }>(
      '/api/admin/users/stats',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch user stats: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data.data;
  }

  // ============================================================================
  // WALLET MANAGEMENT
  // ============================================================================

  /**
   * Search wallets with advanced filtering
   * Route: GET /api/admin/wallets/search
   */
  async searchWallets(filters: WalletSearchFilters = {}): Promise<WalletSearchResponse> {
    const response = await this.client.get<WalletSearchResponse>(
      '/api/admin/wallets/search',
      filters,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to search wallets: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data;
  }

  /**
   * Get recent wallets activity
   * Route: GET /api/admin/web3/recent-wallets
   */
  async getRecentWallets(limit = 10): Promise<{
    recent_wallets: Array<{
      wallet_address: string;
      last_activity: string;
      action_type: string;
      permissions: string[];
    }>;
  }> {
    const response = await this.client.get<{
      data: {
        recent_wallets: Array<{
          wallet_address: string;
          last_activity: string;
          action_type: string;
          permissions: string[];
        }>;
      };
    }>(
      '/api/admin/web3/recent-wallets',
      { limit },
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch recent wallets: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data.data;
  }

  // ============================================================================
  // PERMISSION MANAGEMENT
  // ============================================================================

  /**
   * Get permissions with filtering
   * Route: GET /api/auth/web3/permissions
   */
  async getPermissions(filters: PermissionFilters = {}): Promise<PermissionsResponse> {
    const response = await this.client.get<PermissionsResponse>(
      '/api/auth/web3/permissions',
      filters,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch permissions: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data;
  }

  /**
   * Grant permission to wallet
   * Route: POST /api/auth/web3/permissions/grant
   */
  async grantPermission(request: PermissionGrantRequest): Promise<PermissionOperationResponse> {
    const response = await this.client.post<PermissionOperationResponse>(
      '/api/auth/web3/permissions/grant',
      request,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to grant permission: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data;
  }

  /**
   * Revoke permission from wallet
   * Route: DELETE /api/auth/web3/permissions/revoke
   */
  async revokePermission(request: PermissionRevokeRequest): Promise<PermissionOperationResponse> {
    const response = await this.client.delete<PermissionOperationResponse>(
      '/api/auth/web3/permissions/revoke',
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
        body: JSON.stringify(request),
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to revoke permission: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data;
  }

  // ============================================================================
  // GROUP MANAGEMENT
  // ============================================================================

  /**
   * Assign user to group
   * Route: POST /api/admin/groups/assign
   */
  async assignToGroup(request: GroupAssignRequest): Promise<GroupAssignResponse> {
    const response = await this.client.post<GroupAssignResponse>(
      '/api/admin/groups/assign',
      request,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to assign to group: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data;
  }

  /**
   * Bulk assign modules/permissions to users
   * Route: POST /api/admin/users/bulk/assign-modules
   */
  async bulkAssignModules(request: {
    wallet_addresses: string[];
    modules: string[];
    expires_at?: string;
  }): Promise<{
    success: boolean;
    assigned_count: number;
    failed_assignments: Array<{
      wallet_address: string;
      error: string;
    }>;
  }> {
    const response = await this.client.post<{
      data: {
        success: boolean;
        assigned_count: number;
        failed_assignments: Array<{
          wallet_address: string;
          error: string;
        }>;
      };
    }>(
      '/api/admin/users/bulk/assign-modules',
      request,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to bulk assign modules: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data.data;
  }

  // ============================================================================
  // ANALYTICS & MONITORING
  // ============================================================================

  /**
   * Get admin dashboard analytics
   * Route: GET /api/admin/analytics/dashboard
   */
  async getDashboardAnalytics(): Promise<AdminAnalyticsResponse> {
    const response = await this.client.get<AdminAnalyticsResponse>(
      '/api/admin/analytics/dashboard',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch dashboard analytics: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data;
  }

  /**
   * Get performance metrics
   * Route: GET /api/admin/analytics/performance
   */
  async getPerformanceMetrics(): Promise<PerformanceMetricsResponse> {
    const response = await this.client.get<PerformanceMetricsResponse>(
      '/api/admin/analytics/performance',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch performance metrics: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data;
  }

  /**
   * Get permissions analytics
   * Route: GET /api/admin/analytics/permissions
   */
  async getPermissionAnalytics(): Promise<{
    permission_distribution: Record<string, number>;
    group_membership: Array<{ group: string; count: number }>;
    recent_activity: Array<{
      action: string;
      permission: string;
      wallet_address: string;
      timestamp: string;
    }>;
  }> {
    const response = await this.client.get<{
      data: {
        permission_distribution: Record<string, number>;
        group_membership: Array<{ group: string; count: number }>;
        recent_activity: Array<{
          action: string;
          permission: string;
          wallet_address: string;

          timestamp: string;
        }>;
      };
    }>(
      '/api/admin/analytics/permissions',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch permission analytics: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data.data;
  }

  /**
   * Get cache statistics
   * Route: GET /api/admin/cache/stats
   */
  async getCacheStats(): Promise<{
    cache_hit_rate: number;
    total_requests: number;
    cache_size: number;
    eviction_count: number;
    performance_impact: number;
  }> {
    const response = await this.client.get<{
      data: {
        cache_hit_rate: number;
        total_requests: number;
        cache_size: number;
        eviction_count: number;
        performance_impact: number;
      };
    }>(
      '/api/admin/cache/stats',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch cache stats: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data.data;
  }

  // ============================================================================
  // WEB3 SPECIFIC ADMIN FEATURES
  // ============================================================================

  /**
   * Get NFT gates configuration
   * Route: GET /api/admin/web3/nft-gates
   */
  async getNFTGates(): Promise<{
    nft_gates: Array<{
      id: string;
      contract_address: string;
      chain_id: number;
      required_tokens: number;
      permissions: string[];
      status: string;
    }>;
  }> {
    const response = await this.client.get<{
      data: {
        nft_gates: Array<{
          id: string;
          contract_address: string;
          chain_id: number;
          required_tokens: number;
          permissions: string[];
          status: string;
        }>;
      };
    }>(
      '/api/admin/web3/nft-gates',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch NFT gates: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data.data;
  }

  /**
   * Get token gates configuration
   * Route: GET /api/admin/web3/token-gates
   */
  async getTokenGates(): Promise<{
    token_gates: Array<{
      id: string;
      token_address: string;
      chain_id: number;
      required_balance: string;
      permissions: string[];
      status: string;
    }>;
  }> {
    const response = await this.client.get<{
      data: {
        token_gates: Array<{
          id: string;
          token_address: string;
          chain_id: number;
          required_balance: string;
          permissions: string[];
          status: string;
        }>;
      };
    }>(
      '/api/admin/web3/token-gates',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch token gates: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data.data;
  }

  /**
   * Get DAO proposals
   * Route: GET /api/admin/web3/dao-proposals
   */
  async getDAOProposals(): Promise<{
    proposals: Array<{
      id: string;
      title: string;
      description: string;
      status: string;
      votes_for: number;
      votes_against: number;
      created_at: string;
    }>;
  }> {
    const response = await this.client.get<{
      data: {
        proposals: Array<{
          id: string;
          title: string;
          description: string;
          status: string;
          votes_for: number;
          votes_against: number;
          created_at: string;
        }>;
      };
    }>(
      '/api/admin/web3/dao-proposals',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
          'X-Admin-Context': 'true',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch DAO proposals: ${response.error?.message ?? 'Unknown error'}`);
    }

    return response.data.data;
  }

  // ============================================================================
  // UTILITY METHODS
  // ============================================================================

  /**
   * Check if current user has admin permissions
   */
  async hasAdminPermissions(): Promise<boolean> {
    try {
      await this.getUserStats(); // Simple admin-only API call
      return true;
    } catch (_error) {
      return false;
    }
  }

  /**
   * Validate wallet address format
   */
  static isValidWalletAddress(address: string): boolean {
    return /^0x[a-fA-F0-9]{40}$/.test(address);
  }

  /**
   * Format permission string for display
   */
  static formatPermission(permission: string): string {
    return permission
      .split(':')
      .map(part => part.charAt(0).toUpperCase() + part.slice(1))
      .join(' → ');
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/**
 * Create admin API client for admin applications
 */
export function createAdminClient(client: UnifiedApiClient): AdminAPIClient {
  return new AdminAPIClient(client);
}

/**
 * Create admin client with automatic platform detection
 */
export function createPlatformAdminClient(): AdminAPIClient {
  return new AdminAPIClient(createAdminApiClient());
}