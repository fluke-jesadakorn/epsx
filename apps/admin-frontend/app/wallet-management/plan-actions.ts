'use server';


import {
    CreatePlanRequest,
    PermissionPlan,
    UpdatePlanRequest,
    UserPlanMembership
} from '@/lib/api/plan-management-client';
import { createAdminApiClient, extractArrayOrEmpty , extractData } from '@/shared/api';
import { API_ROUTES } from '@/shared/config/route-constants';
import { revalidatePath } from 'next/cache';

// ============================================================================
// WALLET ACTIONS
// ============================================================================

import type { PermissionSource, Platform } from '@/components/wallet/types';
import {
    DisableWalletRequest,
    EnableWalletRequest,
    WalletData,
    WalletPermission,
    WalletSubscription,
    WalletSummaryDto
} from '@/lib/api/wallet-management-client';

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function mapAssignmentToMembership(assignment: any): UserPlanMembership {
    return {
        id: assignment.id,
        user_id: assignment.wallet_address,
        plan_id: assignment.plan_id,
        granted_by: assignment.assigned_by || 'system',
        granted_at: assignment.assigned_at,
        expires_at: assignment.expires_at,
        is_active: assignment.is_active,
        plan: {
            id: assignment.plan_id,
            name: assignment.plan_name,
            slug: assignment.plan_slug || '',
            description: assignment.plan_description || assignment.plan_type || '',
            plan_type: assignment.plan_type || 'manual',
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
    const res = await apiClient.get<any>(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, {
        wallet_address: userId,
        is_active: true,
        limit: 100,
    });
    return extractArrayOrEmpty<any>(res).map(mapAssignmentToMembership);
}

export async function getPlanMembershipsAction(planId: string): Promise<UserPlanMembership[]> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<any>(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, {
        plan_id: planId,
        is_active: true,
        limit: 100,
    });
    return extractArrayOrEmpty<any>(res).map(mapAssignmentToMembership);
}

export async function getUserPermissionsAction(userId: string): Promise<string[]> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<string[]>(`/api/auth/web3/plans/permissions/${userId}`);
    // If not found (404), return empty array
    if (!res.success && (res.error as any)?.status === 404) {return [];}
    return res.data || [];
}

export async function grantPermissionAction(walletAddress: string, permission: string, expiresAt?: string) {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post('/api/admin/permissions/direct/grant', {
        wallet_address: walletAddress,
        permission_string: permission,
        expires_at: expiresAt,
    });
    if (!res.success) {throw new Error(res.error?.message || 'Failed to grant permission');}
}

export async function revokePermissionAction(walletAddress: string, permission: string) {
    const apiClient = createAdminApiClient({ serverSide: true });
    // Use POST as per client implementation
    const res = await apiClient.post('/api/admin/permissions/direct/revoke', {
        wallet_address: walletAddress,
        permission_string: permission,
    });
    if (!res.success) {throw new Error(res.error?.message || 'Failed to revoke permission');}
}

export async function assignUserToPlanAction(userId: string, planId: string, expiresAt?: string | null) {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, {
        wallet_address: userId,
        plan_id: planId,
        expires_at: expiresAt,
        assignment_source: 'manual',
    });
    if (!res.success) {throw new Error(res.error?.message || 'Failed to assign plan');}
}

export async function removeUserFromPlanAction(userId: string, planId: string) {
    const apiClient = createAdminApiClient({ serverSide: true });

    // First get the assignment ID
    const assignments = await getUserPlansAction(userId);
    const assignment = assignments.find(a => a.plan_id === planId);

    if (assignment) {
        const res = await apiClient.delete(`${API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS}/${assignment.id}`);
        if (!res.success) {throw new Error(res.error?.message || 'Failed to remove plan');}
    }
}

export async function updatePlanAction(planId: string, data: UpdatePlanRequest) {
    const apiClient = createAdminApiClient({ serverSide: true });

    // Map frontend priority_level to backend display_order
    // Also allow direct display_order passing (for drag and drop)
    const payload = {
        ...data,
        display_order: (data as any).display_order ?? data.priority_level
    };

    const res = await apiClient.put(`${API_ROUTES.PERMISSIONS.PLANS}/${planId}`, payload);
    if (!res.success) {throw new Error(res.error?.message || 'Failed to update plan');}
    revalidatePath('/wallet-management/access/plans');
    return res.data;
}

export async function createPlanAction(data: CreatePlanRequest) {
    const apiClient = createAdminApiClient({ serverSide: true });

    const slug = data.name
        .toLowerCase()
        .replace(/[^a-z0-9\s-]/g, '')
        .replace(/\s+/g, '-')
        .replace(/-+/g, '-')
        .trim();

    const backendRequest = {
        name: data.name,
        slug: slug,
        description: data.description || '',
        plan_type: 'subscription',
        permissions: data.permissions,
        display_order: data.priority_level,
        price: data.price,
    };

    const res = await apiClient.post(API_ROUTES.PERMISSIONS.PLANS, backendRequest);
    if (!res.success) {throw new Error(res.error?.message || 'Failed to create plan');}

    revalidatePath('/wallet-management/access/plans');
    return res.data;
}

export async function getPlanAction(planId: string): Promise<PermissionPlan> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<PermissionPlan>(`${API_ROUTES.PERMISSIONS.PLANS}/${planId}`);
    if (!res.success) {throw new Error(res.error?.message || 'Failed to fetch plan');}
    return res.data!;
}

export async function deletePlanAction(planId: string) {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.delete(`${API_ROUTES.PERMISSIONS.PLANS}/${planId}`);
    if (!res.success) {throw new Error(res.error?.message || 'Failed to delete plan');}
    revalidatePath('/wallet-management/access/plans');
}

function detectPlatform(permission: string): Platform {
    if (permission.startsWith('epsx:analytics') || permission.startsWith('epsx:rankings')) {return 'analytics';}
    if (permission.startsWith('epsx-pay:')) {return 'pay';}
    if (permission.startsWith('epsx-token:')) {return 'token';}
    if (permission.startsWith('epsx-markets:')) {return 'markets';}
    return 'analytics';
}

function mapWalletDtoToData(dto: WalletSummaryDto): WalletData {
    const platforms = new Set<Platform>();
    const dtoPermissions = dto.permissions || [];
    dtoPermissions.forEach(p => {
        if (p.permission.startsWith('epsx:analytics') || p.permission.startsWith('epsx:rankings')) {
            platforms.add('analytics');
        } else if (p.permission.startsWith('epsx-pay:')) {
            platforms.add('pay');
        } else if (p.permission.startsWith('epsx-token:')) {
            platforms.add('token');
        } else if (p.permission.startsWith('epsx-markets:')) {
            platforms.add('markets');
        }
    });
    if (platforms.size === 0) {platforms.add('analytics');}

    const permissions: WalletPermission[] = dtoPermissions.map((p, idx) => ({
        id: `perm-${idx}`,
        permission: p.permission,
        platform: detectPlatform(p.permission),
        source: (p.source as PermissionSource) || 'system',
        expiresAt: p.expires_at,
        isActive: p.is_active,
        createdAt: dto.created_at,
    }));

    const subscriptions: WalletSubscription[] = (dto.subscriptions || []).map((s, idx) => ({
        id: `sub-${idx}`,
        planId: s.plan_id,
        planName: s.plan_name,
        status: s.status as 'active' | 'cancelled' | 'expired' | 'paused',
        priceDisplay: '',
        startedAt: s.started_at,
        expiresAt: s.expires_at,
        grantedPermissions: [],
    }));

    let status: 'active' | 'disabled' | 'pending' = 'active';
    if (!dto.is_active) {status = 'disabled';}

    const disableInfo = dto.metadata?.['disable_info'] as WalletData['disableInfo'] | undefined;
    const label = dto.metadata?.['label'] as string | undefined;
    const note = dto.metadata?.['note'] as string | undefined;

    return {
        walletAddress: dto.wallet_address,
        status,
        disableInfo,
        createdAt: dto.created_at,
        lastAuthAt: dto.last_auth_at,
        platforms: Array.from(platforms),
        permissions,
        subscriptions,
        plans: (dto.groups || []).map(g => ({ planName: g.group_name, role: g.role })),
        metadata: dto.metadata,
        label,
        note,
    };
}

export async function fetchWalletDetailAction(walletAddress: string): Promise<WalletData> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<any>(`/api/admin/wallets/${walletAddress}`);
    const responseData = extractData<{ wallet?: WalletSummaryDto } | WalletSummaryDto>(res);
    const walletDto = (responseData as any)?.wallet || responseData;
    if (!walletDto?.wallet_address) {throw new Error('Wallet not found');}
    return mapWalletDtoToData(walletDto);
}

export async function updateWalletMetadataAction(walletAddress: string, data: { label?: string | null; note?: string | null }): Promise<void> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.put(`/api/admin/wallets/${walletAddress}`, {
        metadata: { label: data.label ?? undefined, note: data.note ?? undefined },
    });
    if (!res.success) {throw new Error(res.error?.message || 'Failed to update metadata');}
}

export async function disableWalletAction(walletAddress: string, data: DisableWalletRequest): Promise<void> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post(`/api/admin/wallets/${walletAddress}/disable`, data);
    if (!res.success) {throw new Error(res.error?.message || 'Failed to disable wallet');}
}

export async function enableWalletAction(walletAddress: string, data: EnableWalletRequest): Promise<void> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post(`/api/admin/wallets/${walletAddress}/enable`, data);
    if (!res.success) {throw new Error(res.error?.message || 'Failed to enable wallet');}
}
