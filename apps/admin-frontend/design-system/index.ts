/**
 * Admin Frontend Design System
 *
 * Re-exports shared design system and adds admin-specific extensions.
 * This file provides backwards compatibility for existing imports.
 *
 * Usage:
 * ```typescript
 * import { adminButtonVariants, colors, spacing } from '@/design-system';
 * ```
 */

// ============================================================================
// RE-EXPORT SHARED DESIGN SYSTEM
// ============================================================================

// Re-export all shared tokens and components
export {
  animation, badgeVariants, borderRadius, breakpoints, buttonVariants,
  cardVariants,
  // Components
  cn,
  // Tokens
  colors,
  // Meta
  DESIGN_SYSTEM_VERSION,
  designSystemMeta, getActionButtonVariant,
  // Utilities
  getStatusBadgeVariant, inputVariants, loadingVariants, modalVariants, semanticColors, shadows, spacing, tableVariants, typography, zIndex,
  // Types
  type AnimationDuration,
  type AnimationEasing, type BadgeVariants, type BorderRadius,
  type Breakpoint, type ButtonVariants,
  type CardVariants, type Color,
  type FontSize,
  type FontWeight, type InputVariants, type LoadingVariants, type ModalVariants, type Shadow,
  type Spacing, type TableVariants, type ZIndex
} from '../../../shared/design-system';

// ============================================================================
// ADMIN-SPECIFIC EXTENSIONS
// ============================================================================

// Re-export PancakeSwap x Windows Phone theme (admin-only)
export { PancakePhoneTheme, type PancakePhoneThemeType } from './pancake-phone-theme';

// ============================================================================
// BACKWARDS COMPATIBILITY ALIASES
// ============================================================================

// Alias shared variants with 'admin' prefix for existing code
import {
  badgeVariants as _badgeVariants,
  buttonVariants as _buttonVariants,
  cardVariants as _cardVariants,
  inputVariants as _inputVariants,
  loadingVariants as _loadingVariants,
  modalVariants as _modalVariants,
  tableVariants as _tableVariants,
  type BadgeVariants,
  type ButtonVariants,
  type CardVariants,
  type InputVariants,
  type LoadingVariants,
  type ModalVariants,
  type TableVariants,
} from '../../../shared/design-system';

// Admin-prefixed aliases (for backwards compatibility)
export const adminButtonVariants = _buttonVariants;
export const adminCardVariants = _cardVariants;
export const adminBadgeVariants = _badgeVariants;
export const adminInputVariants = _inputVariants;
export const adminTableVariants = _tableVariants;
export const adminModalVariants = _modalVariants;
export const adminLoadingVariants = _loadingVariants;

// Admin-prefixed type aliases
export type AdminButtonVariants = ButtonVariants;
export type AdminCardVariants = CardVariants;
export type AdminBadgeVariants = BadgeVariants;
export type AdminInputVariants = InputVariants;
export type AdminTableVariants = TableVariants;
export type AdminModalVariants = ModalVariants;
export type AdminLoadingVariants = LoadingVariants;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

import {
  animation as _animation,
  borderRadius as _borderRadius,
  breakpoints as _breakpoints,
  colors as _colors,
  semanticColors as _semanticColors,
  shadows as _shadows,
  spacing as _spacing,
  typography as _typography,
  zIndex as _zIndex,
} from '../../../shared/design-system';

/**
 * Get design system tokens (non-hook version)
 */
export function getDesignSystemTokens() {
  return {
    colors: _colors,
    spacing: _spacing,
    typography: _typography,
    borderRadius: _borderRadius,
    shadows: _shadows,
    zIndex: _zIndex,
    animation: _animation,
    breakpoints: _breakpoints,
    semanticColors: _semanticColors,
  };
}

/**
 * Check if a breakpoint matches (non-hook version)
 */
export function checkBreakpoint(breakpoint: keyof typeof _breakpoints): boolean {
  if (typeof window === 'undefined') {
    return false;
  }
  const query = `(min-width: ${_breakpoints[breakpoint]})`;
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
        background: _colors.background.light,
        foreground: _colors.foreground.light,
        card: _colors.card.light,
      },
    };
  }

  const isDark =
    document.documentElement.classList.contains('dark') ||
    window.matchMedia('(prefers-color-scheme: dark)').matches;

  return {
    isDark,
    colors: {
      background: isDark ? _colors.background.dark : _colors.background.light,
      foreground: isDark ? _colors.foreground.dark : _colors.foreground.light,
      card: isDark ? _colors.card.dark : _colors.card.light,
    },
  };
}

// ============================================================================
// TYPE GUARDS
// ============================================================================

/**
 * Type guard for valid button variants
 */
export function isValidButtonVariant(
  variant: string
): variant is
  | 'primary'
  | 'secondary'
  | 'success'
  | 'destructive'
  | 'warning'
  | 'outline'
  | 'ghost'
  | 'link'
  | 'insight' {
  const validVariants = [
    'primary',
    'secondary',
    'success',
    'destructive',
    'warning',
    'outline',
    'ghost',
    'link',
    'insight',
  ];
  return validVariants.includes(variant);
}

/**
 * Type guard for valid card variants
 */
export function isValidCardVariant(
  variant: string
): variant is
  | 'default'
  | 'insight'
  | 'pancake'
  | 'user'
  | 'permission'
  | 'billing'
  | 'analytics'
  | 'warning'
  | 'error' {
  const validVariants = [
    'default',
    'insight',
    'pancake',
    'user',
    'permission',
    'billing',
    'analytics',
    'warning',
    'error',
  ];
  return validVariants.includes(variant);
}

/**
 * Type guard for valid badge variants
 */
export function isValidBadgeVariant(
  variant: string
): variant is
  | 'active'
  | 'inactive'
  | 'pending'
  | 'suspended'
  | 'premium'
  | 'granted'
  | 'denied'
  | 'inherited'
  | 'paid'
  | 'overdue'
  | 'trial'
  | 'enterprise'
  | 'success'
  | 'warning'
  | 'error'
  | 'info'
  | 'default' {
  const validVariants = [
    'active',
    'inactive',
    'pending',
    'suspended',
    'premium',
    'granted',
    'denied',
    'inherited',
    'paid',
    'overdue',
    'trial',
    'enterprise',
    'success',
    'warning',
    'error',
    'info',
    'default',
  ];
  return validVariants.includes(variant);
}

// ============================================================================
// DEFAULT EXPORT
// ============================================================================

import designSystem from '../../../shared/design-system';

export default designSystem;