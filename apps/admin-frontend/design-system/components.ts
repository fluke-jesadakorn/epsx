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
 * Enhanced card variants for admin dashboards - Windows Phone + PancakeSwap Design
 * Features live tile animations, gradients, and selection patterns
 */
export const adminCardVariants = cva([
  'rounded-2xl border backdrop-blur-sm transition-all duration-300',
  'relative overflow-hidden group',
], {
  variants: {
    variant: {
      // Standard dashboard card
      default: [
        'bg-white/90 dark:bg-gray-800/90 border-gray-200 dark:border-gray-700',
        'shadow-sm hover:shadow-md',
      ],
      
      // Enhanced PancakeSwap-style card with live tile features
      pancake: [
        'bg-gradient-to-br from-yellow-50 via-orange-50 to-yellow-100',
        'dark:from-gray-800 dark:via-orange-900/20 dark:to-gray-700',
        'border-yellow-200/60 dark:border-orange-800/40',
        'shadow-lg hover:shadow-2xl',
        'before:absolute before:top-0 before:right-0 before:w-6 before:h-6',
        'before:bg-gradient-to-bl before:from-yellow-400/60 before:to-transparent',
        'after:absolute after:bottom-2 after:right-2 after:w-1.5 after:h-1.5',
        'after:bg-yellow-400/80 after:rounded-full after:animate-pulse',
      ],
      
      // User management card with Windows Phone styling
      user: [
        'bg-gradient-to-br from-blue-50 via-indigo-50 to-blue-100',
        'dark:from-gray-800 dark:via-blue-900/20 dark:to-gray-700',
        'border-blue-200/60 dark:border-blue-800/40',
        'shadow-lg hover:shadow-2xl',
        'before:absolute before:top-0 before:right-0 before:w-6 before:h-6',
        'before:bg-gradient-to-bl before:from-blue-400/60 before:to-transparent',
        'after:absolute after:bottom-2 after:right-2 after:w-1.5 after:h-1.5',
        'after:bg-blue-400/80 after:rounded-full',
      ],
      
      // Permission card with live tile effects
      permission: [
        'bg-gradient-to-br from-purple-50 via-violet-50 to-purple-100',
        'dark:from-gray-800 dark:via-purple-900/20 dark:to-gray-700',
        'border-purple-200/60 dark:border-purple-800/40',
        'shadow-lg hover:shadow-2xl',
        'before:absolute before:top-0 before:right-0 before:w-6 before:h-6',
        'before:bg-gradient-to-bl before:from-purple-400/60 before:to-transparent',
        'after:absolute after:bottom-2 after:right-2 after:w-1.5 after:h-1.5',
        'after:bg-purple-400/80 after:rounded-full',
      ],
      
      // Billing card with tile aesthetics
      billing: [
        'bg-gradient-to-br from-green-50 via-emerald-50 to-green-100',
        'dark:from-gray-800 dark:via-green-900/20 dark:to-gray-700',
        'border-green-200/60 dark:border-green-800/40',
        'shadow-lg hover:shadow-2xl',
        'before:absolute before:top-0 before:right-0 before:w-6 before:h-6',
        'before:bg-gradient-to-bl before:from-green-400/60 before:to-transparent',
        'after:absolute after:bottom-2 after:right-2 after:w-1.5 after:h-1.5',
        'after:bg-green-400/80 after:rounded-full',
      ],
      
      // Analytics card with enhanced gradients
      analytics: [
        'bg-gradient-to-br from-indigo-50 via-cyan-50 to-indigo-100',
        'dark:from-gray-800 dark:via-indigo-900/20 dark:to-gray-700',
        'border-indigo-200/60 dark:border-indigo-800/40',
        'shadow-lg hover:shadow-2xl',
        'before:absolute before:top-0 before:right-0 before:w-6 before:h-6',
        'before:bg-gradient-to-bl before:from-indigo-400/60 before:to-transparent',
        'after:absolute after:bottom-2 after:right-2 after:w-1.5 after:h-1.5',
        'after:bg-indigo-400/80 after:rounded-full after:animate-pulse',
      ],
      
      // Live tile variant for Windows Phone style
      tile: [
        'bg-gradient-to-br from-gray-100 to-gray-200',
        'dark:from-gray-700 dark:to-gray-800',
        'border-0 shadow-inner',
        'before:absolute before:inset-0 before:bg-gradient-to-br',
        'before:from-transparent before:via-white/10 before:to-transparent',
        'after:absolute after:bottom-1 after:right-1 after:w-2 after:h-2',
        'after:bg-white/60 after:rounded-full',
      ],
      
      // Selected state variant
      selected: [
        'bg-gradient-to-br from-yellow-100 to-orange-100',
        'dark:from-yellow-900/30 dark:to-orange-900/30',
        'border-2 border-yellow-400 dark:border-yellow-500',
        'shadow-lg shadow-yellow-200/50 dark:shadow-yellow-900/30',
        'ring-2 ring-yellow-300/50 ring-offset-2',
        'before:absolute before:top-0 before:right-0 before:w-8 before:h-8',
        'before:bg-gradient-to-bl before:from-yellow-400 before:to-transparent',
        'after:absolute after:top-2 after:right-2 after:w-3 after:h-3',
        'after:bg-white after:rounded-full after:flex after:items-center after:justify-center',
      ],
      
      // Warning/alert card with enhanced styling
      warning: [
        'bg-gradient-to-br from-amber-50 via-yellow-50 to-amber-100',
        'dark:from-amber-900/20 dark:via-yellow-900/20 dark:to-amber-800/10',
        'border-amber-300/60 dark:border-amber-700/40',
        'shadow-lg shadow-amber-200/30 dark:shadow-amber-900/20',
      ],
      
      // Error card with enhanced styling
      error: [
        'bg-gradient-to-br from-red-50 via-pink-50 to-red-100',
        'dark:from-red-900/20 dark:via-pink-900/20 dark:to-red-800/10',
        'border-red-300/60 dark:border-red-700/40',
        'shadow-lg shadow-red-200/30 dark:shadow-red-900/20',
      ],
    },
    
    hover: {
      none: '',
      lift: 'hover:-translate-y-2 hover:rotate-[0.5deg] active:scale-[0.98]',
      glow: 'hover:shadow-2xl hover:shadow-current/10',
      both: 'hover:-translate-y-2 hover:scale-[1.02] hover:shadow-2xl hover:rotate-[0.5deg] active:scale-[0.98] active:rotate-0',
      flip: 'hover:perspective-1000 hover:transform-style-preserve-3d hover:rotate-y-3 active:rotate-y-0',
      scale: 'hover:scale-[1.05] active:scale-[0.95] hover:rotate-1 active:rotate-0',
      intense: 'hover:-translate-y-3 hover:scale-[1.03] hover:shadow-2xl hover:rotate-1 hover:brightness-105 active:scale-[0.97] active:rotate-0 active:brightness-100',
    },
    
    animation: {
      none: '',
      subtle: 'animate-in fade-in-50 slide-in-from-bottom-4 duration-300',
      bounce: 'animate-in fade-in-0 zoom-in-95 duration-500',
      slide: 'animate-in fade-in-0 slide-in-from-left-8 duration-400',
      flip: 'animate-in fade-in-0 duration-500 [animation-fill-mode:both]',
      pulse: 'animate-pulse-subtle',
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
    
    selectable: {
      true: 'group-hover:ring-2 group-hover:ring-yellow-300/50 group-hover:ring-offset-1 transition-all',
      false: '',
    },
    
    size: {
      sm: 'min-h-[120px]',
      default: 'min-h-[140px]',
      md: 'min-h-[160px]',  
      lg: 'min-h-[180px]',
      xl: 'min-h-[220px]',
      tile: 'min-h-[160px] aspect-square', // Windows Phone live tile proportions
    },
  },
  
  defaultVariants: {
    variant: 'default',
    hover: 'both',
    animation: 'subtle',
    padding: 'default',
    interactive: false,
    selectable: false,
    size: 'default',
  },
});

export type AdminCardVariants = VariantProps<typeof adminCardVariants>;

// ============================================================================
// BADGE VARIANTS
// ============================================================================

/**
 * Windows Phone + PancakeSwap status badge variants for admin interfaces
 * Features live tile aesthetics with gradients and animations
 */
export const adminBadgeVariants = cva([
  'inline-flex items-center rounded-xl px-3 py-1.5 relative overflow-hidden',
  'text-xs font-light uppercase tracking-wider transition-all duration-300',
  'shadow-sm hover:shadow-md hover:scale-105 active:scale-95',
  'border border-transparent min-h-[28px]',
], {
  variants: {
    variant: {
      // User status badges with Windows Phone gradients
      active: [
        'bg-gradient-to-r from-green-400 to-emerald-400',
        'text-white shadow-green-200/50',
        'after:absolute after:bottom-1 after:right-1 after:w-1 after:h-1',
        'after:bg-white/80 after:rounded-full after:animate-pulse',
      ],
      inactive: [
        'bg-gradient-to-r from-gray-300 to-slate-300',
        'dark:from-gray-600 dark:to-slate-600',
        'text-gray-700 dark:text-gray-200',
        'border-gray-400/30 dark:border-gray-500/30',
      ],
      pending: [
        'bg-gradient-to-r from-yellow-400 to-orange-400',
        'text-black shadow-yellow-200/50',
        'after:absolute after:bottom-1 after:right-1 after:w-1 after:h-1',
        'after:bg-black/60 after:rounded-full after:animate-pulse',
      ],
      suspended: [
        'bg-gradient-to-r from-red-400 to-orange-500',
        'text-white shadow-red-200/50',
        'after:absolute after:top-1 after:right-1 after:w-1 after:h-1',
        'after:bg-white/80 after:rounded-full',
      ],
      premium: [
        'bg-gradient-to-r from-purple-400 to-pink-400',
        'text-white shadow-purple-200/50',
        'after:absolute after:bottom-1 after:right-1 after:w-1 after:h-1',
        'after:bg-white/80 after:rounded-full after:animate-pulse',
        'before:absolute before:top-0 before:right-0 before:w-3 before:h-3',
        'before:bg-gradient-to-bl before:from-white/30 before:to-transparent',
      ],
      
      // Permission status badges with Windows Phone styling
      granted: [
        'bg-gradient-to-r from-green-400 to-teal-400',
        'text-white shadow-green-200/50',
        'after:absolute after:bottom-1 after:right-1 after:w-1 after:h-1',
        'after:bg-white/80 after:rounded-full after:animate-pulse',
      ],
      denied: [
        'bg-gradient-to-r from-red-400 to-pink-400',
        'text-white shadow-red-200/50',
        'after:absolute after:top-1 after:right-1 after:w-1 after:h-1',
        'after:bg-white/80 after:rounded-full',
      ],
      inherited: [
        'bg-gradient-to-r from-blue-400 to-cyan-400',
        'text-white shadow-blue-200/50',
        'after:absolute after:bottom-1 after:right-1 after:w-1 after:h-1',
        'after:bg-white/80 after:rounded-full after:animate-pulse',
      ],
      temporary: [
        'bg-gradient-to-r from-yellow-400 to-amber-400',
        'text-black shadow-yellow-200/50',
        'after:absolute after:bottom-1 after:right-1 after:w-1 after:h-1',
        'after:bg-black/60 after:rounded-full after:animate-pulse',
      ],
      expired: [
        'bg-gradient-to-r from-gray-400 to-red-400',
        'text-white shadow-gray-200/50',
        'after:absolute after:top-1 after:right-1 after:w-1 after:h-1',
        'after:bg-white/80 after:rounded-full',
        'opacity-75',
      ],
      
      // Billing status badges with PancakeSwap colors
      paid: [
        'bg-gradient-to-r from-green-400 to-emerald-500',
        'text-white shadow-green-200/50',
        'after:absolute after:bottom-1 after:right-1 after:w-1 after:h-1',
        'after:bg-white/80 after:rounded-full after:animate-pulse',
      ],
      overdue: [
        'bg-gradient-to-r from-red-500 to-orange-500',
        'text-white shadow-red-200/50',
        'after:absolute after:top-1 after:right-1 after:w-1.5 after:h-1.5',
        'after:bg-white/80 after:rounded-full after:animate-pulse',
      ],
      trial: [
        'bg-gradient-to-r from-blue-400 to-indigo-400',
        'text-white shadow-blue-200/50',
        'after:absolute after:bottom-1 after:right-1 after:w-1 after:h-1',
        'after:bg-white/80 after:rounded-full after:animate-pulse',
      ],
      enterprise: [
        'bg-gradient-to-r from-purple-500 to-indigo-500',
        'text-white shadow-purple-200/50',
        'after:absolute after:bottom-1 after:right-1 after:w-1.5 after:h-1.5',
        'after:bg-white/80 after:rounded-full after:animate-pulse',
        'before:absolute before:top-0 before:right-0 before:w-4 before:h-4',
        'before:bg-gradient-to-bl before:from-white/30 before:to-transparent',
      ],
      
      // General status badges with enhanced gradients
      success: [
        'bg-gradient-to-r from-green-500 to-lime-400',
        'text-white shadow-green-200/50',
        'after:absolute after:bottom-1 after:right-1 after:w-1.5 after:h-1.5',
        'after:bg-white/90 after:rounded-full after:animate-pulse',
      ],
      warning: [
        'bg-gradient-to-r from-amber-400 to-yellow-400',
        'text-black shadow-amber-200/50',
        'after:absolute after:bottom-1 after:right-1 after:w-1 after:h-1',
        'after:bg-black/70 after:rounded-full after:animate-pulse',
      ],
      error: [
        'bg-gradient-to-r from-red-500 to-rose-400',
        'text-white shadow-red-200/50',
        'after:absolute after:top-1 after:right-1 after:w-1.5 after:h-1.5',
        'after:bg-white/90 after:rounded-full',
      ],
      info: [
        'bg-gradient-to-r from-blue-400 to-sky-400',
        'text-white shadow-blue-200/50',
        'after:absolute after:bottom-1 after:right-1 after:w-1 after:h-1',
        'after:bg-white/80 after:rounded-full after:animate-pulse',
      ],
      
      // Neutral badge with Windows Phone style
      default: [
        'bg-gradient-to-r from-gray-300 to-slate-300',
        'dark:from-gray-600 dark:to-slate-600',
        'text-gray-700 dark:text-gray-200',
        'border-gray-400/30 dark:border-gray-500/30',
        'after:absolute after:bottom-1 after:right-1 after:w-1 after:h-1',
        'after:bg-current after:opacity-40 after:rounded-full',
      ],
    },
    
    size: {
      sm: 'px-2 py-1 text-xs min-h-[24px]',
      default: 'px-3 py-1.5 text-xs min-h-[28px]',
      lg: 'px-4 py-2 text-sm min-h-[32px]',
    },
    
    dot: {
      true: 'pl-2 [&>*:first-child]:mr-2',
      false: '',
    },
    
    animation: {
      none: '',
      pulse: 'animate-pulse',
      bounce: 'animate-bounce',
      subtle: 'hover:animate-pulse',
    },
    
    interactive: {
      true: 'cursor-pointer hover:scale-110 active:scale-90',
      false: '',
    },
  },
  
  defaultVariants: {
    variant: 'default',
    size: 'default',
    dot: false,
    animation: 'none',
    interactive: false,
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