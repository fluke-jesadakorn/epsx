/**
 * UNIFIED WALLETS API CLIENT
 *
 * Wallet search, lookup, and management endpoints.
 * Consolidates wallet-related API calls across EPSX applications.
 *
 * Features:
 * - Wallet search and lookup
 * - Wallet activity tracking
 * - Recent wallets
 * - Wallet statistics
 */

import { UnifiedApiClient, ApiResponse, PaginatedResponse } from '../utils/api-client';

// ============================================================================
// TYPES
// ============================================================================

export interface WalletInfo {
  wallet_address: string;
  user_id?: string;
  permissions: string[];
  groups: string[];
  tier: string;
  status: 'active' | 'inactive' | 'suspended';
  created_at: string;
  last_activity?: string;
  activity_count?: number;
  balance?: string;
  transaction_count?: number;
  metadata?: Record<string, any>;
}

export interface WalletSearchFilters {
  query?: string;
  address?: string;
  tier?: string;
  status?: string;
  has_permission?: string;
  in_group?: string;
  created_after?: string;
  created_before?: string;
  active_after?: string;
  min_balance?: number;
  max_balance?: number;
  min_transactions?: number;
  max_transactions?: number;
  limit?: number;
  offset?: number;
  sort_by?: 'created_at' | 'last_activity' | 'balance' | 'transaction_count';
  sort_order?: 'asc' | 'desc';
}

export interface WalletActivity {
  id: string;
  wallet_address: string;
  action: string;
  timestamp: string;
  details?: Record<string, any>;
  ip_address?: string;
  user_agent?: string;
}

export interface WalletStats {
  total_wallets: number;
  active_wallets: number;
  inactive_wallets: number;
  suspended_wallets: number;
  new_wallets_today: number;
  new_wallets_week: number;
  new_wallets_month: number;
  by_tier: Record<string, number>;
  by_status: Record<string, number>;
  total_balance?: string;
  average_balance?: string;
}

export interface RecentWallet {
  wallet_address: string;
  first_seen: string;
  last_activity?: string;
  tier: string;
  permissions_count: number;
  groups_count: number;
  activity_count: number;
}

// ============================================================================
// WALLETS API CLASS
// ============================================================================

export class WalletsApi {
  private client: UnifiedApiClient;

  constructor(client: UnifiedApiClient) {
    this.client = client;
  }

  // ============================================================================
  // WALLET LOOKUP
  // ============================================================================

  /**
   * Get wallet information
   * GET /api/admin/wallets/{address}
   */
  async getWallet(address: string): Promise<ApiResponse<WalletInfo>> {
    return this.client.get<WalletInfo>(`/api/admin/wallets/${address}`);
  }

  /**
   * Search wallets
   * GET /api/admin/wallets/search
   */
  async searchWallets(filters: WalletSearchFilters): Promise<ApiResponse<PaginatedResponse<WalletInfo>>> {
    const params = new URLSearchParams();
    Object.entries(filters).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        params.append(key, String(value));
      }
    });

    return this.client.get<PaginatedResponse<WalletInfo>>(`/api/admin/wallets/search?${params.toString()}`);
  }

  /**
   * Get recent wallets
   * GET /api/admin/web3/recent-wallets
   */
  async getRecentWallets(filters?: { limit?: number; days?: number }): Promise<ApiResponse<RecentWallet[]>> {
    return this.client.get<RecentWallet[]>('/api/admin/web3/recent-wallets', filters);
  }

  // ============================================================================
  // WALLET ACTIVITY
  // ============================================================================

  /**
   * Get wallet activity history
   * GET /api/admin/wallets/{address}/activity
   */
  async getWalletActivity(address: string, filters?: { limit?: number; offset?: number }): Promise<ApiResponse<PaginatedResponse<WalletActivity>>> {
    return this.client.get<PaginatedResponse<WalletActivity>>(`/api/admin/wallets/${address}/activity`, filters);
  }

  /**
   * Get wallet activity summary
   * GET /api/admin/wallets/{address}/activity/summary
   */
  async getActivitySummary(address: string, period?: '24h' | '7d' | '30d'): Promise<ApiResponse<{
    total_actions: number;
    actions_by_type: Record<string, number>;
    last_action?: WalletActivity;
  }>> {
    return this.client.get(`/api/admin/wallets/${address}/activity/summary`, { period });
  }

  // ============================================================================
  // WALLET MANAGEMENT (Admin only)
  // ============================================================================

  /**
   * Update wallet status
   * PUT /api/admin/wallets/{address}/status
   */
  async updateWalletStatus(address: string, status: 'active' | 'inactive' | 'suspended', reason?: string): Promise<ApiResponse<{ updated: boolean }>> {
    return this.client.put<{ updated: boolean }>(`/api/admin/wallets/${address}/status`, { status, reason });
  }

  /**
   * Update wallet tier
   * PUT /api/admin/wallets/{address}/tier
   */
  async updateWalletTier(address: string, tier: string): Promise<ApiResponse<{ updated: boolean }>> {
    return this.client.put<{ updated: boolean }>(`/api/admin/wallets/${address}/tier`, { tier });
  }

  /**
   * Update wallet metadata
   * PUT /api/admin/wallets/{address}/metadata
   */
  async updateWalletMetadata(address: string, metadata: Record<string, any>): Promise<ApiResponse<{ updated: boolean }>> {
    return this.client.put<{ updated: boolean }>(`/api/admin/wallets/${address}/metadata`, { metadata });
  }

  // ============================================================================
  // BULK OPERATIONS
  // ============================================================================

  /**
   * Get multiple wallets
   * POST /api/admin/wallets/bulk/get
   */
  async bulkGetWallets(addresses: string[]): Promise<ApiResponse<WalletInfo[]>> {
    return this.client.post<WalletInfo[]>('/api/admin/wallets/bulk/get', { addresses });
  }

  /**
   * Update multiple wallet statuses
   * POST /api/admin/wallets/bulk/update-status
   */
  async bulkUpdateStatus(addresses: string[], status: 'active' | 'inactive' | 'suspended', reason?: string): Promise<ApiResponse<{ updated_count: number; failed: string[] }>> {
    return this.client.post<{ updated_count: number; failed: string[] }>('/api/admin/wallets/bulk/update-status', {
      addresses,
      status,
      reason
    });
  }

  // ============================================================================
  // STATISTICS
  // ============================================================================

  /**
   * Get wallet statistics
   * GET /api/admin/wallets/stats
   */
  async getStats(): Promise<ApiResponse<WalletStats>> {
    return this.client.get<WalletStats>('/api/admin/wallets/stats');
  }

  /**
   * Get wallet growth over time
   * GET /api/admin/wallets/stats/growth
   */
  async getGrowthStats(period?: '7d' | '30d' | '90d' | '1y'): Promise<ApiResponse<Array<{
    date: string;
    new_wallets: number;
    active_wallets: number;
    total_wallets: number;
  }>>> {
    return this.client.get('/api/admin/wallets/stats/growth', { period });
  }

  /**
   * Get tier distribution
   * GET /api/admin/wallets/stats/tiers
   */
  async getTierDistribution(): Promise<ApiResponse<Record<string, {
    count: number;
    percentage: number;
    active: number;
  }>>> {
    return this.client.get('/api/admin/wallets/stats/tiers');
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/**
 * Create Wallets API client
 */
export function createWalletsClient(client: UnifiedApiClient): WalletsApi {
  return new WalletsApi(client);
}

// ============================================================================
// EXPORTS
// ============================================================================

export default WalletsApi;
