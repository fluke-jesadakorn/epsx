'use client';

import {
    getWalletAccessSummaryAction,
    type WalletAccessSummary,
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

            if (walletAddress === null || walletAddress === '') {
                setData({
                    availablePermissions: [],
                    availablePlans: [],
                    authorizedPermissions: [],
                    authorizedPlans: [],
                });
                return;
            }

            const summary: WalletAccessSummary = await getWalletAccessSummaryAction(walletAddress);

            const authorizedPermissionIds = new Set(summary.wallet_permissions);
            const authorizedPlanIds = new Set(summary.wallet_assignments.map((a) => a.plan_id));

            // Convert permissions to AccessItems
            const permissionItems: AccessItem[] = summary.available_permissions.map((p: string) => {
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
            const planItems: AccessItem[] = (summary.available_plans as unknown as RawPlanData[]).map((g) => ({
                id: g.id,
                type: 'plan',
                name: g.name,
                description: g.description,
                permissionCount: g.permissions?.length ?? 0,
                permissions: g.permissions ?? [],
                memberCount: g.member_count,
                planGroup: g.plan_group,
            }));

            setData({
                availablePermissions: permissionItems.filter((p) => !authorizedPermissionIds.has(p.id)),
                availablePlans: planItems.filter((g) => !authorizedPlanIds.has(g.id)),
                authorizedPermissions: permissionItems
                    .filter((p) => authorizedPermissionIds.has(p.id))
                    .map((p) => ({ ...p, source: 'direct' })),
                authorizedPlans: planItems
                    .filter((g) => authorizedPlanIds.has(g.id))
                    .map((g) => {
                        const assignment = summary.wallet_assignments.find((a) => a.plan_id === g.id);
                        return {
                            ...g,
                            expiresAt: assignment?.expires_at ?? undefined,
                            assignedAt: assignment?.granted_at ?? undefined,
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
