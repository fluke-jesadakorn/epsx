/**
 * Admin Frontend Component Variants (CVA)
 * 
 * Type-safe component variants using Class Variance Authority (CVA).
 * Provides consistent styling patterns for admin-specific components.
 * 
 * Features:
 * - Type-safe component styling with IntelliSense
 * - Composable variants for flexible design
 * - Admin-specific component patterns
 * - Tree-shakeable (only used variants are bundled)
 */

import { cva, type VariantProps } from 'class-variance-authority';

// ============================================================================
// BUTTON VARIANTS
// ============================================================================

/**
 * Enhanced button variants for admin interfaces
 */
export const adminButtonVariants = cva([
  'inline-flex items-center justify-center whitespace-nowrap',
  'rounded-lg font-semibold text-sm transition-all duration-300',
  'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2',
  'disabled:pointer-events-none disabled:opacity-50',
  'relative overflow-hidden',
], {
  variants: {
    variant: {
      // Primary actions (save, create, confirm)
      primary: [
        'bg-gradient-to-r from-orange-500 to-yellow-500',
        'text-white shadow-lg hover:shadow-xl',
        'hover:scale-105 active:scale-95',
        'focus-visible:ring-orange-500',
      ],
      
      // Secondary actions (cancel, back)
      secondary: [
        'bg-gradient-to-r from-blue-500 to-purple-500',
        'text-white shadow-lg hover:shadow-xl',
        'hover:scale-105 active:scale-95',
        'focus-visible:ring-blue-500',
      ],
      
      // Success actions (approve, activate)
      success: [
        'bg-gradient-to-r from-green-500 to-emerald-500',
        'text-white shadow-lg hover:shadow-xl',
        'hover:scale-105 active:scale-95',
        'focus-visible:ring-green-500',
      ],
      
      // Destructive actions (delete, suspend, revoke)
      destructive: [
        'bg-gradient-to-r from-red-500 to-rose-500',
        'text-white shadow-lg hover:shadow-xl',
        'hover:scale-105 active:scale-95',
        'focus-visible:ring-red-500',
      ],
      
      // Warning actions (pending, trial)
      warning: [
        'bg-gradient-to-r from-amber-500 to-orange-500',
        'text-white shadow-lg hover:shadow-xl',
        'hover:scale-105 active:scale-95',
        'focus-visible:ring-amber-500',
      ],
      
      // Outline variants
      outline: [
        'border-2 border-gray-300 bg-white text-gray-700',
        'hover:bg-gray-50 hover:border-gray-400',
        'focus-visible:ring-gray-500',
      ],
      
      // Ghost variants
      ghost: [
        'text-gray-700 hover:bg-gray-100',
        'focus-visible:ring-gray-500',
      ],
      
      // Link style
      link: [
        'text-orange-600 underline-offset-4 hover:underline',
        'focus-visible:ring-orange-500',
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
});

export type AdminButtonVariants = VariantProps<typeof adminButtonVariants>;

// ============================================================================
// CARD VARIANTS
// ============================================================================

/**
 * Enhanced card variants for admin dashboards
 */
export const adminCardVariants = cva([
  'rounded-2xl border backdrop-blur-sm transition-all duration-300',
  'relative overflow-hidden',
], {
  variants: {
    variant: {
      // Standard dashboard card
      default: [
        'bg-white/90 dark:bg-gray-800/90 border-gray-200 dark:border-gray-700',
        'shadow-sm hover:shadow-md',
      ],
      
      // Enhanced PancakeSwap-style card
      pancake: [
        'bg-gradient-to-br from-white to-orange-50/30',
        'dark:from-gray-800 dark:to-orange-900/10',
        'border-orange-200/50 dark:border-orange-800/30',
        'shadow-lg hover:shadow-xl',
      ],
      
      // User management card
      user: [
        'bg-gradient-to-br from-white to-blue-50/30',
        'dark:from-gray-800 dark:to-blue-900/10',
        'border-blue-200/50 dark:border-blue-800/30',
        'shadow-md hover:shadow-lg',
      ],
      
      // Permission card
      permission: [
        'bg-gradient-to-br from-white to-purple-50/30',
        'dark:from-gray-800 dark:to-purple-900/10',
        'border-purple-200/50 dark:border-purple-800/30',
        'shadow-md hover:shadow-lg',
      ],
      
      // Billing card
      billing: [
        'bg-gradient-to-br from-white to-green-50/30',
        'dark:from-gray-800 dark:to-green-900/10',
        'border-green-200/50 dark:border-green-800/30',
        'shadow-md hover:shadow-lg',
      ],
      
      // Analytics card
      analytics: [
        'bg-gradient-to-br from-white to-indigo-50/30',
        'dark:from-gray-800 dark:to-indigo-900/10',
        'border-indigo-200/50 dark:border-indigo-800/30',
        'shadow-md hover:shadow-lg',
      ],
      
      // Warning/alert card
      warning: [
        'bg-gradient-to-br from-amber-50 to-amber-100/50',
        'dark:from-amber-900/20 dark:to-amber-800/10',
        'border-amber-300/50 dark:border-amber-700/30',
        'shadow-md',
      ],
      
      // Error card
      error: [
        'bg-gradient-to-br from-red-50 to-red-100/50',
        'dark:from-red-900/20 dark:to-red-800/10',
        'border-red-300/50 dark:border-red-700/30',
        'shadow-md',
      ],
    },
    
    hover: {
      none: '',
      lift: 'hover:-translate-y-1 hover:scale-[1.02]',
      glow: 'hover:shadow-xl',
      both: 'hover:-translate-y-1 hover:scale-[1.02] hover:shadow-xl',
    },
    
    padding: {
      none: 'p-0',
      sm: 'p-3',
      default: 'p-4',
      md: 'p-6',
      lg: 'p-8',
    },
    
    interactive: {
      true: 'cursor-pointer',
      false: '',
    },
  },
  
  defaultVariants: {
    variant: 'default',
    hover: 'both',
    padding: 'default',
    interactive: false,
  },
});

export type AdminCardVariants = VariantProps<typeof adminCardVariants>;

// ============================================================================
// BADGE VARIANTS
// ============================================================================

/**
 * Status badge variants for admin interfaces
 */
export const adminBadgeVariants = cva([
  'inline-flex items-center rounded-full px-2.5 py-0.5',
  'text-xs font-medium transition-colors',
], {
  variants: {
    variant: {
      // User status badges
      active: 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400',
      inactive: 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-300',
      pending: 'bg-amber-100 text-amber-800 dark:bg-amber-900/20 dark:text-amber-400',
      suspended: 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400',
      premium: 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-400',
      
      // Permission status badges
      granted: 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400',
      denied: 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400',
      inherited: 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400',
      
      // Billing status badges
      paid: 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400',
      overdue: 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400',
      trial: 'bg-amber-100 text-amber-800 dark:bg-amber-900/20 dark:text-amber-400',
      enterprise: 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-400',
      
      // General status badges
      success: 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400',
      warning: 'bg-amber-100 text-amber-800 dark:bg-amber-900/20 dark:text-amber-400',
      error: 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400',
      info: 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400',
      
      // Neutral badge
      default: 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-300',
    },
    
    size: {
      sm: 'px-2 py-0.5 text-xs',
      default: 'px-2.5 py-0.5 text-xs',
      lg: 'px-3 py-1 text-sm',
    },
    
    dot: {
      true: 'pl-1.5',
      false: '',
    },
  },
  
  defaultVariants: {
    variant: 'default',
    size: 'default',
    dot: false,
  },
});

export type AdminBadgeVariants = VariantProps<typeof adminBadgeVariants>;

// ============================================================================
// TABLE VARIANTS
// ============================================================================

/**
 * Enhanced table variants for admin data displays
 */
export const adminTableVariants = cva([
  'w-full border-collapse',
], {
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
      true: '[&_tbody_tr]:hover:bg-gray-50 [&_tbody_tr]:dark:hover:bg-gray-800/50 [&_tbody_tr]:transition-colors',
      false: '',
    },
  },
  
  defaultVariants: {
    variant: 'default',
    size: 'default',
    hover: true,
  },
});

export type AdminTableVariants = VariantProps<typeof adminTableVariants>;

// ============================================================================
// INPUT VARIANTS
// ============================================================================

/**
 * Form input variants for admin interfaces
 */
export const adminInputVariants = cva([
  'flex w-full rounded-lg border transition-colors',
  'bg-white dark:bg-gray-800',
  'placeholder:text-gray-500 dark:placeholder:text-gray-400',
  'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2',
  'disabled:cursor-not-allowed disabled:opacity-50',
], {
  variants: {
    variant: {
      default: [
        'border-gray-300 dark:border-gray-600',
        'focus-visible:border-orange-500 focus-visible:ring-orange-500',
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
});

export type AdminInputVariants = VariantProps<typeof adminInputVariants>;

// ============================================================================
// MODAL VARIANTS
// ============================================================================

/**
 * Modal variants for admin interfaces
 */
export const adminModalVariants = cva([
  'relative bg-white dark:bg-gray-800 rounded-2xl shadow-2xl',
  'border border-gray-200 dark:border-gray-700',
  'backdrop-blur-sm',
], {
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
});

export type AdminModalVariants = VariantProps<typeof adminModalVariants>;

// ============================================================================
// LOADING VARIANTS
// ============================================================================

/**
 * Loading state variants for admin components
 */
export const adminLoadingVariants = cva([
  'animate-pulse bg-gray-200 dark:bg-gray-700 rounded',
], {
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
});

export type AdminLoadingVariants = VariantProps<typeof adminLoadingVariants>;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Helper function to combine multiple variant classes
 */
export function cn(...classes: (string | undefined | null | false)[]): string {
  return classes.filter(Boolean).join(' ');
}

/**
 * Generate status-specific badge variant
 */
export function getStatusBadgeVariant(status: string): AdminBadgeVariants['variant'] {
  const statusMap: Record<string, AdminBadgeVariants['variant']> = {
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
  
  return statusMap[status.toLowerCase()] || 'default';
}

/**
 * Generate action-specific button variant
 */
export function getActionButtonVariant(action: string): AdminButtonVariants['variant'] {
  const actionMap: Record<string, AdminButtonVariants['variant']> = {
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
  
  return actionMap[action.toLowerCase()] || 'primary';
}