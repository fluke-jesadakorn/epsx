/**
 * Wallet Header Component
 * Comprehensive header for wallet detail view including status and core metadata
 */
'use client';

import { Clock, Copy } from 'lucide-react';
import { useState } from 'react';

import type { WalletData } from './types';
import { WalletStatusBadge } from './WalletStatusBadge';

import { cn } from '@/lib/utils';
import { formatDate, formatTimeAgo } from '@/lib/utils/date';

interface WalletHeaderProps {
    wallet: WalletData;
    className?: string;
}

/**
 *
 * @param root0
 * @param root0.wallet
 * @param root0.className
 */
export function WalletHeader({ wallet, className }: WalletHeaderProps) {
    const [copied, setCopied] = useState(false);

    const handleCopyAddress = async () => {
        await navigator.clipboard.writeText(wallet.walletAddress);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <div className={cn(
            'rounded-xl bg-gradient-to-r from-blue-50 to-purple-50 dark:from-blue-900/10 dark:to-purple-900/10 p-5 border border-blue-100 dark:border-blue-900/30',
            className
        )}>
            {/* Address Row */}
            <div className="flex items-center gap-3 mb-4">
                <div className="flex-1 min-w-0">
                    <code className="text-sm font-mono font-bold text-gray-900 dark:text-white break-all bg-white/50 dark:bg-black/20 px-2 py-1 rounded">
                        {wallet.walletAddress}
                    </code>
                </div>
                <div className="relative flex-shrink-0">
                    <button
                        onClick={handleCopyAddress}
                        className="p-2 rounded-lg hover:bg-white dark:hover:bg-gray-800 text-gray-500 hover:text-blue-600 dark:hover:text-blue-400 transition-colors shadow-sm"
                        title="Copy address"
                    >
                        <Copy className="h-4 w-4" />
                    </button>
                    {copied && (
                        <span className="absolute -top-8 left-1/2 -translate-x-1/2 text-[10px] font-bold text-green-600 dark:text-green-400 bg-white dark:bg-gray-900 px-2 py-1 rounded shadow-sm border border-green-100 dark:border-green-900/30 animate-in fade-in slide-in-from-bottom-1">
                            Copied!
                        </span>
                    )}
                </div>
            </div>

            {/* Status & Timing Metadata */}
            <div className="flex flex-wrap items-center gap-y-3 gap-x-6">
                <WalletStatusBadge status={wallet.status} />

                <div className="flex items-center text-xs text-gray-600 dark:text-gray-400">
                    <Clock className="h-3.5 w-3.5 mr-1.5 opacity-70" />
                    <span>Created {formatTimeAgo(wallet.createdAt)}</span>
                </div>

                {wallet.lastAuthAt && (
                    <div className="flex items-center text-xs text-gray-600 dark:text-gray-400">
                        <span className="w-1 h-1 rounded-full bg-gray-300 dark:bg-gray-600 mx-2 hidden sm:block"></span>
                        <span>Last active {formatTimeAgo(wallet.lastAuthAt)}</span>
                    </div>
                )}
            </div>

            {/* Disable Reason Callout */}
            {wallet.status === 'disabled' && wallet.disableInfo && (
                <div className="mt-5 p-3.5 rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-100 dark:border-amber-900/30">
                    <div className="flex items-center gap-2 text-amber-800 dark:text-amber-400 font-semibold text-sm mb-1.5">
                        <span>⚠️ Access Restricted</span>
                    </div>
                    <p className="text-sm text-gray-700 dark:text-gray-300 leading-relaxed">
                        <strong className="text-amber-700 dark:text-amber-500">Reason:</strong> {wallet.disableInfo.reasonDetails}
                    </p>
                    <div className="mt-2.5 pt-2.5 border-t border-amber-100 dark:border-amber-900/30 text-[11px] text-gray-500 dark:text-gray-400 flex flex-wrap gap-x-3 gap-y-1">
                        <span>Disabled by <strong>{wallet.disableInfo.disabledBy}</strong></span>
                        <span>•</span>
                        <span>{formatDate(wallet.disableInfo.disabledAt)}</span>
                        {wallet.disableInfo.expiresAt && (
                            <>
                                <span>•</span>
                                <span className="text-amber-600 dark:text-amber-500">Expires {formatDate(wallet.disableInfo.expiresAt)}</span>
                            </>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
}
