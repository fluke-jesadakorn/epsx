/**
 * Wallet Access Management Hook
 * Unified hook for managing wallet permissions and plans
 */

'use client';

import { useCallback, useEffect, useState } from 'react';

import { adminApiClient } from '@/lib/api-client';
import { planMgmt } from '@/lib/api/plan-management-client';

// ============================================================================
// TYPES
// ============================================================================

export type AccessItemType = 'permission' | 'plan';

export interface AccessItem {
    id: string;
    type: AccessItemType;
    name: string;
    description?: string;
    icon?: string;
    // Permission-specific
    platform?: string;
    category?: string;
    // Plan-specific
    permissionCount?: number;
    permissions?: string[]; // Added: List of permission IDs in the plan
    memberCount?: number;
    // Assignment info
    expiresAt?: string | null;
    assignedAt?: string;
    source?: string;
}

export interface WalletAccessData {
    // Available items (can be assigned)
    availablePermissions: AccessItem[];
    availablePlans: AccessItem[];
    // Authorized items (currently assigned to wallet)
    authorizedPermissions: AccessItem[];
    authorizedPlans: AccessItem[];
}

export interface UseWalletAccessReturn {
    // Data
    data: WalletAccessData;
    isLoading: boolean;
    error: string | null;
    // Single Actions
    assignPermission: (permissionId: string, expiresAt?: string) => Promise<void>;
    revokePermission: (permissionId: string) => Promise<void>;
    assignPlan: (planId: string, expiresAt?: string) => Promise<void>;
    removePlan: (planId: string) => Promise<void>;
    // Batch Actions
    batchAssignPermissions: (permissionIds: string[], expiresAt?: string) => Promise<void>;
    batchRevokePermissions: (permissionIds: string[]) => Promise<void>;
    batchAssignPlans: (planIds: string[], expiresAt?: string) => Promise<void>;
    batchRemovePlans: (planIds: string[]) => Promise<void>;
    // Refresh
    refresh: () => Promise<void>;
}

// ============================================================================
// HOOK
// ============================================================================

/**
 *
 * @param walletAddress
 */
export function useWalletAccess(walletAddress: string | null): UseWalletAccessReturn {
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [data, setData] = useState<WalletAccessData>({
        availablePermissions: [],
        availablePlans: [],
        authorizedPermissions: [],
        authorizedPlans: [],
    });

    // Load all data
    const loadData = useCallback(async () => {
        if (!walletAddress) { return; }

        try {
            setIsLoading(true);
            setError(null);

            // Fetch available items in parallel - ensure arrays with fallbacks
            const [
                availablePermissionsRaw,
                plansRaw,
            ] = await Promise.all([
                planMgmt.getAvailablePermissions().catch(() => []),
                planMgmt.getPlans().catch(() => []),
            ]);

            // Ensure arrays (in case API returns non-array)
            const availablePermissions: string[] = Array.isArray(availablePermissionsRaw) ? availablePermissionsRaw : [];
            const plans = Array.isArray(plansRaw) ? plansRaw : [];

            // Fetch wallet-specific data separately to handle 404 gracefully
            let walletPlans: Awaited<ReturnType<typeof planMgmt.getUserPlans>> = [];
            let walletPermissions: string[] = [];

            try {
                const result = await planMgmt.getUserPlans(walletAddress);
                walletPlans = Array.isArray(result) ? result : [];
            } catch {
                // 404 means no plans assigned - treat as empty (expected behavior)
            }

            try {
                const result = await planMgmt.getUserPermissions(walletAddress);
                walletPermissions = Array.isArray(result) ? result : [];
            } catch {
                // 404 means no permissions assigned - treat as empty (expected behavior)
            }

            // Parse authorized permissions from wallet data
            const authorizedPermissionIds = new Set(walletPermissions);
            const authorizedPlanIds = new Set(walletPlans.map(wg => wg.plan_id));

            // Convert permissions to AccessItems
            const permissionItems: AccessItem[] = availablePermissions.map(p => {
                const parts = p.split(':');
                return {
                    id: p,
                    type: 'permission' as const,
                    name: p,
                    platform: parts[0],
                    category: parts[1],
                };
            });

            // Convert plans to AccessItems
            const planItems: AccessItem[] = plans.map(g => ({
                id: g.id,
                type: 'plan' as const,
                name: g.name,
                description: g.description,
                permissionCount: g.permissions?.length || 0,
                permissions: g.permissions || [],
                memberCount: g.member_count,
            }));

            // Split into available/authorized
            setData({
                availablePermissions: permissionItems.filter(p => !authorizedPermissionIds.has(p.id)),
                availablePlans: planItems.filter(g => !authorizedPlanIds.has(g.id)),
                authorizedPermissions: permissionItems
                    .filter(p => authorizedPermissionIds.has(p.id))
                    .map(p => ({ ...p, source: 'direct' })),
                authorizedPlans: planItems
                    .filter(g => authorizedPlanIds.has(g.id))
                    .map(g => {
                        const membership = walletPlans.find(wg => wg.plan_id === g.id);
                        return {
                            ...g,
                            expiresAt: membership?.expires_at,
                            assignedAt: membership?.granted_at,
                            source: 'plan',
                        };
                    }),
            });

        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to load wallet access data');
        } finally {
            setIsLoading(false);
        }
    }, [walletAddress]);

    // Load on mount and wallet change
    useEffect(() => {
        loadData();
    }, [loadData]);

    // Assign permission
    const assignPermission = useCallback(async (permissionId: string, expiresAt?: string) => {
        if (!walletAddress) { return; }
        try {
            await adminApiClient.post('/api/admin/permissions/direct/grant', {
                wallet_address: walletAddress,
                permission_string: permissionId,
                expires_at: expiresAt,
            });
            await loadData();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to assign permission');
        }
    }, [walletAddress, loadData]);

    // Revoke permission
    const revokePermission = useCallback(async (permissionId: string) => {
        if (!walletAddress) { return; }
        try {
            // Use POST with query params since DELETE doesn't support body in this client
            await adminApiClient.post('/api/admin/permissions/direct/revoke', {
                wallet_address: walletAddress,
                permission_string: permissionId,
            });
            await loadData();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to revoke permission');
        }
    }, [walletAddress, loadData]);

    // Assign plan
    const assignPlan = useCallback(async (planId: string, expiresAt?: string) => {
        if (!walletAddress) { return; }
        try {
            await planMgmt.assignUserToPlan({
                user_id: walletAddress,
                plan_id: planId,
                expires_at: expiresAt || null,
            });
            await loadData();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to assign plan');
        }
    }, [walletAddress, loadData]);

    // Remove plan
    const removePlan = useCallback(async (planId: string) => {
        if (!walletAddress) { return; }
        try {
            await planMgmt.removeUserFromPlan(walletAddress, planId);
            await loadData();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to remove plan');
        }
    }, [walletAddress, loadData]);

    // Batch assign permissions
    const batchAssignPermissions = useCallback(async (permissionIds: string[], expiresAt?: string) => {
        if (!walletAddress || permissionIds.length === 0) { return; }
        try {
            await Promise.all(
                permissionIds.map(permissionId =>
                    adminApiClient.post('/api/admin/permissions/direct/grant', {
                        wallet_address: walletAddress,
                        permission_string: permissionId,
                        expires_at: expiresAt,
                    })
                )
            );
            await loadData();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to batch assign permissions');
        }
    }, [walletAddress, loadData]);

    // Batch revoke permissions
    const batchRevokePermissions = useCallback(async (permissionIds: string[]) => {
        if (!walletAddress || permissionIds.length === 0) { return; }
        try {
            await Promise.all(
                permissionIds.map(permissionId =>
                    adminApiClient.post('/api/admin/permissions/direct/revoke', {
                        wallet_address: walletAddress,
                        permission_string: permissionId,
                    })
                )
            );
            await loadData();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to batch revoke permissions');
        }
    }, [walletAddress, loadData]);

    // Batch assign plans
    const batchAssignPlans = useCallback(async (planIds: string[], expiresAt?: string) => {
        if (!walletAddress || planIds.length === 0) { return; }
        try {
            await Promise.all(
                planIds.map(planId =>
                    planMgmt.assignUserToPlan({
                        user_id: walletAddress,
                        plan_id: planId,
                        expires_at: expiresAt || null,
                    })
                )
            );
            await loadData();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to batch assign plans');
        }
    }, [walletAddress, loadData]);

    // Batch remove plans
    const batchRemovePlans = useCallback(async (planIds: string[]) => {
        if (!walletAddress || planIds.length === 0) { return; }
        try {
            await Promise.all(
                planIds.map(planId => planMgmt.removeUserFromPlan(walletAddress, planId))
            );
            await loadData();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to batch remove plans');
        }
    }, [walletAddress, loadData]);

    return {
        data,
        isLoading,
        error,
        assignPermission,
        revokePermission,
        assignPlan,
        removePlan,
        batchAssignPermissions,
        batchRevokePermissions,
        batchAssignPlans,
        batchRemovePlans,
        refresh: loadData,
    };
}

export default useWalletAccess;
