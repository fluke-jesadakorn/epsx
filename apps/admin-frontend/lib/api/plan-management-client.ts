/**
 * ADMIN PLAN MANAGEMENT CLIENT
 *
 * Re-exports types from shared and provides admin-specific plan operations.
 * This wrapper adds admin-frontend specific functionality on top of the shared WalletsApi.
 */

'use client';

import { extractArrayOrEmpty } from '@/shared/api';
import { API_ROUTES } from '@/shared/config/route-constants';
import { adminApiClient } from '../api-client';

// Re-export shared types
export {
  createPlansClient, PlansApi, type AssignPlanRequest, type BulkAssignRequest,
  type BulkRemoveRequest, type PlanFilters, type PlanStats, type RemovePlanRequest, type CreatePlanRequest as SharedCreatePlanRequest, type Plan as SharedPlan,
  type PlanMembership as SharedPlanMembership, type UpdatePlanRequest as SharedUpdatePlanRequest
} from '@/shared/api/plans';

// ============================================================================
// ADMIN-SPECIFIC TYPES
// ============================================================================

/**
 * Permission Plan (Admin extended version)
 */
export interface PermissionPlan {
  id: string;
  name: string;
  slug: string;
  description: string;
  plan_type: string;
  permissions: string[];
  price?: number;
  currency?: string;
  billing_cycle?: string;
  is_active: boolean;
  is_system_plan?: boolean;
  is_promoted?: boolean;
  display_order?: number;
  max_members?: number | null;
  auto_assign_enabled?: boolean;
  plan_metadata?: Record<string, unknown>;
  default_expiry_days?: number;
  priority_level?: number;
  created_at: string;
  updated_at: string;
  member_count?: number;
  is_public?: boolean;
}

export type Plan = PermissionPlan;

export interface UserPlanMembership {
  id: string;
  user_id: string;
  plan_id: string;
  granted_by: string;
  granted_at: string;
  expires_at: string | null;
  is_active: boolean;
  plan?: PermissionPlan;
}

export interface Web3AssignmentRule {
  id: string;
  plan_id: string;
  blockchain_network: 'bsc_mainnet' | 'bsc_testnet' | 'ethereum_mainnet' | 'polygon_mainnet' | 'arbitrum_mainnet' | 'optimism_mainnet';
  verification_type: 'nft_ownership' | 'token_balance' | 'dao_membership';
  contract_address?: string;
  token_id?: string;
  minimum_balance?: string;
  is_active: boolean;
  created_by: string;
  created_at: string;
  updated_at: string;
  plan?: PermissionPlan;
}

/**
 * Backend assignment DTO
 */
export interface AssignmentDto {
  id: string;
  wallet_address: string;
  plan_id: string;
  assigned_by?: string;
  assigned_at: string;
  expires_at: string | null;
  is_active: boolean;
  plan_name: string;
  plan_slug?: string;
  plan_description?: string;
  plan_type?: string;
  default_expiry_days?: number;
  priority_level?: number;
}

export interface PlanAssignmentHistory {
  id: string;
  user_id: string;
  user_email?: string;
  user_name?: string;
  plan_id: string;
  plan_name?: string;
  operation_type: 'assign' | 'remove' | 'expire' | 'cleanup';
  operation_source: 'manual' | 'web3_automatic' | 'system_cleanup' | 'bulk_operation';
  performed_by?: string;
  performed_by_name?: string;
  reason?: string;
  expires_at?: string;
  metadata?: Record<string, unknown>;
  created_at: string;
  plan?: PermissionPlan;
}

export interface PlanAnalytics {
  total_plans: number;
  total_active_memberships: number;
  expiring_soon_count: number;
  most_popular_plans: Array<{ plan_name: string; member_count: number }>;
  permission_distribution: Record<string, number>;
}

export interface AssignUserToPlanRequest {
  user_id: string;
  plan_id: string;
  expires_at?: string | null;
  reason?: string;
}

export interface CreatePlanRequest {
  name: string;
  permissions: string[];
  description?: string;
  default_expiry_days?: number;
  priority_level?: number;
  price?: number;
  is_public?: boolean;
  is_active?: boolean;
  plan_metadata?: Record<string, unknown>;
  display_order?: number;
}

export interface UpdatePlanRequest {
  name?: string;
  permissions?: string[];
  description?: string;
  default_expiry_days?: number;
  priority_level?: number;
  price?: number;
  is_public?: boolean;
  is_active?: boolean;
  plan_metadata?: Record<string, unknown>;
  display_order?: number;
}

export interface PermissionDefinitionDto {
  id: string;
  permission: string;
  name?: string | null;
  description?: string | null;
  platform: string;
  category?: string | null;
  is_system: boolean;
  is_active: boolean;
  created_at: string;
}

export interface CreatePermissionDefinitionRequest {
  permission: string;
  name?: string;
  description?: string;
  platform?: string;
  category?: string;
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function mapAssignmentToMembership(assignment: AssignmentDto): UserPlanMembership {
  return {
    id: assignment.id,
    user_id: assignment.wallet_address,
    plan_id: assignment.plan_id,
    granted_by: assignment.assigned_by ?? 'system',
    granted_at: assignment.assigned_at,
    expires_at: assignment.expires_at,
    is_active: assignment.is_active,
    plan: {
      id: assignment.plan_id,
      name: assignment.plan_name,
      slug: assignment.plan_slug ?? '',
      description: assignment.plan_description ?? assignment.plan_type ?? '',
      plan_type: assignment.plan_type ?? 'manual',
      permissions: [],
      is_active: true,
      created_at: assignment.assigned_at,
      updated_at: assignment.assigned_at,
      default_expiry_days: assignment.default_expiry_days,
      priority_level: assignment.priority_level,
    } as PermissionPlan
  };
}

// ============================================================================
// PLAN MANAGEMENT API
// ============================================================================

export const planMgmt = {
  // Plans
  getPlans: async () => {
    const res = await adminApiClient.get<Plan[]>(API_ROUTES.PERMISSIONS.PLANS);
    return extractArrayOrEmpty<Plan>(res);
  },
  getPlan: async (planId: string) => {
    const res = await adminApiClient.get<Plan>(`${API_ROUTES.PERMISSIONS.PLANS}/${planId}`);
    if (!res.success) { throw new Error(res.error?.message); }
    if (!res.data) { throw new Error('Plan data not found'); }
    return res.data;
  },
  createPlan: async (data: CreatePlanRequest) => {
    const res = await adminApiClient.post<PermissionPlan>(API_ROUTES.PERMISSIONS.PLANS, data);
    if (!res.success) { throw new Error(res.error?.message); }
    if (!res.data) { throw new Error('Failed to create plan: No data returned'); }
    return res.data;
  },
  updatePlan: async (planId: string, data: UpdatePlanRequest) => {
    const res = await adminApiClient.put<PermissionPlan>(`${API_ROUTES.PERMISSIONS.PLANS}/${planId}`, data);
    if (!res.success) { throw new Error(res.error?.message); }
    if (!res.data) { throw new Error('Failed to update plan: No data returned'); }
    return res.data;
  },
  deletePlan: async (planId: string) => {
    const res = await adminApiClient.delete(`${API_ROUTES.PERMISSIONS.PLANS}/${planId}`);
    if (!res.success) { throw new Error(res.error?.message); }
  },

  // Permissions
  getAvailablePermissions: async () => {
    const res = await adminApiClient.get<string[]>('/api/admin/permissions/available');
    return extractArrayOrEmpty<string>(res);
  },

  // Memberships
  getUserPlans: async (userId: string) => {
    const res = await adminApiClient.get<AssignmentDto[]>(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, { wallet_address: userId, is_active: true });
    return extractArrayOrEmpty<AssignmentDto>(res).map(mapAssignmentToMembership);
  },
  getPlanMemberships: async (planId: string) => {
    const res = await adminApiClient.get<AssignmentDto[]>(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, { plan_id: planId, is_active: true });
    return extractArrayOrEmpty<AssignmentDto>(res).map(mapAssignmentToMembership);
  },
  assignUserToPlan: async (data: AssignUserToPlanRequest) => {
    const res = await adminApiClient.post(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, {
      wallet_address: data.user_id, // Map user_id to wallet_address for backend
      plan_id: data.plan_id,
      expires_at: data.expires_at,
      assignment_source: 'manual'
    });
    if (!res.success) { throw new Error(res.error?.message); }
  },
  removeUserFromPlan: async (userId: string, planId: string) => {
    // Need assignment ID... this is complex to restore fully without fetch.
    // Simple fetch of assignments first
    const resList = await adminApiClient.get<AssignmentDto[]>(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, { wallet_address: userId, is_active: true });
    const list = extractArrayOrEmpty<AssignmentDto>(resList).map(mapAssignmentToMembership);
    const assignment = list.find(a => a.plan_id === planId);
    if (assignment) {
      await adminApiClient.delete(`${API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS}/${assignment.id}`);
    }
  },

  // Analytics and Monitoring
  getPlanAnalytics: async () => {
    const res = await adminApiClient.get<PlanAnalytics>(API_ROUTES.ADMIN.ANALYTICS_PERMISSIONS);
    if (!res.success) { throw new Error(res.error?.message); }
    if (!res.data) { throw new Error('Analytics data not found'); }
    return res.data;
  },
  getExpiringMemberships: async (days = 7) => {
    const res = await adminApiClient.get<unknown[]>(API_ROUTES.PERMISSIONS.EXPIRING, { days });
    // This endpoint should return a list of expiring memberships
    // Need to verify response structure to map correctly if needed, but for now returning array
    return extractArrayOrEmpty<unknown>(res);
  }
};
