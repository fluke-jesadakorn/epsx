/**
 * Admin Frontend Design System Tokens
 * 
 * Type-safe design tokens specifically tailored for admin interfaces.
 * Extends the base design system with admin-specific semantics and colors.
 * 
 * Features:
 * - Enhanced status indicators for user management
 * - Permission-based color semantics
 * - Admin dashboard-specific spacing and typography
 * - Billing and analytics color schemes
 */

// ============================================================================
// COLOR SYSTEM
// ============================================================================

/**
 * Admin-enhanced color palette with semantic naming
 */
export const colors = {
  // Core colors (from CSS variables)
  background: {
    light: 'hsl(251 100% 98%)',
    dark: 'hsl(220 26% 8%)',
  },
  foreground: {
    light: 'hsl(220 26% 14%)',
    dark: 'hsl(210 40% 95%)',
  },
  
  // Card colors
  card: {
    light: 'hsl(0 0% 100%)',
    dark: 'hsl(220 26% 10%)',
    foreground: {
      light: 'hsl(220 26% 14%)',
      dark: 'hsl(210 40% 95%)',
    },
  },
  
  // Primary brand colors (PancakeSwap inspired)
  primary: {
    50: 'hsl(31 100% 95%)',
    100: 'hsl(31 90% 90%)',
    200: 'hsl(31 95% 85%)',
    300: 'hsl(31 95% 75%)',
    400: 'hsl(31 95% 65%)',
    500: 'hsl(31 100% 50%)', // Main primary - Orange
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
    500: 'hsl(42 100% 70%)', // Main secondary - Yellow
    600: 'hsl(42 95% 65%)',
    700: 'hsl(42 90% 60%)',
    800: 'hsl(42 85% 55%)',
    900: 'hsl(42 80% 50%)',
    foreground: {
      light: 'hsl(220 26% 14%)',
      dark: 'hsl(210 40% 95%)',
    },
  },
  
  // Gradient definitions
  gradients: {
    pancake: 'linear-gradient(135deg, hsl(31 100% 50%) 0%, hsl(42 100% 70%) 100%)',
    pancakeHover: 'linear-gradient(135deg, hsl(31 95% 45%) 0%, hsl(42 95% 65%) 100%)',
    admin: 'linear-gradient(135deg, hsl(217 91% 60%) 0%, hsl(262 83% 58%) 100%)',
    adminHover: 'linear-gradient(135deg, hsl(217 91% 65%) 0%, hsl(262 83% 63%) 100%)',
    analytics: 'linear-gradient(135deg, hsl(215 16% 47%) 0%, hsl(215 19% 35%) 100%)',
    analyticsHover: 'linear-gradient(135deg, hsl(215 20% 52%) 0%, hsl(215 22% 40%) 100%)',
    softHighlight: 'linear-gradient(135deg, hsl(31 95% 92%) 0%, hsl(42 90% 88%) 100%)',
    softHighlightDark: 'linear-gradient(135deg, hsl(31 50% 20%) 0%, hsl(42 60% 18%) 100%)',
  },
  
  // System status colors
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
    50: 'hsl(0 93% 95%)',
    100: 'hsl(0 88% 88%)',
    200: 'hsl(0 88% 75%)',
    300: 'hsl(0 88% 65%)',
    400: 'hsl(0 88% 58%)',
    500: 'hsl(0 85% 60%)', // Main error
    600: 'hsl(0 83% 55%)',
    700: 'hsl(0 80% 50%)',
    800: 'hsl(0 77% 45%)',
    900: 'hsl(0 74% 40%)',
  },
  
  info: {
    50: 'hsl(217 96% 95%)',
    100: 'hsl(217 91% 88%)',
    200: 'hsl(217 91% 75%)',
    300: 'hsl(217 91% 65%)',
    400: 'hsl(217 91% 58%)',
    500: 'hsl(217 91% 60%)', // Main info
    600: 'hsl(217 86% 55%)',
    700: 'hsl(217 81% 50%)',
    800: 'hsl(217 76% 45%)',
    900: 'hsl(217 71% 40%)',
  },
  
  // Admin-specific semantic colors
  admin: {
    // User status colors
    active: 'hsl(142 71% 45%)', // Green for active users
    inactive: 'hsl(215 16% 47%)', // Gray for inactive
    pending: 'hsl(45 93% 47%)', // Amber for pending
    suspended: 'hsl(0 85% 60%)', // Red for suspended
    premium: 'hsl(250 84% 54%)', // Purple for premium
    
    // Permission status colors
    granted: 'hsl(142 71% 45%)', // Green
    denied: 'hsl(0 85% 60%)', // Red
    inherited: 'hsl(217 91% 60%)', // Blue
    
    // Billing status colors
    paid: 'hsl(142 71% 45%)', // Green
    overdue: 'hsl(0 85% 60%)', // Red
    trial: 'hsl(45 93% 47%)', // Amber
    enterprise: 'hsl(250 84% 54%)', // Purple
  },
  
  // Neutral grays
  neutral: {
    50: 'hsl(217 32% 96%)',
    100: 'hsl(217 32% 91%)',
    200: 'hsl(215 28% 85%)',
    300: 'hsl(215 28% 75%)',
    400: 'hsl(215 20% 65%)',
    500: 'hsl(215 16% 47%)', // Main neutral
    600: 'hsl(215 19% 35%)',
    700: 'hsl(215 25% 27%)',
    800: 'hsl(215 28% 17%)',
    900: 'hsl(220 26% 14%)',
  },
} as const;

// ============================================================================
// SPACING SYSTEM
// ============================================================================

/**
 * Consistent spacing scale for admin interfaces
 */
export const spacing = {
  px: '1px',
  0: '0px',
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
  11: '2.75rem',   // 44px (good for touch targets)
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
  64: '16rem',     // 256px (sidebar width)
  72: '18rem',     // 288px
  80: '20rem',     // 320px
  96: '24rem',     // 384px
} as const;

// ============================================================================
// TYPOGRAPHY SYSTEM
// ============================================================================

/**
 * Typography scale optimized for admin interfaces
 */
export const typography = {
  fontSize: {
    xs: ['0.75rem', { lineHeight: '1rem' }],        // 12px
    sm: ['0.875rem', { lineHeight: '1.25rem' }],    // 14px
    base: ['1rem', { lineHeight: '1.5rem' }],       // 16px
    lg: ['1.125rem', { lineHeight: '1.75rem' }],    // 18px
    xl: ['1.25rem', { lineHeight: '1.75rem' }],     // 20px
    '2xl': ['1.5rem', { lineHeight: '2rem' }],      // 24px
    '3xl': ['1.875rem', { lineHeight: '2.25rem' }], // 30px
    '4xl': ['2.25rem', { lineHeight: '2.5rem' }],   // 36px
    '5xl': ['3rem', { lineHeight: '1' }],           // 48px
    '6xl': ['3.75rem', { lineHeight: '1' }],        // 60px
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
  
  fontFamily: {
    sans: ['Inter', 'system-ui', 'sans-serif'],
    mono: ['JetBrains Mono', 'Menlo', 'Monaco', 'Consolas', 'monospace'],
    display: ['Cal Sans', 'Inter', 'system-ui', 'sans-serif'],
  },
  
  letterSpacing: {
    tighter: '-0.05em',
    tight: '-0.025em',
    normal: '0em',
    wide: '0.025em',
    wider: '0.05em',
    widest: '0.1em',
  },
  
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
 * Consistent border radius scale
 */
export const borderRadius = {
  none: '0px',
  sm: '0.125rem',   // 2px
  default: '0.25rem', // 4px
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
 * Elevation system with consistent shadows
 */
export const shadows = {
  sm: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
  default: '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)',
  md: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
  lg: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
  xl: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)',
  '2xl': '0 25px 50px -12px rgb(0 0 0 / 0.25)',
  inner: 'inset 0 2px 4px 0 rgb(0 0 0 / 0.05)',
  none: '0 0 #0000',
  
  // Admin-specific shadows
  'card-hover': '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 10px 10px -5px rgb(0 0 0 / 0.04)',
  'dropdown': '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
  'modal': '0 25px 50px -12px rgb(0 0 0 / 0.25)',
  'button-hover': '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
} as const;

// ============================================================================
// Z-INDEX SYSTEM
// ============================================================================

/**
 * Consistent z-index scale for layering
 */
export const zIndex = {
  auto: 'auto',
  0: '0',
  10: '10',
  20: '20',
  30: '30',
  40: '40',
  50: '50',
  
  // Component-specific z-indices
  dropdown: '1000',
  sticky: '1020',
  fixed: '1030',
  modal: '1040',
  popover: '1050',
  tooltip: '1060',
  toast: '1070',
  loading: '1080',
  
  // Admin-specific layers
  sidebar: '100',
  header: '110',
  'admin-modal': '1100',
  'admin-overlay': '1090',
  'admin-toast': '1200',
} as const;

// ============================================================================
// ANIMATION SYSTEM
// ============================================================================

/**
 * Animation durations and easing functions
 */
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
  
  easing: {
    linear: 'linear',
    in: 'cubic-bezier(0.4, 0, 1, 1)',
    out: 'cubic-bezier(0, 0, 0.2, 1)',
    'in-out': 'cubic-bezier(0.4, 0, 0.2, 1)',
    
    // Custom easing for admin interfaces
    'bounce-in': 'cubic-bezier(0.34, 1.56, 0.64, 1)',
    'slide-out': 'cubic-bezier(0.55, 0, 0.1, 1)',
    'fade-in': 'cubic-bezier(0.16, 1, 0.3, 1)',
  },
} as const;

// ============================================================================
// BREAKPOINTS SYSTEM
// ============================================================================

/**
 * Responsive breakpoints for admin layouts
 */
export const breakpoints = {
  xs: '475px',
  sm: '640px',
  md: '768px',
  lg: '1024px',
  xl: '1280px',
  '2xl': '1536px',
  
  // Admin-specific breakpoints
  'sidebar-collapse': '1024px',
  'dashboard-grid': '768px',
  'table-scroll': '640px',
} as const;

// ============================================================================
// SEMANTIC TOKEN MAPPINGS
// ============================================================================

/**
 * Semantic color mappings for consistent theming
 */
export const semanticColors = {
  // Status mappings
  status: {
    success: colors.admin.active,
    warning: colors.admin.pending,
    error: colors.admin.suspended,
    info: colors.info[500],
  },
  
  // User status mappings
  user: {
    active: colors.admin.active,
    inactive: colors.admin.inactive,
    pending: colors.admin.pending,
    suspended: colors.admin.suspended,
    premium: colors.admin.premium,
  },
  
  // Permission mappings
  permission: {
    granted: colors.admin.granted,
    denied: colors.admin.denied,
    pending: colors.admin.pending,
    inherited: colors.admin.inherited,
  },
  
  // Billing mappings
  billing: {
    paid: colors.admin.paid,
    overdue: colors.admin.overdue,
    trial: colors.admin.trial,
    enterprise: colors.admin.enterprise,
  },
} as const;

// ============================================================================
// TYPE EXPORTS
// ============================================================================

export type Color = keyof typeof colors;
export type Spacing = keyof typeof spacing;
export type FontSize = keyof typeof typography.fontSize;
export type FontWeight = keyof typeof typography.fontWeight;
export type BorderRadius = keyof typeof borderRadius;
export type Shadow = keyof typeof shadows;
export type ZIndex = keyof typeof zIndex;
export type AnimationDuration = keyof typeof animation.duration;
export type AnimationEasing = keyof typeof animation.easing;
export type Breakpoint = keyof typeof breakpoints;