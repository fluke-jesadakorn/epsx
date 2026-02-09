/**
 * Wallet Card Component
 * Premium compact wallet display for list view with action buttons
 */
'use client';

import { BarChart3, CheckCircle2, ChevronRight, Coins, Copy, CreditCard, Edit, MoreHorizontal, ShieldCheck, TrendingUp } from 'lucide-react';
import React, { useState } from 'react';

import { logger } from '@/lib/logger';
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
import { cn, copyToClipboard } from '@/lib/utils';

interface WalletCardProps {
    wallet: WalletData;
    isSelected?: boolean;
    onSelect?: (selected: boolean) => void;
    onView?: () => void;
    onManage?: () => void;
    onDisable?: () => void;
    onEnable?: () => void;
    onEdit?: () => void;
    onUpdateMetadata?: (label: string | null, note: string | null) => Promise<void>;
    className?: string;
}

const PLATFORM_ICONS: Record<Platform, React.ReactNode> = {
    analytics: <BarChart3 className="h-3.5 w-3.5" />,
    pay: <CreditCard className="h-3.5 w-3.5" />,
    token: <Coins className="h-3.5 w-3.5" />,
    markets: <TrendingUp className="h-3.5 w-3.5" />,
};

const PLATFORM_LABELS: Record<Platform, string> = {
    analytics: 'Analytics',
    pay: 'Pay',
    token: 'Token',
    markets: 'Markets',
};

const STATUS_CONFIG: Record<WalletStatus, { label: string; emoji: string; className: string; dotClass: string }> = {
    active: {
        label: 'Active',
        emoji: '🟢',
        className: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
        dotClass: 'bg-emerald-500',
    },
    disabled: {
        label: 'Disabled',
        emoji: '⚠️',
        className: 'bg-red-500/10 text-red-400 border-red-500/20',
        dotClass: 'bg-red-500',
    },
    pending: {
        label: 'Pending',
        emoji: '⏳',
        className: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
        dotClass: 'bg-amber-500',
    },
};

export function WalletCard({
    wallet,
    isSelected = false,
    onSelect,
    onView,
    onManage: _onManage,
    onDisable,
    onEnable,
    onEdit: _onEdit,
    onUpdateMetadata,
    className,
}: WalletCardProps) {
    const [copied, setCopied] = useState(false);
    const [isEditing, setIsEditing] = useState(false);
    const [labelInput, setLabelInput] = useState(wallet.label ?? '');
    const [noteInput, setNoteInput] = useState(wallet.note ?? '');
    const [isSaving, setIsSaving] = useState(false);

    const statusConfig = STATUS_CONFIG[wallet.status];
    const isDisabled = wallet.status === 'disabled';

    const handleCopy = async (e: React.MouseEvent) => {
        e.stopPropagation();
        const success = await copyToClipboard(wallet.walletAddress);
        if (success) {
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    };

    const handleStartEditing = () => {
        setLabelInput(wallet.label ?? '');
        setNoteInput(wallet.note ?? '');
        setIsEditing(true);
    };

    const handleSaveMetadata = async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (!onUpdateMetadata) { return; }

        setIsSaving(true);
        try {
            await onUpdateMetadata(
                labelInput ?? null,
                noteInput ?? null
            );
            setIsEditing(false);
        } catch (error) {
            logger.error('Failed to save metadata', { error });
        } finally {
            setIsSaving(false);
        }
    };

    return (
        <div
            className={cn(
                "group relative w-full overflow-hidden rounded-[24px] border border-white/5 bg-[#0f172a]/60 p-1 backdrop-blur-xl transition-all duration-300 hover:border-[#7645d9]/30 hover:shadow-2xl hover:shadow-[#7645d9]/10",
                isSelected && 'ring-2 ring-[#1fc7d4] bg-[#1fc7d4]/5',
                isDisabled && 'opacity-60 grayscale-[0.5]',
                className
            )}
        >
            {/* Background Gradients */}
            <div className="absolute -left-16 -top-16 h-32 w-32 rounded-full bg-[#1fc7d4]/5 blur-[50px] transition-all duration-500 group-hover:bg-[#1fc7d4]/10" />
            <div className="absolute -right-16 -bottom-16 h-32 w-32 rounded-full bg-[#7645d9]/5 blur-[50px] transition-all duration-500 group-hover:bg-[#7645d9]/10" />

            <div className="relative flex flex-col gap-6 rounded-[20px] bg-white/[0.02] p-4 sm:p-5">

                {/* 1. Identity Section */}
                <div className="flex items-center gap-4 sm:gap-5 min-w-0">
                    {/* Checkbox */}
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

                    {/* Avatar with Status */}
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

                    {/* Details */}
                    <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2 mb-1.5">
                            <span className="font-mono text-base sm:text-lg font-bold text-slate-100/90 tracking-tight truncate">
                                {wallet.walletAddress.slice(0, 6)}...{wallet.walletAddress.slice(Math.max(0, wallet.walletAddress.length - 6))}
                            </span>
                            <button
                                onClick={handleCopy}
                                className={cn(
                                    "shrink-0 rounded-lg p-1.5 transition-colors",
                                    copied ? "text-emerald-400 bg-emerald-400/10" : "text-slate-400 hover:bg-white/10 hover:text-white"
                                )}
                            >
                                {copied ? <CheckCircle2 size={14} /> : <Copy size={14} />}
                            </button>
                        </div>

                        {/* Editable Label/Note */}
                        <div className="flex items-center">
                            {isEditing ? (
                                <div className="flex flex-col gap-2 min-w-[200px] z-20 relative bg-slate-800 shadow-xl p-3 rounded-xl border border-white/10" onClick={e => e.stopPropagation()}>
                                    <Input
                                        value={labelInput}
                                        onChange={e => setLabelInput(e.target.value)}
                                        placeholder="Add a label..."
                                        className="h-8 text-xs bg-slate-900/50 border-white/10"
                                        autoFocus
                                    />
                                    <Input
                                        value={noteInput}
                                        onChange={e => setNoteInput(e.target.value)}
                                        placeholder="Add a note..."
                                        className="h-8 text-xs bg-slate-900/50 border-white/10"
                                    />
                                    <div className="flex justify-end gap-2 mt-1">
                                        <Button size="sm" variant="ghost" className="h-7 px-2 text-xs hover:bg-white/5" onClick={(e) => { e.stopPropagation(); setIsEditing(false); }}>Cancel</Button>
                                        <Button size="sm" className="h-7 px-3 text-xs bg-[#1fc7d4] hover:bg-[#1fc7d4]/90 text-white" onClick={handleSaveMetadata} disabled={isSaving}>
                                            {isSaving ? 'Saving...' : 'Save'}
                                        </Button>
                                    </div>
                                </div>
                            ) : (
                                <div
                                    className="group/edit cursor-pointer flex items-center gap-2 py-0.5 px-1.5 -ml-1.5 rounded-lg hover:bg-white/5 transition-colors"
                                    onClick={(e) => { e.stopPropagation(); handleStartEditing(); }}
                                >
                                    {wallet.label ? (
                                        <Badge variant="outline" className="font-bold bg-[#1fc7d4]/10 text-[#1fc7d4] border-[#1fc7d4]/20 max-w-[150px] truncate">
                                            {wallet.label}
                                        </Badge>
                                    ) : (
                                        <span className="text-xs font-medium text-slate-500 flex items-center gap-1 group-hover/edit:text-slate-300">
                                            <Edit size={12} /> Add Label
                                        </span>
                                    )}

                                    {wallet.note && (
                                        <span className="text-xs text-slate-400 truncate max-w-[120px]">
                                            — {wallet.note}
                                        </span>
                                    )}
                                </div>
                            )}
                        </div>
                    </div>
                </div>

                {/* 2. Stats Grid (Responsive) */}
                <div className="grid grid-cols-2 gap-4 sm:flex sm:flex-wrap sm:items-center sm:gap-6 flex-1">
                    {/* Plan */}
                    <div className="flex flex-col gap-1.5">
                        <span className="text-[10px] font-bold uppercase tracking-wider text-slate-500">Plan</span>
                        <div className={cn(
                            "flex items-center gap-2 rounded-lg border px-3 py-1.5 text-sm font-bold backdrop-blur-md w-full sm:w-auto",
                            wallet.subscriptions[0]
                                ? "border-[#7645d9]/30 bg-[#7645d9]/10 text-[#7645d9]"
                                : "border-slate-700 bg-slate-800/50 text-slate-300"
                        )}>
                            <CreditCard size={14} />
                            {wallet.subscriptions[0]?.planName ?? 'Free'}
                        </div>
                    </div>

                    {/* Platforms Breakdown */}
                    <div className="col-span-2 sm:col-span-1 flex flex-col gap-1.5 min-w-[100px]">
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

                {/* 3. Actions */}
                <div className="grid grid-cols-2 gap-3">
                    <Button
                        onClick={(e) => { e.stopPropagation(); onView?.(); }}
                        className="group/btn relative flex items-center justify-center gap-2 rounded-xl bg-white/5 px-4 py-2.5 text-xs font-bold text-white transition-all hover:bg-white/10 hover:shadow-lg hover:shadow-purple-500/10 active:scale-95 border border-white/5 hover:border-white/10 w-full"
                    >
                        <span>View Details</span>
                        <ChevronRight size={14} className="text-slate-400 transition-transform group-hover/btn:translate-x-1" />
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
                        <DropdownMenuContent align="end" className="w-56 bg-slate-900/95 backdrop-blur-xl border-white/10 text-slate-200">
                            <DropdownMenuItem onClick={(e) => { e.stopPropagation(); handleStartEditing(); }} className="focus:bg-white/10 focus:text-white cursor-pointer group">
                                <Edit className="h-4 w-4 mr-2 text-slate-500 group-hover:text-[#1fc7d4]" /> Edit Label & Note
                            </DropdownMenuItem>
                            <DropdownMenuSeparator className="bg-white/10" />
                            {isDisabled ? (
                                <DropdownMenuItem onClick={(e) => { e.stopPropagation(); onEnable?.(); }} className="focus:bg-emerald-500/10 focus:text-emerald-400 cursor-pointer text-emerald-500">
                                    <CheckCircle2 className="h-4 w-4 mr-2" /> Enable Wallet
                                </DropdownMenuItem>
                            ) : (
                                <DropdownMenuItem onClick={(e) => { e.stopPropagation(); onDisable?.(); }} className="focus:bg-red-500/10 focus:text-red-400 cursor-pointer text-red-500">
                                    <ShieldCheck className="h-4 w-4 mr-2" /> Disable Wallet
                                </DropdownMenuItem>
                            )}
                        </DropdownMenuContent>
                    </DropdownMenu>
                </div>

            </div>
        </div>
    );
}
