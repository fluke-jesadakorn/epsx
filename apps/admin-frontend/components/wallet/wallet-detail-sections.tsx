'use client';

import type { AccessItem } from '@/hooks/use-wallet-access';
import { cn } from '@/lib/utils';
import type { SubscriptionResponse } from '@/shared/api/plans';
import { Badge } from '@/shared/components/ui/badge';
import { Button } from '@/shared/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/components/ui/card';
import { Input } from '@/shared/components/ui/input';
import { Label } from '@/shared/components/ui/label';
import { Building2, ChevronDown, Code2, Copy, ExternalLink, Loader2, Package, Save, User, Wrench } from 'lucide-react';
import Link from 'next/link';
import { useCallback, useMemo, useState } from 'react';
import type { WalletData, WalletStatus } from './types';
import { DraggablePlanItem, DroppablePlanList } from './wallet-components';

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
};

const PLAN_GROUP_CONFIG: Record<string, { label: string; icon: React.ReactNode }> = {
    personal: { label: 'Personal', icon: <User className="h-3.5 w-3.5" /> },
    enterprise: { label: 'Enterprise', icon: <Building2 className="h-3.5 w-3.5" /> },
    api: { label: 'API', icon: <Code2 className="h-3.5 w-3.5" /> },
    custom: { label: 'Custom', icon: <Wrench className="h-3.5 w-3.5" /> },
};

const PLAN_GROUP_ORDER = ['personal', 'enterprise', 'api', 'custom'];

export function WalletAvailablePlansCard({
    plans,
    searchQuery,
    setSearchQuery,
    onManagePlan
}: {
    plans: AccessItem[];
    searchQuery: string;
    setSearchQuery: (query: string) => void;
    onManagePlan: (id: string) => void;
}) {
    const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

    const toggleGroup = useCallback((g: string) => {
        setCollapsed(prev => ({ ...prev, [g]: !(prev[g] ?? false) }));
    }, []);

    const grouped = useMemo(() => {
        const map: Record<string, AccessItem[]> = {};
        for (const g of PLAN_GROUP_ORDER) { map[g] = []; }
        for (const p of plans) {
            const g = p.planGroup ?? 'personal';
            (map[g] ??= []).push(p);
        }
        return map;
    }, [plans]);

    const hasPlans = plans.length > 0;

    return (
        <Card className="flex-1 flex flex-col h-full border border-border/20 bg-card shadow-lg">
            <CardHeader className="pb-3 border-b border-border/20 bg-muted/30">
                <div className="flex items-center justify-between mb-3">
                    <div className="space-y-1">
                        <CardTitle className="text-sm font-semibold text-gray-900 dark:text-foreground">Available Plans</CardTitle>
                        <p className="text-xs text-gray-500 dark:text-muted-foreground">Drag items to the right to assign</p>
                    </div>
                    <Badge variant="outline" className="text-xs border-blue-500/30 text-blue-600 dark:text-blue-400">
                        {plans.length} available
                    </Badge>
                </div>
                <Input
                    placeholder="Search plans..."
                    value={searchQuery}
                    onChange={e => setSearchQuery(e.target.value)}
                    className="h-8 text-xs bg-white/90 dark:bg-background/50 border-border/20 text-gray-800 dark:text-foreground placeholder:text-gray-400 dark:placeholder:text-slate-600 focus:border-blue-500/50"
                />
            </CardHeader>
            <CardContent className="p-0 overflow-y-auto max-h-[600px]">
                {hasPlans ? (
                    PLAN_GROUP_ORDER.map(g => {
                        const groupPlans = grouped[g] ?? [];
                        if (groupPlans.length === 0) { return null; }
                        const cfg = PLAN_GROUP_CONFIG[g];
                        const isCollapsed = collapsed[g] === true;
                        return (
                            <div key={g}>
                                <button
                                    type="button"
                                    onClick={() => toggleGroup(g)}
                                    className="w-full flex items-center gap-2 px-4 py-2 text-[10px] font-semibold uppercase tracking-wider text-gray-500 dark:text-muted-foreground hover:bg-muted/30 transition-colors"
                                >
                                    {cfg?.icon}
                                    <span className="flex-1 text-left">{cfg?.label}</span>
                                    <Badge variant="secondary" className="bg-muted/30 text-gray-500 dark:text-muted-foreground text-[10px] px-1.5 py-0">
                                        {groupPlans.length}
                                    </Badge>
                                    <ChevronDown className={cn('h-3 w-3 transition-transform', isCollapsed && '-rotate-90')} />
                                </button>
                                {!isCollapsed && (
                                    <div className="grid grid-cols-1 gap-3 px-4 pb-3">
                                        {groupPlans.map(plan => (
                                            <DraggablePlanItem
                                                key={plan.id}
                                                id={plan.id}
                                                label={plan.name}
                                                description={plan.description}
                                                onManage={() => onManagePlan(plan.id)}
                                            />
                                        ))}
                                    </div>
                                )}
                            </div>
                        );
                    })
                ) : (
                    <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
                        <Package className="h-10 w-10 mb-3 opacity-20" />
                        <p>No available plans found.</p>
                    </div>
                )}
            </CardContent>
        </Card>
    );
}

export function WalletMetadataCard({
    wallet,
    metadataForm,
    setMetadataForm,
    hasChanges,
    setHasChanges,
    isSaving,
    onSave,
    onDiscard,
    onCopyAddress
}: {
    wallet: WalletData;
    metadataForm: { label: string; note: string };
    setMetadataForm: React.Dispatch<React.SetStateAction<{ label: string; note: string }>>;
    hasChanges: boolean;
    setHasChanges: (val: boolean) => void;
    isSaving: boolean;
    onSave: () => void;
    onDiscard: () => void;
    onCopyAddress: () => void;
}) {
    const statusConfig = STATUS_CONFIG[wallet.status];

    return (
        <Card className="border-0 shadow-lg bg-card ring-1 ring-white/10 overflow-hidden">
            <div className="h-1 bg-gradient-to-r from-amber-500 to-orange-500" />
            <CardHeader className="pb-4 border-b border-border/20 bg-muted/30">
                <CardTitle className="text-sm font-semibold uppercase tracking-wider text-amber-500 flex items-center justify-between">
                    <span>Wallet Details</span>
                    <div className="flex items-center gap-3">
                        <Badge className={cn('px-2 py-0.5 border text-[10px] font-bold shadow-sm', statusConfig.className)}>
                            {statusConfig.label}
                        </Badge>
                        <div className="flex items-center gap-2 px-3 py-1 rounded-full bg-gray-100 dark:bg-black/20 border border-border/20 normal-case font-mono tracking-tight text-[10px]">
                            <span className="text-gray-500 dark:text-muted-foreground">{wallet.walletAddress}</span>
                            <button onClick={onCopyAddress} className="text-muted-foreground hover:text-gray-900 dark:hover:text-white transition-colors">
                                <Copy className="h-2.5 w-2.5" />
                            </button>
                        </div>
                    </div>
                </CardTitle>
            </CardHeader>
            <CardContent className="pt-6 grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                    <Label className="text-gray-600 dark:text-muted-foreground text-xs font-normal">Label</Label>
                    <Input
                        value={metadataForm.label}
                        onChange={(e) => { setMetadataForm(p => ({ ...p, label: e.target.value })); setHasChanges(true); }}
                        placeholder="e.g. Main Trading Wallet"
                        className="h-9 bg-white/90 dark:bg-background/50 border-border/20 text-gray-800 dark:text-foreground text-sm placeholder:text-gray-400 dark:placeholder:text-slate-600 focus:border-purple-500/50 focus:ring-purple-500/20"
                    />
                </div>
                <div className="space-y-2">
                    <Label className="text-gray-600 dark:text-muted-foreground text-xs font-normal">Private Note</Label>
                    <Input
                        value={metadataForm.note}
                        onChange={(e) => { setMetadataForm(p => ({ ...p, note: e.target.value })); setHasChanges(true); }}
                        placeholder="Internal notes..."
                        className="h-9 bg-white/90 dark:bg-background/50 border-border/20 text-gray-800 dark:text-foreground text-sm placeholder:text-gray-400 dark:placeholder:text-slate-600 focus:border-purple-500/50 focus:ring-purple-500/20"
                    />
                </div>
            </CardContent>
            <div className="px-6 pb-4 flex gap-2 justify-end pt-2">
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onDiscard}
                    disabled={!hasChanges}
                    className="text-gray-500 dark:text-muted-foreground hover:text-gray-900 dark:hover:text-white h-8 text-xs font-medium"
                >
                    Discard
                </Button>
                <Button
                    size="sm"
                    onClick={onSave}
                    disabled={isSaving || !hasChanges}
                    className="bg-purple-600 hover:bg-purple-700 text-white h-8 text-xs font-medium"
                >
                    {isSaving && <Loader2 className="mr-2 h-3 w-3 animate-spin" />}
                    <Save className="mr-2 h-3 w-3" />
                    Update Details
                </Button>
            </div>
        </Card>
    );
}

export function WalletSubscriptionCard({
    subscription
}: {
    subscription: SubscriptionResponse;
}) {
    return (
        <div className="relative group">
            <div className="absolute -inset-0.5 bg-gradient-to-br from-purple-500/10 to-blue-500/10 rounded-xl blur-sm opacity-75 group-hover:opacity-100 transition duration-1000" />
            <div className="relative bg-card border border-border/20 rounded-xl p-4 space-y-4">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        <div className="w-10 h-10 rounded-lg bg-purple-500/10 flex items-center justify-center border border-purple-500/20">
                            <Package className="h-5 w-5 text-purple-400" />
                        </div>
                        <div>
                            <h4 className="text-sm font-bold text-foreground uppercase tracking-wider leading-none mb-1">Active Subscription</h4>
                            <p className="text-xs text-gray-500 dark:text-muted-foreground">
                                {subscription.plan_name}
                            </p>
                        </div>
                    </div>
                    <div className="flex items-center gap-2">
                        <Badge className="bg-green-500/10 text-green-400 border-green-500/20 uppercase text-[10px] font-bold">Active</Badge>
                        <Link href={`/wallet-management/access/plans/${subscription.id}`}>
                            <Button size="sm" variant="outline" className="h-7 text-xs gap-1 border-border/20 hover:bg-muted/30">
                                Manage
                                <ExternalLink className="h-3 w-3" />
                            </Button>
                        </Link>
                    </div>
                </div>

                {/* Usage & Quotas */}
                {subscription.current_usage && Object.keys(subscription.current_usage).length > 0 && (
                    <div className="mt-3 pt-3 border-t border-border/20 grid grid-cols-2 gap-4">
                        {Object.entries(subscription.current_usage).map(([key, value]) => (
                            <div key={key}>
                                <span className="text-[10px] text-muted-foreground uppercase font-medium">{key.replace('_', ' ')}</span>
                                <div className="text-sm font-mono text-gray-700 dark:text-muted-foreground">
                                    {String(value)} / {subscription.quota_limits?.[key] !== undefined ? String(subscription.quota_limits[key]) : '∞'}
                                </div>
                            </div>
                        ))}
                    </div>
                )}

                <div className="flex justify-between items-center text-xs text-gray-500 dark:text-muted-foreground pt-2">
                    <span>Started: {subscription.created_at !== '' ? new Date(subscription.created_at).toLocaleDateString() : 'Never'}</span>
                    <span>Expires: {subscription.expires_at !== undefined && subscription.expires_at !== '' ? new Date(subscription.expires_at).toLocaleDateString() : 'Never'}</span>
                </div>
            </div>
        </div>
    );
}

export function WalletAssignedPlansCard({
    authorizedPlans,
    pendingDrops,
    searchQuery,
    setSearchQuery,
    onEdit,
    onManage,
    onDelete,
    onDiscard,
    onSave,
    isSaving,
    hasPending
}: {
    authorizedPlans: AccessItem[];
    pendingDrops: AccessItem[];
    searchQuery: string;
    setSearchQuery: (query: string) => void;
    onEdit: (item: AccessItem) => void;
    onManage: (item: AccessItem) => void;
    onDelete: (id: string) => void;
    onDiscard: () => void;
    onSave: () => void;
    isSaving: boolean;
    hasPending: boolean;
}) {
    return (
        <Card className="flex-1 flex flex-col h-full border border-border/20 bg-card shadow-lg min-h-0 overflow-hidden">
            <div className="h-1 bg-gradient-to-r from-blue-500 to-cyan-500" />
            <CardHeader className="pb-3 border-b border-border/20 bg-muted/30">
                <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-2">
                        <CardTitle className="text-sm font-semibold uppercase tracking-wider text-blue-600 dark:text-blue-400">Assigned Plans</CardTitle>
                        <Badge variant="outline" className="text-[10px] h-4 px-1 border-blue-500/30 text-blue-600 dark:text-blue-400 leading-none">
                            {authorizedPlans.length + pendingDrops.length}
                        </Badge>
                    </div>
                    <Badge variant="outline" className="text-[10px] font-normal border-blue-500/20 text-blue-600/60 dark:text-blue-400/60 uppercase tracking-widest">
                        Drag here
                    </Badge>
                </div>
                <Input
                    placeholder="Search within assignments..."
                    value={searchQuery}
                    onChange={e => setSearchQuery(e.target.value)}
                    className="h-8 text-xs bg-white/90 dark:bg-background/50 border-border/20 text-gray-800 dark:text-foreground placeholder:text-gray-400 dark:placeholder:text-slate-600 focus:border-blue-500/50"
                />
            </CardHeader>
            <CardContent className="p-0 flex flex-col min-h-0">
                <div className="p-4 overflow-y-auto max-h-[400px]">
                    <DroppablePlanList
                        id="assigned-plan-list"
                        items={authorizedPlans.filter(g =>
                            g.name.toLowerCase().includes(searchQuery.toLowerCase())
                        )}
                        pendingItems={pendingDrops.filter(g =>
                            g.name.toLowerCase().includes(searchQuery.toLowerCase())
                        )}
                        emptyMessage="Drag plans here from the left to assign access"
                        onEdit={(item) => onEdit(item as unknown as AccessItem)}
                        onManage={(item) => onManage(item as unknown as AccessItem)}
                        onDelete={onDelete}
                    />
                </div>
                {/* Action Button Bar */}
                <div className="p-3 border-t border-border/20 bg-white/90 dark:bg-background/50 flex gap-2 justify-end">
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={onDiscard}
                        disabled={!hasPending}
                        className="text-gray-500 dark:text-muted-foreground hover:text-gray-900 dark:hover:text-white h-8 text-xs font-medium"
                    >
                        Discard
                    </Button>
                    <Button
                        size="sm"
                        onClick={onSave}
                        disabled={isSaving || !hasPending}
                        className="bg-purple-600 hover:bg-purple-700 text-white h-8 text-xs font-medium"
                    >
                        {isSaving ? <Loader2 className="h-3 w-3 animate-spin mr-1" /> : <Save className="h-3 w-3 mr-1" />}
                        Save Assignments
                    </Button>
                </div>
            </CardContent>
        </Card>
    );
}
