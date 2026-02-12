'use server';

import type {
    AssignmentDto,
    CreatePlanRequest,
    PermissionPlan,
    UpdatePlanRequest,
    UserPlanMembership
} from '@/lib/api/plan-management-client';
import { createAdminApiClient, extractArrayOrEmpty } from '@/shared/api';
import { API_ROUTES } from '@/shared/config/route-constants';
import { revalidatePath } from 'next/cache';

// ============================================================================
// WALLET ACTIONS
// ============================================================================

import type {
    DisableWalletRequest,
    EnableWalletRequest,
    WalletData,
    WalletSummaryDto
} from '@/lib/api/wallet-management-client';
import { mapWalletDtoToData } from '@/lib/mappers/wallet';

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
            tier_level: assignment.priority_level ?? 0,
        } as PermissionPlan
    };
}

// ============================================================================
// SERVER ACTIONS
// ============================================================================

export async function getAvailablePermissionsAction(): Promise<string[]> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<string[]>('/api/admin/permissions/available');
    return extractArrayOrEmpty<string>(res);
}

export async function getPlansAction(): Promise<PermissionPlan[]> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<PermissionPlan[]>(API_ROUTES.PERMISSIONS.PLANS);
    return extractArrayOrEmpty<PermissionPlan>(res);
}

export async function getUserPlansAction(userId: string): Promise<UserPlanMembership[]> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<AssignmentDto[]>(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, {
        wallet_address: userId,
        is_active: true,
        limit: 100,
    });
    return extractArrayOrEmpty<AssignmentDto>(res).map(mapAssignmentToMembership);
}

export async function getPlanMembershipsAction(planId: string): Promise<UserPlanMembership[]> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<AssignmentDto[]>(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, {
        plan_id: planId,
        is_active: true,
        limit: 100,
    });
    return extractArrayOrEmpty<AssignmentDto>(res).map(mapAssignmentToMembership);
}

export async function getUserPermissionsAction(userId: string): Promise<string[]> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<string[]>(`/api/auth/web3/plans/permissions/${userId}`);
    // If not found (404), return empty array
    if (!res.success && res.error?.code === '404') { return []; }
    return res.data ?? [];
}

export async function grantPermissionAction(walletAddress: string, permission: string, expiresAt?: string) {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post('/api/admin/permissions/direct/grant', {
        wallet_address: walletAddress,
        permission_string: permission,
        expires_at: expiresAt,
    });
    if (!res.success) { throw new Error(res.error?.message ?? 'Failed to grant permission'); }
}

export async function revokePermissionAction(walletAddress: string, permission: string) {
    const apiClient = createAdminApiClient({ serverSide: true });
    // Use POST as per client implementation
    const res = await apiClient.post('/api/admin/permissions/direct/revoke', {
        wallet_address: walletAddress,
        permission_string: permission,
    });
    if (!res.success) { throw new Error(res.error?.message ?? 'Failed to revoke permission'); }
}

export async function assignUserToPlanAction(userId: string, planId: string, expiresAt?: string | null) {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, {
        wallet_address: userId,
        plan_id: planId,
        expires_at: expiresAt,
        assignment_source: 'manual',
    });
    if (!res.success) { throw new Error(res.error?.message ?? 'Failed to assign plan'); }
}

export async function removeUserFromPlanAction(userId: string, planId: string) {
    const apiClient = createAdminApiClient({ serverSide: true });

    // First get the assignment ID
    const assignments = await getUserPlansAction(userId);
    const assignment = assignments.find(a => a.plan_id === planId);

    if (assignment) {
        const res = await apiClient.delete(`${API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS}/${assignment.id}`);
        if (!res.success) { throw new Error(res.error?.message ?? 'Failed to remove plan'); }
    }
}

export async function updatePlanAction(planId: string, data: UpdatePlanRequest): Promise<PermissionPlan> {
    const apiClient = createAdminApiClient({ serverSide: true });

    // Pass tier_level directly to backend
    const payload = {
        ...data,
    };

    const res = await apiClient.put<PermissionPlan>(`${API_ROUTES.PERMISSIONS.PLANS}/${planId}`, payload);
    if (!res.success || !res.data) {
        throw new Error(res.error?.message ?? 'Failed to update plan');
    }
    revalidatePath('/wallet-management/access/plans');
    return res.data;
}

export async function createPlanAction(data: CreatePlanRequest): Promise<PermissionPlan> {
    const apiClient = createAdminApiClient({ serverSide: true });

    const slug = data.name
        .toLowerCase()
        .replace(/[^a-z0-9\s-]/g, '')
        .replace(/\s+/g, '-')
        .replace(/-+/g, '-')
        .trim();

    const backendRequest = {
        name: data.name,
        slug,
        description: data.description ?? '',
        plan_type: 'subscription',
        permissions: data.permissions,
        tier_level: data.tier_level,
        price: data.price,
    };

    const res = await apiClient.post<PermissionPlan>(API_ROUTES.PERMISSIONS.PLANS, backendRequest);
    if (!res.success || !res.data) {
        throw new Error(res.error?.message ?? 'Failed to create plan');
    }

    revalidatePath('/wallet-management/access/plans');
    return res.data;
}

export async function getPlanAction(planId: string): Promise<PermissionPlan> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<PermissionPlan>(`${API_ROUTES.PERMISSIONS.PLANS}/${planId}`);
    if (!res.success || !res.data) { throw new Error(res.error?.message ?? 'Failed to fetch plan'); }
    return res.data;
}

export async function deletePlanAction(planId: string) {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.delete(`${API_ROUTES.PERMISSIONS.PLANS}/${planId}`);
    if (!res.success) { throw new Error(res.error?.message ?? 'Failed to delete plan'); }
    revalidatePath('/wallet-management/access/plans');
}

export async function fetchWalletDetailAction(walletAddress: string): Promise<WalletData> {
    const apiClient = createAdminApiClient({ serverSide: true });

    // We expect either a wrapped { wallet: ... } or a flat WalletSummaryDto
    const res = await apiClient.get<Record<string, unknown>>(`/api/admin/wallets/${walletAddress}`);

    if (!res.success || !res.data) {
        throw new Error(res.error?.message ?? 'Wallet not found');
    }

    const data = res.data;

    // Check if it's wrapped in `wallet` property
    if (typeof data.wallet === 'object' && data.wallet !== null) {
        return mapWalletDtoToData(data.wallet as WalletSummaryDto);
    }

    // Check if it looks like a flat wallet object (has wallet_address)
    if (typeof data.wallet_address === 'string') {
        return mapWalletDtoToData(data as unknown as WalletSummaryDto);
    }

    throw new Error('Wallet not found');
}

export async function updateWalletMetadataAction(walletAddress: string, data: { label?: string | null; note?: string | null }): Promise<void> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.put(`/api/admin/wallets/${walletAddress}`, {
        metadata: { label: data.label ?? undefined, note: data.note ?? undefined },
    });
    if (!res.success) { throw new Error(res.error?.message ?? 'Failed to update metadata'); }
}

export async function disableWalletAction(walletAddress: string, data: DisableWalletRequest): Promise<void> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post(`/api/admin/wallets/${walletAddress}/disable`, data);
    if (!res.success) { throw new Error(res.error?.message ?? 'Failed to disable wallet'); }
}

export async function enableWalletAction(walletAddress: string, data: EnableWalletRequest): Promise<void> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post(`/api/admin/wallets/${walletAddress}/enable`, data);
    if (!res.success) { throw new Error(res.error?.message ?? 'Failed to enable wallet'); }
}
