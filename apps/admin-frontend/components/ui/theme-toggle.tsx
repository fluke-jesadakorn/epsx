/**
 * ADMIN THEME TOGGLE - Re-export from Shared
 * 
 * This file now re-exports the unified theme toggle from the shared directory.
 * Uses admin-specific defaults for backward compatibility.
 */

// Re-export all variants from shared
export {
  AdminThemeToggle, AnimatedThemeToggle, GradientThemeToggle,
  MinimalThemeToggle, OptimizedThemeToggle, SimpleThemeToggle, ThemeToggle,
  ThemeToggleCSS, UnifiedThemeToggle, type ThemeToggleIconType,
  type ThemeToggleSize, type ThemeToggleVariant, type UnifiedThemeToggleProps
} from '@/shared/components/ui/unified-theme-toggle';

// Legacy default export that matches admin's original behavior
export { AdminThemeToggle as default } from '@/shared/components/ui/unified-theme-toggle';
