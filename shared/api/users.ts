/**
 * UNIFIED USERS API CLIENT
 *
 * User profile, settings, and data management endpoints.
 * Consolidates user-related API calls across EPSX applications.
 *
 * Features:
 * - User profile management (get, update)
 * - Settings and preferences
 * - Data export and deletion
 * - Email management
 * - Subscription management
 */

import type { ApiResponse, UnifiedApiClient } from '../utils/api-client';

// ============================================================================
// TYPES
// ============================================================================

export interface UserProfile {
  id: string;
  wallet_address: string;
  email?: string;
  display_name?: string;
  permissions: string[];
  tier: string;
  created_at: string;
  last_login?: string;
  status: 'active' | 'inactive' | 'suspended';
  metadata?: Record<string, unknown>;
}

export interface UserSettings {
  notifications_enabled: boolean;
  email_notifications: boolean;
  push_notifications: boolean;
  notification_categories?: string[];
  theme?: 'light' | 'dark' | 'system';
  language?: string;
  timezone?: string;
}

export interface UpdateProfileRequest {
  display_name?: string;
  email?: string;
  metadata?: Record<string, unknown>;
}

export interface UpdateSettingsRequest {
  notifications_enabled?: boolean;
  email_notifications?: boolean;
  push_notifications?: boolean;
  notification_categories?: string[];
  theme?: 'light' | 'dark' | 'system';
  language?: string;
  timezone?: string;
}

export interface EmailChangeRequest {
  new_email: string;
  current_password?: string;
}

export interface EmailVerifyRequest {
  verification_code: string;
}

export interface ExportDataRequest {
  format?: 'json' | 'csv';
  include_analytics?: boolean;
  include_transactions?: boolean;
}

export interface DeleteAccountRequest {
  confirmation: string;
  reason?: string;
}

export interface SubscriptionInfo {
  id: string;
  plan_id: string;
  plan_name: string;
  tier: string;
  status: 'active' | 'cancelled' | 'expired';
  start_date: string;
  end_date?: string;
  auto_renew: boolean;
  payment_method?: string;
}

interface AccessGroup {
  name: string;
  description?: string;
  expires_at?: string;
  permissions: string[];
  source_type: 'plan' | 'group' | 'manual';
  /** When this group was assigned */
  assigned_at?: string;
  /** Who assigned this group */
  assigned_by?: string;
  /** Days remaining until expiration */
  days_remaining?: number;
  /** Whether renewal is available for this plan */
  can_renew?: boolean;
  /** Price for renewal (e.g., "29.99 USDT") */
  renewal_price?: string;
  /** Billing cycle (e.g., "monthly", "yearly") */
  billing_cycle?: string;
}

interface DirectPermission {
  permission: string;
  expires_at?: string;
  /** Days remaining until expiration */
  days_remaining?: number;
  /** When this permission was granted */
  granted_at?: string;
  /** Who granted this permission */
  granted_by?: string;
  /** Source: 'manual' | 'system' */
  source?: string;
}

export interface AccessOverviewData {
  current_tier: string;
  groups: AccessGroup[];
  direct_permissions: DirectPermission[];
}

export interface UserApiKey {
  id: string;
  name: string;
  key: string;
  created_at: string;
  last_used_at?: string;
  usage_count: number;
  is_active: boolean;
  scopes: string[];
}

// ============================================================================
// USERS API CLASS
// ============================================================================

export class UsersApi {
  private client: UnifiedApiClient;

  constructor(client: UnifiedApiClient) {
    this.client = client;
  }

  // ============================================================================
  // PROFILE MANAGEMENT
  // ============================================================================

  /**
   * Get current user profile
   * GET /api/users/profile
   */
  async getProfile(): Promise<ApiResponse<UserProfile>> {
    return await this.client.get<UserProfile>('/api/users/profile');
  }

  /**
   * Update user profile
   * PUT /api/user/profile
   */
  async updateProfile(data: UpdateProfileRequest): Promise<ApiResponse<UserProfile>> {
    return this.client.put<UserProfile>('/api/user/profile', data);
  }

  // ============================================================================
  // SETTINGS MANAGEMENT
  // ============================================================================

  /**
   * Get user settings
   * GET /api/user/settings
   */
  async getSettings(): Promise<ApiResponse<UserSettings>> {
    return this.client.get<UserSettings>('/api/user/settings');
  }

  /**
   * Update user settings
   * PUT /api/user/settings
   */
  async updateSettings(data: UpdateSettingsRequest): Promise<ApiResponse<UserSettings>> {
    return this.client.put<UserSettings>('/api/user/settings', data);
  }

  /**
   * Get notification preferences
   * GET /api/notifications/preferences
   */
  async getNotificationPreferences(): Promise<ApiResponse<UserSettings>> {
    return this.client.get<UserSettings>('/api/notifications/preferences');
  }

  /**
   * Update notification preferences
   * PUT /api/notifications/preferences
   */
  async updateNotificationPreferences(data: Partial<UserSettings>): Promise<ApiResponse<UserSettings>> {
    return this.client.put<UserSettings>('/api/notifications/preferences', data);
  }

  // ============================================================================
  // EMAIL MANAGEMENT
  // ============================================================================

  /**
   * Request email change
   * POST /api/auth/change-email
   */
  async requestEmailChange(data: EmailChangeRequest): Promise<ApiResponse<{ verification_sent: boolean }>> {
    return this.client.post<{ verification_sent: boolean }>('/api/auth/change-email', data);
  }

  /**
   * Verify email change
   * POST /api/auth/verify-email-change
   */
  async verifyEmailChange(data: EmailVerifyRequest): Promise<ApiResponse<{ email_updated: boolean }>> {
    return this.client.post<{ email_updated: boolean }>('/api/auth/verify-email-change', data);
  }

  /**
   * Get Web3 email link status
   * GET /api/auth/web3/email-status
   */
  async getWeb3EmailStatus(): Promise<ApiResponse<{ email_linked: boolean; email?: string }>> {
    return this.client.get<{ email_linked: boolean; email?: string }>('/api/auth/web3/email-status');
  }

  /**
   * Link email to Web3 wallet
   * POST /api/auth/web3/link-email
   */
  async linkEmailToWallet(email: string): Promise<ApiResponse<{ email_linked: boolean }>> {
    return this.client.post<{ email_linked: boolean }>('/api/auth/web3/link-email', { email });
  }

  /**
   * Unlink email from Web3 wallet
   * POST /api/auth/web3/unlink-email
   */
  async unlinkEmailFromWallet(): Promise<ApiResponse<{ email_unlinked: boolean }>> {
    return this.client.post<{ email_unlinked: boolean }>('/api/auth/web3/unlink-email');
  }

  // ============================================================================
  // SUBSCRIPTIONS
  // ============================================================================

  /**
   * Get user subscriptions
   * GET /api/user/subscriptions
   */
  async getSubscriptions(): Promise<ApiResponse<SubscriptionInfo[]>> {
    return this.client.get<SubscriptionInfo[]>('/api/user/subscriptions');
  }

  /**
   * Subscribe to plan
   * POST /api/user/subscribe
   */
  async subscribeToPlan(plan_id: string): Promise<ApiResponse<SubscriptionInfo>> {
    return this.client.post<SubscriptionInfo>('/api/user/subscribe', { plan_id });
  }

  /**
   * Cancel subscription
   * POST /api/user/subscriptions/{subscription_id}/cancel
   */
  async cancelSubscription(subscription_id: string): Promise<ApiResponse<{ cancelled: boolean }>> {
    return this.client.post<{ cancelled: boolean }>(`/api/user/subscriptions/${subscription_id}/cancel`);
  }

  // ============================================================================
  // API KEYS
  // ============================================================================

  /**
   * Get user API keys
   * GET /api/developer-portal/my-keys
   */
  async getApiKeys(filters?: { limit?: number; offset?: number; status?: string }): Promise<ApiResponse<{ api_keys: UserApiKey[]; total: number }>> {
    return this.client.get<{ api_keys: UserApiKey[]; total: number }>('/api/developer-portal/my-keys', filters);
  }

  /**
   * Create API key
   * POST /api/developer-portal/my-keys
   */
  async createApiKey(body: { client_name: string; client_description?: string; permissions?: string[]; plan_ids?: string[] }): Promise<ApiResponse<UserApiKey>> {
    return this.client.post<UserApiKey>('/api/developer-portal/my-keys', body);
  }

  /**
   * Delete API key (Revoke)
   * DELETE /api/developer-portal/my-keys/{key_id}
   */
  async deleteApiKey(key_id: string, reason?: string): Promise<ApiResponse<{ success: boolean; message: string }>> {
    return this.client.delete<{ success: boolean; message: string }>(`/api/developer-portal/my-keys/${key_id}`, { body: JSON.stringify({ reason }) });
  }

  // ============================================================================
  // USAGE ANALYTICS
  // ============================================================================

  /**
   * Get aggregated usage stats
   * GET /api/developer-portal/stats
   */
  async getUsageStats(): Promise<ApiResponse<Record<string, unknown>>> {
    return this.client.get<Record<string, unknown>>('/api/developer-portal/stats');
  }

  /**
   * Get usage history
   * GET /api/developer-portal/usage-history
   */
  async getUsageHistory(days = 7): Promise<ApiResponse<Record<string, unknown>>> {
    return this.client.get<Record<string, unknown>>('/api/developer-portal/usage-history', { days });
  }

  /**
   * Get top endpoints
   * GET /api/developer-portal/top-endpoints
   */
  async getTopEndpoints(days = 7): Promise<ApiResponse<Record<string, unknown>>> {
    return this.client.get<Record<string, unknown>>('/api/developer-portal/top-endpoints', { days });
  }

  /**
   * Get user assigned plans
   * GET /api/developer-portal/my-plans
   */
  async getMyPlans(): Promise<ApiResponse<Record<string, unknown>[]>> {
    return this.client.get<Record<string, unknown>[]>('/api/developer-portal/my-plans');
  }

  // ============================================================================
  // DATA MANAGEMENT
  // ============================================================================

  /**
   * Export user data
   * POST /api/user/export-data
   */
  async exportData(options?: ExportDataRequest): Promise<ApiResponse<{ download_url: string }>> {
    return this.client.post<{ download_url: string }>('/api/user/export-data', options);
  }

  /**
   * Delete user account
   * DELETE /api/user/delete-account
   */
  async deleteAccount(data: DeleteAccountRequest): Promise<ApiResponse<{ deleted: boolean }>> {
    return this.client.delete<{ deleted: boolean }>('/api/user/delete-account', { body: JSON.stringify(data) });
  }

  // ============================================================================
  // PERMISSIONS
  // ============================================================================

  /**
   * Get access overview data (server-side compatible)
   * GET /api/users/access-overview
   */
  async getAccessOverview(): Promise<ApiResponse<AccessOverviewData>> {
    return this.client.get<AccessOverviewData>('/api/users/access-overview');
  }

  /**
   * Get user permissions
   * GET /api/user/permissions
   */
  async getPermissions(): Promise<ApiResponse<{ permissions: string[]; tier: string }>> {
    return this.client.get<{ permissions: string[]; tier: string }>('/api/user/permissions');
  }

  /**
   * Check feature access
   * POST /api/user/features/check
   */
  async checkFeatureAccess(feature: string): Promise<ApiResponse<{ has_access: boolean; reason?: string }>> {
    return this.client.post<{ has_access: boolean; reason?: string }>('/api/user/features/check', { feature });
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/**
 * Create Users API client for frontend
 */
export function createUsersClient(client: UnifiedApiClient): UsersApi {
  return new UsersApi(client);
}

/**
 * Create Users API client for admin
 */
export function createAdminUsersClient(client: UnifiedApiClient): UsersApi {
  return new UsersApi(client);
}

// ============================================================================
// EXPORTS
// ============================================================================

export default UsersApi;
