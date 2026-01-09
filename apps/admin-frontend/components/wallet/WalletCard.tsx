/**
 * Wallet Card Component
 * Compact wallet display for list view with action buttons
 */
'use client';

import { BarChart3, Coins, Copy, CreditCard, Edit, Eye, Loader2, MoreHorizontal, Save, Settings, TrendingUp } from 'lucide-react';
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
import { cn } from '@/lib/utils';

interface WalletCardProps {
    wallet: WalletData;
    isSelected?: boolean;
    onSelect?: (selected: boolean) => void;
    onView?: () => void;
    onManage?: () => void;
    onDisable?: () => void;
    onEnable?: () => void;
    onEdit?: () => void; // Keeping for backward compatibility or "Manage" menu
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

const STATUS_CONFIG: Record<WalletStatus, { label: string; emoji: string; className: string }> = {
    active: {
        label: 'Active',
        emoji: '🟢',
        className: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400',
    },
    disabled: {
        label: 'Disabled',
        emoji: '⚠️',
        className: 'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400',
    },
    pending: {
        label: 'Pending',
        emoji: '⏳',
        className: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400',
    },
};

/**
 *
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
        await navigator.clipboard.writeText(wallet.walletAddress);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    const handleStartEditing = () => {
        setLabelInput(wallet.label || '');
        setNoteInput(wallet.note || '');
        setIsEditing(true);
    };

    const handleSaveMetadata = async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (!onUpdateMetadata) {return;}

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
                'group relative rounded-xl border bg-white transition-all duration-200',
                isDisabled
                    ? 'border-amber-200/50 bg-amber-50/30 dark:border-amber-800/30 dark:bg-amber-950/20 opacity-75'
                    : 'border-gray-200 hover:border-blue-300 hover:shadow-md dark:border-gray-700 dark:hover:border-blue-600',
                isSelected && 'ring-2 ring-blue-500 ring-offset-2 dark:ring-offset-gray-900',
                className
            )}
        >
            <div className="flex flex-col p-4 gap-4">
                {/* Header: Checkbox + Avatar + Address + Status */}
                <div className="flex items-center gap-3">
                    {/* Checkbox */}
                    {onSelect && (
                        <div className="flex items-center">
                            <input
                                type="checkbox"
                                checked={isSelected}
                                onChange={(e) => onSelect(e.target.checked)}
                                className="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-gray-600"
                            />
                        </div>
                    )}

                    {/* Avatar */}
                    <div className={cn(
                        'relative flex h-10 w-10 shrink-0 items-center justify-center rounded-lg text-sm font-semibold shadow-sm',
                        isDisabled
                            ? 'bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-300'
                            : 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300'
                    )}>
                        {wallet.walletAddress.slice(2, 4).toUpperCase()}
                        <div className={cn(
                            'absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full border-2 border-white dark:border-gray-900',
                            wallet.status === 'active' ? 'bg-green-500' :
                                wallet.status === 'disabled' ? 'bg-amber-500' : 'bg-blue-500'
                        )} />
                    </div>

                    {/* Address & Copy */}
                    <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                            <span className="font-mono text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                                {wallet.walletAddress}
                            </span>
                            <button
                                onClick={handleCopy}
                                className={cn(
                                    'shrink-0 p-1 rounded transition-colors',
                                    copied
                                        ? 'bg-green-100 text-green-600 dark:bg-green-900/40 dark:text-green-400'
                                        : 'text-gray-400 hover:bg-gray-100 hover:text-gray-600 dark:hover:bg-gray-800 dark:hover:text-gray-300'
                                )}
                                title="Copy address"
                            >
                                <Copy className="h-3 w-3" />
                            </button>
                        </div>
                    </div>

                    {/* Status Badge (Desktop) */}
                    <div className="hidden sm:block">
                        <Badge className={cn('text-xs px-2 py-0.5 font-medium', statusConfig.className)}>
                            {statusConfig.label}
                        </Badge>
                    </div>

                    {/* Actions Menu */}
                    <div className="flex items-center gap-1">
                        <Button
                            variant="ghost"
                            size="sm"
                            onClick={onView}
                            className="hidden sm:flex h-8 px-2.5 gap-1.5 text-xs hover:bg-blue-50 hover:text-blue-700 dark:hover:bg-blue-900/30 dark:hover:text-blue-400"
                        >
                            <Eye className="h-3.5 w-3.5" />
                            View
                        </Button>

                        <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    className="h-8 w-8 p-0 hover:bg-gray-100 dark:hover:bg-gray-800"
                                >
                                    <MoreHorizontal className="h-4 w-4" />
                                </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end" className="w-44">
                                <DropdownMenuItem onClick={onUpdateMetadata ? (e) => { e.stopPropagation(); handleStartEditing(); } : onEdit}>
                                    <Edit className="h-4 w-4 mr-2" />
                                    <span className="text-sm">Edit Label/Note</span>
                                </DropdownMenuItem>
                                <DropdownMenuItem onClick={onManage}>
                                    <Settings className="h-4 w-4 mr-2" />
                                    <span className="text-sm">Manage</span>
                                </DropdownMenuItem>
                                <DropdownMenuSeparator />
                                {isDisabled ? (
                                    <DropdownMenuItem
                                        onClick={onEnable}
                                        className="text-green-700 dark:text-green-400"
                                    >
                                        <span className="mr-2">🔓</span>
                                        <span className="text-sm">Re-enable</span>
                                    </DropdownMenuItem>
                                ) : (
                                    <DropdownMenuItem
                                        onClick={onDisable}
                                        className="text-amber-700 dark:text-amber-400"
                                    >
                                        <span className="mr-2">⚠️</span>
                                        <span className="text-sm">Disable</span>
                                    </DropdownMenuItem>
                                )}
                            </DropdownMenuContent>
                        </DropdownMenu>
                    </div>
                </div>

                {/* Key Metrics Grid */}
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 py-2 border-t border-b border-gray-50 dark:border-gray-800">
                    <div className="flex flex-col">
                        <span className="text-[10px] uppercase text-gray-400 font-semibold tracking-wide mb-0.5">Plan</span>
                        <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
                            {wallet.subscriptions[0]?.planName || 'Free'}
                        </span>
                    </div>
                    <div className="flex flex-col">
                        <span className="text-[10px] uppercase text-gray-400 font-semibold tracking-wide mb-0.5">Group</span>
                        <div className="flex items-center gap-1">
                            <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
                                {wallet.groups?.[0]?.groupName || 'User'}
                            </span>
                            {(wallet.groups?.length || 0) > 1 && (
                                <span className="text-xs text-gray-500">+{wallet.groups!.length - 1}</span>
                            )}
                        </div>
                    </div>
                    <div className="flex flex-col">
                        <span className="text-[10px] uppercase text-gray-400 font-semibold tracking-wide mb-0.5">Perms</span>
                        <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
                            {activePermissions} <span className="text-gray-500 font-normal">active</span>
                        </span>
                    </div>
                    <div className="flex flex-col">
                        <span className="text-[10px] uppercase text-gray-400 font-semibold tracking-wide mb-0.5">Platforms</span>
                        <div className="flex items-center gap-1.5 flex-wrap">
                            {wallet.platforms.map((platform) => (
                                <div
                                    key={platform}
                                    title={PLATFORM_LABELS[platform]}
                                    className="p-1 rounded bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400"
                                >
                                    {PLATFORM_ICONS[platform]}
                                </div>
                            ))}
                        </div>
                    </div>
                </div>

                {/* Footer: Labels & Notes */}
                {(isEditing) ? (
                    <div className="flex flex-col gap-2 bg-gray-50 dark:bg-gray-900/40 rounded-lg p-3 animate-in fade-in zoom-in-95 duration-200" onClick={(e) => e.stopPropagation()}>
                        <div className="space-y-2">
                            <Input
                                placeholder="Label (max 20 chars)"
                                value={labelInput}
                                onChange={(e) => setLabelInput(e.target.value.slice(0, 20))}
                                className="h-8 text-xs bg-white dark:bg-gray-800"
                                autoFocus
                            />
                            <Textarea
                                placeholder="Note (max 500 chars)"
                                value={noteInput}
                                onChange={(e) => setNoteInput(e.target.value.slice(0, 500))}
                                className="min-h-[60px] text-xs bg-white dark:bg-gray-800 resize-none"
                            />
                            <div className="flex justify-end gap-2 pt-1">
                                <Button
                                    size="sm"
                                    variant="ghost"
                                    className="h-6 px-2 text-xs"
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        setIsEditing(false);
                                    }}
                                >
                                    Cancel
                                </Button>
                                <Button
                                    size="sm"
                                    className="h-6 px-2 text-xs"
                                    onClick={handleSaveMetadata}
                                    disabled={isSaving}
                                >
                                    {isSaving ? <Loader2 className="h-3 w-3 animate-spin mr-1" /> : <Save className="h-3 w-3 mr-1" />}
                                    Save
                                </Button>
                            </div>
                        </div>
                    </div>
                ) : (wallet.label || wallet.note) ? (
                    <div className="flex flex-col gap-2 bg-gray-50 dark:bg-gray-900/40 rounded-lg p-3 group/notes relative">
                        {/* Edit Button overlay */}
                        <button
                            onClick={(e) => {
                                e.stopPropagation();
                                handleStartEditing();
                            }}
                            className="absolute top-2 right-2 p-1 rounded-md bg-white/80 dark:bg-gray-800/80 hover:bg-white dark:hover:bg-gray-800 text-gray-400 hover:text-blue-500 shadow-sm opacity-0 group-hover/notes:opacity-100 transition-all duration-200"
                            title="Edit label/note"
                        >
                            <Edit className="h-3 w-3" />
                        </button>

                        {wallet.label && (
                            <div className="flex items-center justify-between">
                                <span className="text-xs text-gray-500 font-medium">Label</span>
                                <WalletLabelBadge label={wallet.label} size="sm" />
                            </div>
                        )}
                        {wallet.note && (
                            <div className="flex flex-col gap-1">
                                <span className="text-xs text-gray-500 font-medium">Note</span>
                                <p className="text-xs text-gray-700 dark:text-gray-300 line-clamp-2 leading-relaxed whitespace-pre-wrap">
                                    {wallet.note}
                                </p>
                            </div>
                        )}
                    </div>
                ) : (
                    /* Empty State Placeholder - Encourages adding notes */
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            handleStartEditing();
                        }}
                        className="flex items-center gap-2 text-xs text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors px-1 py-1 rounded hover:bg-gray-50 dark:hover:bg-gray-800/50 w-full text-left"
                    >
                        <Edit className="h-3 w-3" />
                        <span>Add label or note...</span>
                    </button>
                )}

                {/* Disable reason warning */}
                {wallet.disableInfo && (
                    <div className="text-xs text-amber-700 dark:text-amber-400 bg-amber-50 dark:bg-amber-900/20 px-3 py-2 rounded-lg border border-amber-100 dark:border-amber-900/30">
                        <span className="font-semibold block mb-0.5">⚠️ Disabled: {wallet.disableInfo.reasonCategory}</span>
                        {wallet.disableInfo.reasonDetails}
                    </div>
                )}
            </div>
        </div>
    );
}
