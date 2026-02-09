'use client';

import {
    getAvailablePermissionsAction,
    getPlansAction,
    getUserPermissionsAction,
    getUserPlansAction,
} from '@/app/wallet-management/plan-actions';
import type { AccessItem, RawPlanData, WalletAccessData } from '@/types/wallet';
import { useCallback, useEffect, useState } from 'react';

export function useWalletAccessData(walletAddress: string | null) {
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [data, setData] = useState<WalletAccessData>({
        availablePermissions: [],
        availablePlans: [],
        authorizedPermissions: [],
        authorizedPlans: [],
    });

    const loadData = useCallback(async () => {
        try {
            setIsLoading(true);
            setError(null);

            // Fetch available items in parallel
            const [availablePermissionsRaw, plansRaw] = await Promise.all([
                getAvailablePermissionsAction().catch(() => []),
                getPlansAction().catch(() => []),
            ]);

            // Fetch wallet-specific data separately
            let walletPlans: Awaited<ReturnType<typeof getUserPlansAction>> = [];
            let walletPermissions: string[] = [];

            if (walletAddress !== null && walletAddress !== '') {
                try {
                    const result = await getUserPlansAction(walletAddress);
                    walletPlans = Array.isArray(result) ? result : [];
                } catch {
                    // Silently fail - 404 means no plans assigned
                }

                try {
                    const result = await getUserPermissionsAction(walletAddress);
                    walletPermissions = Array.isArray(result) ? result : [];
                } catch {
                    // Silently fail - 404 means no permissions assigned
                }
            }

            // Parse authorized permissions from wallet data
            const authorizedPermissionIds = new Set(walletPermissions);
            const authorizedPlanIds = new Set(walletPlans.map((wg) => wg.plan_id));

            // Convert permissions to AccessItems
            const permissionItems: AccessItem[] = availablePermissionsRaw.map((p: string) => {
                const parts = p.split(':');
                return {
                    id: p,
                    type: 'permission',
                    name: p,
                    platform: parts[0] ?? 'unknown',
                    category: parts[1] ?? 'general',
                };
            });

            // Convert plans to AccessItems
            const planItems: AccessItem[] = (plansRaw as unknown as RawPlanData[]).map((g) => ({
                id: g.id,
                type: 'plan',
                name: g.name,
                description: g.description,
                permissionCount: g.permissions?.length ?? 0,
                permissions: g.permissions ?? [],
                memberCount: g.member_count,
            }));

            // Split into available/authorized
            setData({
                availablePermissions: permissionItems.filter((p) => !authorizedPermissionIds.has(p.id)),
                availablePlans: planItems.filter((g) => !authorizedPlanIds.has(g.id)),
                authorizedPermissions: permissionItems
                    .filter((p) => authorizedPermissionIds.has(p.id))
                    .map((p) => ({ ...p, source: 'direct' })),
                authorizedPlans: planItems
                    .filter((g) => authorizedPlanIds.has(g.id))
                    .map((g) => {
                        const membership = walletPlans.find((wg) => wg.plan_id === g.id);
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
        void loadData();
    }, [loadData]);

    return {
        data,
        isLoading,
        error,
        refresh: loadData,
    };
}
