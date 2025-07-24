/**
 * Responsive Design Utilities
 * Mobile-first responsive design utilities for EPSX design system
 */

import { designTokens } from '../tokens/design-tokens';
import { themeConfig } from '../tokens/theme-config';

// Breakpoint utilities
export const breakpoints = {
  xs: parseInt(designTokens.screens.xs),
  sm: parseInt(designTokens.screens.sm),
  md: parseInt(designTokens.screens.md),
  lg: parseInt(designTokens.screens.lg),
  xl: parseInt(designTokens.screens.xl),
  '2xl': parseInt(designTokens.screens['2xl']),
  '3xl': parseInt(designTokens.screens['3xl']),
} as const;

export type Breakpoint = keyof typeof breakpoints;

// Responsive class generator
export function generateResponsiveClasses(
  classes: Partial<Record<Breakpoint | 'base', string>>
): string {
  const { base, ...responsiveClasses } = classes;
  let result = base || '';
  
  Object.entries(responsiveClasses).forEach(([breakpoint, className]) => {
    if (className) {
      result += ` ${breakpoint}:${className}`;
    }
  });
  
  return result.trim();
}

// Responsive value selector
export function getResponsiveValue<T>(
  values: Partial<Record<Breakpoint | 'base', T>>,
  currentBreakpoint: Breakpoint | 'base' = 'base'
): T | undefined {
  // Return exact match if available
  if (values[currentBreakpoint]) {
    return values[currentBreakpoint];
  }
  
  // Fallback logic: find the largest breakpoint smaller than current
  const orderedBreakpoints: (Breakpoint | 'base')[] = ['base', 'xs', 'sm', 'md', 'lg', 'xl', '2xl', '3xl'];
  const currentIndex = orderedBreakpoints.indexOf(currentBreakpoint);
  
  for (let i = currentIndex - 1; i >= 0; i--) {
    const bp = orderedBreakpoints[i];
    if (bp && values[bp]) {
      return values[bp];
    }
  }
  
  return undefined;
}

// Media query utilities
export function createMediaQuery(breakpoint: Breakpoint): string {
  return `(min-width: ${designTokens.screens[breakpoint]})`;
}

export function useMediaQuery(query: string): boolean {
  if (typeof window === 'undefined') return false;
  
  const mediaQuery = window.matchMedia(query);
  return mediaQuery.matches;
}

export function useBreakpoint(): Breakpoint | 'base' {
  if (typeof window === 'undefined') return 'base';
  
  const width = window.innerWidth;
  
  if (width >= breakpoints['3xl']) return '3xl';
  if (width >= breakpoints['2xl']) return '2xl';
  if (width >= breakpoints.xl) return 'xl';
  if (width >= breakpoints.lg) return 'lg';
  if (width >= breakpoints.md) return 'md';
  if (width >= breakpoints.sm) return 'sm';
  if (width >= breakpoints.xs) return 'xs';
  
  return 'base';
}

// Container utilities
export const containerClasses = {
  responsive: 'w-full mx-auto px-4 sm:px-6 lg:px-8',
  maxWidth: {
    xs: 'max-w-none',
    sm: 'max-w-screen-sm',
    md: 'max-w-screen-md',
    lg: 'max-w-screen-lg',
    xl: 'max-w-screen-xl',
    '2xl': 'max-w-screen-2xl',
    '3xl': 'max-w-7xl',
  },
  padding: {
    xs: 'px-4',
    sm: 'px-6',
    md: 'px-6',
    lg: 'px-8',
    xl: 'px-8',
    '2xl': 'px-8',
  },
} as const;

// Grid utilities
export const gridClasses = {
  responsive: {
    auto: 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4',
    autoFit: 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-auto-fit xl:grid-cols-auto-fit',
    cards: 'grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 md:gap-6',
    features: 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 lg:gap-8',
    dashboard: 'grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4 lg:gap-6',
  },
  gap: {
    xs: 'gap-2',
    sm: 'gap-4',
    md: 'gap-6',
    lg: 'gap-8',
    xl: 'gap-12',
  },
} as const;

// Flexbox utilities
export const flexClasses = {
  responsive: {
    stack: 'flex flex-col sm:flex-row',
    center: 'flex items-center justify-center',
    between: 'flex items-center justify-between',
    wrap: 'flex flex-wrap',
  },
  gap: {
    xs: 'gap-2',
    sm: 'gap-4',
    md: 'gap-6',
    lg: 'gap-8',
    xl: 'gap-12',
  },
  direction: {
    responsive: 'flex-col sm:flex-row',
    reverseResponsive: 'flex-col-reverse sm:flex-row',
  },
} as const;

// Typography utilities
export const typographyClasses = {
  heading: {
    responsive: {
      h1: 'text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-bold',
      h2: 'text-xl sm:text-2xl md:text-3xl lg:text-4xl font-bold',
      h3: 'text-lg sm:text-xl md:text-2xl lg:text-3xl font-semibold',
      h4: 'text-base sm:text-lg md:text-xl lg:text-2xl font-semibold',
      h5: 'text-sm sm:text-base md:text-lg font-medium',
      h6: 'text-xs sm:text-sm md:text-base font-medium',
    },
  },
  body: {
    responsive: {
      large: 'text-base sm:text-lg',
      default: 'text-sm sm:text-base',
      small: 'text-xs sm:text-sm',
    },
  },
  lineHeight: {
    tight: 'leading-tight',
    normal: 'leading-normal',
    relaxed: 'leading-relaxed',
  },
} as const;

// Spacing utilities
export const spacingClasses = {
  section: {
    xs: 'py-8',
    sm: 'py-12',
    md: 'py-16',
    lg: 'py-20',
    xl: 'py-24',
    responsive: 'py-8 sm:py-12 md:py-16 lg:py-20',
  },
  margin: {
    responsive: {
      bottom: 'mb-4 sm:mb-6 md:mb-8',
      top: 'mt-4 sm:mt-6 md:mt-8',
      vertical: 'my-4 sm:my-6 md:my-8',
      horizontal: 'mx-4 sm:mx-6 md:mx-8',
    },
  },
  padding: {
    responsive: {
      all: 'p-4 sm:p-6 md:p-8',
      vertical: 'py-4 sm:py-6 md:py-8',
      horizontal: 'px-4 sm:px-6 md:px-8',
    },
  },
} as const;

// Component-specific responsive utilities
export const componentClasses = {
  button: {
    responsive: {
      size: 'h-9 px-3 text-sm sm:h-10 sm:px-4 sm:text-base',
      icon: 'h-8 w-8 sm:h-10 sm:w-10',
    },
  },
  input: {
    responsive: {
      size: 'h-9 px-3 text-sm sm:h-10 sm:px-4 sm:text-base',
    },
  },
  card: {
    responsive: {
      padding: 'p-4 sm:p-6',
      margin: 'm-2 sm:m-4',
      rounded: 'rounded-lg sm:rounded-xl',
    },
  },
  modal: {
    responsive: {
      width: 'w-full sm:w-96 md:w-[28rem] lg:w-[32rem]',
      padding: 'p-4 sm:p-6',
      margin: 'm-4 sm:m-8',
    },
  },
} as const;

// Mobile-specific utilities
export const mobileUtils = {
  touchTarget: {
    minimum: 'min-h-[44px] min-w-[44px]', // Apple's recommended minimum
    comfortable: 'min-h-[48px] min-w-[48px]',
  },
  navigation: {
    bottomSafe: 'pb-safe-bottom',
    topSafe: 'pt-safe-top',
  },
  text: {
    readableSize: 'text-base leading-relaxed',
    maxWidth: 'max-w-prose',
  },
  interaction: {
    hover: 'hover:bg-gray-100 active:bg-gray-200 transition-colors',
    focus: 'focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2',
  },
} as const;

// Utility functions
export function cn(...classes: (string | undefined | null | false)[]): string {
  return classes.filter(Boolean).join(' ');
}

export function createResponsiveClass(
  baseClass: string,
  responsiveVariants: Partial<Record<Breakpoint, string>>
): string {
  let result = baseClass;
  
  Object.entries(responsiveVariants).forEach(([breakpoint, variant]) => {
    if (variant) {
      result += ` ${breakpoint}:${variant}`;
    }
  });
  
  return result;
}

// Theme-aware responsive utilities
export function getThemeResponsiveClass(
  theme: keyof typeof themeConfig.variants,
  component: string,
  variant: string,
  size: Breakpoint = 'xs'
): string {
  const responsiveConfig = themeConfig.responsive;
  return themeConfig.utils.getResponsiveClass(
    responsiveConfig.typography.heading,
    size
  );
}

// Export all utilities as a single object
export const responsiveUtils = {
  breakpoints,
  containerClasses,
  gridClasses,
  flexClasses,
  typographyClasses,
  spacingClasses,
  componentClasses,
  mobileUtils,
  generateResponsiveClasses,
  getResponsiveValue,
  createMediaQuery,
  useMediaQuery,
  useBreakpoint,
  cn,
  createResponsiveClass,
  getThemeResponsiveClass,
} as const;

export type ResponsiveUtils = typeof responsiveUtils;