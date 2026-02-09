/**
 * Wallet Status Badge Component
 * Consistent status representation for wallets
 */
'use client';

import type { WalletStatus } from './types';

import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';

export const STATUS_CONFIG: Record<WalletStatus, { label: string; emoji: string; className: string }> = {
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

interface WalletStatusBadgeProps {
    status: WalletStatus;
    className?: string;
}

/**
 *
 * @param root0
 * @param root0.status
 * @param root0.className
 */
export function WalletStatusBadge({ status, className }: WalletStatusBadgeProps) {
    const config = (STATUS_CONFIG[status] as ((typeof STATUS_CONFIG)[WalletStatus] | undefined)) ?? STATUS_CONFIG.active;

    return (
        <Badge className={cn('px-3 py-1 border font-medium', config.className, className)}>
            <span className="mr-1.5">{config.emoji}</span>
            {config.label}
        </Badge>
    );
}
