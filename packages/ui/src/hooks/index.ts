// Core theme hooks (specific exports to avoid conflicts)
export { useTheme as useThemeHook } from './useTheme.js';
export { useUnifiedTheme } from './useUnifiedTheme.js';
// TODO: Re-enable after fixing zustand dependency
// export { useThemeStore } from './useThemeStore.js';

// Legacy compatibility - re-export theme functionality with provider prefix
export { 
  useTheme, 
  ThemeProvider, 
  withTheme, 
  ThemeVariantSelector, 
  DarkModeToggle 
} from '../providers/theme-provider.js';