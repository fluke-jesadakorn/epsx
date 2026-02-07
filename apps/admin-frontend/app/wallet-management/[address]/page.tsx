/**
 * Wallet Detail Page
 * Unified view for wallet info, plans, and permissions with DND support
 */
'use client';

import { TrashDropZone } from '@/components/wallet/TrashDropZone';
import {
    DndContext,
    DragEndEvent,
    DragOverlay,
    DragStartEvent,
    MouseSensor,
    TouchSensor,
    useSensor,
    useSensors
} from '@dnd-kit/core';
import {
    ArrowLeft,
    Copy,
    ExternalLink,
    Key, Loader2, Package,
    RefreshCw,
    Save,
    ShieldCheck
} from 'lucide-react';
import Link from 'next/link';
import { useParams, useRouter } from 'next/navigation';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { toast } from 'sonner';

import {
    disableWalletAction,
    enableWalletAction,
    fetchWalletDetailAction,
    updateWalletMetadataAction
} from '@/app/wallet-management/plan-actions';
import { DisableWalletModal, type DisableWalletData } from '@/components/wallet/DisableWalletModal';
import { ExpiryDatePicker } from '@/components/wallet/ExpiryDatePicker';
import { ReenableWalletModal, type ReenableWalletData } from '@/components/wallet/ReenableWalletModal';
import type { WalletData, WalletStatus } from '@/components/wallet/types';
import { DraggablePlanItem, DroppablePlanList } from '@/components/wallet/WalletComponents';
import { AccessItem, useWalletAccess } from '@/hooks/useWalletAccess';
import { cn, copyToClipboard } from '@/lib/utils';
import { createPlansClient, type SubscriptionResponse } from '@/shared/api/plans';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { Badge } from '@/shared/components/ui/badge';
import { Button } from '@/shared/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/components/ui/card';
import { Input } from '@/shared/components/ui/input';
import { Label } from '@/shared/components/ui/label';
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

function formatTimeAgo(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffInHours = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60));

    if (diffInHours < 1) { return 'Just now'; }
    if (diffInHours < 24) { return `${diffInHours} hours ago`; }

    const diffInDays = Math.floor(diffInHours / 24);
    if (diffInDays === 1) { return 'Yesterday'; }
    if (diffInDays < 30) { return `${diffInDays} days ago`; }

    return date.toLocaleDateString();
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

    // Core Data State
    const [wallet, setWallet] = useState<WalletData | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [isRefreshing, setIsRefreshing] = useState(false);
    const [copied, setCopied] = useState(false);

    // Access Management Hook
    const {
        data: accessData,
        isLoading: isAccessLoading,
        assignPlan,
        removePlan,
        assignPermission,
        revokePermission,
        refresh: refreshAccess
    } = useWalletAccess(walletAddress);

    // DND & Pending Changes State
    const [activeDragItem, setActiveDragItem] = useState<AccessItem | null>(null);
    const [pendingDrops, setPendingDrops] = useState<AccessItem[]>([]);
    const [isSavingPending, setIsSavingPending] = useState(false);
    const [editingItem, setEditingItem] = useState<{ item: AccessItem, type: 'plan' | 'permission' } | null>(null);

    // Metadata Editing State
    const [metadataForm, setMetadataForm] = useState({ label: '', note: '' });
    const [isSavingMetadata, setIsSavingMetadata] = useState(false);
    const [hasMetadataChanges, setHasMetadataChanges] = useState(false);


    // Filter State
    const [searchQuery, setSearchQuery] = useState('');
    const [assignedSearchQuery, setAssignedSearchQuery] = useState('');

    // Modals
    const [showDisableModal, setShowDisableModal] = useState(false);
    const [showReenableModal, setShowReenableModal] = useState(false);
    const [activeSub, setActiveSub] = useState<SubscriptionResponse | null>(null);
    const [isLoadingSub, setIsLoadingSub] = useState(false);
    const [isActionLoading, setIsActionLoading] = useState(false);

    // Derived Plan Lists
    const allPlans = useMemo(() => [...accessData.authorizedPlans, ...accessData.availablePlans], [accessData.authorizedPlans, accessData.availablePlans]);


    // Load wallet data
    const loadWallet = useCallback(async () => {
        if (!walletAddress) { return; }

        try {
            setIsRefreshing(true);
            const walletData = await fetchWalletDetailAction(walletAddress);
            setWallet(walletData);

            if (walletData) {
                setMetadataForm({
                    label: walletData.label || '',
                    note: walletData.note || '',
                });
                setHasMetadataChanges(false);
            }
        } catch (err) {
            console.error('Failed to load wallet:', err);
            toast.error('Failed to load wallet details');
            router.push('/wallet-management');
        } finally {
            setIsLoading(false);
            setIsRefreshing(false);
        }
    }, [walletAddress, router]);

    useEffect(() => {
        if (isAuthenticated && !authLoading) {
            loadWallet();
        }
    }, [isAuthenticated, authLoading, loadWallet]);

    useEffect(() => {
        if (walletAddress) {
            // Fetch detailed subscription info
            const loadSubscription = async () => {
                setIsLoadingSub(true);
                try {
                    const client = createPlansClient(createAdminApiClient());
                    // Note: Ideally backend supports filtering by user_id, here we filter client-side as fallback
                    const res = await client.getSubscriptions({ limit: 100 });
                    if (res && res.success && res.data && res.data.subscriptions) {
                        const sub = res.data.subscriptions.find((s: SubscriptionResponse) =>
                            s.user_id === walletAddress && s.status === 'active'
                        );
                        if (sub) {setActiveSub(sub);}
                    }
                } catch (e) {
                    console.error('Failed to load subscription details', e);
                } finally {
                    setIsLoadingSub(false);
                }
            };
            loadSubscription();
        }
    }, [walletAddress]);

    // Computed
    const filteredAvailablePlans = useMemo(() => {
        const assignedIds = new Set(accessData.authorizedPlans.map(g => g.id));
        const pendingIds = new Set(pendingDrops.map(g => g.id));

        return accessData.availablePlans.filter(g =>
            !assignedIds.has(g.id) &&
            !pendingIds.has(g.id) &&
            (g.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                g.description?.toLowerCase().includes(searchQuery.toLowerCase()))
        );
    }, [accessData.availablePlans, accessData.authorizedPlans, pendingDrops, searchQuery]);

    // Navigation Logic
    const handleManagePlan = (planId: string) => {
        router.push(`/wallet-management/plans/${planId}?from=/wallet-management/${encodeURIComponent(walletAddress)}`);
    };


    // DND Handlers
    const handleDragStart = (event: DragStartEvent) => {
        const { active } = event;
        const allPermissions = [...accessData.availablePermissions, ...accessData.authorizedPermissions];
        // Note: normalized items for dragging

        const item = active.data.current?.type === 'plan'
            ? allPlans.find(p => p.id === active.id)
            : allPermissions.find(p => p.id === active.id);

        if (item) {
            setActiveDragItem(item);
        }
    };

    const handleDragEnd = (event: DragEndEvent) => {
        const { active, over } = event;
        setActiveDragItem(null);

        if (!over) { return; }

        // Dropped Plan into Assigned List (Top Section)
        if (active.data.current?.type === 'plan' && over.id === 'assigned-plan-list') {
            const plan = accessData.availablePlans.find(g => g.id === active.id);
            if (plan && !pendingDrops.find(p => p.id === plan.id)) {
                setPendingDrops(prev => [...prev, plan]);
                toast.success(`Staged "${plan.name}" for assignment`);
            }
        }


        // If dropping from assigned list (Legacy/Global removal? - Disabled per request but logic exists)
        /* 
        if (accessData.authorizedPermissions.find(p => p.id === itemId)) {
            revokePermission(itemId).then(() => {
                toast.success('Permission revoked');
                refreshAccess();
            });
        }
        */
    };

    // Save Actions
    const handleSavePendingChanges = async () => {
        if (pendingDrops.length === 0) {return;}
        setIsSavingPending(true);
        try {
            await Promise.all(pendingDrops.map(plan => assignPlan(plan.id)));
            toast.success('Access plans assigned successfully');
            setPendingDrops([]);
            refreshAccess();
        } catch (err) {
            console.error('Failed to save changes:', err);
            toast.error('Failed to assign plans');
        } finally {
            setIsSavingPending(false);
        }
    };

    const handleSaveMetadata = async () => {
        if (!wallet) { return; }
        setIsSavingMetadata(true);
        try {
            await updateWalletMetadataAction(wallet.walletAddress, {
                label: metadataForm.label || null,
                note: metadataForm.note || null,
            });
            toast.success('Wallet metadata updated');
            setHasMetadataChanges(false);
            await loadWallet();
        } catch (err) {
            console.error('Failed to update metadata:', err);
            toast.error('Failed to save changes');
        } finally {
            setIsSavingMetadata(false);
        }
    };

    const handleCopyAddress = async () => {
        if (!wallet) { return; }
        const success = await copyToClipboard(wallet.walletAddress);
        if (success) {
            setCopied(true);
            toast.success('Address copied!');
            setTimeout(() => setCopied(false), 2000);
        }
    };

    const handleDisableWallet = async (data: DisableWalletData) => {
        if (!wallet) { return; }
        setIsActionLoading(true);
        try {
            await disableWalletAction(wallet.walletAddress, {
                duration_days: data.duration === 'until_manual' ? null : data.duration,
                reason_category: data.reasonCategory,
                reason_details: data.reasonDetails,
                affected_platforms: data.affectedPlatforms,
                block_login: data.blockLogin,
                pause_subscriptions: data.pauseSubscriptions,
                notify_user: data.notifyUser,
            });
            toast.success('Wallet disabled successfully');
            setShowDisableModal(false);
            await loadWallet();
        } catch (err) {
            console.error('Failed to disable wallet:', err);
            toast.error(err instanceof Error ? err.message : 'Failed to disable wallet');
        } finally {
            setIsActionLoading(false);
        }
    };

    const handleReenableWallet = async (data: ReenableWalletData) => {
        if (!wallet) { return; }
        setIsActionLoading(true);
        try {
            await enableWalletAction(wallet.walletAddress, {
                platforms_to_enable: data.platformsToEnable,
                restore_permissions: data.restorePermissions,
                resume_subscriptions: data.resumeSubscriptions,
                resolution_note: data.resolutionNote,
            });
            toast.success('Wallet re-enabled successfully');
            setShowReenableModal(false);
            await loadWallet();
        } catch (err) {
            console.error('Failed to re-enable wallet:', err);
            toast.error(err instanceof Error ? err.message : 'Failed to re-enable wallet');
        } finally {
            setIsActionLoading(false);
        }
    };

    const statusConfig = wallet ? STATUS_CONFIG[wallet.status] : STATUS_CONFIG.active;
    const hasPending = pendingDrops.length > 0;

    if (authLoading || isLoading) {
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

    if (!wallet) { return null; }

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
                            onClick={() => { loadWallet(); refreshAccess(); }}
                            disabled={isRefreshing}
                            className="gap-2"
                        >
                            <RefreshCw className={cn('h-4 w-4', isRefreshing && 'animate-spin')} />
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
                                <Card className="flex-1 flex flex-col h-full border border-white/10 bg-slate-900/50 shadow-lg">
                                    <CardHeader className="pb-3 border-b border-white/5 bg-white/5">
                                        <div className="flex items-center justify-between mb-3">
                                            <div className="space-y-1">
                                                <CardTitle className="text-sm font-semibold text-slate-200">Available Plans</CardTitle>
                                                <p className="text-xs text-slate-500">Drag items to the right to assign</p>
                                            </div>
                                            <Badge variant="outline" className="text-xs border-blue-500/30 text-blue-400">
                                                {filteredAvailablePlans.length} available
                                            </Badge>
                                        </div>
                                        <Input
                                            placeholder="Search plans..."
                                            value={searchQuery}
                                            onChange={e => setSearchQuery(e.target.value)}
                                            className="h-8 text-xs bg-slate-950/50 border-white/10 text-slate-200 placeholder:text-slate-600 focus:border-blue-500/50"
                                        />
                                    </CardHeader>
                                    <CardContent className="p-4 overflow-y-auto max-h-[600px]">
                                        <div className="grid grid-cols-1 gap-3">
                                            {filteredAvailablePlans.map(plan => (
                                                <DraggablePlanItem
                                                    key={plan.id}
                                                    id={plan.id}
                                                    label={plan.name}
                                                    description={plan.description}
                                                    onManage={() => handleManagePlan(plan.id)}
                                                />
                                            ))}
                                            {filteredAvailablePlans.length === 0 && (
                                                <div className="flex flex-col items-center justify-center py-12 text-slate-500">
                                                    <Package className="h-10 w-10 mb-3 opacity-20" />
                                                    <p>No available plans found.</p>
                                                </div>
                                            )}
                                        </div>
                                    </CardContent>
                                </Card>
                            </div>

                            {/* RIGHT COLUMN: Management (Details, Current Plan, Assigned Plans) */}
                            <div className="lg:col-span-7 flex flex-col gap-6">
                                {/* Section 1: Wallet Details */}
                                <Card className="border-0 shadow-lg bg-slate-900/80 ring-1 ring-white/10 overflow-hidden">
                                    <div className="h-1 bg-gradient-to-r from-amber-500 to-orange-500" />
                                    <CardHeader className="pb-4 border-b border-white/5 bg-white/5">
                                        <CardTitle className="text-sm font-semibold uppercase tracking-wider text-amber-500 flex items-center justify-between">
                                            <span>Wallet Details</span>
                                            <div className="flex items-center gap-3">
                                                <Badge className={cn('px-2 py-0.5 border text-[10px] font-bold shadow-sm', statusConfig.className)}>
                                                    {statusConfig.label}
                                                </Badge>
                                                <div className="flex items-center gap-2 px-3 py-1 rounded-full bg-black/20 border border-white/10 normal-case font-mono tracking-tight text-[10px]">
                                                    <span className="text-slate-400">{walletAddress}</span>
                                                    <button onClick={handleCopyAddress} className="text-slate-500 hover:text-white transition-colors">
                                                        <Copy className="h-2.5 w-2.5" />
                                                    </button>
                                                </div>
                                            </div>
                                        </CardTitle>
                                    </CardHeader>
                                    <CardContent className="pt-6 grid grid-cols-1 md:grid-cols-2 gap-4">
                                        <div className="space-y-2">
                                            <Label className="text-slate-400 text-xs font-normal">Label</Label>
                                            <Input
                                                value={metadataForm.label}
                                                onChange={(e) => { setMetadataForm(p => ({ ...p, label: e.target.value })); setHasMetadataChanges(true); }}
                                                placeholder="e.g. Main Trading Wallet"
                                                className="h-9 bg-slate-950/50 border-white/10 text-slate-200 text-sm placeholder:text-slate-600 focus:border-purple-500/50 focus:ring-purple-500/20"
                                            />
                                        </div>
                                        <div className="space-y-2">
                                            <Label className="text-slate-400 text-xs font-normal">Private Note</Label>
                                            <Input
                                                value={metadataForm.note}
                                                onChange={(e) => { setMetadataForm(p => ({ ...p, note: e.target.value })); setHasMetadataChanges(true); }}
                                                placeholder="Internal notes..."
                                                className="h-9 bg-slate-950/50 border-white/10 text-slate-200 text-sm placeholder:text-slate-600 focus:border-purple-500/50 focus:ring-purple-500/20"
                                            />
                                        </div>
                                    </CardContent>
                                    <div className="px-6 pb-4 flex gap-2 justify-end pt-2">
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            onClick={() => {
                                                if (wallet) {
                                                    setMetadataForm({ label: wallet.label || '', note: wallet.note || '' });
                                                    setHasMetadataChanges(false);
                                                }
                                            }}
                                            disabled={!hasMetadataChanges}
                                            className="text-slate-400 hover:text-white h-8 text-xs font-medium"
                                        >
                                            Discard
                                        </Button>
                                        <Button
                                            size="sm"
                                            onClick={handleSaveMetadata}
                                            disabled={isSavingMetadata || !hasMetadataChanges}
                                            className="bg-purple-600 hover:bg-purple-700 text-white h-8 text-xs font-medium"
                                        >
                                            {isSavingMetadata && <Loader2 className="mr-2 h-3 w-3 animate-spin" />}
                                            <Save className="mr-2 h-3 w-3" />
                                            Update Details
                                        </Button>
                                    </div>
                                </Card>

                                {/* Section 2: Current Subscription (Detailed) */}
                                {activeSub && (
                                    <div className="relative group">
                                        <div className="absolute -inset-0.5 bg-gradient-to-br from-purple-500/10 to-blue-500/10 rounded-xl blur-sm opacity-75 group-hover:opacity-100 transition duration-1000"></div>
                                        <div className="relative bg-slate-900 border border-white/10 rounded-xl p-4 space-y-4">
                                            <div className="flex items-center justify-between">
                                                <div className="flex items-center gap-4">
                                                    <div className="w-10 h-10 rounded-lg bg-purple-500/10 flex items-center justify-center border border-purple-500/20">
                                                        <Package className="h-5 w-5 text-purple-400" />
                                                    </div>
                                                    <div>
                                                        <h4 className="text-sm font-bold text-white uppercase tracking-wider leading-none mb-1">Active Subscription</h4>
                                                        <p className="text-xs text-slate-400">
                                                            {activeSub.plan_name}
                                                        </p>
                                                    </div>
                                                </div>
                                                <div className="flex items-center gap-2">
                                                    <Badge className="bg-green-500/10 text-green-400 border-green-500/20 uppercase text-[10px] font-bold">Active</Badge>
                                                    <Link href={`/subscriptions/${activeSub.id}`}>
                                                        <Button size="sm" variant="outline" className="h-7 text-xs gap-1 border-white/10 hover:bg-white/5">
                                                            Manage
                                                            <ExternalLink className="h-3 w-3" />
                                                        </Button>
                                                    </Link>
                                                </div>
                                            </div>

                                            {/* Usage & Quotas */}
                                            {activeSub.current_usage && Object.keys(activeSub.current_usage).length > 0 && (
                                                <div className="mt-3 pt-3 border-t border-white/5 grid grid-cols-2 gap-4">
                                                    {Object.entries(activeSub.current_usage).map(([key, value]) => (
                                                        <div key={key}>
                                                            <span className="text-[10px] text-slate-500 uppercase font-medium">{key.replace('_', ' ')}</span>
                                                            <div className="text-sm font-mono text-slate-300">
                                                                {value} / {activeSub.quota_limits?.[key] || '∞'}
                                                            </div>
                                                        </div>
                                                    ))}
                                                </div>
                                            )}

                                            <div className="flex justify-between items-center text-xs text-slate-500 pt-2">
                                                <span>Started: {activeSub.created_at ? new Date(activeSub.created_at).toLocaleDateString() : 'Never'}</span>
                                                <span>Expires: {activeSub.expires_at ? new Date(activeSub.expires_at).toLocaleDateString() : 'Never'}</span>
                                            </div>
                                        </div>
                                    </div>
                                )}

                                {/* Section 3: Assigned Plans */}
                                <Card className="flex-1 flex flex-col h-full border border-white/10 bg-slate-900/50 shadow-lg min-h-0 overflow-hidden">
                                    <div className="h-1 bg-gradient-to-r from-blue-500 to-cyan-500" />
                                    <CardHeader className="pb-3 border-b border-white/5 bg-white/5">
                                        <div className="flex items-center justify-between mb-3">
                                            <div className="flex items-center gap-2">
                                                <CardTitle className="text-sm font-semibold uppercase tracking-wider text-blue-400">Assigned Plans</CardTitle>
                                                <Badge variant="outline" className="text-[10px] h-4 px-1 border-blue-500/30 text-blue-400 leading-none">
                                                    {accessData.authorizedPlans.length + pendingDrops.length}
                                                </Badge>
                                            </div>
                                            <Badge variant="outline" className="text-[10px] font-normal border-blue-500/20 text-blue-400/60 uppercase tracking-widest">
                                                Drag here
                                            </Badge>
                                        </div>
                                        <Input
                                            placeholder="Search within assignments..."
                                            value={assignedSearchQuery}
                                            onChange={e => setAssignedSearchQuery(e.target.value)}
                                            className="h-8 text-xs bg-slate-950/50 border-white/10 text-slate-200 placeholder:text-slate-600 focus:border-blue-500/50"
                                        />
                                    </CardHeader>
                                    <CardContent className="p-0 flex flex-col min-h-0">
                                        <div className="p-4 overflow-y-auto max-h-[400px]">
                                            <DroppablePlanList
                                                id="assigned-plan-list"
                                                items={accessData.authorizedPlans.filter(g =>
                                                    g.name.toLowerCase().includes(assignedSearchQuery.toLowerCase())
                                                )}
                                                pendingItems={pendingDrops.filter(g =>
                                                    g.name.toLowerCase().includes(assignedSearchQuery.toLowerCase())
                                                )}
                                                emptyMessage="Drag plans here from the left to assign access"
                                                onEdit={(item) => setEditingItem({ item, type: 'plan' })}
                                                onManage={(item) => handleManagePlan(item.id)}
                                                onDelete={(id) => {
                                                    if (pendingDrops.find(p => p.id === id)) {
                                                        setPendingDrops(prev => prev.filter(p => p.id !== id));
                                                        toast.info('Removed from staging');
                                                    } else {
                                                        const plan = accessData.authorizedPlans.find(g => g.id === id);
                                                        if (confirm(`Are you sure you want to remove access to "${plan?.name}"?`)) {
                                                            removePlan(id).then(() => {
                                                                toast.success('Plan access revoked');
                                                                refreshAccess();
                                                            });
                                                        }
                                                    }
                                                }}
                                            />
                                        </div>
                                        {/* Action Button Bar */}
                                        <div className="p-3 border-t border-white/5 bg-slate-950/50 flex gap-2 justify-end">
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                onClick={() => setPendingDrops([])}
                                                disabled={!hasPending}
                                                className="text-slate-400 hover:text-white h-8 text-xs font-medium"
                                            >
                                                Discard
                                            </Button>
                                            <Button
                                                size="sm"
                                                onClick={handleSavePendingChanges}
                                                disabled={isSavingPending || !hasPending}
                                                className="bg-purple-600 hover:bg-purple-700 text-white h-8 text-xs font-medium"
                                            >
                                                {isSavingPending ? <Loader2 className="h-3 w-3 animate-spin mr-1" /> : <Save className="h-3 w-3 mr-1" />}
                                                Save Assignments
                                            </Button>
                                        </div>
                                    </CardContent>
                                </Card>
                            </div>
                        </div>
                    </div>


                </div>

                {/* DND Overlay */}
                <DragOverlay>
                    {activeDragItem && (
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
                <TrashDropZone isDragging={!!activeDragItem} />

                {/* Disable/Re-enable Modals */}
                {showDisableModal && (
                    <DisableWalletModal
                        walletAddress={wallet.walletAddress}
                        isOpen={true}
                        onClose={() => setShowDisableModal(false)}
                        onConfirm={handleDisableWallet}
                        isLoading={isActionLoading}
                    />
                )}
                {showReenableModal && wallet.disableInfo && (
                    <ReenableWalletModal
                        walletAddress={wallet.walletAddress}
                        disableInfo={wallet.disableInfo}
                        isOpen={true}
                        onClose={() => setShowReenableModal(false)}
                        onConfirm={handleReenableWallet}
                        isLoading={isActionLoading}
                    />
                )}

                {/* Expiry Date Picker for Editing */}
                {editingItem && (
                    <ExpiryDatePicker
                        itemName={editingItem.item.name}
                        itemType={editingItem.type}
                        isOpen={true}
                        onConfirm={async (date) => {
                            try {
                                const expiry = date ? date.toISOString() : undefined;

                                // Check if item is in pending list
                                const isPending = pendingDrops.some(p => p.id === editingItem.item.id);

                                if (isPending) {
                                    setPendingDrops(prev => prev.map(p =>
                                        p.id === editingItem.item.id
                                            ? { ...p, expiresAt: expiry }
                                            : p
                                    ));
                                    toast.success(`Updated pending expiry for "${editingItem.item.name}"`);
                                } else {
                                    if (editingItem.type === 'plan') {
                                        await assignPlan(editingItem.item.id, expiry);
                                        toast.success(`Updated expiry for "${editingItem.item.name}"`);
                                        refreshAccess();
                                    }
                                    // Add permission edit logic here if needed
                                }
                            } catch (err) {
                                toast.error('Failed to update expiry');
                            }
                            setEditingItem(null);
                        }}
                        onCancel={() => setEditingItem(null)}
                    />
                )}
            </div>
        </DndContext>
    );
}

