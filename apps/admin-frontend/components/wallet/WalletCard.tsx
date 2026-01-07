/**
 * Wallet Card Component
 * Compact wallet display for list view with action buttons
 */
'use client';

import { BarChart3, Coins, Copy, CreditCard, Eye, MoreHorizontal, Settings, TrendingUp } from 'lucide-react';
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
import { cn } from '@/lib/utils';

import type { Platform, WalletData, WalletStatus } from './types';
import { WalletLabelBadge } from './WalletLabelBadge';

interface WalletCardProps {
    wallet: WalletData;
    isSelected?: boolean;
    onSelect?: (selected: boolean) => void;
    onView?: () => void;
    onManage?: () => void;
    onDisable?: () => void;
    onEnable?: () => void;
    className?: string;
}

const PLATFORM_ICONS: Record<Platform, React.ReactNode> = {
    analytics: <BarChart3 className="h-3 w-3" />,
    pay: <CreditCard className="h-3 w-3" />,
    token: <Coins className="h-3 w-3" />,
    markets: <TrendingUp className="h-3 w-3" />,
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

function formatWalletAddress(address: string): string {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

function formatTimeAgo(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffInHours = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60));

    if (diffInHours < 1) return 'Just now';
    if (diffInHours < 24) return `${diffInHours}h ago`;

    const diffInDays = Math.floor(diffInHours / 24);
    if (diffInDays < 30) return `${diffInDays}d ago`;

    return date.toLocaleDateString();
}

export function WalletCard({
    wallet,
    isSelected = false,
    onSelect,
    onView,
    onManage,
    onDisable,
    onEnable,
    className,
}: WalletCardProps) {
    const [copied, setCopied] = useState(false);
    const statusConfig = STATUS_CONFIG[wallet.status];
    const isDisabled = wallet.status === 'disabled';
    const activePermissions = wallet.permissions.filter(p => p.isActive).length;

    const handleCopy = async (e: React.MouseEvent) => {
        e.stopPropagation();
        await navigator.clipboard.writeText(wallet.walletAddress);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <div
            className={cn(
                'relative overflow-hidden rounded-2xl bg-gradient-to-r p-0.5 transition-all duration-200',
                isDisabled
                    ? 'from-amber-400/20 via-orange-400/20 to-red-400/20'
                    : 'from-blue-400/10 via-purple-400/10 to-pink-400/10 hover:from-blue-400/20 hover:via-purple-400/20 hover:to-pink-400/20',
                isSelected && 'ring-2 ring-blue-500 ring-offset-2',
                className
            )}
        >
            <div className="flex items-center justify-between rounded-2xl bg-white/95 p-5 backdrop-blur-xl dark:bg-gray-900/95">
                {/* Selection Checkbox */}
                {onSelect && (
                    <div className="mr-4">
                        <input
                            type="checkbox"
                            checked={isSelected}
                            onChange={(e) => onSelect(e.target.checked)}
                            className="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                        />
                    </div>
                )}

                {/* Wallet Info */}
                <div className="flex items-center gap-4 flex-1 min-w-0">
                    {/* Avatar */}
                    <div className={cn(
                        'relative flex h-12 w-12 shrink-0 items-center justify-center rounded-xl font-bold text-white shadow-lg',
                        isDisabled
                            ? 'bg-gradient-to-br from-amber-500 via-orange-500 to-red-500'
                            : 'bg-gradient-to-br from-blue-500 via-purple-500 to-pink-500'
                    )}>
                        {wallet.walletAddress.slice(2, 4).toUpperCase()}
                        <div className={cn(
                            'absolute -top-1 -right-1 h-3 w-3 rounded-full border-2 border-white',
                            wallet.status === 'active' ? 'bg-green-400' :
                                wallet.status === 'disabled' ? 'bg-amber-400' : 'bg-blue-400'
                        )} />
                    </div>

                    {/* Details */}
                    <div className="min-w-0 flex-1">
                        {/* Address */}
                        <div className="flex items-center gap-2 mb-1.5">
                            <span className="font-mono text-sm font-semibold text-gray-900 dark:text-white">
                                {formatWalletAddress(wallet.walletAddress)}
                            </span>
                            <button
                                onClick={handleCopy}
                                className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                                title="Copy address"
                            >
                                <Copy className="h-3 w-3" />
                            </button>
                            {copied && (
                                <span className="text-xs text-green-600 dark:text-green-400">Copied!</span>
                            )}
                        </div>

                        {/* Status & Platforms */}
                        <div className="flex flex-wrap items-center gap-2">
                            <Badge className={cn('text-xs px-2 py-0.5', statusConfig.className)}>
                                {statusConfig.emoji} {statusConfig.label}
                            </Badge>

                            {/* Label Badge */}
                            {wallet.label && (
                                <WalletLabelBadge label={wallet.label} size="sm" />
                            )}

                            {wallet.platforms.map((platform) => (
                                <Badge
                                    key={platform}
                                    variant="outline"
                                    className="text-xs px-2 py-0.5 gap-1"
                                >
                                    {PLATFORM_ICONS[platform]}
                                    {PLATFORM_LABELS[platform]}
                                </Badge>
                            ))}
                        </div>

                        {/* Note snippet */}
                        {wallet.note && (
                            <div className="mt-1.5 text-xs text-gray-500 dark:text-gray-400 truncate max-w-[300px]" title={wallet.note}>
                                📝 {wallet.note.length > 50 ? wallet.note.slice(0, 50) + '...' : wallet.note}
                            </div>
                        )}

                        {/* Disable reason if applicable */}
                        {wallet.disableInfo && (
                            <div className="mt-2 text-xs text-amber-600 dark:text-amber-400">
                                <span className="font-medium">Reason:</span> {wallet.disableInfo.reasonDetails.slice(0, 50)}
                                {wallet.disableInfo.reasonDetails.length > 50 && '...'}
                            </div>
                        )}
                    </div>
                </div>

                {/* Stats */}
                <div className="hidden md:flex items-center gap-8 mx-6">
                    {/* Plan */}
                    <div className="flex flex-col items-start min-w-[60px]">
                        <span className="text-[10px] uppercase text-gray-400 font-semibold tracking-wider mb-0.5">Plan</span>
                        <div className="flex items-center gap-1.5">
                            <span className="text-sm font-medium text-gray-900 dark:text-gray-200">
                                {wallet.subscriptions[0]?.planName || 'Free'}
                            </span>
                        </div>
                    </div>

                    {/* Group */}
                    <div className="flex flex-col items-start min-w-[60px]">
                        <span className="text-[10px] uppercase text-gray-400 font-semibold tracking-wider mb-0.5">Group</span>
                        <div className="flex items-center gap-1.5">
                            <span className="text-sm font-medium text-gray-900 dark:text-gray-200">
                                {wallet.groups?.[0]?.groupName || 'User'}
                                {(wallet.groups?.length || 0) > 1 && (
                                    <span className="text-xs text-gray-500">+{wallet.groups!.length - 1}</span>
                                )}
                            </span>
                        </div>
                    </div>

                    {/* Permission */}
                    <div className="flex flex-col items-start min-w-[60px]">
                        <span className="text-[10px] uppercase text-gray-400 font-semibold tracking-wider mb-0.5">Perms</span>
                        <div className="flex items-center gap-1.5">
                            <span className="text-sm font-medium text-gray-900 dark:text-gray-200">
                                {activePermissions}
                            </span>
                            <span className="text-xs text-gray-500">Access</span>
                        </div>
                    </div>
                </div>

                {/* Actions */}
                <div className="flex items-center gap-2 ml-4">
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={onView}
                        className="h-9 px-3 gap-1.5"
                    >
                        <Eye className="h-4 w-4" />
                        <span className="hidden lg:inline">View</span>
                    </Button>

                    <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                            <Button variant="outline" size="sm" className="h-9 w-9 p-0">
                                <MoreHorizontal className="h-4 w-4" />
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end" className="w-48">
                            <DropdownMenuItem onClick={onView}>
                                <Eye className="h-4 w-4 mr-2" />
                                View Details
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={onManage}>
                                <Settings className="h-4 w-4 mr-2" />
                                Manage Permissions
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            {isDisabled ? (
                                <DropdownMenuItem
                                    onClick={onEnable}
                                    className="text-green-600 dark:text-green-400"
                                >
                                    <span className="mr-2">🔓</span>
                                    Re-enable Access
                                </DropdownMenuItem>
                            ) : (
                                <DropdownMenuItem
                                    onClick={onDisable}
                                    className="text-amber-600 dark:text-amber-400"
                                >
                                    <span className="mr-2">⚠️</span>
                                    Temporarily Disable
                                </DropdownMenuItem>
                            )}
                        </DropdownMenuContent>
                    </DropdownMenu>
                </div>
            </div>
        </div>
    );
}
