/**
 * Component Variants using Class Variance Authority (CVA)
 * 
 * This file provides type-safe, composable component variants that replace
 * the existing custom CSS classes (like btn-pancake, card-pancake, etc.)
 * 
 * Benefits:
 * - Type safety with IntelliSense
 * - Composable variants and compound variants  
 * - Better performance (only needed classes)
 * - Consistent API across components
 * - Easy to extend and modify
 */

import { type VariantProps, cva } from 'class-variance-authority';
import { cn } from '@/lib/utils';

// ============================================================================
// BUTTON VARIANTS
// ============================================================================

/**
 * Button component variants - replaces btn-pancake-* classes
 */
export const buttonVariants = cva(
  // Base styles for all buttons
  [
    'inline-flex items-center justify-center whitespace-nowrap',
    'rounded-xl font-semibold text-sm transition-all duration-300',
    'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2',
    'disabled:pointer-events-none disabled:opacity-50',
    'transform-gpu', // Use GPU acceleration
  ],
  {
    variants: {
      variant: {
        // Primary PancakeSwap-style button
        primary: [
          'bg-gradient-to-r from-orange-500 to-yellow-500',
          'text-white shadow-lg',
          'hover:from-orange-600 hover:to-yellow-600',
          'hover:shadow-xl hover:scale-105',
          'active:scale-95',
          'focus-visible:ring-orange-500',
        ],
        
        // Secondary button
        secondary: [
          'bg-gradient-to-r from-blue-500 to-cyan-500',
          'text-white shadow-lg',
          'hover:from-blue-600 hover:to-cyan-600',
          'hover:shadow-xl hover:scale-105',
          'active:scale-95',
          'focus-visible:ring-blue-500',
        ],
        
        // Success button
        success: [
          'bg-gradient-to-r from-green-500 to-emerald-500',
          'text-white shadow-lg',
          'hover:from-green-600 hover:to-emerald-600',
          'hover:shadow-xl hover:scale-105',
          'active:scale-95',
          'focus-visible:ring-green-500',
        ],
        
        // Warning button
        warning: [
          'bg-gradient-to-r from-yellow-500 to-orange-500',
          'text-white shadow-lg',
          'hover:from-yellow-600 hover:to-orange-600',
          'hover:shadow-xl hover:scale-105',
          'active:scale-95',
          'focus-visible:ring-yellow-500',
        ],
        
        // Destructive button
        destructive: [
          'bg-gradient-to-r from-red-500 to-pink-500',
          'text-white shadow-lg',
          'hover:from-red-600 hover:to-pink-600',
          'hover:shadow-xl hover:scale-105',
          'active:scale-95',
          'focus-visible:ring-red-500',
        ],
        
        // Outline button
        outline: [
          'border-2 border-orange-500 bg-transparent',
          'text-orange-600 dark:text-orange-400',
          'hover:bg-orange-50 dark:hover:bg-orange-950',
          'hover:scale-105',
          'active:scale-95',
          'focus-visible:ring-orange-500',
        ],
        
        // Ghost button
        ghost: [
          'bg-transparent text-gray-700 dark:text-gray-300',
          'hover:bg-gray-100 dark:hover:bg-gray-800',
          'hover:scale-105',
          'active:scale-95',
          'focus-visible:ring-gray-500',
        ],
        
        // Link style button
        link: [
          'bg-transparent text-orange-600 dark:text-orange-400',
          'underline-offset-4 hover:underline',
          'hover:scale-105',
          'active:scale-95',
          'focus-visible:ring-orange-500',
        ],
      },
      
      size: {
        sm: 'h-8 px-3 text-xs',
        md: 'h-10 px-4 text-sm',
        lg: 'h-12 px-6 text-base',
        xl: 'h-14 px-8 text-lg',
        icon: 'h-10 w-10',
      },
      
      glow: {
        true: 'shadow-2xl',
        false: '',
      },
      
      rounded: {
        sm: 'rounded-lg',
        md: 'rounded-xl',
        lg: 'rounded-2xl',
        full: 'rounded-full',
      },
    },
    
    defaultVariants: {
      variant: 'primary',
      size: 'md',
      glow: false,
      rounded: 'md',
    },
    
    compoundVariants: [
      // Glowing effect for primary buttons
      {
        variant: 'primary',
        glow: true,
        class: 'shadow-orange-500/25',
      },
      // Glowing effect for secondary buttons
      {
        variant: 'secondary',
        glow: true,
        class: 'shadow-blue-500/25',
      },
    ],
  }
);

export type ButtonVariants = VariantProps<typeof buttonVariants>;

// ============================================================================
// CARD VARIANTS
// ============================================================================

/**
 * Card component variants - replaces card-pancake-* classes
 */
export const cardVariants = cva(
  // Base styles for all cards
  [
    'relative overflow-hidden rounded-2xl border',
    'bg-white/80 dark:bg-gray-900/80',
    'backdrop-blur-md transition-all duration-300',
  ],
  {
    variants: {
      variant: {
        // Default card
        default: [
          'border-gray-200/50 dark:border-gray-700/50',
          'hover:shadow-lg hover:scale-[1.02]',
          'hover:border-orange-200/50 dark:hover:border-orange-700/50',
        ],
        
        // Enhanced PancakeSwap-style card
        pancake: [
          'border-orange-200/50 dark:border-orange-700/50',
          'bg-gradient-to-br from-white via-orange-50/20 to-yellow-50/30',
          'dark:from-gray-900 dark:via-gray-800 dark:to-orange-900/20',
          'hover:shadow-xl hover:scale-[1.02] hover:-translate-y-1',
          'hover:border-orange-300/50 dark:hover:border-orange-600/50',
        ],
        
        // Glassmorphism card
        glass: [
          'border-white/20 dark:border-gray-700/20',
          'bg-white/10 dark:bg-gray-900/10',
          'backdrop-blur-xl',
          'hover:bg-white/20 dark:hover:bg-gray-900/20',
          'hover:scale-[1.02]',
        ],
        
        // Elevated card
        elevated: [
          'border-gray-200/50 dark:border-gray-700/50',
          'shadow-lg hover:shadow-2xl',
          'hover:scale-[1.02] hover:-translate-y-2',
        ],
        
        // Flat card (no elevation)
        flat: [
          'border-gray-200 dark:border-gray-700',
          'bg-white dark:bg-gray-900',
          'hover:border-orange-200 dark:hover:border-orange-700',
        ],
      },
      
      padding: {
        none: 'p-0',
        sm: 'p-4',
        md: 'p-6',
        lg: 'p-8',
        xl: 'p-10',
      },
      
      glow: {
        true: 'shadow-2xl shadow-orange-500/10',
        false: '',
      },
      
      interactive: {
        true: 'cursor-pointer',
        false: '',
      },
    },
    
    defaultVariants: {
      variant: 'default',
      padding: 'md',
      glow: false,
      interactive: false,
    },
    
    compoundVariants: [
      // Interactive + Pancake variant
      {
        variant: 'pancake',
        interactive: true,
        class: 'hover:shadow-orange-500/20',
      },
      // Glow + Pancake variant
      {
        variant: 'pancake',
        glow: true,
        class: 'shadow-orange-500/15',
      },
    ],
  }
);

export type CardVariants = VariantProps<typeof cardVariants>;

// ============================================================================
// BADGE VARIANTS
// ============================================================================

/**
 * Badge component variants
 */
export const badgeVariants = cva(
  [
    'inline-flex items-center rounded-full border px-2.5 py-0.5',
    'text-xs font-semibold transition-colors',
    'focus:outline-none focus:ring-2 focus:ring-offset-2',
  ],
  {
    variants: {
      variant: {
        primary: [
          'border-transparent bg-gradient-to-r from-orange-500 to-yellow-500',
          'text-white shadow-sm',
        ],
        secondary: [
          'border-transparent bg-gray-100 dark:bg-gray-800',
          'text-gray-900 dark:text-gray-100',
        ],
        success: [
          'border-transparent bg-green-100 dark:bg-green-900',
          'text-green-800 dark:text-green-200',
        ],
        warning: [
          'border-transparent bg-yellow-100 dark:bg-yellow-900',
          'text-yellow-800 dark:text-yellow-200',
        ],
        destructive: [
          'border-transparent bg-red-100 dark:bg-red-900',
          'text-red-800 dark:text-red-200',
        ],
        outline: [
          'border-gray-200 dark:border-gray-700',
          'text-gray-900 dark:text-gray-100',
        ],
      },
      
      size: {
        sm: 'px-2 py-0.5 text-xs',
        md: 'px-2.5 py-0.5 text-xs',
        lg: 'px-3 py-1 text-sm',
      },
    },
    
    defaultVariants: {
      variant: 'primary',
      size: 'md',
    },
  }
);

export type BadgeVariants = VariantProps<typeof badgeVariants>;

// ============================================================================
// GRADIENT TEXT VARIANTS
// ============================================================================

/**
 * Gradient text variants - replaces pancake-gradient-text classes
 */
export const gradientTextVariants = cva(
  [
    'bg-clip-text text-transparent',
    'animate-gradient bg-[length:200%_200%]',
  ],
  {
    variants: {
      gradient: {
        primary: 'bg-gradient-to-r from-orange-500 via-yellow-400 to-orange-600',
        secondary: 'bg-gradient-to-r from-blue-500 via-cyan-400 to-teal-500',
        success: 'bg-gradient-to-r from-green-500 via-emerald-400 to-green-600',
        rainbow: 'bg-gradient-to-r from-purple-500 via-pink-500 to-red-500',
        sunset: 'bg-gradient-to-r from-orange-400 via-red-500 to-pink-500',
      },
      
      animation: {
        none: '',
        slow: 'animate-gradient-slow',
        normal: 'animate-gradient',
        fast: 'animate-gradient-fast',
      },
    },
    
    defaultVariants: {
      gradient: 'primary',
      animation: 'normal',
    },
  }
);

export type GradientTextVariants = VariantProps<typeof gradientTextVariants>;

// ============================================================================
// ANIMATION VARIANTS
// ============================================================================

/**
 * Animation utility variants
 */
export const animationVariants = cva('', {
  variants: {
    float: {
      none: '',
      gentle: 'animate-float-gentle',
      normal: 'animate-float',
      reverse: 'animate-float-reverse',
    },
    
    bounce: {
      none: '',
      gentle: 'animate-bounce-gentle',
      normal: 'animate-bounce',
      slow: 'animate-bounce-slow',
    },
    
    pulse: {
      none: '',
      gentle: 'animate-pulse-gentle',
      normal: 'animate-pulse',
      slow: 'animate-pulse-slow',
    },
    
    scale: {
      none: '',
      hover: 'hover:scale-105',
      press: 'active:scale-95',
      pancake: 'hover:scale-105 active:scale-95',
    },
    
    spin: {
      none: '',
      normal: 'animate-spin',
      slow: 'animate-spin-slow',
    },
  },
  
  defaultVariants: {
    float: 'none',
    bounce: 'none',
    pulse: 'none',
    scale: 'none',
    spin: 'none',
  },
});

export type AnimationVariants = VariantProps<typeof animationVariants>;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Utility function to combine button variants with additional classes
 */
export function createButtonClass(
  variants?: ButtonVariants,
  className?: string
) {
  return cn(buttonVariants(variants), className);
}

/**
 * Utility function to combine card variants with additional classes
 */
export function createCardClass(
  variants?: CardVariants,
  className?: string
) {
  return cn(cardVariants(variants), className);
}

/**
 * Utility function to combine badge variants with additional classes
 */
export function createBadgeClass(
  variants?: BadgeVariants,
  className?: string
) {
  return cn(badgeVariants(variants), className);
}

/**
 * Utility function to combine gradient text variants with additional classes
 */
export function createGradientTextClass(
  variants?: GradientTextVariants,
  className?: string
) {
  return cn(gradientTextVariants(variants), className);
}

/**
 * Utility function to combine animation variants with additional classes
 */
export function createAnimationClass(
  variants?: AnimationVariants,
  className?: string
) {
  return cn(animationVariants(variants), className);
}

// ============================================================================
// EXPORTS
// ============================================================================

export {
  buttonVariants,
  cardVariants,
  badgeVariants,
  gradientTextVariants,
  animationVariants,
};

// Export all variant types
export type {
  ButtonVariants,
  CardVariants,
  BadgeVariants,
  GradientTextVariants,
  AnimationVariants,
};