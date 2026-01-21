/**
 * Wallet Access Management Hook
 * Unified hook for managing wallet permissions and groups
 */

'use client';

import { useCallback, useEffect, useState } from 'react';

import { adminApiClient } from '@/lib/api-client';
import { groupMgmt } from '@/lib/api/group-management-client';

// ============================================================================
// TYPES
// ============================================================================

export type AccessItemType = 'permission' | 'group';

export interface AccessItem {
    id: string;
    type: AccessItemType;
    name: string;
    description?: string;
    icon?: string;
    // Permission-specific
    platform?: string;
    category?: string;
    // Group-specific
    permissionCount?: number;
    permissions?: string[]; // Added: List of permission IDs in the group
    memberCount?: number;
    // Assignment info
    expiresAt?: string | null;
    assignedAt?: string;
    source?: string;
}

export interface WalletAccessData {
    // Available items (can be assigned)
    availablePermissions: AccessItem[];
    availableGroups: AccessItem[];
    // Authorized items (currently assigned to wallet)
    authorizedPermissions: AccessItem[];
    authorizedGroups: AccessItem[];
}

export interface UseWalletAccessReturn {
    // Data
    data: WalletAccessData;
    isLoading: boolean;
    error: string | null;
    // Single Actions
    assignPermission: (permissionId: string, expiresAt?: string) => Promise<void>;
    revokePermission: (permissionId: string) => Promise<void>;
    assignGroup: (groupId: string, expiresAt?: string) => Promise<void>;
    removeGroup: (groupId: string) => Promise<void>;
    // Batch Actions
    batchAssignPermissions: (permissionIds: string[], expiresAt?: string) => Promise<void>;
    batchRevokePermissions: (permissionIds: string[]) => Promise<void>;
    batchAssignGroups: (groupIds: string[], expiresAt?: string) => Promise<void>;
    batchRemoveGroups: (groupIds: string[]) => Promise<void>;
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
        availableGroups: [],
        authorizedPermissions: [],
        authorizedGroups: [],
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
                groupsRaw,
            ] = await Promise.all([
                groupMgmt.getAvailablePermissions().catch(() => []),
                groupMgmt.getGroups().catch(() => []),
            ]);

            // Ensure arrays (in case API returns non-array)
            const availablePermissions: string[] = Array.isArray(availablePermissionsRaw) ? availablePermissionsRaw : [];
            const groups = Array.isArray(groupsRaw) ? groupsRaw : [];

            // Fetch wallet-specific data separately to handle 404 gracefully
            let walletGroups: Awaited<ReturnType<typeof groupMgmt.getUserGroups>> = [];
            let walletPermissions: string[] = [];

            try {
                const result = await groupMgmt.getUserGroups(walletAddress);
                walletGroups = Array.isArray(result) ? result : [];
            } catch {
                // 404 means no groups assigned - treat as empty (expected behavior)
            }

            try {
                const result = await groupMgmt.getUserPermissions(walletAddress);
                walletPermissions = Array.isArray(result) ? result : [];
            } catch {
                // 404 means no permissions assigned - treat as empty (expected behavior)
            }

            // Parse authorized permissions from wallet data
            const authorizedPermissionIds = new Set(walletPermissions);
            const authorizedGroupIds = new Set(walletGroups.map(wg => wg.group_id));

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

            // Convert groups to AccessItems
            const groupItems: AccessItem[] = groups.map(g => ({
                id: g.id,
                type: 'group' as const,
                name: g.name,
                description: g.description,
                permissionCount: g.permissions?.length || 0,
                permissions: g.permissions || [],
                memberCount: g.member_count,
            }));

            // Split into available/authorized
            setData({
                availablePermissions: permissionItems.filter(p => !authorizedPermissionIds.has(p.id)),
                availableGroups: groupItems.filter(g => !authorizedGroupIds.has(g.id)),
                authorizedPermissions: permissionItems
                    .filter(p => authorizedPermissionIds.has(p.id))
                    .map(p => ({ ...p, source: 'direct' })),
                authorizedGroups: groupItems
                    .filter(g => authorizedGroupIds.has(g.id))
                    .map(g => {
                        const membership = walletGroups.find(wg => wg.group_id === g.id);
                        return {
                            ...g,
                            expiresAt: membership?.expires_at,
                            assignedAt: membership?.granted_at,
                            source: 'group',
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

    // Assign group
    const assignGroup = useCallback(async (groupId: string, expiresAt?: string) => {
        if (!walletAddress) { return; }
        try {
            await groupMgmt.assignUserToGroup({
                user_id: walletAddress,
                group_id: groupId,
                expires_at: expiresAt || null,
            });
            await loadData();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to assign group');
        }
    }, [walletAddress, loadData]);

    // Remove group
    const removeGroup = useCallback(async (groupId: string) => {
        if (!walletAddress) { return; }
        try {
            await groupMgmt.removeUserFromGroup(walletAddress, groupId);
            await loadData();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to remove group');
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

    // Batch assign groups
    const batchAssignGroups = useCallback(async (groupIds: string[], expiresAt?: string) => {
        if (!walletAddress || groupIds.length === 0) { return; }
        try {
            await Promise.all(
                groupIds.map(groupId =>
                    groupMgmt.assignUserToGroup({
                        user_id: walletAddress,
                        group_id: groupId,
                        expires_at: expiresAt || null,
                    })
                )
            );
            await loadData();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to batch assign groups');
        }
    }, [walletAddress, loadData]);

    // Batch remove groups
    const batchRemoveGroups = useCallback(async (groupIds: string[]) => {
        if (!walletAddress || groupIds.length === 0) { return; }
        try {
            await Promise.all(
                groupIds.map(groupId => groupMgmt.removeUserFromGroup(walletAddress, groupId))
            );
            await loadData();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to batch remove groups');
        }
    }, [walletAddress, loadData]);

    return {
        data,
        isLoading,
        error,
        assignPermission,
        revokePermission,
        assignGroup,
        removeGroup,
        batchAssignPermissions,
        batchRevokePermissions,
        batchAssignGroups,
        batchRemoveGroups,
        refresh: loadData,
    };
}

export default useWalletAccess;
