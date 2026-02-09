'use client';

import { useRouter } from 'next/navigation';
import { useCallback, useMemo, useState } from 'react';
import { toast } from 'sonner';

import type { AccessItem, UseWalletAccessReturn } from '@/hooks/use-wallet-access';
import type { useWalletData } from '@/hooks/use-wallet-detail';
import { copyToClipboard } from '@/lib/utils';
import { logger } from '@/shared/utils/logger';
import type { DragEndEvent, DragStartEvent } from '@dnd-kit/core';

interface UseWalletDetailViewLogicProps {
    walletAddress: string;
    walletData: ReturnType<typeof useWalletData>;
    accessData: UseWalletAccessReturn;
}

export function useWalletDetailViewLogic({
    walletAddress,
    walletData,
    accessData
}: UseWalletDetailViewLogicProps) {
    const router = useRouter();

    const [activeDragItem, setActiveDragItem] = useState<AccessItem | null>(null);
    const [pendingDrops, setPendingDrops] = useState<AccessItem[]>([]);
    const [editingItem, setEditingItem] = useState<{ item: AccessItem; type: 'plan' | 'permission' } | null>(null);
    const [isSavingPending, setIsSavingPending] = useState(false);

    // Filter State
    const [searchQuery, setSearchQuery] = useState('');
    const [assignedSearchQuery, setAssignedSearchQuery] = useState('');

    const allPlans = useMemo(() =>
        [...accessData.data.authorizedPlans, ...accessData.data.availablePlans],
        [accessData.data.authorizedPlans, accessData.data.availablePlans]
    );

    const filteredAvailablePlans = useMemo(() => {
        const assignedIds = new Set(accessData.data.authorizedPlans.map(g => g.id));
        const pendingIds = new Set(pendingDrops.map(g => g.id));

        return accessData.data.availablePlans.filter(g =>
            !assignedIds.has(g.id) &&
            !pendingIds.has(g.id) &&
            (g.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                (g.description?.toLowerCase().includes(searchQuery.toLowerCase()) ?? false))
        );
    }, [accessData.data.availablePlans, accessData.data.authorizedPlans, pendingDrops, searchQuery]);

    const handleManagePlan = useCallback((planId: string) => {
        router.push(`/wallet-management/plans/${planId}?from=/wallet-management/${encodeURIComponent(walletAddress)}`);
    }, [router, walletAddress]);

    const handleDragStart = useCallback((event: DragStartEvent) => {
        const { active } = event;
        const allPermissions = [...accessData.data.availablePermissions, ...accessData.data.authorizedPermissions];

        const item = active.data.current?.type === 'plan'
            ? allPlans.find(p => p.id === active.id)
            : allPermissions.find(p => p.id === active.id);

        if (item) {
            setActiveDragItem(item);
        }
    }, [allPlans, accessData.data.availablePermissions, accessData.data.authorizedPermissions]);

    const handleDragEnd = useCallback((event: DragEndEvent) => {
        const { active, over } = event;
        setActiveDragItem(null);

        if (!over) { return; }

        if (active.data.current?.type === 'plan' && over.id === 'assigned-plan-list') {
            const plan = accessData.data.availablePlans.find(g => g.id === active.id);
            if (plan && !pendingDrops.find(p => p.id === plan.id)) {
                setPendingDrops(prev => [...prev, plan]);
                toast.success(`Staged "${plan.name}" for assignment`);
            }
        }
    }, [accessData.data.availablePlans, pendingDrops]);

    const handleSavePendingChanges = useCallback(async () => {
        if (pendingDrops.length === 0) { return; }
        setIsSavingPending(true);
        try {
            await Promise.all(pendingDrops.map(plan => accessData.assignPlan(plan.id)));
            toast.success('Access plans assigned successfully');
            setPendingDrops([]);
            void accessData.refresh();
        } catch (_err) {
            logger.error('Failed to save changes:', _err);
            toast.error('Failed to assign plans');
        } finally {
            setIsSavingPending(false);
        }
    }, [pendingDrops, accessData]);

    const handleCopyAddress = useCallback(async () => {
        if (!walletData.wallet) { return; }
        const success = await copyToClipboard(walletData.wallet.walletAddress);
        if (success) {
            toast.success('Address copied!');
        }
    }, [walletData.wallet]);

    return {
        activeDragItem,
        setActiveDragItem,
        pendingDrops,
        setPendingDrops,
        editingItem,
        setEditingItem,
        isSavingPending,
        searchQuery,
        setSearchQuery,
        assignedSearchQuery,
        setAssignedSearchQuery,
        filteredAvailablePlans,
        handleManagePlan,
        handleDragStart,
        handleDragEnd,
        handleSavePendingChanges,
        handleCopyAddress,
        hasPending: pendingDrops.length > 0
    };
}
