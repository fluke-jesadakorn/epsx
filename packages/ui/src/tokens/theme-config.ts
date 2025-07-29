/**
 * EPSX Theme Configuration
 * Centralized theme configuration for PancakeSwap-inspired design system
 */

import { designTokens } from './design-tokens';

// Theme variants for different contexts
export const themeVariants = {
  default: {
    colors: {
      primary: designTokens.colors.primary[500],
      secondary: designTokens.colors.secondary[400],
      accent: designTokens.colors.pancake.green[500],
      success: designTokens.colors.pancake.green[500],
      warning: designTokens.colors.pancake.yellow[500],
      error: designTokens.colors.pancake.red[500],
      info: designTokens.colors.pancake.blue[500],
      muted: designTokens.colors.gray[100],
      border: designTokens.colors.gray[200],
    },
    gradients: {
      primary: designTokens.gradients.primary,
      secondary: designTokens.gradients.secondary,
      accent: designTokens.gradients.accent,
      rainbow: designTokens.gradients.rainbow,
    },
  },
  pancake: {
    colors: {
      primary: designTokens.colors.pancake.orange[500],
      secondary: designTokens.colors.pancake.yellow[400],
      accent: designTokens.colors.pancake.green[500],
      success: designTokens.colors.pancake.green[500],
      warning: designTokens.colors.pancake.yellow[500],
      error: designTokens.colors.pancake.red[500],
      info: designTokens.colors.pancake.blue[500],
    },
    gradients: {
      primary: 'linear-gradient(135deg, hsl(31 100% 50%) 0%, hsl(42 100% 70%) 100%)',
      secondary: 'linear-gradient(135deg, hsl(217 91% 60%) 0%, hsl(184 93% 47%) 100%)',
      accent: 'linear-gradient(135deg, hsl(142 71% 45%) 0%, hsl(31 100% 50%) 100%)',
      rainbow: designTokens.gradients.rainbow,
    },
  },
  trading: {
    colors: {
      primary: designTokens.colors.pancake.green[500],
      secondary: designTokens.colors.pancake.red[500],
      accent: designTokens.colors.pancake.blue[500],
      success: designTokens.colors.pancake.green[500],
      warning: designTokens.colors.pancake.yellow[500],
      error: designTokens.colors.pancake.red[500],
      info: designTokens.colors.pancake.blue[500],
      bullish: designTokens.colors.pancake.green[500],
      bearish: designTokens.colors.pancake.red[500],
      neutral: designTokens.colors.gray[500],
      muted: designTokens.colors.gray[100],
      border: designTokens.colors.gray[200],
    },
    gradients: {
      primary: 'linear-gradient(135deg, hsl(142 71% 45%) 0%, hsl(142 71% 60%) 100%)',
      secondary: 'linear-gradient(135deg, hsl(0 85% 60%) 0%, hsl(0 85% 75%) 100%)',
      accent: 'linear-gradient(135deg, hsl(217 91% 60%) 0%, hsl(184 93% 47%) 100%)',
      rainbow: designTokens.gradients.rainbow,
      bullish: 'linear-gradient(135deg, hsl(142 71% 45%) 0%, hsl(142 71% 60%) 100%)',
      bearish: 'linear-gradient(135deg, hsl(0 85% 60%) 0%, hsl(0 85% 75%) 100%)',
      neutral: 'linear-gradient(135deg, hsl(240 4% 46%) 0%, hsl(240 4% 60%) 100%)',
    },
  },
} as const;

// Component theme configurations
export const componentThemes = {
  button: {
    variants: {
      pancake: {
        base: 'font-semibold transition-all duration-300 hover:scale-105',
        primary: 'bg-gradient-to-r from-orange-500 to-yellow-500 text-white shadow-lg hover:shadow-xl',
        secondary: 'bg-gradient-to-r from-blue-500 to-purple-500 text-white shadow-lg hover:shadow-xl',
        outline: 'border-2 border-orange-500 text-orange-600 hover:bg-orange-500 hover:text-white',
        ghost: 'text-orange-600 hover:bg-orange-100 dark:hover:bg-orange-900/20',
        soft: 'bg-gradient-to-r from-orange-50 to-yellow-50 text-orange-700 hover:from-orange-100 hover:to-yellow-100',
      },
      trading: {
        bullish: 'bg-gradient-to-r from-green-500 to-green-600 text-white hover:from-green-600 hover:to-green-700',
        bearish: 'bg-gradient-to-r from-red-500 to-red-600 text-white hover:from-red-600 hover:to-red-700',
        neutral: 'bg-gradient-to-r from-gray-500 to-gray-600 text-white hover:from-gray-600 hover:to-gray-700',
      },
    },
    sizes: {
      responsive: {
        sm: 'xs:h-8 xs:px-2 xs:text-xs sm:h-9 sm:px-3 sm:text-sm',
        base: 'xs:h-9 xs:px-3 xs:text-sm sm:h-10 sm:px-4 sm:text-base',
        lg: 'xs:h-10 xs:px-4 xs:text-sm sm:h-11 sm:px-6 sm:text-base md:px-8',
      },
    },
  },
  card: {
    variants: {
      pancake: {
        base: 'bg-white/95 dark:bg-gray-900/95 backdrop-blur-12 border border-orange-200/20 dark:border-orange-500/20 shadow-lg',
        elevated: 'bg-white/98 dark:bg-gray-800/98 backdrop-blur-16 border border-orange-300/30 dark:border-orange-400/30 shadow-xl',
        glowing: 'bg-white/95 dark:bg-gray-900/95 backdrop-blur-12 border border-orange-400/40 dark:border-orange-300/40 shadow-2xl shadow-orange-500/20',
      },
      trading: {
        bullish: 'bg-white/95 dark:bg-gray-900/95 backdrop-blur-12 border border-green-200/30 dark:border-green-500/30 shadow-lg shadow-green-500/10',
        bearish: 'bg-white/95 dark:bg-gray-900/95 backdrop-blur-12 border border-red-200/30 dark:border-red-500/30 shadow-lg shadow-red-500/10',
        neutral: 'bg-white/95 dark:bg-gray-900/95 backdrop-blur-12 border border-gray-200/30 dark:border-gray-500/30 shadow-lg',
      },
    },
    spacing: {
      responsive: {
        sm: 'xs:p-3 sm:p-4',
        base: 'xs:p-4 sm:p-6',
        lg: 'xs:p-6 sm:p-8',
      },
    },
  },
  input: {
    variants: {
      pancake: {
        base: 'border-2 border-gray-200 dark:border-gray-700 bg-white/90 dark:bg-gray-800/90 backdrop-blur-8 focus:border-orange-500 focus:ring-orange-500/20',
        success: 'border-green-500 bg-green-50 dark:bg-green-900/20 focus:border-green-600 focus:ring-green-500/20',
        error: 'border-red-500 bg-red-50 dark:bg-red-900/20 focus:border-red-600 focus:ring-red-500/20',
      },
    },
    sizes: {
      responsive: {
        sm: 'xs:h-8 xs:px-2 xs:text-sm sm:h-9 sm:px-3',
        base: 'xs:h-9 xs:px-3 xs:text-sm sm:h-10 sm:px-4 sm:text-base',
        lg: 'xs:h-10 xs:px-4 xs:text-base sm:h-11 sm:px-5',
      },
    },
  },
} as const;

// Responsive design utilities
export const responsiveUtils = {
  container: {
    xs: 'max-w-none px-4',
    sm: 'max-w-screen-sm px-6 mx-auto',
    md: 'max-w-screen-md px-6 mx-auto',
    lg: 'max-w-screen-lg px-8 mx-auto',
    xl: 'max-w-screen-xl px-8 mx-auto',
    '2xl': 'max-w-screen-2xl px-8 mx-auto',
  },
  spacing: {
    section: {
      xs: 'py-8',
      sm: 'py-12',
      md: 'py-16',
      lg: 'py-20',
      xl: 'py-24',
    },
    gap: {
      xs: 'gap-2',
      sm: 'gap-4',
      md: 'gap-6',
      lg: 'gap-8',
      xl: 'gap-12',
    },
  },
  typography: {
    heading: {
      xs: 'text-xl font-bold',
      sm: 'text-2xl font-bold',
      md: 'text-3xl font-bold',
      lg: 'text-4xl font-bold',
      xl: 'text-5xl font-bold',
    },
    body: {
      xs: 'text-sm',
      sm: 'text-base',
      md: 'text-lg',
    },
  },
} as const;

// Animation presets
export const animationPresets = {
  pancake: {
    float: 'animate-float',
    bounce: 'animate-bounce-gentle',
    pulse: 'animate-pulse-gentle',
    wiggle: 'animate-wiggle',
    glow: 'animate-glow',
    gradient: 'animate-gradient-x',
  },
  interactions: {
    hover: 'transition-all duration-200 hover:scale-105',
    press: 'transition-all duration-100 active:scale-95',
    focus: 'transition-all duration-200 focus:scale-105 focus:ring-2 focus:ring-orange-500/20',
  },
  loading: {
    spin: 'animate-spin',
    pulse: 'animate-pulse',
    bounce: 'animate-bounce',
  },
} as const;

// Dark mode variants
export const darkModeVariants = {
  colors: {
    background: {
      primary: 'bg-gray-950',
      secondary: 'bg-gray-900',
      elevated: 'bg-gray-800',
    },
    text: {
      primary: 'text-gray-50',
      secondary: 'text-gray-300',
      muted: 'text-gray-500',
    },
    border: {
      subtle: 'border-gray-800',
      default: 'border-gray-700',
      strong: 'border-gray-600',
    },
  },
  pancake: {
    colors: {
      primary: designTokens.colors.pancake.orange[400],
      secondary: designTokens.colors.pancake.yellow[300],
      accent: designTokens.colors.pancake.green[400],
    },
    gradients: {
      primary: 'linear-gradient(135deg, hsl(31 100% 55%) 0%, hsl(42 100% 75%) 100%)',
      secondary: 'linear-gradient(135deg, hsl(217 91% 65%) 0%, hsl(184 93% 52%) 100%)',
      softHighlight: 'linear-gradient(135deg, hsl(31 50% 20%) 0%, hsl(42 60% 18%) 100%)',
    },
  },
} as const;

// Utility functions for theme generation
export const themeUtils = {
  // Generate CSS custom properties for a theme
  generateCSSVars: (theme: any) => {
    const cssVars: Record<string, string> = {};
    
    // Colors
    if (theme.colors) {
      Object.entries(theme.colors).forEach(([key, value]) => {
        cssVars[`--color-${key}`] = value as string;
      });
    }
    
    // Gradients
    if (theme.gradients) {
      Object.entries(theme.gradients).forEach(([key, value]) => {
        cssVars[`--gradient-${key}`] = value as string;
      });
    }
    
    return cssVars;
  },
  
  // Get responsive class for a given breakpoint
  getResponsiveClass: (classes: Record<string, string>, breakpoint: keyof typeof designTokens.screens) => {
    return classes[breakpoint] || classes.base || '';
  },
  
  // Merge theme variants
  mergeThemes: (...themes: any[]) => {
    return themes.reduce((acc, theme) => ({ ...acc, ...theme }), {});
  },
} as const;

// Export everything
export const themeConfig = {
  variants: themeVariants,
  components: componentThemes,
  responsive: responsiveUtils,
  animations: animationPresets,
  darkMode: darkModeVariants,
  utils: themeUtils,
} as const;

export type ThemeConfig = typeof themeConfig;
export type ThemeVariant = keyof typeof themeVariants;
export type ComponentTheme = typeof componentThemes;
export type ResponsiveUtils = typeof responsiveUtils;