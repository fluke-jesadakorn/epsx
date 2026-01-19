/**
 * Wallet Card Component
 * Premium compact wallet display for list view with action buttons
 */
'use client';

import { BarChart3, Coins, Copy, CreditCard, Edit, Eye, Loader2, MoreHorizontal, Save, Settings, Sparkles, TrendingUp } from 'lucide-react';
import React, { useState } from 'react';

import type { Platform, WalletData, WalletStatus } from './types';
import { WalletLabelBadge } from './WalletLabelBadge';

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
import { Textarea } from '@/components/ui/textarea';
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
                // Base card styles with premium glassmorphism
                'group relative rounded-2xl overflow-hidden text-card-foreground',
                'bg-card',
                'border border-border/60',
                // Smooth transitions
                'transition-all duration-300 ease-out',
                // Hover effects
                'hover:shadow-xl hover:shadow-primary/5',
                'hover:border-border',
                'hover:scale-[1.01] hover:-translate-y-0.5',
                // Disabled state
                isDisabled && 'opacity-80',
                // Selected state
                isSelected && 'ring-2 ring-primary ring-offset-2 dark:ring-offset-background border-primary/50',
                className
            )}
        >
            {/* Subtle gradient overlay on hover */}
            <div className="absolute inset-0 bg-gradient-to-br from-blue-500/0 via-purple-500/0 to-pink-500/0 group-hover:from-blue-500/3 group-hover:via-purple-500/3 group-hover:to-pink-500/3 transition-all duration-500 pointer-events-none" />

            {/* Shine effect on hover */}
            <div className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none overflow-hidden">
                <div className="absolute -inset-full top-0 w-1/2 h-full bg-gradient-to-r from-transparent via-white/10 to-transparent skew-x-12 group-hover:animate-[shimmer_2s_ease-in-out_infinite]" />
            </div>

            <div className="relative flex flex-col p-5 gap-4">
                {/* Header: Checkbox + Avatar + Address + Status */}
                <div className="flex items-center gap-4">
                    {/* Checkbox */}
                    {onSelect && (
                        <div className="flex items-center">
                            <input
                                type="checkbox"
                                checked={isSelected}
                                onChange={(e) => onSelect(e.target.checked)}
                                className="h-4 w-4 rounded-md border-border text-primary focus:ring-primary focus:ring-offset-0 transition-colors cursor-pointer dark:bg-background"
                            />
                        </div>
                    )}

                    {/* Premium Avatar */}
                    <div className="relative group/avatar">
                        <div
                            className={cn(
                                'relative flex h-12 w-12 shrink-0 items-center justify-center rounded-xl text-sm font-bold shadow-lg',
                                'text-white transition-all duration-300',
                                'group-hover/avatar:scale-105 group-hover/avatar:shadow-xl',
                            )}
                            style={{ background: getAvatarGradient(wallet.walletAddress) }}
                        >
                            {wallet.walletAddress.slice(2, 4).toUpperCase()}

                            {/* Animated status indicator */}
                            <div className={cn(
                                'absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full border-2 border-white dark:border-gray-900',
                                statusConfig.dotClass,
                                wallet.status === 'active' && 'animate-pulse',
                            )}>
                                <span className={cn(
                                    'absolute inset-0 rounded-full',
                                    statusConfig.dotClass,
                                    'animate-ping opacity-40'
                                )} />
                            </div>
                        </div>
                    </div>

                    {/* Address & Copy */}
                    <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                            <span className="font-mono text-sm font-semibold text-gray-900 dark:text-gray-100 truncate">
                                {wallet.walletAddress}
                            </span>
                            <button
                                onClick={handleCopy}
                                className={cn(
                                    'shrink-0 p-1.5 rounded-lg transition-all duration-200',
                                    copied
                                        ? 'bg-success/20 text-success scale-110'
                                        : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground hover:scale-110'
                                )}
                                title="Copy address"
                            >
                                <Copy className="h-3.5 w-3.5" />
                            </button>
                        </div>
                    </div>

                    {/* Status Badge */}
                    <div className="hidden sm:block">
                        <Badge className={cn(
                            'text-xs px-3 py-1 font-semibold border rounded-full',
                            'transition-all duration-200 hover:scale-105',
                            statusConfig.className
                        )}>
                            {statusConfig.label}
                        </Badge>
                    </div>

                    {/* Actions Menu */}
                    <div className="flex items-center gap-1.5">
                        <Button
                            variant="ghost"
                            size="sm"
                            onClick={onView}
                            className={cn(
                                'hidden sm:flex h-9 px-4 gap-2 text-sm font-medium rounded-xl',
                                'bg-primary/10 text-primary',
                                'hover:bg-primary/20 hover:text-primary',
                                'transition-all duration-200 hover:scale-105 hover:shadow-md'
                            )}
                        >
                            <Eye className="h-4 w-4" />
                            View
                        </Button>

                        <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    className="h-9 w-9 p-0 rounded-xl hover:bg-gray-100 dark:hover:bg-gray-800 transition-all duration-200 hover:scale-110"
                                >
                                    <MoreHorizontal className="h-4 w-4" />
                                </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end" className="w-48 rounded-xl p-1">
                                <DropdownMenuItem
                                    onClick={onUpdateMetadata ? handleStartEditing : onEdit}
                                    className="rounded-lg"
                                >
                                    <Edit className="h-4 w-4 mr-2" />
                                    <span className="text-sm">Edit Label/Note</span>
                                </DropdownMenuItem>
                                <DropdownMenuItem onClick={onManage} className="rounded-lg">
                                    <Settings className="h-4 w-4 mr-2" />
                                    <span className="text-sm">Manage</span>
                                </DropdownMenuItem>
                                <DropdownMenuSeparator />
                                {isDisabled ? (
                                    <DropdownMenuItem
                                        onClick={onEnable}
                                        className="text-emerald-700 dark:text-emerald-400 rounded-lg"
                                    >
                                        <span className="mr-2">🔓</span>
                                        <span className="text-sm">Re-enable</span>
                                    </DropdownMenuItem>
                                ) : (
                                    <DropdownMenuItem
                                        onClick={onDisable}
                                        className="text-amber-700 dark:text-amber-400 rounded-lg"
                                    >
                                        <span className="mr-2">⚠️</span>
                                        <span className="text-sm">Disable</span>
                                    </DropdownMenuItem>
                                )}
                            </DropdownMenuContent>
                        </DropdownMenu>
                    </div>
                </div>

                {/* Key Metrics Grid - Enhanced */}
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                    {/* Plan */}
                    <div className="flex flex-col p-3 rounded-xl bg-gradient-to-br from-gray-50 to-gray-100/50 dark:from-gray-800/50 dark:to-gray-900/50 border border-gray-100 dark:border-gray-800 transition-all duration-200 hover:border-gray-200 dark:hover:border-gray-700">
                        <span className="text-[10px] uppercase text-gray-500 font-semibold tracking-wider mb-1">Plan</span>
                        <span className="text-sm font-bold text-gray-900 dark:text-gray-100">
                            {wallet.subscriptions[0]?.planName || 'Free'}
                        </span>
                    </div>

                    {/* Group */}
                    <div className="flex flex-col p-3 rounded-xl bg-gradient-to-br from-gray-50 to-gray-100/50 dark:from-gray-800/50 dark:to-gray-900/50 border border-gray-100 dark:border-gray-800 transition-all duration-200 hover:border-gray-200 dark:hover:border-gray-700">
                        <span className="text-[10px] uppercase text-gray-500 font-semibold tracking-wider mb-1">Group</span>
                        <div className="flex items-center gap-1.5">
                            <span className="text-sm font-bold text-gray-900 dark:text-gray-100">
                                {wallet.groups?.[0]?.groupName || 'User'}
                            </span>
                            {(wallet.groups?.length || 0) > 1 && (
                                <span className="text-xs text-gray-500 bg-gray-200 dark:bg-gray-700 px-1.5 py-0.5 rounded-full">+{wallet.groups!.length - 1}</span>
                            )}
                        </div>
                    </div>

                    {/* Permissions */}
                    <div className="flex flex-col p-3 rounded-xl bg-gradient-to-br from-gray-50 to-gray-100/50 dark:from-gray-800/50 dark:to-gray-900/50 border border-gray-100 dark:border-gray-800 transition-all duration-200 hover:border-gray-200 dark:hover:border-gray-700">
                        <span className="text-[10px] uppercase text-gray-500 font-semibold tracking-wider mb-1">Perms</span>
                        <span className="text-sm font-bold text-gray-900 dark:text-gray-100">
                            {activePermissions} <span className="text-gray-500 font-normal text-xs">active</span>
                        </span>
                    </div>

                    {/* Platforms */}
                    <div className="flex flex-col p-3 rounded-xl bg-gradient-to-br from-gray-50 to-gray-100/50 dark:from-gray-800/50 dark:to-gray-900/50 border border-gray-100 dark:border-gray-800 transition-all duration-200 hover:border-gray-200 dark:hover:border-gray-700">
                        <span className="text-[10px] uppercase text-gray-500 font-semibold tracking-wider mb-1">Platforms</span>
                        <div className="flex items-center gap-1.5 flex-wrap">
                            {wallet.platforms.map((platform) => (
                                <div
                                    key={platform}
                                    title={PLATFORM_LABELS[platform]}
                                    className="p-1.5 rounded-lg bg-white dark:bg-gray-800 text-gray-600 dark:text-gray-400 shadow-sm border border-gray-100 dark:border-gray-700 transition-all duration-200 hover:scale-110 hover:text-blue-600 dark:hover:text-blue-400 cursor-default"
                                >
                                    {PLATFORM_ICONS[platform]}
                                </div>
                            ))}
                        </div>
                    </div>
                </div>

                {/* Footer: Labels & Notes - Enhanced */}
                {(isEditing) ? (
                    <div className="flex flex-col gap-3 bg-accent/50 rounded-xl p-4 border border-border animate-in fade-in zoom-in-95 duration-200" onClick={(e) => e.stopPropagation()}>
                        <div className="space-y-3">
                            <Input
                                placeholder="Label (max 20 chars)"
                                value={labelInput}
                                onChange={(e) => setLabelInput(e.target.value.slice(0, 20))}
                                className="h-9 text-sm bg-background rounded-lg"
                                autoFocus
                            />
                            <Textarea
                                placeholder="Note (max 500 chars)"
                                value={noteInput}
                                onChange={(e) => setNoteInput(e.target.value.slice(0, 500))}
                                className="min-h-[70px] text-sm bg-background resize-none rounded-lg"
                            />
                            <div className="flex justify-end gap-2 pt-1">
                                <Button
                                    size="sm"
                                    variant="ghost"
                                    className="h-8 px-3 text-sm rounded-lg"
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        setIsEditing(false);
                                    }}
                                >
                                    Cancel
                                </Button>
                                <Button
                                    size="sm"
                                    className="h-8 px-4 text-sm rounded-lg bg-primary text-primary-foreground hover:opacity-90"
                                    onClick={handleSaveMetadata}
                                    disabled={isSaving}
                                >
                                    {isSaving ? <Loader2 className="h-3.5 w-3.5 animate-spin mr-1.5" /> : <Save className="h-3.5 w-3.5 mr-1.5" />}
                                    Save
                                </Button>
                            </div>
                        </div>
                    </div>
                ) : (wallet.label || wallet.note) ? (
                    <div className="flex flex-col gap-2 bg-gradient-to-br from-gray-50 to-gray-100/50 dark:from-gray-800/30 dark:to-gray-900/30 rounded-xl p-4 border border-gray-100 dark:border-gray-800 group/notes relative transition-all duration-200 hover:border-gray-200 dark:hover:border-gray-700">
                        {/* Edit Button overlay */}
                        <button
                            onClick={(e) => {
                                e.stopPropagation();
                                handleStartEditing();
                            }}
                            className="absolute top-3 right-3 p-1.5 rounded-lg bg-white/80 dark:bg-gray-800/80 hover:bg-white dark:hover:bg-gray-800 text-gray-400 hover:text-blue-500 shadow-sm border border-gray-100 dark:border-gray-700 opacity-0 group-hover/notes:opacity-100 transition-all duration-200 hover:scale-110"
                            title="Edit label/note"
                        >
                            <Edit className="h-3.5 w-3.5" />
                        </button>

                        {wallet.label && (
                            <div className="flex items-center justify-between">
                                <span className="text-xs text-gray-500 font-semibold">Label</span>
                                <WalletLabelBadge label={wallet.label} size="sm" />
                            </div>
                        )}
                        {wallet.note && (
                            <div className="flex flex-col gap-1">
                                <span className="text-xs text-gray-500 font-semibold">Note</span>
                                <p className="text-sm text-gray-700 dark:text-gray-300 line-clamp-2 leading-relaxed whitespace-pre-wrap">
                                    {wallet.note}
                                </p>
                            </div>
                        )}
                    </div>
                ) : (
                    /* Empty State Placeholder - Enhanced */
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            handleStartEditing();
                        }}
                        className="flex items-center gap-2 text-sm text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 transition-all duration-200 px-3 py-2.5 rounded-xl hover:bg-gradient-to-r hover:from-blue-50 hover:to-indigo-50 dark:hover:from-blue-900/20 dark:hover:to-indigo-900/20 border border-dashed border-gray-200 dark:border-gray-700 hover:border-blue-300 dark:hover:border-blue-700 w-full text-left group/add"
                    >
                        <div className="p-1 rounded-md bg-gray-100 dark:bg-gray-800 group-hover/add:bg-blue-100 dark:group-hover/add:bg-blue-900/40 transition-colors duration-200">
                            <Edit className="h-3.5 w-3.5" />
                        </div>
                        <span className="font-medium">Add label or note...</span>
                        <Sparkles className="h-3.5 w-3.5 ml-auto opacity-0 group-hover/add:opacity-100 transition-opacity duration-200" />
                    </button>
                )}

                {/* Disable reason warning */}
                {wallet.disableInfo && (
                    <div className="text-sm text-amber-700 dark:text-amber-400 bg-gradient-to-r from-amber-50 to-orange-50 dark:from-amber-900/20 dark:to-orange-900/20 px-4 py-3 rounded-xl border border-amber-200 dark:border-amber-900/40">
                        <span className="font-semibold block mb-1">⚠️ Disabled: {wallet.disableInfo.reasonCategory}</span>
                        <span className="text-amber-600 dark:text-amber-500">{wallet.disableInfo.reasonDetails}</span>
                    </div>
                )}
            </div>
        </div>
    );
}
