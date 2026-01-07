/**
 * Wallet Label Badge Component
 * Displays a colored badge for wallet labels with auto-assigned colors
 */
'use client';

import { cn } from '@/lib/utils';

// Color palette for labels (auto-assigned based on label text hash)
const LABEL_COLORS = [
    { bg: 'bg-blue-100 dark:bg-blue-900/40', text: 'text-blue-700 dark:text-blue-300', border: 'border-blue-200 dark:border-blue-800' },
    { bg: 'bg-emerald-100 dark:bg-emerald-900/40', text: 'text-emerald-700 dark:text-emerald-300', border: 'border-emerald-200 dark:border-emerald-800' },
    { bg: 'bg-amber-100 dark:bg-amber-900/40', text: 'text-amber-700 dark:text-amber-300', border: 'border-amber-200 dark:border-amber-800' },
    { bg: 'bg-rose-100 dark:bg-rose-900/40', text: 'text-rose-700 dark:text-rose-300', border: 'border-rose-200 dark:border-rose-800' },
    { bg: 'bg-purple-100 dark:bg-purple-900/40', text: 'text-purple-700 dark:text-purple-300', border: 'border-purple-200 dark:border-purple-800' },
    { bg: 'bg-cyan-100 dark:bg-cyan-900/40', text: 'text-cyan-700 dark:text-cyan-300', border: 'border-cyan-200 dark:border-cyan-800' },
    { bg: 'bg-orange-100 dark:bg-orange-900/40', text: 'text-orange-700 dark:text-orange-300', border: 'border-orange-200 dark:border-orange-800' },
    { bg: 'bg-indigo-100 dark:bg-indigo-900/40', text: 'text-indigo-700 dark:text-indigo-300', border: 'border-indigo-200 dark:border-indigo-800' },
];

/**
 * Simple hash function for consistent color assignment
 */
function hashString(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
        const char = str.charCodeAt(i);
        hash = ((hash << 5) - hash) + char;
        hash = hash & hash; // Convert to 32bit integer
    }
    return Math.abs(hash);
}

/**
 * Get color classes for a label based on its text
 */
export function getLabelColor(label: string) {
    const index = hashString(label.toLowerCase()) % LABEL_COLORS.length;
    return LABEL_COLORS[index]!;
}

interface WalletLabelBadgeProps {
    label: string;
    size?: 'sm' | 'md';
    className?: string;
    onRemove?: () => void;
}

export function WalletLabelBadge({
    label,
    size = 'sm',
    className,
    onRemove,
}: WalletLabelBadgeProps) {
    const colors = getLabelColor(label);

    return (
        <span
            className={cn(
                'inline-flex items-center gap-1 font-medium border rounded-full',
                colors.bg,
                colors.text,
                colors.border,
                size === 'sm' ? 'px-2 py-0.5 text-xs' : 'px-3 py-1 text-sm',
                className
            )}
        >
            <span className="truncate max-w-[120px]">{label}</span>
            {onRemove && (
                <button
                    type="button"
                    onClick={(e) => {
                        e.stopPropagation();
                        onRemove();
                    }}
                    className="ml-0.5 hover:bg-black/10 dark:hover:bg-white/10 rounded-full p-0.5 transition-colors"
                    aria-label={`Remove label ${label}`}
                >
                    <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            )}
        </span>
    );
}
