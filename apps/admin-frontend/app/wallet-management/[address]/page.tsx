/**
 * Wallet Detail Page
 * Unified view for wallet info, plans, and permissions with DND support
 */
'use client';

import { TrashDropZone } from '@/components/wallet/trash-drop-zone';
import type {
    DragEndEvent,
    DragStartEvent
} from '@dnd-kit/core';
import {
    DndContext,
    DragOverlay,
    MouseSensor,
    TouchSensor,
    useSensor,
    useSensors
} from '@dnd-kit/core';
import { ArrowLeft, Key, Package, RefreshCw, ShieldCheck } from 'lucide-react';
import Link from 'next/link';
import { useParams, useRouter } from 'next/navigation';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { toast } from 'sonner';

import { logger } from '@/shared/utils/logger';

import {
    disableWalletAction,
    enableWalletAction,
    fetchWalletDetailAction,
    updateWalletMetadataAction
} from '@/app/wallet-management/plan-actions';
import { DisableWalletModal, type DisableWalletData } from '@/components/wallet/disable-wallet-modal';
import { ExpiryDatePicker } from '@/components/wallet/expiry-date-picker';
import { ReenableWalletModal, type ReenableWalletData } from '@/components/wallet/reenable-wallet-modal';
import type { WalletData, WalletStatus } from '@/components/wallet/types';
import {
    WalletAssignedPlansCard,
    WalletAvailablePlansCard,
    WalletMetadataCard,
    WalletSubscriptionCard
} from '@/components/wallet/wallet-detail-sections';
import type { AccessItem } from '@/hooks/use-wallet-access';
import { useWalletAccess } from '@/hooks/use-wallet-access';
import { cn, copyToClipboard } from '@/lib/utils';
import { createPlansClient, type SubscriptionResponse } from '@/shared/api/plans';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { Button } from '@/shared/components/ui/button';
import { Skeleton } from '@/shared/components/ui/skeleton';
import { createAdminApiClient } from '@/shared/utils/api-client';

const STATUS_CONFIG: Record<WalletStatus, { label: string; emoji: string; className: string }> = {
    active: {
        label: 'Active',
        emoji: '🟢',
        className: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400 border-green-200 dark:border-green-800',
    },
    disabled: {
        label: 'Disabled',
        emoji: '⚠️',
        className: 'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400 border-amber-200 dark:border-amber-800',
    },
    pending: {
        label: 'Pending',
        emoji: '⏳',
        className: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400 border-blue-200 dark:border-blue-800',
    },
};

// ============================================================================
// CUSTOM HOOKS
// ============================================================================

interface UseWalletDataContext {
    walletAddress: string;
    router: ReturnType<typeof useRouter>;
}

function useWalletData(ctx: UseWalletDataContext) {
    const [wallet, setWallet] = useState<WalletData | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [isRefreshing, setIsRefreshing] = useState(false);

    const loadWallet = useCallback(async () => {
        if (ctx.walletAddress === '') { return; }

        try {
            setIsRefreshing(true);
            const walletData = await fetchWalletDetailAction(ctx.walletAddress);
            setWallet(walletData);
        } catch (_err) {
            logger.error('Failed to load wallet:', _err);
            toast.error('Failed to load wallet details');
            ctx.router.push('/wallet-management');
        } finally {
            setIsLoading(false);
            setIsRefreshing(false);
        }
    }, [ctx.walletAddress, ctx.router]);

    return { wallet, setWallet, isLoading, isRefreshing, loadWallet };
}

function useMetadataForm(wallet: WalletData | null) {
    const [metadataForm, setMetadataForm] = useState({ label: '', note: '' });
    const [isSaving, setIsSaving] = useState(false);
    const [hasChanges, setHasChanges] = useState(false);

    useEffect(() => {
        if (wallet !== null) {
            setMetadataForm({
                label: wallet.label ?? '',
                note: wallet.note ?? '',
            });
            setHasChanges(false);
        }
    }, [wallet]);

    const handleSave = useCallback(async (walletAddress: string, loadWallet: () => Promise<void>) => {
        if (wallet === null) { return; }
        setIsSaving(true);
        try {
            await updateWalletMetadataAction(walletAddress, {
                label: metadataForm.label ?? null,
                note: metadataForm.note ?? null,
            });
            toast.success('Wallet metadata updated');
            setHasChanges(false);
            await loadWallet();
        } catch (_err) {
            logger.error('Failed to update metadata:', _err);
            toast.error('Failed to save changes');
        } finally {
            setIsSaving(false);
        }
    }, [metadataForm, wallet]);

    return { metadataForm, setMetadataForm, isSaving, hasChanges, setHasChanges, handleSave };
}

function usePlanAssignments(accessData: ReturnType<typeof useWalletAccess>['data']) {
    const [pendingDrops, setPendingDrops] = useState<AccessItem[]>([]);
    const [isSaving, setIsSaving] = useState(false);
    const { assignPlan, refresh: refreshAccess } = useWalletAccess('');

    const handleSave = useCallback(async () => {
        if (pendingDrops.length === 0) { return; }
        setIsSaving(true);
        try {
            await Promise.all(pendingDrops.map(plan => assignPlan(plan.id)));
            toast.success('Access plans assigned successfully');
            setPendingDrops([]);
            refreshAccess();
        } catch (_err) {
            logger.error('Failed to save changes:', _err);
            toast.error('Failed to assign plans');
        } finally {
            setIsSaving(false);
        }
    }, [pendingDrops, assignPlan, refreshAccess]);

    return { pendingDrops, setPendingDrops, isSaving, handleSave };
}

interface UseWalletActionsContext {
    walletAddress: string;
    onActionComplete: () => Promise<void>;
}

function useWalletActions(ctx: UseWalletActionsContext) {
    const [isLoading, setIsLoading] = useState(false);

    const handleDisable = useCallback(async (data: DisableWalletData) => {
        setIsLoading(true);
        try {
            await disableWalletAction(ctx.walletAddress, {
                duration_days: data.duration === 'until_manual' ? null : data.duration,
                reason_category: data.reasonCategory,
                reason_details: data.reasonDetails,
                affected_platforms: data.affectedPlatforms,
                block_login: data.blockLogin,
                pause_subscriptions: data.pauseSubscriptions,
                notify_user: data.notifyUser,
            });
            toast.success('Wallet disabled successfully');
            await ctx.onActionComplete();
        } catch (_err) {
            logger.error('Failed to disable wallet:', _err);
            toast.error(_err instanceof Error ? _err.message : 'Failed to disable wallet');
        } finally {
            setIsLoading(false);
        }
    }, [ctx.walletAddress, ctx.onActionComplete]);

    const handleReenable = useCallback(async (data: ReenableWalletData) => {
        setIsLoading(true);
        try {
            await enableWalletAction(ctx.walletAddress, {
                platforms_to_enable: data.platformsToEnable,
                restore_permissions: data.restorePermissions,
                resume_subscriptions: data.resumeSubscriptions,
                resolution_note: data.resolutionNote,
            });
            toast.success('Wallet re-enabled successfully');
            await ctx.onActionComplete();
        } catch (_err) {
            logger.error('Failed to re-enable wallet:', _err);
            toast.error(_err instanceof Error ? _err.message : 'Failed to re-enable wallet');
        } finally {
            setIsLoading(false);
        }
    }, [ctx.walletAddress, ctx.onActionComplete]);

    return { isLoading, handleDisable, handleReenable };
}

function useSubscriptionData(walletAddress: string) {
    const [activeSub, setActiveSub] = useState<SubscriptionResponse | null>(null);
    const [isLoading, setIsLoading] = useState(false);

    useEffect(() => {
        if (walletAddress === '') { return; }

        const loadSubscription = async () => {
            setIsLoading(true);
            try {
                const client = createPlansClient(createAdminApiClient());
                const res = await client.getSubscriptions({ limit: 100 });
                if (res?.success === true && res.data?.subscriptions !== undefined) {
                    const sub = res.data.subscriptions.find((s: SubscriptionResponse) =>
                        s.user_id === walletAddress && s.status === 'active'
                    );
                    if (sub !== undefined) { setActiveSub(sub); }
                }
            } catch (_e) {
                logger.error('Failed to load subscription details', _e);
            } finally {
                setIsLoading(false);
            }
        };

        loadSubscription();
    }, [walletAddress]);

    return { activeSub, isLoading };
}

export default function WalletDetailPage() {
    const router = useRouter();
    const params = useParams();
    const walletAddress = decodeURIComponent(params['address'] as string);

    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();

    // DND Sensors
    const sensors = useSensors(
        useSensor(MouseSensor, { activationConstraint: { distance: 10 } }),
        useSensor(TouchSensor, { activationConstraint: { delay: 250, tolerance: 5 } })
    );

    // Extract custom hooks
    const walletData = useWalletData({ walletAddress, router });
    const metadataForm = useMetadataForm(walletData.wallet);
    const subscriptionData = useSubscriptionData(walletAddress);

    // Access Management Hook
    const {
        data: accessData,
        isLoading: _isAccessLoading,
        assignPlan,
        removePlan,
        refresh: refreshAccess
    } = useWalletAccess(walletAddress);

    // DND & UI State
    const [activeDragItem, setActiveDragItem] = useState<AccessItem | null>(null);
    const [pendingDrops, setPendingDrops] = useState<AccessItem[]>([]);
    const [editingItem, setEditingItem] = useState<{ item: AccessItem; type: 'plan' | 'permission' } | null>(null);
    const [copied, setCopied] = useState(false);
    const [isSavingPending, setIsSavingPending] = useState(false);

    // Modal States
    const [showDisableModal, setShowDisableModal] = useState(false);
    const [showReenableModal, setShowReenableModal] = useState(false);

    // Filter State
    const [searchQuery, setSearchQuery] = useState('');
    const [assignedSearchQuery, setAssignedSearchQuery] = useState('');

    // Wallet actions
    const walletActions = useWalletActions({
        walletAddress,
        onActionComplete: async () => {
            setShowDisableModal(false);
            setShowReenableModal(false);
            await walletData.loadWallet();
        }
    });

    // Load wallet on auth
    useEffect(() => {
        if (isAuthenticated && !authLoading) {
            walletData.loadWallet();
        }
    }, [isAuthenticated, authLoading, walletData.loadWallet]);

    // Derived Plan Lists
    const allPlans = useMemo(() => [...accessData.authorizedPlans, ...accessData.availablePlans], [accessData.authorizedPlans, accessData.availablePlans]);

    // Computed
    const filteredAvailablePlans = useMemo(() => {
        const assignedIds = new Set(accessData.authorizedPlans.map(g => g.id));
        const pendingIds = new Set(pendingDrops.map(g => g.id));

        return accessData.availablePlans.filter(g =>
            !assignedIds.has(g.id) &&
            !pendingIds.has(g.id) &&
            (g.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                (g.description?.toLowerCase().includes(searchQuery.toLowerCase()) ?? false))
        );
    }, [accessData.availablePlans, accessData.authorizedPlans, pendingDrops, searchQuery]);

    // Navigation Logic
    const handleManagePlan = useCallback((planId: string) => {
        router.push(`/wallet-management/plans/${planId}?from=/wallet-management/${encodeURIComponent(walletAddress)}`);
    }, [router, walletAddress]);

    // DND Handlers
    const handleDragStart = useCallback((event: DragStartEvent) => {
        const { active } = event;
        const allPermissions = [...accessData.availablePermissions, ...accessData.authorizedPermissions];

        const item = active.data.current?.type === 'plan'
            ? allPlans.find(p => p.id === active.id)
            : allPermissions.find(p => p.id === active.id);

        if (item) {
            setActiveDragItem(item);
        }
    }, [allPlans, accessData.availablePermissions, accessData.authorizedPermissions]);

    const handleDragEnd = useCallback((event: DragEndEvent) => {
        const { active, over } = event;
        setActiveDragItem(null);

        if (!over) { return; }

        if (active.data.current?.type === 'plan' && over.id === 'assigned-plan-list') {
            const plan = accessData.availablePlans.find(g => g.id === active.id);
            if (plan && !pendingDrops.find(p => p.id === plan.id)) {
                setPendingDrops(prev => [...prev, plan]);
                toast.success(`Staged "${plan.name}" for assignment`);
            }
        }
    }, [accessData.availablePlans, pendingDrops]);

    // Save Actions
    const handleSavePendingChanges = useCallback(async () => {
        if (pendingDrops.length === 0) { return; }
        setIsSavingPending(true);
        try {
            await Promise.all(pendingDrops.map(plan => assignPlan(plan.id)));
            toast.success('Access plans assigned successfully');
            setPendingDrops([]);
            refreshAccess();
        } catch (_err) {
            console.error('Failed to save changes:', _err);
            toast.error('Failed to assign plans');
        } finally {
            setIsSavingPending(false);
        }
    }, [pendingDrops, assignPlan, refreshAccess]);

    const handleCopyAddress = useCallback(async () => {
        if (!walletData.wallet) { return; }
        const success = await copyToClipboard(walletData.wallet.walletAddress);
        if (success) {
            setCopied(true);
            toast.success('Address copied!');
            setTimeout(() => setCopied(false), 2000);
        }
    }, [walletData.wallet]);

    const statusConfig = walletData.wallet ? STATUS_CONFIG[walletData.wallet.status] : STATUS_CONFIG.active;
    const hasPending = pendingDrops.length > 0;

    if (authLoading === true || walletData.isLoading === true) {
        return (
            <div className="p-6">
                <div className="max-w-6xl mx-auto space-y-6">
                    <Skeleton className="h-10 w-32" />
                    <Skeleton className="h-48 w-full rounded-xl" />
                    <Skeleton className="h-96 w-full rounded-xl" />
                </div>
            </div>
        );
    }

    if (!walletData.wallet) { return null; }

    return (
        <DndContext
            sensors={sensors}
            onDragStart={handleDragStart}
            onDragEnd={handleDragEnd}
        >
            <div className="p-3 sm:p-6">
                <div className="max-w-6xl mx-auto space-y-6">

                    {/* Header */}
                    <div className="flex items-center gap-4">
                        <Link
                            href="/wallet-management"
                            className="p-2 rounded-xl bg-white border hover:bg-gray-50 transition-colors"
                        >
                            <ArrowLeft className="h-5 w-5" />
                        </Link>
                        <div className="flex-1">
                            <h1 className="text-2xl font-bold flex items-center gap-2">
                                <span>👛</span>
                                Wallet Details
                            </h1>
                            <p className="text-sm text-gray-500">
                                Manage wallet access and plans
                            </p>
                        </div>
                        <Button
                            variant="outline"
                            onClick={() => { walletData.loadWallet(); refreshAccess(); }}
                            disabled={walletData.isRefreshing === true}
                            className="gap-2"
                        >
                            <RefreshCw className={cn('h-4 w-4', walletData.isRefreshing === true && 'animate-spin')} />
                            Refresh
                        </Button>
                    </div>

                    {/* Wallet Identity & Access Management Section (Merged Metadata & Plan Assignment) */}
                    <div className="pt-4 border-t border-white/10 space-y-6">
                        <div className="flex items-center justify-between">
                            <div>
                                <h2 className="text-xl font-bold text-white flex items-center gap-2">
                                    <ShieldCheck className="h-5 w-5 text-purple-400" />
                                    Wallet Identity & Access Management
                                </h2>
                                <p className="text-sm text-slate-500 mt-1">Manage wallet identification, subscription plans, and access plans</p>
                            </div>
                        </div>

                        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">

                            {/* LEFT COLUMN: Selection (Available Plans) */}
                            <div className="lg:col-span-5 flex flex-col gap-4">
                                <WalletAvailablePlansCard
                                    plans={filteredAvailablePlans}
                                    searchQuery={searchQuery}
                                    setSearchQuery={setSearchQuery}
                                    onManagePlan={handleManagePlan}
                                />
                            </div>

                            {/* RIGHT COLUMN: Management (Details, Current Plan, Assigned Plans) */}
                            <div className="lg:col-span-7 flex flex-col gap-6">
                                {/* Section 1: Wallet Details */}
                                <WalletMetadataCard
                                    wallet={walletData.wallet}
                                    metadataForm={metadataForm.metadataForm}
                                    setMetadataForm={metadataForm.setMetadataForm}
                                    hasChanges={metadataForm.hasChanges}
                                    setHasChanges={metadataForm.setHasChanges}
                                    isSaving={metadataForm.isSaving}
                                    onSave={() => metadataForm.handleSave(walletAddress, walletData.loadWallet)}
                                    onDiscard={() => {
                                        if (walletData.wallet) {
                                            metadataForm.setMetadataForm({ label: walletData.wallet.label ?? '', note: walletData.wallet.note ?? '' });
                                            metadataForm.setHasChanges(false);
                                        }
                                    }}
                                    onCopyAddress={handleCopyAddress}
                                />

                                {/* Section 2: Current Subscription (Detailed) */}
                                {subscriptionData.activeSub !== null && (
                                    <WalletSubscriptionCard subscription={subscriptionData.activeSub} />
                                )}

                                {/* Section 3: Assigned Plans */}
                                <WalletAssignedPlansCard
                                    authorizedPlans={accessData.authorizedPlans}
                                    pendingDrops={pendingDrops}
                                    searchQuery={assignedSearchQuery}
                                    setSearchQuery={setAssignedSearchQuery}
                                    onEdit={(item: AccessItem) => setEditingItem({ item, type: 'plan' })}
                                    onManage={(item: AccessItem) => handleManagePlan(item.id)}
                                    onDelete={(id: string) => {
                                        if (pendingDrops.find(p => p.id === id) !== undefined) {
                                            setPendingDrops(prev => prev.filter(p => p.id !== id));
                                            toast.info('Removed from staging');
                                        } else {
                                            const plan = accessData.authorizedPlans.find(g => g.id === id);
                                            if (window.confirm(`Are you sure you want to remove access to "${plan?.name ?? 'this plan'}"?`)) {
                                                const handleRemove = async () => {
                                                    try {
                                                        await removePlan(id);
                                                        toast.success('Plan access revoked');
                                                        void refreshAccess();
                                                    } catch (_err) {
                                                        toast.error('Failed to revoke plan access');
                                                    }
                                                };
                                                void handleRemove();
                                            }
                                        }
                                    }}
                                    onDiscard={() => setPendingDrops([])}
                                    onSave={() => { void handleSavePendingChanges(); }}
                                    isSaving={isSavingPending}
                                    hasPending={hasPending}
                                />
                            </div>
                        </div>
                    </div>

                </div>

                {/* DND Overlay */}
                <DragOverlay>
                    {activeDragItem !== null && (
                        <div className={cn(
                            "flex items-center gap-2 px-3 py-2 bg-white dark:bg-gray-800 rounded-lg shadow-xl border opacity-90 scale-105 pointer-events-none",
                            activeDragItem.type === 'permission' ? "border-blue-500" : "border-purple-500"
                        )}>
                            {activeDragItem.type === 'permission' ? (
                                <Key className="h-4 w-4 text-purple-500" />
                            ) : (
                                <Package className="h-4 w-4 text-purple-500" />
                            )}
                            <span className="font-medium text-sm">{activeDragItem.name}</span>
                        </div>
                    )}
                </DragOverlay>

                {/* Trash Zone */}
                <TrashDropZone isDragging={activeDragItem !== null} />

                {/* Disable/Re-enable Modals */}
                {showDisableModal === true && (
                    <DisableWalletModal
                        walletAddress={walletData.wallet?.walletAddress ?? ''}
                        isOpen={true}
                        onClose={() => setShowDisableModal(false)}
                        onConfirm={walletActions.handleDisable}
                        isLoading={walletActions.isLoading}
                    />
                )}
                {showReenableModal === true && walletData.wallet.disableInfo !== undefined && (
                    <ReenableWalletModal
                        walletAddress={walletData.wallet.walletAddress}
                        disableInfo={walletData.wallet.disableInfo}
                        isOpen={true}
                        onClose={() => setShowReenableModal(false)}
                        onConfirm={walletActions.handleReenable}
                        isLoading={walletActions.isLoading}
                    />
                )}

                {/* Expiry Date Picker for Editing */}
                {editingItem !== null && (
                    <ExpiryDatePicker
                        itemName={editingItem.item.name}
                        itemType={editingItem.type}
                        isOpen={true}
                        onConfirm={(date) => {
                            const handleConfirm = async () => {
                                try {
                                    const expiry = date !== null ? date.toISOString() : undefined;

                                    // Check if item is in pending list
                                    const isPending = pendingDrops.some(p => p.id === editingItem.item.id);

                                    if (isPending === true) {
                                        setPendingDrops(prev => prev.map(p =>
                                            p.id === editingItem.item.id
                                                ? { ...p, expiresAt: expiry }
                                                : p
                                        ));
                                        toast.success(`Updated pending expiry for "${editingItem.item.name}"`);
                                    } else if (editingItem.type === 'plan') {
                                        await assignPlan(editingItem.item.id, expiry);
                                        toast.success(`Updated expiry for "${editingItem.item.name}"`);
                                        void refreshAccess();
                                    }
                                } catch (_err) {
                                    toast.error('Failed to update expiry');
                                }
                                setEditingItem(null);
                            };
                            void handleConfirm();
                        }}
                        onCancel={() => setEditingItem(null)}
                    />
                )}
            </div>
        </DndContext>
    );
}

