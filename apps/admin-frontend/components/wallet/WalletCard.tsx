/**
 * Wallet Card Component
 * Premium compact wallet display for list view with action buttons
 */
'use client';

import { BarChart3, CheckCircle2, Coins, Copy, CreditCard, Edit, Eye, MoreHorizontal, TrendingUp } from 'lucide-react';
import React, { useState } from 'react';

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
        className: 'bg-success/10 text-success border-success/20',
        dotClass: 'bg-success',
    },
    disabled: {
        label: 'Disabled',
        emoji: '⚠️',
        className: 'bg-warning/10 text-warning border-warning/20',
        dotClass: 'bg-warning',
    },
    pending: {
        label: 'Pending',
        emoji: '⏳',
        className: 'bg-info/10 text-info border-info/20',
        dotClass: 'bg-info',
    },
};

// Generate a deterministic gradient based on wallet address
function getAvatarGradient(address: string): string {
    const hash = address.slice(2, 10);
    const hue1 = parseInt(hash.slice(0, 4), 16) % 360;
    const hue2 = (hue1 + 40) % 360;
    return `linear-gradient(135deg, hsl(${hue1}, 70%, 60%) 0%, hsl(${hue2}, 80%, 50%) 100%)`;
}

/**
 * @param root0
 * @param root0.wallet
 * @param root0.isSelected
 * @param root0.onSelect
 * @param root0.onView
 * @param root0.onManage
 * @param root0.onDisable
 * @param root0.onEnable
 * @param root0.onEdit
 * @param root0.onUpdateMetadata
 * @param root0.className
 */
export function WalletCard({
    wallet,
    isSelected = false,
    onSelect,
    onView,
    onManage,
    onDisable,
    onEnable,
    onEdit,
    onUpdateMetadata,
    className,
}: WalletCardProps) {
    const [copied, setCopied] = useState(false);
    const [isEditing, setIsEditing] = useState(false);
    const [labelInput, setLabelInput] = useState(wallet.label || '');
    const [noteInput, setNoteInput] = useState(wallet.note || '');
    const [isSaving, setIsSaving] = useState(false);

    const statusConfig = STATUS_CONFIG[wallet.status];
    const isDisabled = wallet.status === 'disabled';
    const activePermissions = wallet.permissions.filter(p => p.isActive).length;

    const handleCopy = async (e: React.MouseEvent) => {
        e.stopPropagation();
        const success = await copyToClipboard(wallet.walletAddress);
        if (success) {
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    };

    const handleStartEditing = () => {
        setLabelInput(wallet.label || '');
        setNoteInput(wallet.note || '');
        setIsEditing(true);
    };

    const handleSaveMetadata = async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (!onUpdateMetadata) { return; }

        setIsSaving(true);
        try {
            await onUpdateMetadata(
                labelInput || null,
                noteInput || null
            );
            setIsEditing(false);
        } catch (error) {
            console.error('Failed to save metadata', error);
        } finally {
            setIsSaving(false);
        }
    };

    return (
        <div
            className={cn(
                // Base structure
                'group relative flex flex-col lg:flex-row lg:items-center overflow-hidden',
                // Styling
                'bg-card hover:bg-muted/30 border border-border/60 hover:border-border',
                'rounded-xl transition-all duration-300',
                // Selection
                isSelected && 'ring-1 ring-primary border-primary/50 bg-primary/5',
                // Disabled opactiy
                isDisabled && 'opacity-70',
                className
            )}
        >
            {/* Left Border Status Indicator */}
            <div className={cn(
                "absolute left-0 top-0 bottom-0 w-1 transition-colors duration-300",
                statusConfig.dotClass
            )} />

            {/* Content Container */}
            <div className="flex flex-1 flex-col lg:flex-row lg:items-center gap-4 p-4 pl-5">

                {/* 1. Identity & Avatar Section */}
                <div className="flex items-center gap-4 min-w-[280px]">
                    {/* Checkbox */}
                    {onSelect && (
                        <div className="flex items-center">
                            <input
                                type="checkbox"
                                checked={isSelected}
                                onChange={(e) => onSelect(e.target.checked)}
                                className="h-4 w-4 rounded border-input text-primary focus:ring-primary/20 accent-primary cursor-pointer"
                            />
                        </div>
                    )}

                    {/* Avatar */}
                    <div className="relative shrink-0">
                        <div
                            className="flex h-10 w-10 items-center justify-center rounded-lg text-xs font-bold text-white shadow-md transition-transform group-hover:scale-105"
                            style={{ background: getAvatarGradient(wallet.walletAddress) }}
                        >
                            {wallet.walletAddress.slice(2, 4).toUpperCase()}
                        </div>
                        {/* Status Dot (Mobile only mainly, or extra indicator) */}
                        <div className={cn(
                            "absolute -bottom-1 -right-1 h-3 w-3 rounded-full border-2 border-card",
                            statusConfig.dotClass
                        )} />
                    </div>

                    {/* Address & Copy */}
                    <div className="flex flex-col min-w-0">
                        <div className="flex items-center gap-1.5">
                            <span className="font-mono text-sm font-medium text-foreground truncate">
                                {wallet.walletAddress}
                            </span>
                            <button
                                onClick={handleCopy}
                                className={cn(
                                    "p-1 rounded-md transition-colors hover:bg-muted text-muted-foreground",
                                    copied && "text-green-500 bg-green-500/10"
                                )}
                            >
                                {copied ? <CheckCircle2 className="h-3 w-3" /> : <Copy className="h-3 w-3" />}
                            </button>
                        </div>
                        {/* Status Label (Mobile) */}
                        <div className="lg:hidden flex items-center gap-2 mt-1">
                            <Badge variant="outline" className={cn("text-[10px] h-5 px-1.5", statusConfig.className)}>
                                {statusConfig.label}
                            </Badge>
                        </div>
                    </div>
                </div>

                {/* 2. Metadata Columns (Desktop Grid / Mobile Stack) */}
                <div className="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-4 lg:gap-6 items-start lg:items-center">

                    {/* Plan & Group (Span 5) */}
                    <div className="lg:col-span-5 flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-muted-foreground">
                        <div className="flex items-center gap-2 min-w-[100px]">
                            <span className="text-xs text-muted-foreground/50 uppercase tracking-wider font-semibold">Plan</span>
                            <span className={cn("font-medium text-foreground", !wallet.subscriptions[0] && "text-muted-foreground")}>
                                {wallet.subscriptions[0]?.planName || 'Free'}
                            </span>
                        </div>

                        <div className="hidden lg:block w-px h-8 bg-border/40" />

                        <div className="flex items-center gap-2">
                            <span className="text-xs text-muted-foreground/50 uppercase tracking-wider font-semibold">Group</span>
                            <span className="font-medium text-foreground">
                                {wallet.plans?.[0]?.planName || 'User'}
                                {(wallet.plans?.length || 0) > 1 && (
                                    <span className="ml-1 text-[10px] bg-muted px-1.5 py-0.5 rounded-full">+{wallet.plans!.length - 1}</span>
                                )}
                            </span>
                        </div>

                        <div className="hidden lg:block w-px h-8 bg-border/40" />

                        <div className="flex items-center gap-2">
                            <Badge variant="secondary" className="h-6 font-normal">
                                {activePermissions} Perms
                            </Badge>
                        </div>
                    </div>

                    {/* Platform Access (Span 4) */}
                    <div className="lg:col-span-4 flex items-center gap-2">
                        <div className="flex gap-1">
                            {wallet.platforms.length > 0 ? wallet.platforms.map(p => (
                                <div key={p} className="p-1.5 bg-muted/50 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted transition-colors" title={PLATFORM_LABELS[p]}>
                                    {PLATFORM_ICONS[p]}
                                </div>
                            )) : (
                                <span className="text-xs text-muted-foreground italic">No access</span>
                            )}
                        </div>
                    </div>

                    {/* Labels/Notes (Span 3) */}
                    <div className="lg:col-span-3">
                        {isEditing ? (
                            <div className="flex flex-col gap-2 min-w-[200px] z-10 relative bg-card shadow-lg p-2 rounded-lg border border-border" onClick={e => e.stopPropagation()}>
                                <Input
                                    value={labelInput}
                                    onChange={e => setLabelInput(e.target.value)}
                                    placeholder="Label"
                                    className="h-7 text-xs"
                                    autoFocus
                                />
                                <div className="flex justify-end gap-1">
                                    <Button size="sm" variant="ghost" className="h-6 px-2 text-[10px]" onClick={(e) => { e.stopPropagation(); setIsEditing(false); }}>Cancel</Button>
                                    <Button size="sm" className="h-6 px-2 text-[10px]" onClick={handleSaveMetadata}>Save</Button>
                                </div>
                            </div>
                        ) : (
                            <div
                                className="group/edit cursor-pointer py-1 px-2 -ml-2 rounded-md hover:bg-muted/50 transition-colors"
                                onClick={(e) => { e.stopPropagation(); handleStartEditing(); }}
                            >
                                {wallet.label ? (
                                    <div className="flex items-center gap-2">
                                        <Badge variant="outline" className="font-normal bg-blue-500/5 text-blue-600 border-blue-200 dark:border-blue-900/30 truncate max-w-[120px]">
                                            {wallet.label}
                                        </Badge>
                                    </div>
                                ) : wallet.note ? (
                                    <div className="flex items-center gap-2 text-xs text-muted-foreground truncate max-w-[150px]">
                                        <span className="truncate">📝 {wallet.note}</span>
                                    </div>
                                ) : (
                                    <div className="flex items-center gap-1 text-xs text-muted-foreground/30 opacity-0 group-hover/group:opacity-100 transition-opacity">
                                        <Edit className="h-3 w-3" />
                                        <span>Add note</span>
                                    </div>
                                )}
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* 3. Action Buttons (Right Aligned on Desktop) */}
            <div className="flex lg:flex-col items-center justify-between lg:justify-center gap-2 border-t lg:border-t-0 lg:border-l border-border/40 bg-muted/5 p-3 lg:w-[140px]">
                <Button
                    variant="outline"
                    size="sm"
                    className="flex-1 lg:w-full h-8 text-xs font-medium hover:bg-primary/5 hover:text-primary hover:border-primary/30"
                    onClick={onView}
                >
                    <Eye className="h-3.5 w-3.5 mr-1.5" />
                    View
                </Button>

                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="sm" className="h-8 w-8 lg:w-full lg:px-2 lg:h-8 hover:bg-muted">
                            <MoreHorizontal className="h-4 w-4 lg:mr-1.5" />
                            <span className="hidden lg:inline text-xs">More</span>
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                        <DropdownMenuItem onClick={onUpdateMetadata ? handleStartEditing : onEdit}>
                            <Edit className="h-4 w-4 mr-2" /> Edit Note
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        {isDisabled ? (
                            <DropdownMenuItem onClick={onEnable}>
                                <span className="mr-2">🔓</span> Enable Wallet
                            </DropdownMenuItem>
                        ) : (
                            <DropdownMenuItem onClick={onDisable}>
                                <span className="mr-2 text-red-500">🚫</span> Disable Wallet
                            </DropdownMenuItem>
                        )}
                    </DropdownMenuContent>
                </DropdownMenu>
            </div>

        </div>
    );
}



