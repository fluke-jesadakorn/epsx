/**
 * UNIFIED PLANS API CLIENT
 *
 * Plan assignment and management endpoints for Web3 permission plans.
 * Consolidates plan-related API calls across EPSX applications.
 *
 * Features:
 * - Plan assignment/removal
 * - Plan listing and filtering
 * - Plan membership queries
 * - Bulk plan operations
 */

import type { ApiResponse, PaginatedResponse } from '../types/api';
import { isApiSuccess } from '../types/api';
import type { PlanAccessData } from '../types/payment';
import type { UnifiedApiClient } from '../utils/api-client';

export { isApiSuccess };

// ============================================================================
// TYPES
// ============================================================================

export interface Plan {
  id: string;
  name: string;
  description?: string;
  permissions: string[];
  member_count?: number;
  created_at: string;
  updated_at?: string;
  is_active: boolean;
  metadata?: Record<string, unknown>;
  /**
   * Price for plans that are fixed amount
   */
  current_price?: string | number;
  /**
   * List of features associated with the plan
   */
  features?: unknown[];
}

export interface SubscriptionResponse {
  id: string;
  wallet_address: string;
  user_id: string;
  plan_id: string;
  plan_name: string;
  permission_plan_name: string;
  permissions_granted: string[];
  plan_type: string;
  access_context: 'internal' | 'external' | 'both';
  api_key?: string;
  api_key_name?: string;
  status: 'active' | 'cancelled' | 'expired' | 'paused';
  expires_at?: string;
  auto_renew: boolean;
  created_at: string;
  started_at?: string;
  updated_at: string;
  metadata?: Record<string, unknown>;
  current_usage?: Record<string, unknown>;
  quota_limits?: Record<string, unknown>;
}

export interface PublicPlan {
  id: string;
  name: string;
  plan_type: string;
  current_price: string;
  effective_price: number;
  promotion_active: boolean;
  promotion_status: string;
  promotion_discount: number;
  promotion_ends_at?: string;
  currency: string;
  billing_cycle: string;
  features: string[];
  permissions: string[];
  is_active: boolean;
  display_order: number;
  plan_group?: string;
}

export interface PlanMembership {
  plan_id: string;
  name: string;
  wallet_address: string;
  assigned_at: string;
  assigned_by?: string;
  expires_at?: number;
  is_active: boolean;
  permissions: string[];
}

export interface PlanFilters {
  search?: string;
  is_active?: boolean;
  has_permission?: string;
  min_members?: number;
  max_members?: number;
  limit?: number;
  offset?: number;
}

export interface MembershipFilters {
  wallet_address?: string;
  plan_id?: string;
  is_active?: boolean;
  assigned_after?: string;
  expires_before?: number;
  limit?: number;
  offset?: number;
}

export interface SubscriptionListQuery {
  page?: number;
  limit?: number;
  status?: string;
  access_context?: string;
  search?: string;
}

export interface AssignPlanRequest {
  wallet_address: string;
  plan_id: string;
  expires_at?: number;
  notes?: string;
  notify_user?: boolean;
}

export interface RemovePlanRequest {
  wallet_address: string;
  plan_id: string;
  reason?: string;
  notify_user?: boolean;
}

export interface BulkAssignRequest {
  wallet_addresses: string[];
  plan_ids: string[];
  expires_at?: number;
  notes?: string;
}

export interface BulkRemoveRequest {
  wallet_addresses: string[];
  plan_ids: string[];
  reason?: string;
}

export interface CreatePlanRequest {
  name: string;
  description?: string;
  permissions: string[];
  metadata?: Record<string, unknown>;
}

export interface UpdatePlanRequest {
  name?: string;
  description?: string;
  permissions?: string[];
  is_active?: boolean;
  metadata?: Record<string, unknown>;
}

export interface CreateSubscriptionRequest {
  user_id: string;
  plan_id: string | number;
  access_context: 'internal' | 'external' | 'both';
  api_key_name?: string;
  expires_at?: string;
  auto_renew: boolean;
  metadata?: Record<string, unknown>;
}

export interface UpdateSubscriptionRequest {
  plan_id?: string;
  status?: 'active' | 'cancelled' | 'expired' | 'paused';
  expires_at?: string;
  auto_renew?: boolean;
  metadata?: Record<string, unknown>;
}

export interface PlanStats {
  total_plans: number;
  active_plans: number;
  total_memberships: number;
  active_memberships: number;
  by_plan: Record<string, number>;
  recent_assignments: number;
  recent_removals: number;
}

export interface ApiKeyResponse {
  id: string;
  client_name: string;
  key_prefix?: string;
  key_preview: string;
  status: 'active' | 'revoked' | 'expired';
  total_requests: number;
  allowed_modules: Array<{ module_id: string; module_name: string }>;
  expires_at: string | null;
  created_at: string;
  permission_groups?: Array<{ id: string; name: string }>;
}

export interface Module {
  id: string;
  name: string;
  display_name: string;
  description?: string;
  status: 'active' | 'inactive';
  category: string;
}

// ============================================================================
// PLANS API CLASS
// ============================================================================

export class PlansApi {
  private client: UnifiedApiClient;

  constructor(client: UnifiedApiClient) {
    this.client = client;
  }

  // ============================================================================
  // PLAN LISTING
  // ============================================================================

  /**
   * List all plans
   * GET /api/admin/plans
   */
  async listPlans(filters?: PlanFilters): Promise<ApiResponse<PaginatedResponse<Plan>>> {
    return this.client.get<PaginatedResponse<Plan>>('/api/admin/plans', filters);
  }

  /**
   * Get public plans
   * GET /api/public/plans
   */
  async getPublicPlans(filters?: { category?: string; group?: string; affiliate_code?: string }): Promise<ApiResponse<PublicPlan[]>> {
    return this.client.get<PublicPlan[]>('/api/public/plans', filters as Record<string, string>);
  }

  /**
   * Get plan by ID
   * GET /api/admin/plans/{plan_id}
   */
  async getPlan(plan_id: string): Promise<ApiResponse<Plan>> {
    return this.client.get<Plan>(`/api/admin/plans/${plan_id}`);
  }

  /**
   * Get plan members
   * GET /api/admin/plans/{plan_id}/members
   */
  async getPlanMembers(plan_id: string, filters?: { limit?: number; offset?: number }): Promise<ApiResponse<PaginatedResponse<PlanMembership>>> {
    return this.client.get<PaginatedResponse<PlanMembership>>(`/api/admin/plans/${plan_id}/members`, filters);
  }

  // ============================================================================
  // PLAN MANAGEMENT (Admin only)
  // ============================================================================

  /**
   * Create new plan
   * POST /api/admin/plans
   */
  async createPlan(data: CreatePlanRequest): Promise<ApiResponse<Plan>> {
    return this.client.post<Plan>('/api/admin/plans', data);
  }

  /**
   * Update plan
   * PUT /api/admin/plans/{plan_id}
   */
  async updatePlan(plan_id: string, data: UpdatePlanRequest): Promise<ApiResponse<Plan>> {
    return this.client.put<Plan>(`/api/admin/plans/${plan_id}`, data);
  }

  /**
   * Delete plan
   * DELETE /api/admin/plans/{plan_id}
   */
  async deletePlan(plan_id: string): Promise<ApiResponse<{ deleted: boolean }>> {
    return this.client.delete<{ deleted: boolean }>(`/api/admin/plans/${plan_id}`);
  }

  // ============================================================================
  // SUBSCRIPTION MANAGEMENT
  // ============================================================================

  /**
   * Create a new subscription
   * POST /api/admin/subscriptions
   */
  async createSubscription(data: CreateSubscriptionRequest): Promise<ApiResponse<SubscriptionResponse>> {
    return this.client.post<SubscriptionResponse>('/api/admin/subscriptions', data);
  }

  /**
   * List all subscriptions
   * GET /api/admin/subscriptions
   */
  async getSubscriptions(filters?: SubscriptionListQuery): Promise<ApiResponse<{ subscriptions: SubscriptionResponse[]; total: number }>> {
    return this.client.get<{ subscriptions: SubscriptionResponse[]; total: number }>('/api/admin/subscriptions', filters);
  }

  /**
   * Get single subscription
   * GET /api/admin/subscriptions/{subscription_id}
   */
  async getSubscription(subscription_id: string): Promise<ApiResponse<SubscriptionResponse>> {
    return this.client.get<SubscriptionResponse>(`/api/admin/subscriptions/${subscription_id}`);
  }

  /**
   * Update a subscription
   * PUT /api/admin/subscriptions/{subscription_id}
   */
  async updateSubscription(subscription_id: string, data: UpdateSubscriptionRequest): Promise<ApiResponse<SubscriptionResponse>> {
    return this.client.put<SubscriptionResponse>(`/api/admin/subscriptions/${subscription_id}`, data);
  }

  /**
   * Cancel a subscription
   * POST /api/admin/subscriptions/{subscription_id}/cancel
   */
  async cancelSubscription(subscription_id: string): Promise<ApiResponse<{ cancelled: boolean }>> {
    return this.client.post<{ cancelled: boolean }>(`/api/admin/subscriptions/${subscription_id}/cancel`);
  }

  // ============================================================================
  // MEMBERSHIP MANAGEMENT
  // ============================================================================

  /**
   * Assign wallet to plan
   * POST /api/admin/plans/assign
   */
  async assignToPlan(data: AssignPlanRequest): Promise<ApiResponse<{ assigned: boolean; membership: PlanMembership }>> {
    return this.client.post<{ assigned: boolean; membership: PlanMembership }>('/api/admin/plans/assign', data);
  }

  /**
   * Remove wallet from plan
   * POST /api/admin/plans/remove
   */
  async removeFromPlan(data: RemovePlanRequest): Promise<ApiResponse<{ removed: boolean }>> {
    return this.client.post<{ removed: boolean }>('/api/admin/plans/remove', data);
  }

  /**
   * Assign wallets to plans in bulk
   * POST /api/admin/plans/bulk/assign
   */
  async bulkAssignToPlans(data: BulkAssignRequest): Promise<ApiResponse<{ assigned_count: number; failed: string[] }>> {
    return this.client.post<{ assigned_count: number; failed: string[] }>('/api/admin/plans/bulk/assign', data);
  }

  /**
   * Remove wallets from plans in bulk
   * POST /api/admin/plans/bulk/remove
   */
  async bulkRemoveFromPlans(data: BulkRemoveRequest): Promise<ApiResponse<{ removed_count: number; failed: string[] }>> {
    return this.client.post<{ removed_count: number; failed: string[] }>('/api/admin/plans/bulk/remove', data);
  }

  // ============================================================================
  // MEMBERSHIP QUERIES
  // ============================================================================

  /**
   * Get wallet's plan memberships
   * GET /api/admin/memberships?wallet_address={address}
   */
  async getWalletMemberships(wallet_address: string): Promise<ApiResponse<PlanMembership[]>> {
    return this.client.get<PlanMembership[]>('/api/admin/memberships', { wallet_address });
  }

  /**
   * List all memberships with filters
   * GET /api/admin/memberships
   */
  async listMemberships(filters?: MembershipFilters): Promise<ApiResponse<PaginatedResponse<PlanMembership>>> {
    return this.client.get<PaginatedResponse<PlanMembership>>('/api/admin/memberships', filters);
  }

  /**
   * Check if wallet is in plan
   * POST /api/plans/check-membership
   */
  async checkMembership(wallet_address: string, plan_id: string): Promise<ApiResponse<{ is_member: boolean; membership?: PlanMembership }>> {
    return this.client.post<{ is_member: boolean; membership?: PlanMembership }>('/api/plans/check-membership', {
      wallet_address,
      plan_id
    });
  }

  /**
   * Get current user's plan access data
   * GET /api/payments/plans/my-plan-access
   */
  async getMyPlanAccess(): Promise<ApiResponse<PlanAccessData>> {
    return this.client.get<PlanAccessData>('/api/payments/plans/my-plan-access');
  }

  // ============================================================================
  // STATISTICS
  // ============================================================================

  /**
   * Get plan statistics
   * GET /api/admin/plans/stats
   */
  async getStats(): Promise<ApiResponse<PlanStats>> {
    return this.client.get<PlanStats>('/api/admin/plans/stats');
  }

  /**
   * Get plan activity history
   * GET /api/admin/plans/{plan_id}/history
   */
  async getPlanHistory(plan_id: string, filters?: { limit?: number }): Promise<ApiResponse<Array<{
    action: 'assigned' | 'removed';
    wallet_address: string;
    timestamp: string;
    performed_by?: string;
  }>>> {
    return this.client.get(`/api/admin/plans/${plan_id}/history`, filters);
  }

  // ============================================================================
  // API KEY & MODULE MANAGEMENT (Admin only)
  // ============================================================================

  /**
   * List all API keys
   * GET /api/admin/developer/keys
   */
  async listApiKeys(filters?: { status?: string; search?: string }): Promise<ApiResponse<{ api_keys: ApiKeyResponse[] }>> {
    return this.client.get<{ api_keys: ApiKeyResponse[] }>('/api/admin/developer/keys', filters);
  }

  /**
   * Get all available modules
   * GET /api/admin/developer/modules
   */
  async getModules(filters?: { status?: string }): Promise<ApiResponse<{ modules: Module[] }>> {
    return this.client.get<{ modules: Module[] }>('/api/admin/developer/modules', filters);
  }

  /**
   * Revoke an API key
   * POST /api/admin/developer/keys/{key_id}/revoke
   */
  async revokeApiKey(keyId: string, reason?: string): Promise<ApiResponse<{ success: boolean }>> {
    return this.client.post<{ success: boolean }>(`/api/admin/developer/keys/${keyId}/revoke`, { reason });
  }

  /**
   * Update API key expiration
   * PUT /api/admin/developer/keys/{key_id}/expiration
   */
  async updateApiKeyExpiration(keyId: string, expiresAt: string | null): Promise<ApiResponse<{ success: boolean }>> {
    return this.client.put<{ success: boolean }>(`/api/admin/developer/keys/${keyId}/expiration`, { expires_at: expiresAt });
  }

  /**
   * Create a new API key
   * POST /api/admin/developer/keys
   */
  async createApiKey(data: {
    client_name: string;
    client_description?: string;
    client_contact_email?: string;
    allowed_modules: string[];
    ip_restrictions?: string[];
    expires_at?: string;
  }): Promise<ApiResponse<ApiKeyResponse & { full_key: string }>> {
    return this.client.post<ApiKeyResponse & { full_key: string }>('/api/admin/developer/keys', data);
  }

  // ============================================================================
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/**
 * Create Plans API client
 */
export function createPlansClient(client: UnifiedApiClient): PlansApi {
  return new PlansApi(client);
}

// ============================================================================
// EXPORTS
// ============================================================================

export default PlansApi;
