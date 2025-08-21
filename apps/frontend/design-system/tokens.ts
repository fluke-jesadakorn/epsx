/**
 * Unified Design System Tokens for EPSX Platform
 * 
 * This file consolidates all design tokens from the existing CSS variables,
 * PancakeSwap theme, and component styles into a type-safe, centralized system.
 * 
 * Benefits:
 * - Type safety with IntelliSense
 * - Centralized token management
 * - Tree-shakeable (only used tokens are bundled)
 * - Easy theme switching
 * - Consistent spacing and colors across components
 */

// ============================================================================
// COLOR SYSTEM
// ============================================================================

/**
 * Base color palette - extracted from existing CSS custom properties
 */
export const colors = {
  // Core colors (from CSS variables)
  background: {
    light: 'hsl(252 100% 99%)',
    dark: 'hsl(240 10% 4%)',
  },
  foreground: {
    light: 'hsl(223 84% 10%)',
    dark: 'hsl(0 0% 95%)',
  },
  
  // Card colors
  card: {
    light: 'hsl(0 0% 100%)',
    dark: 'hsl(240 10% 6%)',
    foreground: {
      light: 'hsl(223 84% 10%)',
      dark: 'hsl(0 0% 95%)',
    },
  },
  
  // Primary colors (PancakeSwap inspired)
  primary: {
    50: 'hsl(31 100% 95%)',
    100: 'hsl(31 90% 90%)',
    200: 'hsl(31 95% 85%)',
    300: 'hsl(31 95% 75%)',
    400: 'hsl(31 95% 65%)',
    500: 'hsl(31 100% 50%)', // Main primary
    600: 'hsl(31 95% 45%)',
    700: 'hsl(31 90% 40%)',
    800: 'hsl(31 85% 35%)',
    900: 'hsl(31 80% 30%)',
    foreground: 'hsl(0 0% 100%)',
  },
  
  // Secondary colors
  secondary: {
    50: 'hsl(42 100% 95%)',
    100: 'hsl(42 95% 88%)',
    200: 'hsl(42 90% 80%)',
    300: 'hsl(42 95% 75%)',
    400: 'hsl(42 100% 70%)',
    500: 'hsl(42 100% 70%)', // Main secondary
    600: 'hsl(42 95% 65%)',
    700: 'hsl(42 90% 60%)',
    800: 'hsl(42 85% 55%)',
    900: 'hsl(42 80% 50%)',
    foreground: {
      light: 'hsl(223 84% 10%)',
      dark: 'hsl(0 0% 95%)',
    },
  },
  
  // Status colors
  success: {
    50: 'hsl(142 76% 95%)',
    100: 'hsl(142 71% 88%)',
    200: 'hsl(142 71% 75%)',
    300: 'hsl(142 71% 65%)',
    400: 'hsl(142 71% 55%)',
    500: 'hsl(142 71% 45%)', // Main success
    600: 'hsl(142 66% 40%)',
    700: 'hsl(142 61% 35%)',
    800: 'hsl(142 56% 30%)',
    900: 'hsl(142 51% 25%)',
  },
  
  warning: {
    50: 'hsl(45 98% 95%)',
    100: 'hsl(45 93% 88%)',
    200: 'hsl(45 93% 75%)',
    300: 'hsl(45 93% 65%)',
    400: 'hsl(45 93% 57%)',
    500: 'hsl(45 93% 47%)', // Main warning
    600: 'hsl(45 88% 42%)',
    700: 'hsl(45 83% 37%)',
    800: 'hsl(45 78% 32%)',
    900: 'hsl(45 73% 27%)',
  },
  
  error: {
    50: 'hsl(0 90% 95%)',
    100: 'hsl(0 85% 88%)',
    200: 'hsl(0 85% 75%)',
    300: 'hsl(0 85% 70%)',
    400: 'hsl(0 85% 65%)',
    500: 'hsl(0 85% 60%)', // Main error
    600: 'hsl(0 80% 55%)',
    700: 'hsl(0 75% 50%)',
    800: 'hsl(0 70% 45%)',
    900: 'hsl(0 65% 40%)',
  },
  
  info: {
    50: 'hsl(217 96% 95%)',
    100: 'hsl(217 91% 88%)',
    200: 'hsl(217 91% 75%)',
    300: 'hsl(217 91% 70%)',
    400: 'hsl(217 91% 65%)',
    500: 'hsl(217 91% 60%)', // Main info
    600: 'hsl(217 86% 55%)',
    700: 'hsl(217 81% 50%)',
    800: 'hsl(217 76% 45%)',
    900: 'hsl(217 71% 40%)',
  },
  
  // Neutral colors
  neutral: {
    50: 'hsl(240 5% 96%)',
    100: 'hsl(240 5% 90%)',
    200: 'hsl(240 6% 85%)',
    300: 'hsl(240 5% 75%)',
    400: 'hsl(240 4% 65%)',
    500: 'hsl(240 4% 46%)',
    600: 'hsl(240 4% 35%)',
    700: 'hsl(240 4% 25%)',
    800: 'hsl(240 4% 16%)',
    900: 'hsl(240 4% 9%)',
  },
  
  // Gradient definitions
  gradients: {
    primary: 'linear-gradient(135deg, hsl(31 100% 50%) 0%, hsl(42 100% 70%) 100%)',
    primaryDark: 'linear-gradient(135deg, hsl(31 100% 55%) 0%, hsl(42 100% 75%) 100%)',
    secondary: 'linear-gradient(135deg, hsl(217 91% 60%) 0%, hsl(184 93% 47%) 100%)',
    secondaryDark: 'linear-gradient(135deg, hsl(217 91% 65%) 0%, hsl(184 93% 52%) 100%)',
    accent: 'linear-gradient(135deg, hsl(142 71% 45%) 0%, hsl(31 100% 50%) 100%)',
    accentDark: 'linear-gradient(135deg, hsl(142 71% 50%) 0%, hsl(31 100% 55%) 100%)',
    softHighlight: 'linear-gradient(135deg, hsl(31 80% 92%) 0%, hsl(42 90% 88%) 100%)',
    softHighlightDark: 'linear-gradient(135deg, hsl(31 50% 20%) 0%, hsl(42 60% 18%) 100%)',
  },
} as const;

// ============================================================================
// SPACING SYSTEM
// ============================================================================

/**
 * Spacing scale based on 0.25rem (4px) base unit
 */
export const spacing = {
  0: '0',
  px: '1px',
  0.5: '0.125rem', // 2px
  1: '0.25rem',    // 4px
  1.5: '0.375rem', // 6px
  2: '0.5rem',     // 8px
  2.5: '0.625rem', // 10px
  3: '0.75rem',    // 12px
  3.5: '0.875rem', // 14px
  4: '1rem',       // 16px
  5: '1.25rem',    // 20px
  6: '1.5rem',     // 24px
  7: '1.75rem',    // 28px
  8: '2rem',       // 32px
  9: '2.25rem',    // 36px
  10: '2.5rem',    // 40px
  11: '2.75rem',   // 44px
  12: '3rem',      // 48px
  14: '3.5rem',    // 56px
  16: '4rem',      // 64px
  20: '5rem',      // 80px
  24: '6rem',      // 96px
  28: '7rem',      // 112px
  32: '8rem',      // 128px
  36: '9rem',      // 144px
  40: '10rem',     // 160px
  44: '11rem',     // 176px
  48: '12rem',     // 192px
  52: '13rem',     // 208px
  56: '14rem',     // 224px
  60: '15rem',     // 240px
  64: '16rem',     // 256px
  72: '18rem',     // 288px
  80: '20rem',     // 320px
  96: '24rem',     // 384px
} as const;

// ============================================================================
// TYPOGRAPHY SYSTEM
// ============================================================================

/**
 * Typography scale and font definitions
 */
export const typography = {
  // Font families
  fonts: {
    sans: ['var(--font-kanit)', 'system-ui', 'sans-serif'],
    mono: ['ui-monospace', 'SFMono-Regular', 'Consolas', 'monospace'],
    pancake: ['Kanit', 'Baloo 2', 'Comic Sans MS', 'Comic Sans', 'cursive', 'sans-serif'],
  },
  
  // Font sizes
  fontSize: {
    xs: ['0.75rem', { lineHeight: '1rem' }],      // 12px
    sm: ['0.875rem', { lineHeight: '1.25rem' }],  // 14px
    base: ['1rem', { lineHeight: '1.5rem' }],     // 16px
    lg: ['1.125rem', { lineHeight: '1.75rem' }],  // 18px
    xl: ['1.25rem', { lineHeight: '1.75rem' }],   // 20px
    '2xl': ['1.5rem', { lineHeight: '2rem' }],    // 24px
    '3xl': ['1.875rem', { lineHeight: '2.25rem' }], // 30px
    '4xl': ['2.25rem', { lineHeight: '2.5rem' }], // 36px
    '5xl': ['3rem', { lineHeight: '1' }],         // 48px
    '6xl': ['3.75rem', { lineHeight: '1' }],      // 60px
    '7xl': ['4.5rem', { lineHeight: '1' }],       // 72px
  },
  
  // Font weights
  fontWeight: {
    thin: '100',
    extralight: '200',
    light: '300',
    normal: '400',
    medium: '500',
    semibold: '600',
    bold: '700',
    extrabold: '800',
    black: '900',
  },
  
  // Letter spacing
  letterSpacing: {
    tighter: '-0.05em',
    tight: '-0.025em',
    normal: '0em',
    wide: '0.025em',
    wider: '0.05em',
    widest: '0.1em',
  },
  
  // Line heights
  lineHeight: {
    none: '1',
    tight: '1.25',
    snug: '1.375',
    normal: '1.5',
    relaxed: '1.625',
    loose: '2',
  },
} as const;

// ============================================================================
// BORDER RADIUS SYSTEM
// ============================================================================

/**
 * Border radius scale
 */
export const borderRadius = {
  none: '0',
  sm: '0.125rem',   // 2px
  DEFAULT: '0.25rem', // 4px
  md: '0.375rem',   // 6px
  lg: '0.5rem',     // 8px
  xl: '0.75rem',    // 12px
  '2xl': '1rem',    // 16px
  '3xl': '1.5rem',  // 24px
  full: '9999px',
} as const;

// ============================================================================
// SHADOW SYSTEM
// ============================================================================

/**
 * Box shadow definitions
 */
export const shadows = {
  none: 'none',
  sm: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
  DEFAULT: '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)',
  md: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
  lg: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
  xl: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)',
  '2xl': '0 25px 50px -12px rgb(0 0 0 / 0.25)',
  '3xl': '0 35px 60px -12px rgb(0 0 0 / 0.25)',
  inner: 'inset 0 2px 4px 0 rgb(0 0 0 / 0.05)',
  
  // PancakeSwap custom shadows
  pancake: '0 4px 12px hsl(255 133 27 / 0.15)',
  pancakeDark: '0 4px 12px hsl(255 133 27 / 0.25)',
  pancakeGlow: '0 0 20px hsl(255 133 27 / 0.3)',
  pancakeGlowDark: '0 0 25px hsl(255 133 27 / 0.4)',
  enhanced: '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1), 0 0 0 1px rgb(0 0 0 / 0.05)',
  enhancedLg: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1), 0 0 0 1px rgb(0 0 0 / 0.05)',
} as const;

// ============================================================================
// BREAKPOINTS
// ============================================================================

/**
 * Responsive breakpoints
 */
export const breakpoints = {
  xs: '475px',
  sm: '640px',
  md: '768px',
  lg: '1024px',
  xl: '1280px',
  '2xl': '1536px',
} as const;

// ============================================================================
// Z-INDEX SCALE
// ============================================================================

/**
 * Z-index scale for layering
 */
export const zIndex = {
  auto: 'auto',
  0: '0',
  10: '10',
  20: '20',
  30: '30',
  40: '40',
  50: '50',
  
  // Semantic z-index values
  dropdown: '1000',
  sticky: '1020',
  fixed: '1030',
  modalBackdrop: '1040',
  offcanvas: '1050',
  modal: '1060',
  popover: '1070',
  tooltip: '1080',
  toast: '1090',
} as const;

// ============================================================================
// TRANSITIONS & ANIMATIONS
// ============================================================================

/**
 * Transition and animation definitions
 */
export const transitions = {
  // Transition durations
  duration: {
    75: '75ms',
    100: '100ms',
    150: '150ms',
    200: '200ms',
    300: '300ms',
    500: '500ms',
    700: '700ms',
    1000: '1000ms',
  },
  
  // Transition timing functions
  timing: {
    linear: 'linear',
    in: 'cubic-bezier(0.4, 0, 1, 1)',
    out: 'cubic-bezier(0, 0, 0.2, 1)',
    inOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
    pancake: 'cubic-bezier(0.4, 0, 0.2, 1)', // PancakeSwap style
  },
  
  // Common transition properties
  all: 'all 300ms cubic-bezier(0.4, 0, 0.2, 1)',
  colors: 'color, background-color, border-color, text-decoration-color, fill, stroke 300ms cubic-bezier(0.4, 0, 0.2, 1)',
  opacity: 'opacity 300ms cubic-bezier(0.4, 0, 0.2, 1)',
  shadow: 'box-shadow 300ms cubic-bezier(0.4, 0, 0.2, 1)',
  transform: 'transform 300ms cubic-bezier(0.4, 0, 0.2, 1)',
} as const;

// ============================================================================
// TYPE HELPERS
// ============================================================================

/**
 * Type helpers for design tokens
 */
export type ColorScale = keyof typeof colors.primary;
export type SpacingScale = keyof typeof spacing;
export type FontSize = keyof typeof typography.fontSize;
export type FontWeight = keyof typeof typography.fontWeight;
export type BorderRadius = keyof typeof borderRadius;
export type Shadow = keyof typeof shadows;
export type Breakpoint = keyof typeof breakpoints;
export type ZIndex = keyof typeof zIndex;

/**
 * Complete design token system
 */
export const tokens = {
  colors,
  spacing,
  typography,
  borderRadius,
  shadows,
  breakpoints,
  zIndex,
  transitions,
} as const;

export type DesignTokens = typeof tokens;