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

import { UnifiedApiClient, ApiResponse } from '../utils/api-client';

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
  metadata?: Record<string, any>;
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
  metadata?: Record<string, any>;
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
   * GET /api/v1/auth/me
   */
  async getProfile(): Promise<ApiResponse<UserProfile>> {
    return this.client.get<UserProfile>('/api/v1/auth/me');
  }

  /**
   * Update user profile
   * PUT /api/v1/user/profile
   */
  async updateProfile(data: UpdateProfileRequest): Promise<ApiResponse<UserProfile>> {
    return this.client.put<UserProfile>('/api/v1/user/profile', data);
  }

  // ============================================================================
  // SETTINGS MANAGEMENT
  // ============================================================================

  /**
   * Get user settings
   * GET /api/v1/user/settings
   */
  async getSettings(): Promise<ApiResponse<UserSettings>> {
    return this.client.get<UserSettings>('/api/v1/user/settings');
  }

  /**
   * Update user settings
   * PUT /api/v1/user/settings
   */
  async updateSettings(data: UpdateSettingsRequest): Promise<ApiResponse<UserSettings>> {
    return this.client.put<UserSettings>('/api/v1/user/settings', data);
  }

  /**
   * Get notification preferences
   * GET /api/v1/notifications/preferences
   */
  async getNotificationPreferences(): Promise<ApiResponse<UserSettings>> {
    return this.client.get<UserSettings>('/api/v1/notifications/preferences');
  }

  /**
   * Update notification preferences
   * PUT /api/v1/notifications/preferences
   */
  async updateNotificationPreferences(data: Partial<UserSettings>): Promise<ApiResponse<UserSettings>> {
    return this.client.put<UserSettings>('/api/v1/notifications/preferences', data);
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
   * GET /api/v1/user/subscriptions
   */
  async getSubscriptions(): Promise<ApiResponse<SubscriptionInfo[]>> {
    return this.client.get<SubscriptionInfo[]>('/api/v1/user/subscriptions');
  }

  /**
   * Subscribe to plan
   * POST /api/v1/user/subscribe
   */
  async subscribeToPlan(plan_id: string): Promise<ApiResponse<SubscriptionInfo>> {
    return this.client.post<SubscriptionInfo>('/api/v1/user/subscribe', { plan_id });
  }

  /**
   * Cancel subscription
   * POST /api/v1/user/subscriptions/{subscription_id}/cancel
   */
  async cancelSubscription(subscription_id: string): Promise<ApiResponse<{ cancelled: boolean }>> {
    return this.client.post<{ cancelled: boolean }>(`/api/v1/user/subscriptions/${subscription_id}/cancel`);
  }

  // ============================================================================
  // API KEYS
  // ============================================================================

  /**
   * Get user API keys
   * GET /api/auth/web3/api-keys
   */
  async getApiKeys(): Promise<ApiResponse<{ api_keys: UserApiKey[] }>> {
    return this.client.get<{ api_keys: UserApiKey[] }>('/api/auth/web3/api-keys');
  }

  /**
   * Create API key
   * POST /api/v1/user/api-keys
   */
  async createApiKey(name: string, scopes?: string[]): Promise<ApiResponse<UserApiKey>> {
    return this.client.post<UserApiKey>('/api/v1/user/api-keys', { name, scopes });
  }

  /**
   * Delete API key
   * DELETE /api/auth/web3/api-keys/{key_id}
   */
  async deleteApiKey(key_id: string): Promise<ApiResponse<{ deleted: boolean }>> {
    return this.client.delete<{ deleted: boolean }>(`/api/auth/web3/api-keys/${key_id}`);
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
   * Get user permissions
   * GET /api/v1/user/permissions
   */
  async getPermissions(): Promise<ApiResponse<{ permissions: string[]; tier: string }>> {
    return this.client.get<{ permissions: string[]; tier: string }>('/api/v1/user/permissions');
  }

  /**
   * Check feature access
   * POST /api/v1/user/features/check
   */
  async checkFeatureAccess(feature: string): Promise<ApiResponse<{ has_access: boolean; reason?: string }>> {
    return this.client.post<{ has_access: boolean; reason?: string }>('/api/v1/user/features/check', { feature });
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
