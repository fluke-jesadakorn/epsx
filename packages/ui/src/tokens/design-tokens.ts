/**
 * EPSX Design System Tokens
 * PancakeSwap-inspired design tokens for consistent theming across the monorepo
 */

// Colors
export const colors = {
  // Semantic colors
  primary: {
    50: 'hsl(31 100% 95%)',
    100: 'hsl(31 100% 90%)',
    200: 'hsl(31 100% 80%)',
    300: 'hsl(31 100% 70%)',
    400: 'hsl(31 100% 60%)',
    500: 'hsl(31 100% 50%)', // Main primary
    600: 'hsl(31 100% 45%)',
    700: 'hsl(31 100% 40%)',
    800: 'hsl(31 100% 35%)',
    900: 'hsl(31 100% 30%)',
  },
  secondary: {
    50: 'hsl(42 100% 95%)',
    100: 'hsl(42 100% 90%)',
    200: 'hsl(42 100% 80%)',
    300: 'hsl(42 100% 75%)',
    400: 'hsl(42 100% 70%)', // Main secondary
    500: 'hsl(42 100% 65%)',
    600: 'hsl(42 100% 60%)',
    700: 'hsl(42 100% 55%)',
    800: 'hsl(42 100% 50%)',
    900: 'hsl(42 100% 45%)',
  },
  
  // PancakeSwap inspired colors
  pancake: {
    orange: {
      50: 'hsl(31 100% 95%)',
      100: 'hsl(31 100% 90%)',
      200: 'hsl(31 100% 80%)',
      300: 'hsl(31 100% 70%)',
      400: 'hsl(31 100% 60%)',
      500: 'hsl(31 100% 50%)',
      600: 'hsl(31 100% 45%)',
      700: 'hsl(31 100% 40%)',
      800: 'hsl(31 100% 35%)',
      900: 'hsl(31 100% 30%)',
    },
    yellow: {
      50: 'hsl(42 100% 95%)',
      100: 'hsl(42 100% 90%)',
      200: 'hsl(42 100% 85%)',
      300: 'hsl(42 100% 80%)',
      400: 'hsl(42 100% 70%)',
      500: 'hsl(42 100% 60%)',
      600: 'hsl(42 100% 55%)',
      700: 'hsl(42 100% 50%)',
      800: 'hsl(42 100% 45%)',
      900: 'hsl(42 100% 40%)',
    },
    blue: {
      50: 'hsl(217 91% 95%)',
      100: 'hsl(217 91% 90%)',
      200: 'hsl(217 91% 80%)',
      300: 'hsl(217 91% 70%)',
      400: 'hsl(217 91% 65%)',
      500: 'hsl(217 91% 60%)',
      600: 'hsl(217 91% 55%)',
      700: 'hsl(217 91% 50%)',
      800: 'hsl(217 91% 45%)',
      900: 'hsl(217 91% 40%)',
    },
    purple: {
      50: 'hsl(280 100% 95%)',
      100: 'hsl(280 100% 90%)',
      200: 'hsl(280 100% 80%)',
      300: 'hsl(280 100% 70%)',
      400: 'hsl(280 100% 60%)',
      500: 'hsl(280 100% 50%)',
      600: 'hsl(280 100% 45%)',
      700: 'hsl(280 100% 40%)',
      800: 'hsl(280 100% 35%)',
      900: 'hsl(280 100% 30%)',
    },
    green: {
      50: 'hsl(142 71% 95%)',
      100: 'hsl(142 71% 90%)',
      200: 'hsl(142 71% 80%)',
      300: 'hsl(142 71% 70%)',
      400: 'hsl(142 71% 60%)',
      500: 'hsl(142 71% 50%)',
      600: 'hsl(142 71% 45%)',
      700: 'hsl(142 71% 40%)',
      800: 'hsl(142 71% 35%)',
      900: 'hsl(142 71% 30%)',
    },
    red: {
      50: 'hsl(0 85% 95%)',
      100: 'hsl(0 85% 90%)',
      200: 'hsl(0 85% 80%)',
      300: 'hsl(0 85% 70%)',
      400: 'hsl(0 85% 65%)',
      500: 'hsl(0 85% 60%)',
      600: 'hsl(0 85% 55%)',
      700: 'hsl(0 85% 50%)',
      800: 'hsl(0 85% 45%)',
      900: 'hsl(0 85% 40%)',
    },
  },

  // Neutral colors
  gray: {
    50: 'hsl(240 5% 96%)',
    100: 'hsl(240 5% 90%)',
    200: 'hsl(240 6% 85%)',
    300: 'hsl(240 5% 75%)',
    400: 'hsl(240 4% 60%)',
    500: 'hsl(240 4% 46%)',
    600: 'hsl(240 5% 34%)',
    700: 'hsl(240 5% 26%)',
    800: 'hsl(240 4% 16%)',
    900: 'hsl(240 10% 6%)',
    950: 'hsl(240 10% 4%)',
  },
} as const;

// Spacing scale (mobile-first)
export const spacing = {
  0: '0px',
  1: '0.25rem', // 4px
  2: '0.5rem',  // 8px
  3: '0.75rem', // 12px
  4: '1rem',    // 16px
  5: '1.25rem', // 20px
  6: '1.5rem',  // 24px
  7: '1.75rem', // 28px
  8: '2rem',    // 32px
  9: '2.25rem', // 36px
  10: '2.5rem', // 40px
  11: '2.75rem', // 44px
  12: '3rem',    // 48px
  14: '3.5rem',  // 56px
  16: '4rem',    // 64px
  18: '4.5rem',  // 72px
  20: '5rem',    // 80px
  24: '6rem',    // 96px
  28: '7rem',    // 112px
  32: '8rem',    // 128px
  36: '9rem',    // 144px
  40: '10rem',   // 160px
  44: '11rem',   // 176px
  48: '12rem',   // 192px
  52: '13rem',   // 208px
  56: '14rem',   // 224px
  60: '15rem',   // 240px
  64: '16rem',   // 256px
  72: '18rem',   // 288px
  80: '20rem',   // 320px
  96: '24rem',   // 384px
} as const;

// Typography scale
export const typography = {
  fontFamily: {
    sans: ['Kanit', 'Inter', 'system-ui', 'sans-serif'],
    pancake: ['Kanit', 'Baloo 2', 'system-ui', 'sans-serif'],
    mono: ['SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', 'monospace'],
  },
  fontSize: {
    // Mobile-first sizes
    xs: ['0.75rem', { lineHeight: '1rem' }],      // 12px
    sm: ['0.875rem', { lineHeight: '1.25rem' }],  // 14px
    base: ['1rem', { lineHeight: '1.5rem' }],     // 16px
    lg: ['1.125rem', { lineHeight: '1.75rem' }],  // 18px
    xl: ['1.25rem', { lineHeight: '1.75rem' }],   // 20px
    '2xl': ['1.5rem', { lineHeight: '2rem' }],    // 24px
    '3xl': ['1.875rem', { lineHeight: '2.25rem' }], // 30px
    '4xl': ['2.25rem', { lineHeight: '2.5rem' }],   // 36px
    '5xl': ['3rem', { lineHeight: '1' }],           // 48px
    '6xl': ['3.75rem', { lineHeight: '1' }],        // 60px
    '7xl': ['4.5rem', { lineHeight: '1' }],         // 72px
    '8xl': ['6rem', { lineHeight: '1' }],           // 96px
    '9xl': ['8rem', { lineHeight: '1' }],           // 128px
  },
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
  letterSpacing: {
    tighter: '-0.05em',
    tight: '-0.025em',
    normal: '0em',
    wide: '0.025em',
    wider: '0.05em',
    widest: '0.1em',
  },
} as const;

// Border radius
export const borderRadius = {
  none: '0px',
  sm: '0.125rem',   // 2px
  base: '0.25rem',  // 4px
  md: '0.375rem',   // 6px
  lg: '0.5rem',     // 8px
  xl: '0.75rem',    // 12px
  '2xl': '1rem',    // 16px
  '3xl': '1.5rem',  // 24px
  full: '9999px',
} as const;

// Shadows
export const boxShadow = {
  sm: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
  base: '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)',
  md: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
  lg: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
  xl: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)',
  '2xl': '0 25px 50px -12px rgb(0 0 0 / 0.25)',
  inner: 'inset 0 2px 4px 0 rgb(0 0 0 / 0.05)',
  none: '0 0 #0000',
  // PancakeSwap inspired shadows
  pancake: '0 4px 12px rgb(255 140 0 / 0.15)',
  'pancake-lg': '0 8px 24px rgb(255 140 0 / 0.2)',
  'pancake-xl': '0 12px 32px rgb(255 140 0 / 0.25)',
} as const;

// Breakpoints (mobile-first)
export const screens = {
  xs: '475px',
  sm: '640px',
  md: '768px',
  lg: '1024px',
  xl: '1280px',
  '2xl': '1536px',
  '3xl': '1600px',
} as const;

// Animation timings
export const animation = {
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
  timing: {
    linear: 'linear',
    in: 'cubic-bezier(0.4, 0, 1, 1)',
    out: 'cubic-bezier(0, 0, 0.2, 1)',
    'in-out': 'cubic-bezier(0.4, 0, 0.2, 1)',
    'bounce': 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
    'pancake': 'cubic-bezier(0.16, 1, 0.3, 1)',
  },
} as const;

// Z-index scale
export const zIndex = {
  0: '0',
  10: '10',
  20: '20',
  30: '30',
  40: '40',
  50: '50',
  auto: 'auto',
  dropdown: '1000',
  sticky: '1020',
  fixed: '1030',
  modal: '1040',
  popover: '1050',
  tooltip: '1060',
  toast: '1070',
} as const;

// Component-specific tokens
export const components = {
  button: {
    height: {
      sm: '2rem',      // 32px
      base: '2.5rem',  // 40px
      lg: '2.75rem',   // 44px
      xl: '3rem',      // 48px
    },
    padding: {
      sm: '0.5rem 0.75rem',    // 8px 12px
      base: '0.5rem 1rem',     // 8px 16px
      lg: '0.75rem 1.5rem',    // 12px 24px
      xl: '1rem 2rem',         // 16px 32px
    },
    fontSize: {
      sm: '0.875rem',  // 14px
      base: '1rem',    // 16px
      lg: '1.125rem',  // 18px
    },
  },
  card: {
    padding: {
      sm: '1rem',      // 16px
      base: '1.5rem',  // 24px
      lg: '2rem',      // 32px
    },
    borderRadius: {
      base: '0.75rem', // 12px
      lg: '1rem',      // 16px
    },
  },
  input: {
    height: {
      sm: '2rem',      // 32px
      base: '2.5rem',  // 40px
      lg: '2.75rem',   // 44px
    },
    padding: {
      x: '0.75rem',    // 12px
      y: '0.5rem',     // 8px
    },
  },
} as const;

// Gradients
export const gradients = {
  primary: 'linear-gradient(135deg, hsl(31 100% 50%) 0%, hsl(42 100% 70%) 100%)',
  secondary: 'linear-gradient(135deg, hsl(217 91% 60%) 0%, hsl(184 93% 47%) 100%)',
  accent: 'linear-gradient(135deg, hsl(142 71% 45%) 0%, hsl(31 100% 50%) 100%)',
  softHighlight: 'linear-gradient(135deg, hsl(31 80% 92%) 0%, hsl(42 90% 88%) 100%)',
  rainbow: 'linear-gradient(135deg, hsl(31 100% 50%) 0%, hsl(42 100% 70%) 20%, hsl(142 71% 45%) 40%, hsl(217 91% 60%) 60%, hsl(280 100% 50%) 80%, hsl(0 85% 60%) 100%)',
} as const;

// Export all tokens as a single object
export const designTokens = {
  colors,
  spacing,
  typography,
  borderRadius,
  boxShadow,
  screens,
  animation,
  zIndex,
  components,
  gradients,
} as const;

export type DesignTokens = typeof designTokens;
export type ColorTokens = typeof colors;
export type SpacingTokens = typeof spacing;
export type TypographyTokens = typeof typography;