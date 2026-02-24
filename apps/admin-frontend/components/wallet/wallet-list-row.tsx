'use client';

import { CheckCircle2, Clock, Copy, CreditCard, Edit, MoreHorizontal, ShieldCheck, User } from 'lucide-react';
import { useRouter } from 'next/navigation';
import React, { useState } from 'react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { cn, copyToClipboard } from '@/lib/utils';
import { formatRelativeTime } from '@/shared/utils';
import type { WalletData } from './types';
import { getPlanDisplay, PLATFORM_ICONS, PLATFORM_LABELS, STATUS_CONFIG } from './wallet-card-sections';

interface WalletListRowProps {
    wallet: WalletData;
    onView?: () => void;
    onEnable?: () => void;
}

export function WalletListRow({ wallet, onView, onEnable }: WalletListRowProps) {
    const [copied, setCopied] = useState(false);
    const router = useRouter();
    const isDisabled = wallet.status === 'disabled';
    const statusConfig = STATUS_CONFIG[wallet.status];
    const plan = getPlanDisplay(wallet);

    const handleCopy = async (e: React.MouseEvent) => {
        e.stopPropagation();
        const success = await copyToClipboard(wallet.walletAddress);
        if (success) {
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    };

    return (
        <div
            className={cn(
                'group flex items-center gap-3 px-4 py-3 border-b border-border/40 hover:bg-muted/20 transition-colors cursor-pointer',
                isDisabled && 'opacity-60'
            )}
            onClick={onView}
        >
            {/* Avatar + Address */}
            <div className="flex items-center gap-3 min-w-0 flex-1">
                <div className="relative shrink-0">
                    <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-br from-[#1fc7d4] to-[#7645d9] text-xs font-black text-white">
                        {wallet.walletAddress.slice(2, 4).toUpperCase()}
                    </div>
                    <div className={cn('absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full border-2 border-background', statusConfig.dotClass)} />
                </div>
                <div className="min-w-0">
                    <div className="flex items-center gap-1.5">
                        <span className="font-mono text-sm font-semibold truncate">
                            {wallet.walletAddress.slice(0, 6)}...{wallet.walletAddress.slice(-4)}
                        </span>
                        <button
                            onClick={(e) => void handleCopy(e)}
                            className={cn(
                                'shrink-0 rounded p-0.5 transition-colors',
                                copied ? 'text-emerald-400' : 'text-muted-foreground hover:text-foreground'
                            )}
                        >
                            {copied ? <CheckCircle2 size={11} /> : <Copy size={11} />}
                        </button>
                    </div>
                    {wallet.label !== undefined && wallet.label !== '' ? (
                        <Badge variant="outline" className="text-[10px] px-1.5 py-0 h-4 bg-[#1fc7d4]/10 text-[#1fc7d4] border-[#1fc7d4]/20 mt-0.5">
                            {wallet.label}
                        </Badge>
                    ) : wallet.note !== undefined && wallet.note !== '' ? (
                        <span className="text-[10px] text-muted-foreground truncate block mt-0.5 max-w-[140px]">{wallet.note}</span>
                    ) : null}
                </div>
            </div>

            {/* Plan */}
            <div className="hidden sm:flex items-center gap-1.5 w-32 shrink-0">
                <CreditCard size={12} className="text-muted-foreground shrink-0" />
                <span className="text-sm truncate">{plan.name}</span>
            </div>

            {/* Platforms */}
            <div className="hidden md:flex items-center gap-1 w-24 shrink-0">
                {wallet.platforms.length > 0 ? wallet.platforms.map(p => (
                    <div key={p} title={PLATFORM_LABELS[p]} className="p-1 rounded text-muted-foreground border border-border/40 hover:text-foreground transition-colors">
                        {PLATFORM_ICONS[p]}
                    </div>
                )) : <span className="text-xs text-muted-foreground">—</span>}
            </div>

            {/* Status */}
            <div className="hidden sm:flex shrink-0 w-20">
                <Badge variant="outline" className={cn('text-xs', statusConfig.className)}>
                    {statusConfig.label}
                </Badge>
            </div>

            {/* Joined */}
            <div className="hidden lg:flex items-center gap-1.5 w-24 shrink-0 text-xs text-muted-foreground">
                <User size={11} className="shrink-0" />
                <span className="truncate">{formatRelativeTime(wallet.createdAt)}</span>
            </div>

            {/* Last Login */}
            <div className="hidden xl:flex items-center gap-1.5 w-24 shrink-0 text-xs text-muted-foreground">
                <Clock size={11} className="shrink-0" />
                <span className="truncate">{wallet.lastAuthAt !== undefined && wallet.lastAuthAt !== '' ? formatRelativeTime(wallet.lastAuthAt) : 'Never'}</span>
            </div>

            {/* Actions */}
            <div className="flex items-center gap-1 shrink-0" onClick={e => e.stopPropagation()}>
                <Button size="sm" variant="ghost" className="h-7 px-2 text-xs gap-1" onClick={onView}>
                    <Edit size={12} /> <span className="hidden sm:inline">Edit</span>
                </Button>
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="sm" className="h-7 w-7 p-0">
                            <MoreHorizontal size={14} />
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end" className="w-44">
                        <DropdownMenuItem onClick={(e) => { void handleCopy(e as unknown as React.MouseEvent); }}>
                            <Copy className="h-3.5 w-3.5 mr-2" /> Copy Address
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        {isDisabled ? (
                            <DropdownMenuItem onClick={() => onEnable?.()} className="text-emerald-500 focus:text-emerald-400">
                                <CheckCircle2 className="h-3.5 w-3.5 mr-2" /> Enable Wallet
                            </DropdownMenuItem>
                        ) : (
                            <DropdownMenuItem
                                onClick={() => router.push(`/wallet-management/wallets/${encodeURIComponent(wallet.walletAddress)}/disable`)}
                                className="text-red-500 focus:text-red-400"
                            >
                                <ShieldCheck className="h-3.5 w-3.5 mr-2" /> Disable Wallet
                            </DropdownMenuItem>
                        )}
                    </DropdownMenuContent>
                </DropdownMenu>
            </div>
        </div>
    );
}
