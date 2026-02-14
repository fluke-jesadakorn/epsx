'use client';

import { BarChart3, CheckCircle2, Clock, Coins, Copy, CreditCard, Edit, MoreHorizontal, ShieldCheck, TrendingUp, User } from 'lucide-react';
import { useRouter } from 'next/navigation';
import React from 'react';

import type { Platform, WalletData, WalletStatus } from './types';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Input } from '@/components/ui/input';
import { cn } from '@/lib/utils';
import { formatRelativeTime, formatTimeRemaining, isFutureDate } from '@/shared/utils';

// --- Constants ---

export const PLATFORM_ICONS: Record<Platform, React.ReactNode> = {
    analytics: <BarChart3 className="h-3.5 w-3.5" />,
    pay: <CreditCard className="h-3.5 w-3.5" />,
    token: <Coins className="h-3.5 w-3.5" />,
    markets: <TrendingUp className="h-3.5 w-3.5" />,
};

export const PLATFORM_LABELS: Record<Platform, string> = {
    analytics: 'Analytics',
    pay: 'Pay',
    token: 'Token',
    markets: 'Markets',
};

export const STATUS_CONFIG: Record<WalletStatus, { label: string; className: string; dotClass: string }> = {
    active: {
        label: 'Active',
        className: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
        dotClass: 'bg-emerald-500',
    },
    disabled: {
        label: 'Disabled',
        className: 'bg-red-500/10 text-red-400 border-red-500/20',
        dotClass: 'bg-red-500',
    },
    pending: {
        label: 'Pending',
        className: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
        dotClass: 'bg-amber-500',
    },
};

// --- Plan Display Helper ---

interface PlanDisplay {
    name: string;
    status?: 'active' | 'cancelled' | 'expired' | 'paused';
    expiresAt?: string;
}

export function getPlanDisplay(wallet: WalletData): PlanDisplay {
    const activeSub = wallet.subscriptions.find(s => s.status === 'active');
    if (activeSub) {
        return { name: activeSub.planName, status: 'active', expiresAt: activeSub.expiresAt };
    }

    const anySub = wallet.subscriptions[0];
    if (anySub) {
        return { name: anySub.planName, status: anySub.status, expiresAt: anySub.expiresAt };
    }

    const planGroup = wallet.plans[0];
    if (planGroup) {
        return { name: planGroup.planName };
    }

    return { name: 'Free' };
}

// --- Sub-Components ---

interface IdentityProps {
    wallet: WalletData;
    isSelected: boolean;
    onSelect?: (selected: boolean) => void;
    copied: boolean;
    onCopy: (e: React.MouseEvent) => Promise<void>;
    isEditing: boolean;
    labelInput: string;
    noteInput: string;
    onLabelChange: (v: string) => void;
    onNoteChange: (v: string) => void;
    onStartEditing: () => void;
    onCancelEditing: () => void;
    onSave: (e: React.MouseEvent) => Promise<void>;
    isSaving: boolean;
}

export function WalletCardIdentity({
    wallet, isSelected, onSelect, copied, onCopy,
    isEditing, labelInput, noteInput, onLabelChange, onNoteChange,
    onStartEditing, onCancelEditing, onSave, isSaving,
}: IdentityProps) {
    const statusConfig = STATUS_CONFIG[wallet.status];

    return (
        <div className="flex items-center gap-4 sm:gap-5 min-w-0">
            {onSelect && (
                <div className="flex items-center shrink-0">
                    <input
                        type="checkbox"
                        checked={isSelected}
                        onChange={(e) => onSelect(e.target.checked)}
                        className="h-5 w-5 rounded border-white/20 bg-white/5 text-[#1fc7d4] focus:ring-[#1fc7d4]/20 focus:ring-offset-0 accent-[#1fc7d4] cursor-pointer"
                    />
                </div>
            )}

            <div className="relative shrink-0">
                <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-[#1fc7d4] to-[#7645d9] blur-md opacity-20 group-hover:opacity-40 transition-opacity" />
                <div className="relative flex h-14 w-14 sm:h-16 sm:w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-[#1fc7d4] to-[#7645d9] text-lg sm:text-xl font-black text-white shadow-inner shadow-white/20">
                    {wallet.walletAddress.slice(2, 4).toUpperCase()}
                </div>
                <div className={cn(
                    "absolute -bottom-1 -right-1 h-4 w-4 rounded-full border-[3px] border-[#0f172a]",
                    statusConfig.dotClass
                )} />
            </div>

            <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2 mb-1.5">
                    <span className="font-mono text-base sm:text-lg font-bold text-slate-100/90 tracking-tight truncate">
                        {wallet.walletAddress.slice(0, 6)}...{wallet.walletAddress.slice(-6)}
                    </span>
                    <button
                        onClick={(e) => void onCopy(e)}
                        className={cn(
                            "shrink-0 rounded-lg p-1.5 transition-colors",
                            copied ? "text-emerald-400 bg-emerald-400/10" : "text-slate-400 hover:bg-white/10 hover:text-white"
                        )}
                    >
                        {copied ? <CheckCircle2 size={14} /> : <Copy size={14} />}
                    </button>
                </div>

                <div className="flex items-center">
                    {isEditing ? (
                        <div className="flex flex-col gap-2 min-w-[200px] z-20 relative bg-slate-800 shadow-xl p-3 rounded-xl border border-white/10" onClick={e => e.stopPropagation()}>
                            <Input value={labelInput} onChange={e => onLabelChange(e.target.value)} placeholder="Add a label..." className="h-8 text-xs bg-slate-900/50 border-white/10" autoFocus />
                            <Input value={noteInput} onChange={e => onNoteChange(e.target.value)} placeholder="Add a note..." className="h-8 text-xs bg-slate-900/50 border-white/10" />
                            <div className="flex justify-end gap-2 mt-1">
                                <Button size="sm" variant="ghost" className="h-7 px-2 text-xs hover:bg-white/5" onClick={(e) => { e.stopPropagation(); onCancelEditing(); }}>Cancel</Button>
                                <Button size="sm" className="h-7 px-3 text-xs bg-[#1fc7d4] hover:bg-[#1fc7d4]/90 text-white" onClick={(e) => void onSave(e)} disabled={isSaving}>
                                    {isSaving ? 'Saving...' : 'Save'}
                                </Button>
                            </div>
                        </div>
                    ) : (
                        <div
                            className="group/edit cursor-pointer flex items-center gap-2 py-0.5 px-1.5 -ml-1.5 rounded-lg hover:bg-white/5 transition-colors"
                            onClick={(e) => { e.stopPropagation(); onStartEditing(); }}
                        >
                            {wallet.label !== undefined && wallet.label !== '' ? (
                                <Badge variant="outline" className="font-bold bg-[#1fc7d4]/10 text-[#1fc7d4] border-[#1fc7d4]/20 max-w-[150px] truncate">
                                    {wallet.label}
                                </Badge>
                            ) : (
                                <span className="text-xs font-medium text-slate-500 flex items-center gap-1 group-hover/edit:text-slate-300">
                                    <Edit size={12} /> Add Label
                                </span>
                            )}
                            {wallet.note !== undefined && wallet.note !== '' ? (
                                <span className="text-xs text-slate-400 truncate max-w-[120px]">
                                    — {wallet.note}
                                </span>
                            ) : null}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

// --- Stats Section ---

const SUB_STATUS_STYLES: Record<string, string> = {
    active: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
    expired: 'bg-red-500/10 text-red-400 border-red-500/20',
    cancelled: 'bg-slate-500/10 text-slate-400 border-slate-500/20',
    paused: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
};

export function WalletCardStats({ wallet }: { wallet: WalletData }) {
    const plan = getPlanDisplay(wallet);
    const hasPlan = plan.name !== 'Free';

    return (
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
            {/* Plan */}
            <div className="flex flex-col gap-1.5">
                <span className="text-[10px] font-bold uppercase tracking-wider text-slate-500">Plan</span>
                <div className={cn(
                    "flex items-center gap-2 rounded-lg border px-3 py-1.5 text-sm font-bold backdrop-blur-md",
                    hasPlan
                        ? "border-[#7645d9]/30 bg-[#7645d9]/10 text-[#7645d9]"
                        : "border-slate-700 bg-slate-800/50 text-slate-300"
                )}>
                    <CreditCard size={14} />
                    <span className="truncate">{plan.name}</span>
                    {plan.status && plan.status !== 'active' ? (
                        <Badge variant="outline" className={cn("text-[9px] px-1.5 py-0 ml-auto", SUB_STATUS_STYLES[plan.status])}>
                            {plan.status}
                        </Badge>
                    ) : null}
                </div>
                {plan.expiresAt !== undefined && plan.expiresAt !== '' && plan.status === 'active' && isFutureDate(plan.expiresAt) ? (
                    <span className="text-[10px] text-slate-500">{formatTimeRemaining(plan.expiresAt)} left</span>
                ) : null}
            </div>

            {/* Joined */}
            <div className="flex flex-col gap-1.5">
                <span className="text-[10px] font-bold uppercase tracking-wider text-slate-500">Joined</span>
                <div className="flex items-center gap-2 text-sm text-slate-300" title={wallet.createdAt}>
                    <User size={14} className="text-slate-500 shrink-0" />
                    <span className="truncate">{formatRelativeTime(wallet.createdAt)}</span>
                </div>
            </div>

            {/* Last Login */}
            <div className="flex flex-col gap-1.5">
                <span className="text-[10px] font-bold uppercase tracking-wider text-slate-500">Last Login</span>
                <div className="flex items-center gap-2 text-sm text-slate-300" title={wallet.lastAuthAt ?? undefined}>
                    <Clock size={14} className="text-slate-500 shrink-0" />
                    <span className="truncate">{wallet.lastAuthAt !== undefined && wallet.lastAuthAt !== '' ? formatRelativeTime(wallet.lastAuthAt) : 'Never'}</span>
                </div>
            </div>

            {/* Platforms */}
            <div className="flex flex-col gap-1.5">
                <span className="text-[10px] font-bold uppercase tracking-wider text-slate-500">Platforms</span>
                <div className="flex gap-1.5">
                    {wallet.platforms.length > 0 ? wallet.platforms.map(p => (
                        <div key={p} className="p-1.5 bg-white/5 rounded-md text-slate-400 hover:text-white hover:bg-white/10 transition-colors border border-white/5" title={PLATFORM_LABELS[p]}>
                            {PLATFORM_ICONS[p]}
                        </div>
                    )) : (
                        <span className="text-xs text-slate-500 italic py-1">None</span>
                    )}
                </div>
            </div>
        </div>
    );
}

// --- Actions Section ---

interface ActionsProps {
    wallet: WalletData;
    onView?: () => void;
    onEnable?: () => void;
    onCopy: (e: React.MouseEvent) => Promise<void>;
}

export function WalletCardActions({ wallet, onView, onEnable, onCopy }: ActionsProps) {
    const router = useRouter();
    const isDisabled = wallet.status === 'disabled';

    return (
        <div className="grid grid-cols-2 gap-3">
            <Button
                onClick={(e) => { e.stopPropagation(); onView?.(); }}
                className="group/btn relative flex items-center justify-center gap-2 rounded-xl bg-white/5 px-4 py-2.5 text-xs font-bold text-white transition-all hover:bg-white/10 hover:shadow-lg hover:shadow-purple-500/10 active:scale-95 border border-white/5 hover:border-white/10 w-full"
            >
                <Edit size={14} className="text-slate-400 group-hover/btn:text-[#1fc7d4] transition-colors" />
                <span>Edit</span>
            </Button>

            <DropdownMenu>
                <DropdownMenuTrigger asChild>
                    <Button
                        variant="ghost"
                        className="flex items-center justify-center gap-2 h-10 w-full rounded-xl border border-transparent text-slate-400 transition-all hover:bg-white/5 hover:text-white hover:border-white/5 active:scale-95 text-xs font-bold"
                    >
                        <MoreHorizontal size={16} />
                        <span className="truncate">More Actions</span>
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-48 bg-slate-900/95 backdrop-blur-xl border-white/10 text-slate-200">
                    <DropdownMenuItem onClick={(e) => { e.stopPropagation(); void onCopy(e as unknown as React.MouseEvent); }} className="focus:bg-white/10 focus:text-white cursor-pointer group">
                        <Copy className="h-4 w-4 mr-2 text-slate-500 group-hover:text-[#1fc7d4]" /> Copy Address
                    </DropdownMenuItem>
                    <DropdownMenuSeparator className="bg-white/10" />
                    {isDisabled ? (
                        <DropdownMenuItem onClick={(e) => { e.stopPropagation(); onEnable?.(); }} className="focus:bg-emerald-500/10 focus:text-emerald-400 cursor-pointer text-emerald-500">
                            <CheckCircle2 className="h-4 w-4 mr-2" /> Enable Wallet
                        </DropdownMenuItem>
                    ) : (
                        <DropdownMenuItem onClick={(e) => { e.stopPropagation(); router.push(`/wallet-management/wallets/${encodeURIComponent(wallet.walletAddress)}/disable`); }} className="focus:bg-red-500/10 focus:text-red-400 cursor-pointer text-red-500">
                            <ShieldCheck className="h-4 w-4 mr-2" /> Disable Wallet
                        </DropdownMenuItem>
                    )}
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    );
}
