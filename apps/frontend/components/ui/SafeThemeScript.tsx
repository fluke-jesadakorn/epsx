/**
 * SAFE THEME SCRIPT - Re-export from Shared
 * 
 * This file re-exports the unified SafeThemeScript from the shared directory.
 * All theme initialization logic is now centralized in @shared/components/ui/SafeThemeScript.
 */

export {
  SafeThemeScript,
  SafeThemeScriptWithNonce,
  themeUtils,
  type ValidTheme
} from '@/shared/components/ui/SafeThemeScript';

// Default export for backward compatibility
export { SafeThemeScript as default } from '@/shared/components/ui/SafeThemeScript';
