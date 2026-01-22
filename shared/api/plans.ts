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

import { ApiResponse, PaginatedResponse } from '../types/api';
import { UnifiedApiClient } from '../utils/api-client';

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
  metadata?: Record<string, any>;
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
  metadata?: Record<string, any>;
}

export interface UpdatePlanRequest {
  name?: string;
  description?: string;
  permissions?: string[];
  is_active?: boolean;
  metadata?: Record<string, any>;
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
  async getPublicPlans(filters?: { category?: string }): Promise<ApiResponse<PublicPlan[]>> {
    return this.client.get<PublicPlan[]>('/api/public/plans', filters);
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
