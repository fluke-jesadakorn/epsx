'use client';

import type { AccessItem } from '@/hooks/use-wallet-access';
import { cn } from '@/lib/utils';
import type { SubscriptionResponse } from '@/shared/api/plans';
import { Badge } from '@/shared/components/ui/badge';
import { Button } from '@/shared/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/components/ui/card';
import { Input } from '@/shared/components/ui/input';
import { Label } from '@/shared/components/ui/label';
import { Copy, ExternalLink, Loader2, Package, Save } from 'lucide-react';
import Link from 'next/link';
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
    pending: {
        label: 'Pending',
        emoji: '⏳',
        className: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400 border-blue-200 dark:border-blue-800',
    },
};

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
    return (
        <Card className="flex-1 flex flex-col h-full border border-white/10 bg-slate-900/50 shadow-lg">
            <CardHeader className="pb-3 border-b border-white/5 bg-white/5">
                <div className="flex items-center justify-between mb-3">
                    <div className="space-y-1">
                        <CardTitle className="text-sm font-semibold text-slate-200">Available Plans</CardTitle>
                        <p className="text-xs text-slate-500">Drag items to the right to assign</p>
                    </div>
                    <Badge variant="outline" className="text-xs border-blue-500/30 text-blue-400">
                        {plans.length} available
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
                    {plans.map(plan => (
                        <DraggablePlanItem
                            key={plan.id}
                            id={plan.id}
                            label={plan.name}
                            description={plan.description}
                            onManage={() => onManagePlan(plan.id)}
                        />
                    ))}
                    {plans.length === 0 && (
                        <div className="flex flex-col items-center justify-center py-12 text-slate-500">
                            <Package className="h-10 w-10 mb-3 opacity-20" />
                            <p>No available plans found.</p>
                        </div>
                    )}
                </div>
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
                            <span className="text-slate-400">{wallet.walletAddress}</span>
                            <button onClick={onCopyAddress} className="text-slate-500 hover:text-white transition-colors">
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
                        onChange={(e) => { setMetadataForm(p => ({ ...p, label: e.target.value })); setHasChanges(true); }}
                        placeholder="e.g. Main Trading Wallet"
                        className="h-9 bg-slate-950/50 border-white/10 text-slate-200 text-sm placeholder:text-slate-600 focus:border-purple-500/50 focus:ring-purple-500/20"
                    />
                </div>
                <div className="space-y-2">
                    <Label className="text-slate-400 text-xs font-normal">Private Note</Label>
                    <Input
                        value={metadataForm.note}
                        onChange={(e) => { setMetadataForm(p => ({ ...p, note: e.target.value })); setHasChanges(true); }}
                        placeholder="Internal notes..."
                        className="h-9 bg-slate-950/50 border-white/10 text-slate-200 text-sm placeholder:text-slate-600 focus:border-purple-500/50 focus:ring-purple-500/20"
                    />
                </div>
            </CardContent>
            <div className="px-6 pb-4 flex gap-2 justify-end pt-2">
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onDiscard}
                    disabled={!hasChanges}
                    className="text-slate-400 hover:text-white h-8 text-xs font-medium"
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
            <div className="relative bg-slate-900 border border-white/10 rounded-xl p-4 space-y-4">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        <div className="w-10 h-10 rounded-lg bg-purple-500/10 flex items-center justify-center border border-purple-500/20">
                            <Package className="h-5 w-5 text-purple-400" />
                        </div>
                        <div>
                            <h4 className="text-sm font-bold text-white uppercase tracking-wider leading-none mb-1">Active Subscription</h4>
                            <p className="text-xs text-slate-400">
                                {subscription.plan_name}
                            </p>
                        </div>
                    </div>
                    <div className="flex items-center gap-2">
                        <Badge className="bg-green-500/10 text-green-400 border-green-500/20 uppercase text-[10px] font-bold">Active</Badge>
                        <Link href={`/subscriptions/${subscription.id}`}>
                            <Button size="sm" variant="outline" className="h-7 text-xs gap-1 border-white/10 hover:bg-white/5">
                                Manage
                                <ExternalLink className="h-3 w-3" />
                            </Button>
                        </Link>
                    </div>
                </div>

                {/* Usage & Quotas */}
                {subscription.current_usage && Object.keys(subscription.current_usage).length > 0 && (
                    <div className="mt-3 pt-3 border-t border-white/5 grid grid-cols-2 gap-4">
                        {Object.entries(subscription.current_usage).map(([key, value]) => (
                            <div key={key}>
                                <span className="text-[10px] text-slate-500 uppercase font-medium">{key.replace('_', ' ')}</span>
                                <div className="text-sm font-mono text-slate-300">
                                    {String(value)} / {subscription.quota_limits?.[key] !== undefined ? String(subscription.quota_limits[key]) : '∞'}
                                </div>
                            </div>
                        ))}
                    </div>
                )}

                <div className="flex justify-between items-center text-xs text-slate-500 pt-2">
                    <span>Started: {subscription.created_at ? new Date(subscription.created_at).toLocaleDateString() : 'Never'}</span>
                    <span>Expires: {subscription.expires_at ? new Date(subscription.expires_at).toLocaleDateString() : 'Never'}</span>
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
        <Card className="flex-1 flex flex-col h-full border border-white/10 bg-slate-900/50 shadow-lg min-h-0 overflow-hidden">
            <div className="h-1 bg-gradient-to-r from-blue-500 to-cyan-500" />
            <CardHeader className="pb-3 border-b border-white/5 bg-white/5">
                <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-2">
                        <CardTitle className="text-sm font-semibold uppercase tracking-wider text-blue-400">Assigned Plans</CardTitle>
                        <Badge variant="outline" className="text-[10px] h-4 px-1 border-blue-500/30 text-blue-400 leading-none">
                            {authorizedPlans.length + pendingDrops.length}
                        </Badge>
                    </div>
                    <Badge variant="outline" className="text-[10px] font-normal border-blue-500/20 text-blue-400/60 uppercase tracking-widest">
                        Drag here
                    </Badge>
                </div>
                <Input
                    placeholder="Search within assignments..."
                    value={searchQuery}
                    onChange={e => setSearchQuery(e.target.value)}
                    className="h-8 text-xs bg-slate-950/50 border-white/10 text-slate-200 placeholder:text-slate-600 focus:border-blue-500/50"
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
                <div className="p-3 border-t border-white/5 bg-slate-950/50 flex gap-2 justify-end">
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={onDiscard}
                        disabled={!hasPending}
                        className="text-slate-400 hover:text-white h-8 text-xs font-medium"
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
