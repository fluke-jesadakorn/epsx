/**
 * UNIFIED PLANS API CLIENT
 *
 * Consolidates all plan and subscription management API calls.
 * Used by admin-frontend for plan CRUD operations and analytics.
 *
 * Features:
 * - Dynamic plan management (create, update, delete)
 * - Subscription lifecycle management
 * - Plan analytics and revenue tracking
 * - API key and module management for developer portal
 * - Type-safe responses with proper error handling
 */

import { ApiResponse, UnifiedApiClient } from '../utils/api-client';

// ============================================================================
// PLAN TYPES
// ============================================================================

export interface PlanFeatureRequest {
  context_name: string; // "web_app", "api_access", "admin_interface"
  feature_key: string;
  feature_config: Record<string, any>;
  resource_cost: number;
  is_active: boolean;
}

export interface CreatePlanRequest {
  name: string;
  description?: string;
  plan_type: string;
  current_price: number;
  currency: string;
  target_audience: string; // "web_users", "api_developers", "enterprises"
  billing_model: string;   // Always "pay_per_use"
  plan_category: string;   // "standard", "api", "enterprise", "custom"
  features: PlanFeatureRequest[];
  metadata?: Record<string, any>;
}

export interface UpdatePlanRequest {
  name?: string;
  description?: string;
  current_price?: number;
  is_active?: boolean;
  features?: PlanFeatureRequest[];
  permissions?: string[];
  metadata?: Record<string, any>;
}

export interface PlanFeatureResponse {
  id: number;
  context_name: string;
  feature_key: string;
  feature_config: Record<string, any>;
  resource_cost: number;
  is_active: boolean;
}

export interface PlanResponse {
  id: number | string; // Support both legacy number and UUID string
  name: string;
  description?: string;
  plan_type: string;
  current_price: number | string; // Support both number and decimal string
  effective_price?: number; // Calculated price with promotion
  promotion_active?: boolean; // Is promotion currently active
  promotion_status?: 'active' | 'upcoming' | 'expired' | 'disabled'; // Promotion status
  promotion_discount?: number; // Discount percentage
  promotion_ends_at?: string; // When active promotion ends
  currency: string;
  target_audience: string;
  billing_model: string;
  plan_category: string;
  is_active: boolean;
  features: PlanFeatureResponse[];
  permissions?: string[];
  metadata?: Record<string, any>;
  created_at: string;
  updated_at?: string;
  subscriber_count: number;
  revenue_last_30_days: number | string; // Support both number and decimal string
}

export interface PlanListResponse {
  plans: PlanResponse[];
  total_count: number;
  has_more: boolean;
}

export interface PlanAnalyticsResponse {
  plan_id: number;
  plan_name: string;
  analytics_period: string;
  subscriber_metrics: {
    total_subscribers: number;
    new_subscribers_this_period: number;
    churned_subscribers_this_period: number;
    subscriber_growth_rate: number;
    subscriber_distribution_by_context: Record<string, number>;
  };
  usage_metrics: {
    total_api_calls: number;
    total_data_transfer_gb: number;
    average_requests_per_subscriber: number;
    peak_usage_time: string;
    resource_utilization_percentage: number;
    top_endpoints: Array<{
      endpoint: string;
      request_count: number;
      avg_response_time_ms: number;
    }>;
  };
  revenue_metrics: {
    total_revenue: number;
    revenue_growth_rate: number;
    average_revenue_per_user: number;
    revenue_by_feature: Record<string, number>;
    projected_monthly_revenue: number;
  };
  performance_metrics: {
    plan_efficiency_score: number;
    cost_per_request: number;
    profit_margin: number;
    rate_limit_hit_rate: number;
    overage_usage_rate: number;
  };
  recommendations: Array<{
    recommendation_type: string;
    priority: string;
    title: string;
    description: string;
    suggested_action?: string;
    potential_impact?: string;
  }>;
}

// ============================================================================
// SUBSCRIPTION TYPES
// ============================================================================

export interface CreateSubscriptionRequest {
  user_id: string;
  plan_id: number;
  access_context: string; // "internal", "external", "both"
  api_key_name?: string;
  expires_at?: string;
  auto_renew: boolean;
  metadata?: Record<string, any>;
}

export interface UpdateSubscriptionRequest {
  status?: string;
  plan_id?: number;
  expires_at?: string;
  auto_renew?: boolean;
}

export interface SubscriptionResponse {
  id: string;
  user_id: string;
  plan_id: number;
  plan_name: string;
  access_context: string;
  api_key?: string;
  api_key_name?: string;
  status: string;
  current_usage: Record<string, any>;
  quota_limits: Record<string, any>;
  started_at: string;
  expires_at?: string;
  auto_renew: boolean;
  last_billed_at?: string;
  next_billing_date?: string;
  created_at: string;
  metadata?: Record<string, any>;
}

// ============================================================================
// API KEY TYPES
// ============================================================================

export interface ApiKeyModuleConfig {
  module_id: string;
  module_name: string;
  access_level: string; // "bronze", "silver", "gold", "platinum", "enterprise"
  custom_quotas?: Record<string, any>;
}

export interface ApiKeyRequest {
  client_name: string;
  client_description?: string;
  client_contact_email?: string;
  allowed_modules: ApiKeyModuleConfig[];
  ip_restrictions?: string[];
  expires_at?: string;
  rate_limits?: Record<string, number>;
}

export interface ApiKeyResponse {
  id: string;
  key_preview: string;
  full_key?: string; // Optional: only returned if user owns key
  client_name: string;
  client_description?: string;
  client_contact_email?: string;
  status: 'active' | 'revoked' | 'expired';
  total_requests: number;
  created_at: string;
  created_by: string;
  expires_at?: string;
  allowed_modules: ApiKeyModuleConfig[];
  ip_restrictions: string[];
  rate_limits: Record<string, number>;
  last_used_at?: string;
  revoked_at?: string;
  revoked_by?: string;
  revocation_reason?: string;
}

export interface Module {
  id: string;
  name: string;
  display_name: string;
  description?: string;
  category: string;
  status: string;
  access_levels: Record<string, any>;
  default_quotas: Record<string, any>;
  endpoints: ModuleEndpoint[];
}

export interface ModuleEndpoint {
  path: string;
  method: string;
  description: string;
  access_level_required: string;
}

// ============================================================================
// PLANS API CLIENT CLASS
// ============================================================================

export class PlansAPIClient {
  constructor(private client: UnifiedApiClient) { }

  // ============================================================================
  // PLAN MANAGEMENT
  // ============================================================================

  /**
   * Get all plans with filtering
   * Route: GET /api/v1/admin/plans
   */
  async getPlans(params: {
    limit?: number;
    offset?: number;
    plan_category?: string;
    target_audience?: string;
    is_active?: boolean;
  } = {}): Promise<ApiResponse<PlanListResponse>> {
    return this.client.get('/api/v1/admin/plans', params);
  }

  /**
   * Get single plan by ID
   * Route: GET /api/admin/plans/:id
   */
  async getPlan(planId: number | string): Promise<ApiResponse<PlanResponse>> {
    return this.client.get(`/api/v1/admin/plans/${planId}`);
  }

  /**
   * Create new plan
   * Route: POST /api/admin/plans
   */
  async createPlan(planData: CreatePlanRequest): Promise<ApiResponse<PlanResponse>> {
    return this.client.post('/api/v1/admin/plans', planData);
  }

  /**
   * Update existing plan
   * Route: PUT /api/admin/plans/:id
   */
  async updatePlan(planId: number | string, planData: UpdatePlanRequest): Promise<ApiResponse<PlanResponse>> {
    return this.client.put(`/api/v1/admin/plans/${planId}`, planData);
  }

  /**
   * Delete plan
   * Route: DELETE /api/admin/plans/:id
   */
  async deletePlan(planId: number | string): Promise<ApiResponse<void>> {
    return this.client.delete(`/api/v1/admin/plans/${planId}`);
  }

  /**
   * Get plan analytics
   * Route: GET /api/admin/plans/:id/analytics
   */
  async getPlanAnalytics(planId: number | string, period?: string): Promise<ApiResponse<PlanAnalyticsResponse>> {
    const params = period ? { period } : {};
    return this.client.get(`/api/v1/admin/plans/${planId}/analytics`, params);
  }

  /**
   * Get plan options for forms (helper)
   */
  async getPlanOptions(): Promise<Array<{
    value: number;
    label: string;
    category: string;
    audience: string;
  }>> {
    const response = await this.getPlans({ is_active: true });
    if (response.success && response.data?.plans) {
      return response.data.plans.map((plan: PlanResponse) => ({
        value: Number(plan.id),
        label: `${plan.name} - ${plan.current_price} ${plan.currency}`,
        category: plan.plan_category,
        audience: plan.target_audience
      }));
    }
    return [];
  }

  // ============================================================================
  // SUBSCRIPTION MANAGEMENT
  // ============================================================================

  /**
   * Get subscriptions with filtering
   * Route: GET /api/v1/payments/admin/subscriptions
   */
  async getSubscriptions(params: {
    limit?: number;
    status?: string;
    access_context?: string;
    plan_id?: number;
  } = {}): Promise<ApiResponse<{ subscriptions: SubscriptionResponse[] }>> {
    return this.client.get('/api/v1/payments/admin/subscriptions', params);
  }

  /**
   * Get single subscription
   * Route: GET /api/v1/payments/subscriptions/:id
   */
  async getSubscription(subscriptionId: string): Promise<ApiResponse<SubscriptionResponse>> {
    return this.client.get(`/api/v1/payments/subscriptions/${subscriptionId}`);
  }

  /**
   * Create subscription
   * Route: POST /api/v1/admin/subscriptions
   */
  async createSubscription(data: CreateSubscriptionRequest): Promise<ApiResponse<SubscriptionResponse>> {
    // Map user_id to wallet_address for backend compatibility
    // permission_group_name is REQUIRED by backend
    const payload = {
      wallet_address: data.user_id,
      plan_id: data.plan_id,
      permission_group_name: (data as any).permission_group_name || `Plan ${data.plan_id}`,
      access_context: data.access_context,
      api_key_name: data.api_key_name,
      expires_at: data.expires_at,
      auto_renew: data.auto_renew,
      metadata: data.metadata,
    };
    return this.client.post('/api/v1/admin/subscriptions', payload);
  }

  /**
   * Update subscription
   * Route: PUT /api/v1/admin/subscriptions/:id
   */
  async updateSubscription(subscriptionId: string, data: UpdateSubscriptionRequest): Promise<ApiResponse<SubscriptionResponse>> {
    return this.client.put(`/api/v1/admin/subscriptions/${subscriptionId}`, data);
  }

  /**
   * Cancel subscription
   * Route: POST /api/v1/admin/subscriptions/:id/cancel
   */
  async cancelSubscription(subscriptionId: string): Promise<ApiResponse<void>> {
    return this.client.post(`/api/v1/admin/subscriptions/${subscriptionId}/cancel`);
  }

  // ============================================================================
  // API KEY MANAGEMENT
  // ============================================================================

  /**
   * List API keys
   * Route: GET /api/v1/admin/developer-portal/api-keys
   * Supports ?wallet=0x... to filter by wallet address
   */
  async listApiKeys(params: {
    limit?: number;
    offset?: number;
    status?: string;
    client_name?: string;
    wallet?: string; // Filter by wallet address
  } = {}): Promise<ApiResponse<{ api_keys: ApiKeyResponse[]; total: number }>> {
    return this.client.get('/api/v1/admin/developer-portal/api-keys', params);
  }

  /**
   * Get API key details
   * Route: GET /api/v1/admin/developer-portal/api-keys/:id
   */
  async getApiKey(keyId: string): Promise<ApiResponse<ApiKeyResponse>> {
    return this.client.get(`/api/v1/admin/developer-portal/api-keys/${keyId}`);
  }

  /**
   * Create API key
   * Route: POST /api/v1/admin/developer-portal/api-keys
   */
  async createApiKey(keyData: ApiKeyRequest): Promise<ApiResponse<ApiKeyResponse>> {
    return this.client.post('/api/v1/admin/developer-portal/api-keys', keyData);
  }

  /**
   * Update API key
   * Route: PUT /api/v1/admin/developer-portal/api-keys/:id
   */
  async updateApiKey(keyId: string, keyData: Partial<ApiKeyRequest>): Promise<ApiResponse<ApiKeyResponse>> {
    return this.client.put(`/api/v1/admin/developer-portal/api-keys/${keyId}`, keyData);
  }

  /**
   * Revoke API key
   * Route: POST /api/v1/admin/developer-portal/api-keys/:id/revoke
   */
  async revokeApiKey(keyId: string, reason: string): Promise<ApiResponse<{ success: boolean }>> {
    return this.client.post(`/api/v1/admin/developer-portal/api-keys/${keyId}/revoke`, { reason });
  }

  /**
   * Update API key expiration date
   * Route: PATCH /api/v1/admin/developer-portal/api-keys/:id/expiration
   */
  async updateApiKeyExpiration(keyId: string, expiresAt: string | null): Promise<ApiResponse<ApiKeyResponse>> {
    return this.client.patch(`/api/v1/admin/developer-portal/api-keys/${keyId}/expiration`, {
      expires_at: expiresAt,
    });
  }

  /**
   * List API keys expiring within the specified number of days
   * Route: GET /api/v1/admin/developer-portal/api-keys/expiring
   */
  async listExpiringApiKeys(params: {
    days?: number; // Default 7 days
    limit?: number;
    offset?: number;
  } = {}): Promise<ApiResponse<{
    api_keys: ApiKeyResponse[];
    total: number;
    days_ahead: number;
  }>> {
    return this.client.get('/api/v1/admin/developer-portal/api-keys/expiring', params);
  }

  /**
   * Regenerate API key
   * Route: POST /api/v1/admin/developer-portal/api-keys/:id/regenerate
   */
  async regenerateApiKey(keyId: string): Promise<ApiResponse<ApiKeyResponse>> {
    return this.client.post(`/api/v1/admin/developer-portal/api-keys/${keyId}/regenerate`);
  }

  // ============================================================================
  // MODULE MANAGEMENT
  // ============================================================================

  /**
   * Get modules
   * Route: GET /api/v1/admin/developer-portal/modules
   */
  async getModules(params: {
    status?: string;
    category?: string;
  } = {}): Promise<ApiResponse<{ modules: Module[]; total: number }>> {
    return this.client.get('/api/v1/admin/developer-portal/modules', params);
  }

  /**
   * Get module details
   * Route: GET /api/v1/admin/developer-portal/modules/:id
   */
  async getModule(moduleId: string): Promise<ApiResponse<Module>> {
    return this.client.get(`/api/v1/admin/developer-portal/modules/${moduleId}`);
  }

  /**
   * Update module
   * Route: PUT /api/v1/admin/developer-portal/modules/:id
   */
  async updateModule(moduleId: string, moduleData: Partial<Module>): Promise<ApiResponse<Module>> {
    return this.client.put(`/api/v1/admin/developer-portal/modules/${moduleId}`, moduleData);
  }

  // ============================================================================
  // USAGE ANALYTICS
  // ============================================================================

  /**
   * Get API key usage stats
   * Route: GET /api/v1/admin/developer-portal/api-keys/:id/usage
   */
  async getApiKeyUsageStats(keyId: string, params: {
    period?: string; // "24h", "7d", "30d", "90d"
    granularity?: string; // "hour", "day", "week"
  } = {}): Promise<ApiResponse<any>> {
    return this.client.get(`/api/v1/admin/developer-portal/api-keys/${keyId}/usage`, params);
  }

  /**
   * Get module usage stats
   * Route: GET /api/v1/admin/developer-portal/modules/:id/usage
   */
  async getModuleUsageStats(moduleId: string, params: {
    period?: string;
    granularity?: string;
  } = {}): Promise<ApiResponse<any>> {
    return this.client.get(`/api/v1/admin/developer-portal/modules/${moduleId}/usage`, params);
  }

  /**
   * Get developer portal stats
   * Route: GET /api/v1/admin/developer-portal/stats
   */
  async getDeveloperPortalStats(): Promise<ApiResponse<any>> {
    return this.client.get('/api/v1/admin/developer-portal/stats');
  }

  // ============================================================================
  // USER-FACING API KEY MANAGEMENT (for main frontend)
  // ============================================================================

  /**
   * List user's own API keys
   * Route: GET /api/v1/developer-portal/my-keys
   */
  async listMyApiKeys(params: {
    limit?: number;
    offset?: number;
    status?: string;
  } = {}): Promise<ApiResponse<{ api_keys: ApiKeyResponse[]; total: number }>> {
    return this.client.get('/api/v1/developer-portal/my-keys', params);
  }

  /**
   * Create API key for current user
   * Route: POST /api/v1/developer-portal/my-keys
   */
  async createMyApiKey(keyData: {
    client_name: string;
    client_description?: string;
    group_ids?: string[];
    /** Individual permission strings to assign */
    permissions?: string[];
    /** @deprecated Use group_ids instead */
    permission_group_ids?: string[];
    ip_restrictions?: string[];
    expires_at?: string;
  }): Promise<ApiResponse<ApiKeyResponse>> {
    // Merge permission_group_ids into group_ids for backward compatibility
    const payload = {
      ...keyData,
      group_ids: keyData.group_ids || keyData.permission_group_ids
    };
    return this.client.post('/api/v1/developer-portal/my-keys', payload);
  }

  /**
   * Get user's API key details
   * Route: GET /api/v1/developer-portal/my-keys/:id
   */
  async getMyApiKey(keyId: string): Promise<ApiResponse<ApiKeyResponse>> {
    return this.client.get(`/api/v1/developer-portal/my-keys/${keyId}`);
  }

  /**
   * Revoke user's own API key
   * Route: DELETE /api/v1/developer-portal/my-keys/:id
   */
  async revokeMyApiKey(keyId: string, reason?: string): Promise<ApiResponse<{ success: boolean }>> {
    // Use POST with reason in body (DELETE doesn't support body in this client)
    return this.client.post(`/api/v1/developer-portal/my-keys/${keyId}/revoke`, { reason: reason || 'Revoked by owner' });
  }

  /**
   * List available groups for API key creation
   * Route: GET /api/v1/developer-portal/available-groups
   */
  async getAvailableGroups(): Promise<ApiResponse<{
    groups: Array<{
      id: string;
      name: string;
      slug: string;
      description: string;
      permissions: string[];
      group_type: string;
      is_active: boolean;
    }>;
  }>> {
    return this.client.get('/api/v1/developer-portal/available-groups');
  }

  /**
   * Get user's assigned permission groups with metadata
   * Route: GET /api/v1/developer-portal/my-groups
   */
  async getMyGroups(): Promise<ApiResponse<{
    groups: Array<{
      id: string;
      name: string;
      slug: string;
      description: string;
      group_type: string;
      permissions: string[];
      expires_at: string | null;
      rate_limit_per_minute: number | null;
      rate_limit_per_day: number | null;
      assigned_at: string;
    }>;
    total_api_keys: number;
    total_requests: number;
  }>> {
    return this.client.get('/api/v1/developer-portal/my-groups');
  }

  /**
   * @deprecated Use getAvailableGroups instead
   */
  async getAvailablePermissionGroups(): Promise<ApiResponse<any>> {
    return this.getAvailableGroups();
  }

  // ============================================================================
  // USAGE ANALYTICS (User Facing)
  // ============================================================================

  /**
   * Get user's aggregated usage stats
   * Route: GET /api/v1/developer-portal/stats
   */
  async getUserUsageStats(): Promise<ApiResponse<{
    total_requests: number;
    average_success_rate: number;
    requests_24h: number;
    error_rate_24h: number;
  }>> {
    return this.client.get('/api/v1/developer-portal/stats');
  }

  /**
   * Get usage history
   * Route: GET /api/v1/developer-portal/usage-history
   */
  async getUserUsageHistory(days: number = 7): Promise<ApiResponse<Array<{
    bucket: string;
    count: number;
  }>>> {
    return this.client.get('/api/v1/developer-portal/usage-history', { days });
  }

  /**
   * Get top endpoints
   * Route: GET /api/v1/developer-portal/top-endpoints
   */
  async getUserTopEndpoints(days: number = 7): Promise<ApiResponse<Array<{
    endpoint: string;
    method: string;
    count: number;
  }>>> {
    return this.client.get('/api/v1/developer-portal/top-endpoints', { days });
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/**
 * Create plans API client
 */
export function createPlansClient(client: UnifiedApiClient): PlansAPIClient {
  return new PlansAPIClient(client);
}

/**
 * Create plans client with automatic platform detection
 */
export function createPlatformPlansClient(): PlansAPIClient {
  const { createAdminApiClient } = require('../utils/api-client');
  return new PlansAPIClient(createAdminApiClient());
}

// ============================================================================
// TYPE GUARDS
// ============================================================================

export function isApiSuccess<T>(response: ApiResponse<T>): response is ApiResponse<T> & { success: true; data: T } {
  return response.success && response.data !== undefined;
}

// Type alias for backward compatibility with useApiClient
export type PlansApi = PlansAPIClient;

