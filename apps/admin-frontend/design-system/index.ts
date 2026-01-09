/**
 * Admin Frontend Design System
 * 
 * Barrel export file for the complete design system.
 * Provides easy imports and a unified API for the design system.
 * 
 * Usage:
 * ```typescript
 * import { adminButtonVariants, colors, spacing } from '@/design-system';
 * ```
 */

// ============================================================================
// DESIGN TOKENS
// ============================================================================

// Import tokens for re-export
import {
  adminBadgeVariants as _adminBadgeVariants,
  adminButtonVariants as _adminButtonVariants,
  adminCardVariants as _adminCardVariants,
  adminInputVariants as _adminInputVariants,
  adminLoadingVariants as _adminLoadingVariants,
  adminModalVariants as _adminModalVariants,
  adminTableVariants as _adminTableVariants,
  cn as _cn,
  getActionButtonVariant as _getActionButtonVariant,
  getStatusBadgeVariant as _getStatusBadgeVariant,
} from './components';
import {
  animation as _animation,
  borderRadius as _borderRadius,
  breakpoints as _breakpoints,
  colors as _colors,
  semanticColors as _semanticColors,
  shadows as _shadows,
  spacing as _spacing,
  typography as _typography,
  zIndex as _zIndex
} from './tokens';

// Import variants for internal use

export {
  type AnimationDuration,
  type AnimationEasing, type BorderRadius, type Breakpoint, type Color, type FontSize,
  type FontWeight, type Shadow, type Spacing, type ZIndex
} from './tokens';

// Re-export tokens safely
export const colors = _colors;
export const spacing = _spacing;
export const typography = _typography;
export const borderRadius = _borderRadius;
export const shadows = _shadows;
export const zIndex = _zIndex;
export const animation = _animation;
export const breakpoints = _breakpoints;
export const semanticColors = _semanticColors;

// ============================================================================
// COMPONENT VARIANTS (CVA)
// ============================================================================

export {
  adminBadgeVariants, adminButtonVariants,
  adminCardVariants, adminInputVariants, adminLoadingVariants, adminModalVariants, adminTableVariants, cn, getActionButtonVariant, getStatusBadgeVariant, type AdminBadgeVariants, type AdminButtonVariants,
  type AdminCardVariants, type AdminInputVariants, type AdminLoadingVariants, type AdminModalVariants, type AdminTableVariants
} from './components';

// Animation system removed for Zero Animation Policy compliance

// ============================================================================
// ANIMATIONS (REMOVED)
// ============================================================================

// Animation system removed for Zero Animation Policy compliance
// All motion and transitions have been replaced with instant state changes

// ============================================================================
// MIGRATION UTILITIES (Removed - migration file not found)
// ============================================================================

// Migration utilities have been removed as the migration file does not exist

// ============================================================================
// DESIGN SYSTEM CONFIGURATION
// ============================================================================

/**
 * Design system version for tracking
 */
export const DESIGN_SYSTEM_VERSION = '1.0.0';

/**
 * Design system metadata
 */
export const designSystemMeta = {
  name: 'Admin Frontend Design System',
  version: DESIGN_SYSTEM_VERSION,
  description: 'Type-safe design system for EPSX admin interfaces',
  author: 'EPSX Team',

  // Component counts for tracking (calculated safely)
  components: {
    buttons: 8, // primary, secondary, success, destructive, warning, outline, ghost, link
    cards: 8,   // default, pancake, user, permission, billing, analytics, warning, error
    badges: 15, // active, inactive, pending, suspended, premium, granted, denied, inherited, paid, overdue, trial, enterprise, success, warning, error, info, default
  },

  // Token counts (calculated safely)
  tokens: {
    colors: Object.keys(_colors || {}).length,
    spacing: Object.keys(_spacing || {}).length,
    typography: Object.keys(_typography?.fontSize || {}).length,
    animations: Object.keys(_animation || {}).length,
  },

  // Migration support (removed - migration file not found)
  migration: {
    legacyClasses: 0,
    cssReplacements: 0,
  },
} as const;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Get design system information
 */
export function getDesignSystemInfo() {
  return designSystemMeta;
}

/**
 * Check if design system is properly loaded
 */
export function validateDesignSystem(): {
  isValid: boolean;
  errors: string[];
  warnings: string[];
} {
  const errors: string[] = [];
  const warnings: string[] = [];

  // Check if required tokens are available
  if (!colors?.primary) {
    errors.push('Primary color tokens not found');
  }

  if (!spacing?.['4']) {
    errors.push('Spacing tokens not found');
  }

  if (!typography?.fontSize) {
    errors.push('Typography tokens not found');
  }

  // Check if component variants are available
  try {
    if (typeof _adminButtonVariants === 'function') {
      _adminButtonVariants({ variant: 'primary' });
    } else {
      errors.push('Button variants not properly configured');
    }
  } catch (_error) {
    errors.push('Button variants not properly configured');
  }

  try {
    if (typeof _adminCardVariants === 'function') {
      _adminCardVariants({ variant: 'default' });
    } else {
      errors.push('Card variants not properly configured');
    }
  } catch (_error) {
    errors.push('Card variants not properly configured');
  }

  // Performance warnings
  if (typeof window !== 'undefined') {
    const hasReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    if (hasReducedMotion) {
      warnings.push('User prefers reduced motion - animations will be disabled');
    }
  }

  return {
    isValid: errors.length === 0,
    errors,
    warnings,
  };
}

/**
 * Development helper to log design system status
 */
export function debugDesignSystem(): void {
  if (process.env.NODE_ENV !== 'development') { return; }

  const info = getDesignSystemInfo();
  const validation = validateDesignSystem();

  console.group('🎨 Admin Frontend Design System');

  if (validation.errors.length > 0) {
     
    console.error('❌ Errors:', validation.errors);
  }

  if (validation.warnings.length > 0) {
     
    console.warn('⚠️ Warnings:', validation.warnings);
  }

  if (validation.isValid) {
    // Design system is valid - no action needed
  }

  console.groupEnd();
}

// ============================================================================
// UTILITY FUNCTIONS FOR COMPONENTS
// ============================================================================

/**
 * Get design system tokens (non-hook version)
 */
export function getDesignSystemTokens() {
  return {
    colors,
    spacing,
    typography,
    borderRadius,
    shadows,
    zIndex,
    animation,
    breakpoints,
    semanticColors,
  };
}

/**
 * Check if a breakpoint matches (non-hook version)
 * @param breakpoint
 */
export function checkBreakpoint(breakpoint: keyof typeof breakpoints): boolean {
  if (typeof window === 'undefined') { return false; }
  const query = `(min-width: ${breakpoints[breakpoint]})`;
  return window.matchMedia(query).matches;
}

/**
 * Get theme-aware colors (non-hook version)
 */
export function getThemeColors() {
  if (typeof window === 'undefined') {
    return {
      isDark: false,
      colors: {
        background: colors.background.light,
        foreground: colors.foreground.light,
        card: colors.card.light,
      },
    };
  }

  const isDark = document.documentElement.classList.contains('dark') ||
    window.matchMedia('(prefers-color-scheme: dark)').matches;

  return {
    isDark,
    colors: {
      background: isDark ? colors.background.dark : colors.background.light,
      foreground: isDark ? colors.foreground.dark : colors.foreground.light,
      card: isDark ? colors.card.dark : colors.card.light,
    },
  };
}

// ============================================================================
// TYPE GUARDS
// ============================================================================

/**
 * Type guard for valid button variants
 * @param variant
 */
export function isValidButtonVariant(variant: string): variant is 'primary' | 'secondary' | 'success' | 'destructive' | 'warning' | 'outline' | 'ghost' | 'link' {
  const validVariants = ['primary', 'secondary', 'success', 'destructive', 'warning', 'outline', 'ghost', 'link'];
  return validVariants.includes(variant);
}

/**
 * Type guard for valid card variants
 * @param variant
 */
export function isValidCardVariant(variant: string): variant is 'default' | 'pancake' | 'user' | 'permission' | 'billing' | 'analytics' | 'warning' | 'error' {
  const validVariants = ['default', 'pancake', 'user', 'permission', 'billing', 'analytics', 'warning', 'error'];
  return validVariants.includes(variant);
}

/**
 * Type guard for valid badge variants
 * @param variant
 */
export function isValidBadgeVariant(variant: string): variant is 'active' | 'inactive' | 'pending' | 'suspended' | 'premium' | 'granted' | 'denied' | 'inherited' | 'paid' | 'overdue' | 'trial' | 'enterprise' | 'success' | 'warning' | 'error' | 'info' | 'default' {
  const validVariants = [
    'active', 'inactive', 'pending', 'suspended', 'premium',
    'granted', 'denied', 'inherited',
    'paid', 'overdue', 'trial', 'enterprise',
    'success', 'warning', 'error', 'info', 'default'
  ];
  return validVariants.includes(variant);
}

// Note: React hooks have been moved to separate hook files to avoid dependencies
// Components that need React functionality should import React separately

// ============================================================================
// DEFAULT EXPORT
// ============================================================================

/**
 * Default export with all design system utilities
 */
const designSystem = {
  // Tokens
  tokens: {
    colors,
    spacing,
    typography,
    borderRadius,
    shadows,
    zIndex,
    animation,
    breakpoints,
    semanticColors,
  },

  // Components
  components: {
    adminButtonVariants: _adminButtonVariants,
    adminCardVariants: _adminCardVariants,
    adminBadgeVariants: _adminBadgeVariants,
    adminTableVariants: _adminTableVariants,
    adminInputVariants: _adminInputVariants,
    adminModalVariants: _adminModalVariants,
    adminLoadingVariants: _adminLoadingVariants,
  },

  // Animations (removed for Zero Animation Policy)
  animations: {
    // Animation system removed for performance and accessibility
  },

  // Migration (removed - migration file not found)

  // Utils
  utils: {
    cn: _cn,
    getStatusBadgeVariant: _getStatusBadgeVariant,
    getActionButtonVariant: _getActionButtonVariant,
    // Animation utilities removed for Zero Animation Policy
  },

  // Meta
  meta: designSystemMeta,
  validateDesignSystem,
  debugDesignSystem,
} as const;

export default designSystem;