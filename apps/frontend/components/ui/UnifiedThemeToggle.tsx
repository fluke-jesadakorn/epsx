/**
 * FRONTEND THEME TOGGLE - Re-export from Shared
 * 
 * This file now re-exports the unified theme toggle from the shared directory.
 * All variants and types are preserved for backward compatibility.
 */

export {
  AnimatedThemeToggle, GradientThemeToggle,
  MinimalThemeToggle, OptimizedThemeToggle, ThemeToggle,
  ThemeToggleCSS, UnifiedThemeToggle, type ThemeToggleIconType,
  type ThemeToggleSize, type ThemeToggleVariant, type UnifiedThemeToggleProps
} from '@/shared/components/ui/UnifiedThemeToggle';

// Re-export default for convenience
export { UnifiedThemeToggle as default } from '@/shared/components/ui/UnifiedThemeToggle';
