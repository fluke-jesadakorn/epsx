/**
 * Shared Design System Component Variants (CVA)
 *
 * Type-safe component variants using Class Variance Authority (CVA).
 * Provides consistent styling patterns for both Frontend and Admin-Frontend apps.
 *
 * Features:
 * - Type-safe component styling with IntelliSense
 * - Composable variants for flexible design
 * - Tree-shakeable (only used variants are bundled)
 */

import { cva, type VariantProps } from 'class-variance-authority';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

/**
 * Common shadow and text styles for interactive components
 */
const INTERACTIVE_SHADOWS = 'text-white shadow-lg hover:shadow-xl';
const INSIGHT_SHADOW = 'shadow-lg hover:shadow-2xl';

// ============================================================================
// BUTTON VARIANTS
// ============================================================================

/**
 * Enhanced button variants for all interfaces
 */
export const buttonVariants = cva(
    [
        'inline-flex items-center justify-center whitespace-nowrap',
        'rounded-lg font-semibold text-sm',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2',
        'disabled:pointer-events-none disabled:opacity-50',
        'relative overflow-hidden',
    ],
    {
        variants: {
            variant: {
                // Primary actions (save, create, confirm)
                primary: [
                    'bg-gradient-to-r from-orange-500 to-yellow-500',
                    INTERACTIVE_SHADOWS,
                    'focus-visible:ring-orange-500',
                ],

                // Secondary actions (cancel, back)
                secondary: [
                    'bg-gradient-to-r from-blue-500 to-purple-500',
                    INTERACTIVE_SHADOWS,
                    'focus-visible:ring-blue-500',
                ],

                // Success actions (approve, activate)
                success: [
                    'bg-gradient-to-r from-green-500 to-emerald-500',
                    INTERACTIVE_SHADOWS,
                    'focus-visible:ring-green-500',
                ],

                // Destructive actions (delete, suspend, revoke)
                destructive: [
                    'bg-gradient-to-r from-red-500 to-rose-500',
                    INTERACTIVE_SHADOWS,
                    'focus-visible:ring-red-500',
                ],

                // Warning actions (pending, trial)
                warning: [
                    'bg-gradient-to-r from-amber-500 to-orange-500',
                    INTERACTIVE_SHADOWS,
                    'focus-visible:ring-amber-500',
                ],

                // Outline variants
                outline: [
                    'border-2 border-gray-300 bg-white text-gray-700',
                    'hover:bg-gray-50 hover:border-gray-400',
                    'focus-visible:ring-gray-500',
                    'dark:border-gray-600 dark:bg-gray-800 dark:text-gray-200',
                    'dark:hover:bg-gray-700 dark:hover:border-gray-500',
                ],

                // Ghost variants
                ghost: [
                    'text-gray-700 hover:bg-gray-100',
                    'focus-visible:ring-gray-500',
                    'dark:text-gray-200 dark:hover:bg-gray-800',
                ],

                // Link style
                link: [
                    'text-orange-600 underline-offset-4 hover:underline',
                    'focus-visible:ring-orange-500',
                    'dark:text-orange-400',
                ],

                // Analytics/Insight style
                insight: [
                    'bg-gradient-to-r from-blue-500 to-indigo-500',
                    INTERACTIVE_SHADOWS,
                    'focus-visible:ring-blue-500',
                ],
            },

            size: {
                sm: 'h-8 px-3 text-xs',
                default: 'h-10 px-4 py-2',
                lg: 'h-12 px-6 py-3 text-base',
                xl: 'h-14 px-8 py-4 text-lg',
                icon: 'h-10 w-10 p-0',
            },

            glow: {
                none: '',
                subtle: 'hover:shadow-lg',
                medium: 'hover:shadow-xl',
                strong: 'hover:shadow-2xl',
            },

            fullWidth: {
                true: 'w-full',
                false: '',
            },
        },

        defaultVariants: {
            variant: 'primary',
            size: 'default',
            glow: 'medium',
            fullWidth: false,
        },
    }
);

export type ButtonVariants = VariantProps<typeof buttonVariants>;

// ============================================================================
// CARD VARIANTS
// ============================================================================

/**
 * Enhanced card variants for dashboards
 */
export const cardVariants = cva(
    ['rounded-2xl border backdrop-blur-sm', 'relative overflow-hidden group'],
    {
        variants: {
            variant: {
                // Standard dashboard card
                default: [
                    'bg-white/90 dark:bg-gray-800/90 border-gray-200 dark:border-gray-700',
                    'shadow-sm hover:shadow-md',
                ],

                // Enhanced analytics-style card
                insight: [
                    'bg-gradient-to-br from-blue-50 via-indigo-50 to-blue-100',
                    'dark:from-gray-800 dark:via-blue-900/20 dark:to-gray-700',
                    'border-blue-200/60 dark:border-blue-800/40',
                    INSIGHT_SHADOW,
                ],

                // PancakeSwap-style card
                pancake: [
                    'bg-gradient-to-br from-yellow-50 via-orange-50 to-yellow-100',
                    'dark:from-gray-800 dark:via-orange-900/20 dark:to-gray-700',
                    'border-yellow-200/60 dark:border-orange-800/40',
                    INSIGHT_SHADOW,
                ],

                // User management card
                user: [
                    'bg-gradient-to-br from-blue-50 via-indigo-50 to-blue-100',
                    'dark:from-gray-800 dark:via-blue-900/20 dark:to-gray-700',
                    'border-blue-200/60 dark:border-blue-800/40',
                    INSIGHT_SHADOW,
                ],

                // Permission card
                permission: [
                    'bg-gradient-to-br from-purple-50 via-violet-50 to-purple-100',
                    'dark:from-gray-800 dark:via-purple-900/20 dark:to-gray-700',
                    'border-purple-200/60 dark:border-purple-800/40',
                    INSIGHT_SHADOW,
                ],

                // Billing card
                billing: [
                    'bg-gradient-to-br from-green-50 via-emerald-50 to-green-100',
                    'dark:from-gray-800 dark:via-green-900/20 dark:to-gray-700',
                    'border-green-200/60 dark:border-green-800/40',
                    INSIGHT_SHADOW,
                ],

                // Analytics card
                analytics: [
                    'bg-gradient-to-br from-indigo-50 via-cyan-50 to-indigo-100',
                    'dark:from-gray-800 dark:via-indigo-900/20 dark:to-gray-700',
                    'border-indigo-200/60 dark:border-indigo-800/40',
                    INSIGHT_SHADOW,
                ],

                // Warning card
                warning: [
                    'bg-gradient-to-br from-amber-50 via-yellow-50 to-amber-100',
                    'dark:from-amber-900/20 dark:via-yellow-900/20 dark:to-amber-800/10',
                    'border-amber-300/60 dark:border-amber-700/40',
                    'shadow-lg shadow-amber-200/30 dark:shadow-amber-900/20',
                ],

                // Error card
                error: [
                    'bg-gradient-to-br from-red-50 via-pink-50 to-red-100',
                    'dark:from-red-900/20 dark:via-pink-900/20 dark:to-red-800/10',
                    'border-red-300/60 dark:border-red-700/40',
                    'shadow-lg shadow-red-200/30 dark:shadow-red-900/20',
                ],
            },

            hover: {
                none: '',
                lift: 'hover:shadow-lg',
                glow: 'hover:shadow-2xl hover:shadow-current/10',
                both: 'hover:shadow-2xl',
            },

            padding: {
                none: 'p-0',
                xs: 'p-2',
                sm: 'p-3',
                default: 'p-4',
                md: 'p-6',
                lg: 'p-8',
                xl: 'p-10',
            },

            interactive: {
                true: 'cursor-pointer select-none touch-manipulation',
                false: '',
            },
        },

        defaultVariants: {
            variant: 'default',
            hover: 'both',
            padding: 'default',
            interactive: false,
        },
    }
);

export type CardVariants = VariantProps<typeof cardVariants>;

// ============================================================================
// BADGE VARIANTS
// ============================================================================

const SHADOW_GREEN_TEXT_WHITE = 'text-white shadow-green-200/50';
const SHADOW_RED_TEXT_WHITE = 'text-white shadow-red-200/50';

/**
 * Status badge variants
 */
export const badgeVariants = cva(
    [
        'inline-flex items-center rounded-xl px-3 py-1.5 relative overflow-hidden',
        'text-xs font-light uppercase tracking-wider',
        'shadow-sm hover:shadow-md',
        'border border-transparent min-h-[28px]',
    ],
    {
        variants: {
            variant: {
                // User status badges
                active: [
                    'bg-gradient-to-r from-green-400 to-emerald-400',
                    SHADOW_GREEN_TEXT_WHITE,
                ],
                inactive: [
                    'bg-gradient-to-r from-gray-300 to-slate-300',
                    'dark:from-gray-600 dark:to-slate-600',
                    'text-gray-700 dark:text-gray-200',
                ],
                pending: [
                    'bg-gradient-to-r from-yellow-400 to-orange-400',
                    'text-black shadow-yellow-200/50',
                ],
                suspended: [
                    'bg-gradient-to-r from-red-400 to-orange-500',
                    SHADOW_RED_TEXT_WHITE,
                ],
                premium: [
                    'bg-gradient-to-r from-purple-400 to-pink-400',
                    'text-white shadow-purple-200/50',
                ],

                // Permission status badges
                granted: [
                    'bg-gradient-to-r from-green-400 to-teal-400',
                    SHADOW_GREEN_TEXT_WHITE,
                ],
                denied: [
                    'bg-gradient-to-r from-red-400 to-pink-400',
                    SHADOW_RED_TEXT_WHITE,
                ],
                inherited: [
                    'bg-gradient-to-r from-blue-400 to-cyan-400',
                    'text-white shadow-blue-200/50',
                ],

                // Billing status badges
                paid: [
                    'bg-gradient-to-r from-green-400 to-emerald-500',
                    SHADOW_GREEN_TEXT_WHITE,
                ],
                overdue: [
                    'bg-gradient-to-r from-red-500 to-orange-500',
                    SHADOW_RED_TEXT_WHITE,
                ],
                trial: [
                    'bg-gradient-to-r from-blue-400 to-indigo-400',
                    'text-white shadow-blue-200/50',
                ],
                enterprise: [
                    'bg-gradient-to-r from-purple-500 to-indigo-500',
                    'text-white shadow-purple-200/50',
                ],

                // General status badges
                success: [
                    'bg-gradient-to-r from-green-500 to-lime-400',
                    SHADOW_GREEN_TEXT_WHITE,
                ],
                warning: [
                    'bg-gradient-to-r from-amber-400 to-yellow-400',
                    'text-black shadow-amber-200/50',
                ],
                error: [
                    'bg-gradient-to-r from-red-500 to-rose-400',
                    SHADOW_RED_TEXT_WHITE,
                ],
                info: [
                    'bg-gradient-to-r from-blue-400 to-sky-400',
                    'text-white shadow-blue-200/50',
                ],

                // Neutral badge
                default: [
                    'bg-gradient-to-r from-gray-300 to-slate-300',
                    'dark:from-gray-600 dark:to-slate-600',
                    'text-gray-700 dark:text-gray-200',
                ],
            },

            size: {
                sm: 'px-2 py-1 text-xs min-h-[24px]',
                default: 'px-3 py-1.5 text-xs min-h-[28px]',
                lg: 'px-4 py-2 text-sm min-h-[32px]',
            },

            interactive: {
                true: 'cursor-pointer hover:brightness-110',
                false: '',
            },
        },

        defaultVariants: {
            variant: 'default',
            size: 'default',
            interactive: false,
        },
    }
);

export type BadgeVariants = VariantProps<typeof badgeVariants>;

// ============================================================================
// INPUT VARIANTS
// ============================================================================

/**
 * Form input variants
 */
export const inputVariants = cva(
    [
        'flex w-full rounded-lg border',
        'bg-white dark:bg-gray-800',
        'placeholder:text-gray-500 dark:placeholder:text-gray-400',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2',
        'disabled:cursor-not-allowed disabled:opacity-50',
    ],
    {
        variants: {
            variant: {
                default: [
                    'border-gray-300 dark:border-gray-600',
                    'focus-visible:border-blue-500 focus-visible:ring-blue-500',
                ],
                success: [
                    'border-green-300 dark:border-green-600',
                    'focus-visible:border-green-500 focus-visible:ring-green-500',
                ],
                warning: [
                    'border-amber-300 dark:border-amber-600',
                    'focus-visible:border-amber-500 focus-visible:ring-amber-500',
                ],
                error: [
                    'border-red-300 dark:border-red-600',
                    'focus-visible:border-red-500 focus-visible:ring-red-500',
                ],
            },

            size: {
                sm: 'h-8 px-3 text-sm',
                default: 'h-10 px-3 py-2',
                lg: 'h-12 px-4 py-3 text-lg',
            },
        },

        defaultVariants: {
            variant: 'default',
            size: 'default',
        },
    }
);

export type InputVariants = VariantProps<typeof inputVariants>;

// ============================================================================
// TABLE VARIANTS
// ============================================================================

/**
 * Enhanced table variants
 */
export const tableVariants = cva(['w-full border-collapse'], {
    variants: {
        variant: {
            default: 'border-spacing-0',
            striped: 'border-spacing-0',
            bordered: 'border border-gray-200 dark:border-gray-700',
        },

        size: {
            sm: '[&_th]:px-3 [&_th]:py-2 [&_td]:px-3 [&_td]:py-2 text-sm',
            default: '[&_th]:px-4 [&_th]:py-3 [&_td]:px-4 [&_td]:py-3',
            lg: '[&_th]:px-6 [&_th]:py-4 [&_td]:px-6 [&_td]:py-4 text-base',
        },

        hover: {
            true: '[&_tbody_tr]:hover:bg-gray-50 [&_tbody_tr]:dark:hover:bg-gray-800/50',
            false: '',
        },
    },

    defaultVariants: {
        variant: 'default',
        size: 'default',
        hover: true,
    },
});

export type TableVariants = VariantProps<typeof tableVariants>;

// ============================================================================
// MODAL VARIANTS
// ============================================================================

/**
 * Modal variants
 */
export const modalVariants = cva(
    [
        'relative bg-white dark:bg-gray-800 rounded-2xl shadow-2xl',
        'border border-gray-200 dark:border-gray-700',
        'backdrop-blur-sm',
    ],
    {
        variants: {
            size: {
                sm: 'max-w-md',
                default: 'max-w-lg',
                lg: 'max-w-2xl',
                xl: 'max-w-4xl',
                full: 'max-w-7xl',
            },

            variant: {
                default: '',
                pancake: [
                    'bg-gradient-to-br from-white to-orange-50/50',
                    'dark:from-gray-800 dark:to-orange-900/10',
                    'border-orange-200/50 dark:border-orange-800/30',
                ],
                warning: [
                    'bg-gradient-to-br from-amber-50 to-amber-100/50',
                    'dark:from-amber-900/20 dark:to-amber-800/10',
                    'border-amber-300/50 dark:border-amber-700/30',
                ],
                error: [
                    'bg-gradient-to-br from-red-50 to-red-100/50',
                    'dark:from-red-900/20 dark:to-red-800/10',
                    'border-red-300/50 dark:border-red-700/30',
                ],
            },
        },

        defaultVariants: {
            size: 'default',
            variant: 'default',
        },
    }
);

export type ModalVariants = VariantProps<typeof modalVariants>;

// ============================================================================
// LOADING VARIANTS
// ============================================================================

/**
 * Loading state variants
 */
export const loadingVariants = cva(
    ['bg-gray-200 dark:bg-gray-700 rounded opacity-75'],
    {
        variants: {
            variant: {
                text: 'h-4',
                heading: 'h-6',
                button: 'h-10',
                card: 'h-32',
                avatar: 'h-10 w-10 rounded-full',
                table: 'h-8',
            },

            width: {
                full: 'w-full',
                '3/4': 'w-3/4',
                '1/2': 'w-1/2',
                '1/4': 'w-1/4',
                auto: 'w-auto',
            },
        },

        defaultVariants: {
            variant: 'text',
            width: 'full',
        },
    }
);

export type LoadingVariants = VariantProps<typeof loadingVariants>;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Generate status-specific badge variant
 */
export function getStatusBadgeVariant(
    status: string
): BadgeVariants['variant'] {
    const statusMap: Record<string, BadgeVariants['variant']> = {
        active: 'active',
        inactive: 'inactive',
        pending: 'pending',
        suspended: 'suspended',
        premium: 'premium',
        granted: 'granted',
        denied: 'denied',
        inherited: 'inherited',
        paid: 'paid',
        overdue: 'overdue',
        trial: 'trial',
        enterprise: 'enterprise',
    };

    return statusMap[status.toLowerCase()] ?? 'default';
}

/**
 * Generate action-specific button variant
 */
export function getActionButtonVariant(
    action: string
): ButtonVariants['variant'] {
    const actionMap: Record<string, ButtonVariants['variant']> = {
        create: 'primary',
        save: 'primary',
        confirm: 'primary',
        approve: 'success',
        activate: 'success',
        delete: 'destructive',
        suspend: 'destructive',
        revoke: 'destructive',
        pending: 'warning',
        trial: 'warning',
        cancel: 'outline',
        back: 'ghost',
    };

    return actionMap[action.toLowerCase()] ?? 'primary';
}
