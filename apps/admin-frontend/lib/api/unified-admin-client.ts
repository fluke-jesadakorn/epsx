/**
 * Unified Admin API Client
 * Consolidates all API functionality into a single, consistent interface
 * Replaces: api-client.ts, admin-client.ts, adminApiService.ts, and related files
 */

import { env } from '@/config/env';
import { getBackendUrl } from '../../../../shared/utils/url-resolver';
import type { ApiResponse, ApiError } from '../../../../shared/types/api';

// Using shared API types from shared/types/api

// Request Configuration
export interface RequestConfig extends RequestInit {
  timeout?: number;
  serverSide?: boolean;
}

// Dynamic Plan Management Types
export interface CreatePlanRequest {
  name: string;
  description?: string;
  plan_type: string;
  current_price: number;
  currency: string;
  target_audience: string; // "web_users", "api_developers", "enterprises"
  billing_model: string;   // "subscription", "pay_per_use", "hybrid"
  plan_category: string;   // "standard", "api", "enterprise", "custom"
  features: PlanFeatureRequest[];
  metadata?: Record<string, any>;
}

export interface PlanFeatureRequest {
  context_name: string; // "web_app", "api_access", "admin_interface"
  feature_key: string;
  feature_config: Record<string, any>;
  resource_cost: number;
  is_active: boolean;
}

export interface UpdatePlanRequest {
  name?: string;
  description?: string;
  current_price?: number;
  is_active?: boolean;
  features?: PlanFeatureRequest[];
  metadata?: Record<string, any>;
}

export interface PlanResponse {
  id: number;
  name: string;
  description?: string;
  plan_type: string;
  current_price: number;
  currency: string;
  target_audience: string;
  billing_model: string;
  plan_category: string;
  is_active: boolean;
  features: PlanFeatureResponse[];
  metadata?: Record<string, any>;
  created_at: string;
  updated_at?: string;
  subscriber_count: number;
  revenue_last_30_days: number;
}

export interface PlanFeatureResponse {
  id: number;
  context_name: string;
  feature_key: string;
  feature_config: Record<string, any>;
  resource_cost: number;
  is_active: boolean;
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

// Subscription Management Types
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

// API Key Management Types
export interface ApiKeyRequest {
  client_name: string;
  client_description?: string;
  client_contact_email?: string;
  allowed_modules: ApiKeyModuleConfig[];
  ip_restrictions?: string[];
  expires_at?: string;
  rate_limits?: Record<string, number>;
}

export interface ApiKeyModuleConfig {
  module_id: string;
  module_name: string;
  access_level: string; // "bronze", "silver", "gold", "platinum", "enterprise"
  custom_quotas?: Record<string, any>;
}

export interface ApiKeyResponse {
  id: string;
  key_prefix: string;
  full_key?: string; // Only returned on creation
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

// Base API Client Class
export class UnifiedAdminClient {
  private baseURL: string;
  private token?: string;
  private isServerSide: boolean;

  constructor(baseURL?: string, token?: string, serverSide = false) {
    this.baseURL = baseURL || this.getBackendUrl();
    this.token = token;
    this.isServerSide = serverSide;
  }

  private getBackendUrl(): string {
    return getBackendUrl('client');
  }

  // Core HTTP Methods
  private async makeRequest<T>(url: string, config: RequestConfig = {}): Promise<ApiResponse<T>> {
    const fullUrl = `${this.baseURL}${url}`;
    const { timeout = 30000, serverSide, ...options } = config;

    // Handle authentication
    let authToken = this.token;
    if (!authToken && (serverSide || this.isServerSide)) {
      try {
        // For server-side requests, try to get token from cookies
        if (typeof window === 'undefined') {
          const { cookies } = await import('next/headers');
          const cookieStore = await cookies();
          authToken = cookieStore.get('access_token')?.value;
        }
      } catch (error) {
        console.warn('Could not access cookies for server-side request');
      }
    }

    const headers = {
      'Content-Type': 'application/json',
      ...(authToken && { Authorization: `Bearer ${authToken}` }),
      ...options.headers,
    };

    const requestConfig: RequestInit = {
      cache: serverSide ? 'no-store' : 'default',
      credentials: serverSide ? undefined : 'include',
      ...options,
      headers,
    };

    try {
      console.log('🔍 Making request:', { url: fullUrl, config: requestConfig });
      const response = await fetch(fullUrl, requestConfig);
      
      console.log('📡 Response received:', { 
        status: response.status, 
        statusText: response.statusText, 
        ok: response.ok,
        headers: Object.fromEntries(response.headers.entries())
      });
      
      // Handle authentication errors
      if (response.status === 401) {
        if (typeof window !== 'undefined') {
          window.location.href = '/login';
        }
        return {
          success: false,
          error: 'Unauthorized - please log in again',
          status: 401
        };
      }
      
      let data = null;
      try {
        if (response.ok) {
          const text = await response.text();
          console.log('📄 Response text length:', text.length, 'first 200 chars:', text.substring(0, 200));
          
          if (!text || text.trim() === '') {
            console.error('❌ Empty response body');
            return {
              success: false,
              error: 'Server returned empty response',
              status: response.status
            };
          }
          
          data = JSON.parse(text);
        } else {
          // Handle non-OK responses
          const text = await response.text();
          console.log('❌ Error response text:', text);
          try {
            data = text ? JSON.parse(text) : null;
          } catch {
            data = null;
          }
        }
      } catch (parseError) {
        console.error('❌ JSON parse error:', parseError);
        return {
          success: false,
          error: 'Failed to parse response JSON',
          status: response.status
        };
      }
      
      if (!response.ok) {
        const errorMessage = data?.message || `HTTP error: ${response.status} ${response.statusText}`;
        console.error('❌ HTTP error:', { status: response.status, message: errorMessage, data });
        return {
          success: false,
          error: errorMessage,
          status: response.status
        };
      }

      console.log('✅ Successful response:', { data: data });
      return {
        success: true,
        data,
        status: response.status
      };
    } catch (error) {
      console.error('API request failed:', { url: fullUrl, error });
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Network error',
        status: 0
      };
    }
  }

  async get<T>(url: string, params?: Record<string, any>, config?: RequestConfig): Promise<ApiResponse<T>> {
    const queryString = params ? '?' + new URLSearchParams(params).toString() : '';
    return this.makeRequest<T>(`${url}${queryString}`, { method: 'GET', ...config });
  }

  async post<T>(url: string, data?: any, config?: RequestConfig): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(url, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
      ...config,
    });
  }

  async put<T>(url: string, data?: any, config?: RequestConfig): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(url, {
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
      ...config,
    });
  }

  async delete<T>(url: string, config?: RequestConfig): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(url, { method: 'DELETE', ...config });
  }

  // Authentication Management
  setAuthToken(token: string) {
    this.token = token;
  }

  removeAuthToken() {
    this.token = undefined;
  }

  // User Management API
  async getUsers(params: { offset?: number; limit?: number; search?: string } = {}) {
    const { offset = 0, limit = 50, search } = params;
    const queryParams: Record<string, any> = { offset, limit };
    
    // Add search parameter if provided
    if (search && search.trim()) {
      queryParams.search = search;
    }
    
    return this.get('/api/v1/admin/users', queryParams);
  }

  async getUser(userId: string) {
    return this.get(`/api/v1/admin/users/${userId}`);
  }

  async createUser(userData: {
    email: string;
    permissions: string[];
    display_name?: string;
  }) {
    return this.post('/api/v1/admin/users', userData);
  }

  async updateUser(userId: string, userData: any) {
    return this.put(`/api/v1/admin/users/${userId}`, userData);
  }

  async deleteUser(userId: string) {
    return this.delete(`/api/v1/admin/users/${userId}`);
  }

  async searchUsers(query: string) {
    return this.get('/api/v1/admin/users/search', { q: query });
  }

  async getUserStats() {
    return this.get('/api/v1/admin/analytics/user-statistics');
  }

  // Permission Management API
  async getUserPermissions(userId: string) {
    return this.get(`/api/v1/admin/users/${userId}/permissions`);
  }

  async grantPermission(userId: string, permission: string, expiresAt?: string) {
    return this.post('/api/v1/admin/permissions/grant', {
      user_id: userId,
      permission,
      expires_at: expiresAt
    });
  }

  async revokePermission(userId: string, permission: string) {
    return this.post('/api/v1/admin/permissions/revoke', {
      user_id: userId,
      permission
    });
  }

  async bulkGrantPermissions(userIds: string[], permissions: string[], expiresAt?: string) {
    return this.post('/api/v1/admin/permissions/bulk-grant', {
      user_ids: userIds,
      permissions,
      expires_at: expiresAt
    });
  }

  async getPermissionAnalytics() {
    return this.get('/api/v1/admin/analytics/permissions');
  }

  async getPermissionExpiryStatus(userId: string) {
    return this.get(`/api/v1/admin/users/${userId}/permissions/expiry-status`);
  }

  // Notification Management API  
  async getNotifications(params?: {
    page?: number;
    limit?: number;
    type?: string;
    priority?: string;
    read?: boolean;
    userId?: string;
    startDate?: string;
    endDate?: string;
  }) {
    return this.get('/api/v1/notifications', params);
  }

  async getNotification(id: string) {
    return this.get(`/api/v1/notifications/${id}`);
  }

  async createNotification(data: {
    title: string;
    message: string;
    type: string;
    priority: string;
    userId?: string;
    userIds?: string[];
    actionUrl?: string;
    metadata?: Record<string, any>;
  }) {
    return this.post('/api/v1/notifications', data);
  }

  async updateNotification(id: string, data: any) {
    return this.put(`/api/v1/notifications/${id}`, data);
  }

  async deleteNotification(id: string) {
    return this.delete(`/api/v1/notifications/${id}`);
  }

  async markNotificationRead(id: string) {
    return this.put(`/api/v1/notifications/${id}/read`);
  }

  async broadcastNotification(data: {
    title: string;
    message: string;
    type: string;
    priority: string;
    userIds?: string[];
    allUsers?: boolean;
  }) {
    return this.post('/api/v1/notifications/broadcast', data);
  }

  async getNotificationStats(userId?: string) {
    return this.get('/api/v1/notifications/stats', userId ? { userId } : {});
  }

  // Analytics API
  async getEPSRankings() {
    return this.get('/api/v1/analytics/eps-rankings');
  }

  async getEPSHealth() {
    return this.get('/api/v1/analytics/eps-rankings/health');
  }

  async getPerformanceMetrics() {
    return this.get('/api/v1/admin/analytics/performance');
  }

  async getDashboardData() {
    return this.get('/api/v1/admin/analytics/dashboard');
  }

  async getCacheStats() {
    return this.get('/api/v1/admin/cache/stats');
  }

  // System Management API
  async getSystemConfig() {
    return this.get('/api/v1/settings/system');
  }

  async getFeatureFlags() {
    return this.get('/api/v1/settings/feature-flags');
  }

  async updateSystemConfig(config: Record<string, any>) {
    return this.put('/api/v1/settings/system', config);
  }

  // Stock Ranking API
  async getStockRankingPackages() {
    return this.get('/api/v1/admin/stock-ranking/packages');
  }

  async assignStockRankingPackage(userId: string, packageId: string, expiresAt?: string) {
    return this.post('/api/v1/admin/stock-ranking/assign', {
      userId,
      packageId,
      expiresAt
    });
  }

  async getStockRankingAssignments() {
    return this.get('/api/v1/admin/stock-ranking/assignments');
  }

  async extendStockRankingAssignment(assignmentId: string, newExpiresAt: string) {
    return this.put(`/api/v1/admin/stock-ranking/assignments/${assignmentId}/extend`, {
      expires_at: newExpiresAt
    });
  }

  async revokeStockRankingAssignment(assignmentId: string) {
    return this.delete(`/api/v1/admin/stock-ranking/assignments/${assignmentId}`);
  }

  // Dynamic Plan Management API
  async getPlans(params: {
    limit?: number;
    offset?: number;
    plan_category?: string;
    target_audience?: string;
    is_active?: boolean;
  } = {}) {
    return this.get('/api/v1/admin/plans', params);
  }

  async getPlan(planId: number) {
    return this.get(`/api/v1/admin/plans/${planId}`);
  }

  async createPlan(planData: CreatePlanRequest) {
    return this.post('/api/v1/admin/plans', planData);
  }

  async updatePlan(planId: number, planData: UpdatePlanRequest) {
    return this.put(`/api/v1/admin/plans/${planId}`, planData);
  }

  async deletePlan(planId: number) {
    return this.delete(`/api/v1/admin/plans/${planId}`);
  }

  async getPlanAnalytics(planId: number, period?: string) {
    const params = period ? { period } : {};
    return this.get(`/api/v1/admin/plans/${planId}/analytics`, params);
  }

  // Subscription Management API
  async getSubscriptions(params: {
    limit?: number;
    status?: string;
    access_context?: string;
    plan_id?: number;
  } = {}) {
    return this.get('/api/v1/admin/subscriptions', params);
  }

  async getSubscription(subscriptionId: string) {
    return this.get(`/api/v1/admin/subscriptions/${subscriptionId}`);
  }

  async createSubscription(subscriptionData: CreateSubscriptionRequest) {
    return this.post('/api/v1/admin/subscriptions', subscriptionData);
  }

  async updateSubscription(subscriptionId: string, subscriptionData: UpdateSubscriptionRequest) {
    return this.put(`/api/v1/admin/subscriptions/${subscriptionId}`, subscriptionData);
  }

  async cancelSubscription(subscriptionId: string) {
    return this.post(`/api/v1/admin/subscriptions/${subscriptionId}/cancel`);
  }

  // Helper method for getting plan options for forms
  async getPlanOptions() {
    const response = await this.get('/api/v1/admin/plans', { is_active: true });
    if (response.success && response.data?.plans) {
      return response.data.plans.map((plan: any) => ({
        value: plan.id,
        label: `${plan.name} - ${plan.current_price} ${plan.currency}`,
        category: plan.plan_category,
        audience: plan.target_audience
      }));
    }
    return [];
  }

  // API Key Management Methods
  async listApiKeys(params: {
    limit?: number;
    offset?: number;
    status?: string;
    client_name?: string;
  } = {}) {
    return this.get<{ api_keys: ApiKeyResponse[]; total: number }>('/api/v1/admin/developer-portal/api-keys', params);
  }

  async getApiKey(keyId: string) {
    return this.get<ApiKeyResponse>(`/api/v1/admin/developer-portal/api-keys/${keyId}`);
  }

  async createApiKey(keyData: ApiKeyRequest) {
    return this.post<ApiKeyResponse>('/api/v1/admin/developer-portal/api-keys', keyData);
  }

  async updateApiKey(keyId: string, keyData: Partial<ApiKeyRequest>) {
    return this.put<ApiKeyResponse>(`/api/v1/admin/developer-portal/api-keys/${keyId}`, keyData);
  }

  async revokeApiKey(keyId: string, reason: string) {
    return this.post<{ success: boolean }>(`/api/v1/admin/developer-portal/api-keys/${keyId}/revoke`, { reason });
  }

  async regenerateApiKey(keyId: string) {
    return this.post<ApiKeyResponse>(`/api/v1/admin/developer-portal/api-keys/${keyId}/regenerate`);
  }

  // Module Management Methods
  async getModules(params: {
    status?: string;
    category?: string;
  } = {}) {
    return this.get<{ modules: Module[]; total: number }>('/api/v1/admin/developer-portal/modules', params);
  }

  async getModule(moduleId: string) {
    return this.get<Module>(`/api/v1/admin/developer-portal/modules/${moduleId}`);
  }

  async updateModule(moduleId: string, moduleData: Partial<Module>) {
    return this.put<Module>(`/api/v1/admin/developer-portal/modules/${moduleId}`, moduleData);
  }

  // Usage Analytics Methods
  async getApiKeyUsageStats(keyId: string, params: {
    period?: string; // "24h", "7d", "30d", "90d"
    granularity?: string; // "hour", "day", "week"
  } = {}) {
    return this.get(`/api/v1/admin/developer-portal/api-keys/${keyId}/usage`, params);
  }

  async getModuleUsageStats(moduleId: string, params: {
    period?: string;
    granularity?: string;
  } = {}) {
    return this.get(`/api/v1/admin/developer-portal/modules/${moduleId}/usage`, params);
  }

  async getDeveloperPortalStats() {
    return this.get('/api/v1/admin/developer-portal/stats');
  }
}

// Factory Functions
export function createAdminClient(baseURL?: string, token?: string): UnifiedAdminClient {
  return new UnifiedAdminClient(baseURL, token, false);
}

export function createServerAdminClient(baseURL?: string, token?: string): UnifiedAdminClient {
  return new UnifiedAdminClient(baseURL, token, true);
}

// Default Instances
export const adminClient = createAdminClient();
export const serverAdminClient = createServerAdminClient();

// Type Guards
export function isApiError(error: any): error is ApiError {
  return error && typeof error.message === 'string' && typeof error.status === 'number';
}

export function isApiSuccess<T>(response: ApiResponse<T>): response is ApiResponse<T> & { success: true; data: T } {
  return response.success && !!response.data;
}

// Error Handler Utility
export class APIError extends Error {
  constructor(public status: number, message: string, public code?: string) {
    super(message);
    this.name = 'APIError';
  }
}